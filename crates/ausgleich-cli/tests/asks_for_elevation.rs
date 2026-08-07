//! One default-suite test that shells out to a tool needing a privileged
//! account, and nothing else on this branch.
//!
//! It exists to red the elevation rule in `tests/suites.rs`. The rule reads the
//! source text, so it refuses this file whether or not the test runs, and the
//! test carries `#[ignore]` so that no run anywhere raises the prompt the rule
//! exists against. On the machine this repository is written on, that prompt
//! takes the maintainer's attention for a run they did not start.

use std::process::Command;

#[test]
#[ignore = "the rule reads the source, so this never has to run, and running it \
            would raise the consent prompt the rule exists to prevent"]
fn a_test_that_reconfigures_the_firewall() {
    let status = Command::new("netsh")
        .args(["advfirewall", "firewall", "show", "rule", "name=all"])
        .status()
        .expect("the tool runs");
    assert!(status.success());
}
