//! `tri lean reach` -- which proof files the build root actually reaches.
//!
//! Lake builds a `lean_lib` by building its root module and, transitively, whatever
//! that module imports. `roots` defaults to the library name and `globs` defaults to
//! `Glob.one` of each root, so a lakefile that names no globs compiles exactly one
//! file plus its import closure. `proofs/lean4/Trinity.lean` is twelve lines long and
//! holds nine imports; every other file in the tree is reached only if one of those
//! nine leads to it.
//!
//! Twelve do not. Fifteen thousand lines sit beside the seven thousand that compile,
//! in the same directory, under the same lakefile, indistinguishable in an editor.
//!
//! A build graph is a claim about coverage that nothing prints. `lake build` prints
//! what it compiled, never what it skipped, so a file outside the graph produces no
//! output at all -- not an error, not a warning, not a line. The only way to see the
//! omission is to ask the root what it reaches and compare that with what exists.
//!
//! This also re-populates a ratchet. `lean-proofs.yml` counts `sorry` by grepping the
//! whole directory; the build compiles the closure. Those are different sets, and four
//! of the five admitted proofs the gate counts are in files the build never opens.

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One proof file, as the tree holds it.
pub struct LeanFile {
    pub module: String,
    pub lines: usize,
    pub sorries: usize,
}

/// Line counts `sorry` the way `lean-proofs.yml` does: one per matching LINE, with the
/// identifier delimited so `sorryAx` and `no_sorry` do not count.
pub fn sorries_in(src: &str) -> usize {
    src.lines().filter(|l| line_has_sorry(l)).count()
}

fn line_has_sorry(line: &str) -> bool {
    let b = line.as_bytes();
    let mut i = 0;
    while let Some(off) = line[i..].find("sorry") {
        let s = i + off;
        let e = s + 5;
        let before_ok = s == 0 || !is_word(b[s - 1]);
        let after_ok = e == b.len() || !is_word(b[e]);
        if before_ok && after_ok {
            return true;
        }
        i = s + 1;
    }
    false
}

fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The module name a path denotes, given the library root directory.
///
/// `Trinity.lean` is the module `Trinity`; `Trinity/IcarusLowerable/Ast.lean` is
/// `Trinity.IcarusLowerable.Ast`. Returns `None` for anything that is not a `.lean`
/// file under `dir`.
pub fn module_of(dir: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(dir).ok()?;
    let s = rel.to_str()?.strip_suffix(".lean")?;
    Some(s.replace(std::path::MAIN_SEPARATOR, "."))
}

/// Every `import X` in a file, in order. Lean requires imports at the top with no
/// leading whitespace, so an indented `import` inside a docstring is not one.
pub fn imports_in(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|l| l.strip_prefix("import "))
        .map(|r| r.split_whitespace().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The library root a lakefile declares, from `lean_lib «Name»` or `lean_lib Name`.
///
/// Returns `None` rather than guessing: a lakefile whose shape this does not recognise
/// must stop the command, because a wrong root turns every file in the tree into a
/// false finding.
pub fn lib_root(lakefile: &str) -> Option<String> {
    for line in lakefile.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("lean_lib ") else {
            continue;
        };
        let name = rest
            .trim()
            .trim_start_matches('\u{ab}')
            .split(|c: char| c == '\u{bb}' || c.is_whitespace())
            .next()
            .unwrap_or("");
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// The transitive closure of local imports from `root`, as module names.
///
/// Imports that resolve to no file in the map are dependencies (Mathlib, Std) and are
/// simply not followed -- they are outside the library, not stranded inside it.
pub fn closure(files: &BTreeMap<String, Vec<String>>, root: &str) -> BTreeSet<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![root.to_string()];
    while let Some(m) = stack.pop() {
        if seen.contains(&m) {
            continue;
        }
        let Some(imports) = files.get(&m) else {
            continue;
        };
        seen.insert(m);
        for i in imports {
            if files.contains_key(i) {
                stack.push(i.clone());
            }
        }
    }
    seen
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

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().and_then(|x| x.to_str()) == Some("lean") {
            out.push(p);
        }
    }
    Ok(())
}

pub fn run(all: bool) -> Result<()> {
    let repo = repo_root()?;
    let dir = repo.join("proofs/lean4");
    let lakefile = dir.join("lakefile.lean");
    let lake = std::fs::read_to_string(&lakefile)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", lakefile.display(), e))?;

    if lake.contains("globs") {
        anyhow::bail!(
            "{} sets `globs`, which makes the library build more than its root's import \
             closure. This command reads the closure, so its answer would be wrong here.",
            lakefile.display()
        );
    }
    let root = lib_root(&lake).ok_or_else(|| {
        anyhow::anyhow!(
            "no `lean_lib` in {} -- without the root there is nothing to walk from, and \
             a report of zero unreachable files would be this parser, not the tree",
            lakefile.display()
        )
    })?;

    let mut paths = Vec::new();
    walk(&dir, &mut paths)?;
    let mut info: BTreeMap<String, LeanFile> = BTreeMap::new();
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in &paths {
        if p.file_name().and_then(|s| s.to_str()) == Some("lakefile.lean") {
            continue;
        }
        let Some(m) = module_of(&dir, p) else { continue };
        let src = std::fs::read_to_string(p)?;
        edges.insert(m.clone(), imports_in(&src));
        info.insert(
            m.clone(),
            LeanFile {
                module: m,
                lines: src.lines().count(),
                sorries: sorries_in(&src),
            },
        );
    }

    if !info.contains_key(&root) {
        anyhow::bail!(
            "lakefile names `{root}` but {}/{root}.lean does not exist",
            dir.display()
        );
    }
    let reached = closure(&edges, &root);
    if reached.len() < 2 {
        anyhow::bail!(
            "the root `{root}` reaches only itself -- either the library really is one \
             file, or `import` lines are not being read; refusing to call every other \
             file stranded on that basis"
        );
    }

    let sum = |set: bool| -> (usize, usize, usize) {
        let it = info.values().filter(|f| reached.contains(&f.module) == set);
        it.fold((0, 0, 0), |(n, l, s), f| (n + 1, l + f.lines, s + f.sorries))
    };
    let (rn, rl, rs) = sum(true);
    let (un, ul, us) = sum(false);

    println!("BUILD-GRAPH REACHABILITY  ({}/{}.lean)", dir.display(), root);
    println!();
    println!("                       files    lines    sorry");
    println!("  reached by the root  {rn:>5}  {rl:>7}  {rs:>7}");
    println!("  NOT reached          {un:>5}  {ul:>7}  {us:>7}");
    println!();

    if un == 0 {
        println!("Every file under the lakefile is in the root's import closure.");
        return Ok(());
    }

    println!("Stranded -- present, shaped like proofs, compiled by nothing:");
    for f in info.values().filter(|f| !reached.contains(&f.module)) {
        let s = if f.sorries > 0 {
            format!("   {} sorry", f.sorries)
        } else {
            String::new()
        };
        println!("    {:<44} {:>5} lines{}", f.module, f.lines, s);
    }
    println!();
    if all {
        println!("Reached:");
        for m in &reached {
            println!("    {m}");
        }
        println!();
    }
    println!(
        "`lake build` prints what it compiled and never what it skipped, so these produce\n\
         no output at all -- not an error, not a warning. A build graph is a claim about\n\
         coverage that nothing prints, and the root file is the whole of the claim."
    );
    if us > 0 {
        println!();
        println!(
            "{us} of the {} admitted proofs counted by `lean-proofs.yml` are in these files.\n\
             That gate greps the directory; the build compiles the closure. Its comment says\n\
             a `sorry` compiles -- for these it does not, because nothing compiles them.",
            rs + us
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_nothing_imports_is_not_in_the_closure() {
        let mut g = BTreeMap::new();
        g.insert("R".into(), vec!["R.A".into(), "Mathlib".into()]);
        g.insert("R.A".into(), vec!["R.B".into()]);
        g.insert("R.B".into(), vec![]);
        g.insert("R.Orphan".into(), vec!["R.A".into()]);
        let c = closure(&g, "R");
        assert!(c.contains("R.B"), "reached through R.A");
        assert!(
            !c.contains("R.Orphan"),
            "importing a reached module does not make a file reachable -- edges point \
             the other way, and this is the whole defect"
        );
        assert!(!c.contains("Mathlib"), "a dependency is not a stranded file");
    }

    #[test]
    fn imports_are_read_only_at_column_zero() {
        let src = "import A\nimport B.C\n  import D\n/-- import E -/\ndef f := 1\n";
        assert_eq!(imports_in(src), vec!["A", "B.C"], "indented ones are prose");
    }

    #[test]
    fn sorry_is_counted_per_line_and_delimited() {
        assert_eq!(sorries_in("  exact sorry\n"), 1);
        assert_eq!(sorries_in("  sorry; sorry\n"), 1, "the gate counts lines");
        assert_eq!(sorries_in("  exact sorryAx _\n"), 0);
        assert_eq!(sorries_in("  def no_sorry := 1\n"), 0);
    }

    #[test]
    fn the_root_comes_from_the_lakefile_not_from_a_guess() {
        assert_eq!(lib_root("@[default_target]\nlean_lib \u{ab}Trinity\u{bb} where\n").as_deref(), Some("Trinity"));
        assert_eq!(lib_root("lean_lib Foo where\n").as_deref(), Some("Foo"));
        assert_eq!(lib_root("package X where\n"), None, "no lib means no root");
    }

    #[test]
    fn module_names_come_from_paths_under_the_library_dir() {
        let d = Path::new("/p/lean4");
        assert_eq!(module_of(d, Path::new("/p/lean4/Trinity.lean")).as_deref(), Some("Trinity"));
        assert_eq!(
            module_of(d, Path::new("/p/lean4/Trinity/Icarus/Ast.lean")).as_deref(),
            Some("Trinity.Icarus.Ast")
        );
    }
}
