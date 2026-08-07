//! One default-suite test that binds a socket off loopback, and nothing else on
//! this branch.
//!
//! It exists to red the loopback guard in `tests/suites.rs`. The guard reads the
//! source text, so it refuses this file whether or not the test ever runs, and
//! the test carries `#[ignore]` so that no run anywhere actually binds off
//! loopback. On the maintainer's machine that bind would raise a firewall
//! dialog, which is the interruption the rule exists to prevent, and a fixture
//! that causes the harm it is demonstrating is not a fixture worth having.

use std::net::TcpListener;

#[test]
#[ignore = "the guard reads the source, so this never has to run, and running it \
            would raise the firewall dialog the rule exists to prevent"]
fn a_test_that_binds_every_interface() {
    let listener = TcpListener::bind("0.0.0.0:0").expect("a socket binds");
    drop(listener);
}
