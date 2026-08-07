//! The split between the default suite and the suites that are not run by
//! default, and the rules that keep the default one runnable everywhere.
//!
//! #14 states the requirement and #20 makes it a gate. The default suite opens
//! no window, needs no privileged account, binds no socket off loopback and
//! writes nothing outside a temporary directory. Anything that needs more than
//! that is a separate invocation, named for what it needs rather than called
//! integration, and is not run on a pull request.
//!
//! Everything about the split lives in this one file on purpose. The roster
//! below is read three ways: to print what a run did not cover, to check the
//! manifest still separates those targets, and to decide which test sources the
//! rules on the default suite apply to. Split across files, the third reader
//! would carry a second copy of the suite list, and the copy that drifts is the
//! one that decides which files are exempt from a guard.
//!
//! What this file does not do is run the separated suites or measure them. It
//! says they exist, that the default run did not cover them, and what asking
//! for them costs.
//!
//! # The promise no machine here refuses
//!
//! Of the four promises named above, three are refused by a check in this file.
//! **A test that opens a window is not one of them, and no check here is
//! coming.** #96 is where that was argued and this is the sentence it asks for.
//!
//! Nothing in this tree can tell that a test drew something. A window is opened
//! by a call into the host's graphical system, usually several layers inside a
//! dependency, and there is no artefact here for a check to read that would
//! distinguish a test that rendered to a file from one that put a window on a
//! screen. The nearest available thing is a pattern over the names of graphical
//! crates, which would be a list that goes stale the day a new one is
//! published, and a list like that earns a green tick it has not paid for.
//!
//! So this one stays prose, and it stays prose deliberately rather than while
//! somebody gets round to it. What holds it is the rule in #14 that anything
//! which draws renders to a file, and the fact that nothing in this workspace
//! draws at all yet.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A suite the default run does not cover.
struct Separated {
    /// The name a person uses for it.
    label: &'static str,
    /// The cargo feature that turns it on. Not in `default`, and
    /// `no_separated_suite_is_on_by_default` is what keeps it out.
    feature: &'static str,
    /// The test target it runs, which is `tests/<target>.rs`.
    target: &'static str,
    /// What it needs that the default suite may not have.
    needs: &'static str,
    /// What asking for it costs, in the words a person weighing it would use.
    cost: &'static str,
}

/// Every suite that is not the default one.
///
/// Two, and the reason there are exactly two is in #14: one for anything that
/// fetches a source document over the network, one for anything that samples or
/// sweeps and therefore takes minutes. There is no hardware in this project's
/// path and no hardware-bound suite is invented to look thorough.
const SEPARATED: &[Separated] = &[
    Separated {
        label: "network-bound",
        feature: "suite-network",
        target: "suite_network",
        needs: "egress to a server this project does not control",
        cost: "requests to a third party, and a verdict that moves when their \
               server does",
    },
    Separated {
        label: "long-running",
        feature: "suite-long",
        target: "suite_long",
        needs: "time, because it samples or sweeps rather than asserting once",
        cost: "minutes rather than seconds, on every run that asks for it",
    },
];

/// What the default suite promises, printed beside it so the announcement says
/// what was covered as well as what was not.
const DEFAULT_SUITE_PROMISE: &str = "no display, no elevation, no socket off loopback, \
                                     no write outside a temporary directory";

fn repo_root() -> PathBuf {
    // <root>/crates/ausgleich-cli -> <root>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the manifest directory has a repository root two levels up")
        .to_path_buf()
}

fn cli_manifest() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).expect("this crate's own manifest is readable")
}

/// The command that invokes one separated suite.
fn invocation(suite: &Separated) -> String {
    format!(
        "cargo test --package ausgleich-cli --features {} --test {}",
        suite.feature, suite.target
    )
}

#[test]
fn the_default_run_says_which_suites_it_did_not_run() {
    // Written to the process's own stderr handle rather than through
    // `eprintln!`. The test harness captures the macros, so a message written
    // with one is shown only when a run already failed or already passed
    // `--nocapture`, and the whole point of this announcement is to be there on
    // an ordinary green run. The handle itself is outside what the harness
    // redirects.
    //
    // It is a test rather than a step in the workflow so that it reaches a
    // contributor's own `cargo test` and not only the runner. A gate that
    // discloses what a local run does not is a gate teaching people that the
    // disclosure is the gate's business.
    let mut out = std::io::stderr();
    let mut say = |line: &str| {
        writeln!(out, "{line}").expect("the process stderr handle accepts a line");
    };

    say("");
    say("suites in this workspace");
    say(&format!(
        "  default        ran      {DEFAULT_SUITE_PROMISE}"
    ));
    for suite in SEPARATED {
        say(&format!(
            "  {:<14} NOT RUN  needs {}",
            suite.label, suite.needs
        ));
        say(&format!("  {:<14}          costs {}", "", suite.cost));
        say(&format!("  {:<14}          {}", "", invocation(suite)));
    }
    say("");
    say("A green default run is not a statement about the suites marked NOT RUN.");
    say("");

    assert!(
        !SEPARATED.is_empty(),
        "the roster is empty, so this announcement said nothing and passed. A \
         run that covered less than everything has to say so, and an empty list \
         cannot."
    );
}

#[test]
fn every_separated_suite_is_a_target_the_default_run_cannot_build() {
    // `required-features` is what does the separating, and it does it earlier
    // than a runtime skip: the target is not compiled at all unless the feature
    // is asked for. A `#[ignore]` would leave the code in the default build,
    // where a network dependency would still be linked and a long sweep would
    // still be compiled into the binary the coverage run measures.
    let manifest = cli_manifest();
    let mut wrong: Vec<String> = Vec::new();
    for suite in SEPARATED {
        let declared = test_targets(&manifest)
            .into_iter()
            .find(|target| target.name == suite.target);
        match declared {
            None => wrong.push(format!(
                "{}: no [[test]] section names the target {:?}",
                suite.label, suite.target
            )),
            Some(target) if !target.required_features.iter().any(|f| f == suite.feature) => wrong
                .push(format!(
                    "{}: [[test]] {:?} does not require {:?}, so the default run builds it",
                    suite.label, suite.target, suite.feature
                )),
            Some(_) => {
                let source = repo_root()
                    .join("crates/ausgleich-cli/tests")
                    .join(format!("{}.rs", suite.target));
                if !source.is_file() {
                    wrong.push(format!(
                        "{}: {} does not exist, so the suite cannot run green when \
                         it is invoked",
                        suite.label,
                        source.display()
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "separated suites that are not separated by the manifest: {wrong:#?}"
    );
}

#[test]
fn no_test_target_is_separated_without_being_on_the_roster() {
    // The other direction, and the one that matters more. A target given
    // `required-features` and left off the roster is a suite the default run
    // does not cover and does not mention, which is the exact reading this
    // whole split exists to prevent.
    let manifest = cli_manifest();
    let undeclared: Vec<String> = test_targets(&manifest)
        .into_iter()
        .filter(|target| !target.required_features.is_empty())
        .filter(|target| !SEPARATED.iter().any(|suite| suite.target == target.name))
        .map(|target| target.name)
        .collect();
    assert!(
        undeclared.is_empty(),
        "test targets held back by required-features with no entry in SEPARATED: \
         {undeclared:?}. Add the suite to the roster with what it needs and what \
         it costs, or take the required-features line out. A suite the default \
         run neither runs nor names is one a reader counts as covered."
    );
}

#[test]
fn no_separated_suite_is_on_by_default() {
    // A feature listed under `default` is a feature that is always on, and the
    // separation would be a line in a manifest that changes nothing.
    let manifest = cli_manifest();
    let defaults = default_features(&manifest);
    let leaked: Vec<&str> = SEPARATED
        .iter()
        .map(|suite| suite.feature)
        .filter(|feature| defaults.iter().any(|d| d == feature))
        .collect();
    assert!(
        leaked.is_empty(),
        "these features are in the default set, so their suites run on every \
         ordinary invocation: {leaked:?}"
    );
}

#[test]
fn no_default_suite_test_binds_off_loopback() {
    // #14 made enforceable. A socket bound off loopback is the failure this
    // project can least afford in a test: on Windows it raises a firewall
    // dialog only an administrator can answer, which turns one test into an
    // interactive prompt nobody sees in a pipeline, and a suite that stops
    // running is a gate that means nothing.
    //
    // What this can see and what it cannot. It reads the argument each `::bind`
    // call is given and accepts only a loopback address written there in the
    // source. An address arriving in a variable is refused, and that is
    // deliberate rather than a limitation: a computed address is exactly the
    // case a reader cannot check. What escapes it is a bind performed by a
    // dependency, or by a call spelled some other way, and neither is something
    // a pattern over this tree can reach.
    //
    // The separated suites are exempt, taken from the roster above rather than
    // from a second list, because the network-bound suite is where a socket is
    // allowed to be the point.
    //
    // Library and binary sources are not in scope. `no_network_call_outside_the
    // _fetch_binary` in greppable_invariants.rs already refuses the types
    // outright there, so a second pattern over the same paths would be a
    // weaker copy of a stronger rule.
    //
    // The needle is assembled rather than written out, for the reason the
    // suppression scan in greppable_invariants.rs assembles its openers: this
    // file is one of the files the scan reads, and spelling the literal here
    // would make the check refuse itself.
    let needle = format!("::bind{}", '(');
    let mut offenders: Vec<String> = Vec::new();
    for (path, text) in default_suite_sources() {
        let mut from = 0usize;
        while let Some(found) = text[from..].find(&needle) {
            let start = from + found;
            let body_start = start + needle.len();
            let argument = balanced_argument(&text[body_start..]);
            if !is_loopback(argument) {
                let line = text[..start].lines().count();
                offenders.push(format!("{path}:{line}: bind({argument})"));
            }
            from = body_start;
        }
    }
    assert!(
        offenders.is_empty(),
        "default-suite tests binding a socket off loopback: {offenders:#?}. Bind \
         127.0.0.1 or ::1 with the address written in the source, or move the \
         test into the network-bound suite. Off loopback, Windows raises a \
         firewall dialog only an administrator can answer, and a suite that \
         needs a person is a suite that stops running."
    );
}

#[test]
fn no_default_suite_test_writes_outside_a_temporary_directory() {
    // #14 made enforceable, and #96 is where the gap this closes was written
    // down. A test that writes into the checkout leaves the tree dirty for the
    // next run, and a test that writes into a home directory or a system path
    // leaves something behind on a machine somebody else is using. Both make a
    // suite that passes once and fails afterwards, which is the slowest kind of
    // failure to diagnose.
    //
    // Reads are not in scope, and that is the whole design. Several tests here
    // read the tree on purpose, this one included. What is refused is a call
    // that creates, writes, moves or deletes.
    //
    // What it can see and what it cannot. It reads the argument each writing
    // call is given and accepts only a path that names a temporary-directory
    // source in the source text. A path arriving in a variable is refused, for
    // the reason the loopback rule refuses a computed address: it is exactly
    // the case a reader cannot check. A write performed by a dependency, or
    // through a handle obtained somewhere else, is outside what a pattern over
    // this tree can reach.
    //
    // The needles are assembled for the reason the loopback one is: this file
    // is one of the files the scan reads.
    let open = '(';
    let needles = [
        format!("fs::write{open}"),
        format!("File::create{open}"),
        format!("fs::create_dir{open}"),
        format!("fs::create_dir_all{open}"),
        format!("fs::remove_file{open}"),
        format!("fs::remove_dir_all{open}"),
        format!("fs::rename{open}"),
        format!("fs::copy{open}"),
    ];
    let mut offenders: Vec<String> = Vec::new();
    for (path, text) in default_suite_sources() {
        for needle in &needles {
            let mut from = 0usize;
            while let Some(found) = text[from..].find(needle.as_str()) {
                let start = from + found;
                let body_start = start + needle.len();
                let argument = balanced_argument(&text[body_start..]);
                if !is_temporary(argument) {
                    let line = text[..start].lines().count();
                    offenders.push(format!("{path}:{line}: {needle}{argument})"));
                }
                from = body_start;
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "default-suite tests writing outside a temporary directory: \
         {offenders:#?}. Build the path from std::env::temp_dir and say so in \
         the source, or move the test into a separated suite. A test that \
         writes into the checkout leaves the tree dirty for the run after it."
    );
}

/// One tool that cannot do its job without a privileged account, and what it
/// does.
///
/// The names are split in two and joined below, for the reason the other
/// needles in this file are assembled: this file is one of the files the scan
/// reads, and a name spelled out here would make the check refuse itself.
const ELEVATION_TOOLS: &[(&str, &str, &str)] = &[
    (
        "net",
        "sh",
        "reconfigures the firewall or the network stack",
    ),
    ("sc", ".exe", "creates or controls a service"),
    ("sch", "tasks", "registers a scheduled task"),
    ("msi", "exec", "runs an installer"),
    ("run", "as", "starts a process as another account"),
    ("su", "do", "starts a process as another account"),
    ("dev-", "certs", "writes to a certificate store"),
];

#[test]
fn no_default_suite_test_shells_out_to_something_that_needs_elevation() {
    // #14 made enforceable, and the other half of what #96 holds open. A test
    // that needs a privileged account is a test that stops running, and on a
    // machine with a person at it the prompt it raises steals their attention
    // for a run they did not start.
    //
    // This refuses a shape and not a meaning, and the difference matters more
    // here than for the other two rules in this file. It catches the tools
    // somebody actually reaches for, named in the table above with what each
    // one does. A program that asks for elevation under a name nobody listed
    // walks straight through, and the table is a floor rather than a
    // guarantee: it holds what has been thought of.
    //
    // The comparison is over lowercased text with a boundary on each side, so
    // a tool name appearing inside a longer word is not a match.
    let mut offenders: Vec<String> = Vec::new();
    for (path, text) in default_suite_sources() {
        let haystack = text.to_ascii_lowercase();
        for (head, tail, what_it_does) in ELEVATION_TOOLS {
            let name = format!("{head}{tail}");
            let mut from = 0usize;
            while let Some(found) = haystack[from..].find(&name) {
                let start = from + found;
                let end = start + name.len();
                if is_standalone(&haystack, start, end) {
                    let line = text[..start].lines().count() + 1;
                    offenders.push(format!("{path}:{line}: {name}, which {what_it_does}"));
                }
                from = end;
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "default-suite tests naming a tool that needs a privileged account: \
         {offenders:#?}. A test that needs elevation is skipped and the skip is \
         disclosed, never worked around, because the prompt it raises can only \
         be answered by a person sitting at the machine."
    );
}

/// Every tracked default-suite test source, with its text.
///
/// The separated suites are excluded, taken from the roster above rather than
/// from a second list. It refuses to return nothing, so a scope that stops
/// matching cannot be read by its callers as a tree with nothing to find.
fn default_suite_sources() -> Vec<(String, String)> {
    let mut scope = vec![":(glob)crates/*/tests/**/*.rs".to_string()];
    for suite in SEPARATED {
        scope.push(format!(":(exclude,glob)crates/*/tests/{}.rs", suite.target));
    }
    let scope_args: Vec<&str> = scope.iter().map(String::as_str).collect();
    let mut args = vec!["ls-files", "--"];
    args.extend_from_slice(&scope_args);
    let listing = git(&args);
    assert!(
        !listing.trim().is_empty(),
        "no default-suite test sources were found, so every check over them \
         examined nothing and passed. The scope is {scope_args:?}."
    );
    let root = repo_root();
    listing
        .lines()
        .map(|path| {
            let text =
                std::fs::read_to_string(root.join(path)).expect("a tracked test source reads");
            (path.to_string(), text)
        })
        .collect()
}

/// Whether a match sits on its own rather than inside a longer word.
fn is_standalone(haystack: &str, start: usize, end: usize) -> bool {
    let before = haystack[..start].chars().next_back();
    let after = haystack[end..].chars().next();
    let is_word = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    !is_word(before) && !is_word(after)
}

/// Whether a path argument names a temporary directory in the source rather
/// than somewhere else or somewhere computed.
fn is_temporary(argument: &str) -> bool {
    let text = argument.to_ascii_lowercase();
    text.contains("temp_dir") || text.contains("tempdir")
}

/// The text between an opening parenthesis and the one that closes it.
fn balanced_argument(after_open: &str) -> &str {
    let mut depth = 1usize;
    for (offset, ch) in after_open.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &after_open[..offset];
                }
            }
            _ => {}
        }
    }
    after_open
}

/// Whether a bind argument names loopback in the source rather than somewhere
/// else or somewhere computed.
fn is_loopback(argument: &str) -> bool {
    let text = argument.to_ascii_lowercase();
    text.contains("127.0.0.1") || text.contains("::1") || text.contains("localhost")
}

fn git(args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed, which is a tooling failure rather than a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// One `[[test]]` section of a manifest.
struct TestTarget {
    name: String,
    required_features: Vec<String>,
}

/// The `[[test]]` sections of a manifest, read as text.
///
/// `cargo metadata` reports test targets but not their `required-features` in
/// any format version this workspace can ask for, so the file on disk is the
/// only statement of the thing being asserted. The manifest is a dozen lines
/// long and every value in it is a plain string, which is the case where
/// reading text is honest rather than lazy.
fn test_targets(manifest: &str) -> Vec<TestTarget> {
    let mut targets: Vec<TestTarget> = Vec::new();
    let mut current: Option<TestTarget> = None;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(target) = current.take() {
                targets.push(target);
            }
            if trimmed == "[[test]]" {
                current = Some(TestTarget {
                    name: String::new(),
                    required_features: Vec::new(),
                });
            }
            continue;
        }
        let Some(target) = current.as_mut() else {
            continue;
        };
        if let Some(value) = trimmed.strip_prefix("name") {
            target.name = unquote_scalar(value);
        } else if let Some(value) = trimmed.strip_prefix("required-features") {
            target.required_features = unquote_array(value);
        }
    }
    if let Some(target) = current.take() {
        targets.push(target);
    }
    targets
}

/// The features listed under `default` in a manifest's `[features]` table, or
/// none where there is no such entry.
fn default_features(manifest: &str) -> Vec<String> {
    let mut in_features = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_features = trimmed == "[features]";
            continue;
        }
        if in_features {
            if let Some(value) = trimmed.strip_prefix("default") {
                return unquote_array(value);
            }
        }
    }
    Vec::new()
}

/// `= "thing"` becomes `thing`.
fn unquote_scalar(after_key: &str) -> String {
    after_key
        .trim_start()
        .trim_start_matches('=')
        .trim()
        .trim_matches('"')
        .to_string()
}

/// `= ["a", "b"]` becomes `a`, `b`.
fn unquote_array(after_key: &str) -> Vec<String> {
    after_key
        .trim_start()
        .trim_start_matches('=')
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}
