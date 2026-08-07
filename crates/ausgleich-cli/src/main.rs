//! The command line.
//!
//! The only crate that reads arguments, and the only one besides the loader that
//! touches the filesystem. It is where the four libraries are assembled into a
//! run, which is why it is the one member allowed to see all of them.

fn main() -> std::process::ExitCode {
    // There is no command surface yet. What an operator types is #78, and
    // guessing at it here would put a shape in the tree that nobody argued for.
    // Failing loudly is the honest placeholder: a binary that exits zero and
    // prints nothing reads as a run that worked.
    eprintln!("ausgleich: no command surface yet, so there is nothing to run");
    std::process::ExitCode::from(2)
}
