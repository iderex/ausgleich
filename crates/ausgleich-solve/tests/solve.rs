//! What the solve produces, what it refuses, and what each number is checked
//! against, #57.
//!
//! Every expected number below comes from arithmetic the solve does not do. The
//! estimates and the covariance of the line fit are the closed form for a
//! straight line through three points, written out from the inverse of a two by
//! two cross-product, and that matrix is one the module never forms. The
//! reflections in the hand-worked case were done on paper. Neither is a number
//! this code produced on the day and was then asserted against itself.
//!
//! Where a case can be exact it is compared for equality, and where it cannot
//! the allowance is stated once, in units of the last place, rather than picked
//! per case. A tolerance chosen per assertion is a tolerance chosen after
//! seeing the answer.

use ausgleich_solve::seam::read_problem;
use ausgleich_solve::solve::{Fit, Refusal, solve};
use ausgleich_solve::whiten::{Whitened, whiten};

/// How far a computed number may sit from the number the algebra gives, in
/// units of the last place of the expected answer.
///
/// Sixteen, because each number below is a handful of rounded operations away
/// from its definition and each one can carry half a unit: the reflections, the
/// back substitution, the inversion of the triangle and the product that makes
/// the covariance. It is far below any difference a reader of this program
/// would act on and far above what the arithmetic here can accumulate, which is
/// what a tolerance is for. It is stated once so that a case cannot be admitted
/// by widening its own.
const UNITS_IN_THE_LAST_PLACE: f64 = 16.0;

/// Fail unless `found` sits within the stated allowance of `expected`.
fn same(found: f64, expected: f64, what: &str) {
    let allowance = UNITS_IN_THE_LAST_PLACE * f64::EPSILON * expected.abs().max(1.0);
    let apart = (found - expected).abs();
    assert!(
        apart <= allowance,
        "{what}: the solve gives {found:?} and the algebra gives {expected:?}, \
         which are {apart:?} apart against an allowance of {allowance:?}"
    );
}

/// A problem document, with the parameters, the observations, the covariance
/// and the design written in.
///
/// Assembled here rather than kept as fixture files, because every case in this
/// file turns on one of the four and a reader should see which one without
/// opening a second document.
fn document(
    parameters: &[&str],
    observations: &[(&str, f64)],
    covariance: &str,
    design: &str,
) -> String {
    let mut text = String::from(
        "[header]\n\
         input_set = \"first\"\n\
         input_set_hash = \"sha256:0000\"\n\
         code_version = \"0.0.0\"\n\
         code_commit = \"0000000\"\n\
         toolchain = \"1.85.0\"\n\
         command_line = \"ausgleich adjust\"\n",
    );
    for identifier in parameters {
        text.push_str(&format!(
            "\n[[parameter]]\nidentifier = \"{identifier}\"\nunit = \"1\"\n"
        ));
    }
    for (identifier, value) in observations {
        text.push_str(&format!(
            "\n[[observation]]\nidentifier = \"{identifier}\"\nvalue = {value:?}\n"
        ));
    }
    text.push_str(&format!(
        "\n[problem]\ncovariance = {covariance}\ndesign = {design}\n"
    ));
    text
}

/// The whitened problem this document holds, or a failure naming which layer
/// refused it.
///
/// A document this file writes badly should stop the test that wrote it rather
/// than arrive at the solve as something else.
fn whitened(text: &str) -> Whitened {
    let problem = match read_problem(text) {
        Ok(found) => found,
        Err(refusal) => panic!("the seam refused a document this test wrote: {refusal}"),
    };
    match whiten(&problem) {
        Ok(found) => found,
        Err(refusal) => panic!("the factorisation refused a covariance this test wrote: {refusal}"),
    }
}

/// The straight line through three points, as a problem.
///
/// Two parameters, the intercept and the slope, and three observations at
/// abscissae zero, one and two with an uncorrelated uncertainty of one each. It
/// is the smallest problem that is over-determined, has a design whose columns
/// are neither orthogonal nor the same, and has an answer anybody can work out
/// from the closed form.
fn line_fit() -> Fit {
    let text = document(
        &["intercept", "slope"],
        &[("first", 1.0), ("second", 3.0), ("third", 4.0)],
        "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[1.0, 0.0], [1.0, 1.0], [1.0, 2.0]]",
    );
    match solve(&whitened(&text)) {
        Ok(found) => found,
        Err(refusal) => panic!("the line fit is determined by its data: {refusal}"),
    }
}

#[test]
fn the_line_fit_agrees_with_the_closed_form_answer() {
    // The cross-product of that design is [[3, 3], [3, 5]], its determinant is
    // 6, and the right-hand side is [8, 11], so the closed-form answer is
    //
    //   intercept = 7 / 6      slope = 3 / 2
    //
    // The solve forms none of those matrices. It reflects the design onto a
    // triangle and substitutes back through it, so the two routes to these
    // numbers share no arithmetic.
    let fit = line_fit();

    same(fit.parameters()[0], 7.0 / 6.0, "the intercept");
    same(fit.parameters()[1], 3.0 / 2.0, "the slope");
}

#[test]
fn the_reflection_is_taken_away_from_the_leading_entry() {
    // The first column of that design is three ones, so the length of it is the
    // square root of three, and the leading entry is positive. The reflection
    // is taken away from that entry, which puts the negative of the length on
    // the diagonal.
    //
    // This is the only place the choice is visible. Both signs factor the
    // matrix, and the estimates and the covariance are the same under either,
    // so a suite that asserts only those two would let the arithmetic that
    // loses digits by cancellation land without a word.
    let fit = line_fit();

    assert_eq!(fit.triangle()[0][0], -(3.0_f64).sqrt());
}

#[test]
fn the_diagonal_is_the_length_rather_than_the_arithmetic_that_lands_beside_it() {
    // The turn puts the length of the column on the diagonal by construction,
    // and on this design the arithmetic that carries the column through the
    // turn lands a unit in the last place away from that length. The reduction
    // writes the length.
    //
    // The column here is [-8, -2], whose length is the square root of 68, and
    // the leading entry is negative, so the turn is taken the other way and the
    // diagonal is positive.
    //
    // A diagonal that is one unit out is not one number being slightly wrong.
    // Every estimate is divided by it and every entry of the covariance is
    // divided by it twice, so it is the entry of the triangle that reaches
    // furthest.
    let text = document(
        &["first", "second"],
        &[("one", 1.0), ("two", 2.0)],
        "[[1.0, 0.0], [0.0, 1.0]]",
        "[[-8.0, -3.0], [-2.0, -9.0]]",
    );

    let fit = match solve(&whitened(&text)) {
        Ok(found) => found,
        Err(refusal) => panic!("two data and two constants that are told apart: {refusal}"),
    };

    assert_eq!(fit.triangle()[0][0], (68.0_f64).sqrt());
}

#[test]
fn the_zeros_under_the_diagonal_are_written_rather_than_left_to_the_arithmetic() {
    // The reflection sends everything under the diagonal to zero by
    // construction and the arithmetic arrives near that rather than at it. On
    // the line fit above it lands exactly on zero, which is luck rather than a
    // property, so the case here is a design where it does not: the reduction
    // leaves a residue under the diagonal and the entry a reader is handed is
    // the zero the definition fixes instead.
    //
    // Nothing is asserted about the fit itself. This case exists for one entry
    // of the triangle, and a design with a closed-form answer would have been a
    // design whose arithmetic came out exact.
    let text = document(
        &["first", "second"],
        &[("one", 1.0), ("two", 2.0), ("three", 3.0)],
        "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[-8.0, 8.0], [-5.0, 0.0], [4.0, -5.0]]",
    );

    let fit = match solve(&whitened(&text)) {
        Ok(found) => found,
        Err(refusal) => panic!("these two columns are not one column: {refusal}"),
    };

    assert_eq!(fit.triangle()[1][0], 0.0);
}

#[test]
fn the_covariance_is_the_closed_form_inverse_of_the_cross_product() {
    // With the determinant at 6, the inverse of [[3, 3], [3, 5]] is
    //
    //   [[ 5 / 6, -1 / 2],
    //    [-1 / 2,  1 / 2]]
    //
    // which is what the covariance of a whitened least-squares fit is. Nothing
    // in the module inverts that matrix: it inverts the triangle and multiplies
    // that by its own transpose.
    let fit = line_fit();

    same(fit.covariance()[0][0], 5.0 / 6.0, "the intercept variance");
    same(fit.covariance()[0][1], -1.0 / 2.0, "the covariance above");
    same(fit.covariance()[1][0], -1.0 / 2.0, "the covariance below");
    same(fit.covariance()[1][1], 1.0 / 2.0, "the slope variance");
}

#[test]
fn the_covariance_undoes_the_triangle_the_estimates_came_out_of() {
    // The property that says the two outputs belong to one another rather than
    // to two problems: the triangle transposed, times the triangle, times the
    // covariance, is the identity. It is asserted over the reported triangle
    // and the reported covariance, so a covariance computed from anything but
    // that triangle moves it.
    //
    // The product is formed here, in a test, which is the one place forming it
    // costs nothing: the digits it spends are spent on checking an answer that
    // has already been computed without it.
    let fit = line_fit();
    let triangle = fit.triangle();
    let covariance = fit.covariance();
    let width = triangle.len();

    for row in 0..width {
        for column in 0..width {
            let mut found = 0.0;
            for (middle, above) in covariance.iter().enumerate() {
                let mut squared = 0.0;
                for entries in triangle {
                    squared += entries[row] * entries[middle];
                }
                found += squared * above[column];
            }
            let expected = if row == column { 1.0 } else { 0.0 };
            same(
                found,
                expected,
                &format!("entry {row},{column} of the product"),
            );
        }
    }
}

#[test]
fn the_uncertainties_and_the_correlations_come_off_the_covariance() {
    // The square roots of five sixths and of a half, and a correlation of the
    // negative square root of three fifths, since
    //
    //   -1 / 2 divided by the square root of 5 / 6 times 1 / 2
    //
    // is the negative square root of 3 / 5. The two uncertainties differ, which
    // is what makes this case able to tell a division by the pair from a
    // division by either one of them twice.
    let fit = line_fit();
    let spread = fit.uncertainties();
    let correlations = fit.correlations();

    same(
        spread[0],
        (5.0_f64 / 6.0).sqrt(),
        "the intercept uncertainty",
    );
    same(spread[1], (1.0_f64 / 2.0).sqrt(), "the slope uncertainty");
    same(correlations[0][0], 1.0, "a constant with itself");
    same(correlations[1][1], 1.0, "the other with itself");
    same(
        correlations[0][1],
        -(3.0_f64 / 5.0).sqrt(),
        "the correlation above the diagonal",
    );
    same(
        correlations[1][0],
        -(3.0_f64 / 5.0).sqrt(),
        "the correlation below the diagonal",
    );
}

#[test]
fn the_reflections_are_the_ones_worked_out_by_hand() {
    // A design chosen so that every square root and every division in the
    // reduction is exact in binary, which is why this case compares for
    // equality and the ones above do not.
    //
    //   design [[0, 5], [3, 0], [4, 0]]      observations [25, 50, 100]
    //
    // The first column is [0, 3, 4], whose length is 5. Its leading entry is
    // neither positive nor negative, and the reflection is taken away from it,
    // so the diagonal is -5. The reflector is [5, 3, 4] and the square of its
    // length is 50. It sends the second column [5, 0, 0] to [0, -3, -4] and the
    // observations to [-110, -31, -8].
    //
    // The second column then leads with -3, so the reflection is taken away in
    // the other direction and the diagonal is +5. The reflector is [-8, -4],
    // the square of its length is 80, and the observations become [25, 20].
    //
    // Back substitution gives 25 / 5 = 5 for the slope and -110 / -5 = 22 for
    // the intercept.
    //
    // Both signs of the diagonal appear here, and one of them appears on a
    // leading entry that is exactly zero. That is the case a comparison written
    // as "not greater than zero" or as "different from zero" gets wrong while
    // every other case in this file stays green.
    let text = document(
        &["first", "second"],
        &[("one", 25.0), ("two", 50.0), ("three", 100.0)],
        "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[0.0, 5.0], [3.0, 0.0], [4.0, 0.0]]",
    );

    let fit = match solve(&whitened(&text)) {
        Ok(found) => found,
        Err(refusal) => panic!("this design has two directions in it: {refusal}"),
    };

    assert_eq!(fit.triangle(), [vec![-5.0, 0.0], vec![0.0, 5.0]]);
    assert_eq!(fit.parameters(), [22.0, 5.0]);
    same(fit.covariance()[0][0], 1.0 / 25.0, "the first variance");
    same(fit.covariance()[1][1], 1.0 / 25.0, "the second variance");
    same(fit.covariance()[0][1], 0.0, "the covariance between them");
}

#[test]
fn a_column_the_others_already_reach_is_refused_with_the_singular_values() {
    // The second column is twice the first, so the two parameters are one
    // parameter and every pair that adds up the same way fits identically. The
    // reduction sends the first column onto the diagonal and the second is left
    // with nothing below it, which is where this is caught.
    //
    // The singular values say more than the refusal alone: the square root of
    // 45 and a zero, so one direction is determined and one is absent, rather
    // than the whole problem being weak.
    let text = document(
        &["first", "second"],
        &[("one", 1.0), ("two", 2.0), ("three", 3.0)],
        "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[1.0, 2.0], [2.0, 4.0], [2.0, 4.0]]",
    );

    let refusal = match solve(&whitened(&text)) {
        Ok(fit) => panic!("a design of one column twice has no unique fit: {fit:?}"),
        Err(refusal) => refusal,
    };

    let Refusal::RankDeficient {
        column,
        singular_values,
    } = &refusal;
    assert_eq!(*column, 1);
    assert_eq!(singular_values.len(), 2);
    same(singular_values[0], (45.0_f64).sqrt(), "the singular value");
    assert_eq!(singular_values[1], 0.0);
}

#[test]
fn fewer_data_than_parameters_runs_out_at_the_column_with_no_rows_left() {
    // One datum and two adjusted constants. There is nothing wrong with any
    // number in this problem and it still determines neither parameter, which
    // is why the refusal is about a column rather than about a value.
    //
    // It arrives at the same refusal by a different route from the case above:
    // there the column had rows and they were all zero, here the column has no
    // rows at all. The second is what a design with more columns than rows does
    // at every column past the last row.
    let text = document(
        &["first", "second"],
        &[("one", 1.0)],
        "[[1.0]]",
        "[[1.0, 2.0]]",
    );

    let refusal = match solve(&whitened(&text)) {
        Ok(fit) => panic!("one datum cannot determine two constants: {fit:?}"),
        Err(refusal) => refusal,
    };

    let Refusal::RankDeficient {
        column,
        singular_values,
    } = &refusal;
    assert_eq!(*column, 1);
    assert_eq!(singular_values[1], 0.0);
}

#[test]
fn the_refusal_names_the_column_and_says_it_returns_no_answer_in_its_place() {
    // The reason the text is asserted rather than left to a reader: "rank
    // deficient" on its own sends somebody to read the whole input set, and a
    // reader who takes the refusal for a numerical accident goes looking for a
    // tolerance to widen. The column, the singular values and the sentence
    // about what is not returned are the message doing its job.
    let text = document(
        &["first", "second"],
        &[("one", 1.0), ("two", 2.0), ("three", 3.0)],
        "[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]",
        "[[1.0, 2.0], [2.0, 4.0], [2.0, 4.0]]",
    );

    let refusal = match solve(&whitened(&text)) {
        Ok(fit) => panic!("a design of one column twice has no unique fit: {fit:?}"),
        Err(refusal) => refusal,
    };

    let said = refusal.to_string();
    assert!(said.contains("column 1"), "{said}");
    assert!(said.contains("0.0"), "{said}");
    assert!(said.contains("pseudo-inverse"), "{said}");
    assert!(format!("{refusal:?}").contains("RankDeficient"));
}
