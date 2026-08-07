//! The binary runs, says what it says, and exits with the code it promises.
//!
//! It exists for the coverage check as much as for the behaviour. A floor
//! measured against a workspace where nothing executable is ever executed is a
//! floor of zero, and a floor of zero is a number no change can fall below. One
//! test that actually runs the binary is what makes the measurement mean
//! something, and it is the smallest test that does.

use std::process::Command;

#[test]
fn the_binary_reports_that_there_is_no_command_surface_and_exits_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_ausgleich"))
        .output()
        .expect("the binary this crate builds can be run");

    assert_eq!(
        output.status.code(),
        Some(2),
        "a binary with nothing to run may not exit zero: exiting zero and \
         printing nothing reads as a run that worked"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no command surface"),
        "the binary should say why it did nothing, and said: {stderr:?}"
    );

    assert!(
        output.stdout.is_empty(),
        "nothing goes to stdout, which is where a result will go once there is \
         one: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}
