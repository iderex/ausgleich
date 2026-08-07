//! The long-running suite: anything that samples or sweeps, and therefore
//! takes minutes rather than seconds.
//!
//! A separate invocation, held back by `required-features = ["suite-long"]` and
//! named on the roster in `tests/suites.rs`, which refuses a target held back
//! without an entry there.
//!
//! Invoke it deliberately:
//!
//! ```text
//! cargo test --package ausgleich-cli --features suite-long --test suite_long
//! ```
//!
//! It holds no sweep today. The things that will sample or sweep are the
//! leave-one-out runs in #70, the mutation run in #63 and the numerical
//! validation in #62, and none of them exists yet. The suite is here first
//! because #14 makes the split a birth requirement: a long test written before
//! there is anywhere to put it goes into the default suite, and the default
//! suite then quietly takes minutes.

use std::io::Write;

#[test]
fn this_suite_says_what_it_is_and_what_it_does_not_yet_hold() {
    // No assertion, and no invented one, for the reason written in
    // tests/suite_network.rs. What this run proves is that the invocation
    // resolves, compiles and exits green.
    let mut out = std::io::stderr();
    writeln!(out, "\nlong-running suite").expect("the process stderr handle accepts a line");
    writeln!(
        out,
        "  no test here samples or sweeps. The runs that will are #62, #63 and \
         #70,\n  so this suite is the empty half of a split that exists before \
         the work does."
    )
    .expect("the process stderr handle accepts a line");
}
