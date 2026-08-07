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

/// A library item newer than the floor, with the lint that catches it silenced,
/// so the msrv build check has something only it can refuse.
///
/// u32::is_multiple_of is stable on the pinned toolchain and unstable on the
/// floor. clippy::incompatible_msrv sees that from rust-version alone, and the
/// allow below is what somebody writes when they meet that error and read it as
/// noise. After the allow, every check that runs on the pinned toolchain is
/// green and the code still does not build for the operator the floor exists
/// for. That is the gap this check closes.
#[allow(
    clippy::incompatible_msrv,
    reason = "the fixture exists to prove the floor build refuses what a silenced lint lets through"
)]
pub fn divides_evenly(n: u32, by: u32) -> bool {
    n.is_multiple_of(by)
}
