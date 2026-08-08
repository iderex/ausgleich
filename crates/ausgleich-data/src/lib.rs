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
//! that says where a number came from, and the correlation coefficient record
//! that carries one, #31. The other record shapes are #30 and #32, the manifest
//! is #35, and the units table is #34.

pub mod coefficient;
pub mod provenance;
