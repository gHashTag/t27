//! One type name, more than one definition.
//!
//! WHY THIS EXISTS
//! ---------------
//! The quantifier census computes a domain size `|D|` from declared types. For
//! fifteen names it had to answer "unbounded" — not because the type is
//! infinite, but because **which type** is undetermined: the name has several
//! definitions in the corpus and nothing says which one a spec means.
//!
//! Nothing else in this repository reports that. Every cross-spec type
//! resolution — the census, any future enumerating lowering, any reader — is
//! guessing on these names, and each of them guesses silently.
//!
//! WHAT IT DISTINGUISHES, AND WHY THE DISTINCTION MATTERS
//! -----------------------------------------------------
//!   * CONFLICTED — the field lists differ. Two specs disagree about one type.
//!     A consumer that picks either one is wrong half the time and says nothing.
//!   * DUPLICATED — the same fields written twice. Harmless to a resolver, and
//!     still worth naming: it is the state a CONFLICT starts from, one edit ago.
//!
//! It reports and does not fail. Whether two `ProofStep` types in unrelated
//! subsystems should be renamed is a judgement about the corpus, not a rule a
//! tool may enforce.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum TypesCmd {
    /// Type names with more than one definition in the spec tree.
    Dup {
        /// Print the fields of every definition, not just the conflicted ones.
        #[arg(long)]
        all: bool,
    },
    /// Hold the conflicted set: a new conflict fails, and a resolved one fails
    /// until it is blessed away.
    ///
    /// Identity-keyed, not a count. A count cannot see a SWAP -- one name
    /// resolved while another appears leaves the total unchanged and the ledger
    /// wrong, which is the failure mode the corpus ratchet in this repository
    /// was rebuilt to avoid.
    Ratchet {
        /// Rewrite the ledger from what this run measured.
        #[arg(long)]
        bless: bool,
    },
}

/// Where the conflicted set is pinned.
const LEDGER: &str = "docs/reports/type_conflicts.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Ledger {
    /// What wrote it, so a reader knows which command to re-run.
    generated_by: String,
    /// Why these are tolerated at all.
    reason: String,
    /// Sorted, so a diff stays line-local.
    conflicted: Vec<String>,
}

/// `(new, resolved)` between a pinned set and an observed one.
///
/// Set difference in both directions, deliberately. A COUNT cannot see a swap:
/// one name resolved while another appears leaves the total unchanged and the
/// ledger wrong.
pub fn drift(pinned: &[String], observed: &[String]) -> (Vec<String>, Vec<String>) {
    let p: std::collections::BTreeSet<&String> = pinned.iter().collect();
    let o: std::collections::BTreeSet<&String> = observed.iter().collect();
    (
        o.difference(&p).map(|s| (*s).clone()).collect(),
        p.difference(&o).map(|s| (*s).clone()).collect(),
    )
}

fn ratchet(root: &std::path::Path, observed: &[String], bless: bool) -> Result<()> {
    let path = root.join(LEDGER);
    if bless {
        let l = Ledger {
            generated_by: "tri types ratchet --bless".to_string(),
            reason: "Type names with more than one definition. Each is a name whose \
                     domain size cannot be computed -- not because the type is infinite \
                     but because WHICH type is undetermined. See #2774."
                .to_string(),
            conflicted: observed.to_vec(),
        };
        let mut text = serde_json::to_string_pretty(&l)?;
        text.push('\n');
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        println!(
            "  blessed {} conflicted name(s) -> {}",
            observed.len(),
            LEDGER
        );
        return Ok(());
    }

    // T31 in this repository: absence is NOT amnesty. A verification mode with
    // no oracle is a hard failure, never a silent self-blessing.
    let Ok(text) = std::fs::read_to_string(&path) else {
        println!("  RATCHET: FAIL -- no ledger at {LEDGER}.");
        println!("  Run `tri types ratchet --bless` once, review the file, and commit it.");
        println!("  Absence is not amnesty.");
        std::process::exit(1);
    };
    let l: Ledger =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;

    let (new, gone) = drift(&l.conflicted, observed);

    println!(
        "  ledger {} name(s), observed {}",
        l.conflicted.len(),
        observed.len()
    );
    for n in &new {
        println!("    + {n}  NEW conflict");
    }
    for n in &gone {
        println!("    - {n}  resolved -- remove it from the ledger");
    }
    if new.is_empty() && gone.is_empty() {
        println!("  RATCHET: CLEAN");
        return Ok(());
    }
    println!();
    println!("  A RESOLVED name fails too, on purpose. An entry that stops being");
    println!("  true and stays in the ledger is slack the next conflict hides in --");
    println!("  the same rule the corpus ratchet applies to an unexpected PASS.");
    std::process::exit(1);
}

/// `const Name = struct {` -- the Zig spelling, and the one the corpus uses
/// most. Returns a slice that still begins with the NAME, so the shared path
/// can extract it the same way it does for `struct Name {`.
fn const_struct_name(t: &str) -> Option<&str> {
    let rest = t
        .strip_prefix("const ")
        .or_else(|| t.strip_prefix("pub const "))?;
    let (name, after) = rest.split_once('=')?;
    let name = name.trim();
    if name.is_empty() || !name.chars().next()?.is_ascii_alphabetic() || name.contains(' ') {
        return None;
    }
    if !after.trim().starts_with("struct") {
        return None;
    }
    let start = t.find(name)?;
    Some(&t[start..])
}

/// One `struct Name { ... }` as written: where it is, and its field types in
/// source order.
#[derive(Clone, PartialEq, Eq)]
pub struct Def {
    pub file: String,
    pub line: usize,
    pub fields: Vec<(String, String)>,
}

/// Parse every `struct Name { ... }` out of one source.
///
/// Deliberately syntactic: a `struct` inside a comment or a string is not
/// excluded, because the corpus does not contain one and a parser here would be
/// a second implementation of the lexer that could disagree with it. If that
/// changes, this is the line that has to change with it.
pub fn defs_in(file: &str, src: &str) -> Vec<(String, Def)> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let t = lines[i].trim();
        // W705: THREE spellings declare a type in this corpus, and the first
        // version of this scanner saw one of them. Measured after an adversarial
        // re-count said so:
        //
        //     struct Name { ... }          301 lines
        //     pub struct Name { ... }      154 lines
        //     const Name = struct { ... }  737 lines   <- the Zig idiom
        //
        // Reporting "299 struct definitions" over a corpus that declares types
        // four times that often is not a small error: every duplicate-name
        // verdict was drawn from a quarter of the population.
        if t.starts_with("//") {
            i += 1;
            continue;
        }
        let rest = if let Some(r) = t.strip_prefix("struct ") {
            r
        } else if let Some(r) = t.strip_prefix("pub struct ") {
            r
        } else if let Some(r) = const_struct_name(t) {
            r
        } else {
            i += 1;
            continue;
        };
        // The name ends at a brace, a paren, a semicolon or a space. The first
        // version split on `{` and whitespace only, so `struct CallID(str);`
        // produced the name `CallID(str);` and every newtype fell through to the
        // braced path.
        let name = rest
            .split(|c: char| c == '{' || c == '(' || c == ';' || c.is_whitespace())
            .next()
            .unwrap_or("")
            .trim();
        if name.is_empty() || !name.chars().next().unwrap().is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        // A NEWTYPE has no braced body: `struct CallID(str);`. Scanning forward
        // for a `}` from here swallowed everything up to the next closing brace
        // -- which in specs/account/repo.t27 is three newtypes followed by a
        // real `struct Info { ... }`, and Info vanished. Fifteen definitions
        // were missing from the first version of this scanner, and the count it
        // printed (284) disagreed with `grep -c '^\s*struct '` (299).
        //
        // Found by cross-checking against grep before shipping, which is the
        // only reason it was found at all: the wrong number was plausible.
        let after = rest.trim_start_matches(name).trim_start();
        if after.starts_with('(') || after.starts_with(';') {
            out.push((
                name.to_string(),
                Def {
                    file: file.to_string(),
                    line: i + 1,
                    // The payload of a newtype is not a named field. Recording
                    // it as one would make `struct A(str)` and `struct A(u8)`
                    // compare equal, which is exactly the conflict this command
                    // exists to see.
                    fields: vec![(
                        "(newtype)".to_string(),
                        after
                            .trim_start_matches('(')
                            .split(')')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    )],
                },
            ));
            i += 1;
            continue;
        }
        // `struct PollSlow {}` closes on its own line. Scanning forward for a
        // line that STARTS with `}` walked past it and swallowed the three
        // definitions after it -- the same cascade as the newtype, one shape
        // later, and the reason the count was still four short after that fix.
        if after.contains('}') {
            out.push((
                name.to_string(),
                Def {
                    file: file.to_string(),
                    line: i + 1,
                    fields: Vec::new(),
                },
            ));
            i += 1;
            continue;
        }
        let mut fields = Vec::new();
        let mut j = i + 1;
        while j < lines.len() {
            let l = lines[j].trim();
            if l.starts_with('}') {
                break;
            }
            if !l.starts_with("//") {
                if let Some((n, ty)) = l.split_once(':') {
                    // W707: a field may be `pub name: T`. Splitting on `:` then
                    // rejecting a name containing a space threw every such field
                    // away, so `pub struct HealthStatus { pub is_healthy: bool,
                    // ... }` parsed as a struct with NO fields -- and compared
                    // equal to an unrelated empty definition of the same name,
                    // which the detector then called DUPLICATED instead of
                    // CONFLICTED. Found by an agent asked to check coverage,
                    // not by me.
                    let n = n.trim().strip_prefix("pub ").unwrap_or(n.trim()).trim();
                    let ty = ty.trim().trim_end_matches(',').trim();
                    // A field is `name: Type`. Anything with a space in the name
                    // is a line this does not understand, and is skipped rather
                    // than recorded as a field with a wrong name.
                    if !n.is_empty() && !ty.is_empty() && !n.contains(' ') {
                        fields.push((n.to_string(), ty.to_string()));
                    }
                }
            }
            j += 1;
        }
        out.push((
            name.to_string(),
            Def {
                file: file.to_string(),
                line: i + 1,
                fields,
            },
        ));
        i = j + 1;
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

fn read_specs(root: &std::path::Path) -> Vec<(String, String)> {
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
                    out.push((p.strip_prefix(root).unwrap_or(&p).display().to_string(), s));
                }
            }
        }
    }
    out.sort();
    out
}

/// How two or more definitions of one name relate.
pub fn verdict(defs: &[Def]) -> &'static str {
    let first = &defs[0].fields;
    if defs.iter().all(|d| &d.fields == first) {
        "DUPLICATED"
    } else {
        "CONFLICTED"
    }
}

pub fn run(cmd: &TypesCmd) -> Result<()> {
    let root = repo_root()?;
    let all = match cmd {
        TypesCmd::Dup { all } => *all,
        TypesCmd::Ratchet { bless } => {
            let specs = read_specs(&root);
            if specs.is_empty() {
                anyhow::bail!(
                    "no specs under {}/specs -- nothing was read",
                    root.display()
                );
            }
            let mut by_name: BTreeMap<String, Vec<Def>> = BTreeMap::new();
            for (f, src) in &specs {
                for (n, d) in defs_in(f, src) {
                    by_name.entry(n).or_default().push(d);
                }
            }
            let observed: Vec<String> = by_name
                .iter()
                .filter(|(_, v)| v.len() > 1 && verdict(v) == "CONFLICTED")
                .map(|(k, _)| k.clone())
                .collect();
            return ratchet(&root, &observed, *bless);
        }
    };
    let specs = read_specs(&root);
    if specs.is_empty() {
        anyhow::bail!(
            "no specs under {}/specs -- nothing was read",
            root.display()
        );
    }
    let mut by_name: BTreeMap<String, Vec<Def>> = BTreeMap::new();
    let mut total = 0usize;
    for (f, src) in &specs {
        for (n, d) in defs_in(f, src) {
            total += 1;
            by_name.entry(n).or_default().push(d);
        }
    }

    let multi: Vec<(&String, &Vec<Def>)> = by_name.iter().filter(|(_, v)| v.len() > 1).collect();
    let conflicted: Vec<_> = multi
        .iter()
        .filter(|(_, v)| verdict(v) == "CONFLICTED")
        .collect();

    for (name, defs) in &multi {
        let v = verdict(defs);
        if v == "DUPLICATED" && !all {
            continue;
        }
        println!("  {name}  {v}  ({} definitions)", defs.len());
        for d in defs.iter() {
            println!("      {}:{}", d.file, d.line);
            println!(
                "          {}",
                if d.fields.is_empty() {
                    "(no fields this reader could parse)".to_string()
                } else {
                    d.fields
                        .iter()
                        .map(|(n, t)| format!("{n}: {t}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            );
        }
        println!();
    }

    println!(
        "  {total} struct definition(s), {} distinct name(s)",
        by_name.len()
    );
    println!("  {} name(s) defined more than once", multi.len());
    println!(
        "      CONFLICTED (field lists differ)  {}",
        conflicted.len()
    );
    println!(
        "      DUPLICATED (same fields twice)   {}",
        multi.len() - conflicted.len()
    );
    println!();
    println!("  A CONFLICTED name has no answer to \"what is |D|\" -- not because the");
    println!("  type is infinite but because WHICH type is undetermined. The quantifier");
    println!("  census reports these as unbounded for that reason (#2774).");
    println!();
    println!("  This reports and does not fail. Whether two same-named types in");
    println!("  unrelated subsystems should be renamed is a judgement about the corpus.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Vec<(String, Def)> {
        defs_in("x.t27", src)
    }

    #[test]
    fn fields_are_read_in_order_and_comments_are_not_fields() {
        let d = parse("struct S {\n    // a comment: not a field\n    a: u8,\n    b: Trit,\n}\n");
        assert_eq!(d.len(), 1);
        assert_eq!(
            d[0].1.fields,
            vec![("a".into(), "u8".into()), ("b".into(), "Trit".into())]
        );
    }

    /// The distinction the command exists to make.
    #[test]
    fn same_fields_is_duplicated_and_a_difference_is_conflicted() {
        let a = parse("struct S {\n    a: u8,\n}\n")[0].1.clone();
        let b = parse("struct S {\n    a: u8,\n}\n")[0].1.clone();
        let c = parse("struct S {\n    a: u16,\n}\n")[0].1.clone();
        assert_eq!(verdict(&[a.clone(), b]), "DUPLICATED");
        assert_eq!(verdict(&[a, c]), "CONFLICTED");
    }

    /// A field ORDER difference is a difference: two specs that list the same
    /// fields in different orders describe different layouts to a backend.
    #[test]
    fn field_order_is_part_of_the_definition() {
        let a = parse("struct S {\n    a: u8,\n    b: u8,\n}\n")[0]
            .1
            .clone();
        let b = parse("struct S {\n    b: u8,\n    a: u8,\n}\n")[0]
            .1
            .clone();
        assert_eq!(verdict(&[a, b]), "CONFLICTED");
    }

    /// Two definitions in ONE file count. The hazard is not cross-file, it is
    /// one name with two meanings, and a file can disagree with itself.
    #[test]
    fn two_definitions_in_one_file_are_still_two() {
        let d = parse("struct S {\n    a: u8,\n}\n\nstruct S {\n    a: u16,\n}\n");
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].0, "S");
        assert_eq!(d[1].0, "S");
    }

    /// The bug that made the first version's count wrong: a newtype has no
    /// braced body, so scanning for `}` swallowed every definition after it.
    #[test]
    fn a_newtype_does_not_swallow_the_definitions_after_it() {
        let d =
            parse("struct OrgID(str);\nstruct AccessToken(str);\nstruct Info {\n    a: u8,\n}\n");
        let names: Vec<&str> = d.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["OrgID", "AccessToken", "Info"], "{names:?}");
    }

    /// And two newtypes over different payloads are a CONFLICT, not a match.
    #[test]
    fn newtypes_over_different_payloads_conflict() {
        let a = parse("struct A(str);\n")[0].1.clone();
        let b = parse("struct A(u8);\n")[0].1.clone();
        assert_eq!(verdict(&[a, b]), "CONFLICTED");
    }

    /// The second cascade: an empty struct closing on its own line.
    #[test]
    fn an_empty_one_line_struct_does_not_swallow_the_next() {
        let d = parse("struct A {}\nstruct B {}\nstruct C {\n    x: u8,\n}\n");
        let names: Vec<&str> = d.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["A", "B", "C"], "{names:?}");
    }

    /// The case a count cannot see: one resolved, one appeared, total unchanged.
    #[test]
    fn a_swap_at_a_constant_count_is_two_findings() {
        let pinned = vec!["A".to_string(), "B".to_string()];
        let observed = vec!["A".to_string(), "C".to_string()];
        let (new, gone) = drift(&pinned, &observed);
        assert_eq!(new, vec!["C".to_string()]);
        assert_eq!(gone, vec!["B".to_string()]);
        assert_eq!(pinned.len(), observed.len(), "the count is identical");
    }

    /// A resolved conflict is a failure, not a quiet win: slack in the ledger
    /// is where the next one hides.
    #[test]
    fn a_resolved_name_is_reported() {
        let (new, gone) = drift(&["A".to_string(), "B".to_string()], &["A".to_string()]);
        assert!(new.is_empty());
        assert_eq!(gone, vec!["B".to_string()]);
    }

    #[test]
    fn agreement_is_silence() {
        let (new, gone) = drift(&["A".to_string()], &["A".to_string()]);
        assert!(new.is_empty() && gone.is_empty());
    }

    /// A `pub` field is a field. Dropping them made a five-field struct read as
    /// empty, and an empty struct compares equal to any other empty one.
    #[test]
    fn a_pub_field_is_read() {
        let d = parse("pub struct S {\n    pub is_healthy: bool,\n    pub code: u16,\n}\n");
        assert_eq!(
            d[0].1.fields,
            vec![
                ("is_healthy".to_string(), "bool".to_string()),
                ("code".to_string(), "u16".to_string())
            ]
        );
    }

    /// And the consequence: five fields versus none is a CONFLICT, not a match.
    #[test]
    fn a_populated_struct_conflicts_with_an_empty_one_of_the_same_name() {
        let a = parse("pub struct S {\n    pub a: bool,\n}\n")[0].1.clone();
        let b = parse("pub const S = struct {\n};\n")[0].1.clone();
        assert_eq!(verdict(&[a, b]), "CONFLICTED");
    }

    #[test]
    fn a_struct_with_no_parseable_fields_is_still_a_definition() {
        let d = parse("struct Empty {\n}\n");
        assert_eq!(d.len(), 1);
        assert!(d[0].1.fields.is_empty());
    }
}
