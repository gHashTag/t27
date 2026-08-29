//! Specs the compiler cannot read, ranked by the CONSTRUCT that stops it.
//!
//! WHY THIS EXISTS
//! ---------------
//! The obvious census groups by the compiler's message. That census is wrong,
//! and it was shipped once: `import x`, `algorithm y {`, `type T = T`,
//! `impl X {` and an English sentence all print
//! "unexpected token after expression statement: Ident". The message names the
//! state the parser recovered INTO, not what it choked on, so grouping by it
//! reported five different defects as one 23-strong "parser gap".
//!
//! Grouping by what the line CONTAINS gives a work queue instead: on the day
//! this was written the top rows were path-qualified module names (9) and
//! body-less function prototypes (9), and the first of those was one grammar
//! change worth six specs.
//!
//! WHAT IT ABSTAINS ON
//! -------------------
//! When the failing line begins with a construct the top level ACCEPTS -- `fn`,
//! `pub`, `struct`, `const` -- the line is a symptom and the cause is upstream
//! of it. Roughly two thirds of the failures land there. Naming a construct in
//! that case would be inventing one, so the row says so and stops.
use anyhow::Result;
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum UnparsedCmd {
    /// Rank the constructs that stop the compiler, most specs first.
    Report {
        /// Name the specs under each construct.
        #[arg(long)]
        list: bool,
    },
}

/// (name, matcher) in priority order -- first match wins.
///
/// Ordered so that the more specific shape is tested first: a body-less `fn`
/// prototype must be recognised before the accepted-keyword abstention, or it
/// disappears into "cause is upstream" and the largest actionable row with it.
fn classify(line: &str) -> Option<&'static str> {
    let t = line.trim();
    let head = t.strip_prefix("pub ").unwrap_or(t);

    // `fn f(a: T) -> U;` -- a signature with no body.
    if head.starts_with("fn ") && t.ends_with(';') {
        return Some("fn NAME(..) -> T;   body-less prototype");
    }
    // `struct Id(str);` -- a tuple/newtype struct.
    if head.starts_with("struct ") && t.ends_with(");") {
        return Some("struct NAME(T);     tuple / newtype struct");
    }
    // `module a::b { }` / `module a::b;`
    if head.starts_with("module ") && head.contains("::") {
        return Some("module a::b         path-qualified module name");
    }
    if head.starts_with("import ") {
        return Some("import ..           import statement");
    }
    if head.starts_with("use ") {
        return Some("use ..              use declaration");
    }
    if head.starts_with("trait ") {
        return Some("trait NAME          trait declaration");
    }
    if head.starts_with("impl ") {
        return Some("impl NAME           impl block");
    }
    if head.starts_with("algorithm ") {
        return Some("algorithm NAME {    algorithm block");
    }
    if head.starts_with("type ") {
        return Some("type T = U          type alias");
    }
    if t.starts_with("\\\\") {
        return Some("\\\\ ...              Zig multiline string block");
    }
    // A macro call: an identifier immediately followed by `!(`.
    if let Some(i) = t.find("!(") {
        if t[..i]
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .count()
            > 0
        {
            return Some("name!(..)           Rust-style macro invocation");
        }
    }
    None
}

/// Constructs the parser accepts -- at the top level or inside a body. A
/// failing line that starts with one of these is a symptom; the cause is
/// earlier in the file.
///
/// The statement keywords were missing at first and 15 failures fell into
/// "not decided" that were plainly upstream: `return`, `let`, `}`.
const ACCEPTED: [&str; 17] = [
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
    "for",
    "while",
    "if",
    "else",
    "switch",
    "try",
];

/// A file under `fixtures/` is BROKEN ON PURPOSE -- it is the reference input
/// for a detector, not debt. `tools/specs_generate_baseline.txt` already omits
/// all 21 of them; a census that counts them disagrees with the repository's
/// own ledger. They are printed on their own line rather than dropped, because
/// a number that silently excludes something is the defect this file exists to
/// avoid.
fn is_fixture(path: &str) -> bool {
    path.contains("/fixtures/")
}

fn accepted_head(line: &str) -> bool {
    let t = line.trim();
    let t = t.strip_prefix("pub ").unwrap_or(t);
    ACCEPTED.iter().any(|k| {
        t.strip_prefix(k)
            .is_some_and(|r| r.starts_with(|c: char| c.is_whitespace() || c == '('))
    })
}

fn line_of(text: &str) -> Option<usize> {
    let i = text.find("line ")?;
    let rest = &text[i + 5..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

pub fn run(cmd: &UnparsedCmd, root: PathBuf) -> Result<()> {
    let UnparsedCmd::Report { list } = cmd;
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file());
    let Some(t27c) = t27c else {
        anyhow::bail!(
            "no compiler -- the census asks it which line stops it, and its\n  \
             absence is not a clean bill.\n  cargo build --release -p t27c"
        );
    };

    let out = std::process::Command::new("git")
        .args(["ls-files", "*.t27"])
        .current_dir(&root)
        .output()?;
    let specs: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| root.join(s).is_file())
        .collect();

    let mut by: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    let mut upstream = 0usize;
    let mut unnamed: Vec<(String, String)> = Vec::new();
    let mut failing = 0usize;
    let mut fixtures = 0usize;
    // The rows that fall out of every bucket. Counted, because 36 + 27 + 30
    // came to 93 against a total of 97 and the four were leaving through a
    // bare `continue`.
    let mut unlocated = 0usize;

    for spec in &specs {
        let o = std::process::Command::new(&t27c)
            .arg("check")
            .arg(spec)
            .current_dir(&root)
            .output();
        let Ok(o) = o else { continue };
        if o.status.success() {
            continue;
        }
        if is_fixture(spec) {
            fixtures += 1;
            continue;
        }
        failing += 1;
        let text = String::from_utf8_lossy(&o.stderr) + String::from_utf8_lossy(&o.stdout);
        let Some(n) = line_of(&text) else {
            unlocated += 1;
            continue;
        };
        let Ok(src) = std::fs::read_to_string(root.join(spec)) else {
            unlocated += 1;
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        if n == 0 || n > lines.len() {
            unlocated += 1;
            continue;
        }
        let line = lines[n - 1];
        match classify(line) {
            Some(k) => by.entry(k).or_default().push(spec.clone()),
            None if accepted_head(line) => upstream += 1,
            None => unnamed.push((spec.clone(), line.trim().chars().take(52).collect())),
        }
    }

    let named: usize = by.values().map(|v| v.len()).sum();
    println!("  specs tracked                       {}", specs.len());
    println!("  ... the compiler cannot read        {failing}");
    println!("  ... construct NAMED on that line    {named}");
    println!("  ... cause is UPSTREAM, not named    {upstream}");
    println!("  ... not decided, nothing claimed    {}", unnamed.len());
    if unlocated > 0 {
        println!("  ... error names no readable line     {unlocated}");
    }
    if fixtures > 0 {
        println!("  broken ON PURPOSE under fixtures/    {fixtures}  (detector inputs, not debt)");
    }

    if by.is_empty() {
        println!();
        println!("  No failing line carries a construct this census recognises.");
        return Ok(());
    }

    let mut rows: Vec<(&&str, &Vec<String>)> = by.iter().collect();
    rows.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    println!();
    println!("  work queue -- one grammar change per row, largest first");
    for (k, v) in &rows {
        println!("      {:>4}  {k}", v.len());
        if *list {
            for s in v.iter() {
                println!("            {s}");
            }
        }
    }

    println!();
    println!("  The UPSTREAM count is not a residue to be reduced: those lines");
    println!("  are `fn`, `struct`, `const` -- constructs the parser accepts. The");
    println!("  defect is earlier in the file and this census will not guess it.");
    if !*list {
        println!();
        println!("  --list names the specs under each row.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prototype_is_named_before_the_abstention_swallows_it() {
        // `fn` is an ACCEPTED top-level keyword, so an ordering that tests the
        // abstention first loses the largest actionable row entirely.
        assert_eq!(
            classify("pub fn poll(x: [str]) -> Result;"),
            Some("fn NAME(..) -> T;   body-less prototype")
        );
        assert!(accepted_head("pub fn poll(x: [str]) -> Result;"));
    }

    #[test]
    fn a_function_with_a_body_is_not_a_prototype() {
        assert_eq!(classify("pub fn poll(x: u32) -> bool {"), None);
    }

    #[test]
    fn a_plain_module_is_not_a_path() {
        assert_eq!(classify("module Foo {"), None);
        assert_eq!(
            classify("module github::auth {"),
            Some("module a::b         path-qualified module name")
        );
    }

    #[test]
    fn a_sentence_containing_a_keyword_is_not_that_keyword() {
        // "importantly" starts with "import"; "typed" starts with "type".
        assert_eq!(classify("importantly, the bridge is read-only."), None);
        assert_eq!(classify("typed values flow through the VM."), None);
        assert!(!accepted_head("constant folding is described here"));
    }

    #[test]
    fn a_fixture_is_not_debt() {
        assert!(is_fixture(
            "bootstrap/tests/fixtures/damage/damage_class_01.t27"
        ));
        assert!(!is_fixture("specs/github/auth.t27"));
        // The word must be a PATH SEGMENT, not a substring of a file name.
        assert!(!is_fixture("specs/tools/fixtures_report.t27"));
    }

    #[test]
    fn a_statement_keyword_abstains_like_a_top_level_one() {
        for l in ["return x;", "let y = 1;", "    for (a) |b| {"] {
            assert_eq!(classify(l), None, "{l}");
            assert!(accepted_head(l), "{l}");
        }
    }

    #[test]
    fn a_macro_needs_a_name_in_front_of_the_bang() {
        assert_eq!(
            classify("assert_eq!(a, b);"),
            Some("name!(..)           Rust-style macro invocation")
        );
        assert_eq!(classify("if x != (a) {"), None);
    }
}
