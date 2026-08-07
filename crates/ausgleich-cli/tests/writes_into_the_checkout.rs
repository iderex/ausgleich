//! One default-suite test that writes into the checkout, and nothing else on
//! this branch.
//!
//! It exists to red the write rule in `tests/suites.rs`. The rule reads the
//! source text, so it refuses this file whether or not the test runs, and the
//! test carries `#[ignore]` so that no run anywhere actually leaves a file
//! behind in somebody's working tree.

use std::fs;

#[test]
#[ignore = "the rule reads the source, so this never has to run, and running it \
            would leave the file in the checkout that the rule exists against"]
fn a_test_that_leaves_a_file_in_the_tree() {
    fs::write("scratch.txt", b"left behind").expect("a file is written");
}
