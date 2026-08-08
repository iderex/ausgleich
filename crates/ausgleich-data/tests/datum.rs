//! What the input datum record refuses, and what it accepts.
//!
//! #30 asks for five refusals, each with a one-change fixture proving it bites,
//! and #4 asks for one fixture per required field rather than one for the class
//! of a missing field. The second count is the one that bites: a single missing
//! field fixture leaves a required field the code forgot to require green, so
//! the loop below walks every field and the named tests beside it are the five.
//!
//! Four of the refusals are properties of a single record and are reached by
//! one change to the file on disk. The fifth, an identifier two files carry, is
//! a property of a set, so the one change is made to a valid pair of records
//! rather than to a single one.
//!
//! The changes are made here rather than by keeping a near-copy of the fixture
//! on disk for each rule, for the reason the two suites beside this one give: a
//! directory of near-copies drifts apart from the valid one, and then a rule is
//! proved against a file that stopped being a neighbour.

use ausgleich_data::datum::{self, Datum, Refusal, Refused, UncertaintyForm};
use ausgleich_data::provenance;

const FIXTURE: &str = include_str!("fixtures/fixture-datum-one.toml");

/// The name the fixture's own file has, which is the name its identifier
/// requires. The rule tying the two together is proved against the real name.
const NAME: &str = "fixture-datum-one.toml";

/// Every field the record requires of itself, written as the refusal names it.
///
/// The provenance block's own fields are not here. They are required, and the
/// suite next to this one proves each of them, so restating the list would put
/// the same obligation in two places and let one of them drift.
const FIELDS: &[&str] = &[
    "datum.identifier",
    "datum.quantity",
    "datum.unit",
    "datum.value",
    "datum.uncertainty",
    "datum.uncertainty_form",
    "datum.enters_the_adjustment",
];

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

/// The fixture with one more line inside the `[name]` table.
///
/// The optional field is absent from the fixture, because a live datum has
/// nothing that replaced it. Proving what happens when it is present therefore
/// means adding the line rather than changing one.
fn also(name: &str, line: &str) -> String {
    let header = format!("[{name}]");
    let mut kept: Vec<String> = Vec::new();
    let mut found = false;
    for existing in FIXTURE.lines() {
        kept.push(existing.to_owned());
        if existing.trim() == header {
            kept.push(line.to_owned());
            found = true;
        }
    }
    assert!(found, "expected a [{name}] table in the fixture");
    kept.join("\n")
}

/// The fixture without the whole `[name]` table.
fn without_table(name: &str) -> String {
    let header = format!("[{name}]");
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    let mut found = false;
    for line in FIXTURE.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            inside = trimmed == header;
            found = found || inside;
        }
        if !inside {
            kept.push(line);
        }
    }
    assert!(found, "expected a [{name}] table in the fixture");
    kept.join("\n")
}

/// The refusal a document earns, or a failure saying it earned none.
fn refused(file: &str, document: &str) -> Refused {
    match datum::read(file, document) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the record was accepted"),
    }
}

/// The refusal a document read under the fixture's own name earns.
fn refusal(document: &str) -> Refusal {
    refused(NAME, document).refusal
}

/// The record a document parses to, or a failure carrying the refusal.
fn accepted(file: &str, document: &str) -> Datum {
    match datum::read(file, document) {
        Ok(found) => found,
        Err(refused) => panic!("expected the record to be accepted: {refused}"),
    }
}

/// The fixture under another identifier, read under the file name that
/// identifier requires.
///
/// `changes` replaces a line that is already there and `extra` adds one that is
/// not, which is the difference between a required field and the optional one.
fn another(identifier: &str, changes: &[(&str, &str)], extra: &[&str]) -> (String, Datum) {
    let mut text = instead("datum.identifier", &format!("\"{identifier}\""));
    for (path, replacement) in changes {
        text = edit(&text, path, Some(replacement));
    }
    for line in extra {
        let mut kept: Vec<String> = Vec::new();
        for existing in text.lines() {
            kept.push(existing.to_owned());
            if existing.trim() == "[datum]" {
                kept.push((*line).to_owned());
            }
        }
        text = kept.join("\n");
    }
    let file = format!("{identifier}.toml");
    let record = accepted(&file, &text);
    (file, record)
}

#[test]
fn the_fixture_reads_and_carries_its_provenance_block() {
    let record = accepted(NAME, FIXTURE);
    assert_eq!(record.identifier(), "fixture-datum-one");
    assert_eq!(record.quantity(), "fixture-quantity-one");
    assert_eq!(record.unit(), "hertz");
    assert!(record.value() > 0.0);
    assert!(record.uncertainty() > 0.0);
    assert!(matches!(
        record.uncertainty_form(),
        UncertaintyForm::Absolute
    ));
    assert!(record.enters_the_adjustment());
    assert_eq!(record.superseded_by(), None);
    assert_eq!(record.file_name(), NAME);
    // The block is read by its own reader from the same bytes, so a datum
    // carries every field a coefficient does rather than a summary of them.
    assert_eq!(record.provenance().read_from.locator, "Table II");
    assert_eq!(record.provenance().method, "kibble-balance");
    assert_eq!(record.provenance().version, provenance::BLOCK_VERSION);
    assert!(format!("{record:?}").contains("fixture-quantity-one"));
}

// #4's count rather than #30's. One fixture per required field, each differing
// from the valid file by the one line it is about.
#[test]
fn a_record_missing_any_required_field_is_refused_by_that_field() {
    for path in FIELDS {
        match refusal(&without(path)) {
            Refusal::Missing { field } => assert_eq!(&field, path),
            other => panic!("{path}: expected a missing-field refusal, got {other}"),
        }
    }
}

// The first of #30's five refusals, named on its own as well as reached inside
// the loop above, because a rule proved only inside a loop is a rule somebody
// deletes with the loop.
#[test]
fn a_record_with_no_value_is_refused_for_the_value() {
    match refusal(&without("datum.value")) {
        Refusal::Missing { field } => assert_eq!(field, "datum.value"),
        other => panic!("expected a missing-value refusal, got {other}"),
    }
}

#[test]
fn a_record_with_no_table_of_its_own_is_refused_by_that_table() {
    match refusal(&without_table("datum")) {
        Refusal::Missing { field } => assert_eq!(field, "datum"),
        other => panic!("expected a missing-table refusal, got {other}"),
    }
}

#[test]
fn a_table_written_as_a_value_is_refused_as_a_table() {
    // The mistake somebody actually makes. The document stops at that point, so
    // nothing after it needs to be present.
    let document = format!(
        "version = {}\ndatum = \"fixture-datum-one\"\n",
        provenance::BLOCK_VERSION
    );
    match refusal(&document) {
        Refusal::WrongKind { field, expected } => {
            assert_eq!(field, "datum");
            assert_eq!(expected, "a table");
        }
        other => panic!("expected a wrong-kind refusal for the table, got {other}"),
    }
}

// The second of #30's five. The format refuses digits it cannot read before any
// field is looked at, so what this reader adds is the two cases the format
// accepts and a measurement cannot be: a number that is not a number, and a
// field of another kind entirely.
#[test]
fn a_number_that_is_not_a_finite_number_is_refused_by_its_field() {
    for (path, written) in [
        ("datum.value", "nan"),
        ("datum.value", "inf"),
        ("datum.uncertainty", "-inf"),
    ] {
        match refusal(&instead(path, written)) {
            Refusal::NotAFiniteNumber { field, found } => {
                assert_eq!(field, path);
                // Written rather than compared, because one of the three is not
                // equal to itself and an assertion on the value would pass for
                // the wrong reason.
                assert!(!found.is_finite(), "{written} was accepted");
            }
            other => panic!("{written}: expected a not-a-number refusal, got {other}"),
        }
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
fn a_field_of_the_wrong_kind_is_refused_and_the_message_says_what_was_expected() {
    // One case per kind of value the record reads, because each is a separate
    // reader and a shared message would prove only one of them.
    let cases: &[(&str, &str, &str)] = &[
        ("datum.identifier", "4", "a string"),
        ("datum.value", "\"1234.5\"", "a number"),
        ("datum.enters_the_adjustment", "\"true\"", "true or false"),
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
            other => panic!("{path}: expected a wrong-kind refusal, got {other}"),
        }
        assert!(refused.to_string().contains(expected));
    }
}

#[test]
fn an_optional_field_of_the_wrong_kind_is_refused_rather_than_ignored() {
    match refusal(&also("datum", "superseded_by = 4")) {
        Refusal::WrongKind { field, expected } => {
            assert_eq!(field, "datum.superseded_by");
            assert_eq!(expected, "a string");
        }
        other => panic!("expected a wrong-kind refusal, got {other}"),
    }
}

#[test]
fn a_value_written_as_an_integer_reads_as_the_number_it_is() {
    // A whole number is written `1234` as readily as `1234.0`, and refusing the
    // first would be a refusal about notation rather than about the value.
    let record = accepted(NAME, &instead("datum.value", "1234"));
    assert_eq!(record.value(), 1234.0);
}

// The third of #30's five.
#[test]
fn an_uncertainty_below_zero_is_refused_by_its_value() {
    match refusal(&instead("datum.uncertainty", "-0.02")) {
        Refusal::NegativeUncertainty { found } => assert!(found < 0.0),
        other => panic!("expected a negative-uncertainty refusal, got {other}"),
    }
}

#[test]
fn an_uncertainty_form_that_is_neither_of_the_two_is_refused() {
    match refusal(&instead("datum.uncertainty_form", "\"approximate\"")) {
        Refusal::UnknownUncertaintyForm { found } => assert_eq!(found, "approximate"),
        other => panic!("expected an uncertainty-form refusal, got {other}"),
    }
}

#[test]
fn an_absolute_uncertainty_is_already_absolute_and_a_relative_one_is_converted() {
    let absolute = accepted(NAME, FIXTURE);
    assert_eq!(absolute.absolute_uncertainty(), absolute.uncertainty());

    // The line #6 asks for and neither #30 nor #34 carried: a form stated in
    // the file, converted in one place rather than at every site that wants a
    // number. The value is negative here because a relative uncertainty is a
    // fraction of a magnitude, and a signed one would be a width below zero.
    let relative = accepted(
        NAME,
        &edit(
            &instead("datum.value", "-2.0"),
            "datum.uncertainty_form",
            Some("\"relative\""),
        ),
    );
    assert!(matches!(
        relative.uncertainty_form(),
        UncertaintyForm::Relative
    ));
    assert!(relative.value() < 0.0);
    assert_eq!(
        relative.absolute_uncertainty(),
        relative.uncertainty() * 2.0
    );
    assert!(relative.absolute_uncertainty() > 0.0);
}

#[test]
fn a_superseded_record_says_what_replaced_it() {
    let record = accepted(
        NAME,
        &also("datum", "superseded_by = \"fixture-datum-two\""),
    );
    assert_eq!(record.superseded_by(), Some("fixture-datum-two"));
}

#[test]
fn a_record_saying_it_replaced_itself_is_refused_by_that_identifier() {
    match refusal(&also("datum", "superseded_by = \"fixture-datum-one\"")) {
        Refusal::SupersededByItself { identifier } => {
            assert_eq!(identifier, "fixture-datum-one");
        }
        other => panic!("expected a self-supersession refusal, got {other}"),
    }
}

// The fourth of #30's five.
#[test]
fn a_file_not_named_for_the_identifier_it_carries_is_refused_with_the_name_it_needs() {
    let refused = refused("fixture-datum-two.toml", FIXTURE);
    assert_eq!(refused.file, "fixture-datum-two.toml");
    match &refused.refusal {
        Refusal::FileNameDisagrees { expected } => assert_eq!(expected, NAME),
        other => panic!("expected a file-name refusal, got {other}"),
    }
    assert!(refused.to_string().contains(NAME));
}

#[test]
fn a_record_is_placed_by_its_directory_and_named_by_its_file() {
    // A caller knowing a datum by its path gets the path back in the refusal,
    // and what the identifier is compared against is the last segment. Both
    // separators, because the loader will be handed either.
    for file in [
        NAME,
        "data/2018/fixture-datum-one.toml",
        "data\\2018\\fixture-datum-one.toml",
    ] {
        assert_eq!(accepted(file, FIXTURE).identifier(), "fixture-datum-one");
    }
    let refused = refused("data/2018/fixture-datum-two.toml", FIXTURE);
    assert_eq!(refused.file, "data/2018/fixture-datum-two.toml");
    assert!(matches!(refused.refusal, Refusal::FileNameDisagrees { .. }));
}

#[test]
fn a_record_whose_provenance_block_is_refused_carries_that_refusal() {
    let refused = refused(NAME, &without("measurement.method"));
    match &refused.refusal {
        Refusal::Provenance(provenance::Refusal::Missing { field }) => {
            assert_eq!(field, "measurement.method");
        }
        other => panic!("expected a provenance refusal, got {other}"),
    }
    // Carried as an error rather than only as a value, so a caller can put it
    // behind whatever error type it already returns, and the block's own
    // refusal is reachable through the chain rather than only in the text.
    let carried: &dyn std::error::Error = &refused;
    let inner = carried.source().expect("the refusal is the source");
    assert!(inner.source().is_some(), "the block's refusal is under it");
}

// The fifth of #30's five, and the one that is not a property of a file. The
// one change is to the file the second record was read from.
#[test]
fn one_identifier_carried_by_two_files_is_refused_naming_both() {
    let (first_file, first) = another("fixture-datum-two", &[], &[]);
    let (second_file, second) = another("fixture-datum-three", &[], &[]);
    let valid = vec![(first_file, first), (second_file, second)];
    let counted = match datum::census(&valid) {
        Ok(found) => found,
        Err(refused) => panic!("expected the set to be counted: {refused}"),
    };
    assert_eq!(counted.files, 2);

    let (_, again) = another("fixture-datum-two", &[], &[]);
    let (_, other) = another("fixture-datum-two", &[], &[]);
    let colliding = vec![
        ("data/2018/fixture-datum-two.toml".to_owned(), again),
        ("data/2010/fixture-datum-two.toml".to_owned(), other),
    ];
    match datum::census(&colliding) {
        Err(refused) => {
            assert_eq!(refused.file, "data/2010/fixture-datum-two.toml");
            match &refused.refusal {
                Refusal::IdentifierStatedTwice {
                    identifier,
                    also_in,
                } => {
                    assert_eq!(identifier, "fixture-datum-two");
                    assert_eq!(also_in, "data/2018/fixture-datum-two.toml");
                }
                other => panic!("expected a stated-twice refusal, got {other}"),
            }
        }
        Ok(_) => panic!("expected a refusal, and the set was counted"),
    }
}

#[test]
fn the_census_counts_what_a_reader_of_a_changed_set_wants_to_know() {
    let (in_the_fit, entering) = another("fixture-datum-two", &[], &[]);
    let (comparison_file, comparison) = another(
        "fixture-datum-three",
        &[("datum.enters_the_adjustment", "false")],
        &[],
    );
    assert!(!comparison.enters_the_adjustment());
    let (replaced_file, replaced) = another(
        "fixture-datum-four",
        &[],
        &["superseded_by = \"fixture-datum-two\""],
    );

    let set = vec![
        (in_the_fit, entering),
        (comparison_file, comparison),
        (replaced_file, replaced),
    ];
    let counted = match datum::census(&set) {
        Ok(found) => found,
        Err(refused) => panic!("expected the set to be counted: {refused}"),
    };
    assert_eq!(counted.files, 3);
    assert_eq!(counted.entering_the_adjustment, 2);
    assert_eq!(counted.for_comparison_only, 1);
    assert_eq!(counted.superseded, 1);
    assert_eq!(counted.identifiers.len(), 3);
    assert!(counted.identifiers.contains("fixture-datum-four"));

    let printed = counted.to_string();
    assert!(printed.contains("input data over 3 files"), "{printed}");
    assert!(printed.contains("for comparison only      1"), "{printed}");
    assert!(!format!("{counted:?}").is_empty());
}

#[test]
fn every_refusal_prints_and_debugs() {
    // A refusal a caller cannot print is a refusal that reaches nobody, and the
    // shapes below are the whole set. Each is produced rather than constructed,
    // so a variant that stops being reachable fails here.
    let (_, again) = another("fixture-datum-two", &[], &[]);
    let (_, other) = another("fixture-datum-two", &[], &[]);
    let colliding = vec![
        ("first.toml".to_owned(), again),
        ("second.toml".to_owned(), other),
    ];
    let stated_twice = match datum::census(&colliding) {
        Err(found) => found.refusal,
        Ok(_) => panic!("expected a refusal, and the set was counted"),
    };
    let refusals = [
        refusal("this is not a document = = ="),
        refusal(&without("datum.value")),
        refusal(&instead("datum.value", "\"1234.5\"")),
        refusal(&without("measurement.method")),
        refusal(&instead("datum.value", "nan")),
        refusal(&instead("datum.uncertainty", "-0.02")),
        refusal(&instead("datum.uncertainty_form", "\"approximate\"")),
        refusal(&also("datum", "superseded_by = \"fixture-datum-one\"")),
        refused("fixture-datum-two.toml", FIXTURE).refusal,
        stated_twice,
    ];
    for refused in &refusals {
        assert!(!refused.to_string().is_empty());
        assert!(!format!("{refused:?}").is_empty());
        let carried: &dyn std::error::Error = refused;
        // Only the wrapped block refusal has something under it. The rest end
        // here, and a chain that claimed otherwise would send a reader looking
        // for a cause that does not exist.
        let expected = matches!(refused, Refusal::Provenance(_));
        assert_eq!(carried.source().is_some(), expected, "{refused}");
    }
}
