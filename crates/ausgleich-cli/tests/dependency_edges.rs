//! The dependency arrows between the members of this workspace, asserted here
//! so that an added edge reds the suite instead of being noticed in review.
//!
//! The arrows point one way and the shape is argued in #17. What this file adds
//! is that the shape is a fact somebody can check rather than a paragraph.
//!
//! It fails closed in both directions. An edge in a manifest with no line in
//! `ALLOWED` fails, which is the case the issue is about. An entry in `ALLOWED`
//! that no manifest carries fails too, so a boundary cannot be quietly relaxed
//! by deleting the dependency and leaving the sentence that justified it. The
//! member list is checked the same way, so adding a crate to the workspace
//! reddens this file until its arrows are written down.
//!
//! The edges come from `cargo tree`, not from reading the manifests as text. A
//! dependency can be written several ways and can arrive through a feature or a
//! platform-specific table; what the resolver ended up with is the thing worth
//! asserting.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One allowed arrow, with the reason it is allowed.
struct Edge {
    from: &'static str,
    to: &'static str,
    why: &'static str,
}

/// Every dependency between two members of this workspace that is allowed.
/// Nothing else is.
const ALLOWED: &[Edge] = &[
    Edge {
        from: "ausgleich-equations",
        to: "ausgleich-data",
        why: "an equation is written against the record types it reads",
    },
    Edge {
        from: "ausgleich-report",
        to: "ausgleich-data",
        why: "the report names the input set the result came from",
    },
    Edge {
        from: "ausgleich-report",
        to: "ausgleich-solve",
        why: "the report renders the types the solve returns",
    },
    Edge {
        from: "ausgleich-cli",
        to: "ausgleich-data",
        why: "the command line is where a run is assembled, so it sees all four",
    },
    Edge {
        from: "ausgleich-cli",
        to: "ausgleich-equations",
        why: "the command line is where a run is assembled, so it sees all four",
    },
    Edge {
        from: "ausgleich-cli",
        to: "ausgleich-solve",
        why: "the command line is where a run is assembled, so it sees all four",
    },
    Edge {
        from: "ausgleich-cli",
        to: "ausgleich-report",
        why: "the command line is where a run is assembled, so it sees all four",
    },
];

/// The members this file's arrows were written against. A member added to the
/// workspace and not added here reddens `the_member_list_is_the_one_the_arrows_were_written_against`.
const MEMBERS: &[&str] = &[
    "ausgleich-cli",
    "ausgleich-data",
    "ausgleich-equations",
    "ausgleich-report",
    "ausgleich-solve",
];

fn workspace_root() -> PathBuf {
    // <root>/crates/ausgleich-cli -> <root>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the manifest directory has a workspace root two levels up")
        .to_path_buf()
}

/// `cargo tree` with the given extra arguments, run against this workspace.
///
/// Every line it prints is `<name> v<version> (<path>)`, so the first
/// whitespace-separated token is the package name. Blank lines separate roots
/// when several are printed and are dropped.
fn tree(extra: &[&str]) -> Vec<String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let root = workspace_root();
    let mut cmd = Command::new(cargo);
    cmd.current_dir(&root)
        .arg("tree")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--prefix")
        .arg("none")
        .arg("--format")
        .arg("{p}")
        // Every kind of edge counts. A boundary that a dev-dependency or a
        // build-dependency may cross is not a boundary.
        .arg("--edges")
        .arg("normal,build,dev")
        .args(extra);
    let out = cmd.output().expect("cargo tree runs");
    assert!(
        out.status.success(),
        "cargo tree failed: {}\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("cargo tree prints utf-8")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_whitespace()
                .next()
                .expect("a non-empty line has a first token")
                .to_string()
        })
        .collect()
}

/// The workspace members, as the resolver sees them.
fn members() -> BTreeSet<String> {
    tree(&["--workspace", "--depth", "0"]).into_iter().collect()
}

/// The direct dependencies of one member that are themselves members.
fn edges_from(member: &str, all_members: &BTreeSet<String>) -> BTreeSet<String> {
    let lines = tree(&["--package", member, "--depth", "1"]);
    // The first line is the package itself; everything after it is a direct
    // dependency.
    lines
        .into_iter()
        .skip(1)
        .filter(|name| all_members.contains(name) && name != member)
        .collect()
}

fn declared_from(member: &str) -> BTreeSet<String> {
    ALLOWED
        .iter()
        .filter(|edge| edge.from == member)
        .map(|edge| edge.to.to_string())
        .collect()
}

#[test]
fn the_member_list_is_the_one_the_arrows_were_written_against() {
    let observed = members();
    let expected: BTreeSet<String> = MEMBERS.iter().map(|m| m.to_string()).collect();
    assert_eq!(
        observed, expected,
        "the workspace members changed. Add or remove the crate in MEMBERS and \
         write its arrows into ALLOWED, or this file is asserting a shape the \
         workspace no longer has."
    );
}

#[test]
fn no_member_depends_on_a_member_that_is_not_declared() {
    let all = members();
    let mut undeclared: Vec<String> = Vec::new();
    for member in &all {
        for dep in edges_from(member, &all) {
            if !declared_from(member).contains(&dep) {
                undeclared.push(format!("{member} -> {dep}"));
            }
        }
    }
    assert!(
        undeclared.is_empty(),
        "dependency edges with no line in ALLOWED: {undeclared:?}. The arrows in \
         this workspace point one way and each one has a reason written next to \
         it. Add the edge to ALLOWED with its reason, or take the dependency out."
    );
}

#[test]
fn every_declared_edge_is_a_dependency_that_actually_exists() {
    let all = members();
    let mut missing: Vec<String> = Vec::new();
    for edge in ALLOWED {
        let observed = edges_from(edge.from, &all);
        if !observed.contains(edge.to) {
            missing.push(format!("{} -> {} ({})", edge.from, edge.to, edge.why));
        }
    }
    assert!(
        missing.is_empty(),
        "declared edges that no manifest carries: {missing:?}. An entry here that \
         nothing uses is a permission nobody asked for; delete the line rather \
         than leaving it as cover for a future edge."
    );
}

#[test]
fn the_solver_depends_on_no_member_of_this_workspace() {
    // Stated on its own because it is the arrow the layout exists for. The
    // solver carries fixture tests and a mutation run, so it may not need a
    // directory to exist before it can be called, and the file seam is only
    // honest while the solver cannot see the loader.
    let all = members();
    let observed = edges_from("ausgleich-solve", &all);
    assert!(
        observed.is_empty(),
        "ausgleich-solve now depends on {observed:?}. Structs go in and structs \
         come out; anything it needs to know arrives as a value."
    );
}
