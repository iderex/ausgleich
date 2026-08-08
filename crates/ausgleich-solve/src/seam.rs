//! The two files the seam in #13 is made of: the assembled problem going in,
//! and the result coming back.
//!
//! Anything that wants to fit the same problem differently reads the problem
//! file and writes the result file, and needs nothing else from this workspace.
//! That is what keeps a sampler, or a reimplementation in another language, out
//! of the crate where the reproducibility rule lives, and it is why the means
//! decision for the hierarchical model in #71 could be taken without a library
//! landing here.
//!
//! This module holds the shapes and the text they are written in. It opens no
//! file: a document arrives as a string and leaves as a string, so the crate
//! keeps the property the workspace layout gave it, and the path is opened by
//! the command line. The round-trip test is therefore a test over values and
//! bytes rather than over a directory.
//!
//! The reader is `toml`'s, the same one the record types use, for the reason
//! that crate's manifest gives: a parser this project maintains is a parser
//! #37 owes a fuzz target for. The writer is ours, and it has to be, because
//! the property this seam is judged on is that writing what was read gives the
//! same bytes back. That needs one canonical order and one number format, and a
//! serialiser that is free to reorder or to reformat cannot promise either.
//!
//! Numbers are written in the shortest form that reads back as the same double,
//! which is what `{:?}` on a float gives. Truncating a mantissa here would show
//! up as a wrong constant in the last digits, which is the failure the
//! round-trip property exists to catch.
//!
//! What this module does not judge, said plainly so a green read is not taken
//! for more than it is. It does not know whether the covariance can be
//! factorised; a matrix that is symmetric and not positive definite passes here
//! and is refused by name in #56, and a rule implemented twice can be deleted
//! from one place and stay green through the other. It does not know what a
//! unit means, which is #34. It does not know that the identifiers in the
//! result file are the ones the problem file named, because the two files are
//! read by different programs at different times and nothing here holds both.

use std::collections::BTreeSet;
use std::fmt;

use toml::Value;
use toml::value::Table;

/// What was refused about a document.
///
/// Every variant names the field as the file addresses it, so a refusal can be
/// carried to the line that caused it without the reader holding the document.
#[derive(Debug)]
pub enum Refusal {
    /// The bytes are not a document at all.
    Unreadable(String),
    /// A field the format requires is absent.
    Missing {
        /// The field, written as it is addressed in the file.
        field: String,
    },
    /// A field holds something of the wrong kind.
    WrongKind {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What was expected there.
        expected: &'static str,
    },
    /// A number field holds something that is not a finite number.
    ///
    /// The format reads `nan` and `inf` as numbers. Either one in a design
    /// matrix empties a row of the fit and leaves the run looking well.
    NotAFiniteNumber {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What the file said.
        found: f64,
    },
    /// A string field carries a character that is not printable.
    ///
    /// The writer puts these strings back between quotes with only a quote and
    /// a backslash escaped, so a control character would come out as bytes that
    /// no longer read back. Refusing it on the way in is what lets the writer
    /// be total.
    ControlCharacter {
        /// The field, written as it is addressed in the file.
        field: String,
    },
    /// A list the format requires an entry in is empty.
    NoRows {
        /// The list, written as it is addressed in the file.
        field: String,
    },
    /// One identifier is carried by two entries.
    ///
    /// A parameter or an observation is addressed by its identifier, so two
    /// entries answering to one name is a fit whose rows nobody can tell apart.
    StatedTwice {
        /// The list the repeat is in.
        field: String,
        /// The identifier both entries carry.
        identifier: String,
    },
    /// A matrix has a number of rows the rest of the document does not allow.
    WrongRowCount {
        /// The matrix, written as it is addressed in the file.
        field: String,
        /// The count the rest of the document fixes.
        expected: usize,
        /// The count the file has.
        found: usize,
    },
    /// A row of a matrix has a length the rest of the document does not allow.
    WrongColumnCount {
        /// The row, written as it is addressed in the file.
        field: String,
        /// The count the rest of the document fixes.
        expected: usize,
        /// The count the file has.
        found: usize,
    },
    /// A covariance disagrees with itself across the diagonal.
    ///
    /// A covariance is symmetric by construction. One that is not says the
    /// document was assembled by something that lost track of which index was
    /// which, and every number in it is then in doubt.
    NotSymmetric {
        /// The matrix, written as it is addressed in the file.
        field: String,
        /// The row of the pair that disagrees.
        row: usize,
        /// The column of the pair that disagrees.
        column: usize,
    },
    /// A count that cannot be below zero is.
    CountBelowZero {
        /// The field, written as it is addressed in the file.
        field: String,
        /// What the file said.
        found: i64,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(detail) => {
                write!(out, "the file is not readable as a document: {detail}")
            }
            Self::Missing { field } => write!(out, "the file has no {field}"),
            Self::WrongKind { field, expected } => {
                write!(out, "the file's {field} is not {expected}")
            }
            Self::NotAFiniteNumber { field, found } => {
                write!(out, "the file's {field} is {found}, which is not a number")
            }
            Self::ControlCharacter { field } => write!(
                out,
                "the file's {field} carries a character that is not printable, \
                 and a seam file is text a person can read"
            ),
            Self::NoRows { field } => write!(
                out,
                "the file lists no {field}, and a problem with none of those is \
                 not a problem"
            ),
            Self::StatedTwice { field, identifier } => write!(
                out,
                "two entries of {field} are called {identifier}, and a row named \
                 twice is a row nobody can address"
            ),
            Self::WrongRowCount {
                field,
                expected,
                found,
            } => write!(
                out,
                "{field} has {found} rows where the rest of the file fixes {expected}"
            ),
            Self::WrongColumnCount {
                field,
                expected,
                found,
            } => write!(
                out,
                "{field} has {found} entries where the rest of the file fixes {expected}"
            ),
            Self::NotSymmetric { field, row, column } => write!(
                out,
                "{field} disagrees with itself at row {row} and column {column}, \
                 and a covariance is symmetric by construction"
            ),
            Self::CountBelowZero { field, found } => {
                write!(
                    out,
                    "the file's {field} is {found}, and a number of iterations \
                     is never below zero"
                )
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// What produced a file, carried by both files in the seam.
///
/// The fields are #3's. This module holds them and never fills them in: what a
/// run knows about its own input set is #35, and what it knows about itself is
/// the command line's. A field with nothing to put in it is written as an empty
/// string rather than left out, so a reader never has to tell an absent field
/// from an unknown one.
#[derive(Debug)]
pub struct Header {
    input_set: String,
    input_set_hash: String,
    code_version: String,
    code_commit: String,
    toolchain: String,
    command_line: String,
}

impl Header {
    /// The name of the input set the problem was assembled from.
    #[must_use]
    pub fn input_set(&self) -> &str {
        &self.input_set
    }

    /// The hash of that input set, as #35 defines it.
    #[must_use]
    pub fn input_set_hash(&self) -> &str {
        &self.input_set_hash
    }

    /// The version of the code that wrote the file.
    #[must_use]
    pub fn code_version(&self) -> &str {
        &self.code_version
    }

    /// The commit of the code that wrote the file.
    #[must_use]
    pub fn code_commit(&self) -> &str {
        &self.code_commit
    }

    /// The toolchain that built the code that wrote the file.
    #[must_use]
    pub fn toolchain(&self) -> &str {
        &self.toolchain
    }

    /// The command line that produced the file.
    #[must_use]
    pub fn command_line(&self) -> &str {
        &self.command_line
    }
}

/// One adjusted constant, as the problem file names it.
#[derive(Debug)]
pub struct Parameter {
    identifier: String,
    unit: String,
}

impl Parameter {
    /// The identifier the design matrix's columns are in the order of.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The unit the parameter is carried in.
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }
}

/// One entry of the observation vector.
#[derive(Debug)]
pub struct Observation {
    identifier: String,
    value: f64,
}

impl Observation {
    /// The identifier of the datum this row came from.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The observed value, in the unit the input set carried.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// The assembled problem, as one file holds it.
///
/// The only way to make one is [`read_problem`], so a problem that exists has
/// already passed every refusal the format makes.
#[derive(Debug)]
pub struct Problem {
    header: Header,
    parameters: Vec<Parameter>,
    observations: Vec<Observation>,
    covariance: Vec<Vec<f64>>,
    design: Vec<Vec<f64>>,
}

impl Problem {
    /// What produced the file.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The adjusted constants, in the order the design matrix's columns are in.
    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// The observation vector, in the order the rows are in.
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }

    /// The covariance of the observations, row by row.
    #[must_use]
    pub fn covariance(&self) -> &[Vec<f64>] {
        &self.covariance
    }

    /// The design matrix, row by row.
    #[must_use]
    pub fn design(&self) -> &[Vec<f64>] {
        &self.design
    }
}

/// One adjusted constant, as the result file states it.
#[derive(Debug)]
pub struct Estimate {
    identifier: String,
    value: f64,
}

impl Estimate {
    /// The identifier, which is the problem file's.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The fitted value, in the unit the problem file gave the parameter.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// What one observation was left with.
#[derive(Debug)]
pub struct Residual {
    identifier: String,
    normalised: f64,
}

impl Residual {
    /// The identifier of the observation this residual belongs to.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The residual divided by the uncertainty of its own row.
    #[must_use]
    pub fn normalised(&self) -> f64 {
        self.normalised
    }
}

/// How the fit that produced the result stopped.
#[derive(Debug)]
pub struct Convergence {
    iterations: i64,
    converged: bool,
    criterion: String,
    final_step: f64,
}

impl Convergence {
    /// How many linearisations were taken.
    #[must_use]
    pub fn iterations(&self) -> i64 {
        self.iterations
    }

    /// Whether the criterion was met, rather than the budget being spent.
    #[must_use]
    pub fn converged(&self) -> bool {
        self.converged
    }

    /// The criterion, in the words the run states it in.
    #[must_use]
    pub fn criterion(&self) -> &str {
        &self.criterion
    }

    /// The size of the last step taken.
    #[must_use]
    pub fn final_step(&self) -> f64 {
        self.final_step
    }
}

/// The result of a fit, as one file holds it.
///
/// It is called this rather than a result because the language already has a
/// type of that name and the seam is not worth the confusion. The file is the
/// result file of #13.
#[derive(Debug)]
pub struct Solution {
    header: Header,
    parameters: Vec<Estimate>,
    covariance: Vec<Vec<f64>>,
    residuals: Vec<Residual>,
    convergence: Convergence,
}

impl Solution {
    /// What produced the file.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The fitted parameters, in the order the covariance is in.
    #[must_use]
    pub fn parameters(&self) -> &[Estimate] {
        &self.parameters
    }

    /// The covariance of the fitted parameters, row by row.
    #[must_use]
    pub fn covariance(&self) -> &[Vec<f64>] {
        &self.covariance
    }

    /// What each observation was left with.
    #[must_use]
    pub fn residuals(&self) -> &[Residual] {
        &self.residuals
    }

    /// How the fit stopped.
    #[must_use]
    pub fn convergence(&self) -> &Convergence {
        &self.convergence
    }
}

/// The document as one table, or a refusal saying it is not a document.
fn document(text: &str) -> Result<Table, Refusal> {
    toml::from_str(text).map_err(|error| Refusal::Unreadable(error.to_string()))
}

/// The table at `key`, addressed as `field` in a refusal.
///
/// Every reader below takes the key it looks up and the address a refusal
/// should name separately, because the two differ the moment a table is nested.
/// Rewriting the address afterwards was the other way to do it, and it needs an
/// arm for refusals that cannot arrive, which is an arm no test can reach.
fn table_at<'a>(within: &'a Table, key: &str, field: &str) -> Result<&'a Table, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::Table(found)) => Ok(found),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a table",
        }),
    }
}

/// The array at `key`.
fn array_at<'a>(within: &'a Table, key: &str, field: &str) -> Result<&'a [Value], Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::Array(found)) => Ok(found),
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a list",
        }),
    }
}

/// The string at `key`, refused if it carries anything unprintable.
fn text(within: &Table, key: &str, field: &str) -> Result<String, Refusal> {
    match within.get(key) {
        None => Err(Refusal::Missing {
            field: field.to_owned(),
        }),
        Some(Value::String(found)) => {
            if found.chars().any(char::is_control) {
                return Err(Refusal::ControlCharacter {
                    field: field.to_owned(),
                });
            }
            Ok(found.clone())
        }
        Some(_) => Err(Refusal::WrongKind {
            field: field.to_owned(),
            expected: "a string",
        }),
    }
}

/// One number, written either way, refused if it is not finite.
///
/// A value of exactly one is written `1` as readily as `1.0`, and refusing the
/// first would be a refusal about notation rather than about the number.
fn cell(found: Option<&Value>, field: &str) -> Result<f64, Refusal> {
    let number = match found {
        None => {
            return Err(Refusal::Missing {
                field: field.to_owned(),
            });
        }
        Some(Value::Float(found)) => *found,
        #[expect(
            clippy::cast_precision_loss,
            reason = "an integer past the exact range of a double is a number \
                      the file should have written as a float, and reading it \
                      to the nearest double is what every other reader of this \
                      format does"
        )]
        Some(Value::Integer(found)) => *found as f64,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: field.to_owned(),
                expected: "a number",
            });
        }
    };
    if number.is_finite() {
        Ok(number)
    } else {
        Err(Refusal::NotAFiniteNumber {
            field: field.to_owned(),
            found: number,
        })
    }
}

/// The number at `key`.
fn number(within: &Table, key: &str, field: &str) -> Result<f64, Refusal> {
    cell(within.get(key), field)
}

/// The list of tables at `key`, refused if it is empty or holds anything else.
fn rows<'a>(within: &'a Table, key: &str) -> Result<Vec<&'a Table>, Refusal> {
    let found = array_at(within, key, key)?;
    if found.is_empty() {
        return Err(Refusal::NoRows {
            field: key.to_owned(),
        });
    }
    let mut out = Vec::with_capacity(found.len());
    for (index, entry) in found.iter().enumerate() {
        match entry {
            Value::Table(row) => out.push(row),
            _ => {
                return Err(Refusal::WrongKind {
                    field: addressed(key, index),
                    expected: "a table",
                });
            }
        }
    }
    Ok(out)
}

/// One entry of a list, as a refusal addresses it.
fn addressed(field: &str, index: usize) -> String {
    format!("{field}[{index}]")
}

/// One field of one entry of a list, as a refusal addresses it.
fn addressed_field(field: &str, index: usize, key: &str) -> String {
    format!("{field}[{index}].{key}")
}

/// Refuse a list whose identifiers are not all different.
fn all_different(identifiers: &[&str], field: &str) -> Result<(), Refusal> {
    let mut seen = BTreeSet::new();
    for identifier in identifiers {
        if !seen.insert(*identifier) {
            return Err(Refusal::StatedTwice {
                field: field.to_owned(),
                identifier: (*identifier).to_owned(),
            });
        }
    }
    Ok(())
}

/// A matrix of the stated shape, or a refusal naming the shape it has.
fn matrix(
    within: &Table,
    key: &str,
    field: &str,
    expected_rows: usize,
    expected_columns: usize,
) -> Result<Vec<Vec<f64>>, Refusal> {
    let found = array_at(within, key, field)?;
    if found.len() != expected_rows {
        return Err(Refusal::WrongRowCount {
            field: field.to_owned(),
            expected: expected_rows,
            found: found.len(),
        });
    }
    let mut out = Vec::with_capacity(found.len());
    for (index, entry) in found.iter().enumerate() {
        let Value::Array(row) = entry else {
            return Err(Refusal::WrongKind {
                field: addressed(field, index),
                expected: "a list",
            });
        };
        if row.len() != expected_columns {
            return Err(Refusal::WrongColumnCount {
                field: addressed(field, index),
                expected: expected_columns,
                found: row.len(),
            });
        }
        let mut numbers = Vec::with_capacity(row.len());
        for (column, value) in row.iter().enumerate() {
            numbers.push(cell(
                Some(value),
                &addressed(&addressed(field, index), column),
            )?);
        }
        out.push(numbers);
    }
    Ok(out)
}

/// Refuse a matrix that disagrees with itself across the diagonal.
///
/// The two numbers are compared exactly and not within a tolerance. Both are
/// read from the file rather than computed here, so a document that means them
/// to be equal writes them equal, and a tolerance would be a number somebody
/// picked standing between a reader and a file that says two different things.
fn symmetric(values: &[Vec<f64>], field: &str) -> Result<(), Refusal> {
    for (row, entries) in values.iter().enumerate() {
        for (column, value) in entries.iter().enumerate().take(row) {
            if *value != values[column][row] {
                return Err(Refusal::NotSymmetric {
                    field: field.to_owned(),
                    row,
                    column,
                });
            }
        }
    }
    Ok(())
}

/// The header both files carry.
fn header(within: &Table) -> Result<Header, Refusal> {
    let block = table_at(within, "header", "header")?;
    Ok(Header {
        input_set: text(block, "input_set", "header.input_set")?,
        input_set_hash: text(block, "input_set_hash", "header.input_set_hash")?,
        code_version: text(block, "code_version", "header.code_version")?,
        code_commit: text(block, "code_commit", "header.code_commit")?,
        toolchain: text(block, "toolchain", "header.toolchain")?,
        command_line: text(block, "command_line", "header.command_line")?,
    })
}

/// Read a problem file.
///
/// # Errors
///
/// Returns the first thing the document is refused for.
pub fn read_problem(text_of_file: &str) -> Result<Problem, Refusal> {
    let whole = document(text_of_file)?;
    let head = header(&whole)?;

    let mut parameters = Vec::new();
    for (index, row) in rows(&whole, "parameter")?.into_iter().enumerate() {
        parameters.push(Parameter {
            identifier: text(
                row,
                "identifier",
                &addressed_field("parameter", index, "identifier"),
            )?,
            unit: text(row, "unit", &addressed_field("parameter", index, "unit"))?,
        });
    }
    let names: Vec<&str> = parameters
        .iter()
        .map(|one| one.identifier.as_str())
        .collect();
    all_different(&names, "parameter")?;

    let mut observations = Vec::new();
    for (index, row) in rows(&whole, "observation")?.into_iter().enumerate() {
        observations.push(Observation {
            identifier: text(
                row,
                "identifier",
                &addressed_field("observation", index, "identifier"),
            )?,
            value: number(
                row,
                "value",
                &addressed_field("observation", index, "value"),
            )?,
        });
    }
    let names: Vec<&str> = observations
        .iter()
        .map(|one| one.identifier.as_str())
        .collect();
    all_different(&names, "observation")?;

    let block = table_at(&whole, "problem", "problem")?;
    let covariance = matrix(
        block,
        "covariance",
        "problem.covariance",
        observations.len(),
        observations.len(),
    )?;
    symmetric(&covariance, "problem.covariance")?;
    let design = matrix(
        block,
        "design",
        "problem.design",
        observations.len(),
        parameters.len(),
    )?;

    Ok(Problem {
        header: head,
        parameters,
        observations,
        covariance,
        design,
    })
}

/// Read a result file.
///
/// # Errors
///
/// Returns the first thing the document is refused for.
pub fn read_solution(text_of_file: &str) -> Result<Solution, Refusal> {
    let whole = document(text_of_file)?;
    let head = header(&whole)?;

    let mut parameters = Vec::new();
    for (index, row) in rows(&whole, "parameter")?.into_iter().enumerate() {
        parameters.push(Estimate {
            identifier: text(
                row,
                "identifier",
                &addressed_field("parameter", index, "identifier"),
            )?,
            value: number(row, "value", &addressed_field("parameter", index, "value"))?,
        });
    }
    let names: Vec<&str> = parameters
        .iter()
        .map(|one| one.identifier.as_str())
        .collect();
    all_different(&names, "parameter")?;

    let mut residuals = Vec::new();
    for (index, row) in rows(&whole, "residual")?.into_iter().enumerate() {
        residuals.push(Residual {
            identifier: text(
                row,
                "identifier",
                &addressed_field("residual", index, "identifier"),
            )?,
            normalised: number(
                row,
                "normalised",
                &addressed_field("residual", index, "normalised"),
            )?,
        });
    }
    let names: Vec<&str> = residuals
        .iter()
        .map(|one| one.identifier.as_str())
        .collect();
    all_different(&names, "residual")?;

    let block = table_at(&whole, "result", "result")?;
    let covariance = matrix(
        block,
        "covariance",
        "result.covariance",
        parameters.len(),
        parameters.len(),
    )?;
    symmetric(&covariance, "result.covariance")?;

    let block = table_at(&whole, "convergence", "convergence")?;
    let iterations = match block.get("iterations") {
        None => {
            return Err(Refusal::Missing {
                field: "convergence.iterations".to_owned(),
            });
        }
        Some(Value::Integer(found)) => *found,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: "convergence.iterations".to_owned(),
                expected: "a whole number",
            });
        }
    };
    if iterations < 0 {
        return Err(Refusal::CountBelowZero {
            field: "convergence.iterations".to_owned(),
            found: iterations,
        });
    }
    let converged = match block.get("converged") {
        None => {
            return Err(Refusal::Missing {
                field: "convergence.converged".to_owned(),
            });
        }
        Some(Value::Boolean(found)) => *found,
        Some(_) => {
            return Err(Refusal::WrongKind {
                field: "convergence.converged".to_owned(),
                expected: "true or false",
            });
        }
    };
    let convergence = Convergence {
        iterations,
        converged,
        criterion: text(block, "criterion", "convergence.criterion")?,
        final_step: number(block, "final_step", "convergence.final_step")?,
    };

    Ok(Solution {
        header: head,
        parameters,
        covariance,
        residuals,
        convergence,
    })
}

/// One string, between quotes, with the two characters that would end it early
/// escaped.
///
/// Nothing else needs an escape, because a string that reached here carries no
/// control character: [`text`] refused it.
fn quoted(value: &str, out: &mut String) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(character),
        }
    }
    out.push('"');
}

/// One key and its string value, on a line of its own.
fn wrote_text(key: &str, value: &str, out: &mut String) {
    out.push_str(key);
    out.push_str(" = ");
    quoted(value, out);
    out.push('\n');
}

/// One key and its number, on a line of its own.
///
/// The shortest form that reads back as the same double, which is what the
/// debug formatting of a float gives. It always carries a point or an exponent,
/// so the format reads it back as a float rather than as a whole number.
fn wrote_number(key: &str, value: f64, out: &mut String) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&format!("{value:?}"));
    out.push('\n');
}

/// A matrix, one row to a line.
///
/// Every row ends in a comma, the last one included, because the format allows
/// it and a writer with no special case for the last row is a writer with one
/// fewer thing to get wrong.
fn wrote_matrix(key: &str, values: &[Vec<f64>], out: &mut String) {
    out.push_str(key);
    out.push_str(" = [\n");
    for row in values {
        out.push_str("  [");
        for (column, value) in row.iter().enumerate() {
            if column > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{value:?}"));
        }
        out.push_str("],\n");
    }
    out.push_str("]\n");
}

/// The header, as both files write it.
fn wrote_header(value: &Header, out: &mut String) {
    out.push_str("[header]\n");
    wrote_text("input_set", &value.input_set, out);
    wrote_text("input_set_hash", &value.input_set_hash, out);
    wrote_text("code_version", &value.code_version, out);
    wrote_text("code_commit", &value.code_commit, out);
    wrote_text("toolchain", &value.toolchain, out);
    wrote_text("command_line", &value.command_line, out);
}

impl fmt::Display for Problem {
    /// The canonical document for this problem.
    ///
    /// One order and one number format, so that writing what was read gives the
    /// same bytes back and two runs that assembled the same problem produce the
    /// same file. That is #3's rule reaching the seam.
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text_of_file = String::new();
        wrote_header(&self.header, &mut text_of_file);
        for parameter in &self.parameters {
            text_of_file.push_str("\n[[parameter]]\n");
            wrote_text("identifier", &parameter.identifier, &mut text_of_file);
            wrote_text("unit", &parameter.unit, &mut text_of_file);
        }
        for observation in &self.observations {
            text_of_file.push_str("\n[[observation]]\n");
            wrote_text("identifier", &observation.identifier, &mut text_of_file);
            wrote_number("value", observation.value, &mut text_of_file);
        }
        text_of_file.push_str("\n[problem]\n");
        wrote_matrix("covariance", &self.covariance, &mut text_of_file);
        wrote_matrix("design", &self.design, &mut text_of_file);
        out.write_str(&text_of_file)
    }
}

impl fmt::Display for Solution {
    /// The canonical document for this result, on the same terms as
    /// [`Problem`]'s.
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text_of_file = String::new();
        wrote_header(&self.header, &mut text_of_file);
        for parameter in &self.parameters {
            text_of_file.push_str("\n[[parameter]]\n");
            wrote_text("identifier", &parameter.identifier, &mut text_of_file);
            wrote_number("value", parameter.value, &mut text_of_file);
        }
        for residual in &self.residuals {
            text_of_file.push_str("\n[[residual]]\n");
            wrote_text("identifier", &residual.identifier, &mut text_of_file);
            wrote_number("normalised", residual.normalised, &mut text_of_file);
        }
        text_of_file.push_str("\n[result]\n");
        wrote_matrix("covariance", &self.covariance, &mut text_of_file);
        text_of_file.push_str("\n[convergence]\n");
        text_of_file.push_str(&format!("iterations = {}\n", self.convergence.iterations));
        text_of_file.push_str(&format!("converged = {}\n", self.convergence.converged));
        wrote_text("criterion", &self.convergence.criterion, &mut text_of_file);
        wrote_number("final_step", self.convergence.final_step, &mut text_of_file);
        out.write_str(&text_of_file)
    }
}
