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
            out.push(Edge {
                candidates: vec![own.join(p)],
            });
            continue;
        }
        let Some(name) = mod_decl(t) else {
            if !t.is_empty() && !t.starts_with("//") && !t.starts_with("#[") {
                pending_path = None;
            }
            continue;
        };
        if let Some(p) = pending_path.take() {
            out.push(Edge {
                candidates: vec![own.join(p)],
            });
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
    let r = t.strip_prefix("pub ").map(|r| r.trim_start()).unwrap_or(t);
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
        let Ok(c) = std::fs::canonicalize(&p) else {
            continue;
        };
        if !seen.insert(c.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&c) else {
            continue;
        };
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

/// The ledger's ceiling for each crate.
fn ceilings(repo: &Path) -> Result<std::collections::BTreeMap<String, usize>> {
    let p = repo.join("docs/reports/orphan_modules.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p)?)
        .map_err(|e| anyhow::anyhow!("{}: {}", p.display(), e))?;
    let o = v
        .get("ceilings")
        .and_then(|c| c.as_object())
        .ok_or_else(|| anyhow::anyhow!("{}: no `ceilings` object", p.display()))?;
    Ok(o.iter()
        .filter_map(|(k, n)| n.as_u64().map(|n| (k.clone(), n as usize)))
        .collect())
}

/// Negative control: the detector must SEE a planted orphan.
///
/// A gate that cannot fail is not a gate. This builds a two-file crate in a temp
/// directory -- one declared, one not -- and exits non-zero unless exactly the
/// undeclared file comes back.
pub fn self_check() -> Result<()> {
    let dir = std::env::temp_dir().join(format!("tri-mods-selfcheck-{}", std::process::id()));
    let src = dir.join("src");
    std::fs::create_dir_all(&src)?;
    std::fs::write(src.join("main.rs"), "mod wired;\nfn main() {}\n")?;
    std::fs::write(src.join("wired.rs"), "pub fn f() {}\n")?;
    std::fs::write(src.join("stranded.rs"), "pub fn g() {}\n")?;
    let reached = reach(&src.join("main.rs"), &src)?;
    let _ = std::fs::remove_dir_all(&dir);
    let saw_wired = reached.iter().any(|p| p.ends_with("wired.rs"));
    let saw_stranded = reached.iter().any(|p| p.ends_with("stranded.rs"));
    if !saw_wired {
        anyhow::bail!("self-check: a DECLARED file was reported unreachable -- the walk is broken");
    }
    if saw_stranded {
        anyhow::bail!(
            "self-check: an UNDECLARED file was reported as reached. This gate cannot fail, \
             so a green run from it means nothing."
        );
    }
    println!("self-check: planted orphan seen, declared file reached. The gate can fail.");
    Ok(())
}

/// Every crate the workspace declares, rather than a list written once.
///
/// It was `["bootstrap", "cli/tri"]` while `Cargo.toml` named FIVE members, so
/// the census walked two of five and printed their sum as the repository's
/// population. The guard immediately below refuses a crate that has been
/// REMOVED -- "the crate list in this command is stale, and a report of zero
/// orphans would be that staleness" -- and there was none for one being ADDED.
///
/// A guard written as a list goes stale by addition. This reads the list cargo
/// reads, so adding a member adds it to the census in the same commit.
fn members(repo: &Path) -> Result<Vec<String>> {
    members_from(&std::fs::read_to_string(repo.join("Cargo.toml"))?)
}

/// Split out so its test needs no fixture on disk. The first version wrote to a
/// fixed path under the temp dir and the binary runs its tests twice, so the two
/// runs raced over one file and the test failed intermittently -- green by luck
/// until it was not.
fn members_from(text: &str) -> Result<Vec<String>> {
    let i = text.find("members").ok_or_else(|| {
        anyhow::anyhow!("Cargo.toml names no `members` -- this census has no population")
    })?;
    let rest = &text[i..];
    let (a, b) = (
        rest.find('[')
            .ok_or_else(|| anyhow::anyhow!("`members` is not a list"))?,
        rest.find(']')
            .ok_or_else(|| anyhow::anyhow!("`members` list is not closed"))?,
    );
    let out: Vec<String> = rest[a + 1..b]
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if out.is_empty() {
        anyhow::bail!("`members` is empty -- a census over no crates would report zero orphans");
    }
    Ok(out)
}

/// The files cargo compiles without any `mod` line naming them.
///
/// `src/lib.rs` and `src/main.rs` are roots, and so is every `src/bin/*.rs`:
/// cargo discovers binary targets by LAYOUT. Walking `mod` edges alone calls
/// `cli/dlc10/src/bin/dlc10.rs` an orphan, which is a false positive -- and a
/// detector's false positives are how a check gets muted.
fn roots(src: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = ["lib.rs", "main.rs"]
        .iter()
        .map(|f| src.join(f))
        .filter(|p| p.is_file())
        .collect();
    if let Ok(rd) = std::fs::read_dir(src.join("bin")) {
        let mut bins: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .collect();
        bins.sort();
        out.extend(bins);
    }
    out
}

pub fn run(gate: bool) -> Result<()> {
    let repo = repo_root()?;
    let crates = members(&repo)?;
    println!("ORPHANED SOURCE FILES  (nothing declares them; nothing compiles them)");
    println!();
    let mut total_files = 0usize;
    let mut total_orphans = 0usize;
    let mut total_tests = 0usize;
    let ceil = if gate { Some(ceilings(&repo)?) } else { None };
    let mut breaches: Vec<String> = Vec::new();
    for c in &crates {
        let c = c.as_str();
        let src = repo.join(c).join("src");
        if !src.is_dir() {
            anyhow::bail!(
                "{} does not exist -- the crate list in this command is stale, \
                           and a report of zero orphans would be that staleness",
                src.display()
            );
        }
        let rs = roots(&src);
        if rs.is_empty() {
            anyhow::bail!("{}: no lib.rs, main.rs or src/bin/*.rs", src.display());
        }
        let mut reached: BTreeSet<PathBuf> = BTreeSet::new();
        for r in &rs {
            reached.extend(reach(r, &src)?);
        }
        let mut all = Vec::new();
        walk(&src, &mut all)?;
        // The guard is for a reader that stopped working, not for a small crate.
        // `cli/flash-spi` is ONE file: it reaches exactly its root, and that is
        // the right answer. The blunt version -- "fewer than two files reached"
        // -- called that a broken reader and reported a real orphan under the
        // wrong name. Fire only when edges WERE declared and none resolved.
        let declared: usize = rs
            .iter()
            .filter_map(|r| {
                let t = std::fs::read_to_string(r).ok()?;
                let dir = r.parent().unwrap_or(&src).to_path_buf();
                Some(edges_from(&t, r, &dir).len())
            })
            .sum();
        if declared > 0 && reached.len() <= rs.len() {
            anyhow::bail!(
                "{} declares {} `mod` edge(s) from {} root(s) and reaches only {} file(s) -- \
                 the declarations are not being read, and every other file under src/ would \
                 be a false orphan",
                c,
                declared,
                rs.len(),
                reached.len()
            );
        }
        let mut orphans: Vec<(PathBuf, usize, usize)> = Vec::new();
        for p in &all {
            let Ok(cp) = std::fs::canonicalize(p) else {
                continue;
            };
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
            "  {c:<18} {:>3} files under src/, {:>3} compiled, {:>3} orphaned{}",
            all.len(),
            all.len() - orphans.len(),
            orphans.len(),
            if outside > 0 {
                format!("   (+{outside} reached outside src/ via #[path])")
            } else {
                String::new()
            }
        );
        if let Some(cmap) = &ceil {
            match cmap.get(c) {
                None => breaches.push(format!(
                    "{c}: no ceiling in docs/reports/orphan_modules.json. A crate the ledger \
                     does not name is a crate this gate does not watch."
                )),
                Some(&want) if orphans.len() > want => breaches.push(format!(
                    "{c}: orphaned rose {want} -> {}. A file left the build and nothing else \
                     would have said so.",
                    orphans.len()
                )),
                Some(&want) if orphans.len() < want => breaches.push(format!(
                    "{c}: {} orphaned but the ceiling still says {want}. Lower it in this \
                     commit so the next one cannot hide in the slack.",
                    orphans.len()
                )),
                Some(_) => {}
            }
        }
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
    if ceil.is_some() {
        println!();
        if breaches.is_empty() {
            println!("ORPHAN CEILING: CLEAN");
        } else {
            for b in &breaches {
                println!("::error::{b}");
            }
            anyhow::bail!("orphan ceiling breached in {} crate(s)", breaches.len());
        }
    }
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
            vec![
                PathBuf::from("/s/a/b/c.rs"),
                PathBuf::from("/s/a/b/c/mod.rs")
            ],
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

    /// PROBE: the population is what the workspace declares, all of it.
    ///
    /// The list was written once as `["bootstrap", "cli/tri"]` and the
    /// workspace grew to five members, so the census walked two of five and
    /// printed their sum -- 132 -- as the repository's Rust population. The
    /// real number is 136, and the three unwalked crates were watched by no
    /// ratchet at all.
    ///
    /// What this does NOT cover: it exercises `members()` and nothing else, so
    /// it stays green if `run` keeps a list of its own. Measured -- restoring
    /// the hardcoded list left this test passing while the report went back to
    /// 132. `the_ratchet_watches_every_member_and_no_ghost` is the one that
    /// bites.
    #[test]
    fn every_workspace_member_is_in_the_census() {
        let ok = "[workspace]\nresolver = \"2\"\nmembers = [\"a\", \"b/c\", \"d\"]\nexclude = [\"zz\"]\n";
        assert_eq!(members_from(ok).unwrap(), vec!["a", "b/c", "d"]);

        // A census over no crates would report zero orphans and look healthy.
        assert!(
            members_from("[workspace]\nmembers = []\n").is_err(),
            "an empty members list is refused"
        );
        assert!(
            members_from("[package]\nname = \"x\"\n").is_err(),
            "a manifest with no members is refused"
        );
    }

    /// The ledger names every workspace member, and only members.
    ///
    /// This is the test that has teeth. The first version of it exercised
    /// `members()` on a fixture and PASSED while `run` kept its own hardcoded
    /// list -- a test of the helper's existence, not of the caller reading it,
    /// which is the exact defect class this change is about. Written again as
    /// a comparison of two files read by two different readers: `Cargo.toml`
    /// says who the members are, `docs/reports/orphan_modules.json` says who
    /// the ratchet watches, and a member in one and not the other is a crate
    /// nothing would have reported.
    #[test]
    fn the_ratchet_watches_every_member_and_no_ghost() {
        let repo = repo_root().expect("repo root");
        let declared: std::collections::BTreeSet<String> =
            members(&repo).expect("members").into_iter().collect();
        let watched: std::collections::BTreeSet<String> =
            ceilings(&repo).expect("ceilings").keys().cloned().collect();
        let unwatched: Vec<&String> = declared.difference(&watched).collect();
        let ghosts: Vec<&String> = watched.difference(&declared).collect();
        assert!(
            unwatched.is_empty(),
            "workspace members with no ceiling -- watched by nothing: {unwatched:?}"
        );
        assert!(
            ghosts.is_empty(),
            "ceilings for crates the workspace does not declare: {ghosts:?}"
        );
    }

    /// COUNTER: a `src/bin/*.rs` is compiled without any `mod` line naming it.
    ///
    /// Cargo discovers binary targets by LAYOUT. Walking `mod` edges alone
    /// calls `cli/dlc10/src/bin/dlc10.rs` an orphan, and a detector's false
    /// positives are how a check gets muted.
    #[test]
    fn a_bin_target_is_a_root_not_an_orphan() {
        // Per-process, because the binary runs its tests more than once and a
        // fixed path makes the two runs race over one directory.
        let dir = std::env::temp_dir().join(format!("tri-modreach-bin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(src.join("bin")).unwrap();
        std::fs::write(src.join("lib.rs"), "pub fn f() {}\n").unwrap();
        std::fs::write(src.join("bin").join("tool.rs"), "fn main() {}\n").unwrap();

        let rs = roots(&src);
        assert_eq!(rs.len(), 2, "lib.rs and the bin target are both roots");
        assert!(
            rs.iter().any(|p| p.ends_with("bin/tool.rs")),
            "the bin target is one of them"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_path_attribute_does_not_carry_to_a_later_unrelated_mod() {
        let e = edges_from(
            "#[path = \"x.rs\"]\nmod c;\nmod d;\n",
            Path::new("/s/a/b.rs"),
            Path::new("/s/a/b"),
        );
        assert_eq!(
            e[1].candidates[0],
            PathBuf::from("/s/a/b/d.rs"),
            "d is ordinary"
        );
    }
}
