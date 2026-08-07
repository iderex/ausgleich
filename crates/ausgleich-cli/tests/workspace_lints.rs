//! Every member of this workspace inherits the workspace lint table.
//!
//! Warnings are denied in `[workspace.lints]` at the root, and that table
//! reaches a member only where the member's own manifest says `[lints]
//! workspace = true`. The two lines are easy to leave out of a crate somebody
//! adds later, and leaving them out fails in the worst direction: the new crate
//! compiles with warnings allowed while every check stays green, so the tree
//! learns that a warning is survivable in exactly the place nobody is looking.
//!
//! This reads the manifests as text, which the neighbouring
//! `dependency_edges.rs` deliberately does not do for edges. The reason is that
//! there is nothing else to read. `cargo metadata` does not report the lints
//! table in any format version this workspace can ask for, so the resolver has
//! no view of it and the file on disk is the only statement of the fact.
//!
//! What it does not check: whether the workspace table says anything useful.
//! It checks that a member is attached to it. The levels themselves are in the
//! root manifest with their reasons, and a change there is a change to one
//! visible line.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // <root>/crates/ausgleich-cli -> <root>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the manifest directory has a workspace root two levels up")
        .to_path_buf()
}

/// Every directory under `crates/` that holds a `Cargo.toml`.
///
/// Derived from the filesystem rather than from a list in this file, so a crate
/// added to the workspace is covered the moment it exists. A list here would be
/// a second place to forget the new member, which is the failure this file is
/// about.
fn member_manifests() -> Vec<PathBuf> {
    let crates = workspace_root().join("crates");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("the crates directory is readable")
        .map(|entry| entry.expect("a directory entry is readable").path())
        .map(|dir| dir.join("Cargo.toml"))
        .filter(|manifest| manifest.is_file())
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "no member manifests found under {}, so this file would pass by \
         examining nothing",
        crates.display()
    );
    found
}

/// The `[lints]` table of a manifest, as the lines between that header and the
/// next one.
///
/// Written as a scan over section headers rather than a substring search,
/// because `workspace = true` appears under `[dependencies]` entries too and a
/// substring search would read one of those as this table.
fn lints_section(manifest: &str) -> Option<Vec<&str>> {
    let mut lines = manifest.lines().skip_while(|line| line.trim() != "[lints]");
    lines.next()?;
    Some(
        lines
            .take_while(|line| !line.trim_start().starts_with('['))
            .collect(),
    )
}

#[test]
fn every_member_inherits_the_workspace_lint_table() {
    let mut detached: Vec<String> = Vec::new();
    for manifest_path in member_manifests() {
        let text = std::fs::read_to_string(&manifest_path).expect("a member manifest is readable");
        let inherits = lints_section(&text)
            .is_some_and(|body| body.iter().any(|line| line.trim() == "workspace = true"));
        if !inherits {
            detached.push(manifest_path.display().to_string());
        }
    }
    assert!(
        detached.is_empty(),
        "these members do not inherit [workspace.lints]: {detached:?}. Add\n\n\
         [lints]\nworkspace = true\n\n\
         to each. Without it the crate compiles with warnings allowed and every \
         check stays green, which is the one way this rule fails without saying \
         so."
    );
}
