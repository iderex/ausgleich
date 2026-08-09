//! Properties of this tree that a pattern over it can refuse, refused here
//! rather than in a reviewer's head.
//!
//! Each invariant is one test, with the reason it exists written next to it and
//! the bound on what it can see written next to that. A pattern that catches a
//! shape and not a meaning says so, because a reader who believes it catches the
//! meaning stops looking for the meaning.
//!
//! The patterns run through `git grep` rather than over a directory walk. That
//! is deliberate: git decides what is tracked, so a file nobody committed cannot
//! quietly satisfy a check, and an ignored build directory cannot fail one.
//!
//! Every scope below is a git pathspec. A pathspec naming a directory that does
//! not exist yet matches nothing and passes, so an invariant about the data
//! directories or about the fetch binary is vacuous until they land rather than
//! broken. Where that is the case the test says so.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    // <root>/crates/ausgleich-cli -> <root>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the manifest directory has a repository root two levels up")
        .to_path_buf()
}

fn git(args: &[&str]) -> (i32, String) {
    let out = Command::new("git")
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("git runs");
    let code = out.status.code().expect("git exits with a code");
    assert!(
        code < 2,
        "git exited {code}, which is a tooling failure rather than a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (code, String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Refuse every tracked line inside `scope` that matches `pattern`.
///
/// `git grep` exits 0 when it matched, 1 when it did not, and 2 or more when it
/// could not look. The assertion above turns the third case into a failure, so a
/// broken scanner is never read as a clean tree.
fn refuse(pattern: &str, scope: &[&str], why: &str) {
    let mut args = vec!["grep", "-nE", pattern, "--"];
    args.extend_from_slice(scope);
    let (code, out) = git(&args);
    assert!(
        code == 1,
        "{why}\n\npattern: {pattern}\nscope: {scope:?}\n\nlines refused:\n{out}"
    );
}

#[test]
fn no_panicking_unwrap_or_expect_in_library_code() {
    // A solver that panics on bad input is a solver an operator cannot script.
    // The command line's top level may panic, because there the panic is the
    // error report.
    //
    // What this cannot see: a `#[cfg(test)]` module inside `src/`. The pattern
    // reads paths, not compilation flags, so a unit test written there would be
    // refused along with the library. Unit tests go under `tests/`, and that is
    // a constraint this check imposes rather than one it merely enforces.
    refuse(
        r"\.(unwrap|expect)\(",
        &[
            ":(glob)crates/*/src/**/*.rs",
            ":(exclude)crates/ausgleich-cli/src/main.rs",
        ],
        "A panicking unwrap or expect in library code. Return an error instead; \
         the command line's top level is the one place a panic is the report.",
    );
}

#[test]
fn no_physical_constant_written_as_a_literal_in_source() {
    // A constant in source is a constant nobody can cite. The physics lives in
    // data files with a provenance block, so a reviewer checks it against the
    // publication instead of against a diff.
    //
    // What this catches is the SHAPE of a quoted constant: a decimal carried to
    // five or more places, or anything in scientific notation. It does not know
    // what a number means, so a five-digit array index would be refused and a
    // two-digit constant would not. The bound is deliberate and the direction is
    // the safe one.
    refuse(
        r"[0-9]\.[0-9]{5,}|[0-9](\.[0-9]+)?[eE][-+]?[0-9]+",
        &[":(glob)crates/*/src/**/*.rs"],
        "A number in source with the shape of a physical constant. Constants \
         live in the data directories with the publication they came from.",
    );
}

#[test]
fn no_uncertainty_enlargement_applied_outside_the_loader() {
    // #10 made enforceable. An expansion factor is data, it is applied where the
    // data is read, and the default run has none. Applying one anywhere else
    // means a number in the report was enlarged by code the report does not
    // name.
    //
    // This reads the naming convention rather than the behaviour: the function
    // that applies an enlargement is called `apply_expansion...` or
    // `enlarge_uncertainty...`, and it may only exist in the loader. Naming an
    // expansion factor is not applying one, which is why the report crate is not
    // in the scope of a pattern over those two verbs.
    refuse(
        r"apply_expansion|enlarge_uncertaint",
        &[
            ":(glob)crates/*/src/**/*.rs",
            ":(exclude,glob)crates/ausgleich-data/src/**/*.rs",
        ],
        "An uncertainty enlargement applied outside the loader. Expansion \
         factors are data and are applied where the data is read.",
    );
}

#[test]
fn no_hand_written_partial_derivative_in_the_linearisation_path() {
    // #9 made enforceable. A hand-derived Jacobian is a second implementation of
    // the physics that has to be kept in step with the first, and the failure is
    // silent: the fit still converges, just to a slightly wrong place, with
    // uncertainties that look fine.
    //
    // The pattern reads the names such a function is given. It cannot see a
    // derivative written out inline with no telling name, which is why the
    // finite-difference cross-check in #9 is the other half and not an
    // afterthought.
    refuse(
        r"partial_derivative|\bdfd[a-z0-9_]*|\bd_[a-z0-9]+_d_[a-z0-9]+",
        &[
            ":(glob)crates/ausgleich-solve/src/**/*.rs",
            ":(glob)crates/ausgleich-equations/src/**/*.rs",
        ],
        "A hand-written partial derivative in the linearisation path. The \
         Jacobian comes from dual numbers through the same residual function.",
    );
}

#[test]
fn no_parallel_reduction_or_system_linear_algebra_in_the_solve_path() {
    // #3 made enforceable as far as a pattern reaches. That decision asks for no
    // parallel floating-point reduction anywhere in the solve path, a fixed
    // summation order, and no system BLAS, because floating-point addition is
    // not associative: a sum split across pieces and recombined in whatever
    // order the pieces finished gives a different last digit each run. The
    // failure is the quiet one. Nothing crashes, no test looks at the digits
    // that moved, and the first symptom is two machines quoting different
    // constants.
    //
    // Manifests are in scope alongside source, because a threading runtime or a
    // linear algebra binding arrives as a dependency line before it arrives as
    // a call.
    //
    // What this cannot see, and it is most of the decision. It reads names. A
    // reduction parallelised through a crate named nothing like these, or a
    // dependency that spawns threads two levels down, is invisible to it. So is
    // a summation whose order follows the order values arrived in, which needs
    // no thread at all to differ between two runs. The measurement that would
    // catch those is #3's own Done-when, two runs compared byte for byte, and
    // it needs a program that writes an output file. This is the half that can
    // be had today, and it is the cheaper half.
    refuse(
        r"rayon|par_iter|par_chunks|par_bridge|par_sort|std::thread|thread::spawn|available_parallelism|num_cpus|\b[a-z_]*blas\b|\b[a-z_]*lapack\b|\bmkl\b",
        &[
            ":(glob)crates/ausgleich-solve/src/**/*.rs",
            ":(glob)crates/ausgleich-equations/src/**/*.rs",
            "crates/ausgleich-solve/Cargo.toml",
            "crates/ausgleich-equations/Cargo.toml",
        ],
        "A parallel reduction, a thread count, or a system linear algebra \
         library in the solve path. Floating-point addition is not associative, \
         so a reduction whose order depends on how the work was split makes the \
         digits this program prints a fact about the machine.",
    );
}

#[test]
fn no_normal_equations_and_no_repair_of_a_covariance_that_will_not_factor() {
    // Two absences #2 and #56 state and nothing refused. They are one test
    // rather than two because they are the same claim about the same crate:
    // the solve path takes the problem as it is given and neither squares it
    // nor mends it. Splitting them would be two changes to this file for one
    // property, and one of them would land second and move the other.
    //
    // Forming the normal-equations matrix squares the condition number, and
    // this problem is ill-conditioned by construction, so the digits are gone
    // before the solve begins. Repairing a covariance that will not factor is
    // worse than losing digits: an asserted coefficient that cannot hold
    // together with its neighbours is a finding about a published table, and a
    // diagonal quietly enlarged until the factorisation succeeds turns that
    // finding into a run that looks ordinary.
    //
    // Both are the shape somebody adds in good faith. A ridge term or a nudged
    // diagonal reads as a helpful fix for a crash to a reader who has not read
    // those two issues, which is why the refusal names them.
    //
    // The bound, and it is the same one every pattern here carries. It reads
    // the names such a function is given, so a formation or a repair written
    // inline with no telling name walks through it. What would catch that is a
    // reader, and the second half of #56 is the fixture that proves the refusal
    // fires rather than the pattern that proves nothing was written.
    //
    // The names are written as identifier fragments and not as words, because
    // the file this protects argues in prose that it does none of these things.
    // A pattern over the words would be refused by the paragraph saying the
    // code does not do what the pattern refuses.
    refuse(
        r"normal_equations|gram_matrix|a_transpose_a|nudge_|_nudge|add_to_diagonal|make_positive_definite|nearest_positive|shrink_toward|regularis|regulariz|jitter",
        &[":(glob)crates/ausgleich-solve/src/**/*.rs"],
        "The normal-equations matrix, or a repair of a covariance that will not \
         factor, in the solve path. The problem is whitened and solved as it \
         stands: a covariance that cannot be factored is a finding about the \
         input data and not a matrix to mend.",
    );
}

#[test]
fn no_network_call_outside_the_fetch_binary() {
    // #15 made enforceable. The program reads local files and writes local
    // files, in any build, with no flag that turns a request on. A tool that
    // phones home is a tool an organisation will not run and a person cannot
    // use quietly.
    //
    // Manifests are in scope as well as source, because a networking crate
    // arrives as a dependency line before it arrives as a call.
    //
    // The exclusion names a crate that does not exist yet. The fetch tool is
    // #83; until it lands this pattern has nothing to exempt, which makes the
    // rule stricter today rather than weaker.
    refuse(
        r"std::net|TcpStream|TcpListener|UdpSocket|\breqwest\b|\bureq\b|\bhyper\b|\bcurl\b|\bisahc\b",
        &[
            ":(glob)crates/*/src/**/*.rs",
            ":(glob)crates/*/Cargo.toml",
            ":(exclude,glob)crates/ausgleich-fetch/**",
        ],
        "A network call, or a networking dependency, outside the fetch binary. \
         The program makes no request at runtime, in any build.",
    );
}

#[test]
fn no_dense_correlation_matrix_file_in_the_data_directories() {
    // #5 made enforceable. A dense matrix checked in erases the difference
    // between a coefficient somebody published and a zero nobody ever asserted,
    // and that difference is one of the few things this rebuild can preserve
    // that the printed tables cannot.
    //
    // This reads the file list rather than file contents, so it refuses a file
    // that announces itself and cannot see a dense matrix under an innocent
    // name. What makes that unrepresentable is the record schema in #31, not
    // this pattern.
    //
    // Vacuous until the data directories exist, which is #38 onwards.
    let (_, out) = git(&["ls-files", "--", ":(glob)data/**"]);
    let offenders: Vec<&str> = out
        .lines()
        .filter(|path| {
            let lower = path.to_ascii_lowercase();
            lower.contains("matrix") || lower.ends_with(".mat")
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "A dense correlation matrix file in the data directories: {offenders:?}. \
         A correlation coefficient is an asserted pair with a source; zero is \
         the absence of a record rather than a stored zero."
    );
}

#[test]
fn no_suppression_attribute_without_a_reason_string() {
    // A silenced lint carries its argument. Without one, the next reader cannot
    // tell a considered exception from a lint somebody found annoying.
    //
    // This one is not a single grep. The formatter splits a long attribute
    // across lines, so a line-oriented pattern would pass exactly the
    // suppressions that are long enough to need explaining. The scan below finds
    // each attribute and reads to its closing bracket.
    //
    // The openers are assembled rather than written out, and that is not style.
    // This file is one of the files the scan reads. Spelled literally, the four
    // strings below would be four suppressions with no reason inside the check
    // that refuses them, and the check would refuse itself. Written this way the
    // sequence never appears in the source, so the file stays in scope instead
    // of being excluded from its own rule.
    let hash = '#';
    let openers = [
        format!("{hash}[allow("),
        format!("{hash}![allow("),
        format!("{hash}[expect("),
        format!("{hash}![expect("),
    ];
    let (_, listing) = git(&["ls-files", "--", ":(glob)crates/**/*.rs"]);
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    for path in listing.lines() {
        let text = std::fs::read_to_string(root.join(path)).expect("a tracked source file reads");
        for opener in &openers {
            let mut from = 0usize;
            while let Some(found) = text[from..].find(opener) {
                let start = from + found;
                let body_start = start + opener.len();
                let mut depth = 1usize;
                let mut end = body_start;
                for (offset, ch) in text[body_start..].char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = body_start + offset;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let body = &text[body_start..end];
                if !body.contains("reason") {
                    let line = text[..start].lines().count() + 1;
                    offenders.push(format!("{path}:{line}: {opener}{body})"));
                }
                from = body_start;
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "A suppression attribute with no reason string: {offenders:#?}. Write \
         `reason = \"...\"` inside it, so the next reader can tell a considered \
         exception from a lint somebody found annoying."
    );
}
