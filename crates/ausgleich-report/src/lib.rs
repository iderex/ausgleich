//! The report an operator reads.
//!
//! It renders a result together with the input set that produced it, which is
//! why it sees both the record types and the solve's own types and nothing else.
//! It writes no file: the caller decides where the bytes go, so a test can read
//! a report without a temporary directory.
//!
//! Empty for now. The report is #79, and what has to be in the header of every
//! output file is #3.
