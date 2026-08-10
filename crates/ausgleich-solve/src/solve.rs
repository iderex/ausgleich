//! The least-squares solve and the covariance of what it fitted, #57.
//!
//! One factorisation, and everything after it is read off that factorisation.
//! The whitened design matrix is turned into an upper triangle by reflections,
//! the estimates come out of the triangle by back substitution, and the
//! covariance of the estimates is the same triangle inverted and multiplied by
//! its own transpose. Nothing here computes the covariance a second way.
//!
//! That is #57's own argument and it is worth keeping in front of a reader: a
//! covariance computed by a route of its own can disagree with the solution it
//! belongs to, and nothing in the printed values would show it. Two numbers
//! that came out of one triangle cannot disagree about which problem they are
//! about.
//!
//! The reflections are the reason no cross-product is formed, which is #2. The
//! triangle here is the triangle of the whitened design; the matrix of the
//! normal equations is its own transpose times itself, and forming that squares
//! the condition number, which spends half the available digits before the
//! solve begins on a problem that is ill conditioned to start with.
//!
//! The uncertainties and the correlations are not stored. They are the square
//! roots of the diagonal of the covariance and the covariance divided by the
//! outer product of those roots, so keeping them as fields would be keeping a
//! second copy of numbers already here, and two copies of one fact are two
//! things that can disagree. They are computed where they are asked for.
//!
//! What this module does not do, said plainly so a green read is not taken for
//! more than it is.
//!
//! It does not scale the covariance by how large the residuals came out. The
//! covariance it reports is the one the observation covariance in the input set
//! implies, and enlarging it by a factor read off this fit would be an
//! uncertainty enlargement applied in the solver, which is what #10 refuses and
//! what #61 puts in a file of records instead.
//!
//! It computes no residual, no chi squared and no Birge ratio. Those are #59,
//! they are reported next to the values rather than inside the solve, and the
//! observation vector this module reduces alongside the design carries what
//! they are made of.
//!
//! It writes no file. The result file of #13 is in [`crate::seam`] and the only
//! way to hold one is to read one, so a fit cannot be written out yet, which is
//! the leg of #57 this module does not reach.
//!
//! The summation order is fixed by the loops and by nothing else. No part of
//! this is split across threads and recombined, because floating-point addition
//! is not associative and a sum recombined in whatever order the pieces
//! finished gives a different last digit each run, which is #3.

use core::fmt;

use crate::conditioning::conditioning;
use crate::whiten::Whitened;

/// What the whitened problem was refused for.
///
/// One variant today. It is an enum rather than a struct so a second refusal
/// widens this type later instead of replacing it, and so a reader matching on
/// it is told by the compiler when that happens.
#[derive(Debug)]
pub enum Refusal {
    /// The whitened design does not determine every parameter on its own.
    ///
    /// A column that is already reachable from the columns before it leaves a
    /// direction in which the data says nothing, and every point along that
    /// direction fits equally well. The usual cause is two adjusted constants
    /// that no datum tells apart, which is a finding about the input set rather
    /// than a matrix to mend.
    RankDeficient {
        /// The column the reduction ran out at, counting from zero.
        ///
        /// Columns are in the order the problem file lists its parameters, so
        /// the caller that holds the problem file names the parameter. This
        /// module is handed the whitened problem and the whitened problem
        /// carries no names.
        column: usize,
        /// The singular values of the whitened design, largest first.
        ///
        /// #2 asks for these where the rank is in question. They say how far
        /// from determined the problem is rather than only that it is: a
        /// smallest value at zero with the rest far above it is one direction
        /// missing, and a run of small ones is a problem that is nearly not
        /// there in several directions at once.
        singular_values: Vec<f64>,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RankDeficient {
                column,
                singular_values,
            } => write!(
                out,
                "the whitened design does not determine the parameter in column \
                 {column}, counting from zero: that column is already reached by \
                 the columns before it, so the data leaves a direction in which \
                 every answer fits as well as every other. The singular values \
                 of the whitened design are {}. Nothing here returns a \
                 pseudo-inverse, because a fit along a direction the data does \
                 not determine looks exactly like an answer.",
                singular_values
                    .iter()
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        }
    }
}

impl core::error::Error for Refusal {}

/// A fitted problem: the triangle it was fitted through, the estimates, and
/// their covariance.
///
/// The rows and the columns are the problem file's parameters in the problem
/// file's order, so an entry here is the parameter at the same index and no
/// reader has to carry a permutation in their head.
#[derive(Debug)]
pub struct Fit {
    triangle: Vec<Vec<f64>>,
    parameters: Vec<f64>,
    covariance: Vec<Vec<f64>>,
}

impl Fit {
    /// The upper triangle the whitened design was reduced to, row by row.
    ///
    /// Carried rather than dropped because it is the thing this issue's first
    /// sentence is about. A reader who wants to check that the covariance below
    /// belongs to the estimates beside it can do it from this and nothing else:
    /// the covariance is this triangle inverted and multiplied by its own
    /// transpose, and the estimates are this triangle back-substituted.
    ///
    /// Rows are full length and the entries below the diagonal are zero. That
    /// costs a few numbers at this size and saves every reader a convention to
    /// remember, which is the choice [`Whitened::factor`] made for the same
    /// reason.
    #[must_use]
    pub fn triangle(&self) -> &[Vec<f64>] {
        &self.triangle
    }

    /// The fitted value of every adjusted constant.
    #[must_use]
    pub fn parameters(&self) -> &[f64] {
        &self.parameters
    }

    /// The covariance of the fitted values, row by row.
    #[must_use]
    pub fn covariance(&self) -> &[Vec<f64>] {
        &self.covariance
    }

    /// The uncertainty of every fitted value.
    ///
    /// The square root of its own variance, which is what the diagonal of the
    /// covariance holds. No entry of that diagonal is zero: it is the squared
    /// length of a row of the inverted triangle, and a triangle with no zero on
    /// its diagonal has no inverse with an empty row.
    #[must_use]
    pub fn uncertainties(&self) -> Vec<f64> {
        self.covariance
            .iter()
            .enumerate()
            .map(|(index, row)| row[index].sqrt())
            .collect()
    }

    /// The correlation matrix of the fitted values, row by row.
    ///
    /// #57 asks for this beside the uncertainties because the published
    /// adjustments report it and because a user who takes two constants out of
    /// a table and combines them without it gets an uncertainty that is wrong
    /// in whichever direction the correlation runs.
    #[must_use]
    pub fn correlations(&self) -> Vec<Vec<f64>> {
        let spread = self.uncertainties();
        self.covariance
            .iter()
            .zip(spread.iter())
            .map(|(row, mine)| {
                row.iter()
                    .zip(spread.iter())
                    .map(|(value, theirs)| value / (mine * theirs))
                    .collect()
            })
            .collect()
    }
}

/// Fit the whitened problem, and report the covariance of what was fitted.
///
/// # Errors
///
/// [`Refusal::RankDeficient`] where the whitened design leaves a direction the
/// data does not determine, naming the column the reduction ran out at and the
/// singular values of the whole matrix.
pub fn solve(whitened: &Whitened) -> Result<Fit, Refusal> {
    let width = whitened.design().iter().map(Vec::len).fold(0, usize::max);
    let mut rows = with_the_observations_alongside(whitened);
    for column in 0..width {
        // The observation vector is reduced by the same reflections as the
        // design, in the same call, because it is the last column of the same
        // matrix. Two copies of this arithmetic are two places a sign can be
        // wrong and one place a suite can be green, which is the argument
        // [`crate::whiten`] makes for its own substitution routine.
        let Some(()) = reduce(&mut rows, column, width) else {
            return Err(Refusal::RankDeficient {
                column,
                singular_values: conditioning(whitened).design_singular_values().to_vec(),
            });
        };
    }
    let triangle = triangle_of(&rows, width);
    let inverse = inverted(&triangle);
    Ok(Fit {
        parameters: estimates(&mut rows, width),
        covariance: covariance_of(&inverse),
        triangle,
    })
}

/// The design matrix with the observation vector as one more column.
///
/// Both come out of one [`Whitened`], so the row of the design and the entry of
/// the observation vector that meet in a row here are the row of the problem
/// file at the same index.
fn with_the_observations_alongside(whitened: &Whitened) -> Vec<Vec<f64>> {
    whitened
        .design()
        .iter()
        .zip(whitened.observations())
        .map(|(row, value)| {
            let mut alongside = row.clone();
            alongside.push(*value);
            alongside
        })
        .collect()
}

/// Turn one column onto the diagonal, and carry the whole matrix through the
/// same turn.
///
/// The turn is a reflection: the part of the column at and below the diagonal
/// is sent onto the first axis, which puts its length on the diagonal and
/// nothing below it. Every column from this one on, the observation vector
/// included, is reflected with it, so the whole matrix is being written in a
/// turned frame rather than the columns being changed one at a time.
///
/// The column being turned is carried through with the rest rather than left
/// out of the loop, and then the two things the turn puts there by construction
/// are written: the length on the diagonal, and zeros below it. The arithmetic
/// arrives at both with a rounding error each, and a triangle that carries a
/// residue where the definition fixes a zero is a triangle every number after
/// it inherits the residue from.
///
/// `None` where that part of the column is nothing at all, which is the whole
/// of what this file can say about rank. A column of zeros is a column the ones
/// before it already reach exactly, and it is also the column back substitution
/// would divide by. The comparison is written as one that has to succeed, so a
/// length that cannot be compared takes the refusing side by construction.
///
/// What that does not reach, and it is the larger half: a column the others
/// nearly reach leaves a length that is small and not zero, and this returns an
/// answer for it. There is no threshold here to widen, because there is no
/// threshold. What says how near a problem came is the condition number in
/// [`crate::conditioning`], reported for a reader to judge.
///
/// The reflection is taken away from the leading entry rather than towards it,
/// which is why the diagonal takes the sign opposite to that entry. Towards it,
/// the subtraction that forms the reflector is a difference of two nearly equal
/// numbers, and the digits it loses are the digits the answer is made of.
fn reduce(rows: &mut [Vec<f64>], column: usize, width: usize) -> Option<()> {
    let mut squares = 0.0;
    for row in rows.iter().skip(column) {
        squares += row[column] * row[column];
    }
    let length = squares.sqrt();
    if length > 0.0 {
        let diagonal = if rows[column][column] < 0.0 {
            length
        } else {
            -length
        };
        let mut reflector = vec![0.0; rows.len()];
        for (index, row) in rows.iter().enumerate().skip(column) {
            reflector[index] = row[column];
        }
        reflector[column] -= diagonal;
        // Never zero where the length above is not: the entry the diagonal was
        // taken away from grew by at least that length, so the sum below has a
        // term at least its square.
        let mut scale = 0.0;
        for value in reflector.iter().skip(column) {
            scale += value * value;
        }
        for target in column..=width {
            let mut inner = 0.0;
            for (index, row) in rows.iter().enumerate().skip(column) {
                inner += reflector[index] * row[target];
            }
            let factor = (inner + inner) / scale;
            for (index, row) in rows.iter_mut().enumerate().skip(column) {
                row[target] -= factor * reflector[index];
            }
        }
        // The diagonal first, then the zeros under it, and in that order. The
        // other order writes the zeros over a diagonal that has not been put
        // there yet.
        rows[column][column] = diagonal;
        for row in rows.iter_mut().skip(column + 1) {
            row[column] = 0.0;
        }
        Some(())
    } else {
        None
    }
}

/// The upper triangle, out of the reduced matrix.
///
/// A copy of the top left square and nothing more, because the reduction wrote
/// the zeros under the diagonal rather than leaving them to be substituted
/// here. The columns of the observation vector and the rows past the last
/// parameter are left behind: those rows are what the residuals are made of and
/// they belong to #59.
fn triangle_of(rows: &[Vec<f64>], width: usize) -> Vec<Vec<f64>> {
    rows.iter()
        .take(width)
        .map(|entries| entries.iter().take(width).copied().collect())
        .collect()
}

/// The estimates, by back substitution through the reduced matrix.
///
/// The last column is the observation vector as the same reflections left it,
/// so the equation solved here is the triangle against that column, and no
/// second pass over the data is needed to form a right-hand side. The answer
/// replaces that column entry by entry, which is what makes each substitution
/// read the entries the ones before it wrote.
///
/// Nothing here divides by zero: [`reduce`] wrote a diagonal entry for every
/// column and refused the problem where it could not.
fn estimates(rows: &mut [Vec<f64>], width: usize) -> Vec<f64> {
    for row in (0..width).rev() {
        let mut carried = rows[row][width];
        for (step, below) in rows.iter().enumerate().take(width).skip(row + 1) {
            carried -= rows[row][step] * below[width];
        }
        rows[row][width] = carried / rows[row][row];
    }
    rows.iter().take(width).map(|row| row[width]).collect()
}

/// The inverse of an upper triangle, which is upper triangular too.
///
/// One column of the inverse at a time, each by the same back substitution
/// [`estimates`] does, against a column of the identity instead of against the
/// observations. The diagonal entry of the inverse is not a case of its own:
/// substituting the column of the identity that carries its one in that row
/// arrives at it.
fn inverted(triangle: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let width = triangle.len();
    let mut inverse = vec![vec![0.0; width]; width];
    for column in 0..width {
        let mut answer = vec![0.0; width];
        answer[column] = 1.0;
        for row in (0..=column).rev() {
            let mut carried = answer[row];
            for step in (row + 1)..=column {
                carried -= triangle[row][step] * answer[step];
            }
            answer[row] = carried / triangle[row][row];
        }
        for (row, value) in answer.into_iter().enumerate() {
            inverse[row][column] = value;
        }
    }
    inverse
}

/// The covariance of the estimates, from the inverted triangle and nothing
/// else.
///
/// The covariance of a least-squares fit of a whitened problem is the inverse
/// of the triangle times that inverse transposed. It is one product of one
/// matrix with itself, which is why it cannot be about a different problem than
/// the estimates are: both were read off the same triangle.
fn covariance_of(inverse: &[Vec<f64>]) -> Vec<Vec<f64>> {
    inverse
        .iter()
        .map(|left| {
            inverse
                .iter()
                .map(|right| {
                    left.iter()
                        .zip(right.iter())
                        .map(|(mine, theirs)| mine * theirs)
                        .sum::<f64>()
                })
                .collect()
        })
        .collect()
}
