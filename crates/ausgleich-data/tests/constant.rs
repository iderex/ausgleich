//! What the adjusted constant record refuses, and what it accepts.
//!
//! #32 names two refusals in its Done-when, a missing unit and a missing
//! starting-value source, and both are here under their own names as well as
//! inside the loop that walks every required field. A rule proved only inside a
//! loop is a rule somebody deletes with the loop.
//!
//! The loop is the count #4 asks for rather than the count of named rules: one
//! fixture per required field, each differing from the valid file by the one
//! line it is about. A single missing-field fixture leaves every field the code
//! forgot to require green.
//!
//! Blankness is proved per field for the same reason. A required string written
//! as `""` is present to any reader that asks only whether the key is there, and
//! it is the shape a half-finished file actually has.
//!
//! The changes are made here rather than by keeping a near-copy of the fixture
//! on disk for each rule, for the reason the suites beside this one give: a
//! directory of near-copies drifts apart from the valid one, and then a rule is
//! proved against a file that stopped being a neighbour.

use ausgleich_data::constant::{self, AdjustedConstant, RECORD_VERSION, Refusal, Refused};
use ausgleich_data::units::{self, UnitTable};

const FIXTURE: &str = include_str!("fixtures/fixture-quantity-one.toml");

/// The name the fixture's own file has, which is the name its identifier
/// requires. The rule tying the two together is proved against the real name.
const NAME: &str = "fixture-quantity-one.toml";

/// Every field the record requires, written as the refusal names it.
const FIELDS: &[&str] = &[
    "version",
    "constant.identifier",
    "constant.symbol",
    "constant.unit",
    "constant.dimension",
    "constant.exact_by_definition",
    "starting_value.value",
    "starting_value.source",
];

/// The required fields that are strings, which are the ones blankness reaches.
const TEXT_FIELDS: &[&str] = &[
    "constant.identifier",
    "constant.symbol",
    "constant.unit",
    "constant.dimension",
    "starting_value.source",
];

/// The committed table, which is what a record in this repository is judged
/// against.
fn table() -> UnitTable {
    match units::committed() {
        Ok(found) => found,
        Err(refused) => panic!("the committed unit table reads: {refused}"),
    }
}

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
    match constant::read(file, document, &table()) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the record was accepted"),
    }
}

/// The refusal a document read under the fixture's own name earns.
fn refusal(document: &str) -> Refusal {
    refused(NAME, document).refusal
}

/// The record a document parses to, or a failure carrying the refusal.
fn accepted(file: &str, document: &str) -> AdjustedConstant {
    match constant::read(file, document, &table()) {
        Ok(found) => found,
        Err(refused) => panic!("expected the record to be accepted: {refused}"),
    }
}

#[test]
fn the_fixture_reads_and_every_field_comes_back() {
    let record = accepted(NAME, FIXTURE);
    assert_eq!(record.identifier(), "fixture-quantity-one");
    assert_eq!(record.symbol(), "X_f");
    assert_eq!(record.unit(), "Hz");
    assert_eq!(record.dimension(), "frequency");
    assert!(!record.exact_by_definition());
    assert!(record.starting_value() > 0.0);
    assert!(
        record
            .starting_value_source()
            .contains("fixture publication")
    );
    assert_eq!(record.file_name(), NAME);
    assert!(format!("{record:?}").contains("fixture-quantity-one"));
}

// #4's count rather than the count of named rules. One fixture per required
// field, each differing from the valid file by the one line it is about.
#[test]
fn a_record_missing_any_required_field_is_refused_by_that_field() {
    for path in FIELDS {
        match refusal(&without(path)) {
            Refusal::Missing { field } => assert_eq!(&field, path),
            other => panic!("{path}: expected a missing-field refusal, got {other}"),
        }
    }
}

// The first of the two refusals #32's Done-when names.
#[test]
fn a_record_with_no_unit_is_refused_for_the_unit() {
    match refusal(&without("constant.unit")) {
        Refusal::Missing { field } => assert_eq!(field, "constant.unit"),
        other => panic!("expected a missing-unit refusal, got {other}"),
    }
}

// The second. A starting value with no source is the case #32 argues about: the
// number is a legitimate choice and the file has to say where it came from.
#[test]
fn a_starting_value_with_no_source_is_refused_for_the_source() {
    match refusal(&without("starting_value.source")) {
        Refusal::Missing { field } => assert_eq!(field, "starting_value.source"),
        other => panic!("expected a missing-source refusal, got {other}"),
    }
}

#[test]
fn a_required_string_written_as_nothing_is_refused_by_that_field() {
    for path in TEXT_FIELDS {
        match refusal(&instead(path, "\"\"")) {
            Refusal::Blank { field } => assert_eq!(&field, path),
            other => panic!("{path}: expected a blank-field refusal, got {other}"),
        }
    }
}

#[test]
fn a_source_of_nothing_but_spaces_is_refused_as_blank() {
    // The near miss. A file whose author meant to come back to it carries
    // whitespace rather than an empty string, and a check reading only for the
    // empty string would pass it.
    let refused = refusal(&instead("starting_value.source", "\"   \""));
    match &refused {
        Refusal::Blank { field } => assert_eq!(field, "starting_value.source"),
        other => panic!("expected a blank-field refusal, got {other}"),
    }
    assert!(refused.to_string().contains("says nothing"));
}

#[test]
fn a_record_without_one_of_its_tables_is_refused_by_that_table() {
    for name in ["constant", "starting_value"] {
        match refusal(&without_table(name)) {
            Refusal::Missing { field } => assert_eq!(field, name),
            other => panic!("{name}: expected a missing-table refusal, got {other}"),
        }
    }
}

#[test]
fn a_table_written_as_a_value_is_refused_as_a_table() {
    // The mistake somebody actually makes. Both tables are looked for before any
    // field is read, so the second case carries a `[constant]` table and the
    // refusal is about the one under test rather than about the one in front of
    // it.
    let cases = [
        (
            "constant",
            format!("version = {RECORD_VERSION}\nconstant = \"fixture-quantity-one\"\n"),
        ),
        (
            "starting_value",
            format!(
                "version = {RECORD_VERSION}\nstarting_value = \"1234.5\"\n\n\
                 [constant]\nidentifier = \"fixture-quantity-one\"\n"
            ),
        ),
    ];
    for (name, document) in &cases {
        match refusal(document) {
            Refusal::WrongKind { field, expected } => {
                assert_eq!(&field, name);
                assert_eq!(expected, "a table");
            }
            other => panic!("{name}: expected a wrong-kind refusal for the table, got {other}"),
        }
    }
}

#[test]
fn a_record_written_under_another_version_is_refused_for_the_version() {
    // Refused for the version rather than for whichever field the shape moved,
    // which is what a reader of a later file needs to be told.
    match refusal(&instead("version", &(RECORD_VERSION + 1).to_string())) {
        Refusal::UnknownVersion { found } => assert_eq!(found, RECORD_VERSION + 1),
        other => panic!("expected an unknown-version refusal, got {other}"),
    }
}

#[test]
fn a_version_that_is_not_an_integer_is_refused_as_one() {
    match refusal(&instead("version", "\"1\"")) {
        Refusal::WrongKind { field, expected } => {
            assert_eq!(field, "version");
            assert_eq!(expected, "an integer");
        }
        other => panic!("expected a wrong-kind refusal for the version, got {other}"),
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
        ("constant.identifier", "4", "a string"),
        ("constant.exact_by_definition", "\"true\"", "true or false"),
        ("starting_value.value", "\"1234.5\"", "a number"),
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
fn a_starting_value_that_is_not_a_finite_number_is_refused_by_its_field() {
    for written in ["nan", "inf", "-inf"] {
        match refusal(&instead("starting_value.value", written)) {
            Refusal::NotAFiniteNumber { field, found } => {
                assert_eq!(field, "starting_value.value");
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
fn a_starting_value_written_as_an_integer_reads_as_the_number_it_is() {
    // A whole number is written `1234` as readily as `1234.0`, and refusing the
    // first would be a refusal about notation rather than about the value.
    let record = accepted(NAME, &instead("starting_value.value", "1234"));
    assert_eq!(record.starting_value(), 1234.0);
}

#[test]
fn a_quantity_exact_by_definition_says_so() {
    // The field #32 exists for. One adjustment fits this quantity and another
    // defines it, and which of the two is being reproduced is a fact about the
    // file rather than a branch in the solver.
    let record = accepted(NAME, &instead("constant.exact_by_definition", "true"));
    assert!(record.exact_by_definition());
}

#[test]
fn a_file_not_named_for_the_identifier_it_carries_is_refused_with_the_name_it_needs() {
    let refused = refused("fixture-quantity-two.toml", FIXTURE);
    assert_eq!(refused.file, "fixture-quantity-two.toml");
    match &refused.refusal {
        Refusal::FileNameDisagrees { expected } => assert_eq!(expected, NAME),
        other => panic!("expected a file-name refusal, got {other}"),
    }
    assert!(refused.to_string().contains(NAME));
}

#[test]
fn a_record_is_placed_by_its_directory_and_named_by_its_file() {
    // A caller knowing a record by its path gets the path back in the refusal,
    // and what the identifier is compared against is the last segment. Both
    // separators, because the loader will be handed either.
    for file in [
        NAME,
        "data/2018/fixture-quantity-one.toml",
        "data\\2018\\fixture-quantity-one.toml",
    ] {
        assert_eq!(accepted(file, FIXTURE).identifier(), "fixture-quantity-one");
    }
    let refused = refused("data/2018/fixture-quantity-two.toml", FIXTURE);
    assert_eq!(refused.file, "data/2018/fixture-quantity-two.toml");
    assert!(matches!(refused.refusal, Refusal::FileNameDisagrees { .. }));
}

// The declared dimension is what the unit is judged against, and these three are
// what the declaration buys. Derived from the unit instead, none of them could
// fire.
#[test]
fn a_unit_the_table_does_not_carry_is_refused() {
    match refusal(&instead("constant.unit", "\"hertz\"")) {
        Refusal::Unit(units::Refusal::UnknownUnit { found }) => assert_eq!(found, "hertz"),
        other => panic!("expected an unknown-unit refusal, got {other}"),
    }
}

#[test]
fn a_dimension_the_table_does_not_carry_is_refused() {
    match refusal(&instead("constant.dimension", "\"frequenz\"")) {
        Refusal::Unit(units::Refusal::UnknownDimension { found }) => assert_eq!(found, "frequenz"),
        other => panic!("expected an unknown-dimension refusal, got {other}"),
    }
}

#[test]
fn a_unit_of_another_dimension_than_the_one_declared_is_refused_naming_both() {
    // The mistake the declaration exists to catch. Seconds and hertz are the
    // reciprocal pair a transcription slips between, and the record says which
    // of the two the quantity is.
    let refused = refusal(&instead("constant.unit", "\"s\""));
    match &refused {
        Refusal::Unit(units::Refusal::DimensionMismatch {
            unit,
            belongs_to,
            declared,
        }) => {
            assert_eq!(unit, "s");
            assert_eq!(belongs_to, "time");
            assert_eq!(declared, "frequency");
        }
        other => panic!("expected a dimension-mismatch refusal, got {other}"),
    }
    // The field is named in the message, so the person who has to fix it is
    // told which of the two lines to read.
    assert!(refused.to_string().contains("constant.unit"), "{refused}");
}

#[test]
fn every_refusal_prints_and_debugs() {
    // A refusal a caller cannot print is a refusal that reaches nobody, and the
    // shapes below are the whole set. Each is produced rather than constructed,
    // so a variant that stops being reachable fails here.
    let refusals = [
        refusal("this is not a document = = ="),
        refusal(&without("constant.unit")),
        refusal(&instead("constant.identifier", "4")),
        refusal(&instead("starting_value.source", "\"\"")),
        refusal(&instead("version", &(RECORD_VERSION + 1).to_string())),
        refusal(&instead("starting_value.value", "nan")),
        refused("fixture-quantity-two.toml", FIXTURE).refusal,
        refusal(&instead("constant.unit", "\"hertz\"")),
    ];
    for refused in &refusals {
        assert!(!refused.to_string().is_empty());
        assert!(!format!("{refused:?}").is_empty());
        let carried: &dyn std::error::Error = refused;
        // Only the wrapped table refusal has something under it. The rest end
        // here, and a chain that claimed otherwise would send a reader looking
        // for a cause that does not exist.
        let expected = matches!(refused, Refusal::Unit(_));
        assert_eq!(carried.source().is_some(), expected, "{refused}");
    }
}

#[test]
fn a_refusal_is_carried_as_an_error_with_the_table_refusal_under_it() {
    let refused = refused(NAME, &instead("constant.unit", "\"hertz\""));
    // Carried as an error rather than only as a value, so a caller can put it
    // behind whatever error type it already returns, and the table's own
    // refusal is reachable through the chain rather than only in the text.
    let carried: &dyn std::error::Error = &refused;
    let inner = carried.source().expect("the refusal is the source");
    assert!(inner.source().is_some(), "the table's refusal is under it");
    assert!(format!("{refused:?}").contains("UnknownUnit"));
}
