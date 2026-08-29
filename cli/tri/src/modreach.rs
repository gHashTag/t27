//! `tri mods orphan` -- `.rs` files no crate root reaches.
//!
//! Rust compiles a file only if a `mod` declaration leads to it from the crate root.
//! A file nothing declares is not an unused file: it is a file the compiler never
//! opens. It cannot warn, cannot fail to build, and its `#[test]` functions do not
//! exist as far as `cargo test` is concerned.
//!
//! That is how `tri elab` disappeared. #2427, a change to the Zig lexer, removed
//! `mod elab;` and two more lines from `main.rs` in one hunk and left the 319-line
//! file. The suite went 358 -> 354 and nothing printed a word: `cargo build` cannot
//! error on a file it does not compile, and no gate reads the test count.
//!
//! The same shape as #2895 in another language. `lake build` never prints what it
//! skipped; `cargo build` never prints what it did not compile. Every build system
//! reports the work it did, and a coverage claim is about the work it did not.
//!
//! Resolution follows the language, not a heuristic: `mod c;` inside `a/b.rs` means
//! `a/b/c.rs` or `a/b/c/mod.rs`, and `#[path]` and `include!` are edges too. A looser
//! rule -- "is this stem named by any `mod` anywhere" -- hides real orphans behind a
//! same-named module in an unrelated directory.

use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One edge out of a source file: the paths a declaration can resolve to.
#[derive(Debug, PartialEq)]
pub struct Edge {
    pub candidates: Vec<PathBuf>,
}

/// Every file `src` reaches, given that `src` itself lives at `here` and its child
/// modules live under `dir`.
///
/// `mod c;` in `a/b.rs` looks in `a/b/`; the same line in `a/mod.rs` or `a/lib.rs`
/// looks in `a/`. `#[path = "p"]` and `include!("p")` are resolved against `here`'s
/// own directory in both cases.
pub fn edges_from(src: &str, here: &Path, dir: &Path) -> Vec<Edge> {
    let own = here.parent().unwrap_or(Path::new(""));
    let mut out = Vec::new();
    let mut pending_path: Option<String> = None;
    for line in src.lines() {
        let t = line.trim();
        if let Some(p) = attr_path(t) {
            pending_path = Some(p);
            continue;
        }
        if let Some(p) = include_path(t) {
            out.push(Edge { candidates: vec![own.join(p)] });
            continue;
        }
        let Some(name) = mod_decl(t) else {
            if !t.is_empty() && !t.starts_with("//") && !t.starts_with("#[") {
                pending_path = None;
            }
            continue;
        };
        if let Some(p) = pending_path.take() {
            out.push(Edge { candidates: vec![own.join(p)] });
        } else {
            out.push(Edge {
                candidates: vec![
                    dir.join(format!("{name}.rs")),
                    dir.join(&name).join("mod.rs"),
                ],
            });
        }
    }
    out
}

/// `mod name;` -- the file-backed form only. `mod name {` is inline and needs no file.
pub fn mod_decl(t: &str) -> Option<String> {
    let r = t
        .strip_prefix("pub ")
        .map(|r| r.trim_start())
        .unwrap_or(t);
    let r = if r.starts_with("pub(") {
        r.split_once(')')?.1.trim_start()
    } else {
        r
    };
    let r = r.strip_prefix("mod ")?;
    let name = r.strip_suffix(';')?.trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some(name.to_string())
}

/// `#[path = "x.rs"]`
pub fn attr_path(t: &str) -> Option<String> {
    let r = t.strip_prefix("#[path")?.trim_start().strip_prefix('=')?;
    let r = r.trim_start().strip_prefix('"')?;
    Some(r.split('"').next()?.to_string())
}

/// `include!("x.rs")`
pub fn include_path(t: &str) -> Option<String> {
    let r = t.find("include!(\"").map(|i| &t[i + 10..])?;
    Some(r.split('"').next()?.to_string())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
    Ok(())
}

/// Files reachable from `root`, following `mod`, `#[path]` and `include!`.
pub fn reach(root: &Path, src_dir: &Path) -> Result<BTreeSet<PathBuf>> {
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        let Ok(c) = std::fs::canonicalize(&p) else { continue };
        if !seen.insert(c.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&c) else { continue };
        // Children of `x/mod.rs`, `src/main.rs` and `src/lib.rs` live in that file's
        // own directory; children of `x/y.rs` live in `x/y/`.
        let stem = c.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let own = c.parent().unwrap_or(src_dir).to_path_buf();
        let dir = if matches!(stem, "mod" | "main" | "lib") {
            own
        } else {
            c.with_extension("")
        };
        for e in edges_from(&text, &c, &dir) {
            for cand in e.candidates {
                if cand.is_file() {
                    stack.push(cand);
                    break;
                }
            }
        }
    }
    Ok(seen)
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

pub fn run() -> Result<()> {
    let repo = repo_root()?;
    let crates = ["bootstrap", "cli/tri"];
    println!("ORPHANED SOURCE FILES  (nothing declares them; nothing compiles them)");
    println!();
    let mut total_files = 0usize;
    let mut total_orphans = 0usize;
    let mut total_tests = 0usize;
    for c in crates {
        let src = repo.join(c).join("src");
        if !src.is_dir() {
            anyhow::bail!("{} does not exist -- the crate list in this command is stale, \
                           and a report of zero orphans would be that staleness", src.display());
        }
        let root = ["main.rs", "lib.rs"]
            .iter()
            .map(|f| src.join(f))
            .find(|p| p.is_file())
            .ok_or_else(|| anyhow::anyhow!("{}: no main.rs or lib.rs", src.display()))?;
        let reached = reach(&root, &src)?;
        if reached.len() < 2 {
            anyhow::bail!(
                "{} reaches {} file(s) from {} -- `mod` lines are not being read, and \
                 every other file would be a false orphan",
                c,
                reached.len(),
                root.display()
            );
        }
        let mut all = Vec::new();
        walk(&src, &mut all)?;
        let mut orphans: Vec<(PathBuf, usize, usize)> = Vec::new();
        for p in &all {
            let Ok(cp) = std::fs::canonicalize(p) else { continue };
            if reached.contains(&cp) {
                continue;
            }
            let text = std::fs::read_to_string(p).unwrap_or_default();
            orphans.push((
                p.strip_prefix(&repo).unwrap_or(p).to_path_buf(),
                text.lines().count(),
                text.matches("#[test]").count(),
            ));
        }
        orphans.sort();
        total_files += all.len();
        total_orphans += orphans.len();
        total_tests += orphans.iter().map(|o| o.2).sum::<usize>();
        // `reached` can include files OUTSIDE src/ -- `#[path = "../../gen/..."]` is a
        // real edge -- so it is not comparable to `all.len()`. Report the difference of
        // the two sets that ARE comparable, and say where the rest went.
        let outside = reached.len() - (all.len() - orphans.len());
        println!(
            "  {c:<12} {:>3} files under src/, {:>3} compiled, {:>3} orphaned{}",
            all.len(),
            all.len() - orphans.len(),
            orphans.len(),
            if outside > 0 { format!("   (+{outside} reached outside src/ via #[path])") } else { String::new() }
        );
        for (p, lines, tests) in &orphans {
            let t = if *tests > 0 {
                format!("   {tests} #[test] that cargo cannot see")
            } else {
                String::new()
            };
            println!("      {:<52} {:>5} lines{}", p.display(), lines, t);
        }
    }
    println!();
    println!(
        "  {total_orphans} of {total_files} files, carrying {total_tests} test(s) that do not exist \
         for cargo."
    );
    println!();
    println!(
        "An undeclared file is not an unused file -- it is a file the compiler never\n\
         opens. `cargo build` cannot error on it and `cargo test` counts none of its\n\
         tests, so removing the `mod` line lowers the suite in silence (#2900)."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mod_in_a_leaf_file_looks_in_that_files_own_subdirectory() {
        let e = edges_from("mod c;\n", Path::new("/s/a/b.rs"), Path::new("/s/a/b"));
        assert_eq!(
            e[0].candidates,
            vec![PathBuf::from("/s/a/b/c.rs"), PathBuf::from("/s/a/b/c/mod.rs")],
            "not /s/a/c.rs -- a stem-name match anywhere in the tree would hide orphans"
        );
    }

    #[test]
    fn inline_modules_need_no_file_and_are_not_edges() {
        assert_eq!(mod_decl("mod c {"), None, "inline");
        assert_eq!(mod_decl("mod c;").as_deref(), Some("c"));
        assert_eq!(mod_decl("pub mod c;").as_deref(), Some("c"));
        assert_eq!(mod_decl("pub(crate) mod c;").as_deref(), Some("c"));
    }

    #[test]
    fn path_and_include_are_edges_too() {
        let e = edges_from(
            "#[path = \"../gen/x.rs\"]\nmod c;\ninclude!(\"g/y.rs\");\n",
            Path::new("/s/a/b.rs"),
            Path::new("/s/a/b"),
        );
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].candidates, vec![PathBuf::from("/s/a/../gen/x.rs")]);
        assert_eq!(e[1].candidates, vec![PathBuf::from("/s/a/g/y.rs")]);
    }

    #[test]
    fn a_path_attribute_does_not_carry_to_a_later_unrelated_mod() {
        let e = edges_from(
            "#[path = \"x.rs\"]\nmod c;\nmod d;\n",
            Path::new("/s/a/b.rs"),
            Path::new("/s/a/b"),
        );
        assert_eq!(e[1].candidates[0], PathBuf::from("/s/a/b/d.rs"), "d is ordinary");
    }
}
