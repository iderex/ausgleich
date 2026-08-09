//! What the factorisation accepts, what it refuses, and what the whitened
//! problem comes out as, #56.
//!
//! Every expected number below was worked out by hand from the definition and
//! is written next to the case that uses it, rather than being whatever the
//! code produced on the day. Each covariance was chosen so that every square
//! root and every division in it is exact in binary floating point, which is
//! why the comparisons are equality and not a tolerance. A tolerance here would
//! be a number somebody picked standing between the test and the arithmetic.
//!
//! The documents are built from one canonical problem with one substitution
//! made in it, so a reader sees the single difference a case turns on instead
//! of reading a second document to find out what changed.

use ausgleich_solve::seam::read_problem;
use ausgleich_solve::whiten::{Refusal, whiten};

/// The canonical problem, with the covariance and the design left to fill in.
///
/// Two observations, two parameters, and a header that says nothing about any
/// measurement. `{covariance}` and `{design}` are the only holes in it.
fn problem_with(covariance: &str, design: &str) -> String {
    format!(
        r#"[header]
input_set = "first"
input_set_hash = "sha256:0000"
code_version = "0.0.0"
code_commit = "0000000"
toolchain = "1.85.0"
command_line = "ausgleich assemble"

[[parameter]]
identifier = "alpha"
unit = "1"

[[parameter]]
identifier = "rydberg"
unit = "m^-1"

[[observation]]
identifier = "mass-one"
value = 1.5

[[observation]]
identifier = "mass-two"
value = -2.0

[problem]
covariance = {covariance}
design = {design}
"#
    )
}

/// The identity, so a case about the covariance is not also a case about the
/// design.
const IDENTITY: &str = "[[1.0, 0.0], [0.0, 1.0]]";

/// Read a problem the seam accepts, or fail the test where it does not.
///
/// A document this file writes badly should stop the test that wrote it rather
/// than arrive at the factorisation as something else.
fn problem(text: &str) -> ausgleich_solve::seam::Problem {
    match read_problem(text) {
        Ok(found) => found,
        Err(refusal) => panic!("the seam refused a document this test wrote: {refusal}"),
    }
}

#[test]
fn a_diagonal_covariance_divides_each_row_by_its_standard_deviation() {
    // The case with no correlation at all, where the answer is arithmetic
    // anybody can check in their head. Variances 4 and 16, so the factor is
    // diagonal with 2 and 4 on it, and whitening is division by those.
    //
    //   observations   1.5 / 2  = 0.75      -2.0 / 4 = -0.5
    //   design         1.0 / 2  = 0.5        1.0 / 4 =  0.25
    let found = whiten(&problem(&problem_with(
        "[[4.0, 0.0], [0.0, 16.0]]",
        IDENTITY,
    )))
    .expect("a diagonal covariance with positive variances is factorable");

    assert_eq!(found.factor(), [vec![2.0, 0.0], vec![0.0, 4.0]]);
    assert_eq!(found.observations(), [0.75, -0.5]);
    assert_eq!(found.design(), [vec![0.5, 0.0], vec![0.0, 0.25]]);
}

#[test]
fn a_correlated_covariance_whitens_to_the_factorisation_worked_out_by_hand() {
    // The case the whole issue is about: an off-diagonal entry, so whitening
    // mixes the rows rather than scaling them.
    //
    //   covariance   [[4, 2], [2, 5]]
    //   l[0][0] = sqrt(4)     = 2
    //   l[1][0] = 2 / 2       = 1
    //   l[1][1] = sqrt(5 - 1) = 2
    //
    // Then forward substitution, with the observations 1.5 and -2.0:
    //
    //   z[0] = 1.5 / 2                 =  0.75
    //   z[1] = (-2.0 - 1 * 0.75) / 2   = -1.375
    //
    // and the identity design, column by column:
    //
    //   row 0    [1 / 2, 0 / 2]                       = [ 0.5,  0.0]
    //   row 1    [(0 - 1 * 0.5) / 2, (1 - 0) / 2]     = [-0.25, 0.5]
    let found = whiten(&problem(&problem_with(
        "[[4.0, 2.0], [2.0, 5.0]]",
        IDENTITY,
    )))
    .expect("this covariance is positive definite");

    assert_eq!(found.factor(), [vec![2.0, 0.0], vec![1.0, 2.0]]);
    assert_eq!(found.observations(), [0.75, -1.375]);
    assert_eq!(found.design(), [vec![0.5, 0.0], vec![-0.25, 0.5]]);
}

#[test]
fn the_factor_times_its_own_transpose_is_the_covariance_again() {
    // The property the factorisation is judged on, over a matrix big enough
    // that every term of the sum inside the loop is used. The covariance was
    // built by choosing the factor first and multiplying it out by hand:
    //
    //   L = [[ 2, 0, 0],
    //        [ 3, 2, 0],
    //        [-1, 4, 1]]
    //
    //   L * L^T = [[ 4,  6, -2],
    //              [ 6, 13,  5],
    //              [-2,  5, 18]]
    //
    // so the factorisation has one right answer and it is L. A sign flipped
    // anywhere in the inner sum moves at least one entry of it.
    //
    // No entry of L below the diagonal is 1, and that is chosen rather than
    // incidental. The one term this fixture drives through the inner sum is
    // `mine[0] * done[0]`, and with a 1 in it multiplication and division agree,
    // so the arithmetic the sum exists for would go unexamined by a fixture that
    // otherwise looks complete. A mutation run found exactly that.
    let text = problem_with(
        "[[4.0, 6.0, -2.0], [6.0, 13.0, 5.0], [-2.0, 5.0, 18.0]]",
        "[[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]",
    )
    .replace(
        "[[observation]]\nidentifier = \"mass-two\"\nvalue = -2.0\n",
        "[[observation]]\nidentifier = \"mass-two\"\nvalue = -2.0\n\n\
         [[observation]]\nidentifier = \"mass-three\"\nvalue = 4.0\n",
    );

    let found = whiten(&problem(&text)).expect("this covariance is positive definite");

    assert_eq!(
        found.factor(),
        [
            vec![2.0, 0.0, 0.0],
            vec![3.0, 2.0, 0.0],
            vec![-1.0, 4.0, 1.0],
        ]
    );
}

#[test]
fn a_covariance_whose_correlation_is_too_strong_is_refused_over_both_rows() {
    // Unit variances with a covariance of 2 between them, which asserts a
    // correlation of 2 and is the shape a mistyped coefficient has. The first
    // row is fine on its own: 1 is a positive variance. The pair is not, and
    // the pivot says so, since 1 - 2 * 2 = -3.
    let refusal = whiten(&problem(&problem_with(
        "[[1.0, 2.0], [2.0, 1.0]]",
        IDENTITY,
    )))
    .expect_err("a correlation of two cannot hold together");

    let Refusal::NotPositiveDefinite { order, data } = &refusal;
    assert_eq!(*order, 2);
    assert_eq!(data, &["mass-one".to_owned(), "mass-two".to_owned()]);
}

#[test]
fn the_refused_prefix_stops_at_the_row_that_failed_and_names_no_data_after_it() {
    // Three observations, and the second variance is negative. The refusal is
    // about the first two rows, so the third observation is not named. This is
    // what makes the order and the list a statement about where the
    // factorisation stopped rather than about the size of the file.
    let text = problem_with(
        "[[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]",
    )
    .replace(
        "[[observation]]\nidentifier = \"mass-two\"\nvalue = -2.0\n",
        "[[observation]]\nidentifier = \"mass-two\"\nvalue = -2.0\n\n\
         [[observation]]\nidentifier = \"mass-three\"\nvalue = 4.0\n",
    );

    let refusal = whiten(&problem(&text)).expect_err("a negative variance is not a variance");

    let Refusal::NotPositiveDefinite { order, data } = &refusal;
    assert_eq!(*order, 2);
    assert_eq!(data, &["mass-one".to_owned(), "mass-two".to_owned()]);
}

#[test]
fn a_variance_of_zero_is_refused_rather_than_divided_by() {
    // A datum quoted with no uncertainty at all. The boundary matters: a test
    // written as "less than zero" accepts this row, and the division that
    // follows produces infinities that travel all the way into the output
    // without anything going red.
    let refusal = whiten(&problem(&problem_with(
        "[[0.0, 0.0], [0.0, 1.0]]",
        IDENTITY,
    )))
    .expect_err("a variance of zero is refused");

    let Refusal::NotPositiveDefinite { order, data } = &refusal;
    assert_eq!(*order, 1);
    assert_eq!(data, &["mass-one".to_owned()]);
}

#[test]
fn a_covariance_entry_that_is_not_a_number_never_reaches_the_factorisation() {
    // Written here rather than assumed. The factorisation's pivot test is a
    // comparison that has to succeed, so a value that cannot be compared falls
    // to the refusing side by construction; this asserts the reason no fixture
    // in this file exercises that path, which is that the document is refused
    // one layer earlier and never arrives.
    let refusal = read_problem(&problem_with("[[nan, 0.0], [0.0, 1.0]]", IDENTITY))
        .expect_err("the seam refuses a covariance cell that is not finite");

    assert!(refusal.to_string().contains("problem.covariance[0][0]"));
}

#[test]
fn the_refusal_names_the_data_and_says_the_prefix_is_not_the_smallest_set() {
    // The reason the text is asserted rather than left to a reader: "not
    // positive definite" on its own sends somebody to read three hundred
    // files, and a reader who takes the named submatrix for the smallest one
    // goes looking for a defect in the wrong place. Both halves are the
    // message doing its job.
    let refusal = whiten(&problem(&problem_with(
        "[[1.0, 2.0], [2.0, 1.0]]",
        IDENTITY,
    )))
    .expect_err("a correlation of two cannot hold together");

    let said = refusal.to_string();
    assert!(said.contains("mass-one, mass-two"), "{said}");
    assert!(said.contains("not the smallest set of data"), "{said}");
    assert!(format!("{refusal:?}").contains("NotPositiveDefinite"));
}

#[test]
fn the_whitened_problem_says_what_it_is_when_it_is_printed() {
    // The debug form is what a fixture failure prints, so it is worth one
    // assertion that it carries the numbers rather than the type name alone.
    let found = whiten(&problem(&problem_with(
        "[[4.0, 0.0], [0.0, 16.0]]",
        IDENTITY,
    )))
    .expect("a diagonal covariance with positive variances is factorable");

    assert!(format!("{found:?}").contains("0.75"));
}
