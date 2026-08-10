//! What the two conditioning numbers come out as, #56.
//!
//! Every expected number below was worked out by hand from the definition and
//! is written next to the case that uses it, rather than being whatever the
//! code produced on the day.
//!
//! Most of the cases compare for equality, because their covariance and design
//! were chosen so that every square root and every division in them is exact in
//! binary floating point. Two cannot be: an eigenvalue of one and six tenths is
//! not a binary fraction, and no correlation coefficient makes both the
//! coefficient and the factor entry beside it exact, since that would need two
//! binary fractions whose squares sum to one. Those carry a bound of a few units
//! in the last place, which is what a handful of roundings can cost, and not a
//! number read off the output: a routine computing something other than these
//! numbers misses by a margin nothing like that size.
//!
//! The pass counts are asserted as hard as the numbers are, and they are the
//! leg that says the rotation angle is the right one. The singular values of a
//! matrix survive any rotation of its columns, right angle or wrong, so a
//! routine turning through the wrong angle still arrives at them given enough
//! passes and the numbers alone cannot tell the two apart. What the wrong angle
//! costs is passes: the angle here is the one that makes a pair orthogonal at
//! once, so a pair takes one pass to settle and the next finds nothing to do.

use ausgleich_solve::conditioning::{Conditioning, conditioning};
use ausgleich_solve::seam::read_problem;
use ausgleich_solve::whiten::whiten;

/// A problem document, with as many parameters and observations as it is given
/// identifiers for.
///
/// One builder rather than one per shape, because the cases below differ in
/// how many rows and columns they have and a second copy of this text would be
/// a second place for the header to go stale.
fn document(parameters: &[&str], observations: &[&str], covariance: &str, design: &str) -> String {
    let mut text = String::from(
        r#"[header]
input_set = "first"
input_set_hash = "sha256:0000"
code_version = "0.0.0"
code_commit = "0000000"
toolchain = "1.85.0"
command_line = "ausgleich assemble"
"#,
    );
    for name in parameters {
        text.push_str(&format!(
            "\n[[parameter]]\nidentifier = \"{name}\"\nunit = \"1\"\n"
        ));
    }
    for name in observations {
        text.push_str(&format!(
            "\n[[observation]]\nidentifier = \"{name}\"\nvalue = 1.5\n"
        ));
    }
    text.push_str(&format!(
        "\n[problem]\ncovariance = {covariance}\ndesign = {design}\n"
    ));
    text
}

/// Two parameters and two observations, which is the smallest shape that has an
/// off-diagonal entry to argue about.
fn two_by_two(covariance: &str, design: &str) -> String {
    document(
        &["alpha", "rydberg"],
        &["mass-one", "mass-two"],
        covariance,
        design,
    )
}

/// The two-by-two identity, as a covariance that whitens by dividing by one or
/// as a design that leaves a case about the covariance alone.
const IDENTITY: &str = "[[1.0, 0.0], [0.0, 1.0]]";

/// The three-by-three identity, for the cases with three observations.
const IDENTITY_THREE: &str = "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]";

/// A few units in the last place of the expected value.
///
/// Stated once rather than beside each use, and derived from the arithmetic
/// rather than from the output: one turn of a pair of columns is a handful of
/// multiplications, a subtraction and two square roots, each of which can cost
/// half a unit in the last place.
fn close_enough(found: f64, expected: f64) {
    let allowed = 8.0 * f64::EPSILON * expected.abs();
    assert!(
        (found - expected).abs() <= allowed,
        "expected {expected}, found {found}, which is further off than {allowed}"
    );
}

/// The conditioning of a document this file wrote, or a failure naming which
/// step refused it.
///
/// A document this file writes badly should stop the test that wrote it rather
/// than arrive at the arithmetic as something else.
fn conditioning_of(text: &str) -> Conditioning {
    let problem = match read_problem(text) {
        Ok(found) => found,
        Err(refusal) => panic!("the seam refused a document this test wrote: {refusal}"),
    };
    match whiten(&problem) {
        Ok(found) => conditioning(&found),
        Err(refusal) => panic!("the factorisation refused a covariance this test wrote: {refusal}"),
    }
}

#[test]
fn uncorrelated_data_of_different_precision_have_a_correlation_matrix_of_ones() {
    // Variances of four and twenty five, so the two rows are of very different
    // precision and the covariance itself has eigenvalues four and twenty five.
    // The correlation matrix does not: with nothing asserted between the rows
    // it is the identity, and both its eigenvalues are one exactly.
    //
    // This is the case that says the scaling to unit rows happens. Without it
    // the numbers below would be four and twenty five.
    let found = conditioning_of(&two_by_two("[[4.0, 0.0], [0.0, 25.0]]", IDENTITY));

    assert_eq!(found.correlation_eigenvalues(), [1.0, 1.0]);
    assert_eq!(found.smallest_correlation_eigenvalue(), 1.0);
    assert_eq!(found.correlation_sweeps(), 1);
}

#[test]
fn a_correlated_pair_has_the_eigenvalues_one_either_side_of_the_coefficient() {
    // The covariance [[4, 6], [6, 25]] is standard deviations of two and five
    // with six tenths asserted between them, since 0.6 * 2 * 5 = 6. A two by
    // two correlation matrix with c off the diagonal has eigenvalues 1 + c and
    // 1 - c, so these are one and six tenths, and four tenths.
    //
    // The factorisation this reads is [[2, 0], [3, 4]], whose rows are of
    // length two and five, so the correlation factor is [[1, 0], [0.6, 0.8]]
    // and that times its own transpose is [[1, 0.6], [0.6, 1]].
    //
    // Two passes: the first turns the one pair there is, the second finds
    // nothing left to turn. A wrong angle takes more.
    let found = conditioning_of(&two_by_two("[[4.0, 6.0], [6.0, 25.0]]", IDENTITY));

    close_enough(found.smallest_correlation_eigenvalue(), 0.4);
    close_enough(found.correlation_eigenvalues()[1], 1.6);
    assert_eq!(found.correlation_sweeps(), 2);
}

#[test]
fn the_eigenvalues_come_out_smallest_first() {
    // The order is part of what this reports, because the smallest is the end a
    // reader is looking for and a list in whatever order the columns finished
    // in would make them hunt for it.
    let found = conditioning_of(&two_by_two("[[4.0, 6.0], [6.0, 25.0]]", IDENTITY));

    let values = found.correlation_eigenvalues();
    assert!(
        values[0] < values[1],
        "the eigenvalues came out as {values:?}, which is not smallest first"
    );
}

#[test]
fn a_design_with_orthogonal_columns_is_conditioned_by_their_lengths() {
    // Columns of length three and four with nothing between them, and a
    // covariance that whitens by dividing by one, so the design reaches the
    // arithmetic as it stands. Orthogonal columns are already singular vectors,
    // so the singular values are the lengths and the condition number is four
    // thirds. Every number here is exact in binary.
    let found = conditioning_of(&two_by_two(IDENTITY, "[[3.0, 0.0], [0.0, 4.0]]"));

    assert_eq!(found.design_singular_values(), [4.0, 3.0]);
    assert_eq!(found.design_condition_number(), 4.0 / 3.0);
    assert_eq!(found.design_sweeps(), 1);
}

#[test]
fn a_design_with_orthogonal_columns_of_one_length_is_left_alone() {
    // Two columns of equal length with nothing between them. This is the case
    // the turn cannot take: the angle it would go through is a ratio of two
    // differences that are both zero, so a pair skipped for the wrong reason,
    // or not skipped at all, produces numbers that are not numbers rather than
    // numbers that are slightly wrong.
    let found = conditioning_of(&two_by_two(IDENTITY, "[[3.0, 0.0], [0.0, 3.0]]"));

    assert_eq!(found.design_singular_values(), [3.0, 3.0]);
    assert_eq!(found.design_condition_number(), 1.0);
    assert_eq!(found.design_sweeps(), 1);
}

#[test]
fn the_condition_number_is_of_the_whitened_design_and_not_of_the_one_in_the_file() {
    // The design in the file is the identity, whose condition number is one,
    // and the case above shows that is what gets reported when the whitening is
    // a division by one. Here the covariance has variances four and sixteen, so
    // the factor is diagonal with two and four on it and the whitened design is
    // diagonal with one half and one quarter. Its singular values are those and
    // its condition number is two.
    //
    // The two cases together say which of the two matrices is read.
    let found = conditioning_of(&two_by_two("[[4.0, 0.0], [0.0, 16.0]]", IDENTITY));

    assert_eq!(found.design_singular_values(), [0.5, 0.25]);
    assert_eq!(found.design_condition_number(), 2.0);
}

#[test]
fn a_design_whose_two_columns_are_the_same_has_no_smallest_singular_value() {
    // Both parameters enter every observation identically, so no datum
    // distinguishes them and the design has rank one. The smaller singular
    // value is zero exactly and the condition number is infinite.
    //
    // That is the answer #2 asks for rather than a quiet pseudo-inverse: the
    // rank is not in question here, it is gone, and the singular values say so
    // where a solved-anyway result would not.
    let found = conditioning_of(&two_by_two(IDENTITY, "[[1.0, 1.0], [1.0, 1.0]]"));

    assert_eq!(found.design_singular_values()[1], 0.0);
    close_enough(found.design_singular_values()[0], 2.0);
    assert!(
        found.design_condition_number().is_infinite(),
        "a rank one design reported a finite condition number of {}",
        found.design_condition_number()
    );
}

#[test]
fn more_observations_than_parameters_is_the_ordinary_shape() {
    // Three observations of two parameters, which is what an adjustment looks
    // like and what the two-row cases cannot show. The covariance is the
    // identity so the design arrives as it stands.
    //
    // The columns are (2, 0, 0) and (0, 1, 2), of lengths two and the square
    // root of five, with nothing between them. So the singular values are those
    // two and the condition number is the square root of five over two.
    let found = conditioning_of(&document(
        &["alpha", "rydberg"],
        &["mass-one", "mass-two", "mass-three"],
        IDENTITY_THREE,
        "[[2.0, 0.0], [0.0, 1.0], [0.0, 2.0]]",
    ));

    assert_eq!(found.design_singular_values(), [5.0_f64.sqrt(), 2.0]);
    assert_eq!(found.design_condition_number(), 5.0_f64.sqrt() / 2.0);
    assert_eq!(found.correlation_eigenvalues(), [1.0, 1.0, 1.0]);
}

#[test]
fn three_parameters_are_three_pairs_of_columns_and_not_one() {
    // Every case above has two columns and therefore one pair, so the loop that
    // walks the pairs is a loop that runs once and a routine looking at only
    // the first pair would pass all of them. Three columns are three pairs.
    //
    // The columns are (1, 0, 0), (0, 2, 0) and (0, 0, 3), so the singular
    // values are one, two and three and the condition number is three.
    let found = conditioning_of(&document(
        &["alpha", "rydberg", "planck"],
        &["mass-one", "mass-two", "mass-three"],
        IDENTITY_THREE,
        "[[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]",
    ));

    assert_eq!(found.design_singular_values(), [3.0, 2.0, 1.0]);
    assert_eq!(found.design_condition_number(), 3.0);
}

#[test]
fn a_correlation_over_three_rows_takes_more_than_one_pass_to_settle() {
    // Three rows all asserted against each other, so no single pass of the
    // pairs leaves them orthogonal: turning the first pair moves both columns,
    // which is what the third pair then has to undo some of.
    //
    // The covariance is the identity with one half asserted between every pair,
    // scaled to variances of one, so the correlation matrix is its own
    // covariance. A matrix of ones on the diagonal and c everywhere else, of
    // order three, has eigenvalues 1 + 2c and 1 - c twice, so these are two and
    // one half, and one half twice.
    let found = conditioning_of(&document(
        &["alpha", "rydberg"],
        &["mass-one", "mass-two", "mass-three"],
        "[[1.0, 0.5, 0.5], [0.5, 1.0, 0.5], [0.5, 0.5, 1.0]]",
        "[[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]",
    ));

    close_enough(found.smallest_correlation_eigenvalue(), 0.5);
    close_enough(found.correlation_eigenvalues()[2], 2.0);
    assert!(
        found.correlation_sweeps() > 2,
        "three asserted rows settled in {} pass(es), which is fewer than the \
         pairs need",
        found.correlation_sweeps()
    );
    assert!(
        found.correlation_sweeps() < ausgleich_solve::conditioning::SWEEPS,
        "the rotations ran out of passes at {}, so the eigenvalues above are \
         where they were stopped rather than where they settled",
        found.correlation_sweeps()
    );
}

#[test]
fn the_singular_values_come_out_largest_first() {
    // The other end from the eigenvalues, and for the same reason: the largest
    // is what the condition number is read against.
    let found = conditioning_of(&document(
        &["alpha", "rydberg"],
        &["mass-one", "mass-two", "mass-three"],
        IDENTITY_THREE,
        "[[2.0, 0.0], [0.0, 1.0], [0.0, 2.0]]",
    ));

    let values = found.design_singular_values();
    assert!(
        values[0] > values[1],
        "the singular values came out as {values:?}, which is not largest first"
    );
}

#[test]
fn the_conditioning_says_what_it_is_when_it_is_printed() {
    // A diagnostic that prints as a name and nothing else is one somebody has
    // to open a debugger to read.
    let found = conditioning_of(&two_by_two("[[4.0, 0.0], [0.0, 16.0]]", IDENTITY));

    let printed = format!("{found:?}");
    assert!(
        printed.contains("correlation_eigenvalues") && printed.contains("0.25"),
        "the conditioning printed as {printed}, which does not carry its numbers"
    );
}
