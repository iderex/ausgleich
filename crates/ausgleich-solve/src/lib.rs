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
//! Also here is the factorisation and the whitening, #56: the covariance comes
//! out of the problem as a factor, and the observation vector and the design
//! matrix are whitened with it. The solve that is handed the result is #57 and
//! is not written yet.
//!
//! Beside it are the two numbers #56 asks a run to print about the whitened
//! problem, the smallest eigenvalue of the correlation matrix and the condition
//! number of the whitened design matrix. They are computed here and printed
//! nowhere, because there is no run in this workspace to print them into.

pub mod conditioning;
pub mod seam;
pub mod whiten;
