//! The input datum record, and what a set of them counts to.
//!
//! The decision is #4 and the record is #30. One datum is one file, so a
//! correction to one number is a one-file diff a reviewer can check against a
//! publication in a minute, and a per-value citation is possible at all.
//!
//! A record file is a provenance block with a table of its own beside it. The
//! block is read from the same bytes by its own reader rather than from a
//! subtable this module unpacks, so there is one place in the workspace that
//! decides what a block is, and a datum inherits every refusal that reader
//! already makes.
//!
//! The record's own fields are in a `[datum]` table rather than at the top
//! level, which is where the correlation coefficient puts its two. The reason
//! is a name: the block already owns the top-level `uncertainty`, and a datum
//! has an uncertainty of its own, so a top-level field of that name would be
//! two different things called uncertainty in one file, and the format would
//! refuse the file for a reason that says nothing about either of them.
//!
//! Two fields are here because they are easy to get wrong and expensive to get
//! wrong quietly. The uncertainty says whether it is absolute or relative,
//! since the publications use both and a silent convention is a factor of a
//! million waiting to happen. And a datum says whether it enters the adjustment
//! or is present for comparison only, because a table in a publication often
//! lists both, and a value that was not part of the fit must not become part of
//! ours by sitting in the same directory.
//!
//! The file name is not decoration either. A record is named for the identifier
//! it carries, `<identifier>.toml` and nothing else, and a disagreement between
//! the two is refused. #30 asks for that refusal and no issue on the board said
//! what the mapping was, so it is fixed here, in the code that refuses a
//! departure from it, rather than left to whoever writes the first directory.
//!
//! What this module does not judge, said plainly because a reader of a green
//! run would otherwise assume it did. It does not know what a unit means or
//! whether it matches the quantity, which is the table in #34. It does not know
//! that the quantity exists, which is the record in #32. It does not resolve
//! `superseded_by` against anything, so a record naming a replacement that is
//! nowhere in the tree passes here; where that is checked is a question for the
//! command that walks the whole tree, #36, because a run that loads one target
//! adjustment can legitimately not hold the record that replaced a datum.
//!
//! An uncertainty of zero is not refused here. It is not a shape defect, it is
//! a covariance that cannot be factorised, and #56 refuses a covariance that is
//! not positive definite by name. A rule implemented in two places can be
//! deleted from one and stay green through the other, which is the failure a
//! proof of a guard exists to prevent.
//!
//! The readers below are this module's own rather than the provenance module's,
//! which is what the correlation coefficient does too. A shared reader would
//! have to name the field in the vocabulary of one record and would then be
//! naming it wrongly for the other, and a refusal that names the wrong field is
//! the kind a reader stops trusting.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use toml::Value;
use toml::value::Table;

use crate::provenance::{self, Provenance};

/// The table the record's own fields sit in.
const OWN: &str = "datum";

/// The extension a record file carries.
const EXTENSION: &str = ".toml";

/// Whether the quoted uncertainty is the number itself or a fraction of the
/// value.
///
/// Kept as the publication wrote it rather than converted on the way in, so a
/// reader of the file sees what the table said. The conversion is
/// [`Datum::absolute_uncertainty`], in one place, rather than at every site
/// that wants a number.
#[derive(Debug)]
pub enum UncertaintyForm {
    /// The uncertainty is in the same unit as the value.
    Absolute,
    /// The uncertainty is a fraction of the value.
    Relative,
}

/// One input datum, as one file holds it.
///
/// The fields are private and the only way to make one is [`read`], so a datum
/// that exists has already passed every refusal that can be made about a single
/// record.
#[derive(Debug)]
pub struct Datum {
    identifier: String,
    quantity: String,
    unit: String,
    value: f64,
    uncertainty: f64,
    uncertainty_form: UncertaintyForm,
    enters_the_adjustment: bool,
    superseded_by: Option<String>,
    provenance: Provenance,
}

impl Datum {
    /// The identifier the observational equations refer to this datum by.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The quantity the datum measures.
    #[must_use]
    pub fn quantity(&self) -> &str {
        &self.quantity
    }

    /// The unit the value is written in, as the file writes it.
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// The value, in the unit the file names.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The uncertainty, in the form the file states.
    #[must_use]
    pub fn uncertainty(&self) -> f64 {
        self.uncertainty
    }

    /// Which of the two forms the uncertainty is written in.
    #[must_use]
    pub fn uncertainty_form(&self) -> &UncertaintyForm {
        &self.uncertainty_form
    }

    /// The uncertainty in the unit of the value, whichever form the file used.
    ///
    /// The absolute value of the value rather than the value, because a
    /// relative uncertainty of a negative quantity is a fraction of its
    /// magnitude and a signed one would be an uncertainty below zero.
    #[must_use]
    pub fn absolute_uncertainty(&self) -> f64 {
        match self.uncertainty_form {
            UncertaintyForm::Absolute => self.uncertainty,
            UncertaintyForm::Relative => self.uncertainty * self.value.abs(),
        }
    }

    /// Whether the datum enters the fit, or is present for comparison only.
    #[must_use]
    pub fn enters_the_adjustment(&self) -> bool {
        self.enters_the_adjustment
    }

    /// The identifier of the datum that replaced this one, where one did.
    ///
    /// An identifier is never reused, so a superseded datum keeps its file and
    /// says what replaced it. An adjustment run last year has to keep meaning
    /// what it meant.
    #[must_use]
    pub fn superseded_by(&self) -> Option<&str> {
        self.superseded_by.as_deref()
    }

    /// Where the number came from.
    #[must_use]
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The name the file carrying this record has to have.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}{EXTENSION}", self.identifier)
    }
}

/// Why a datum, or a set of them, was refused.
///
/// Every variant carries the field or the identifier it is about. The file it
/// is about is carried by [`Refused`], once, rather than by each variant.
#[derive(Debug)]
pub enum Refusal {
    /// The bytes are not a document this format can read at all.
    ///
    /// A number written with digits the format cannot read arrives here rather
    /// than at one of the two below, because the document stops being readable
    /// before any field is looked at.
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
    /// The provenance block the record carries was refused.
    Provenance(provenance::Refusal),
    /// A number field holds something that is not a finite number.
    ///
    /// The format reads `nan` and `inf` as numbers, and neither is a
    /// measurement. A value of either would reach the solve and come out the
    /// other side having quietly emptied a row of the fit.
    NotAFiniteNumber {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What the file said.
        found: f64,
    },
    /// The uncertainty is below zero.
    NegativeUncertainty {
        /// What the file said.
        found: f64,
    },
    /// The uncertainty form is neither of the two.
    UnknownUncertaintyForm {
        /// What the file said.
        found: String,
    },
    /// The record says it was replaced by itself.
    SupersededByItself {
        /// The identifier, written once.
        identifier: String,
    },
    /// The file is not named for the identifier the record carries.
    FileNameDisagrees {
        /// The name the file has to have.
        expected: String,
    },
    /// Two files in the set carry one identifier.
    ///
    /// Not a property of a single record, so it is refused where a set is
    /// counted rather than where a file is read. An identifier is what an
    /// observational equation names a datum by, and two data answering to one
    /// name is a fit whose rows nobody can tell apart.
    IdentifierStatedTwice {
        /// The identifier both files carry.
        identifier: String,
        /// The other file carrying it.
        also_in: String,
    },
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
            Self::Provenance(refused) => {
                write!(out, "the record's provenance block was refused: {refused}")
            }
            Self::NotAFiniteNumber { field, found } => {
                write!(
                    out,
                    "the record's {field} is {found}, which is not a number"
                )
            }
            Self::NegativeUncertainty { found } => write!(
                out,
                "the uncertainty {found} is below zero, and an uncertainty is a width"
            ),
            Self::UnknownUncertaintyForm { found } => write!(
                out,
                "the uncertainty form {found} is neither absolute nor relative"
            ),
            Self::SupersededByItself { identifier } => write!(
                out,
                "the record says {identifier} was replaced by itself, and an \
                 identifier is never reused"
            ),
            Self::FileNameDisagrees { expected } => write!(
                out,
                "a record is named for the identifier it carries, so this file \
                 has to be called {expected}"
            ),
            Self::IdentifierStatedTwice {
                identifier,
                also_in,
            } => write!(
                out,
                "the identifier {identifier} is also carried by {also_in}, and \
                 an equation naming it could mean either"
            ),
        }
    }
}

impl std::error::Error for Refusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provenance(refused) => Some(refused),
            _ => None,
        }
    }
}

/// A refusal, and the file it is about.
///
/// The file is named here rather than in every variant above, so a refusal
/// gains the file by being returned rather than by each site remembering to
/// carry it.
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
fn addressed(key: &str) -> String {
    format!("{OWN}.{key}")
}

/// The record's own table, or a refusal naming it.
fn own(record: &Table) -> Result<&Table, Refusal> {
    match record.get(OWN) {
        None => Err(Refusal::Missing {
            field: OWN.to_owned(),
        }),
        Some(Value::Table(found)) => Ok(found),
        Some(_) => Err(Refusal::WrongKind {
            field: OWN.to_owned(),
            expected: "a table",
        }),
    }
}

/// The string at `key`.
fn text(within: &Table, key: &str) -> Result<String, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: addressed(key),
        }),
        Some(Value::String(found)) => Ok(found.clone()),
        Some(_) => Err(Refusal::WrongKind {
            field: addressed(key),
            expected: "a string",
        }),
    }
}

/// The string at `key`, where the field is allowed to be absent.
fn optional_text(within: &Table, key: &str) -> Result<Option<String>, Refusal> {
    match within.get(key) {
        None => Ok(None),
        Some(Value::String(found)) => Ok(Some(found.clone())),
        Some(_) => Err(Refusal::WrongKind {
            field: addressed(key),
            expected: "a string",
        }),
    }
}

/// The number at `key`, written either way.
///
/// A value of exactly one is written `1` as readily as `1.0`, and refusing the
/// first would be a refusal about notation rather than about the number.
fn number(within: &Table, key: &str) -> Result<f64, Refusal> {
    let found = match within.get(key) {
        None => {
            return Err(Refusal::Missing {
                field: addressed(key),
            });
        }
        Some(Value::Float(found)) => *found,
        Some(Value::Integer(found)) => *found as f64,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: addressed(key),
                expected: "a number",
            });
        }
    };
    if !found.is_finite() {
        return Err(Refusal::NotAFiniteNumber {
            field: addressed(key),
            found,
        });
    }
    Ok(found)
}

/// The true or false at `key`.
fn boolean(within: &Table, key: &str) -> Result<bool, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: addressed(key),
        }),
        Some(Value::Boolean(found)) => Ok(*found),
        Some(_) => Err(Refusal::WrongKind {
            field: addressed(key),
            expected: "true or false",
        }),
    }
}

impl UncertaintyForm {
    fn read(found: &str) -> Result<Self, Refusal> {
        match found {
            "absolute" => Ok(Self::Absolute),
            "relative" => Ok(Self::Relative),
            _ => Err(Refusal::UnknownUncertaintyForm {
                found: found.to_owned(),
            }),
        }
    }
}

/// The name of the file, given a path or a name.
///
/// A caller that knows a datum by its path says so, and the refusal it gets
/// back names the path it used. What the identifier is compared against is the
/// last segment of it, because a record is named by its file and placed by its
/// directory.
fn name_of(file: &str) -> &str {
    match file.rsplit_once(['/', '\\']) {
        Some((_, name)) => name,
        None => file,
    }
}

/// Read one datum record, or refuse it by name.
///
/// `file` is the file the bytes came from, as the caller knows it. Nothing here
/// opens it: the reader is given the name and the bytes, so every rule this
/// module carries is provable with no directory in existence, and the loader
/// passes in what it read.
///
/// The record's own fields are read before the block. A file whose uncertainty
/// is below zero is refused for that rather than for a missing digitisation
/// date further down, because the first is what the reader has to fix and the
/// second is what they would otherwise see.
pub fn read(file: &str, document: &str) -> Result<Datum, Refused> {
    fields(file, document).map_err(|refusal| Refused {
        file: file.to_owned(),
        refusal,
    })
}

/// Everything [`read`] does, before the file name is attached to the refusal.
fn fields(file: &str, document: &str) -> Result<Datum, Refusal> {
    let record: Table =
        toml::from_str(document).map_err(|error| Refusal::Unreadable(error.to_string()))?;
    let own = own(&record)?;

    let identifier = text(own, "identifier")?;
    let expected = format!("{identifier}{EXTENSION}");
    if name_of(file) != expected {
        return Err(Refusal::FileNameDisagrees { expected });
    }

    let quantity = text(own, "quantity")?;
    let unit = text(own, "unit")?;
    let value = number(own, "value")?;

    let uncertainty = number(own, "uncertainty")?;
    if uncertainty < 0.0 {
        return Err(Refusal::NegativeUncertainty { found: uncertainty });
    }
    let uncertainty_form = UncertaintyForm::read(&text(own, "uncertainty_form")?)?;

    let enters_the_adjustment = boolean(own, "enters_the_adjustment")?;

    let superseded_by = optional_text(own, "superseded_by")?;
    if superseded_by.as_deref() == Some(identifier.as_str()) {
        return Err(Refusal::SupersededByItself { identifier });
    }

    let provenance = provenance::read(document).map_err(Refusal::Provenance)?;

    Ok(Datum {
        identifier,
        quantity,
        unit,
        value,
        uncertainty,
        uncertainty_form,
        enters_the_adjustment,
        superseded_by,
        provenance,
    })
}

/// What a run says about the input data it read.
///
/// A count and a list rather than a status word. A reader who has just changed
/// one file learns more from four numbers about the set than from the word
/// that says the set was accepted.
#[derive(Debug)]
pub struct Census {
    /// How many files were read.
    pub files: usize,
    /// Every identifier in the set, which is what the coefficients are
    /// assembled against.
    pub identifiers: BTreeSet<String>,
    /// How many of the data enter the fit.
    pub entering_the_adjustment: usize,
    /// How many are present for comparison only.
    pub for_comparison_only: usize,
    /// How many say they were replaced by another datum.
    pub superseded: usize,
}

impl fmt::Display for Census {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Assembled and written once rather than a line at a time. Each `?`
        // between two writes is an error branch nothing reaches, and an
        // unreachable branch is a hole under the coverage floor rather than a
        // safeguard.
        let lines = [
            format!("input data over {} files", self.files),
            format!(
                "  entering the adjustment  {}",
                self.entering_the_adjustment
            ),
            format!("  for comparison only      {}", self.for_comparison_only),
            format!("  superseded               {}", self.superseded),
            String::new(),
            "A datum present for comparison only is not in the fit. Nothing here has".to_owned(),
            "judged a unit or a quantity, so a count is not a validation.".to_owned(),
        ];
        write!(out, "{}", lines.join("\n"))
    }
}

/// Count a set of records, refusing an identifier that two files carry.
///
/// Each entry is the file a record was read from and the record. Uniqueness is
/// a property of a set rather than of a file, so it is refused here and in no
/// other place: implemented twice, it could be deleted from one and stay green
/// through the other.
pub fn census(records: &[(String, Datum)]) -> Result<Census, Refused> {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut entering = 0usize;
    let mut superseded = 0usize;

    for (file, datum) in records {
        if let Some(also_in) = seen.get(datum.identifier()) {
            return Err(Refused {
                file: file.clone(),
                refusal: Refusal::IdentifierStatedTwice {
                    identifier: datum.identifier().to_owned(),
                    also_in: also_in.clone(),
                },
            });
        }
        seen.insert(datum.identifier().to_owned(), file.clone());
        if datum.enters_the_adjustment() {
            entering += 1;
        }
        if datum.superseded_by().is_some() {
            superseded += 1;
        }
    }

    Ok(Census {
        files: records.len(),
        identifiers: seen.into_keys().collect(),
        entering_the_adjustment: entering,
        for_comparison_only: records.len() - entering,
        superseded,
    })
}
