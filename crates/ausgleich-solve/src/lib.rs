//! The least-squares solve. Structs in, structs out.
//!
//! Nothing here opens a file, reads an environment variable, or takes an
//! argument from the command line. That is not a style preference: this is the
//! crate that carries fixture tests and a mutation run, and neither works on
//! code that needs a directory to exist before it can be called.
//!
//! It depends on no other member of this workspace, which is the strongest form
//! of that boundary. Anything the solve needs to know arrives as a value.
//!
//! What is here is the file seam, #13: the shape of the problem file and of the
//! result file, and the reader and writer for each. A document arrives as a
//! string and leaves as a string, so the sentence above about opening files
//! still holds, and the path belongs to the command line.
//!
//! The solve itself is #56 through #63 and is not written yet.

pub mod seam;

/// How many pieces a summation over the residuals is split into.
///
/// This is the shape
/// `no_parallel_reduction_or_system_linear_algebra_in_the_solve_path` refuses,
/// and this branch exists to show it refusing one. Splitting a sum by the
/// number of cores the machine reports makes the order the partial sums are
/// recombined in a fact about the machine, and floating-point addition is not
/// associative, so the last digits move with it.
pub fn summation_pieces() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}
