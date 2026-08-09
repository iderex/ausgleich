//! The record for a quantity the adjustment solves for.
//!
//! The decision is #32. Every adjusted constant gets a file: the identifier the
//! observational equations refer to it by, the symbol the publications print,
//! the unit and the dimension, a starting value with the source it came from,
//! and whether the quantity is exact by definition rather than adjusted.
//!
//! The exact-by-definition field is the reason this record exists rather than a
//! list in source. Under the SI as redefined in 2019 several constants have
//! defined values, so whether a quantity is a parameter or a constant depends on
//! which adjustment is being reproduced. As a field it is one file per
//! adjustment; as a branch in the solver it is a rebuild of the solver for the
//! second target.
//!
//! The starting value carries its source because the fit is nonlinear and is
//! solved by iterating a linearisation. A start taken from the adjustment being
//! reproduced is a legitimate choice and it is also the place a reader would
//! reasonably suspect the answer of having been assumed, so the file says where
//! the number came from and the convergence report shows the path from it.
//!
//! ## The dimension is declared here, and why it is not derived
//!
//! The unit table's check takes a unit and the dimension the quantity declares.
//! Nothing in the tree declared one, so #36 recorded that `validate` could not
//! apply the check to a set and named this record as the plausible home. It is
//! taken here, and the alternative it was weighed against is worth writing down
//! rather than leaving as a thing somebody re-derives.
//!
//! The alternative is to derive the dimension from the unit through the table.
//! It carries the fact once, and it also makes the check unfalsifiable: a
//! dimension read out of the table entry for a unit agrees with that entry by
//! construction, so the refusal could never fire and a guard that cannot fail is
//! not a guard. Declaring both is what gives a wrong unit something to disagree
//! with. A record whose unit was mistyped into another dimension is refused
//! here, which is the mistake somebody actually makes: `T` and `s` are one key
//! apart on the shape of the file, and both are units of something.
//!
//! The cost is the usual cost of stating a thing twice. A correction to one of
//! the two is refused rather than silently carried, which is the direction that
//! stops a run.
//!
//! ## What this module does not judge
//!
//! Said plainly, because a reader of a green run would otherwise assume it did.
//!
//! It does not know that the identifier is one any equation uses, and it does
//! not refuse a quantity marked exact that an equation treats as a parameter.
//! That is a relation between this record and the equation registry, which is
//! #49, and #32's own Done-when carries it as the leg this module cannot reach.
//!
//! It does not resolve the starting value's source against anything. The field
//! is prose a reader checks against a publication, and the only thing refused
//! about it is a file that leaves it empty.
//!
//! It does not count a set. Two files naming one identifier is a property of a
//! directory rather than of a file, and the command that walks a whole tree is
//! #36.
//!
//! The reader is handed the file name, the bytes and the unit table, and opens
//! nothing. Every rule below is therefore provable with no data directory in
//! existence, which is what the two record modules beside it do and for the same
//! reason.

use std::fmt;

use toml::Value;
use toml::value::Table;

use crate::units::{self, UnitTable};

/// The record version this crate writes, and the only one it reads.
///
/// A file written under an older version has to keep loading once a second
/// version exists, so the refusal below names the version it found rather than
/// calling the file malformed.
pub const RECORD_VERSION: i64 = 1;

/// The table the quantity's own fields sit in.
const OWN: &str = "constant";

/// The table the starting value and its source sit in.
///
/// Its own table rather than two fields beside the rest, because the source
/// belongs to the value and a top-level `source` would read as the source of
/// the record.
const START: &str = "starting_value";

/// The extension a record file carries.
const EXTENSION: &str = ".toml";

/// One quantity the adjustment solves for, as one file holds it.
///
/// The fields are private and the only way to make one is [`read`], so a record
/// that exists has already passed every refusal that can be made about a single
/// file.
#[derive(Debug)]
pub struct AdjustedConstant {
    identifier: String,
    symbol: String,
    unit: String,
    dimension: String,
    exact_by_definition: bool,
    starting_value: f64,
    starting_value_source: String,
}

impl AdjustedConstant {
    /// The identifier the observational equations refer to this quantity by.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The symbol, as the publications print it.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// The unit the value is written in, as one symbol of the table.
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// The dimension the quantity is of, as the table names it.
    #[must_use]
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    /// Whether the quantity has a defined value rather than a fitted one.
    #[must_use]
    pub fn exact_by_definition(&self) -> bool {
        self.exact_by_definition
    }

    /// The value the linearisation starts from, in the unit the file names.
    #[must_use]
    pub fn starting_value(&self) -> f64 {
        self.starting_value
    }

    /// Where the starting value came from.
    #[must_use]
    pub fn starting_value_source(&self) -> &str {
        &self.starting_value_source
    }

    /// The name the file carrying this record has to have.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}{EXTENSION}", self.identifier)
    }
}

/// Why a record was refused.
///
/// Every variant carries the field it is about. The file it is about is carried
/// by [`Refused`], once, rather than by each variant.
#[derive(Debug)]
pub enum Refusal {
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
    /// A required string is present and says nothing.
    ///
    /// A field written as an empty string satisfies every reader that asks only
    /// whether the key is there, and tells the next person exactly as much as an
    /// absent one. The starting value's source is the field this is for: #32
    /// asks for a missing source to be refused, and a source of `""` is that
    /// file.
    Blank {
        /// The field, written as it is addressed in the file.
        field: String,
    },
    /// The record states a version this crate does not read.
    UnknownVersion {
        /// The version the file stated.
        found: i64,
    },
    /// A number field holds something that is not a finite number.
    ///
    /// The format reads `nan` and `inf` as numbers, and a linearisation started
    /// at either produces a run whose every output is the same word.
    NotAFiniteNumber {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What the file said.
        found: f64,
    },
    /// The file is not named for the identifier the record carries.
    FileNameDisagrees {
        /// The name the file has to have.
        expected: String,
    },
    /// The unit and the declared dimension were refused by the unit table.
    Unit(units::Refusal),
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(detail) => {
                write!(out, "the record is not readable as a document: {detail}")
            }
            Self::Missing { field } => write!(out, "the record has no {field}"),
            Self::WrongKind { field, expected } => {
                write!(out, "the record's {field} is not {expected}")
            }
            Self::Blank { field } => write!(
                out,
                "the record's {field} is empty, and a field that says nothing is \
                 the field that is missing"
            ),
            Self::UnknownVersion { found } => write!(
                out,
                "the record states version {found}, and this reader reads \
                 version {RECORD_VERSION}"
            ),
            Self::NotAFiniteNumber { field, found } => {
                write!(
                    out,
                    "the record's {field} is {found}, which is not a number"
                )
            }
            Self::FileNameDisagrees { expected } => write!(
                out,
                "a record is named for the identifier it carries, so this file \
                 has to be called {expected}"
            ),
            Self::Unit(refused) => write!(
                out,
                "the record's {} was refused against the unit table: {refused}",
                addressed(OWN, "unit")
            ),
        }
    }
}

impl std::error::Error for Refusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unit(refused) => Some(refused),
            _ => None,
        }
    }
}

/// A refusal, and the file it is about.
#[derive(Debug)]
pub struct Refused {
    /// The file, as the caller named it.
    pub file: String,
    /// What was refused about it.
    pub refusal: Refusal,
}

impl fmt::Display for Refused {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}: {}", self.file, self.refusal)
    }
}

impl std::error::Error for Refused {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.refusal)
    }
}

/// The field, as a refusal addresses it.
fn addressed(table: &str, key: &str) -> String {
    format!("{table}.{key}")
}

/// The subtable at `key`, or a refusal naming it.
fn subtable<'a>(within: &'a Table, key: &str) -> Result<&'a Table, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: key.to_owned(),
        }),
        Some(Value::Table(found)) => Ok(found),
        Some(_) => Err(Refusal::WrongKind {
            field: key.to_owned(),
            expected: "a table",
        }),
    }
}

/// The string at `key`, refused when it is absent or says nothing.
fn text(within: &Table, table: &str, key: &str) -> Result<String, Refusal> {
    let field = addressed(table, key);
    match within.get(key) {
        None => Err(Refusal::Missing { field }),
        Some(Value::String(found)) if found.trim().is_empty() => Err(Refusal::Blank { field }),
        Some(Value::String(found)) => Ok(found.clone()),
        Some(_) => Err(Refusal::WrongKind {
            field,
            expected: "a string",
        }),
    }
}

/// The integer at `key`.
fn integer(within: &Table, field: &str) -> Result<i64, Refusal> {
    match within.get(field) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::Integer(found)) => Ok(*found),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "an integer",
        }),
    }
}

/// The number at `key`, written either way.
///
/// A starting value of exactly one is written `1` as readily as `1.0`, and
/// refusing the first would be a refusal about notation rather than about the
/// number.
fn number(within: &Table, table: &str, key: &str) -> Result<f64, Refusal> {
    let field = addressed(table, key);
    let found = match within.get(key) {
        None => return Err(Refusal::Missing { field }),
        Some(Value::Float(found)) => *found,
        Some(Value::Integer(found)) => *found as f64,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field,
                expected: "a number",
            });
        }
    };
    if !found.is_finite() {
        return Err(Refusal::NotAFiniteNumber { field, found });
    }
    Ok(found)
}

/// The true or false at `key`.
fn boolean(within: &Table, table: &str, key: &str) -> Result<bool, Refusal> {
    let field = addressed(table, key);
    match within.get(key) {
        None => Err(Refusal::Missing { field }),
        Some(Value::Boolean(found)) => Ok(*found),
        Some(_) => Err(Refusal::WrongKind {
            field,
            expected: "true or false",
        }),
    }
}

/// The name of the file, given a path or a name.
///
/// A caller that knows a record by its path says so, and the refusal it gets
/// back names the path it used. What the identifier is compared against is the
/// last segment of it, because a record is named by its file and placed by its
/// directory.
fn name_of(file: &str) -> &str {
    match file.rsplit_once(['/', '\\']) {
        Some((_, name)) => name,
        None => file,
    }
}

/// Read one adjusted constant record, or refuse it by name.
///
/// `file` is the file the bytes came from, as the caller knows it, and `table`
/// is the unit table the unit is judged against. Nothing here opens either: the
/// reader is given the name, the bytes and the table, and the caller passes in
/// what it read.
///
/// The shape of the record is read before the unit is judged, so a file missing
/// its starting value is refused for that rather than for a unit its author was
/// going to correct anyway.
pub fn read(file: &str, document: &str, table: &UnitTable) -> Result<AdjustedConstant, Refused> {
    fields(file, document, table).map_err(|refusal| Refused {
        file: file.to_owned(),
        refusal,
    })
}

/// Everything [`read`] does, before the file name is attached to the refusal.
fn fields(file: &str, document: &str, table: &UnitTable) -> Result<AdjustedConstant, Refusal> {
    let record: Table =
        toml::from_str(document).map_err(|error| Refusal::Unreadable(error.to_string()))?;

    // The version is read before anything else. A file written under a version
    // this reader does not know is refused for that, and not for whichever
    // field the shape moved.
    let version = integer(&record, "version")?;
    if version != RECORD_VERSION {
        return Err(Refusal::UnknownVersion { found: version });
    }

    let own = subtable(&record, OWN)?;
    let start = subtable(&record, START)?;

    let identifier = text(own, OWN, "identifier")?;
    let expected = format!("{identifier}{EXTENSION}");
    if name_of(file) != expected {
        return Err(Refusal::FileNameDisagrees { expected });
    }

    let symbol = text(own, OWN, "symbol")?;
    let unit = text(own, OWN, "unit")?;
    let dimension = text(own, OWN, "dimension")?;
    let exact_by_definition = boolean(own, OWN, "exact_by_definition")?;

    let starting_value = number(start, START, "value")?;
    let starting_value_source = text(start, START, "source")?;

    table
        .check(file, &addressed(OWN, "unit"), &unit, &dimension)
        .map_err(|refused| Refusal::Unit(refused.refusal))?;

    Ok(AdjustedConstant {
        identifier,
        symbol,
        unit,
        dimension,
        exact_by_definition,
        starting_value,
        starting_value_source,
    })
}
