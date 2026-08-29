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
    /// Cross-check the written classification against what the tree says today.
    ///
    /// `docs/TYPE_CONFLICTS.md` splits every conflicted name into DRIFT (one
    /// concept, two definitions) and DISTINCT (two concepts, one name). That
    /// split is a READING, taken on a day, and readings go stale: a name gets
    /// converged, a name gets added, a definition moves. This reports both
    /// directions of the drift so the document cannot quietly describe a tree
    /// that no longer exists.
    Classified,
    /// Names defined more than once INSIDE ONE FILE.
    ///
    /// `dup` asks whether two files claim one name; this asks whether one file
    /// defines a name twice. They are different defects: the first is a naming
    /// collision, the second is a copy whose halves have drifted apart. Fails
    /// when two copies of one name state different values.
    Redef,
}

/// Where the conflicted set is pinned.
const LEDGER: &str = "docs/reports/type_conflicts.json";

/// Where each conflicted name's verdict and the reading behind it are written.
const CLASSIFICATION: &str = "docs/reports/type_conflicts_classified.json";

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

#[derive(serde::Deserialize)]
struct ClassifiedName {
    name: String,
    verdict: String,
}

#[derive(serde::Deserialize)]
struct Classification {
    names: Vec<ClassifiedName>,
}

/// Report the classification against a live reading. Non-empty drift in either
/// direction exits non-zero: a stale row and an unjudged conflict are both a
/// document making a claim the tree does not support.
fn classified(root: &std::path::Path, observed: &[String]) -> Result<()> {
    let path = root.join(CLASSIFICATION);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("{} is missing -- see docs/TYPE_CONFLICTS.md", path.display()))?;
    let c: Classification = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not readable as a classification", path.display()))?;

    let names: Vec<String> = c.names.iter().map(|n| n.name.clone()).collect();
    // `drift` is the same set difference in both directions the ratchet uses;
    // one implementation, so the two commands cannot disagree about what a
    // difference is.
    let (unjudged, stale) = drift(&names, observed);

    let d = c.names.iter().filter(|n| n.verdict == "DRIFT").count();
    let x = c.names.iter().filter(|n| n.verdict == "DISTINCT").count();
    println!("  classification: {} name(s) -- {d} DRIFT, {x} DISTINCT", names.len());
    println!("  tree today:     {} conflicted name(s)", observed.len());

    for n in &stale {
        println!("  STALE     {n}: classified, but no longer conflicting -- drop the row");
    }
    for n in &unjudged {
        println!("  UNJUDGED  {n}: conflicting, but nothing has read it");
    }

    if stale.is_empty() && unjudged.is_empty() {
        println!("\n  OK: every conflicted name in the tree has a written verdict, and every");
        println!("  written verdict is about a name that is still conflicting.");
        return Ok(());
    }
    anyhow::bail!(
        "{} stale row(s) and {} unjudged conflict(s). Re-read them and update {}.",
        stale.len(),
        unjudged.len(),
        CLASSIFICATION
    )
}

// ---------------------------------------------------------------------------
// redef: one name defined twice IN THE SAME FILE
// ---------------------------------------------------------------------------

/// One top-level definition in one file: where it starts, and its body.
#[derive(Clone)]
pub struct Redef {
    pub line: usize,
    pub body: String,
}

/// Strip what must not be counted when tracking bracket depth: line comments
/// and string literals. A `{` inside `"a { b"` is not a block.
fn depth_relevant(line: &str) -> String {
    let mut out = String::new();
    let mut chars = line.chars().peekable();
    let mut in_str: Option<char> = None;
    while let Some(c) = chars.next() {
        match in_str {
            Some(q) => {
                if c == '\\' {
                    chars.next();
                } else if c == q {
                    in_str = None;
                }
            }
            None => {
                if c == '/' && chars.peek() == Some(&'/') {
                    break;
                } else if c == '"' || c == '\'' {
                    in_str = Some(c);
                } else {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// The name a top-level definition introduces, if the line starts one.
fn def_name(line: &str) -> Option<&str> {
    let t = line.trim_start();
    let t = t.strip_prefix("pub ").unwrap_or(t);
    let rest = ["const ", "fn ", "type ", "struct ", "enum "]
        .iter()
        .find_map(|kw| t.strip_prefix(kw))?;
    let name: &str = rest
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// The kinds that are always a module member, never a binding inside a body.
///
/// `const` is absent deliberately. See `redefs_in`.
const MEMBER_KINDS: &[&str] = &["fn ", "struct ", "enum ", "type "];

/// Does this file write `module Name { ... }` rather than `module Name;`?
///
/// 231 specs of 650 use the braced form, 392 the statement form and 27 have no
/// module line at all. The two forms put their contents at different bracket
/// depths, so the answer decides where "top level" is.
fn has_braced_module(lines: &[&str]) -> bool {
    lines.iter().any(|l| {
        let t = l.trim_start();
        t.strip_prefix("module ")
            .is_some_and(|r| r.trim_end().ends_with('{'))
    })
}

/// Every top-level definition in one source, grouped by name.
///
/// TOP LEVEL IS BRACKET DEPTH ZERO -- OR ONE, INSIDE A BRACED MODULE.
///
/// Four rulers were tried and three were wrong, each differently:
///
/// - A fixed four spaces reports 43 files of 650. 41 are local `const`
///   bindings, because `specs/numeric/gf16.t27` defines at column 0 with
///   bodies at four while `specs/ml/optimizer/adamw.t27` indents everything
///   under `module AdamW;`.
/// - Bracket depth zero reports 2, both real -- and is blind to the 231 specs
///   written as `module Name { ... }`, where every definition sits at depth 1.
/// - The smallest indent any definition is written at handles the braced form
///   and puts the locals straight back: `specs/api/c_api_contract.t27` has no
///   definition outside its test blocks, so the smallest indent IS the locals'
///   indent, and it reports `a`, `b`, `v`, `sim`.
///
/// What works is depth zero, or depth one when the file opens a braced module,
/// with `const` accepted only in the first case. A `const` at braced-module
/// depth may be a binding in a body whose braces this line-wise counter
/// mistracked; a `fn`, `struct`, `enum` or `type` there is a member. The
/// corpus has 2616 such members, so the rule is exercised broadly, and it
/// reports THREE files with no false positive: the two the depth rule already
/// found, plus `specs/file/operations.t27`, which declares `fn delete` twice
/// with different arities.
///
/// KNOWN LIMIT, not a bug to be surprised by later: a duplicated top-level
/// `const` inside a braced module is NOT reported. Accepting it costs sixteen
/// files of false positives, so the miss is the cheaper error -- but it is a
/// miss, and the honest fix is to ask the parser rather than to count braces
/// in a fifth way.
pub fn redefs_in(src: &str) -> BTreeMap<String, Vec<Redef>> {
    let lines: Vec<&str> = src.split('\n').collect();
    let base: i32 = if has_braced_module(&lines) { 1 } else { 0 };
    let mut starts: Vec<(usize, String)> = Vec::new();
    let mut depth: i32 = 0;
    for (i, line) in lines.iter().enumerate() {
        if depth == base {
            if let Some(n) = def_name(line) {
                let t = line.trim_start();
                let t = t.strip_prefix("pub ").unwrap_or(t);
                if base == 0 || MEMBER_KINDS.iter().any(|k| t.starts_with(k)) {
                    starts.push((i, n.to_string()));
                }
            }
        }
        let c = depth_relevant(line);
        depth += c.matches(['{', '(', '[']).count() as i32;
        depth -= c.matches(['}', ')', ']']).count() as i32;
        if depth < 0 {
            depth = 0;
        }
    }
    let mut out: BTreeMap<String, Vec<Redef>> = BTreeMap::new();
    for (k, (i, name)) in starts.iter().enumerate() {
        let end = starts.get(k + 1).map(|(j, _)| *j).unwrap_or(lines.len());
        out.entry(name.clone()).or_default().push(Redef {
            line: i + 1,
            body: lines[*i..end].join("\n").trim_end().to_string(),
        });
    }
    out.retain(|_, v| v.len() > 1);
    out
}

/// The `name: value` pairs a definition states, in source order.
///
/// Compared instead of the raw body because a body comparison cannot tell a
/// changed SCORE from a changed comment, and a bare number scan cannot tell a
/// score from a type width: scanning every number in `adamw.t27` reported ten
/// names with drifting values, and all ten were `GF16` read as 16 and a year
/// read out of a comment. Comparing stated fields reports zero there, which is
/// the truth.
pub fn fields_of(body: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in body.split('\n') {
        let t = line.trim();
        if t.starts_with("//") {
            continue;
        }
        let Some((k, v)) = t.split_once(':') else {
            continue;
        };
        let k = k.trim();
        // Digits belong in a field name. Requiring letters only silently drops
        // `pass_at_1` and `pass_at_5` -- the score fields, which is to say every
        // field this check exists to compare. It reported zero numeric drift in
        // a file that has five.
        if k.is_empty()
            || !k
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            || !k.starts_with(|c: char| c.is_ascii_lowercase())
        {
            continue;
        }
        let v = v.trim().trim_end_matches(',').trim();
        let v = match v.split_once("//") {
            Some((head, _)) => head.trim(),
            None => v,
        };
        if !v.is_empty() {
            out.insert(k.to_string(), v.to_string());
        }
    }
    out
}

/// How two or more definitions of one name differ.
///
/// Split three ways because one word would hide the difference that matters.
/// In `specs/igla/coder/benchmark.t27` every one of thirteen names differs
/// somehow, but only five differ in a NUMBER -- the rest differ in a provenance
/// string that got more specific over time. Calling all thirteen the same thing
/// is what a coarser reading of this file did, and it makes a citation getting
/// better look like a score contradicting itself.
#[derive(PartialEq, Eq, Debug)]
pub enum Divergence {
    /// Byte-identical bodies: redundant, and nothing can be read two ways.
    Identical,
    /// A field two copies both state holds different NUMBERS. Whichever copy a
    /// consumer takes, it takes a different quantity. This is the one that fails.
    Numbers,
    /// A field one copy states and the other does not: the definitions are of
    /// different shapes, not of different values.
    Fields,
    /// A field two copies both state holds different text.
    Text,
    /// The declaration lines themselves differ: two functions of the same name
    /// with different parameter lists or return types. `fields_of` reads
    /// `name: value` lines and a signature is not one, so without this the two
    /// `fn delete` in `specs/file/operations.t27` -- one taking three
    /// arguments, one taking one -- classify as differing "only in prose".
    Signature,
    /// The stated fields agree; the bodies differ elsewhere (a comment).
    Prose,
}

fn is_number(v: &str) -> bool {
    let t = v.trim().trim_end_matches(&[',', ';'][..]).trim();
    !t.is_empty() && t.parse::<f64>().is_ok()
}

pub fn divergence(copies: &[Redef]) -> Divergence {
    if copies.iter().all(|c| c.body == copies[0].body) {
        return Divergence::Identical;
    }
    let sets: Vec<_> = copies.iter().map(|c| fields_of(&c.body)).collect();
    let mut keys: Vec<&String> = sets.iter().flat_map(|s| s.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut missing = false;
    let mut text = false;
    for k in keys {
        let vals: Vec<Option<&String>> = sets.iter().map(|s| s.get(k)).collect();
        if vals.iter().any(|v| v.is_none()) {
            if vals.iter().any(|v| v.is_some()) {
                missing = true;
            }
            continue;
        }
        let vs: Vec<&str> = vals.into_iter().map(|v| v.unwrap().as_str()).collect();
        if vs.iter().any(|v| *v != vs[0]) {
            if vs.iter().all(|v| is_number(v)) {
                return Divergence::Numbers;
            }
            text = true;
        }
    }
    if text {
        return Divergence::Text;
    }
    if missing {
        return Divergence::Fields;
    }
    let head = |b: &str| -> String {
        b.lines()
            .next()
            .unwrap_or("")
            .split("//")
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let first = head(&copies[0].body);
    if copies.iter().any(|c| head(&c.body) != first) {
        return Divergence::Signature;
    }
    Divergence::Prose
}

/// Report every name defined more than once inside one file.
fn redef(root: &std::path::Path) -> Result<()> {
    let specs = read_specs(root);
    if specs.is_empty() {
        anyhow::bail!("no specs under {}/specs -- nothing was read", root.display());
    }
    println!("REDEFINED IN ONE FILE -- read {} specs", specs.len());
    println!();
    let mut numbers = 0usize;
    let mut fields = 0usize;
    let mut text = 0usize;
    let mut signature = 0usize;
    let mut prose = 0usize;
    let mut identical = 0usize;
    for (f, src) in &specs {
        let dups = redefs_in(src);
        if dups.is_empty() {
            continue;
        }
        println!("{}  ({} name(s))", f, dups.len());
        for (name, copies) in &dups {
            let d = divergence(copies);
            let tag = match d {
                Divergence::Numbers => {
                    numbers += 1;
                    "NUMBERS "
                }
                Divergence::Fields => {
                    fields += 1;
                    "fields  "
                }
                Divergence::Text => {
                    text += 1;
                    "text    "
                }
                Divergence::Signature => {
                    signature += 1;
                    "SIGNATURE"
                }
                Divergence::Prose => {
                    prose += 1;
                    "prose   "
                }
                Divergence::Identical => {
                    identical += 1;
                    "identical"
                }
            };
            let at: Vec<String> = copies.iter().map(|c| c.line.to_string()).collect();
            println!("  {} x{}  {:<28} lines {}", tag, copies.len(), name, at.join(","));
            if d == Divergence::Signature {
                for c in copies.iter() {
                    println!("             {}", c.body.lines().next().unwrap_or("").trim());
                }
            }
            if d == Divergence::Numbers || d == Divergence::Fields {
                let sets: Vec<_> = copies.iter().map(|c| fields_of(&c.body)).collect();
                let mut keys: Vec<&String> = sets.iter().flat_map(|s| s.keys()).collect();
                keys.sort();
                keys.dedup();
                for k in keys {
                    let vals: Vec<String> = sets
                        .iter()
                        .map(|s| s.get(k).cloned().unwrap_or_else(|| "(absent)".into()))
                        .collect();
                    if vals.iter().any(|v| v != &vals[0]) {
                        println!("             {:<16} {}", k, vals.join("  |  "));
                    }
                }
            }
        }
        println!();
    }
    println!(
        "{} name(s) whose copies state different NUMBERS; {} differ in which fields\n         they state, {} in the text of a field, {} only in prose, {} identical.",
        numbers, fields, text, prose, identical
    );
    if numbers > 0 {
        println!();
        println!(
            "A name whose copies state different numbers has no single answer: the\n\
             consumer takes whichever copy the compiler kept, and `t27c parse` accepts\n\
             all of them with exit 0 and no diagnostic. Text and field drift are\n\
             reported, not failed -- a citation getting more specific is not a\n\
             contradiction."
        );
        std::process::exit(1);
    }
    Ok(())
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
        TypesCmd::Redef => return redef(&root),
        TypesCmd::Classified => {
            let specs = read_specs(&root);
            if specs.is_empty() {
                anyhow::bail!("no specs under {}/specs -- nothing was read", root.display());
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
            return classified(&root, &observed);
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

#[cfg(test)]
mod redef_tests {
    use super::*;

    #[test]
    fn a_local_binding_is_not_a_second_definition() {
        // Both `const sign` lines sit at four spaces inside a function body.
        // An indentation rule calls that a redefinition; bracket depth does not.
        // This is the difference between reading 43 files and reading 2.
        let src = "pub fn a(v: f32) GF16 {\n    const sign = 1;\n}\n\
                   pub fn b(v: f32) GF16 {\n    const sign = 2;\n}\n";
        let d = redefs_in(src);
        assert!(d.get("sign").is_none(), "a body binding is not top level");
        assert!(d.is_empty(), "two distinct functions redefine nothing");
    }

    #[test]
    fn a_name_repeated_at_depth_zero_is_found() {
        let src = "pub fn f() -> T {\n    return 1;\n}\n\npub fn f() -> T {\n    return 2;\n}\n";
        let d = redefs_in(src);
        assert_eq!(d.get("f").map(|v| v.len()), Some(2));
    }

    #[test]
    fn a_module_header_does_not_open_a_block() {
        // `module M;` is a statement, so what follows it is still depth zero
        // even though the corpus indents it. Without this the whole-module copy
        // in adamw.t27 is invisible.
        let src = "module M;\n\n    const A = 1;\n\n    const A = 1;\n";
        assert_eq!(redefs_in(src).get("A").map(|v| v.len()), Some(2));
    }

    #[test]
    fn a_field_name_may_contain_a_digit() {
        // The regression this test exists for: a key filter of lowercase-only
        // dropped `pass_at_1` and `pass_at_5`, so a file with five score
        // disagreements reported zero. Letters-only is not a field name rule.
        let f = fields_of("x {\n    pass_at_1: 0.487,\n    pass_at_5: 0.525,\n}");
        assert_eq!(f.get("pass_at_1").map(String::as_str), Some("0.487"));
        assert_eq!(f.get("pass_at_5").map(String::as_str), Some("0.525"));
    }

    fn copies(bodies: &[&str]) -> Vec<Redef> {
        bodies
            .iter()
            .enumerate()
            .map(|(i, b)| Redef { line: i + 1, body: (*b).to_string() })
            .collect()
    }

    #[test]
    fn a_differing_number_outranks_a_differing_string() {
        // Both fields differ. The verdict must be Numbers: a score that
        // contradicts itself is the defect, and a citation that got more
        // specific must not be able to mask it.
        let d = divergence(&copies(&[
            "f {\n    pass_at_1: 0.487,\n    benchmark: \"VerilogEval\",\n}",
            "f {\n    pass_at_1: 0.0,\n    benchmark: \"VerilogEval (arXiv:1)\",\n}",
        ]));
        assert_eq!(d, Divergence::Numbers);
    }

    #[test]
    fn a_citation_getting_more_specific_is_text_not_numbers() {
        let d = divergence(&copies(&[
            "f {\n    pass_at_1: 0.857,\n    benchmark: \"VerilogEval\",\n}",
            "f {\n    pass_at_1: 0.857,\n    benchmark: \"VerilogEval (arXiv:1)\",\n}",
        ]));
        assert_eq!(d, Divergence::Text);
    }

    #[test]
    fn a_field_present_in_one_copy_only_is_shape_not_value() {
        // adamw.t27: one copy has `use_phi_betas`, the other `phi_variant`.
        // Two shapes of one config, not two answers to one question.
        let d = divergence(&copies(&[
            "S = struct {\n    use_phi_betas: bool,\n}",
            "S = struct {\n    phi_variant: PhiVariant,\n}",
        ]));
        assert_eq!(d, Divergence::Fields);
    }

    #[test]
    fn identical_copies_are_named_as_such() {
        let d = divergence(&copies(&["f {\n    a: 1,\n}", "f {\n    a: 1,\n}"]));
        assert_eq!(d, Divergence::Identical);
    }

    #[test]
    fn a_brace_inside_a_string_does_not_open_a_block() {
        let src = "const A = \"{\";\nconst A = \"{\";\n";
        assert_eq!(redefs_in(src).get("A").map(|v| v.len()), Some(2));
    }

    #[test]
    fn a_braced_module_puts_its_members_one_level_down() {
        // 231 of 650 specs are written this way. A depth-zero rule finds no
        // definition at all in any of them, which is how `fn delete` declared
        // twice in specs/file/operations.t27 stayed invisible.
        let src = "module M {\n    fn f() {\n        return 1;\n    }\n\n    fn f(x: u8) {\n        return x;\n    }\n}\n";
        assert_eq!(redefs_in(src).get("f").map(|v| v.len()), Some(2));
    }

    #[test]
    fn a_const_inside_a_braced_module_is_a_documented_miss() {
        // Pins the LIMIT, not a success. A `const` at braced-module depth is
        // not treated as a member, so this genuine duplicate is NOT reported.
        // Accepting it costs sixteen files of local bindings that a line-wise
        // brace counter cannot tell apart from members.
        //
        // The fixture is at module depth on purpose: a `const` inside a test
        // block is excluded by depth alone, so a test written that way passes
        // whether or not the kind filter exists and proves nothing.
        let src = "module M {\n    const v = 1;\n\n    const v = 2;\n}\n";
        assert!(
            redefs_in(src).get("v").is_none(),
            "if this now reports, the kind filter changed and the sixteen \
             false-positive files must be re-measured"
        );
    }

    #[test]
    fn a_const_at_column_zero_is_still_a_definition() {
        // The exclusion is scoped to the braced form. adamw.t27 duplicates 18
        // names under `module AdamW;`, most of them consts, and must keep
        // being reported.
        let src = "module M;\n\nconst A = 1;\n\nconst A = 1;\n";
        assert_eq!(redefs_in(src).get("A").map(|v| v.len()), Some(2));
    }
}
