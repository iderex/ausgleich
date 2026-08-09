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
//! What is here is the provenance block, #33, which is the part of every record
//! that says where a number came from, the correlation coefficient record that
//! carries one, #31, the input datum record, #30, the unit table with the
//! dimensional check, #34, and the adjusted constant record, #32, which is the
//! one record that declares a dimension for the check to judge a unit against.
//! The manifest is #35.

pub mod coefficient;
pub mod constant;
pub mod datum;
pub mod provenance;
pub mod units;
