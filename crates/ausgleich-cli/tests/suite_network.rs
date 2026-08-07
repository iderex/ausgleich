//! The network-bound suite: anything that fetches a source document over the
//! network.
//!
//! It is a separate invocation and is not built at all by an ordinary run. The
//! manifest holds it back with `required-features = ["suite-network"]`, and
//! `tests/suites.rs` refuses a target held back that way without an entry on
//! its roster, so this suite cannot become a thing the default run neither runs
//! nor mentions.
//!
//! Invoke it deliberately:
//!
//! ```text
//! cargo test --package ausgleich-cli --features suite-network --test suite_network
//! ```
//!
//! It holds no test that reaches a network today, and that is a statement about
//! the tree rather than about the suite. Nothing here fetches anything: the
//! source fetch tool is #83 and does not exist, and until it does there is
//! nothing for this suite to be about. The suite is here first because the
//! split is a birth requirement in #14 rather than a cleanup, and because a
//! suite invented at the moment somebody needs it is a suite invented under
//! pressure to make one test pass.

use std::io::Write;

#[test]
fn this_suite_says_what_it_is_and_what_it_does_not_yet_hold() {
    // Written to the process stderr handle rather than through a macro, for the
    // reason the announcement in tests/suites.rs is: the harness captures the
    // macros, and a disclosure that only appears on a failing run is not a
    // disclosure.
    //
    // There is no assertion here and no invented one. A test asserting
    // `cfg!(feature = "suite-network")` would be true by construction, since the
    // target does not compile without the feature, and a tautology dressed as a
    // check is worse than no check. What this run proves is that the invocation
    // above resolves, compiles and exits green.
    let mut out = std::io::stderr();
    writeln!(out, "\nnetwork-bound suite").expect("the process stderr handle accepts a line");
    writeln!(
        out,
        "  no test here reaches a network. The fetch tool is #83 and is not in \
         the tree,\n  so this suite is the empty half of a split that exists \
         before the work does."
    )
    .expect("the process stderr handle accepts a line");
}
