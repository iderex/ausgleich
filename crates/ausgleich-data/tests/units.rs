//! What the unit table refuses, what the dimensional check refuses, and what
//! the round trip is allowed to lose.
//!
//! Two kinds of test here, kept apart on purpose. The rules are proved against
//! fixture tables written in this file, because a rule proved against the
//! committed table proves the state of the tree on the day it ran rather than
//! the guard. What is asserted about the committed table is separate and says
//! so: that it reads, that every dimension in it has exactly one base, and that
//! every entry carrying a factor survives a round trip.
//!
//! ## The tolerance, and where it comes from
//!
//! #34 asks for a round trip "within a stated tolerance" and states none, and
//! the finding already written into that issue says why picking one while the
//! test is being written is picking one that passes. So it is argued from the
//! arithmetic instead.
//!
//! A round trip is two operations on doubles: one multiply into the base unit
//! and one divide back out. Each is correctly rounded, so each carries a
//! relative error of at most one unit in the last place, which is
//! `f64::EPSILON / 2`. Composed, the bound is `f64::EPSILON` plus a term in the
//! square of it that is far below the last bit. The tolerance below is twice
//! that bound, which is two bits of headroom and no more, so a factor that
//! loses a digit fails rather than passes.
//!
//! It is a relative tolerance because the values it is applied to span the
//! table, from a factor of one to a factor near ten to the minus nineteen, and
//! an absolute tolerance would be vacuous at one end and impossible at the
//! other.

use ausgleich_data::units::{self, Factor, Refusal, TableRefusal, UnitTable};

/// Two units in the last place, argued in the module comment above.
const TOLERANCE: f64 = 2.0 * f64::EPSILON;

/// The file and the field a caller passes in. Every refusal names both.
const FILE: &str = "data/2018/fixture-datum-one.toml";
const FIELD: &str = "datum.unit";

/// A table document carrying the entries given, at the version this reader
/// reads.
fn document(entries: &[&str]) -> String {
    let mut text = format!("version = {}\n", units::TABLE_VERSION);
    for entry in entries {
        text.push_str("\n[[unit]]\n");
        text.push_str(entry);
        text.push('\n');
    }
    text
}

/// A base unit of a dimension: factor exactly one, exact by definition.
fn base(symbol: &str, dimension: &str) -> String {
    format!(
        "symbol = \"{symbol}\"\ndimension = \"{dimension}\"\n\
         factor_is = \"exact-by-definition\"\nfactor = 1.0"
    )
}

/// A fixture table with one dimension, its base, and one scaled unit.
fn fixture() -> String {
    document(&[
        &base("Hz", "frequency"),
        "symbol = \"MHz\"\ndimension = \"frequency\"\n\
         factor_is = \"exact-by-definition\"\nfactor = 1e6",
        &base("kg", "mass"),
        // Invented, like every other number in a fixture in this crate. The
        // committed table carries no measured entry today, so this arm of the
        // schema is proved here and nowhere else, which is said in the table
        // itself as well.
        "symbol = \"zz\"\ndimension = \"mass\"\nfactor_is = \"measured\"\n\
         factor = 2.0\nsource = \"A fixture standing in for a published conversion\"",
        "symbol = \"u\"\ndimension = \"mass\"\n\
         factor_is = \"an-adjusted-constant\"\nconstant = \"atomic-mass-constant\"",
    ])
}

/// The table a document reads to, or a failure carrying the refusal.
fn accepted(document: &str) -> UnitTable {
    match units::read(document) {
        Ok(found) => found,
        Err(refused) => panic!("expected the table to be read: {refused}"),
    }
}

/// The refusal a document earns, or a failure saying it earned none.
fn refusal(document: &str) -> TableRefusal {
    match units::read(document) {
        Err(found) => found,
        Ok(_) => panic!("expected a refusal, and the table was read"),
    }
}

/// The refusal a unit and a declared dimension earn against the fixture table.
fn refused(symbol: &str, declared: &str) -> units::Refused {
    match accepted(&fixture()).check(FILE, FIELD, symbol, declared) {
        Err(found) => found,
        Ok(()) => panic!("expected a refusal, and the unit was accepted"),
    }
}

#[test]
fn the_committed_table_reads_and_carries_the_kinds_it_says_it_does() {
    let table = match units::committed() {
        Ok(found) => found,
        Err(refused) => panic!("the committed table has to read: {refused}"),
    };
    assert_eq!(table.version(), units::TABLE_VERSION);

    let mut exact = 0usize;
    let mut measured = 0usize;
    let mut adjusted = 0usize;
    for unit in table.units() {
        match unit.factor() {
            Factor::ExactByDefinition(_) => exact += 1,
            Factor::Measured { .. } => measured += 1,
            Factor::AnAdjustedConstant { .. } => adjusted += 1,
        }
    }
    assert!(exact > 0);
    assert_eq!(adjusted, 1, "the atomic mass unit is the one without one");
    // Stated rather than asserted away: the committed table has no measured
    // entry today, so that arm is proved by the fixture above and by nothing in
    // the tree. A green run here is not evidence that the tree exercises it.
    assert_eq!(measured, 0);

    assert!(table.unit("Hz").is_some());
    assert!(table.unit("nope").is_none());
    assert!(format!("{table:?}").contains("frequency"));
}

#[test]
fn every_dimension_of_the_committed_table_has_exactly_one_base_unit() {
    let table = accepted(include_str!("../unit-table.toml"));
    assert!(!table.dimensions().is_empty());
    for dimension in table.dimensions() {
        let symbol = match table.base_of(dimension) {
            Some(found) => found,
            None => panic!("{dimension} has no base unit"),
        };
        let unit = match table.unit(symbol) {
            Some(found) => found,
            None => panic!("{symbol} is a base and is not an entry"),
        };
        assert_eq!(unit.dimension(), dimension);
        assert_eq!(unit.factor().value(), Some(1.0));
        assert_eq!(unit.symbol(), symbol);
    }
    assert_eq!(table.base_of("nothing-of-that-name"), None);
}

// #34's third clause, over the table this repository committed rather than over
// a fixture, because the clause is about the entries that exist.
#[test]
fn every_committed_factor_survives_a_round_trip_within_the_stated_tolerance() {
    let table = match units::committed() {
        Ok(found) => found,
        Err(refused) => panic!("the committed table has to read: {refused}"),
    };
    let mut round_tripped = 0usize;
    let mut without_a_factor: Vec<&str> = Vec::new();

    for unit in table.units() {
        if unit.factor().value().is_none() {
            // A round trip through a factor that is itself being fitted is not
            // a test of the table, so these are counted and named rather than
            // skipped quietly.
            without_a_factor.push(unit.symbol());
            continue;
        }
        for probe in [1.0, 3.0 / 7.0, 1234.5, 1.0 / 3.0] {
            let into = match table.into_base(FILE, FIELD, unit.symbol(), unit.dimension(), probe) {
                Ok(found) => found,
                Err(refused) => panic!("{}: {refused}", unit.symbol()),
            };
            let back = match table.from_base(FILE, FIELD, unit.symbol(), into) {
                Ok(found) => found,
                Err(refused) => panic!("{}: {refused}", unit.symbol()),
            };
            let lost = (back - probe).abs() / probe.abs();
            assert!(
                lost <= TOLERANCE,
                "{} lost {lost} of {probe}, over the tolerance {TOLERANCE}",
                unit.symbol()
            );
        }
        round_tripped += 1;
    }

    assert!(round_tripped > 0);
    assert_eq!(
        without_a_factor,
        vec!["u"],
        "the entries this clause does not reach"
    );
}

#[test]
fn a_unit_converts_into_its_base_and_back_by_the_factor_the_table_carries() {
    let table = accepted(&fixture());
    let into = match table.into_base(FILE, FIELD, "MHz", "frequency", 2.0) {
        Ok(found) => found,
        Err(refused) => panic!("expected a conversion: {refused}"),
    };
    assert_eq!(into, 2_000_000.0);
    match table.from_base(FILE, FIELD, "MHz", into) {
        Ok(found) => assert_eq!(found, 2.0),
        Err(refused) => panic!("expected a conversion: {refused}"),
    }
    // A measured factor converts like any other. What is different about it is
    // that it carries where it came from.
    match table.unit("zz").map(units::Unit::factor) {
        Some(Factor::Measured { value, source }) => {
            assert_eq!(*value, 2.0);
            assert!(source.contains("fixture"));
        }
        other => panic!("expected a measured factor, got {other:?}"),
    }
    match table.into_base(FILE, FIELD, "zz", "mass", 3.0) {
        Ok(found) => assert_eq!(found, 6.0),
        Err(refused) => panic!("expected a conversion: {refused}"),
    }
}

#[test]
fn a_conversion_runs_the_dimensional_check_before_it_looks_for_a_factor() {
    let table = accepted(&fixture());
    match table.into_base(FILE, FIELD, "MHz", "mass", 2.0) {
        Err(found) => assert!(matches!(found.refusal, Refusal::DimensionMismatch { .. })),
        Ok(converted) => panic!("expected a refusal, and it converted to {converted}"),
    }

    // The line above passes whichever way round the two steps are written,
    // because MHz has a factor and the check refuses either way. This one does
    // not. `u` is a unit of mass with no factor at all, offered for a quantity
    // declared as frequency, so both steps have something to say and only the
    // one that ran first says it. The dimension is the defect the file has, and
    // being told instead that the conversion would need the atomic mass
    // constant sends the reader to argue with the unit table over a value whose
    // unit was never the right kind.
    match table.into_base(FILE, FIELD, "u", "frequency", 12.0) {
        Err(found) => match found.refusal {
            Refusal::DimensionMismatch { belongs_to, .. } => assert_eq!(belongs_to, "mass"),
            other => panic!("the factor was looked for before the dimension: {other}"),
        },
        Ok(converted) => panic!("expected a refusal, and it converted to {converted}"),
    }
}

// The first of #34's two refusals about a value.
#[test]
fn an_unknown_unit_is_refused_with_the_file_and_the_field_named() {
    let refused = refused("MHZ", "frequency");
    assert_eq!(refused.file, FILE);
    assert_eq!(refused.field, FIELD);
    match &refused.refusal {
        Refusal::UnknownUnit { found } => assert_eq!(found, "MHZ"),
        other => panic!("expected an unknown-unit refusal, got {other}"),
    }
    let printed = refused.to_string();
    assert!(printed.contains(FILE), "{printed}");
    assert!(printed.contains(FIELD), "{printed}");
    assert!(printed.contains("MHZ"), "{printed}");
}

// The second of the two.
#[test]
fn a_unit_of_another_dimension_is_refused_with_both_dimensions_named() {
    let refused = refused("MHz", "mass");
    assert_eq!(refused.file, FILE);
    assert_eq!(refused.field, FIELD);
    match &refused.refusal {
        Refusal::DimensionMismatch {
            unit,
            belongs_to,
            declared,
        } => {
            assert_eq!(unit, "MHz");
            assert_eq!(belongs_to, "frequency");
            assert_eq!(declared, "mass");
        }
        other => panic!("expected a dimension refusal, got {other}"),
    }
    assert!(refused.to_string().contains("frequency"));
}

#[test]
fn a_dimension_no_entry_names_is_refused_rather_than_treated_as_a_mismatch() {
    // A quantity declaring a dimension nothing knows is a defect in the
    // quantity, and saying "MHz is a unit of frequency, not of luminance" would
    // send the reader to the wrong file.
    match refused("MHz", "luminance").refusal {
        Refusal::UnknownDimension { found } => assert_eq!(found, "luminance"),
        other => panic!("expected an unknown-dimension refusal, got {other}"),
    }
}

#[test]
fn a_unit_whose_factor_is_an_adjusted_constant_refuses_the_conversion_by_name() {
    let table = accepted(&fixture());
    // The dimensional check passes: it is a unit of mass and the quantity says
    // mass. What fails is the conversion, and it fails naming what it would
    // need.
    assert!(table.check(FILE, FIELD, "u", "mass").is_ok());
    let refused = match table.into_base(FILE, FIELD, "u", "mass", 12.0) {
        Err(found) => found,
        Ok(converted) => panic!("expected a refusal, and it converted to {converted}"),
    };
    match &refused.refusal {
        Refusal::NotAConversion { unit, constant } => {
            assert_eq!(unit, "u");
            assert_eq!(constant, "atomic-mass-constant");
        }
        other => panic!("expected a not-a-conversion refusal, got {other}"),
    }
    assert!(refused.to_string().contains("atomic-mass-constant"));

    // And the same in the other direction, which is what keeps the round trip
    // from being the place somebody quietly invents a factor.
    match table.from_base(FILE, FIELD, "u", 12.0) {
        Err(found) => assert!(matches!(found.refusal, Refusal::NotAConversion { .. })),
        Ok(converted) => panic!("expected a refusal, and it converted to {converted}"),
    }
    match table.from_base(FILE, FIELD, "nope", 12.0) {
        Err(found) => assert!(matches!(found.refusal, Refusal::UnknownUnit { .. })),
        Ok(converted) => panic!("expected a refusal, and it converted to {converted}"),
    }
}

#[test]
fn dimensionless_is_a_dimension_and_is_checked_like_any_other() {
    // The common case this catches is a ratio quoted with a stray unit.
    let table = match units::committed() {
        Ok(found) => found,
        Err(refused) => panic!("the committed table has to read: {refused}"),
    };
    assert!(table.check(FILE, FIELD, "1", "dimensionless").is_ok());
    match table.check(FILE, FIELD, "MHz", "dimensionless") {
        Err(found) => assert!(matches!(found.refusal, Refusal::DimensionMismatch { .. })),
        Ok(()) => panic!("a frequency was accepted for a dimensionless quantity"),
    }
}

#[test]
fn bytes_that_are_not_a_document_are_refused_as_unreadable() {
    match refusal("this is not a document = = =") {
        TableRefusal::Unreadable(detail) => assert!(!detail.is_empty()),
        other => panic!("expected an unreadable refusal, got {other}"),
    }
}

#[test]
fn a_table_with_no_version_or_no_entries_is_refused_by_what_is_missing() {
    match refusal("") {
        TableRefusal::Missing { field } => assert_eq!(field, "version"),
        other => panic!("expected a missing-version refusal, got {other}"),
    }
    match refusal(&format!("version = {}\n", units::TABLE_VERSION)) {
        TableRefusal::Missing { field } => assert_eq!(field, "unit"),
        other => panic!("expected a missing-unit refusal, got {other}"),
    }
}

#[test]
fn a_table_field_of_the_wrong_kind_is_refused_and_says_what_was_expected() {
    let cases: &[(String, &str, &str)] = &[
        ("version = \"1\"\n".to_owned(), "version", "an integer"),
        (
            format!("version = {}\nunit = \"Hz\"\n", units::TABLE_VERSION),
            "unit",
            "an array of tables",
        ),
        (
            format!("version = {}\nunit = [\"Hz\"]\n", units::TABLE_VERSION),
            "unit[0]",
            "a table",
        ),
        (document(&["symbol = 4"]), "unit[0].symbol", "a string"),
        (
            document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
                 factor_is = \"exact-by-definition\"\nfactor = \"one\""]),
            "unit[0].factor",
            "a number",
        ),
    ];
    for (text, field, expected) in cases {
        let refused = refusal(text);
        match &refused {
            TableRefusal::WrongKind {
                field: said,
                expected: what,
            } => {
                assert_eq!(said, field);
                assert_eq!(what, expected);
            }
            other => panic!("{field}: expected a wrong-kind refusal, got {other}"),
        }
        assert!(refused.to_string().contains(expected));
    }
}

#[test]
fn a_table_stating_a_version_this_reader_does_not_know_is_refused_by_the_version() {
    let stated = units::TABLE_VERSION + 1;
    match refusal(&format!("version = {stated}\n")) {
        TableRefusal::UnknownVersion { found } => assert_eq!(found, stated),
        other => panic!("expected an unknown-version refusal, got {other}"),
    }
}

#[test]
fn an_entry_missing_any_of_the_fields_its_kind_needs_is_refused_by_that_field() {
    let cases: &[(String, &str)] = &[
        (document(&["symbol = \"Hz\""]), "unit[0].dimension"),
        (
            document(&["symbol = \"Hz\"\ndimension = \"frequency\""]),
            "unit[0].factor_is",
        ),
        (
            document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
                 factor_is = \"exact-by-definition\""]),
            "unit[0].factor",
        ),
        (
            document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
                 factor_is = \"measured\"\nsource = \"somewhere\""]),
            "unit[0].factor",
        ),
        (
            document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
                 factor_is = \"measured\"\nfactor = 1.0"]),
            "unit[0].source",
        ),
        (
            document(&["symbol = \"u\"\ndimension = \"mass\"\n\
                 factor_is = \"an-adjusted-constant\""]),
            "unit[0].constant",
        ),
    ];
    for (text, field) in cases {
        match refusal(text) {
            TableRefusal::Missing { field: said } => assert_eq!(&said, field),
            other => panic!("{field}: expected a missing-field refusal, got {other}"),
        }
    }
}

#[test]
fn a_factor_written_as_a_whole_number_reads_as_the_number_it_is() {
    // A factor of one is written `1` as readily as `1.0`, and refusing the
    // first would be a refusal about notation rather than about the number. It
    // also has to go on counting as the base of its dimension, which is derived
    // from the factor being exactly one.
    let text = document(&[
        "symbol = \"Hz\"\ndimension = \"frequency\"\n\
         factor_is = \"exact-by-definition\"\nfactor = 1",
        "symbol = \"kHz\"\ndimension = \"frequency\"\n\
         factor_is = \"exact-by-definition\"\nfactor = 1000",
    ]);
    let table = accepted(&text);
    assert_eq!(table.base_of("frequency"), Some("Hz"));
    match table.into_base(FILE, FIELD, "kHz", "frequency", 2.0) {
        Ok(found) => assert_eq!(found, 2000.0),
        Err(refused) => panic!("expected a conversion: {refused}"),
    }
}

#[test]
fn a_factor_kind_that_is_none_of_the_three_is_refused() {
    let text = document(&[
        "symbol = \"Hz\"\ndimension = \"frequency\"\nfactor_is = \"whatever\"\nfactor = 1.0",
    ]);
    match refusal(&text) {
        TableRefusal::UnknownFactorKind { found } => assert_eq!(found, "whatever"),
        other => panic!("expected an unknown-kind refusal, got {other}"),
    }
}

#[test]
fn a_factor_that_is_not_a_finite_positive_number_is_refused_by_its_symbol() {
    let written = |factor: &str| {
        document(&[&format!(
            "symbol = \"Hz\"\ndimension = \"frequency\"\n\
             factor_is = \"exact-by-definition\"\nfactor = {factor}"
        )])
    };
    match refusal(&written("nan")) {
        TableRefusal::FactorIsNotANumber { symbol } => assert_eq!(symbol, "Hz"),
        other => panic!("expected a not-a-number refusal, got {other}"),
    }
    // Zero and below are refused together, because a conversion multiplies by
    // the factor and the round trip divides back by it.
    for factor in ["0.0", "-1.0"] {
        match refusal(&written(factor)) {
            TableRefusal::FactorIsNotPositive { symbol, found } => {
                assert_eq!(symbol, "Hz");
                assert!(found <= 0.0);
            }
            other => panic!("{factor}: expected a not-positive refusal, got {other}"),
        }
    }
}

#[test]
fn one_symbol_written_by_two_entries_is_refused_by_that_symbol() {
    let text = document(&[&base("Hz", "frequency"), &base("Hz", "time")]);
    match refusal(&text) {
        TableRefusal::SymbolStatedTwice { symbol } => assert_eq!(symbol, "Hz"),
        other => panic!("expected a stated-twice refusal, got {other}"),
    }
}

#[test]
fn a_dimension_with_no_unit_of_factor_one_is_refused_by_that_dimension() {
    let text = document(&[
        &base("Hz", "frequency"),
        "symbol = \"pm\"\ndimension = \"length\"\n\
         factor_is = \"exact-by-definition\"\nfactor = 1e-12",
    ]);
    match refusal(&text) {
        TableRefusal::DimensionWithNoBase { dimension } => assert_eq!(dimension, "length"),
        other => panic!("expected a no-base refusal, got {other}"),
    }
}

#[test]
fn a_dimension_with_two_units_of_factor_one_is_refused_naming_both() {
    let text = document(&[&base("Bq", "frequency"), &base("Hz", "frequency")]);
    match refusal(&text) {
        TableRefusal::DimensionWithTwoBases {
            dimension,
            first,
            second,
        } => {
            assert_eq!(dimension, "frequency");
            // Walked in the order symbols sort, so the two are named the same
            // way whichever order the file wrote them in.
            assert_eq!(first, "Bq");
            assert_eq!(second, "Hz");
        }
        other => panic!("expected a two-bases refusal, got {other}"),
    }
}

#[test]
fn every_refusal_prints_and_debugs() {
    // Each is produced rather than constructed, so a variant that stops being
    // reachable fails here.
    let stated = units::TABLE_VERSION + 1;
    let table_refusals = [
        refusal("this is not a document = = ="),
        refusal(""),
        refusal("version = \"1\"\n"),
        refusal(&format!("version = {stated}\n")),
        refusal(&document(&[
            "symbol = \"Hz\"\ndimension = \"frequency\"\nfactor_is = \"whatever\"",
        ])),
        refusal(&document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
             factor_is = \"exact-by-definition\"\nfactor = nan"])),
        refusal(&document(&["symbol = \"Hz\"\ndimension = \"frequency\"\n\
             factor_is = \"exact-by-definition\"\nfactor = 0.0"])),
        refusal(&document(&[&base("Hz", "frequency"), &base("Hz", "time")])),
        refusal(&document(&[
            &base("Hz", "frequency"),
            "symbol = \"pm\"\ndimension = \"length\"\n\
             factor_is = \"exact-by-definition\"\nfactor = 1e-12",
        ])),
        refusal(&document(&[
            &base("Bq", "frequency"),
            &base("Hz", "frequency"),
        ])),
    ];
    for refused in &table_refusals {
        assert!(!refused.to_string().is_empty());
        assert!(!format!("{refused:?}").is_empty());
        let carried: &dyn std::error::Error = refused;
        assert!(carried.source().is_none());
    }

    let table = accepted(&fixture());
    let value_refusals = [
        refused("MHZ", "frequency"),
        refused("MHz", "luminance"),
        refused("MHz", "mass"),
        match table.into_base(FILE, FIELD, "u", "mass", 1.0) {
            Err(found) => found,
            Ok(_) => panic!("expected a refusal"),
        },
    ];
    for refused in &value_refusals {
        assert!(refused.to_string().starts_with(FILE));
        assert!(!format!("{refused:?}").is_empty());
        assert!(!refused.refusal.to_string().is_empty());
        let carried: &dyn std::error::Error = refused;
        // The refusal is reachable through the chain rather than only in the
        // text, and it is where the chain ends.
        let inner = carried.source().expect("the refusal is the source");
        assert!(inner.source().is_none());
    }
}
