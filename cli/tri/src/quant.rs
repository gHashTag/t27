//! Every quantified clause in the corpus, with the size of the domain it ranges
//! over.
//!
//! WHY THIS EXISTS
//! ---------------
//! 90% of everything the t27 parser discards is universal quantification, in
//! four notations. What to DO about that is #2774, an owner decision that binds
//! four backends. Three independently written proposals for it disagreed about
//! the lowering and agreed exactly about the first step: **report before you
//! lower**, because the ceiling cannot be chosen without knowing the
//! distribution of domain sizes, and nobody has ever measured it.
//!
//! This is that report and nothing else. It changes no parse, no lowering, no
//! generated artefact, and no discard count. It reads the specs.
//!
//! WHAT A DOMAIN SIZE MEANS HERE
//! -----------------------------
//! `|D|` is computed from DECLARED TYPES ALONE -- never from a guard, never from
//! a value. A binder over `Trit` ranges over 3 values whatever the body says
//! about it. Guard narrowing (`x.len() == 4` collapsing a slice axis) is
//! deliberately NOT implemented: it is the part that needs a semantics, and this
//! command must not be the thing that quietly decides one.
//!
//! `BOTTOM` -- printed as `unbounded` -- is absorbing. A product with one
//! unbounded axis is unbounded. That is the honest answer for `string`, for a
//! slice with no pinned length, and for a type this command cannot resolve:
//! **an unresolved name is not assumed small.**
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum QuantCmd {
    /// Every quantified clause, its binders, and the size of its domain.
    Report {
        /// Print one line per clause instead of the summary.
        #[arg(long)]
        full: bool,
        /// Domain sizes at or below this are called walkable. Choosing this
        /// number is the decision this report exists to inform; the default is
        /// deliberately small.
        #[arg(long, default_value_t = 65536u128)]
        ceiling: u128,
    },
}

/// What the type of one binder is worth.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Size {
    /// A finite, statically known count.
    Finite(u128),
    /// Not computable from the declaration. Absorbing.
    Unbounded,
}

fn primitive(t: &str) -> Option<u128> {
    Some(match t {
        "bool" => 2,
        "Trit" | "trit" => 3,
        "u2" | "i2" => 4,
        "u4" | "i4" => 16,
        "u8" | "i8" | "char" => 256,
        "u16" | "i16" => 65_536,
        "u32" | "i32" | "f32" => 1u128 << 32,
        "u64" | "i64" | "usize" | "isize" | "f64" => 1u128 << 64,
        "u128" | "i128" => u128::MAX,
        _ => return None,
    })
}

/// `struct Name { field: Type, ... }` as written in the specs, by name.
///
/// A name defined more than once is recorded as CONFLICTED and treated as
/// unbounded: 50 struct names in this corpus have several definitions, and
/// picking one of them would change `|D|` by an unbounded factor with nothing
/// saying which was picked.
struct Structs {
    fields: BTreeMap<String, Vec<String>>,
    conflicted: std::collections::BTreeSet<String>,
}

/// W704: this used to have its OWN struct scanner, and that scanner had two
/// bugs -- a newtype (`struct CallID(str);`) and a one-line empty body
/// (`struct PollSlow {}`) each swallowed the definitions after them. It counted
/// 284 definitions where `grep` counted 299, and reported 15 conflicted names
/// where there are 16.
///
/// The scanner now lives in one place. Two implementations of one measurement
/// is two numbers that can disagree, and these did.
fn scan_structs(specs: &[(PathBuf, String)]) -> Structs {
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut conflicted = std::collections::BTreeSet::new();
    for (p, src) in specs {
        for (name, d) in crate::types_dup::defs_in(&p.display().to_string(), src) {
            let fs: Vec<String> = d.fields.iter().map(|(_, t)| t.clone()).collect();
            if let Some(prev) = fields.get(&name) {
                if *prev != fs {
                    conflicted.insert(name.clone());
                }
            }
            fields.insert(name, fs);
        }
    }
    Structs { fields, conflicted }
}

fn size_of(ty: &str, s: &Structs, depth: usize) -> Size {
    let ty = ty.trim().trim_end_matches(',').trim();
    if depth > 8 {
        return Size::Unbounded;
    }
    if let Some(n) = primitive(ty) {
        return Size::Finite(n);
    }
    // `[N]T` is |T|^N when N is a literal; `[]T` and `[T]` have no pinned length.
    if let Some(rest) = ty.strip_prefix('[') {
        if let Some((n, elem)) = rest.split_once(']') {
            let n = n.trim();
            if n.is_empty() {
                return Size::Unbounded;
            }
            if let Ok(k) = n.parse::<u32>() {
                if let Size::Finite(e) = size_of(elem, s, depth + 1) {
                    return match e.checked_pow(k.min(64)) {
                        Some(v) => Size::Finite(v),
                        None => Size::Finite(u128::MAX),
                    };
                }
            }
            return Size::Unbounded;
        }
    }
    if s.conflicted.contains(ty) {
        return Size::Unbounded;
    }
    if let Some(fs) = s.fields.get(ty) {
        let mut acc: u128 = 1;
        for f in fs {
            match size_of(f, s, depth + 1) {
                Size::Finite(n) => acc = acc.saturating_mul(n),
                Size::Unbounded => return Size::Unbounded,
            }
        }
        return Size::Finite(acc);
    }
    Size::Unbounded
}

#[derive(Clone)]
struct Clause {
    file: String,
    line: usize,
    notation: &'static str,
    binders: Vec<(String, String)>,
    text: String,
}

/// The four notations, recognised on the source line.
fn scan_clauses(specs: &[(PathBuf, String)]) -> Vec<Clause> {
    let mut out = Vec::new();
    for (p, src) in specs {
        for (i, raw) in src.lines().enumerate() {
            let t = raw.trim();
            let (notation, binder_text) = if let Some(r) = t.strip_prefix("forall ") {
                ("prefix", r.to_string())
            } else if let Some(idx) = t.find(": forall ") {
                ("colon", t[idx + 9..].to_string())
            } else if let Some(idx) = t.find(" for all ") {
                ("suffix-all", t[idx + 9..].to_string())
            } else if let Some(idx) = t.find(" for any ") {
                ("suffix-any", t[idx + 9..].to_string())
            } else {
                continue;
            };
            // `x : T, y : U, <body>` -- binders are the leading `name : Type`
            // pairs. A comma-separated piece with no colon ends the binder list;
            // everything after it is body, and this command does not read bodies.
            let mut binders = Vec::new();
            for piece in binder_text.split(',') {
                let piece = piece.trim();
                let Some((n, ty)) = piece.split_once(':') else {
                    break;
                };
                let n = n.trim();
                let ty = ty.trim();
                if n.is_empty() || ty.is_empty() || n.contains(' ') {
                    break;
                }
                binders.push((n.to_string(), ty.to_string()));
            }
            out.push(Clause {
                file: p.display().to_string(),
                line: i + 1,
                notation,
                binders,
                text: t.to_string(),
            });
        }
    }
    out
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running `git rev-parse --show-toplevel`")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

fn read_specs(root: &std::path::Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("specs")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().map(|n| n == "scratch").unwrap_or(false) {
                    continue;
                }
                stack.push(p);
            } else if p.extension().and_then(|x| x.to_str()) == Some("t27") {
                if let Ok(s) = std::fs::read_to_string(&p) {
                    let rel = p.strip_prefix(root).unwrap_or(&p).to_path_buf();
                    out.push((rel, s));
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn run(cmd: &QuantCmd) -> Result<()> {
    let QuantCmd::Report { full, ceiling } = cmd;
    let root = repo_root()?;
    let specs = read_specs(&root);
    if specs.is_empty() {
        anyhow::bail!(
            "no specs under {}/specs -- nothing was read",
            root.display()
        );
    }
    let structs = scan_structs(&specs);
    let clauses = scan_clauses(&specs);

    let mut by_notation: BTreeMap<&str, usize> = BTreeMap::new();
    let (mut walkable, mut over, mut unbounded, mut no_binder) = (0usize, 0usize, 0usize, 0usize);
    let mut walkable_sizes: Vec<(u128, String, usize)> = Vec::new();

    for c in &clauses {
        *by_notation.entry(c.notation).or_default() += 1;
        if c.binders.is_empty() {
            no_binder += 1;
            if *full {
                println!("  {}:{}  no binder ({})", c.file, c.line, c.notation);
            }
            continue;
        }
        let mut total: Option<u128> = Some(1);
        for (_, ty) in &c.binders {
            match size_of(ty, &structs, 0) {
                Size::Finite(n) => total = total.map(|t| t.saturating_mul(n)),
                Size::Unbounded => {
                    total = None;
                    break;
                }
            }
        }
        let verdict = match total {
            None => {
                unbounded += 1;
                "unbounded".to_string()
            }
            Some(n) if n <= *ceiling => {
                walkable += 1;
                walkable_sizes.push((n, c.file.clone(), c.line));
                format!("walkable |D| = {n}")
            }
            Some(n) => {
                over += 1;
                format!("finite but over ceiling |D| = {n}")
            }
        };
        if *full {
            println!(
                "  {}:{}  {}  [{}]  {}",
                c.file,
                c.line,
                verdict,
                c.binders
                    .iter()
                    .map(|(n, t)| format!("{n}: {t}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                &c.text[..c.text.len().min(52)]
            );
        }
    }

    println!();
    println!("  quantified clauses found      {}", clauses.len());
    for (n, k) in &by_notation {
        println!("    {:<26} {}", n, k);
    }
    println!();
    println!("  DOMAIN, from declared types only, ceiling {ceiling}");
    println!("    walkable                    {walkable}");
    println!("    finite but over the ceiling {over}");
    println!("    unbounded                   {unbounded}");
    println!("    no binder this can read     {no_binder}");
    if !walkable_sizes.is_empty() {
        walkable_sizes.sort();
        let biggest = walkable_sizes.last().unwrap();
        println!();
        println!(
            "    largest walkable domain     {} ({}:{})",
            biggest.0, biggest.1, biggest.2
        );
    }
    if !structs.conflicted.is_empty() {
        println!();
        println!(
            "  {} struct name(s) have MORE THAN ONE definition and are treated as",
            structs.conflicted.len()
        );
        println!("  unbounded. Picking one would change |D| by an unbounded factor with");
        println!("  nothing recording which was picked:");
        for n in structs.conflicted.iter().take(8) {
            println!("      {n}");
        }
    }
    println!();
    println!("  No guard is read. `x.len() == 4` does not narrow anything here --");
    println!("  that is the part that needs a semantics, and this report must not be");
    println!("  the thing that quietly decides one. See #2774.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Structs {
        Structs {
            fields: [(
                "Pair".to_string(),
                vec!["bool".to_string(), "Trit".to_string()],
            )]
            .into_iter()
            .collect(),
            conflicted: Default::default(),
        }
    }

    #[test]
    fn a_trit_is_three_and_a_struct_is_the_product() {
        assert_eq!(size_of("Trit", &s(), 0), Size::Finite(3));
        assert_eq!(size_of("Pair", &s(), 0), Size::Finite(6));
    }

    /// The rule that matters: an unresolved name is NOT assumed small.
    #[test]
    fn an_unknown_type_is_unbounded_not_one() {
        assert_eq!(size_of("ModelConfig", &s(), 0), Size::Unbounded);
        assert_eq!(size_of("string", &s(), 0), Size::Unbounded);
        assert_eq!(size_of("[]u8", &s(), 0), Size::Unbounded);
    }

    #[test]
    fn a_pinned_array_length_is_a_power_and_an_unpinned_one_is_not() {
        assert_eq!(size_of("[3]Trit", &s(), 0), Size::Finite(27));
        assert_eq!(size_of("[]Trit", &s(), 0), Size::Unbounded);
    }

    /// A conflicted struct name is unbounded even though its fields resolve.
    #[test]
    fn a_struct_defined_twice_is_unbounded() {
        let mut st = s();
        st.conflicted.insert("Pair".to_string());
        assert_eq!(size_of("Pair", &st, 0), Size::Unbounded);
    }

    #[test]
    fn the_four_notations_are_recognised() {
        let src = "\
    invariant a: forall c : Cfg, c.x > 0
    forall input : In
    assert f(a) == f(b) for all Trit
    assert g(a) == g(b) for any a : Trit, b : Trit
";
        let cs = scan_clauses(&[(PathBuf::from("x.t27"), src.to_string())]);
        let mut kinds: Vec<&str> = cs.iter().map(|c| c.notation).collect();
        kinds.sort();
        assert_eq!(kinds, vec!["colon", "prefix", "suffix-all", "suffix-any"]);
    }

    /// A suffix with no `name : Type` yields no binder rather than a wrong one.
    #[test]
    fn a_prose_suffix_has_no_binder() {
        let cs = scan_clauses(&[(
            PathBuf::from("x.t27"),
            "    assert p(x) for all positive integer n\n".to_string(),
        )]);
        assert_eq!(cs.len(), 1);
        assert!(cs[0].binders.is_empty(), "{:?}", cs[0].binders);
    }
}
