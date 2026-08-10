//! The two numbers that say how far the answer can be trusted, #56.
//!
//! A least-squares result is a set of digits whatever the problem was, and the
//! digits look the same whether the fit was well posed or nearly singular. The
//! smallest eigenvalue of the correlation matrix says whether the asserted
//! coefficients only just hold together; the condition number of the whitened
//! design matrix says how much of the input precision survives the solve. #56
//! asks a run to print both, and #2 asks for the second by name and for the
//! singular values themselves where the rank is in question.
//!
//! Both come out of one routine, and that is the reason this module is small.
//! The singular values of a matrix are computed by rotating pairs of its
//! columns towards orthogonality until they stop moving, and the lengths of the
//! columns that are left are the singular values. The eigenvalues of the
//! correlation matrix are the squares of the singular values of its factor, so
//! the same routine answers both questions and there is one piece of arithmetic
//! to prove rather than two.
//!
//! The correlation matrix is never assembled. Its factor is the covariance
//! factor [`crate::whiten`] already produced with each row scaled to unit
//! length, because a lower triangular factor whose rows have unit length is the
//! factor of a matrix with ones down the diagonal, which is what a correlation
//! matrix is. Reading the eigenvalues off that factor, as squares of singular
//! values, is the same direction the whitening takes for the same reason: a
//! quantity read from a factor keeps the digits that assembling and then
//! decomposing a product would spend.
//!
//! This is not the route #2 refuses, and the difference is worth stating
//! plainly because the two can be confused. What that decision refuses is
//! solving through the cross-product of the design matrix, which squares the
//! condition number of the solve. Nothing here forms that product, nothing here
//! solves anything, and the matrix whose condition number is being reported is
//! never squared into another matrix. Each rotation reads three sums over one
//! pair of columns and writes the pair back. That the singular values so
//! obtained keep their relative accuracy is a claim, made from the shape of the
//! algorithm and not from a measurement in this tree; no comparison against an
//! independent computation of the same numbers exists here, and #62 is where
//! one would live.
//!
//! Nothing here refuses anything, and that is deliberate. A covariance that
//! cannot hold together is refused in [`crate::whiten`], before this module is
//! reachable at all: the only way to hold a [`Whitened`] is to have factored
//! the covariance, so every row of the factor has a diagonal entry greater than
//! zero and the scaling below cannot divide by nothing. These two numbers are
//! reported for a reader to judge, and no threshold in this file decides
//! whether a run continues.
//!
//! The sweep count is a cap and not a tolerance. It is there so the routine
//! ends rather than to decide anything, and there is no number in this file
//! somebody could widen to let a bad table through, because nothing here
//! accepts or rejects a table. The comparison that skips a pair is against zero
//! exactly, so the file carries no tolerance either.
//!
//! What the cap could hide is reported rather than left silent. Each of the two
//! answers carries the number of passes it took, and the last pass of that
//! count is the one after which no column length moved, so a reader can see
//! that the rotations settled instead of assuming it. A count that reached the
//! cap is a count that ran out, and the numbers beside it are where the
//! rotations were when they were stopped.
//!
//! Settling is judged on the answer rather than on the angles: a pass whose
//! column lengths are the ones the pass before it left is a pass that changed
//! nothing a reader is being told. That is a comparison of the numbers this
//! module reports against themselves, so it needs no tolerance either.
//!
//! The summation order is fixed by the loops and the pairs are visited in one
//! order. No part of this is split across threads and recombined, because
//! floating-point addition is not associative and a sum recombined in whatever
//! order the pieces finished gives a different last digit each run, which is
//! #3.
//!
//! What this module does not do. It does not print: there is no run in this
//! workspace to print into, what an operator types is #78 and what the report
//! says is #79, so the fourth leg of #56 stays open on the printing and not on
//! the arithmetic. It does not decide whether a condition number is too large
//! or a smallest eigenvalue too near zero; those are judgements a reader makes
//! and the publication's own thresholds are #60's.

use crate::whiten::Whitened;

/// The most passes over every pair of columns the rotations are given.
///
/// A cap, so the routine ends. A pass that moves no column length is the last
/// one, and at the sizes here a handful of passes is what settling costs, so
/// the cap is chosen to sit far past that rather than close to it: the cost of
/// a pass nobody needed is nothing, and the cost of stopping early is a wrong
/// number nobody can see is wrong.
///
/// It is public because the counts reported beside the numbers are only
/// readable against it. A count that reached this is a count that ran out.
pub const SWEEPS: usize = 30;

/// What the conditioning of a whitened problem is.
///
/// Both lists are as long as they can be: one eigenvalue per observation, one
/// singular value per parameter.
#[derive(Debug)]
pub struct Conditioning {
    correlation_eigenvalues: Vec<f64>,
    correlation_sweeps: usize,
    design_singular_values: Vec<f64>,
    design_sweeps: usize,
}

impl Conditioning {
    /// The eigenvalues of the correlation matrix, smallest first.
    ///
    /// Sorted for a reader rather than for the arithmetic. The smallest is the
    /// interesting end, because it is the one that goes to zero as the asserted
    /// coefficients approach a set that cannot hold together.
    #[must_use]
    pub fn correlation_eigenvalues(&self) -> &[f64] {
        &self.correlation_eigenvalues
    }

    /// The smallest eigenvalue of the correlation matrix.
    ///
    /// Taken from the values rather than from their order, so it stays the
    /// smallest if the sort above ever changes.
    #[must_use]
    pub fn smallest_correlation_eigenvalue(&self) -> f64 {
        self.correlation_eigenvalues
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
    }

    /// The singular values of the whitened design matrix, largest first.
    ///
    /// #2 asks for these where the rank is in question, so they are carried
    /// rather than reduced to the ratio below and thrown away.
    #[must_use]
    pub fn design_singular_values(&self) -> &[f64] {
        &self.design_singular_values
    }

    /// How many passes over the column pairs the correlation factor took.
    ///
    /// The last pass of the count is the one after which no length moved, so a
    /// count below [`SWEEPS`] is the statement that the rotations settled and
    /// the numbers above are what they settled on. A count equal to [`SWEEPS`]
    /// is the statement that they had not, and that the answer is where the cap
    /// stopped them rather than where they were going.
    ///
    /// Reported rather than turned into a verdict here, for the reason the
    /// module gives: nothing in this file refuses anything.
    #[must_use]
    pub fn correlation_sweeps(&self) -> usize {
        self.correlation_sweeps
    }

    /// How many passes over the column pairs the whitened design took.
    ///
    /// Read the same way as [`Self::correlation_sweeps`].
    #[must_use]
    pub fn design_sweeps(&self) -> usize {
        self.design_sweeps
    }

    /// The condition number of the whitened design matrix.
    ///
    /// The largest singular value over the smallest. A rank deficient design
    /// has a smallest singular value of zero and this is infinite, which is the
    /// honest answer and the one #2 asks for instead of a quiet pseudo-inverse.
    #[must_use]
    pub fn design_condition_number(&self) -> f64 {
        let largest = self
            .design_singular_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let smallest = self
            .design_singular_values
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        largest / smallest
    }
}

/// The conditioning of a whitened problem.
///
/// Takes the whitened problem and nothing else, so the covariance factor and
/// the design matrix it reads cannot be from two different documents.
#[must_use]
pub fn conditioning(whitened: &Whitened) -> Conditioning {
    let (lengths, correlation_sweeps) = singular_values(&correlation_factor(whitened.factor()));
    let mut correlation_eigenvalues: Vec<f64> =
        lengths.into_iter().map(|value| value * value).collect();
    correlation_eigenvalues.sort_by(|left, right| left.total_cmp(right));
    let (mut design_singular_values, design_sweeps) = singular_values(whitened.design());
    design_singular_values.sort_by(|left, right| right.total_cmp(left));
    Conditioning {
        correlation_eigenvalues,
        correlation_sweeps,
        design_singular_values,
        design_sweeps,
    }
}

/// The covariance factor with every row scaled to unit length.
///
/// That is the factor of the correlation matrix. Writing the covariance as its
/// factor times the factor's transpose, and the correlation as the covariance
/// with each row and each column divided by the square root of the diagonal
/// entry it belongs to, the two divisions land on the same factor from either
/// side, and the square root of the diagonal entry is the length of that row of
/// the factor.
///
/// No row has zero length. A row's length is at least its diagonal entry, and
/// the factorisation refuses the covariance where that is not greater than
/// zero, so a [`Whitened`] in hand is the proof that the division below has a
/// number to divide by.
fn correlation_factor(factor: &[Vec<f64>]) -> Vec<Vec<f64>> {
    factor
        .iter()
        .map(|row| {
            let length = row.iter().map(|entry| entry * entry).sum::<f64>().sqrt();
            row.iter().map(|entry| entry / length).collect()
        })
        .collect()
}

/// The singular values of a matrix given by its rows, in the order its columns
/// happen to be left in.
///
/// The columns are rotated pairwise towards orthogonality; orthogonal columns
/// are the singular vectors scaled by the singular values, so once no pair
/// moves the lengths of the columns are the answer.
fn singular_values(rows: &[Vec<f64>]) -> (Vec<f64>, usize) {
    let width = rows.iter().map(Vec::len).fold(0, usize::max);
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); width];
    for row in rows {
        for (column, entry) in row.iter().enumerate() {
            columns[column].push(*entry);
        }
    }
    let mut lengths = lengths_of(&columns);
    let mut sweeps = 0;
    for _ in 0..SWEEPS {
        sweeps += 1;
        for left in 0..width {
            for right in (left + 1)..width {
                rotate(&mut columns, left, right);
            }
        }
        let after = lengths_of(&columns);
        if after == lengths {
            break;
        }
        lengths = after;
    }
    (lengths, sweeps)
}

/// The length of every column.
fn lengths_of(columns: &[Vec<f64>]) -> Vec<f64> {
    columns
        .iter()
        .map(|column| column.iter().map(|entry| entry * entry).sum::<f64>().sqrt())
        .collect()
}

/// Turn one pair of columns towards orthogonality.
///
/// The angle is the one that makes the pair exactly orthogonal, written in the
/// form that takes the smaller of the two rotations. Writing it as the larger
/// one converges too and loses digits on the column that was already almost
/// right, which is the whole reason for choosing between them.
///
/// A pair that is already orthogonal is left alone, and the comparison is
/// against zero exactly because that is the case the arithmetic below cannot
/// take: two columns of equal length with nothing between them give a slope of
/// zero over zero, and every number after it is then not a number.
fn rotate(columns: &mut [Vec<f64>], left: usize, right: usize) {
    let (before, after) = columns.split_at_mut(right);
    let one = &mut before[left];
    let other = &mut after[0];
    let mut here = 0.0;
    let mut there = 0.0;
    let mut across = 0.0;
    for (mine, yours) in one.iter().zip(other.iter()) {
        here += mine * mine;
        there += yours * yours;
        across += mine * yours;
    }
    if across == 0.0 {
        return;
    }
    let slope = (there - here) / (across + across);
    let tangent = slope.signum() / (slope.abs() + (slope * slope + 1.0).sqrt());
    let cosine = 1.0 / (tangent * tangent + 1.0).sqrt();
    let sine = cosine * tangent;
    for (mine, yours) in one.iter_mut().zip(other.iter_mut()) {
        let was = *mine;
        *mine = cosine * was - sine * *yours;
        *yours = sine * was + cosine * *yours;
    }
}
