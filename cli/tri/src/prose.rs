//! Specs a literate author left prose in, and how far that prose is from code.
//!
//! WHY THIS EXISTS
//! ---------------
//! `#` starts a line comment in t27 -- deliberately, so that a Markdown heading
//! in a `.t27` file becomes a comment "which is what they always were in
//! intent" (bootstrap/src/compiler.rs). What was never handled is the PARAGRAPH
//! under the heading. A spec written in the literate style
//!
//!     ## Specification
//!
//!     Zig-backed FFI bridge for Trinity VSA core.
//!
//! parses the heading and stops on the sentence. Measured the day this was
//! written: 13 specs use the `## Specification` template and NONE of them
//! generated; 16 carry any Markdown heading and none generated. Against a base
//! rate of 104 non-generating specs in 650, that is not a coincidence -- but
//! the heading is not the cause, the paragraph under it is. The first reading
//! blamed the heading and a two-line probe disproved it.
//!
//! WHAT IT DOES NOT DO
//! -------------------
//! It never edits a line the compiler did not stop on, and it refuses outright
//! when the line it stops on looks like code. A spec whose real obstacle is an
//! unimplemented construct is reported as such and left alone.
use crate::unparsed::parse_failures;
use anyhow::Result;
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum ProseCmd {
    /// Which specs are blocked only by prose, and how many lines that is.
    ///
    /// Runs the compiler, comments the line it names, and asks again -- so the
    /// question "is this prose?" is answered by the compiler rather than by a
    /// pattern. Two guards stop it from commenting code; see `--fix`.
    Report {
        /// Rewrite the files whose only obstacle is prose.
        ///
        /// Prefixes each blocking line with `// `, preserving the text exactly.
        /// Refuses a file the moment the compiler stops on something that looks
        /// like code, and refuses the whole rewrite if the set of declaration
        /// lines is not identical before and after.
        #[arg(long)]
        fix: bool,
        /// Report every spec, including the ones blocked by code.
        #[arg(long)]
        all: bool,
    },
}

/// A line that opens or continues a t27 declaration.
fn is_decl(line: &str) -> bool {
    const KW: [&str; 21] = [
        "fn",
        "struct",
        "enum",
        "const",
        "var",
        "test",
        "invariant",
        "bench",
        "module",
        "let",
        "return",
        "if",
        "else",
        "for",
        "while",
        "switch",
        "try",
        "impl",
        "type",
        "use",
        "using",
    ];
    let t = line.trim_start();
    let t = t.strip_prefix("pub ").unwrap_or(t);
    KW.iter().any(|k| {
        t.strip_prefix(k).is_some_and(|r| {
            r.is_empty() || r.starts_with(|c: char| !c.is_alphanumeric() && c != '_')
        })
    })
}

/// The discriminator that a shape test gets wrong.
///
/// A rule refusing every line with `(`, `:`, `=` or `->` refuses ORDINARY
/// ENGLISH: measured across the 13 literate specs, every one stopped on a
/// sentence like `Part of Phase 4: Quality & Performance (Issue #48)`. Prose
/// carries punctuation. What separates them is the END of the line -- t27 code
/// at a line boundary ends with `{`, `;` or `,` and a sentence does not.
///
/// It errs toward refusing: a wrapped sentence ending in a comma is declined
/// rather than edited, which costs a repair and never costs a line of code.
fn is_structural(line: &str) -> bool {
    matches!(
        line.trim_end().chars().last(),
        Some('{') | Some(';') | Some(',') | Some('}')
    )
}

fn decls(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|l| is_decl(l))
        .map(|l| l.trim().to_string())
        .collect()
}

/// The reasons a walk decides nothing. They are literals rather than `String`
/// because the set is fixed and a reader needs to be able to enumerate it --
/// see `BROKEN_INSTRUMENT`, which names the two that mean the tool failed.
const BROKEN_INSTRUMENT: &[&str] = &["unreadable", "compiler did not run", "cannot write probe"];

enum Outcome {
    Prose(usize),
    BlockedByCode(usize, String),
    Other(&'static str),
}

fn walk(t27c: &Path, root: &Path, spec: &Path, cap: usize) -> (Vec<String>, Outcome) {
    let Ok(text) = std::fs::read_to_string(spec) else {
        return (Vec::new(), Outcome::Other("unreadable"));
    };
    let orig: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let mut cur = orig.clone();
    let mut touched = 0usize;
    let tmp = std::env::temp_dir().join(format!(
        "tri-prose-{}.t27",
        spec.file_name().unwrap_or_default().to_string_lossy()
    ));
    for _ in 0..cap {
        if std::fs::write(&tmp, format!("{}\n", cur.join("\n"))).is_err() {
            return (cur, Outcome::Other("cannot write probe"));
        }
        let out = std::process::Command::new(t27c)
            .arg("check")
            .arg(&tmp)
            .current_dir(root)
            .output();
        let Ok(out) = out else {
            return (cur, Outcome::Other("compiler did not run"));
        };
        if out.status.success() {
            let _ = std::fs::remove_file(&tmp);
            if decls(&orig) != decls(&cur) {
                return (cur, Outcome::Other("declarations changed -- refused"));
            }
            return (cur, Outcome::Prose(touched));
        }
        let text = String::from_utf8_lossy(&out.stderr) + String::from_utf8_lossy(&out.stdout);
        let Some(n) = line_of(&text) else {
            let _ = std::fs::remove_file(&tmp);
            return (cur, Outcome::Other("no line in the error"));
        };
        if n == 0 || n > cur.len() {
            let _ = std::fs::remove_file(&tmp);
            return (cur, Outcome::Other("line out of range"));
        }
        let line = cur[n - 1].clone();
        if line.trim_start().starts_with("//") {
            let _ = std::fs::remove_file(&tmp);
            return (cur, Outcome::Other("stops on a comment"));
        }
        if is_decl(&line) || is_structural(&line) {
            let _ = std::fs::remove_file(&tmp);
            return (
                cur,
                Outcome::BlockedByCode(n, line.trim().chars().take(56).collect()),
            );
        }
        cur[n - 1] = format!("// {line}");
        touched += 1;
    }
    let _ = std::fs::remove_file(&tmp);
    (cur, Outcome::Other("cap reached"))
}

fn line_of(text: &str) -> Option<usize> {
    let i = text.find("line ")?;
    let rest = &text[i + 5..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

pub fn run(cmd: &ProseCmd, root: PathBuf) -> Result<()> {
    let ProseCmd::Report { fix, all } = cmd;
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file());
    let Some(t27c) = t27c else {
        anyhow::bail!(
            "no compiler -- this command asks the compiler which line is prose, and\n  \
             its absence is not a clean bill.\n  cargo build --release -p t27c"
        );
    };

    // ONE shared scope, so this command and `tri unparsed` cannot disagree
    // about which specs a census may speak about. They did, and the gap was
    // exactly the two rules each sibling had to learn on its own: this one
    // reported "107 specs that do not parse" where `unparsed` reported 76 --
    // 21 fixtures broken ON PURPOSE, and 10 specs that parse and fail later.
    let scope = parse_failures(&root, &t27c);
    let (fixtures, other_stage) = (scope.fixtures, scope.other_stage());

    let mut prose: Vec<(PathBuf, usize, Vec<String>)> = Vec::new();
    let mut code: Vec<(PathBuf, usize, String)> = Vec::new();
    // Every `Outcome::Other` carries a hand-written reason, and the sole reader
    // bound it to a wildcard and counted. Eight distinct reasons collapsed into
    // one number, and two of them -- "unreadable" and "compiler did not run" --
    // say the INSTRUMENT failed, which a reader of "NOT DECIDED" cannot tell
    // apart from "cap reached".
    let mut other: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut scanned = 0usize;

    for (rel, _) in &scope.failures {
        let spec = &root.join(rel);
        scanned += 1;
        let (fixed, outcome) = walk(&t27c, &root, spec, 200);
        match outcome {
            Outcome::Prose(0) => *other.entry("no prose line to comment").or_default() += 1,
            Outcome::Prose(n) => prose.push((spec.clone(), n, fixed)),
            Outcome::BlockedByCode(n, l) => code.push((spec.clone(), n, l)),
            Outcome::Other(why) => *other.entry(why).or_default() += 1,
        }
    }

    let rel = |p: &Path| p.strip_prefix(&root).unwrap_or(p).display().to_string();

    println!("  specs refused at PARSE             {scanned}");
    if other_stage > 0 {
        println!("  ... refused at a LATER stage       {other_stage}  (they parse)");
    }
    if fixtures > 0 {
        println!("  broken ON PURPOSE under fixtures/  {fixtures}  (detector inputs, not debt)");
    }
    println!("  ... blocked ONLY by prose          {}", prose.len());
    println!("  ... blocked by code                {}", code.len());
    let other_total: usize = other.values().sum();
    if other_total > 0 {
        println!("  ... NOT DECIDED, nothing claimed   {other_total}");
        for (why, n) in &other {
            let tag = if BROKEN_INSTRUMENT.contains(why) {
                "  <-- the tool failed, not the spec"
            } else {
                ""
            };
            println!("        {n:>4}  {why}{tag}");
        }
    }

    if !prose.is_empty() {
        println!();
        println!("  prose-blocked, and the line count that would be commented");
        for (p, n, _) in prose.iter().take(20) {
            println!("      {n:>4}  {}", rel(p));
        }
        if prose.len() > 20 {
            println!("      ... and {} more", prose.len() - 20);
        }
    }

    if *all && !code.is_empty() {
        println!();
        println!("  blocked by code -- the compiler stops on something this may not touch");
        for (p, n, l) in code.iter().take(20) {
            println!("      {}:{n}  {l}", rel(p));
        }
        if code.len() > 20 {
            println!("      ... and {} more", code.len() - 20);
        }
    }

    if !*fix {
        println!();
        println!("  --fix comments those lines, preserving the text exactly. It is a");
        println!("  STATEMENT that the prose was documentation and not a declaration");
        println!("  someone half-wrote. Read the diff: every changed line must be the");
        println!("  old line with `// ` in front, and nothing else.");
        return Ok(());
    }

    let mut written = 0usize;
    for (p, _, fixed) in &prose {
        if std::fs::write(p, format!("{}\n", fixed.join("\n"))).is_ok() {
            written += 1;
        }
    }
    println!();
    println!("  files rewritten                    {written}");
    println!();
    println!("  Now re-seal them, or the seals still record that they do not generate:");
    println!("      t27c seal <spec> --save   &&   tri seals sync-twins");
    println!("      python3 tools/check_specs_generate.py --update-baseline");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sentence_with_punctuation_is_not_structural() {
        // The shape test that failed: ordinary English carries ( ) : = ->
        for s in [
            "Part of Phase 4: Quality & Performance (Issue #48)",
            "Tiny MLP: 100 -> 64 -> 10 (simplified MNIST-like).",
            "phi^2 + 1/phi^2 = 3 = TRINITY",
            "Zig-backed FFI bridge for Trinity VSA core.",
        ] {
            assert!(!is_structural(s), "{s}");
            assert!(!is_decl(s), "{s}");
        }
    }

    #[test]
    fn code_at_a_line_boundary_is_structural() {
        for s in [
            "test \"C-API: version\" {",
            "const ver = trinity_vsa_version();",
            "    field: u32,",
            "}",
        ] {
            assert!(is_structural(s) || is_decl(s), "{s}");
        }
    }

    // The refusal this errs toward: a wrapped sentence ending in a comma is
    // declined rather than edited. Costs a repair, never costs a line of code.
    #[test]
    fn a_wrapped_sentence_is_declined_not_edited() {
        assert!(is_structural(
            "File-based, read-only API surface that downstream tools (chip-repo CI,"
        ));
    }

    #[test]
    fn is_decl_matches_the_keyword_not_a_prefix() {
        assert!(is_decl("fn a() {"));
        assert!(is_decl("pub const X = 1;"));
        assert!(!is_decl("fnord is not a keyword"));
        assert!(!is_decl("constant folding is described here"));
    }

    #[test]
    fn the_reason_reaches_the_tally() {
        // Eight `Outcome::Other` sites each write a distinct hand-written
        // reason, and the sole reader bound it to a wildcard and counted.
        // Two of those reasons -- "unreadable" and "compiler did not run" --
        // say the TOOL failed, which a reader of "NOT DECIDED" could not tell
        // apart from "cap reached". The defect is a discarding pattern at the
        // call site, so that is what this reads.
        let src = include_str!("prose.rs");
        let prod = src.split(concat!("#[cfg(te", "st)]")).next().unwrap();
        // This slice stops at the FIRST test module. Measured with
        // `gates::test_module_lines` (a state machine, not a split): of the 46
        // files here carrying a test module, gates.rs alone holds 79 items
        // after its first. The FILE count varies with the matcher -- 13, 11 or
        // 9 depending on whether you split on the string, on the bare
        // attribute line, or additionally reject matches inside string
        // literals -- so it is not quoted here. An audit found the 10 and 130
        // I first wrote reproduce under none of those definitions.
        // THIS file has none, so the slice is whole today -- the anchor below
        // is defence, not a live repair. If a test module ever lands above the
        // subject, `contains` goes false and the negative assertion passes
        // because it is looking at nothing.
        assert!(
            prod.contains("Outcome::Other("),
            "the production slice no longer reaches the subject -- this test would pass vacuously"
        );
        assert!(
            !prod.contains(concat!("Outcome::Other(", "_)")),
            "the reason is discarded at the match arm again"
        );
        // Every reason named in BROKEN_INSTRUMENT must be a reason that is
        // actually constructed, or the annotation silently marks nothing.
        for why in super::BROKEN_INSTRUMENT {
            assert!(
                prod.contains(&format!("Outcome::Other(\"{why}\")")),
                "BROKEN_INSTRUMENT names `{why}`, which no site constructs"
            );
        }
    }

    #[test]
    fn line_of_reads_the_first_number_after_line() {
        assert_eq!(line_of("parse error near line 38: bad"), Some(38));
        assert_eq!(line_of("nothing here"), None);
    }
}
