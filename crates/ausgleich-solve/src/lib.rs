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
//! Empty for now. The solve is #56 through #63, and the file seam that lets a
//! different fit read the same problem is #13.

/// An uncertainty enlargement applied outside the loader, so the third
/// invariant has something to refuse. Not for merging.
pub fn apply_expansion_factor(uncertainty: f64, factor: f64) -> f64 {
    uncertainty * factor
}
