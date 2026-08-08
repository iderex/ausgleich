//! What the two files of the seam in #13 accept, what they refuse, and the
//! round trip that is the reason the seam is worth having.
//!
//! The two canonical documents below are written in this source rather than
//! stored beside it. The property under test is byte equality, and a fixture
//! whose bytes matter is a fixture git's own text normalisation may quietly
//! repair on the way in; `.gitattributes` says so in the paragraph that decided
//! it. Written here, what the compiler produces is what the test compares.
//!
//! Every malformed document is the canonical one with one substitution made in
//! it, so a reader can see the single difference that is being refused rather
//! than reading a second document to find out what changed. The substitutions
//! are the one-character kind somebody actually makes: a field left out, a
//! number written as text, a matrix one row short.

use ausgleich_solve::seam::{Refusal, read_problem, read_solution};

/// The problem file as the writer writes it. Nothing in it is a measurement;
/// the numbers are small so that the shortest form of each is obvious to a
/// reader checking the round trip by eye.
const PROBLEM: &str = r#"[header]
input_set = "first"
input_set_hash = "sha256:0000"
code_version = "0.0.0"
code_commit = "0000000"
toolchain = "1.85.0"
command_line = "ausgleich assemble \"first\" C:\\sets"

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
value = -2.25

[problem]
covariance = [
  [4.0, 0.5],
  [0.5, 9.0],
]
design = [
  [1.0, 0.0],
  [0.0, 1.0],
]
"#;

/// The result file as the writer writes it, for the problem above.
const RESULT: &str = r#"[header]
input_set = "first"
input_set_hash = "sha256:0000"
code_version = "0.0.0"
code_commit = "0000000"
toolchain = "1.85.0"
command_line = "ausgleich solve \"first\" C:\\sets"

[[parameter]]
identifier = "alpha"
value = 137.5

[[parameter]]
identifier = "rydberg"
value = 2.75

[[residual]]
identifier = "mass-one"
normalised = 0.25

[[residual]]
identifier = "mass-two"
normalised = -1.5

[result]
covariance = [
  [0.25, 0.0],
  [0.0, 1.0],
]

[convergence]
iterations = 3
converged = true
criterion = "the largest parameter step is below a millionth of its uncertainty"
final_step = 0.5
"#;

/// The canonical problem with one substitution made in it.
fn problem_with(from: &str, to: &str) -> String {
    assert!(
        PROBLEM.contains(from),
        "the substitution {from} is not in the canonical problem, so the test \
         below would be refusing the document it meant to keep"
    );
    PROBLEM.replace(from, to)
}

/// The canonical result with one substitution made in it.
fn result_with(from: &str, to: &str) -> String {
    assert!(
        RESULT.contains(from),
        "the substitution {from} is not in the canonical result, so the test \
         below would be refusing the document it meant to keep"
    );
    RESULT.replace(from, to)
}

/// The refusal a problem document earns, or a failure naming what it was
/// accepted as.
fn problem_refusal(document: &str) -> Refusal {
    match read_problem(document) {
        Ok(_) => panic!("the document was accepted:\n{document}"),
        Err(refusal) => refusal,
    }
}

/// The refusal a result document earns.
fn result_refusal(document: &str) -> Refusal {
    match read_solution(document) {
        Ok(_) => panic!("the document was accepted:\n{document}"),
        Err(refusal) => refusal,
    }
}

#[test]
fn writing_the_problem_back_gives_the_same_bytes() {
    // The property the seam is judged on. A reader that dropped a digit, a
    // writer that reordered a table, or a number format that lost the last bit
    // of a mantissa all show up here as a diff, and nowhere else.
    let problem = read_problem(PROBLEM).expect("the canonical problem reads");
    assert_eq!(problem.to_string(), PROBLEM);
}

#[test]
fn writing_the_result_back_gives_the_same_bytes() {
    let result = read_solution(RESULT).expect("the canonical result reads");
    assert_eq!(result.to_string(), RESULT);
}

#[test]
fn reading_what_was_written_gives_the_same_document_again() {
    // The round trip started from the other end, so that a document the writer
    // produced is one the reader takes, and not only the reverse.
    let once = read_problem(PROBLEM).expect("reads").to_string();
    let twice = read_problem(&once)
        .expect("what was written reads")
        .to_string();
    assert_eq!(once, twice);

    let once = read_solution(RESULT).expect("reads").to_string();
    let twice = read_solution(&once)
        .expect("what was written reads")
        .to_string();
    assert_eq!(once, twice);
}

#[test]
fn the_problem_carries_what_the_seam_promises() {
    let problem = read_problem(PROBLEM).expect("reads");
    let header = problem.header();
    assert_eq!(header.input_set(), "first");
    assert_eq!(header.input_set_hash(), "sha256:0000");
    assert_eq!(header.code_version(), "0.0.0");
    assert_eq!(header.code_commit(), "0000000");
    assert_eq!(header.toolchain(), "1.85.0");
    assert_eq!(
        header.command_line(),
        r#"ausgleich assemble "first" C:\sets"#
    );

    assert_eq!(problem.parameters().len(), 2);
    assert_eq!(problem.parameters()[0].identifier(), "alpha");
    assert_eq!(problem.parameters()[1].unit(), "m^-1");

    assert_eq!(problem.observations().len(), 2);
    assert_eq!(problem.observations()[0].identifier(), "mass-one");
    assert_eq!(problem.observations()[1].value(), -2.25);

    assert_eq!(problem.covariance()[0][1], 0.5);
    assert_eq!(problem.design()[1][1], 1.0);
}

#[test]
fn the_result_carries_what_the_seam_promises() {
    let result = read_solution(RESULT).expect("reads");
    assert_eq!(result.header().input_set(), "first");
    assert_eq!(result.parameters().len(), 2);
    assert_eq!(result.parameters()[0].identifier(), "alpha");
    assert_eq!(result.parameters()[0].value(), 137.5);
    assert_eq!(result.covariance()[0][0], 0.25);
    assert_eq!(result.residuals().len(), 2);
    assert_eq!(result.residuals()[1].identifier(), "mass-two");
    assert_eq!(result.residuals()[1].normalised(), -1.5);

    let convergence = result.convergence();
    assert_eq!(convergence.iterations(), 3);
    assert!(convergence.converged());
    assert!(
        convergence
            .criterion()
            .starts_with("the largest parameter step")
    );
    assert_eq!(convergence.final_step(), 0.5);
}

#[test]
fn both_files_can_be_shown_to_somebody_debugging_one() {
    // The shapes print their own fields. A seam whose values cannot be shown is
    // one where the first disagreement between two implementations turns into a
    // print statement somebody adds and then removes.
    let problem = read_problem(PROBLEM).expect("reads");
    assert!(format!("{problem:?}").contains("rydberg"));
    let result = read_solution(RESULT).expect("reads");
    assert!(format!("{result:?}").contains("iterations"));
    let refusal = problem_refusal(&problem_with("value = 1.5", "value = \"1.5\""));
    assert!(format!("{refusal:?}").contains("WrongKind"));
}

#[test]
fn a_whole_number_is_read_as_a_number() {
    // The format writes one as `1` as readily as `1.0`, and refusing the first
    // would be a refusal about notation. The writer settles on one of the two,
    // so the document that comes back is not the document that went in, and
    // that is the one direction the round trip does not promise.
    let problem = read_problem(&problem_with("value = 1.5", "value = 1")).expect("reads");
    assert_eq!(problem.observations()[0].value(), 1.0);
    assert!(problem.to_string().contains("value = 1.0"));
}

#[test]
fn bytes_that_are_not_a_document_are_refused() {
    let refusal = problem_refusal("[header");
    assert!(matches!(refusal, Refusal::Unreadable(_)));
    assert!(refusal.to_string().starts_with("the file is not readable"));
}

#[test]
fn a_document_with_no_header_is_refused() {
    let refusal = problem_refusal(&problem_with("[header]\n", ""));
    assert!(matches!(refusal, Refusal::Missing { .. }));
    assert_eq!(refusal.to_string(), "the file has no header");
}

#[test]
fn a_header_that_is_not_a_table_is_refused() {
    let refusal = problem_refusal(&problem_with("[header]\n", "header = 5\n"));
    assert_eq!(refusal.to_string(), "the file's header is not a table");
}

/// Every field the header carries, in the order the writer writes them.
const HEADER_FIELDS: &[&str] = &[
    "input_set",
    "input_set_hash",
    "code_version",
    "code_commit",
    "toolchain",
    "command_line",
];

/// The canonical problem with one line of its header taken out.
fn problem_without_header_field(field: &str) -> String {
    let opening = format!("{field} = ");
    let kept: Vec<&str> = PROBLEM
        .lines()
        .filter(|line| !line.starts_with(&opening))
        .collect();
    assert_eq!(
        kept.len() + 1,
        PROBLEM.lines().count(),
        "taking out {field} took out something other than one line"
    );
    format!("{}\n", kept.join("\n"))
}

#[test]
fn a_header_missing_any_one_of_its_fields_is_refused() {
    // Each field on its own rather than one of them standing for the other
    // five. A field read by a line no test exercised is a field that can be
    // dropped from the reader and leave every test green.
    for field in HEADER_FIELDS {
        let refusal = problem_refusal(&problem_without_header_field(field));
        assert_eq!(
            refusal.to_string(),
            format!("the file has no header.{field}")
        );
    }
}

#[test]
fn a_header_field_that_is_not_a_string_is_refused() {
    let refusal = problem_refusal(&problem_with("toolchain = \"1.85.0\"", "toolchain = 185"));
    assert_eq!(
        refusal.to_string(),
        "the file's header.toolchain is not a string"
    );
}

#[test]
fn a_string_carrying_a_control_character_is_refused() {
    // The writer escapes a quote and a backslash and nothing else, so a string
    // that arrived carrying a tab would leave as bytes that no longer read
    // back. This is the refusal that lets the writer be total.
    let refusal = problem_refusal(&problem_with(
        "input_set = \"first\"",
        "input_set = \"fi\\trst\"",
    ));
    assert!(matches!(refusal, Refusal::ControlCharacter { .. }));
    assert!(
        refusal
            .to_string()
            .starts_with("the file's header.input_set carries a character")
    );
}

#[test]
fn a_document_with_no_parameters_at_all_is_refused() {
    let refusal = problem_refusal(&problem_with("[[parameter]]", "[[was_a_parameter]]"));
    assert_eq!(refusal.to_string(), "the file has no parameter");
}

/// The two `[[parameter]]` blocks of the canonical problem, as one string, so a
/// test can put a list of its own in their place. It goes at the top of the
/// document rather than where the blocks were: a bare key written after
/// `[header]` would belong to the header, and the test would then be refusing
/// the wrong thing while looking as though it worked.
const PARAMETER_BLOCKS: &str = "[[parameter]]\nidentifier = \"alpha\"\nunit = \"1\"\n\n\
                                [[parameter]]\nidentifier = \"rydberg\"\nunit = \"m^-1\"\n\n";

/// The canonical problem with its parameter blocks replaced by `list`.
fn problem_with_parameter_list(list: &str) -> String {
    format!("{list}\n{}", problem_with(PARAMETER_BLOCKS, ""))
}

#[test]
fn an_empty_parameter_list_is_refused() {
    let refusal = problem_refusal(&problem_with_parameter_list("parameter = []"));
    assert!(matches!(refusal, Refusal::NoRows { .. }));
    assert!(
        refusal
            .to_string()
            .starts_with("the file lists no parameter")
    );
}

#[test]
fn a_parameter_list_holding_something_that_is_not_a_table_is_refused() {
    let refusal = problem_refusal(&problem_with_parameter_list("parameter = [\"alpha\"]"));
    assert_eq!(
        refusal.to_string(),
        "the file's parameter[0] is not a table"
    );
}

#[test]
fn a_parameter_missing_its_unit_is_refused() {
    let refusal = problem_refusal(&problem_with("unit = \"m^-1\"\n", ""));
    assert_eq!(refusal.to_string(), "the file has no parameter[1].unit");
}

#[test]
fn a_parameter_with_no_identifier_is_refused() {
    // One test per list that is read by identifier, because each list is read
    // by its own line and a refusal that never runs is a refusal nobody has
    // seen work.
    let refusal = problem_refusal(&problem_with(
        "identifier = \"alpha\"
",
        "",
    ));
    assert_eq!(
        refusal.to_string(),
        "the file has no parameter[0].identifier"
    );
}

#[test]
fn an_observation_with_no_identifier_is_refused() {
    let refusal = problem_refusal(&problem_with(
        "identifier = \"mass-one\"
",
        "",
    ));
    assert_eq!(
        refusal.to_string(),
        "the file has no observation[0].identifier"
    );
}

#[test]
fn a_fitted_parameter_with_no_identifier_is_refused() {
    let refusal = result_refusal(&result_with(
        "identifier = \"alpha\"
",
        "",
    ));
    assert_eq!(
        refusal.to_string(),
        "the file has no parameter[0].identifier"
    );
}

#[test]
fn a_residual_with_no_identifier_is_refused() {
    let refusal = result_refusal(&result_with(
        "identifier = \"mass-one\"
",
        "",
    ));
    assert_eq!(
        refusal.to_string(),
        "the file has no residual[0].identifier"
    );
}

#[test]
fn two_parameters_with_one_identifier_are_refused() {
    let refusal = problem_refusal(&problem_with(
        "identifier = \"rydberg\"",
        "identifier = \"alpha\"",
    ));
    assert!(matches!(refusal, Refusal::StatedTwice { .. }));
    assert!(
        refusal
            .to_string()
            .starts_with("two entries of parameter are called alpha")
    );
}

#[test]
fn two_observations_with_one_identifier_are_refused() {
    let refusal = problem_refusal(&problem_with(
        "identifier = \"mass-two\"",
        "identifier = \"mass-one\"",
    ));
    assert!(matches!(refusal, Refusal::StatedTwice { .. }));
}

#[test]
fn an_observation_with_no_value_is_refused() {
    let refusal = problem_refusal(&problem_with("value = -2.25\n", ""));
    assert_eq!(refusal.to_string(), "the file has no observation[1].value");
}

#[test]
fn an_observation_whose_value_is_not_a_number_is_refused() {
    let refusal = problem_refusal(&problem_with("value = 1.5", "value = \"1.5\""));
    assert_eq!(
        refusal.to_string(),
        "the file's observation[0].value is not a number"
    );
}

#[test]
fn an_observation_whose_value_is_not_finite_is_refused() {
    // The format reads `nan` and `inf` as numbers. Either one empties a row of
    // the fit and leaves a run that still prints a table.
    let refusal = problem_refusal(&problem_with("value = 1.5", "value = nan"));
    assert!(matches!(refusal, Refusal::NotAFiniteNumber { .. }));
    assert!(refusal.to_string().ends_with("which is not a number"));
}

#[test]
fn a_covariance_that_is_not_a_list_is_refused() {
    let refusal = problem_refusal(&problem_with(
        "covariance = [\n  [4.0, 0.5],\n  [0.5, 9.0],\n]",
        "covariance = 4",
    ));
    assert_eq!(
        refusal.to_string(),
        "the file's problem.covariance is not a list"
    );
}

#[test]
fn a_problem_with_no_covariance_is_refused() {
    let refusal = problem_refusal(&problem_with(
        "covariance = [\n  [4.0, 0.5],\n  [0.5, 9.0],\n]\n",
        "",
    ));
    assert_eq!(refusal.to_string(), "the file has no problem.covariance");
}

#[test]
fn a_covariance_one_row_short_is_refused() {
    let refusal = problem_refusal(&problem_with("  [0.5, 9.0],\n", ""));
    assert!(matches!(refusal, Refusal::WrongRowCount { .. }));
    assert_eq!(
        refusal.to_string(),
        "problem.covariance has 1 rows where the rest of the file fixes 2"
    );
}

#[test]
fn a_covariance_row_that_is_not_a_list_is_refused() {
    let refusal = problem_refusal(&problem_with("  [4.0, 0.5],", "  4.0,"));
    assert_eq!(
        refusal.to_string(),
        "the file's problem.covariance[0] is not a list"
    );
}

#[test]
fn a_covariance_row_of_the_wrong_length_is_refused() {
    let refusal = problem_refusal(&problem_with("  [4.0, 0.5],", "  [4.0, 0.5, 0.5],"));
    assert!(matches!(refusal, Refusal::WrongColumnCount { .. }));
    assert_eq!(
        refusal.to_string(),
        "problem.covariance[0] has 3 entries where the rest of the file fixes 2"
    );
}

#[test]
fn a_covariance_entry_that_is_not_a_number_is_refused() {
    let refusal = problem_refusal(&problem_with("  [4.0, 0.5],", "  [\"4.0\", 0.5],"));
    assert_eq!(
        refusal.to_string(),
        "the file's problem.covariance[0][0] is not a number"
    );
}

#[test]
fn a_covariance_that_disagrees_with_itself_is_refused() {
    // Symmetry is the one property of the covariance this file judges. Whether
    // it can be factorised is #56's, and a rule implemented twice can be
    // deleted from one place and stay green through the other.
    let refusal = problem_refusal(&problem_with("  [0.5, 9.0],", "  [0.25, 9.0],"));
    assert!(matches!(refusal, Refusal::NotSymmetric { .. }));
    assert_eq!(
        refusal.to_string(),
        "problem.covariance disagrees with itself at row 1 and column 0, and a \
         covariance is symmetric by construction"
    );
}

#[test]
fn a_design_matrix_of_the_wrong_width_is_refused() {
    // The width is the number of parameters and the height is the number of
    // observations, so the two are checked against different counts and a
    // matrix that is square by accident does not pass on that.
    let refusal = problem_refusal(&problem_with(
        "design = [\n  [1.0, 0.0],",
        "design = [\n  [1.0],",
    ));
    assert_eq!(
        refusal.to_string(),
        "problem.design[0] has 1 entries where the rest of the file fixes 2"
    );
}

#[test]
fn a_document_with_no_observations_at_all_is_refused() {
    let refusal = problem_refusal(&problem_with("[[observation]]", "[[was_an_observation]]"));
    assert_eq!(refusal.to_string(), "the file has no observation");
}

#[test]
fn a_problem_with_no_problem_table_is_refused() {
    let refusal = problem_refusal(&problem_with(
        "[problem]
",
        "",
    ));
    assert_eq!(refusal.to_string(), "the file has no problem");
}

#[test]
fn a_result_that_is_not_a_document_is_refused() {
    // The two files are read by two functions, so a refusal is proved on both
    // rather than on whichever one was written first.
    let refusal = result_refusal("[header");
    assert!(matches!(refusal, Refusal::Unreadable(_)));
}

#[test]
fn a_result_with_no_header_is_refused() {
    let refusal = result_refusal(&result_with(
        "[header]
",
        "",
    ));
    assert_eq!(refusal.to_string(), "the file has no header");
}

#[test]
fn a_result_with_no_parameters_at_all_is_refused() {
    let refusal = result_refusal(&result_with("[[parameter]]", "[[was_a_parameter]]"));
    assert_eq!(refusal.to_string(), "the file has no parameter");
}

#[test]
fn a_fitted_parameter_whose_value_is_not_a_number_is_refused() {
    let refusal = result_refusal(&result_with("value = 137.5", "value = \"137.5\""));
    assert_eq!(
        refusal.to_string(),
        "the file's parameter[0].value is not a number"
    );
}

#[test]
fn two_fitted_parameters_with_one_identifier_are_refused() {
    let refusal = result_refusal(&result_with(
        "identifier = \"rydberg\"",
        "identifier = \"alpha\"",
    ));
    assert!(matches!(refusal, Refusal::StatedTwice { .. }));
}

#[test]
fn a_result_with_no_result_table_is_refused() {
    let refusal = result_refusal(&result_with("[result]\n", ""));
    assert_eq!(refusal.to_string(), "the file has no result");
}

#[test]
fn a_result_covariance_of_the_wrong_order_is_refused() {
    let refusal = result_refusal(&result_with("  [0.0, 1.0],\n", ""));
    assert_eq!(
        refusal.to_string(),
        "result.covariance has 1 rows where the rest of the file fixes 2"
    );
}

#[test]
fn a_result_covariance_that_disagrees_with_itself_is_refused() {
    let refusal = result_refusal(&result_with("  [0.0, 1.0],", "  [0.5, 1.0],"));
    assert!(matches!(refusal, Refusal::NotSymmetric { .. }));
}

#[test]
fn a_result_with_no_residuals_is_refused() {
    let refusal = result_refusal(&result_with("[[residual]]", "[[was_a_residual]]"));
    assert_eq!(refusal.to_string(), "the file has no residual");
}

#[test]
fn a_residual_with_no_number_is_refused() {
    let refusal = result_refusal(&result_with("normalised = -1.5\n", ""));
    assert_eq!(
        refusal.to_string(),
        "the file has no residual[1].normalised"
    );
}

#[test]
fn two_residuals_for_one_observation_are_refused() {
    let refusal = result_refusal(&result_with(
        "identifier = \"mass-two\"",
        "identifier = \"mass-one\"",
    ));
    assert!(matches!(refusal, Refusal::StatedTwice { .. }));
}

#[test]
fn a_result_with_no_convergence_record_is_refused() {
    let refusal = result_refusal(&result_with("[convergence]\n", ""));
    assert_eq!(refusal.to_string(), "the file has no convergence");
}

#[test]
fn a_convergence_record_with_no_iteration_count_is_refused() {
    let refusal = result_refusal(&result_with("iterations = 3\n", ""));
    assert_eq!(
        refusal.to_string(),
        "the file has no convergence.iterations"
    );
}

#[test]
fn an_iteration_count_that_is_not_a_whole_number_is_refused() {
    let refusal = result_refusal(&result_with("iterations = 3", "iterations = 3.5"));
    assert_eq!(
        refusal.to_string(),
        "the file's convergence.iterations is not a whole number"
    );
}

#[test]
fn an_iteration_count_below_zero_is_refused() {
    let refusal = result_refusal(&result_with("iterations = 3", "iterations = -1"));
    assert!(matches!(refusal, Refusal::CountBelowZero { .. }));
    assert_eq!(
        refusal.to_string(),
        "the file's convergence.iterations is -1, and a number of iterations is \
         never below zero"
    );
}

#[test]
fn a_convergence_record_that_does_not_say_whether_it_converged_is_refused() {
    let refusal = result_refusal(&result_with("converged = true\n", ""));
    assert_eq!(refusal.to_string(), "the file has no convergence.converged");
}

#[test]
fn a_converged_field_that_is_not_a_yes_or_a_no_is_refused() {
    // The one that matters. A run that spent its budget and stopped writes
    // `false` here, and a file where that field is a word instead is a file a
    // reader can misread as success.
    let refusal = result_refusal(&result_with("converged = true", "converged = \"yes\""));
    assert_eq!(
        refusal.to_string(),
        "the file's convergence.converged is not true or false"
    );
}

#[test]
fn a_convergence_record_with_no_criterion_is_refused() {
    let refusal = result_refusal(&result_with(
        "criterion = \"the largest parameter step is below a millionth of its uncertainty\"\n",
        "",
    ));
    assert_eq!(refusal.to_string(), "the file has no convergence.criterion");
}

#[test]
fn a_convergence_record_with_no_final_step_is_refused() {
    let refusal = result_refusal(&result_with("final_step = 0.5\n", ""));
    assert_eq!(
        refusal.to_string(),
        "the file has no convergence.final_step"
    );
}
