//! The provenance block a published measurement carries with it.
//!
//! The field list is #7 and the version field is #33. The block says where a
//! number came from and nothing about what the number is, which is why the same
//! shape serves a board recording measurements of quantities this adjustment
//! never touches. A block carrying the quantity as well would be this
//! repository's shape wearing a shared name.
//!
//! Every refusal names the field it is about. A validator that says a file is
//! wrong without saying where is a validator whose output somebody stops
//! reading.
//!
//! The method is checked against a vocabulary in a committed file rather than
//! against a list in this source, so adding a term is an edit a reviewer sees
//! as an edit to the vocabulary.

use std::fmt;

use toml::Value;
use toml::value::Table;

/// The block version this crate writes, and the only one it reads.
///
/// A file written under an older version has to keep loading once a second
/// version exists, so the refusal below names the version it found rather than
/// saying the file is malformed.
pub const BLOCK_VERSION: i64 = 1;

/// The vocabulary, as committed. Read at compile time so that the terms this
/// crate judges against and the terms a reviewer reads are the same bytes.
const VOCABULARY: &str = include_str!("../method-vocabulary.txt");

/// The one line of the vocabulary that is not a term.
const VERSION_PREFIX: &str = "version ";

/// Which publication a value was read out of.
#[derive(Debug)]
pub struct Publication {
    /// The authors, in the order the publication prints them.
    pub authors: Vec<String>,
    /// The title.
    pub title: String,
    /// The journal or the report series.
    pub venue: String,
    /// The year of publication.
    pub year: i64,
    /// The digital object identifier.
    pub doi: String,
}

/// Where in the publication the value sits.
#[derive(Debug)]
pub struct ReadFrom {
    /// The table or the equation, in the publication's own numbering.
    pub locator: String,
    /// The page it is printed on.
    pub page: i64,
}

/// Whether the publication measured the value or worked it out from others.
///
/// The distinction is not bookkeeping. A value the publication derived is a
/// correlation waiting to be asserted, so a record that hides the difference
/// hides a coefficient.
#[derive(Debug)]
pub enum Origin {
    /// The publication measured it.
    Measured,
    /// The publication derived it from other values.
    DerivedByThePublication,
}

/// What kind of uncertainty the publication quoted.
#[derive(Debug)]
pub enum UncertaintyKind {
    /// From the scatter of the publication's own repeated observations.
    Statistical,
    /// From the publication's own budget of effects that do not average away.
    Systematic,
    /// The two taken together, which is what most tables print.
    Combined,
}

/// How the quoted uncertainty is to be read.
#[derive(Debug)]
pub struct Uncertainty {
    /// Which kind the publication quoted.
    pub kind: UncertaintyKind,
    /// The factor the quoted interval already carries.
    pub coverage_factor: f64,
}

/// Who turned the printed value into a file, and who checked them.
#[derive(Debug)]
pub struct Digitisation {
    /// Who read the value out of the publication.
    pub read_by: String,
    /// When they read it.
    pub read_on: String,
    /// Who read it a second time and confirmed the first reading.
    pub confirmed_by: String,
    /// When the second reading was made.
    pub confirmed_on: String,
}

/// The block itself.
#[derive(Debug)]
pub struct Provenance {
    /// Which version of this shape the file was written under.
    pub version: i64,
    /// The publication.
    pub publication: Publication,
    /// The table or equation and its page.
    pub read_from: ReadFrom,
    /// The date the measurement was reported, which is not the publication
    /// date and is often the more useful of the two.
    pub reported: String,
    /// The method, as one term of the committed vocabulary.
    pub method: String,
    /// Whether the publication measured the value or derived it.
    pub origin: Origin,
    /// How the quoted uncertainty is to be read.
    pub uncertainty: Uncertainty,
    /// The digitisation trail.
    pub digitisation: Digitisation,
}

/// Why a block was refused.
///
/// Every variant carries the field or the value it is about, because a refusal
/// a person cannot act on is a refusal they route around.
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
    /// The block states a version this crate does not read.
    UnknownVersion {
        /// The version the file stated.
        found: i64,
    },
    /// The method is not a term of the committed vocabulary.
    MethodOutsideVocabulary {
        /// What the file said.
        found: String,
    },
    /// The uncertainty kind is not one of the three.
    UnknownUncertaintyKind {
        /// What the file said.
        found: String,
    },
    /// The origin is neither of the two.
    UnknownOrigin {
        /// What the file said.
        found: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(detail) => {
                write!(out, "the block is not readable as a document: {detail}")
            }
            Self::Missing { field } => write!(out, "the block has no {field}"),
            Self::WrongKind { field, expected } => {
                write!(out, "the block's {field} is not {expected}")
            }
            Self::UnknownVersion { found } => write!(
                out,
                "the block states version {found}, and this reader reads version {BLOCK_VERSION}"
            ),
            Self::MethodOutsideVocabulary { found } => write!(
                out,
                "the method {found} is not a term of vocabulary version {}",
                vocabulary_version()
            ),
            Self::UnknownUncertaintyKind { found } => write!(
                out,
                "the uncertainty kind {found} is not statistical, systematic or combined"
            ),
            Self::UnknownOrigin { found } => write!(
                out,
                "the origin {found} is neither measured nor derived-by-the-publication"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// The lines of the vocabulary file that say something.
fn vocabulary_entries() -> impl Iterator<Item = &'static str> {
    VOCABULARY
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

/// Which version of the vocabulary this crate carries.
///
/// Stated in the file rather than beside it, so the version and the terms move
/// in one edit and cannot disagree.
pub fn vocabulary_version() -> &'static str {
    let mut stated = "";
    for entry in vocabulary_entries() {
        if let Some(rest) = entry.strip_prefix(VERSION_PREFIX) {
            stated = rest;
        }
    }
    stated
}

/// Every method term, in the order the file lists them.
pub fn methods() -> Vec<&'static str> {
    vocabulary_entries()
        .filter(|entry| !entry.starts_with(VERSION_PREFIX))
        .collect()
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

/// The string at `key`, or a refusal naming `field` as the file addresses it.
fn text(within: &Table, key: &str, field: &str) -> Result<String, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::String(found)) => Ok(found.clone()),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a string",
        }),
    }
}

/// The array of strings at `key`.
fn texts(within: &Table, key: &str, field: &str) -> Result<Vec<String>, Refusal> {
    let items = match within.get(key) {
        None => {
            return Err(Refusal::Missing {
                field: field.to_owned(),
            });
        }
        Some(Value::Array(found)) => found,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: field.to_owned(),
                expected: "an array of strings",
            });
        }
    };
    let mut collected = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::String(found) => collected.push(found.clone()),
            _ => {
                return Err(Refusal::WrongKind {
                    field: field.to_owned(),
                    expected: "an array of strings",
                });
            }
        }
    }
    Ok(collected)
}

/// The integer at `key`.
fn integer(within: &Table, key: &str, field: &str) -> Result<i64, Refusal> {
    match within.get(key) {
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
/// A coverage factor of one is written `1` as readily as `1.0`, and refusing
/// the first would be a refusal about notation rather than about the value.
fn number(within: &Table, key: &str, field: &str) -> Result<f64, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::Float(found)) => Ok(*found),
        Some(Value::Integer(found)) => Ok(*found as f64),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a number",
        }),
    }
}

/// The date at `key`, in the format's own date type rather than as a string.
///
/// The format already refuses a date that is not one, so nothing here has to
/// re-implement that judgement. What is stored is the canonical spelling.
fn date(within: &Table, key: &str, field: &str) -> Result<String, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::Datetime(found)) => Ok(found.to_string()),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a date",
        }),
    }
}

impl UncertaintyKind {
    fn read(found: &str) -> Result<Self, Refusal> {
        match found {
            "statistical" => Ok(Self::Statistical),
            "systematic" => Ok(Self::Systematic),
            "combined" => Ok(Self::Combined),
            _ => Err(Refusal::UnknownUncertaintyKind {
                found: found.to_owned(),
            }),
        }
    }
}

impl Origin {
    fn read(found: &str) -> Result<Self, Refusal> {
        match found {
            "measured" => Ok(Self::Measured),
            "derived-by-the-publication" => Ok(Self::DerivedByThePublication),
            _ => Err(Refusal::UnknownOrigin {
                found: found.to_owned(),
            }),
        }
    }
}

/// Read a provenance block, or refuse it by name.
pub fn read(document: &str) -> Result<Provenance, Refusal> {
    let block: Table =
        toml::from_str(document).map_err(|error| Refusal::Unreadable(error.to_string()))?;

    // The version is read before anything else. A file written under a version
    // this reader does not know is refused for that, and not for whichever
    // field the shape moved.
    let version = integer(&block, "version", "version")?;
    if version != BLOCK_VERSION {
        return Err(Refusal::UnknownVersion { found: version });
    }

    let publication = subtable(&block, "publication")?;
    let read_from = subtable(&block, "read_from")?;
    let measurement = subtable(&block, "measurement")?;
    let uncertainty = subtable(&block, "uncertainty")?;
    let digitisation = subtable(&block, "digitisation")?;

    let method = text(measurement, "method", "measurement.method")?;
    if !methods().contains(&method.as_str()) {
        return Err(Refusal::MethodOutsideVocabulary { found: method });
    }

    Ok(Provenance {
        version,
        publication: Publication {
            authors: texts(publication, "authors", "publication.authors")?,
            title: text(publication, "title", "publication.title")?,
            venue: text(publication, "venue", "publication.venue")?,
            year: integer(publication, "year", "publication.year")?,
            doi: text(publication, "doi", "publication.doi")?,
        },
        read_from: ReadFrom {
            locator: text(read_from, "locator", "read_from.locator")?,
            page: integer(read_from, "page", "read_from.page")?,
        },
        reported: date(measurement, "reported", "measurement.reported")?,
        method,
        origin: Origin::read(&text(measurement, "origin", "measurement.origin")?)?,
        uncertainty: Uncertainty {
            kind: UncertaintyKind::read(&text(uncertainty, "kind", "uncertainty.kind")?)?,
            coverage_factor: number(
                uncertainty,
                "coverage_factor",
                "uncertainty.coverage_factor",
            )?,
        },
        digitisation: Digitisation {
            read_by: text(digitisation, "read_by", "digitisation.read_by")?,
            read_on: date(digitisation, "read_on", "digitisation.read_on")?,
            confirmed_by: text(digitisation, "confirmed_by", "digitisation.confirmed_by")?,
            confirmed_on: date(digitisation, "confirmed_on", "digitisation.confirmed_on")?,
        },
    })
}
