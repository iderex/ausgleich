//! What the correlation coefficient record refuses, and what it accepts.
//!
//! #31 asks for four refusals, each with a one-change fixture proving it bites.
//! Two of the four are properties of a single record and are reached by one
//! change to the file on disk. The other two are properties of a set, so the
//! one change is made to a valid pair of records rather than to a single one:
//! there is no such thing as a lone record that names an unknown datum, because
//! what makes an identifier unknown is the set it is assembled against.
//!
//! The changes are made here rather than by keeping a near-copy of the fixture
//! on disk for each rule, for the reason the provenance suite beside this one
//! gives: a directory of near-copies drifts apart from the valid one, and then a
//! rule is proved against a file that stopped being a neighbour.

use std::collections::BTreeSet;

use ausgleich_data::coefficient::{self, Census, Coefficient, Refusal};

const FIXTURE: &str = include_str!("fixtures/correlation-coefficient.toml");

/// `text` with the one line for `path` taken out, or replaced.
///
/// `path` is `section.key`, or a bare key for a field at the top of the file.
/// The walk asserts that it changed exactly one line, so a path that stops
/// matching the fixture fails here rather than quietly testing nothing.
fn edit(text: &str, path: &str, replacement: Option<&str>) -> String {
    let (section, key) = match path.split_once('.') {
        Some(split) => split,
        None => ("", path),
    };
    let mut kept: Vec<String> = Vec::new();
    let mut current = "";
    let mut hits = 0usize;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current = trimmed.trim_start_matches('[').trim_end_matches(']');
        }
        let names_the_key = trimmed
            .split_once('=')
            .is_some_and(|(name, _)| name.trim() == key);
        if current == section && names_the_key {
            hits += 1;
            if let Some(value) = replacement {
                kept.push(format!("{key} = {value}"));
            }
        } else {
            kept.push(line.to_owned());
        }
    }
    assert_eq!(hits, 1, "expected exactly one {path} line in the fixture");
    kept.join("\n")
}

/// The fixture without the field at `path`.
fn without(path: &str) -> String {
    edit(FIXTURE, path, None)
}

/// The fixture with the value at `path` written as `value`.
fn instead(path: &str, value: &str) -> String {
    edit(FIXTURE, path, Some(value))
}

/// The fixture with its two identifiers and its coefficient replaced.
///
/// This is what makes a set: three changed lines from one file, so every record
/// in a set-level test is still the fixture and differs from it only where the
/// test is about.
fn record(first: &str, second: &str, value: &str) -> Coefficient {
    let written = format!("[\"{first}\", \"{second}\"]");
    let text = edit(&instead("between", &written), "value", Some(value));
    accepted(&text)
}

/// The refusal a document earns, or a failure saying it earned none.
fn refusal(document: &str) -> Refusal {
    match coefficient::read(document) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the record was accepted"),
    }
}

/// The record a document parses to, or a failure carrying the refusal.
fn accepted(document: &str) -> Coefficient {
    match coefficient::read(document) {
        Ok(found) => found,
        Err(refused) => panic!("expected the record to be accepted: {refused}"),
    }
}

/// The identifiers of a set of four data, which is six pairs.
fn four_data() -> BTreeSet<String> {
    ["one", "two", "three", "four"]
        .iter()
        .map(|name| format!("fixture-datum-{name}"))
        .collect()
}

/// The refusal a set earns, or a failure saying it earned none.
fn refused_set(records: &[Coefficient], data: &BTreeSet<String>) -> Refusal {
    match coefficient::assemble(records, data) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the set was assembled"),
    }
}

/// The matrix and the census a set assembles to.
fn assembled(
    records: &[Coefficient],
    data: &BTreeSet<String>,
) -> (coefficient::Correlations, Census) {
    match coefficient::assemble(records, data) {
        Ok(found) => found,
        Err(refused) => panic!("expected the set to assemble: {refused}"),
    }
}

#[test]
fn the_fixture_reads_and_carries_its_provenance_block() {
    let record = accepted(FIXTURE);
    assert_eq!(
        record.between(),
        &[
            "fixture-datum-one".to_owned(),
            "fixture-datum-two".to_owned()
        ]
    );
    assert!(record.value() > 0.0);
    // The block is read by its own reader from the same bytes, so a record
    // carries every field a datum does rather than a summary of them.
    assert_eq!(record.provenance().read_from.locator, "Table VII");
    assert_eq!(record.provenance().method, "penning-trap-cyclotron-ratio");
    assert!(!record.derived_by_the_publication());
    assert!(format!("{record:?}").contains("fixture-datum-two"));
}

#[test]
fn a_coefficient_the_publication_derived_says_so_from_the_block_and_not_from_a_second_field() {
    let derived = accepted(&instead(
        "measurement.origin",
        "\"derived-by-the-publication\"",
    ));
    assert!(derived.derived_by_the_publication());
}

// The first of #31's four refusals.
#[test]
fn a_coefficient_outside_the_closed_interval_is_refused_by_its_value() {
    for written in ["1.5", "-1.5", "nan"] {
        match refusal(&instead("value", written)) {
            Refusal::OutsideTheInterval { found } => {
                // Written rather than compared, because one of the three is not
                // equal to itself and an assertion on the value would pass for
                // the wrong reason.
                assert!(!(-1.0..=1.0).contains(&found), "{written} was accepted");
            }
            other => panic!("{written}: expected an interval refusal, got {other}"),
        }
    }
}

#[test]
fn both_ends_of_the_interval_are_inside_it_and_read_as_written() {
    // The interval is closed, and a coefficient of exactly one is written `1`
    // as readily as `1.0`. Refusing the integer spelling would be a refusal
    // about notation.
    assert_eq!(accepted(&instead("value", "1")).value(), 1.0);
    assert_eq!(accepted(&instead("value", "-1.0")).value(), -1.0);
}

// The second of #31's four refusals.
#[test]
fn a_record_naming_one_datum_twice_is_refused_by_that_identifier() {
    let written = "[\"fixture-datum-one\", \"fixture-datum-one\"]";
    match refusal(&instead("between", written)) {
        Refusal::OneDatumTwice { named } => assert_eq!(named, "fixture-datum-one"),
        other => panic!("expected a self-naming refusal, got {other}"),
    }
}

// The third of #31's four refusals. The one change is to the second record of a
// valid pair, and it is the identifier rather than the file's shape.
#[test]
fn a_record_naming_a_datum_that_does_not_exist_is_refused_by_that_identifier() {
    let data = four_data();
    let valid = [
        record("fixture-datum-one", "fixture-datum-two", "0.24"),
        record("fixture-datum-three", "fixture-datum-four", "-0.1"),
    ];
    let (_, census) = assembled(&valid, &data);
    assert_eq!(census.asserted, 2);

    let changed = [
        record("fixture-datum-one", "fixture-datum-two", "0.24"),
        record("fixture-datum-three", "fixture-datum-five", "-0.1"),
    ];
    match refused_set(&changed, &data) {
        Refusal::UnknownDatum { named } => assert_eq!(named, "fixture-datum-five"),
        other => panic!("expected an unknown-datum refusal, got {other}"),
    }
}

// The fourth of #31's four refusals, proved in both orders, because a
// normalisation that only worked one way would pass a test written one way.
#[test]
fn a_pair_asserted_twice_in_either_order_is_refused_by_the_pair() {
    let data = four_data();
    for (first, second) in [
        ("fixture-datum-one", "fixture-datum-two"),
        ("fixture-datum-two", "fixture-datum-one"),
    ] {
        let records = [
            record("fixture-datum-one", "fixture-datum-two", "0.24"),
            record(first, second, "-0.1"),
        ];
        match refused_set(&records, &data) {
            Refusal::PairStatedTwice {
                first: left,
                second: right,
            } => {
                assert_eq!(left, "fixture-datum-one");
                assert_eq!(right, "fixture-datum-two");
            }
            other => panic!("{first} and {second}: expected a duplicate refusal, got {other}"),
        }
    }
}

#[test]
fn absence_is_zero_and_a_datum_with_itself_is_one() {
    let data = four_data();
    let records = [record("fixture-datum-two", "fixture-datum-one", "0.24")];
    let (matrix, _) = assembled(&records, &data);

    // Asserted once, readable in both orders, because the pair is the fact and
    // the order it was written in is not.
    assert_eq!(
        matrix.between("fixture-datum-one", "fixture-datum-two"),
        0.24
    );
    assert_eq!(
        matrix.between("fixture-datum-two", "fixture-datum-one"),
        0.24
    );
    // Never asserted. Zero because no record says otherwise, which is a
    // different statement from a stored zero and is why this is not a dense
    // file.
    assert_eq!(
        matrix.between("fixture-datum-three", "fixture-datum-four"),
        0.0
    );
    // The diagonal is a definition rather than an assertion, which is why a
    // record stating it is refused above.
    assert_eq!(
        matrix.between("fixture-datum-one", "fixture-datum-one"),
        1.0
    );
    assert_eq!(matrix.asserted(), 1);
    assert!(format!("{matrix:?}").contains("fixture-datum-one"));
}

#[test]
fn the_census_counts_what_was_asserted_what_was_left_at_zero_and_what_was_named_by_nothing() {
    let data = four_data();
    let records = [
        record("fixture-datum-one", "fixture-datum-two", "0.24"),
        record("fixture-datum-two", "fixture-datum-three", "-0.1"),
    ];
    let (_, census) = assembled(&records, &data);
    assert_eq!(census.data, 4);
    assert_eq!(census.asserted, 2);
    assert_eq!(census.derived, 0);
    // Four data are six pairs, and two of the six were asserted.
    assert_eq!(census.pairs_at_zero, 4);
    assert_eq!(census.uncorrelated, vec!["fixture-datum-four".to_owned()]);
    assert!(format!("{census:?}").contains("fixture-datum-four"));
}

#[test]
fn the_census_reads_as_it_will_print() {
    // The text rather than the fields, because the text is what a reader sees
    // and a field nobody prints tells nobody anything. Printed as well as
    // compared, so a run with --nocapture shows the thing this asserts.
    let data = four_data();
    let derived = edit(
        &instead("measurement.origin", "\"derived-by-the-publication\""),
        "between",
        Some("[\"fixture-datum-two\", \"fixture-datum-three\"]"),
    );
    let records = [
        record("fixture-datum-one", "fixture-datum-two", "0.24"),
        accepted(&derived),
    ];
    let (_, census) = assembled(&records, &data);
    let printed = census.to_string();
    println!("{printed}");
    assert_eq!(
        printed,
        "\
correlation coefficients over 4 data
  asserted                        2
  of those derived, not stated    1
  pairs left at zero              4
  data in no asserted coefficient 1
    fixture-datum-four

A datum in no asserted coefficient is either independent or a gap in
the digitisation, and nothing here knows which."
    );
}

#[test]
fn a_set_naming_every_datum_leaves_the_last_line_of_the_census_empty() {
    // The other side of the line above. A census that only ever printed a
    // non-empty list would not prove the list is derived from the set.
    let data: BTreeSet<String> = ["fixture-datum-one", "fixture-datum-two"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let records = [record("fixture-datum-one", "fixture-datum-two", "0.24")];
    let (_, census) = assembled(&records, &data);
    assert_eq!(census.pairs_at_zero, 0);
    assert!(census.uncorrelated.is_empty());
    assert!(
        census
            .to_string()
            .contains("data in no asserted coefficient 0")
    );
}

#[test]
fn an_empty_set_assembles_to_an_empty_matrix_rather_than_to_a_refusal() {
    // An input set still being digitised is a legitimate thing to run against,
    // and the arithmetic over its pair count has to hold at zero data.
    let (matrix, census) = assembled(&[], &BTreeSet::new());
    assert_eq!(matrix.asserted(), 0);
    assert_eq!(census.data, 0);
    assert_eq!(census.pairs_at_zero, 0);
}

#[test]
fn a_record_missing_either_of_its_own_fields_is_refused_by_that_field() {
    for path in ["between", "value"] {
        match refusal(&without(path)) {
            Refusal::Missing { field } => assert_eq!(field, path),
            other => panic!("{path}: expected a missing-field refusal, got {other}"),
        }
    }
}

#[test]
fn a_field_of_the_wrong_kind_is_refused_and_the_message_says_what_was_expected() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "between",
            "\"fixture-datum-one\"",
            "an array of two identifiers",
        ),
        (
            "between",
            "[\"fixture-datum-one\"]",
            "an array of two identifiers",
        ),
        (
            "between",
            "[\"fixture-datum-one\", \"fixture-datum-two\", \"fixture-datum-three\"]",
            "an array of two identifiers",
        ),
        ("between", "[1, 2]", "an array of two identifiers"),
        ("value", "\"a quarter\"", "a number"),
    ];
    for (path, written, expected) in cases {
        let refused = refusal(&instead(path, written));
        match &refused {
            Refusal::WrongKind {
                field,
                expected: said,
            } => {
                assert_eq!(field, path);
                assert_eq!(said, expected);
            }
            other => panic!("{path} as {written}: expected a wrong-kind refusal, got {other}"),
        }
        assert!(refused.to_string().contains(expected));
    }
}

#[test]
fn bytes_that_are_not_a_document_are_refused_as_unreadable() {
    match refusal("this is not a document = = =") {
        Refusal::Unreadable(detail) => assert!(!detail.is_empty()),
        other => panic!("expected an unreadable refusal, got {other}"),
    }
}

#[test]
fn a_record_whose_provenance_block_is_refused_carries_that_refusal_rather_than_replacing_it() {
    // The block's own reader decides what a block is, and the record says which
    // of its parts was refused instead of flattening it to "malformed".
    let refused = refusal(&instead("measurement.method", "\"a method nobody agreed\""));
    match &refused {
        Refusal::Provenance(inner) => {
            assert!(inner.to_string().contains("a method nobody agreed"));
        }
        other => panic!("expected a provenance refusal, got {other}"),
    }
    let carried: &dyn std::error::Error = &refused;
    assert!(carried.source().is_some());
    assert!(refused.to_string().contains("provenance block was refused"));
}

#[test]
fn every_refusal_prints_debugs_and_says_whether_it_stands_on_another() {
    // Each is produced rather than constructed, so a variant that stops being
    // reachable fails here. The set is the whole enum.
    let data = four_data();
    let twice = [
        record("fixture-datum-one", "fixture-datum-two", "0.24"),
        record("fixture-datum-two", "fixture-datum-one", "0.24"),
    ];
    let unknown = [record("fixture-datum-one", "fixture-datum-two", "0.24")];
    let refusals = [
        refusal("this is not a document = = ="),
        refusal(&without("between")),
        refusal(&instead("value", "\"a quarter\"")),
        refusal(&instead("measurement.method", "\"a method nobody agreed\"")),
        refusal(&instead("value", "1.5")),
        refusal(&instead(
            "between",
            "[\"fixture-datum-one\", \"fixture-datum-one\"]",
        )),
        refused_set(&unknown, &BTreeSet::new()),
        refused_set(&twice, &data),
    ];
    for refused in &refusals {
        assert!(!refused.to_string().is_empty());
        assert!(!format!("{refused:?}").is_empty());
        let carried: &dyn std::error::Error = refused;
        let stands_on_another = matches!(refused, Refusal::Provenance(_));
        assert_eq!(carried.source().is_some(), stands_on_another);
    }
}
