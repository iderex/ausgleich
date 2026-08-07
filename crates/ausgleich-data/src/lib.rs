//! The record types for an input set, and the loader that reads them off disk.
//!
//! This crate owns two things that are deliberately kept apart. The record
//! types are plain data with no idea where they came from, so anything can
//! build one in a test. The loader is the only place in the workspace, outside
//! the command line, that reads a path.
//!
//! It sees no other member of this workspace. The solver in particular is
//! invisible from here, and the reverse holds too, so nothing in the solve path
//! can come to depend on a directory existing.
//!
//! Empty for now. The record shapes are #30, #31 and #32, the manifest is #35,
//! and the units table is #34. The crate exists ahead of them so that the
//! dependency arrows are asserted by a test before there is code to route
//! around them.

/// One dead function, and nothing else on this branch.
///
/// It exists to red the `build` check and only that check. `dead_code` is a
/// rustc lint, so clippy reads it too, and a fixture that reds two checks
/// proves neither of them by itself. Clippy runs with `--cfg clippy` set, so
/// the attribute below silences the lint there and leaves it standing for every
/// other route that compiles this crate.
#[cfg_attr(clippy, allow(dead_code))]
fn a_function_nobody_calls() {}
