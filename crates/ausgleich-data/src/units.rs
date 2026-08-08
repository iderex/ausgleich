//! The unit table, and the dimensional check that runs when a value loads.
//!
//! The decision is #6 and the table is #34. Every value carries a unit drawn
//! from a table in this repository, an unknown unit is refused rather than
//! guessed at, and a value whose unit does not match the dimension its quantity
//! declares is refused rather than converted.
//!
//! The failure this prevents is the one that does not crash. A wrong conversion
//! inside a mixture of frequencies, masses, ratios and quantities whose
//! definition changed in 2019 produces a plausible wrong answer, and load is
//! the only cheap place to catch it. There is no mode where the mismatch
//! becomes a warning, because a warning in a pipeline is a message nobody
//! reads.
//!
//! The table is data. `../unit-table.toml` is read here at compile time, so the
//! entries this code judges against and the entries a reviewer reads are the
//! same bytes, which is what the method vocabulary next door does and for the
//! same reason. It is data rather than source for a second reason as well: a
//! conversion factor written in Rust is refused by the invariants gate, which
//! reads the shape of a number and cannot tell an exact factor from a measured
//! one.
//!
//! A factor is one of three kinds and the third is why this is not a map of
//! symbol to number. A unit whose factor is a quantity the adjustment itself
//! produces has no factor to write down, and a table carrying one would feed an
//! output of the fit back in as an input to the loader, silently, with every
//! run afterwards conditioned on it. Such a unit is representable and
//! converting a value in it is refused by name. The unified atomic mass unit is
//! the case; the table says why.
//!
//! What a dimension is here, and what it is not, is argued in the table itself
//! rather than restated here.
//!
//! The check takes the file and the field from its caller rather than opening
//! anything, so every rule below is provable with no data directory in
//! existence and the loader passes in what it read. Nothing in this module
//! touches the filesystem.

use std::collections::BTreeMap;
use std::fmt;

use toml::Value;
use toml::value::Table;

/// The table version this crate writes, and the only one it reads.
pub const TABLE_VERSION: i64 = 1;

/// The table, as committed.
const COMMITTED: &str = include_str!("../unit-table.toml");

/// The factor of a unit that is the base of its dimension.
const BASE: f64 = 1.0;

/// Where a factor comes from, and whether there is one at all.
#[derive(Debug)]
pub enum Factor {
    /// The factor follows from a definition.
    ExactByDefinition(f64),
    /// The factor is a published number, with the source it was read from.
    Measured {
        /// The factor.
        value: f64,
        /// Where it was read.
        source: String,
    },
    /// There is no factor. The number is a quantity the adjustment produces.
    AnAdjustedConstant {
        /// The identifier of the constant that would be the factor.
        constant: String,
    },
}

impl Factor {
    /// The number, where the kind has one.
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::ExactByDefinition(value) | Self::Measured { value, .. } => Some(*value),
            Self::AnAdjustedConstant { .. } => None,
        }
    }
}

/// One entry of the table.
#[derive(Debug)]
pub struct Unit {
    symbol: String,
    dimension: String,
    factor: Factor,
}

impl Unit {
    /// The symbol a datum writes.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// The dimension the unit belongs to.
    #[must_use]
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    /// The factor to the base unit of that dimension.
    #[must_use]
    pub fn factor(&self) -> &Factor {
        &self.factor
    }
}

/// Why the table itself was refused.
///
/// Separate from [`Refusal`], which is about a value being loaded. One is a
/// defect in this repository and the other is a defect in an input file, and a
/// reader should never have to work out which they are looking at.
#[derive(Debug)]
pub enum TableRefusal {
    /// The bytes are not a document this format can read at all.
    Unreadable(String),
    /// A required field is absent.
    Missing {
        /// The field, written as it is addressed in the file.
        field: String,
    },
    /// A required field is present and is the wrong kind of value.
    WrongKind {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What was expected there.
        expected: &'static str,
    },
    /// The table states a version this reader does not know.
    UnknownVersion {
        /// The version the file stated.
        found: i64,
    },
    /// The kind of factor is not one of the three.
    UnknownFactorKind {
        /// What the file said.
        found: String,
    },
    /// A factor is not a finite number.
    FactorIsNotANumber {
        /// The unit it belongs to.
        symbol: String,
    },
    /// A factor is zero or below.
    ///
    /// A conversion multiplies by it and the round trip divides by it, so
    /// neither is a conversion at all.
    FactorIsNotPositive {
        /// The unit it belongs to.
        symbol: String,
        /// What the file said.
        found: f64,
    },
    /// Two entries carry one symbol.
    SymbolStatedTwice {
        /// The symbol both entries carry.
        symbol: String,
    },
    /// A dimension has no unit whose factor is exactly one.
    ///
    /// Converting to base units means nothing without one, so the dimension
    /// would be a set of factors pointing at no place.
    DimensionWithNoBase {
        /// The dimension.
        dimension: String,
    },
    /// A dimension has two units whose factor is exactly one.
    DimensionWithTwoBases {
        /// The dimension.
        dimension: String,
        /// The first of the two, in the order symbols sort.
        first: String,
        /// The second.
        second: String,
    },
}

impl fmt::Display for TableRefusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(detail) => {
                write!(out, "the table is not readable as a document: {detail}")
            }
            Self::Missing { field } => write!(out, "the table has no {field}"),
            Self::WrongKind { field, expected } => {
                write!(out, "the table's {field} is not {expected}")
            }
            Self::UnknownVersion { found } => write!(
                out,
                "the table states version {found}, and this reader reads version {TABLE_VERSION}"
            ),
            Self::UnknownFactorKind { found } => write!(
                out,
                "the factor kind {found} is not exact-by-definition, measured \
                 or an-adjusted-constant"
            ),
            Self::FactorIsNotANumber { symbol } => {
                write!(out, "the factor of {symbol} is not a finite number")
            }
            Self::FactorIsNotPositive { symbol, found } => write!(
                out,
                "the factor of {symbol} is {found}, and a conversion multiplies \
                 by it and divides back by it"
            ),
            Self::SymbolStatedTwice { symbol } => {
                write!(out, "the symbol {symbol} is written by two entries")
            }
            Self::DimensionWithNoBase { dimension } => write!(
                out,
                "no unit of {dimension} has a factor of one, so converting to \
                 base units means nothing there"
            ),
            Self::DimensionWithTwoBases {
                dimension,
                first,
                second,
            } => write!(
                out,
                "{first} and {second} both have a factor of one, so {dimension} \
                 has two base units"
            ),
        }
    }
}

impl std::error::Error for TableRefusal {}

/// Why a value was refused against the table.
#[derive(Debug)]
pub enum Refusal {
    /// The unit is not in the table.
    ///
    /// Refused rather than guessed at. A table that quietly accepts a typo will
    /// eventually accept the wrong power of ten.
    UnknownUnit {
        /// What the file said.
        found: String,
    },
    /// The dimension the quantity declares is in no entry of the table.
    UnknownDimension {
        /// What was declared.
        found: String,
    },
    /// The unit belongs to another dimension than the quantity declares.
    DimensionMismatch {
        /// The unit the file wrote.
        unit: String,
        /// The dimension that unit belongs to.
        belongs_to: String,
        /// The dimension the quantity declares.
        declared: String,
    },
    /// The unit has no factor, because its factor is an adjusted constant.
    NotAConversion {
        /// The unit the file wrote.
        unit: String,
        /// The constant the conversion would need.
        constant: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUnit { found } => write!(
                out,
                "the unit {found} is in no entry of the table, and a unit is \
                 refused rather than guessed at"
            ),
            Self::UnknownDimension { found } => {
                write!(out, "the dimension {found} is in no entry of the table")
            }
            Self::DimensionMismatch {
                unit,
                belongs_to,
                declared,
            } => write!(
                out,
                "the unit {unit} is a unit of {belongs_to}, and the quantity is \
                 declared as {declared}"
            ),
            Self::NotAConversion { unit, constant } => write!(
                out,
                "converting {unit} needs {constant}, which the adjustment \
                 produces rather than defines, so there is no factor to apply"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// A refusal, and the file and field it is about.
#[derive(Debug)]
pub struct Refused {
    /// The file, as the caller named it.
    pub file: String,
    /// The field within it, as the caller addressed it.
    pub field: String,
    /// What was refused.
    pub refusal: Refusal,
}

impl fmt::Display for Refused {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}, {}: {}", self.file, self.field, self.refusal)
    }
}

impl std::error::Error for Refused {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.refusal)
    }
}

/// The field of the entry at `index`, as a refusal addresses it.
fn at(index: usize, key: &str) -> String {
    format!("unit[{index}].{key}")
}

/// The string at `key` of an entry.
fn text(within: &Table, index: usize, key: &str) -> Result<String, TableRefusal> {
    match within.get(key) {
        None => Err(TableRefusal::Missing {
            field: at(index, key),
        }),
        Some(Value::String(found)) => Ok(found.clone()),
        Some(_) => Err(TableRefusal::WrongKind {
            field: at(index, key),
            expected: "a string",
        }),
    }
}

/// The number at `key` of an entry, written either way.
fn number(within: &Table, index: usize, key: &str, symbol: &str) -> Result<f64, TableRefusal> {
    let found = match within.get(key) {
        None => {
            return Err(TableRefusal::Missing {
                field: at(index, key),
            });
        }
        Some(Value::Float(found)) => *found,
        Some(Value::Integer(found)) => *found as f64,
        Some(_) => {
            return Err(TableRefusal::WrongKind {
                field: at(index, key),
                expected: "a number",
            });
        }
    };
    if !found.is_finite() {
        return Err(TableRefusal::FactorIsNotANumber {
            symbol: symbol.to_owned(),
        });
    }
    if found <= 0.0 {
        return Err(TableRefusal::FactorIsNotPositive {
            symbol: symbol.to_owned(),
            found,
        });
    }
    Ok(found)
}

/// The kind of factor an entry declares, with whatever that kind requires.
fn factor(entry: &Table, index: usize, symbol: &str) -> Result<Factor, TableRefusal> {
    let kind = text(entry, index, "factor_is")?;
    match kind.as_str() {
        "exact-by-definition" => Ok(Factor::ExactByDefinition(number(
            entry, index, "factor", symbol,
        )?)),
        "measured" => Ok(Factor::Measured {
            value: number(entry, index, "factor", symbol)?,
            source: text(entry, index, "source")?,
        }),
        "an-adjusted-constant" => Ok(Factor::AnAdjustedConstant {
            constant: text(entry, index, "constant")?,
        }),
        _ => Err(TableRefusal::UnknownFactorKind { found: kind }),
    }
}

/// The unit table, read.
#[derive(Debug)]
pub struct UnitTable {
    version: i64,
    units: BTreeMap<String, Unit>,
    bases: BTreeMap<String, String>,
}

impl UnitTable {
    /// Which version of the shape the table was written under.
    #[must_use]
    pub fn version(&self) -> i64 {
        self.version
    }

    /// Every entry, in the order symbols sort.
    #[must_use]
    pub fn units(&self) -> Vec<&Unit> {
        self.units.values().collect()
    }

    /// The entry for a symbol.
    #[must_use]
    pub fn unit(&self, symbol: &str) -> Option<&Unit> {
        self.units.get(symbol)
    }

    /// Every dimension the table knows, in the order they sort.
    #[must_use]
    pub fn dimensions(&self) -> Vec<&str> {
        self.bases.keys().map(String::as_str).collect()
    }

    /// The base unit of a dimension, which is its unit of factor one.
    #[must_use]
    pub fn base_of(&self, dimension: &str) -> Option<&str> {
        self.bases.get(dimension).map(String::as_str)
    }

    /// Refuse a unit this table does not know, or one that belongs to another
    /// dimension than the quantity declares.
    ///
    /// `file` and `field` are the caller's, and appear in the refusal, so the
    /// person who has to fix it is told where to look.
    pub fn check(
        &self,
        file: &str,
        field: &str,
        symbol: &str,
        declared: &str,
    ) -> Result<(), Refused> {
        let refuse = |refusal| Refused {
            file: file.to_owned(),
            field: field.to_owned(),
            refusal,
        };
        let Some(unit) = self.units.get(symbol) else {
            return Err(refuse(Refusal::UnknownUnit {
                found: symbol.to_owned(),
            }));
        };
        if !self.bases.contains_key(declared) {
            return Err(refuse(Refusal::UnknownDimension {
                found: declared.to_owned(),
            }));
        }
        if unit.dimension != declared {
            return Err(refuse(Refusal::DimensionMismatch {
                unit: symbol.to_owned(),
                belongs_to: unit.dimension.clone(),
                declared: declared.to_owned(),
            }));
        }
        Ok(())
    }

    /// The factor of a unit, or a refusal saying there is none.
    fn factor_of(&self, file: &str, field: &str, symbol: &str) -> Result<f64, Refused> {
        let refuse = |refusal| Refused {
            file: file.to_owned(),
            field: field.to_owned(),
            refusal,
        };
        let Some(unit) = self.units.get(symbol) else {
            return Err(refuse(Refusal::UnknownUnit {
                found: symbol.to_owned(),
            }));
        };
        // Matched on the kind rather than on whether a number came back, so
        // the arm that refuses is the arm that knows what to name. Asking
        // `value()` first and then asking again what kind it was would leave a
        // branch nothing can reach.
        match &unit.factor {
            Factor::ExactByDefinition(value) => Ok(*value),
            Factor::Measured { value, .. } => Ok(*value),
            Factor::AnAdjustedConstant { constant } => Err(refuse(Refusal::NotAConversion {
                unit: symbol.to_owned(),
                constant: constant.clone(),
            })),
        }
    }

    /// A value in `symbol`, converted into the base unit of its dimension.
    ///
    /// The dimensional check runs first, so a value that would be converted by
    /// the wrong factor is refused before any arithmetic happens.
    pub fn into_base(
        &self,
        file: &str,
        field: &str,
        symbol: &str,
        declared: &str,
        value: f64,
    ) -> Result<f64, Refused> {
        self.check(file, field, symbol, declared)?;
        Ok(value * self.factor_of(file, field, symbol)?)
    }

    /// A value in the base unit of a dimension, written back in `symbol`.
    ///
    /// The inverse of [`Self::into_base`], and what the round trip is over.
    pub fn from_base(
        &self,
        file: &str,
        field: &str,
        symbol: &str,
        value: f64,
    ) -> Result<f64, Refused> {
        Ok(value / self.factor_of(file, field, symbol)?)
    }
}

/// Read a unit table, or refuse it by name.
pub fn read(document: &str) -> Result<UnitTable, TableRefusal> {
    let table: Table =
        toml::from_str(document).map_err(|error| TableRefusal::Unreadable(error.to_string()))?;

    let version = match table.get("version") {
        None => {
            return Err(TableRefusal::Missing {
                field: "version".to_owned(),
            });
        }
        Some(Value::Integer(found)) => *found,
        Some(_) => {
            return Err(TableRefusal::WrongKind {
                field: "version".to_owned(),
                expected: "an integer",
            });
        }
    };
    if version != TABLE_VERSION {
        return Err(TableRefusal::UnknownVersion { found: version });
    }

    let entries = match table.get("unit") {
        None => {
            return Err(TableRefusal::Missing {
                field: "unit".to_owned(),
            });
        }
        Some(Value::Array(found)) => found,
        Some(_) => {
            return Err(TableRefusal::WrongKind {
                field: "unit".to_owned(),
                expected: "an array of tables",
            });
        }
    };

    let mut units: BTreeMap<String, Unit> = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let Value::Table(entry) = entry else {
            return Err(TableRefusal::WrongKind {
                field: format!("unit[{index}]"),
                expected: "a table",
            });
        };
        let symbol = text(entry, index, "symbol")?;
        let dimension = text(entry, index, "dimension")?;
        let factor = factor(entry, index, &symbol)?;
        if units.contains_key(&symbol) {
            return Err(TableRefusal::SymbolStatedTwice { symbol });
        }
        units.insert(
            symbol.clone(),
            Unit {
                symbol,
                dimension,
                factor,
            },
        );
    }

    // The base of a dimension is derived rather than declared, so nothing in
    // the file can disagree with the factors it carries. Walked in the order
    // symbols sort, so a table with two bases names the same two either way it
    // was written.
    let mut bases: BTreeMap<String, String> = BTreeMap::new();
    for unit in units.values() {
        if unit.factor.value() != Some(BASE) {
            continue;
        }
        if let Some(first) = bases.get(&unit.dimension) {
            return Err(TableRefusal::DimensionWithTwoBases {
                dimension: unit.dimension.clone(),
                first: first.clone(),
                second: unit.symbol.clone(),
            });
        }
        bases.insert(unit.dimension.clone(), unit.symbol.clone());
    }
    for unit in units.values() {
        if !bases.contains_key(&unit.dimension) {
            return Err(TableRefusal::DimensionWithNoBase {
                dimension: unit.dimension.clone(),
            });
        }
    }

    Ok(UnitTable {
        version,
        units,
        bases,
    })
}

/// The table this repository committed.
///
/// Read rather than cached, so the one call site that would hold a failed parse
/// forever does not exist and every caller sees the same refusal.
pub fn committed() -> Result<UnitTable, TableRefusal> {
    read(COMMITTED)
}
