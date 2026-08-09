//! Factor the observation covariance and whiten the problem with it, #56.
//!
//! The decision this implements is #2: the covariance is factored and the
//! problem is whitened, and the normal-equations matrix is never formed.
//! Forming it squares the condition number, and this problem is ill-conditioned
//! by construction, so half the available digits would be spent before the
//! solve began.
//!
//! What arrives is a [`Problem`], which is to say a document that has already
//! been refused for everything the format can refuse it for: the covariance is
//! square, it is the size of the observation vector, it agrees with itself
//! across the diagonal exactly, and the design matrix has one row per
//! observation and one column per parameter. So the only thing left for this
//! module to refuse is the one thing the shape cannot show, which is whether
//! the numbers can hold together as a covariance at all.
//!
//! That refusal is the substance of #56 and it is a refusal rather than a
//! repair. No nudging of the diagonal, no shrinkage toward a diagonal matrix,
//! no dropping of the coefficient that caused it. A published table that cannot
//! be assembled into a valid covariance is a finding about the table, and the
//! finding is the interesting output.
//!
//! Nothing here has a tolerance. A pivot is accepted when it is greater than
//! zero and refused otherwise, which is the definition rather than an
//! approximation of it, and it means there is no number in this file somebody
//! could widen to make a bad table pass.
//!
//! The comparison is written as one that has to succeed rather than as a
//! negated one, and the shape is the point. A value that cannot be compared at
//! all takes the refusing side by construction, so the case needs no second
//! test beside the first that somebody could delete on its own. Nothing in the
//! suite reaches that case: the seam refuses a covariance cell that is not
//! finite, so every pivot here is built out of finite numbers, and the path
//! that could still produce one is an overflow inside the factorisation, which
//! no fixture in this crate constructs.
//!
//! The summation order is fixed by the loops and by nothing else. No part of
//! this is split across threads and recombined, because floating-point addition
//! is not associative and a sum recombined in whatever order the pieces
//! finished gives a different last digit each run, which is #3.
//!
//! What this module does not do, said plainly so a green read is not taken for
//! more than it is. It does not assemble the covariance from the uncertainties
//! and the asserted coefficients; the covariance arrives assembled, in the
//! file. It does not compute the smallest eigenvalue of the correlation matrix
//! or the condition number of the whitened design matrix, which are the two
//! numbers #56 asks a run to print, and there is no run in this workspace to
//! print them into. It does not solve anything: what it returns is the whitened
//! problem a solve is handed, and the solve is #57.

use core::fmt;

use crate::seam::{Observation, Problem};

/// What the covariance was refused for.
///
/// One variant today. It is an enum rather than a struct so a second refusal
/// widens this type later instead of replacing it, and so a reader matching on
/// it is told by the compiler when that happens.
#[derive(Debug)]
pub enum Refusal {
    /// The covariance is not positive definite, so it is not a covariance.
    ///
    /// The usual cause is an asserted correlation coefficient that cannot hold
    /// together with its neighbours, which is a finding about the published
    /// table rather than a numerical accident.
    NotPositiveDefinite {
        /// How many rows of the file the failing leading submatrix covers.
        order: usize,
        /// The observations in that submatrix, in the file's own row order.
        data: Vec<String>,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPositiveDefinite { order, data } => write!(
                out,
                "the covariance is not positive definite: the first {order} row(s) \
                 of the file cannot hold together as one, over the observations \
                 {}. That is the first prefix of the file's own row order that \
                 fails, and not the smallest set of data that cannot hold \
                 together: the same broken table in another row order names other \
                 observations. Nothing here repairs it, because a covariance that \
                 cannot be factored is a finding about the input data.",
                data.join(", "),
            ),
        }
    }
}

impl core::error::Error for Refusal {}

/// A problem whose covariance has been factored out of it.
///
/// The rows are the file's rows in the file's order, so an entry here is the
/// entry of the problem file at the same index and no reader has to carry a
/// permutation in their head.
#[derive(Debug)]
pub struct Whitened {
    factor: Vec<Vec<f64>>,
    observations: Vec<f64>,
    design: Vec<Vec<f64>>,
}

impl Whitened {
    /// The lower triangular factor of the covariance, row by row.
    ///
    /// Rows are full length and the entries above the diagonal are zero. That
    /// costs a few numbers at this size and saves every reader a convention to
    /// remember.
    #[must_use]
    pub fn factor(&self) -> &[Vec<f64>] {
        &self.factor
    }

    /// The whitened observation vector.
    #[must_use]
    pub fn observations(&self) -> &[f64] {
        &self.observations
    }

    /// The whitened design matrix, row by row.
    #[must_use]
    pub fn design(&self) -> &[Vec<f64>] {
        &self.design
    }
}

/// Factor the covariance and whiten the problem with it.
///
/// # Errors
///
/// [`Refusal::NotPositiveDefinite`] where the covariance cannot be factored,
/// naming the first prefix of the file's rows that fails.
pub fn whiten(problem: &Problem) -> Result<Whitened, Refusal> {
    let factor = factor_covariance(problem)?;
    let observations: Vec<Vec<f64>> = problem
        .observations()
        .iter()
        .map(|one| vec![Observation::value(one)])
        .collect();
    Ok(Whitened {
        observations: forward_substitute(&factor, &observations)
            .into_iter()
            .flatten()
            .collect(),
        design: forward_substitute(&factor, problem.design()),
        factor,
    })
}

/// The Cholesky factor of the covariance, or the prefix that refused it.
///
/// A factorisation fails at a leading principal minor, so the row it fails at
/// is the first prefix of the file's rows that cannot hold together, which is
/// what the refusal says and all it can say.
fn factor_covariance(problem: &Problem) -> Result<Vec<Vec<f64>>, Refusal> {
    let covariance = problem.covariance();
    let mut factor: Vec<Vec<f64>> = Vec::with_capacity(covariance.len());
    for (row, entries) in covariance.iter().enumerate() {
        // One row at a time, out of the rows already finished, so nothing here
        // reads a number this loop has not written yet.
        let mut mine = vec![0.0; covariance.len()];
        for (column, done) in factor.iter().enumerate() {
            let mut entry = entries[column];
            for (ours, theirs) in mine.iter().zip(done.iter()).take(column) {
                entry -= ours * theirs;
            }
            mine[column] = entry / done[column];
        }
        let mut pivot = entries[row];
        for entry in mine.iter().take(row) {
            pivot -= entry * entry;
        }
        if pivot > 0.0 {
            mine[row] = pivot.sqrt();
        } else {
            return Err(Refusal::NotPositiveDefinite {
                order: row + 1,
                data: problem
                    .observations()
                    .iter()
                    .take(row + 1)
                    .map(|one| one.identifier().to_owned())
                    .collect(),
            });
        }
        factor.push(mine);
    }
    Ok(factor)
}

/// Solve `factor * result = rows` for a lower triangular factor.
///
/// One routine for the observation vector and for the design matrix, because
/// two copies of this arithmetic are two places a sign can be wrong and one
/// place a suite can be green. The vector arrives as a matrix of one column.
///
/// Every diagonal entry is greater than zero, because [`factor_covariance`]
/// refuses the covariance otherwise, so nothing here divides by zero.
fn forward_substitute(factor: &[Vec<f64>], rows: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let mut result: Vec<Vec<f64>> = Vec::with_capacity(rows.len());
    for (row, entries) in rows.iter().enumerate() {
        let mut carried = entries.clone();
        for (found, coefficient) in result.iter().zip(factor[row].iter()) {
            for (value, taken) in carried.iter_mut().zip(found.iter()) {
                *value -= coefficient * taken;
            }
        }
        for value in &mut carried {
            *value /= factor[row][row];
        }
        result.push(carried);
    }
    result
}
