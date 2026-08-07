//! What the provenance block refuses, and what it accepts.
//!
//! Every refusal below is reached from a valid fixture by one change, so a test
//! that goes green proves the rule rather than proving that the parser dislikes
//! the file. The one change is made here, in the test, rather than by keeping a
//! near-copy of the fixture on disk for each rule: a directory of near-copies
//! drifts apart from the valid one, and then a rule is proved against a file
//! that stopped being a neighbour.
//!
//! The two fixtures on disk are the two that are not near-misses. One is from
//! this repository's domain and one is from the domain the block is shared
//! with, and #33 asks for the second one by name.

use ausgleich_data::provenance::{self, Origin, Refusal, UncertaintyKind};

const ADJUSTMENT: &str = include_str!("fixtures/adjustment-input.toml");
const SIBLING: &str = include_str!("fixtures/sibling-board-measurement.toml");

/// The sections the block requires, each of which is a table.
const SECTIONS: &[&str] = &[
    "publication",
    "read_from",
    "measurement",
    "uncertainty",
    "digitisation",
];

/// Every required field, written as the refusal names it.
const FIELDS: &[&str] = &[
    "version",
    "publication.authors",
    "publication.title",
    "publication.venue",
    "publication.year",
    "publication.doi",
    "read_from.locator",
    "read_from.page",
    "measurement.reported",
    "measurement.method",
    "measurement.origin",
    "uncertainty.kind",
    "uncertainty.coverage_factor",
    "digitisation.read_by",
    "digitisation.read_on",
    "digitisation.confirmed_by",
    "digitisation.confirmed_on",
];

/// The fixture with the one line for `path` taken out, or replaced.
///
/// `path` is `section.key`, or a bare key for a field at the top of the file.
/// The walk asserts that it changed exactly one line, so a path that stops
/// matching the fixture fails here rather than quietly testing nothing.
fn edit(path: &str, replacement: Option<&str>) -> String {
    let (section, key) = match path.split_once('.') {
        Some(split) => split,
        None => ("", path),
    };
    let mut kept: Vec<String> = Vec::new();
    let mut current = "";
    let mut hits = 0usize;
    for line in ADJUSTMENT.lines() {
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
    edit(path, None)
}

/// The fixture with the value at `path` written as `value`.
fn instead(path: &str, value: &str) -> String {
    edit(path, Some(value))
}

/// The fixture without the whole `[name]` table.
fn without_section(name: &str) -> String {
    let header = format!("[{name}]");
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    let mut found = false;
    for line in ADJUSTMENT.lines() {
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
fn refusal(document: &str) -> Refusal {
    match provenance::read(document) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the block was accepted"),
    }
}

/// The block a document parses to, or a failure carrying the refusal.
fn accepted(document: &str) -> provenance::Provenance {
    match provenance::read(document) {
        Ok(block) => block,
        Err(refused) => panic!("expected the block to be accepted: {refused}"),
    }
}

#[test]
fn the_fixture_from_this_repositorys_domain_reads() {
    let block = accepted(ADJUSTMENT);
    assert_eq!(block.version, provenance::BLOCK_VERSION);
    assert_eq!(block.publication.authors.len(), 3);
    assert_eq!(block.publication.venue, "Journal of Fixtures");
    assert_eq!(block.publication.year, 2019);
    assert_eq!(block.read_from.locator, "Table IV");
    assert_eq!(block.read_from.page, 17);
    assert_eq!(block.reported, "2018-11-05");
    assert_eq!(block.method, "kibble-balance");
    assert!(matches!(block.origin, Origin::Measured));
    assert!(matches!(block.uncertainty.kind, UncertaintyKind::Combined));
    assert_eq!(block.digitisation.read_by, "Reader, A.");
    assert_eq!(block.digitisation.confirmed_on, "2026-02-13");
    // Formatted rather than only read, because the block is a thing a caller
    // will print when it refuses something else.
    assert!(format!("{block:?}").contains("kibble-balance"));
}

#[test]
fn a_fixture_from_the_sibling_boards_domain_reads_unchanged() {
    // #33's own test. The bytes on disk are the bytes read: nothing in this
    // test edits the fixture, because a fixture that has to be adjusted to pass
    // is the answer this test exists to catch.
    let block = accepted(SIBLING);
    assert_eq!(block.method, "isotope-dilution-mass-spectrometry");
    assert!(matches!(block.origin, Origin::DerivedByThePublication));
    assert!(matches!(
        block.uncertainty.kind,
        UncertaintyKind::Systematic
    ));
    // Written as an integer in that fixture, and read as the number it is.
    assert!(block.uncertainty.coverage_factor > 1.0);
    assert!(format!("{block:?}").contains("Systematic"));
}

#[test]
fn the_method_of_the_sibling_fixture_is_a_term_of_the_committed_vocabulary() {
    // The sharing is a property of the vocabulary rather than of the fixture.
    // A vocabulary holding only this repository's own methods would make the
    // fixture above fail, and this says so where a reader is looking at the
    // vocabulary rather than at the fixture.
    let terms = provenance::methods();
    assert!(terms.contains(&"isotope-dilution-mass-spectrometry"));
    assert!(terms.contains(&"kibble-balance"));
}

#[test]
fn the_vocabulary_states_its_own_version_and_holds_nothing_else_odd() {
    assert_eq!(provenance::vocabulary_version(), "1");
    let terms = provenance::methods();
    assert!(!terms.is_empty());
    for term in &terms {
        assert!(!term.starts_with("version "), "{term} is the version line");
        assert!(!term.contains(' '), "{term} carries a space");
    }
    let mut sorted = terms.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), terms.len(), "a term is listed twice");
}

#[test]
fn a_block_missing_any_required_field_is_refused_by_that_field() {
    for path in FIELDS {
        match refusal(&without(path)) {
            Refusal::Missing { field } => assert_eq!(&field, path),
            other => panic!("{path}: expected a missing-field refusal, got {other}"),
        }
    }
}

#[test]
fn a_block_missing_any_required_table_is_refused_by_that_table() {
    for name in SECTIONS {
        match refusal(&without_section(name)) {
            Refusal::Missing { field } => assert_eq!(&field, name),
            other => panic!("{name}: expected a missing-table refusal, got {other}"),
        }
    }
}

#[test]
fn a_block_with_no_version_is_refused_for_the_version_and_not_for_the_shape() {
    // Named on its own as well as inside the loop above, because #33 asks for
    // this refusal by name and a rule proved only inside a loop is a rule
    // somebody deletes with the loop.
    match refusal(&without("version")) {
        Refusal::Missing { field } => assert_eq!(field, "version"),
        other => panic!("expected a missing-version refusal, got {other}"),
    }
}

#[test]
fn a_block_stating_a_version_this_reader_does_not_know_is_refused_by_the_version() {
    let stated = provenance::BLOCK_VERSION + 1;
    match refusal(&instead("version", &stated.to_string())) {
        Refusal::UnknownVersion { found } => assert_eq!(found, stated),
        other => panic!("expected an unknown-version refusal, got {other}"),
    }
}

#[test]
fn a_method_outside_the_vocabulary_is_refused_and_the_message_names_the_term() {
    let refused = refusal(&instead("measurement.method", "\"a method nobody agreed\""));
    match &refused {
        Refusal::MethodOutsideVocabulary { found } => {
            assert_eq!(found, "a method nobody agreed");
        }
        other => panic!("expected a vocabulary refusal, got {other}"),
    }
    let message = refused.to_string();
    assert!(message.contains("a method nobody agreed"), "{message}");
    assert!(
        message.contains(provenance::vocabulary_version()),
        "{message}"
    );
}

#[test]
fn an_uncertainty_kind_outside_the_three_is_refused() {
    match refusal(&instead("uncertainty.kind", "\"approximate\"")) {
        Refusal::UnknownUncertaintyKind { found } => assert_eq!(found, "approximate"),
        other => panic!("expected an uncertainty-kind refusal, got {other}"),
    }
}

#[test]
fn the_third_uncertainty_kind_reads_as_well_as_the_other_two() {
    let block = accepted(&instead("uncertainty.kind", "\"statistical\""));
    assert!(matches!(
        block.uncertainty.kind,
        UncertaintyKind::Statistical
    ));
    assert!(format!("{:?}", block.uncertainty).contains("Statistical"));
}

#[test]
fn an_origin_that_is_neither_of_the_two_is_refused() {
    match refusal(&instead("measurement.origin", "\"assumed\"")) {
        Refusal::UnknownOrigin { found } => assert_eq!(found, "assumed"),
        other => panic!("expected an origin refusal, got {other}"),
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
    // One case per kind of value the block reads, because each is a separate
    // reader and a shared message would prove only one of them.
    let cases: &[(&str, &str, &str)] = &[
        ("publication.title", "4", "a string"),
        (
            "publication.authors",
            "\"Beispiel, A.\"",
            "an array of strings",
        ),
        (
            "publication.authors",
            "[\"Beispiel, A.\", 4]",
            "an array of strings",
        ),
        ("publication.year", "\"2019\"", "an integer"),
        ("uncertainty.coverage_factor", "\"one\"", "a number"),
        ("measurement.reported", "\"2018-11-05\"", "a date"),
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
fn a_table_that_is_not_a_table_is_refused_as_one() {
    // Reached by writing the name as a value rather than as a heading, which is
    // the mistake somebody actually makes. The document stops at that point, so
    // nothing after it needs to be present.
    let document = format!(
        "version = {}\npublication = \"Journal of Fixtures\"\n",
        provenance::BLOCK_VERSION
    );
    match refusal(&document) {
        Refusal::WrongKind { field, expected } => {
            assert_eq!(field, "publication");
            assert_eq!(expected, "a table");
        }
        other => panic!("expected a wrong-kind refusal for the table, got {other}"),
    }
}

#[test]
fn every_refusal_prints_and_debugs() {
    // A refusal a caller cannot print is a refusal that reaches nobody, and the
    // shapes below are the whole set. Each is produced rather than constructed,
    // so a variant that stops being reachable fails here.
    let stated = provenance::BLOCK_VERSION + 1;
    let refusals = [
        refusal("this is not a document = = ="),
        refusal(&without("version")),
        refusal(&instead("publication.title", "4")),
        refusal(&instead("version", &stated.to_string())),
        refusal(&instead("measurement.method", "\"a method nobody agreed\"")),
        refusal(&instead("uncertainty.kind", "\"approximate\"")),
        refusal(&instead("measurement.origin", "\"assumed\"")),
    ];
    for refused in &refusals {
        assert!(!refused.to_string().is_empty());
        assert!(!format!("{refused:?}").is_empty());
        // Carried as an error rather than only as a value, so a caller can put
        // it behind whatever error type it already returns.
        let carried: &dyn std::error::Error = refused;
        assert!(carried.source().is_none());
    }
}
