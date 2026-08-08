//! The correlation coefficient record, and the matrix assembled from a set of
//! them.
//!
//! The decision is #5 and the record is #31. A coefficient is an asserted fact
//! with a source, so it is a record naming the two data and the value, and the
//! matrix is built at load time. Zero is the absence of a record rather than a
//! stored zero, which is the whole reason this is not a dense file: a stored
//! zero and a pair nobody ever wrote about are different statements, and the
//! printed tables cannot tell them apart.
//!
//! A record file is a provenance block with two more fields at its top level.
//! The block is read from the same bytes by its own reader rather than from a
//! subtable this module unpacks, so there is one place in the workspace that
//! decides what a block is, and a coefficient inherits every refusal that
//! reader already makes.
//!
//! Whether a coefficient was stated by the publication or worked out by it from
//! other coefficients is the block's own `measurement.origin` field, and no
//! second field is added here. Two fields saying one thing can disagree, and
//! the one that is wrong is then a coin toss. The field is worded for a
//! measured quantity, which reads oddly over a coefficient; the wording belongs
//! to the shared block in #7 and is not changed from here.
//!
//! What this module cannot do is know which data exist. The set of identifiers
//! is given to [`assemble`], because the data are #30's records and this crate
//! assembles the matrix against whatever was loaded rather than against a list
//! it keeps of its own.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use toml::Value;
use toml::value::Table;

use crate::provenance::{self, Origin, Provenance};

/// One asserted correlation coefficient, as one file holds it.
///
/// The fields are private and the only way to make one is [`read`], so a
/// coefficient that exists has already passed the refusals that can be made
/// about a single record. [`assemble`] relies on that: it does no arithmetic
/// that a self-naming or out-of-range record could push out of range.
#[derive(Debug)]
pub struct Coefficient {
    between: [String; 2],
    value: f64,
    provenance: Provenance,
}

impl Coefficient {
    /// The two data the coefficient is between, as the file names them.
    #[must_use]
    pub fn between(&self) -> &[String; 2] {
        &self.between
    }

    /// The coefficient, inside the closed interval from minus one to one.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Where the number came from.
    #[must_use]
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Whether the publication worked this coefficient out from others rather
    /// than stating it.
    ///
    /// Worth carrying per record because it is a place two adjustments can
    /// differ: a derived coefficient is somebody's expansion of a statement,
    /// and a later reader may disagree with the expansion without disagreeing
    /// with the publication.
    #[must_use]
    pub fn derived_by_the_publication(&self) -> bool {
        matches!(self.provenance.origin, Origin::DerivedByThePublication)
    }
}

/// Why a coefficient, or a set of them, was refused.
///
/// Every variant carries the identifier or the value it is about. A refusal
/// that says a set is wrong without saying which record is a refusal somebody
/// answers by deleting records until it stops.
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
    /// The provenance block the record carries was refused.
    Provenance(provenance::Refusal),
    /// The value is outside the closed interval from minus one to one.
    ///
    /// A value that is not a number is refused here too, because it is not
    /// inside the interval either and there is no second thing to say about it.
    OutsideTheInterval {
        /// What the file said.
        found: f64,
    },
    /// The record names one datum as both of its two.
    OneDatumTwice {
        /// The identifier, written once.
        named: String,
    },
    /// The record names a datum that is not in the set being assembled.
    UnknownDatum {
        /// The identifier the record named.
        named: String,
    },
    /// Two records assert the same pair, in one order or in both.
    ///
    /// A refusal rather than a last-one-wins. The two can disagree, and a
    /// silent choice between them is the kind of defect nobody finds.
    PairStatedTwice {
        /// The first identifier of the pair, in sorted order.
        first: String,
        /// The second.
        second: String,
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
            Self::OutsideTheInterval { found } => write!(
                out,
                "the coefficient {found} is outside the closed interval from minus one to one"
            ),
            Self::OneDatumTwice { named } => write!(
                out,
                "the record names {named} as both of its two data, and a datum's \
                 correlation with itself is one by definition rather than by assertion"
            ),
            Self::UnknownDatum { named } => write!(
                out,
                "the record names {named}, which is not a datum of the set being assembled"
            ),
            Self::PairStatedTwice { first, second } => write!(
                out,
                "the pair {first} and {second} is asserted more than once, in one \
                 order or in both, and the assertions can disagree"
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

/// The pair of identifiers at `key`, or a refusal naming it.
fn pair(within: &Table, key: &str) -> Result<[String; 2], Refusal> {
    let items = match within.get(key) {
        None => {
            return Err(Refusal::Missing {
                field: key.to_owned(),
            });
        }
        Some(Value::Array(found)) => found,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: key.to_owned(),
                expected: "an array of two identifiers",
            });
        }
    };
    let [Value::String(first), Value::String(second)] = items.as_slice() else {
        return Err(Refusal::WrongKind {
            field: key.to_owned(),
            expected: "an array of two identifiers",
        });
    };
    Ok([first.clone(), second.clone()])
}

/// The number at `key`, written either way.
///
/// A coefficient of exactly one is written `1` as readily as `1.0`, and the
/// interval is closed, so refusing the first would be a refusal about notation
/// rather than about the value.
fn number(within: &Table, key: &str) -> Result<f64, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: key.to_owned(),
        }),
        Some(Value::Float(found)) => Ok(*found),
        Some(Value::Integer(found)) => Ok(*found as f64),
        Some(_) => Err(Refusal::WrongKind {
            field: key.to_owned(),
            expected: "a number",
        }),
    }
}

/// Read one coefficient record, or refuse it by name.
///
/// The record's own two fields are read before the block. A file whose
/// coefficient is outside the interval is refused for that rather than for a
/// missing digitisation date further down, because the first is what the reader
/// has to fix and the second is what they would see.
pub fn read(document: &str) -> Result<Coefficient, Refusal> {
    let record: Table =
        toml::from_str(document).map_err(|error| Refusal::Unreadable(error.to_string()))?;

    let between = pair(&record, "between")?;
    if between[0] == between[1] {
        return Err(Refusal::OneDatumTwice {
            named: between[0].clone(),
        });
    }

    let value = number(&record, "value")?;
    if !(-1.0..=1.0).contains(&value) {
        return Err(Refusal::OutsideTheInterval { found: value });
    }

    let provenance = provenance::read(document).map_err(Refusal::Provenance)?;

    Ok(Coefficient {
        between,
        value,
        provenance,
    })
}

/// The pair, in sorted order, so that the two orders of one assertion are one
/// key.
fn ordered(first: &str, second: &str) -> (String, String) {
    if first <= second {
        (first.to_owned(), second.to_owned())
    } else {
        (second.to_owned(), first.to_owned())
    }
}

/// The assembled matrix. Every pair nobody asserted is zero, and no zero is
/// stored.
#[derive(Debug)]
pub struct Correlations {
    asserted: BTreeMap<(String, String), f64>,
}

impl Correlations {
    /// The coefficient between two data.
    ///
    /// One for a datum with itself, which is a definition and not an assertion.
    /// Zero for a pair no record names, which is the absence this shape exists
    /// to keep distinguishable from an asserted zero.
    #[must_use]
    pub fn between(&self, first: &str, second: &str) -> f64 {
        if first == second {
            return 1.0;
        }
        self.asserted
            .get(&ordered(first, second))
            .copied()
            .unwrap_or(0.0)
    }

    /// How many coefficients were asserted.
    #[must_use]
    pub fn asserted(&self) -> usize {
        self.asserted.len()
    }
}

/// What a run says about the coefficients it read.
///
/// The last field is the one worth printing. A datum correlated with nothing is
/// either genuinely independent or a gap in the digitisation, and a run that
/// prints only the first two numbers lets a reader assume the first.
#[derive(Debug)]
pub struct Census {
    /// How many data the matrix was assembled over.
    pub data: usize,
    /// How many coefficients were asserted.
    pub asserted: usize,
    /// How many of those the publication derived rather than stated.
    pub derived: usize,
    /// How many pairs were left at zero.
    pub pairs_at_zero: usize,
    /// The data appearing in no asserted coefficient at all.
    pub uncorrelated: Vec<String>,
}

impl fmt::Display for Census {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Assembled and written once rather than a line at a time. Each `?`
        // between two writes is an error branch nothing reaches, and an
        // unreachable branch is a hole under the coverage floor rather than a
        // safeguard.
        let mut lines = vec![
            format!("correlation coefficients over {} data", self.data),
            format!("  asserted                        {}", self.asserted),
            format!("  of those derived, not stated    {}", self.derived),
            format!("  pairs left at zero              {}", self.pairs_at_zero),
            format!(
                "  data in no asserted coefficient {}",
                self.uncorrelated.len()
            ),
        ];
        lines.extend(
            self.uncorrelated
                .iter()
                .map(|identifier| format!("    {identifier}")),
        );
        lines.push(String::new());
        lines.push(
            "A datum in no asserted coefficient is either independent or a gap in".to_owned(),
        );
        lines.push("the digitisation, and nothing here knows which.".to_owned());
        write!(out, "{}", lines.join("\n"))
    }
}

/// Assemble the matrix from a set of records over a set of data.
///
/// `data` is every datum identifier the run loaded. A record naming anything
/// else is refused rather than ignored: an identifier renamed on one side only
/// is the mistake this catches, and dropping the record would turn a rename
/// into a coefficient that silently became zero.
pub fn assemble(
    records: &[Coefficient],
    data: &BTreeSet<String>,
) -> Result<(Correlations, Census), Refusal> {
    let mut asserted: BTreeMap<(String, String), f64> = BTreeMap::new();
    let mut named: BTreeSet<String> = BTreeSet::new();
    let mut derived = 0usize;

    for record in records {
        for identifier in record.between() {
            if !data.contains(identifier) {
                return Err(Refusal::UnknownDatum {
                    named: identifier.clone(),
                });
            }
        }
        let key = ordered(&record.between[0], &record.between[1]);
        if asserted.contains_key(&key) {
            return Err(Refusal::PairStatedTwice {
                first: key.0,
                second: key.1,
            });
        }
        if record.derived_by_the_publication() {
            derived += 1;
        }
        named.insert(record.between[0].clone());
        named.insert(record.between[1].clone());
        asserted.insert(key, record.value);
    }

    // Every pair of two distinct data, which is what the matrix has room for
    // above its diagonal. The count is derived rather than stored, so it cannot
    // disagree with the set it is about.
    let pairs = data.len() * data.len().saturating_sub(1) / 2;
    let census = Census {
        data: data.len(),
        asserted: asserted.len(),
        derived,
        pairs_at_zero: pairs - asserted.len(),
        uncorrelated: data.difference(&named).cloned().collect(),
    };

    Ok((Correlations { asserted }, census))
}
