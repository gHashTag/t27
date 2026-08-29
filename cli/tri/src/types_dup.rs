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
        let Some(rest) = t.strip_prefix("struct ") else {
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
                    let n = n.trim();
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
    let TypesCmd::Dup { all } = cmd;
    let root = repo_root()?;
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
        if v == "DUPLICATED" && !*all {
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

    #[test]
    fn a_struct_with_no_parseable_fields_is_still_a_definition() {
        let d = parse("struct Empty {\n}\n");
        assert_eq!(d.len(), 1);
        assert!(d[0].1.fields.is_empty());
    }
}
