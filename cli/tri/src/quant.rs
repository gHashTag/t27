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
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum QuantCmd {
    /// Every quantified clause, its binders, and the size of its domain.
    Report {
        /// Print one line per clause instead of the summary.
        #[arg(long)]
        full: bool,
        /// Domain sizes at or below this are called walkable. Choosing this
        /// number is the decision this report exists to inform. The default
        /// 65536 is 2^16, the 16-bit machine word; it admits 42 clauses that
        /// carry 99.4% of the walk cost. See the CEILING SWEEP section, which
        /// prints every ceiling that is not a synonym for another one.
        #[arg(long, default_value_t = 65536u128)]
        ceiling: u128,
    },
    /// Every line carrying a quantifier keyword, and whether the census read it.
    ///
    /// `report` counts what its own matcher built, so it cannot report what its
    /// matcher never saw. This asks the question the other way round with a
    /// DELIBERATELY LOOSER reader -- the bare letters, no boundary rules -- and
    /// prints every line the strict scanner did not turn into a clause.
    ///
    /// It fails only on a residue line where the keyword stands as its own WORD,
    /// because that is a quantifier the census owed an answer about. A residue
    /// line whose letters sit inside an identifier is printed as a count and
    /// forgiven.
    ///
    /// Written after `find("forall ")` -- a matcher keyed on the SPELLING, with a
    /// trailing space -- hid the one clause in the corpus that writes the keyword
    /// alone. The census said 922 where the corpus holds 923, and the bucket that
    /// clause belongs in, `no binder this can read`, was one short.
    Audit,
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
/// unbounded: picking one of them would change `|D|` by an unbounded factor
/// with nothing saying which was picked. The count is PRINTED by the report --
/// it used to be written here as 50, while the binary printed 80. A census
/// tool whose source misreports its own output is the broken ruler.
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
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut conflicted = std::collections::BTreeSet::new();
    for (p, src) in specs {
        for (name, d) in crate::types_dup::defs_in(&p.display().to_string(), src) {
            // W705: compare the SAME thing `tri types dup` compares -- name and
            // type per field, not types alone. Comparing types only made two
            // definitions with the same shapes and different field names agree,
            // and the two commands then printed 77 and 79 for one question.
            let key: Vec<String> = d.fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
            let fs: Vec<String> = d.fields.iter().map(|(_, t)| t.clone()).collect();
            if let Some(prev) = keys.get(&name) {
                if *prev != key {
                    conflicted.insert(name.clone());
                }
            }
            keys.insert(name.clone(), key);
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

/// Blank out `//` tails and string literals, keeping every column in place.
///
/// W705: without this the scanner matched `for all` inside prose. An independent
/// re-count found that 99 of the 135 rows it filed as suffix notations were
/// comments (88), string literals (5), markdown prose (3) and a block comment --
/// so the published suffix count was two thirds English.
///
/// A third instrument in this same repository already disagreed:
/// `t27c parse-complete --fallbacks` reports the suffix shape at 38 events, and
/// `docs/DISCARD_WHAT_IS_LEFT.md` prints 35 + 4. Nobody reconciled 38 with 135,
/// and the census was published anyway.
fn mask(line: &str) -> String {
    let b: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(b.len());
    let mut i = 0usize;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i];
        if in_str {
            out.push(if c == '"' { '"' } else { ' ' });
            if c == '"' && (i == 0 || b[i - 1] != '\\') {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push('"');
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            // Everything from here is a comment; keep the columns.
            for _ in i..b.len() {
                out.push(' ');
            }
            break;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The binder forms the corpus actually writes, all found by reading it.
///
/// The first version read exactly one -- `name : Type` -- and filed everything
/// else as "no binder this can read", a phrase that described the scanner and
/// was printed as though it described the source. The domains it lost that way
/// are the SMALLEST in the corpus: `for all Trit a, b` is nine values.
/// Split on commas at bracket depth zero. A type may contain a comma inside
/// `<>`/`()`/`[]`; a binder list separator never can.
fn split_top(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut cur = String::new();
    for ch in text.chars() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if ch == ',' && depth == 0 {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    out.push(cur);
    out.into_iter()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

/// Cut at a `{` that OPENS A SEGMENT -- the body of a clause written
/// `forall bits: u8, { … }` is not part of the binder list.
///
/// The "opens a segment" qualifier is load-bearing. `for any a, b in {1, -1}`
/// also has a top-level brace, and it is the DOMAIN. Cutting there is what made
/// the old reader return no binder for that clause -- while the module's own
/// doc comment used it as the example of a binder it could read.
fn before_body(text: &str) -> &str {
    let mut depth = 0usize;
    let mut fresh = true; // nothing but space since the last top-level comma
    for (i, ch) in text.char_indices() {
        match ch {
            '{' if depth == 0 && fresh => return &text[..i],
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if ch == ',' && depth == 0 {
            fresh = true;
        } else if !ch.is_whitespace() {
            fresh = false;
        }
    }
    text
}

fn is_ident(x: &str) -> bool {
    !x.is_empty()
        && x.chars()
            .next()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            == Some(true)
        && x.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Is this the text of a TYPE, or of a predicate that happens to follow a colon?
///
/// `forall F, D : NMSE(F, D) >= 0` writes `:` to mean "such that". Reading it as
/// an ascription is how the old reader printed a binder named `D` of type `F,` --
/// a name and a type neither of which appears in the source.
fn is_type_token(t: &str) -> bool {
    let t = t.trim();
    if t.is_empty() {
        return false;
    }
    // A REJECTOR, not an acceptor. The corpus writes types this reader has
    // never seen -- `[u32]`, `[]const u8`, `m.assigns` -- and an acceptor
    // written from the types it HAS seen throws those away, which is a louder
    // lie than the one it was built to fix. So: reject only what cannot be a
    // type, and let anything else through to `size_of`, whose job is to say
    // "unbounded" when it does not recognise something.
    let first = t.split_whitespace().next().unwrap_or("");
    if matches!(first, "if" | "let" | "match" | "then" | "else" | "return") {
        return false;
    }
    // A call or a comparison is a predicate: `NMSE(F, D) >= 0` after a colon
    // means "such that", not "has type".
    if t.contains('(') || t.contains(')') {
        return false;
    }
    ![
        "==", "!=", ">=", "<=", "&&", "||", " > ", " < ", " + ", " = ",
    ]
    .iter()
    .any(|op| t.contains(op))
}

/// The type a declaration segment ascribes, or `None` if what follows the colon
/// is not a type at all.
fn ascribed_type(rest: &str) -> Option<String> {
    let rest = rest.split(" where ").next().unwrap_or("").trim();
    // A spaced bracket type keeps its words; anything else ends at the space.
    let ty = if rest.starts_with('[') {
        rest.to_string()
    } else {
        rest.split_whitespace().next().unwrap_or("").to_string()
    };
    let ty = ty.trim_end_matches(',').trim().to_string();
    if ty.is_empty() || !is_type_token(&ty) {
        return None;
    }
    Some(ty)
}

/// The binders a clause declares -- ALL of them.
///
/// W710: this used to be `upto.split_once(':')`, one split, so the first colon
/// ended the world. That is right for the 501 single-binder rows and, by pure
/// accident, for the 69 rows that put the clause BODY after a comma -- and
/// wrong for the 297 rows that write a colon per binder. A clause binding four
/// variables was sized as though it bound one, and since a missing binder can
/// only make a domain smaller, the report systematically over-promised what
/// could be checked by brute force.
///
/// The rule now: split into top-level comma segments, walk them left to right
/// accumulating binders, and STOP at the first segment that is not a binder.
/// The stop is the whole correctness argument -- it is the only thing that
/// distinguishes `p : T, q : U` (two binders) from `p : T, body(p) >= 0` (one).
fn parse_binders(text: &str) -> Vec<(String, String)> {
    let t = before_body(text.trim());
    let mut out: Vec<(String, String)> = Vec::new();

    // `Type name[, name]` -- `for all Trit a, b`, `for all i8 x`.
    if let Some((head, tail)) = t.split_once(char::is_whitespace) {
        let head = head.trim();
        // `is_ident` matters: without it `F,` in `forall F, D : …` reads as a
        // type name, and the reader invents a binder out of a predicate.
        let looks_type = is_ident(head)
            && (primitive(head).is_some() || head.chars().next().unwrap().is_ascii_uppercase());
        if looks_type {
            let names: Vec<&str> = tail
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|x| x.trim())
                .filter(|x| !x.is_empty())
                .take_while(|x| is_ident(x))
                .collect();
            if !names.is_empty() {
                for n in names {
                    out.push((n.to_string(), head.to_string()));
                }
                return out;
            }
        }
    }

    // The segment walk. `pending` holds names still waiting for the type that a
    // later segment will give them: `a, b : Trit` arrives as two segments.
    let mut pending: Vec<String> = Vec::new();
    for seg in split_top(t) {
        // A second quantifier keyword inside the binder text -- `scan_clauses`
        // finds only the first `forall `, so the rest arrives here.
        let seg = seg
            .strip_prefix("forall ")
            .or_else(|| seg.strip_prefix("for all "))
            .or_else(|| seg.strip_prefix("for any "))
            .unwrap_or(&seg)
            .trim()
            .to_string();

        // `name[, name] in <domain>` -- a domain that is a collection or a
        // range is recorded with its text so `size_of` calls it unbounded,
        // rather than the scanner pretending there is no binder.
        if let Some((names, dom)) = seg.split_once(" in ") {
            let dom = dom
                .split(" where ")
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches(':')
                .trim()
                .to_string();
            let ns: Vec<&str> = names.split_whitespace().collect();
            if !dom.is_empty() && !ns.is_empty() && ns.iter().all(|n| is_ident(n)) {
                for n in pending.drain(..).chain(ns.iter().map(|s| s.to_string())) {
                    out.push((n, dom.clone()));
                }
                continue;
            }
            break;
        }

        // `name[ name]* : Type` -- names may also have arrived comma-grouped in
        // earlier segments, which is why `pending` is drained here.
        if let Some((names, rest)) = seg.split_once(':') {
            // `gf16::GF16` is one token, not a name and an ascription.
            if rest.starts_with(':') {
                if is_ident(&seg) {
                    pending.push(seg);
                    continue;
                }
                break;
            }
            let ns: Vec<&str> = names.split_whitespace().collect();
            match (ascribed_type(rest), ns.iter().all(|n| is_ident(n))) {
                (Some(ty), true) if !ns.is_empty() => {
                    for n in pending.drain(..).chain(ns.iter().map(|s| s.to_string())) {
                        out.push((n, ty.clone()));
                    }
                    continue;
                }
                // The colon meant "such that", or the names are not names.
                _ => break,
            }
        }

        // A bare name waits for the type a later segment will name.
        if is_ident(&seg) {
            pending.push(seg);
            continue;
        }

        // Not a binder. Everything after it is the clause body.
        break;
    }

    out
}

/// `kw` as a WORD, not as a prefix that happens to be followed by a space.
///
/// `find("forall ")` reads the spelling and decides a question about the KIND
/// of the line. `specs/igla/coder/benchmark.t27:3827` writes the keyword alone
/// -- `invariant …: forall` with no binders and the predicate below it -- so
/// it matched nothing and never became a clause at all. The census has a
/// bucket for exactly that shape, `no binder this can read`, and the clause
/// could not reach it.
///
/// Measured before changing it: of 883 `forall` lines in the corpus, ZERO have
/// an identifier character before the keyword, so tightening the left edge
/// cannot drop a clause this corpus already counts. The change is additive.
fn find_keyword(t: &str, kw: &str) -> Option<usize> {
    let b = t.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut from = 0usize;
    while let Some(rel) = t[from..].find(kw) {
        let i = from + rel;
        let j = i + kw.len();
        if (i == 0 || !ident(b[i - 1])) && (j >= b.len() || !ident(b[j])) {
            return Some(i);
        }
        from = j;
    }
    None
}

/// The notations, recognised on the MASKED source line.
fn scan_clauses(specs: &[(PathBuf, String)]) -> Vec<Clause> {
    let mut out = Vec::new();
    for (p, src) in specs {
        for (i, raw) in src.lines().enumerate() {
            let masked = mask(raw);
            let t = masked.trim();
            if t.is_empty() {
                continue;
            }
            let (notation, binder_text) = if let Some(idx) = find_keyword(t, "forall") {
                ("forall", t[idx + 6..].trim_start().to_string())
            } else if let Some(idx) = t.find(" for all ") {
                ("suffix-all", t[idx + 9..].to_string())
            } else if let Some(idx) = t.find(" for any ") {
                ("suffix-any", t[idx + 9..].to_string())
            } else if let Some(idx) = t.find(" for positive ") {
                ("suffix-positive", t[idx + 14..].to_string())
            } else {
                continue;
            };
            out.push(Clause {
                file: p.display().to_string(),
                line: i + 1,
                notation,
                binders: parse_binders(&binder_text),
                text: raw.trim().to_string(),
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

/// A call the corpus writes, and how many arguments it passes.
///
/// `args` is `None` when the parenthesis does not close inside the clause
/// window -- the reader ABSTAINS there rather than scoring the call short.
#[derive(Debug, PartialEq, Eq)]
pub struct Call {
    pub name: String,
    pub args: Option<usize>,
    /// A `.` immediately left of the name: `x.len()`. The receiver IS an
    /// argument, so 0-vs-1 is a convention, not a mismatch.
    pub method: bool,
}

/// Every call in a clause body, with its argument count.
///
/// The count is a paren-depth walk, not a comma split: 80 clause windows carry
/// a nested paren group, and `f(g(a, b), c)` has two arguments, not three.
pub fn calls(body: &str) -> Vec<Call> {
    const KW: [&str; 13] = [
        "if", "while", "for", "return", "forall", "assert", "let", "match", "else", "and", "or",
        "given", "then",
    ];
    let b: String = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let ch: Vec<char> = b.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < ch.len() {
        if !(ch[i].is_ascii_lowercase() || ch[i] == '_') {
            i += 1;
            continue;
        }
        let start = i;
        while i < ch.len() && (ch[i].is_ascii_alphanumeric() || ch[i] == '_') {
            i += 1;
        }
        let name: String = ch[start..i].iter().collect();
        let mut j = i;
        while j < ch.len() && ch[j] == ' ' {
            j += 1;
        }
        if j >= ch.len() || ch[j] != '(' || KW.contains(&name.as_str()) {
            continue;
        }
        let method = start > 0 && ch[start - 1] == '.';
        out.push(Call {
            name,
            args: count_args(&ch, j),
            method,
        });
        i = j + 1;
    }
    out
}

/// Arguments between `open` and its matching close, or `None` if it never
/// closes. An opening bracket counts as content: `f([])` passes ONE argument,
/// and a walk that only notices non-space characters scores it zero.
fn count_args(ch: &[char], open: usize) -> Option<usize> {
    let (mut depth, mut n, mut seen) = (0i32, 0usize, false);
    for (k, c) in ch.iter().enumerate().skip(open) {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                if depth > 1 {
                    seen = true;
                }
            }
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(if seen { n + 1 } else { 0 });
                }
                seen = true;
            }
            ',' if depth == 1 => n += 1,
            c if !c.is_whitespace() => seen = true,
            _ => {}
        }
        let _ = k;
    }
    None
}

/// Does this call pass a number of arguments no declaration in scope accepts?
///
/// Returns `(name, passed, declared)` per offending call. ABSTAINS -- returns
/// nothing for that call -- when the paren does not close, when the name is in
/// method position, when no declaration is visible, or when the visible scope
/// offers more than one arity. Four of the six rules are abstentions.
///
/// The compiler cannot answer this. `parse_invariant_clause` discards a
/// quantified clause on purpose (#2774), so the body produces no AST at all and
/// every AST-based check -- including `t27c check-calls`, which finds 95 of
/// these corpus-wide -- is blind to it by construction. Measured: 0 of the 20
/// clause-site candidates appear in `check-calls` output; 15 of 15 partner
/// sites outside clause bodies do.
pub fn arity_mismatches(scope: &Scope, file: &str, body: &str) -> Vec<(String, usize, usize)> {
    let empty = std::collections::BTreeSet::new();
    let vis = scope.visible.get(file).unwrap_or(&empty);
    let mut out = Vec::new();
    for c in calls(body) {
        if c.method {
            continue;
        }
        let Some(passed) = c.args else { continue };
        let mut arities = std::collections::BTreeSet::new();
        for f in vis {
            if let Some(a) = scope.arity.get(&(f.clone(), c.name.clone())) {
                arities.extend(a.iter().copied());
            }
        }
        // Exactly one declared arity, or the answer is a question about
        // overloading that t27 has not settled.
        if arities.len() != 1 {
            continue;
        }
        let want = *arities.iter().next().unwrap();
        if want != passed {
            out.push((c.name, passed, want));
        }
    }
    out
}

/// Is this clause VACUOUS -- true in every model, whatever the functions do?
///
/// The field's word, not one invented here: Beer, Ben-David, Eisner & Rodeh,
/// *Efficient Detection of Vacuity in Temporal Model Checking*. A property
/// passes vacuously when a subformula does not affect its truth; the degenerate
/// case that passes under every interpretation is a TAUTOLOGY, and a guard that
/// is never true is ANTECEDENT FAILURE.
///
/// A vacuous clause is not a wrong claim -- it IS true, which is the problem.
/// It passes every checker forever, it counts as coverage, and nothing will
/// ever flag it, because there is nothing to flag.
///
/// Decided from the clause TEXT alone. Nothing is evaluated, no type is
/// resolved, and every rule that cannot decide ABSTAINS rather than guessing.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Vacuity {
    /// The predicate never mentions the variable it quantifies over.
    BinderUnused(String),
    /// `A == A` for syntactically identical A, after `+ 0` is folded away.
    Reflexive(String),
    /// `P ==> P`.
    SelfImplication(String),
    /// `X != undefined` -- the compiler says in its own words that this
    /// "constrain[s] the value not at all ... which is trivially true".
    NotUndefined(String),
}

impl Vacuity {
    pub fn kind(&self) -> &'static str {
        match self {
            Vacuity::BinderUnused(_) => "binder never used in the predicate",
            Vacuity::Reflexive(_) => "A == A",
            Vacuity::SelfImplication(_) => "P ==> P",
            Vacuity::NotUndefined(_) => "X != undefined",
        }
    }
    /// The conjunct that decided it. Printing this, and not the whole body, is
    /// what lets a reader falsify the verdict in one glance.
    pub fn evidence(&self) -> &str {
        match self {
            Vacuity::BinderUnused(e)
            | Vacuity::Reflexive(e)
            | Vacuity::SelfImplication(e)
            | Vacuity::NotUndefined(e) => e,
        }
    }
}

/// Top-level conjuncts, split PER SOURCE LINE and then on `&&` / ` and ` at
/// paren depth zero.
///
/// Never flatten the body first. Measured on this corpus, a backreference regex
/// over the flattened text finds `depth == depth` inside
/// `int4_dequantize_bank(codes, depth, width).depth == depth` -- a real
/// preservation claim -- and `b == b` inside `a * b == b * a`, real
/// commutativity. Three false positives in eight hits, and one true positive
/// lost, is how a check gets classified as noise in its first pull request.
pub fn conjuncts(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.split("//").next().unwrap_or("");
        let (mut depth, mut cur) = (0i32, String::new());
        let ch: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < ch.len() {
            match ch[i] {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
            if depth == 0 && ch[i] == '&' && i + 1 < ch.len() && ch[i + 1] == '&' {
                out.push(std::mem::take(&mut cur));
                i += 2;
                continue;
            }
            if depth == 0 && ch[i] == ' ' && ch[i..].iter().collect::<String>().starts_with(" and ")
            {
                out.push(std::mem::take(&mut cur));
                i += 5;
                continue;
            }
            cur.push(ch[i]);
            i += 1;
        }
        out.push(cur);
    }
    out.into_iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

/// Fold the one arithmetic identity the corpus writes: `e + 0` is `e`.
fn fold(e: &str) -> String {
    e.trim()
        .trim_end_matches(|c: char| c == ';' || c == ',')
        .trim()
        .strip_suffix("+ 0")
        .map(|x| x.trim().to_string())
        .unwrap_or_else(|| e.trim().to_string())
}

/// Every reason this clause is vacuous, or an empty list.
///
/// `binders` are the declared names; `body` is the clause window.
pub fn vacuity(binders: &[String], body: &str) -> Vec<Vacuity> {
    let mut out = Vec::new();

    // The predicate is often on the SAME line as the binder list, so "does the
    // name appear below the head" is the wrong question. Count occurrences and
    // subtract the declarations. A line-based version emitted seven hits of
    // which four were same-line predicates.
    let decls: usize = binders
        .iter()
        .map(|b| {
            body.matches(&format!("{b} :")).count()
                + body.matches(&format!("{b}:")).count()
                + body.matches(&format!("{b} in ")).count()
        })
        .sum();
    let uses: usize = binders.iter().map(|b| count_word(body, b)).sum();
    if !binders.is_empty() && uses > 0 && uses <= decls {
        out.push(Vacuity::BinderUnused(
            body.lines().last().unwrap_or("").trim().to_string(),
        ));
    }

    for c in conjuncts(body) {
        if let Some((l, r)) = split_once_top(&c, "==>") {
            if l.trim() == r.trim() && !l.trim().is_empty() {
                out.push(Vacuity::SelfImplication(c.clone()));
                continue;
            }
        }
        if let Some((l, r)) = split_once_top(&c, "!=") {
            if r.trim().trim_end_matches(';') == "undefined" && !l.trim().is_empty() {
                out.push(Vacuity::NotUndefined(c.clone()));
                continue;
            }
        }
        if let Some((l, r)) = split_once_top(&c, "==") {
            let (l, r) = (fold(l), fold(r));
            if l == r && !l.is_empty() {
                out.push(Vacuity::Reflexive(c.clone()));
            }
        }
    }
    out
}

/// Count whole-word occurrences of `w`, so `a` does not match `arctan`.
fn count_word(hay: &str, w: &str) -> usize {
    let h: Vec<char> = hay.chars().collect();
    let n: Vec<char> = w.chars().collect();
    let mut k = 0;
    let mut i = 0;
    while i + n.len() <= h.len() {
        if h[i..i + n.len()] == n[..] {
            let before = i == 0 || !(h[i - 1].is_ascii_alphanumeric() || h[i - 1] == '_');
            let j = i + n.len();
            let after = j >= h.len() || !(h[j].is_ascii_alphanumeric() || h[j] == '_');
            if before && after {
                k += 1;
            }
        }
        i += 1;
    }
    k
}

/// Split on the first `op` that is at paren depth zero.
fn split_once_top<'a>(s: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    let ch: Vec<char> = s.chars().collect();
    let o: Vec<char> = op.chars().collect();
    let mut depth = 0i32;
    let mut i = 0;
    while i + o.len() <= ch.len() {
        match ch[i] {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 && ch[i..i + o.len()] == o[..] {
            // `==` must not match inside `==>` or `!=`.
            if op == "==" && (i + o.len() < ch.len() && ch[i + o.len()] == '>') {
                i += 1;
                continue;
            }
            if op == "==" && i > 0 && (ch[i - 1] == '!' || ch[i - 1] == '<' || ch[i - 1] == '>') {
                i += 1;
                continue;
            }
            let b = s.char_indices().nth(i).map(|(b, _)| b)?;
            let e = s
                .char_indices()
                .nth(i + o.len())
                .map(|(b, _)| b)
                .unwrap_or(s.len());
            return Some((&s[..b], &s[e..]));
        }
        i += 1;
    }
    None
}

/// Every `fn` name a spec file defines, and every module it `use`s.
///
/// The scope a clause's names resolve in is its OWN file plus the files its
/// `use` lines name. A name defined ten times corpus-wide but once inside that
/// scope is resolved; a name defined ten times and imported zero times is not.
pub struct Scope {
    /// name -> the spec files that define it
    pub defs: BTreeMap<String, std::collections::BTreeSet<String>>,
    /// (file, name) -> the parameter counts declared there
    pub arity: BTreeMap<(String, String), std::collections::BTreeSet<usize>>,
    /// spec file -> the spec files it can see (itself included)
    pub visible: BTreeMap<String, std::collections::BTreeSet<String>>,
}

pub fn scan_scope(specs: &[(PathBuf, String)]) -> Scope {
    let mut defs: BTreeMap<String, std::collections::BTreeSet<String>> = BTreeMap::new();
    let mut visible: BTreeMap<String, std::collections::BTreeSet<String>> = BTreeMap::new();
    let mut arity: BTreeMap<(String, String), std::collections::BTreeSet<usize>> = BTreeMap::new();
    let names: std::collections::BTreeSet<String> =
        specs.iter().map(|(p, _)| p.display().to_string()).collect();
    for (p, src) in specs {
        let file = p.display().to_string();
        let mut vis = std::collections::BTreeSet::new();
        vis.insert(file.clone());
        for line in src.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("use ") {
                // `use base::types;` names `specs/base/types.t27`.
                let m = rest.trim_end_matches(';').trim().replace("::", "/");
                let cand = format!("specs/{m}.t27");
                if let Some(hit) = names.iter().find(|n| n.ends_with(&cand)) {
                    vis.insert(hit.clone());
                }
            }
            let d = t.strip_prefix("pub ").unwrap_or(t);
            if let Some(rest) = d.strip_prefix("fn ") {
                if let Some(n) = rest.split('(').next() {
                    let n = n.trim();
                    if !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        defs.entry(n.to_string()).or_default().insert(file.clone());
                        // ABSTAIN on a declaration whose parameter list does
                        // not close on its own line. 17 declarations in this
                        // corpus wrap -- `fn cordic_top(` opens and the four
                        // parameters follow below -- and reading only the head
                        // line records arity 0 for them. That produced 19 of
                        // 31 rows in the first run of this column, every one a
                        // fabricated defect against a correct call.
                        if let Some((_, r)) = rest.split_once('(') {
                            if let Some(ps) = r.split(')').next() {
                                if r.contains(')') {
                                    let k = ps.split(',').filter(|x| !x.trim().is_empty()).count();
                                    arity
                                        .entry((file.clone(), n.to_string()))
                                        .or_default()
                                        .insert(k);
                                }
                            }
                        }
                    }
                }
            }
        }
        visible.insert(file, vis);
    }
    Scope {
        defs,
        arity,
        visible,
    }
}

/// What a clause's names resolve to, in its own file's scope.
///
/// Says NOTHING about whether the clause is true. It answers only the question
/// the census could not: is this clause even about functions that exist?
pub fn resolution(scope: &Scope, file: &str, body: &str) -> (Vec<String>, Vec<String>) {
    let empty = std::collections::BTreeSet::new();
    let vis = scope.visible.get(file).unwrap_or(&empty);
    let (mut undef, mut amb) = (Vec::new(), Vec::new());
    for n in called_names(body) {
        let seen: Vec<&String> = scope
            .defs
            .get(&n)
            .map(|d| d.intersection(vis).collect())
            .unwrap_or_default();
        match seen.len() {
            0 => undef.push(n),
            1 => {}
            _ => amb.push(n),
        }
    }
    (undef, amb)
}

/// Where a clause's body sits: from its own line down, while the indentation
/// holds and no new top-level construct starts.
///
/// A WINDOW, not a parse. The clause text the scanner stores is one line --
/// `forall i : u8` -- and the predicate lives below it. Twelve lines is the
/// bound; the window is stated rather than tuned, because a detector adjusted
/// until it hits its own motivating examples has stopped being evidence.
pub fn clause_body(lines: &[&str], line: usize) -> String {
    if line == 0 || line > lines.len() {
        return String::new();
    }
    let head = lines[line - 1];
    let ind = head.len() - head.trim_start().len();
    let mut out = vec![head.to_string()];
    for l in lines.iter().skip(line).take(11) {
        if l.trim().is_empty() || (l.len() - l.trim_start().len()) < ind || starts_construct(l) {
            break;
        }
        out.push((*l).to_string());
    }
    out.join("\n")
}

fn starts_construct(l: &str) -> bool {
    let t = l.trim_start().trim_start_matches("pub ");
    // W714: `bench ` and `given ` were missing, and the window is bounded by
    // indentation -- so an invariant written at indent 0 (gemm.t27:260) swallowed
    // the whole `bench booth_mul_latency` block that follows it. Measured: one
    // clause in 924 overruns, five have an indent-0 head. A list-shaped guard
    // goes stale by addition, which this repository has now seen three times.
    [
        "invariant ",
        "test ",
        "bench ",
        "given ",
        "fn ",
        "const ",
        "module ",
        "use ",
    ]
    .iter()
    .any(|k| t.starts_with(k))
}

/// Every name called in a clause body, minus the keywords that look like calls.
pub fn called_names(body: &str) -> std::collections::BTreeSet<String> {
    const KW: [&str; 13] = [
        "if", "while", "for", "return", "forall", "assert", "let", "match", "else", "and", "or",
        "given", "then",
    ];
    let mut out = std::collections::BTreeSet::new();
    let b: String = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let bytes: Vec<char> = b.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_lowercase() || bytes[i] == '_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == '_') {
                i += 1;
            }
            let mut j = i;
            while j < bytes.len() && bytes[j] == ' ' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == '(' {
                let n: String = bytes[start..i].iter().collect();
                if !KW.contains(&n.as_str()) {
                    out.insert(n);
                }
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Every ceiling that is not a synonym for another one, with what it admits.
///
/// A ceiling only matters where a domain size sits: raising it anywhere between
/// two adjacent sizes changes nothing. So a sweep that samples round numbers
/// reports flat regions it never measured -- the plateau tops ARE the distinct
/// sizes, and they come out of one sorted pass.
///
/// Returns `(top, cumulative_clauses, cumulative_sum, admitted_here)`.
pub fn plateaus(finite: &[u128]) -> Vec<(u128, usize, u128, usize)> {
    let mut sizes: Vec<u128> = finite.to_vec();
    sizes.sort_unstable();
    let mut out = Vec::new();
    let (mut n, mut s) = (0usize, 0u128);
    let mut i = 0;
    while i < sizes.len() {
        let t = sizes[i];
        let mut k = 0;
        while i < sizes.len() && sizes[i] == t {
            k += 1;
            i += 1;
        }
        n += k;
        s = s.saturating_add(t.saturating_mul(k as u128));
        out.push((t, n, s, k));
    }
    out
}

/// `2^16` when the number is an exact power of two, grouped digits otherwise.
///
/// 82% of the finite domain sizes in this corpus are exact powers of 256 --
/// it is a hardware-spec corpus, and the quantifiers run over register widths.
/// Printing `18 446 744 073 709 551 616` hides that; printing `2^64` states it.
fn width(n: u128) -> String {
    if n != 0 && n & (n - 1) == 0 {
        let mut e = 0u32;
        let mut v = n;
        while v > 1 {
            v >>= 1;
            e += 1;
        }
        if e >= 8 {
            return format!("2^{e}");
        }
    }
    if n == u128::MAX {
        return "saturated".to_string();
    }
    group(n)
}

/// Grouped digits while they stay readable, then an exponent. A twenty-eight
/// digit evaluation count in a fixed-width column pushes every other column off
/// the line, and nobody reads the twenty-eighth digit of a number that saturated.
fn compact(n: u128) -> String {
    if n == u128::MAX {
        return ">= saturated".to_string();
    }
    if n < 1_000_000_000_000_000 {
        return group(n);
    }
    let d = n.to_string();
    format!("~{}.{}e{}", &d[..1], &d[1..3], d.len() - 1)
}

/// Digits in groups of three. A nine-digit evaluation count is unreadable
/// otherwise, and this report exists to be read by someone choosing a ceiling.
fn group(n: u128) -> String {
    let d = n.to_string();
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
        if i > 0 && (d.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Does any line carry a quantifier keyword the census did not read?
///
/// The looser reader is the whole point: a control sharing the strict matcher
/// would agree with it by construction and measure nothing.
fn audit() -> Result<()> {
    let root = repo_root()?;
    let specs = read_specs(&root);
    if specs.is_empty() {
        anyhow::bail!(
            "no specs under {}/specs -- nothing was read",
            root.display()
        );
    }
    let read: BTreeSet<(String, usize)> = scan_clauses(&specs)
        .iter()
        .map(|c| (c.file.clone(), c.line))
        .collect();

    const KEYWORDS: [&str; 4] = ["forall", "for all", "for any", "for positive"];
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';

    let (mut loose, mut word_residue, mut inside_ident) = (0usize, Vec::new(), 0usize);
    for (path, src) in &specs {
        let file = path.display().to_string();
        for (i, raw) in src.lines().enumerate() {
            let masked = mask(raw);
            let t = masked.trim();
            let Some(kw) = KEYWORDS.iter().find(|k| t.contains(**k)) else {
                continue;
            };
            loose += 1;
            if read.contains(&(file.clone(), i + 1)) {
                continue;
            }
            let b = t.as_bytes();
            let idx = t.find(*kw).unwrap_or(0);
            let j = idx + kw.len();
            let standalone = (idx == 0 || !ident(b[idx - 1])) && (j >= b.len() || !ident(b[j]));
            if standalone {
                word_residue.push((file.clone(), i + 1, *kw, raw.trim().to_string()));
            } else {
                inside_ident += 1;
            }
        }
    }

    println!();
    println!("  the census counts what its matcher built, so it cannot report what");
    println!("  its matcher never saw. This reads the corpus with the bare letters.");
    println!();
    println!("      lines carrying the letters       {loose}");
    println!("      turned into a clause             {}", read.len());
    println!("      letters inside an identifier     {inside_ident}  (not a quantifier; forgiven)");
    println!(
        "      keyword as its own WORD, unread  {}",
        word_residue.len()
    );
    if !word_residue.is_empty() {
        println!();
        for (f, l, kw, text) in &word_residue {
            println!("      {f}:{l}  [{kw}]");
            println!("          {text}");
        }
        println!();
        anyhow::bail!(
            "{} line(s) write a quantifier keyword as a word and the census built no clause for them",
            word_residue.len()
        );
    }
    println!();
    println!("  Every quantifier keyword written as a word is a clause the census counted.");
    Ok(())
}

pub fn run(cmd: &QuantCmd) -> Result<()> {
    if let QuantCmd::Audit = cmd {
        return audit();
    }
    let QuantCmd::Report { full, ceiling } = cmd else {
        unreachable!("Audit returned above")
    };
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
    let scope = scan_scope(&specs);
    let by_file: BTreeMap<String, Vec<&str>> = specs
        .iter()
        .map(|(p, s)| (p.display().to_string(), s.lines().collect()))
        .collect();
    // Per walkable clause: does every name in its body resolve in its own
    // file's scope? Says nothing about truth -- only whether the clause is
    // about functions that exist.
    let mut unresolved: BTreeMap<String, usize> = BTreeMap::new();
    let (mut res_ok, mut res_undef, mut res_amb) = (0usize, 0usize, 0usize);
    // Vacuity is decided from the clause text alone, so it is measured over ALL
    // 924 clauses -- not only the walkable ones. Scope matters: `!= undefined`
    // occurs 577 times in specs/, and only a handful are in quantified clauses.
    // Widening past the clause list would multiply the output for no signal.
    let mut vac: Vec<(String, usize, Vacuity)> = Vec::new();
    let mut says_something = 0usize;
    // Arity is measured over ALL 924 clauses, not the walkable ones: only 1 of
    // the 12 sites is walkable, and a walkable-only column would print 1 and
    // look finished.
    let mut arity_bad: Vec<(String, usize, String, usize, usize)> = Vec::new();

    let mut by_notation: BTreeMap<&str, usize> = BTreeMap::new();
    let (mut walkable, mut over, mut unbounded, mut no_binder) = (0usize, 0usize, 0usize, 0usize);
    let mut walkable_sizes: Vec<(u128, String, usize)> = Vec::new();
    // Every finite |D|, walkable or not. `--ceiling` never enters `size_of`, so
    // this multiset is ceiling-invariant and the whole sweep falls out of one
    // scan -- no re-reading the corpus per candidate ceiling.
    let mut finite_sizes: Vec<u128> = Vec::new();
    // An unbounded clause whose every unbounded binder names a CONFLICTED
    // struct is unbounded for a reason someone could fix.
    let (mut touch_conflict, mut hostage) = (0usize, 0usize);

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
        let (mut unb, mut unb_conflicted) = (0usize, 0usize);
        for (_, ty) in &c.binders {
            match size_of(ty, &structs, 0) {
                Size::Finite(n) => total = total.map(|t| t.saturating_mul(n)),
                Size::Unbounded => {
                    total = None;
                    unb += 1;
                    if structs.conflicted.contains(ty.trim()) {
                        unb_conflicted += 1;
                    }
                }
            }
        }
        if let Some(n) = total {
            finite_sizes.push(n);
        } else {
            if unb_conflicted > 0 {
                touch_conflict += 1;
                if unb_conflicted == unb {
                    hostage += 1;
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
                let body = by_file
                    .get(&c.file)
                    .map(|l| clause_body(l, c.line))
                    .unwrap_or_default();
                let (u, a) = resolution(&scope, &c.file, &body);
                if !u.is_empty() {
                    res_undef += 1;
                    for n in &u {
                        *unresolved.entry(n.clone()).or_default() += 1;
                    }
                } else if !a.is_empty() {
                    res_amb += 1;
                    for n in &a {
                        *unresolved.entry(format!("{n} (ambiguous)")).or_default() += 1;
                    }
                } else {
                    res_ok += 1;
                }
                format!("walkable |D| = {n}")
            }
            Some(n) => {
                over += 1;
                format!("finite but over ceiling |D| = {n}")
            }
        };
        {
            let vbody = by_file
                .get(&c.file)
                .map(|l| clause_body(l, c.line))
                .unwrap_or_default();
            let names: Vec<String> = c.binders.iter().map(|(n, _)| n.clone()).collect();
            for (n, got, want) in arity_mismatches(&scope, &c.file, &vbody) {
                arity_bad.push((c.file.clone(), c.line, n, got, want));
            }
            let v = vacuity(&names, &vbody);
            if v.is_empty() {
                says_something += 1;
            } else {
                for one in v {
                    vac.push((c.file.clone(), c.line, one));
                }
            }
        }
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
    if !walkable_sizes.is_empty() {
        let mut hist: BTreeMap<u128, usize> = BTreeMap::new();
        for (n, _, _) in &walkable_sizes {
            *hist.entry(*n).or_default() += 1;
        }
        let sum: u128 = walkable_sizes.iter().map(|(n, _, _)| *n).sum();
        println!();
        println!("  COST OF WALKING THEM, if a backend enumerated the declared domain");
        println!(
            "    sum of |D| over the {walkable} walkable clauses   {} evaluations",
            group(sum)
        );
        for (n, k) in &hist {
            println!(
                "        |D| = {:>7}  x {:>3}  = {:>13}",
                width(*n),
                k,
                group(n.saturating_mul(*k as u128))
            );
        }
        if let Some((top, k)) = hist.iter().next_back() {
            let share = *top * (*k as u128);
            println!(
                "    {k} clause(s) ({:.1}% of walkable) carry {} evaluations ({:.2}% of the total).",
                100.0 * *k as f64 / walkable as f64,
                group(share),
                100.0 * share as f64 / sum as f64
            );
        }
        println!();
        println!("    THIS IS AN ITERATION COUNT, NOT A COST. No backend executes an");
        println!("    enumerated quantifier today, so nothing has ever paid this number.");
        println!("    It is also blind to body weight: the same |D| costs a constant-time");
        println!("    body and a sixteen-iteration one the same here, and they are not the");
        println!("    same. Do not quote it as seconds. See #2774.");

        println!();
        println!("    Against the whole quantified corpus, not just the finite part:");
        println!(
            "      finite (walkable + over ceiling)  {:>4} of {}   {:.1}%",
            finite_sizes.len(),
            clauses.len(),
            100.0 * finite_sizes.len() as f64 / clauses.len() as f64
        );
        println!(
            "      walkable at this ceiling          {:>4} of {}   {:.1}%",
            walkable,
            clauses.len(),
            100.0 * walkable as f64 / clauses.len() as f64
        );
        println!(
            "    No ceiling moves the {unbounded} unbounded clauses -- that is {:.1}% of the",
            100.0 * unbounded as f64 / clauses.len() as f64
        );
        println!("    corpus, and it is the larger half.");

        println!();
        println!("  DOES THE CLAUSE NAME FUNCTIONS THAT EXIST?");
        println!("    Resolved in the clause's own file plus the files its `use` lines name.");
        println!("    This says NOTHING about whether a clause is TRUE -- only whether it is");
        println!("    about anything. A clause naming an undefined function cannot be checked");
        println!("    by any means, and counting it beside the checkable ones hides that.");
        println!();
        println!("      every name resolves        {res_ok}");
        println!("      names a function nobody defines in scope   {res_undef}");
        println!("      names one defined more than once in scope  {res_amb}");
        if !unresolved.is_empty() {
            println!();
            println!("    The names, and how many walkable clauses each blocks:");
            let mut v: Vec<(&String, &usize)> = unresolved.iter().collect();
            v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (n, k) in v {
                println!("      {k:>3}  {n}");
            }
            println!();
            println!("    There is no builtin table here on purpose. A language-level name");
            println!("    appears in this list like any other, and a reader recognises it at");
            println!("    a glance -- an allowlist would have been tuned until the number");
            println!("    looked right, which is not a measurement.");
        }

        println!();
        println!("  DOES THE CALL PASS THE RIGHT NUMBER OF ARGUMENTS?");
        println!("    The compiler cannot answer this here. `parse_invariant_clause` discards a");
        println!("    quantified clause on purpose (#2774), so the body produces no AST and every");
        println!("    AST-based check is blind to it -- including `t27c check-calls`, which finds");
        println!("    95 of these corpus-wide and 0 inside a clause.");
        println!();
        println!(
            "      call sites that could not compile   {}",
            arity_bad.len()
        );
        if !arity_bad.is_empty() {
            let mut seen: BTreeMap<(&String, usize), ()> = BTreeMap::new();
            for (f, l, _, _, _) in &arity_bad {
                seen.insert((f, *l), ());
            }
            println!("      distinct clauses                    {}", seen.len());
            println!();
            for (f, l, n, got, want) in &arity_bad {
                println!("      {f}:{l}  {n}  passes {got}, declared {want}");
            }
        }
        println!();
        println!("    Four of the six rules ABSTAIN rather than guess: an unclosed paren, a name");
        println!("    in method position (`x.len()` -- the receiver IS the argument), a name no");
        println!("    declaration in scope defines, and a name whose visible scope offers more");
        println!("    than one arity. A naive scan of the same bodies reports ~317; 97% of that");
        println!("    is the receiver convention, and none of it is a property of the corpus.");

        println!();
        println!("  IS THE CLAUSE VACUOUS?");
        println!("    Decided from the clause text alone. Nothing is evaluated and no type is");
        println!("    resolved. This says NOTHING about whether a clause is TRUE -- a vacuous");
        println!("    clause IS true, in every model, which is exactly the problem: it passes");
        println!("    every checker forever and counts as coverage while saying nothing.");
        println!();
        println!("      says something            {says_something}");
        println!("      vacuous                   {}", vac.len());
        {
            let mut by: BTreeMap<&str, usize> = BTreeMap::new();
            for (_, _, v) in &vac {
                *by.entry(v.kind()).or_default() += 1;
            }
            println!();
            for (k, n) in by.iter().rev() {
                println!("       {n:>3}  tautology, {k}");
            }
            for k in [
                "A == A",
                "P ==> P",
                "X != undefined",
                "binder never used in the predicate",
            ] {
                if !by.contains_key(k) {
                    println!("         0  {k} -- looked for, not found");
                }
            }
        }
        println!();
        println!("    NOT DECIDED HERE: whether a bound is the declared TYPE's own range --");
        println!("    `x >= 0` on an unsigned, `x <= 255` on a u8, all variants of an enum");
        println!("    listed. That needs a type table, and a type table carries judgement: a");
        println!("    name with two definitions flips a verdict when a refactor touches it.");
        println!("    The kinds above need no types at all, which is why they ship first.");
        println!();
        println!("    ONE ASSUMPTION IS LOAD-BEARING and is not proved here: `f(x) == f(x)` is");
        println!("    a tautology only if spec functions are deterministic. This repository");
        println!("    states determinism as a purity TARGET, not a proved property.");
        if *full && !vac.is_empty() {
            println!();
            for (f, l, v) in &vac {
                println!("      {f}:{l}  {}  {}", v.kind(), v.evidence());
            }
        }

        // The sweep. Every DISTINCT finite size is a plateau top: raising the
        // ceiling anywhere between two adjacent sizes changes nothing, so a
        // sweep that samples round numbers reports flat regions it never
        // measured. Derived from the multiset in one pass instead.
        let rows = plateaus(&finite_sizes);
        println!();
        println!("  CEILING SWEEP -- every ceiling that is not a synonym for another one.");
        println!("  |D| never consults --ceiling, so this whole table is one scan.");
        println!();
        println!("        ceiling  walkable                sum |D|   admits");
        for (t, cum_n, cum_s, k) in &rows {
            let (t, cum_n, cum_s, k) = (*t, *cum_n, *cum_s, *k);
            let mark = if t == *ceiling { "  <- DEFAULT" } else { "" };
            println!(
                "    {:>11} {:>9} {:>22}   +{:<3} at |D| = {}{}",
                width(t),
                cum_n,
                compact(cum_s),
                k,
                width(t),
                mark
            );
        }
        if finite_sizes.iter().any(|n| *n == u128::MAX) {
            println!();
            println!("    The last row is a LOWER BOUND, not a value: `primitive` maps u128 to");
            println!("    u128::MAX and `size_of` saturates, so those rows are overflowed");
            println!("    products rather than measured sizes.");
        }
        println!();
        println!("    Every count above is a lower bound for a second reason: {hostage} unbounded");
        println!("    clause(s) are unbounded ONLY because a struct name has more than one");
        println!(
            "    definition ({touch_conflict} touch such a name). Resolving them -- see `tri types dup`"
        );
        println!("    -- can only move clauses INWARD. It cannot move any out.");
    }

    if !structs.conflicted.is_empty() {
        println!();
        println!(
            "  {} struct name(s) have MORE THAN ONE definition and are treated as",
            structs.conflicted.len()
        );
        println!("  unbounded. Picking one would change |D| by an unbounded factor with");
        println!("  nothing recording which was picked:");
        // Eight of eighty, printed with no marker, reads as the whole list.
        // A truncation nobody is told about is the same defect one level down
        // from the ones this report exists to name.
        for n in structs.conflicted.iter().take(8) {
            println!("      {n}");
        }
        if structs.conflicted.len() > 8 {
            println!(
                "      ... and {} more. The full list, with a DRIFT/DISTINCT verdict",
                structs.conflicted.len() - 8
            );
            println!("      and the reading behind each: `tri types classified`.");
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

    /// W705: `invariant NAME: forall x : T` and a bare `forall x : T` are the
    /// SAME construct written two ways, and the first version split them into
    /// two of four buckets while missing a real fourth form entirely. One
    /// `forall` bucket, and the suffix forms named separately.
    #[test]
    fn the_notations_are_recognised() {
        let src = "\
    invariant a: forall c : Cfg, c.x > 0
    forall input : In
    assert f(a) == f(b) for all Trit a, b
    assert g(a) == g(b) for any a, b in {1, -1}
    assert h(x) for positive x, integer n
";
        let cs = scan_clauses(&[(PathBuf::from("x.t27"), src.to_string())]);
        let mut kinds: Vec<&str> = cs.iter().map(|c| c.notation).collect();
        kinds.sort();
        assert_eq!(
            kinds,
            vec![
                "forall",
                "forall",
                "suffix-all",
                "suffix-any",
                "suffix-positive"
            ]
        );
    }

    /// A comment is not a clause. 99 of 135 rows in the first census were.
    #[test]
    fn comments_and_strings_are_not_clauses() {
        let src = "\
    // Verify: trit_min(a, a) == a for all a
    const equation = \"holds for all x > 0\";
    return .pos;  // x^0 = 1 for any x != 0
";
        let cs = scan_clauses(&[(PathBuf::from("x.t27"), src.to_string())]);
        assert!(
            cs.is_empty(),
            "{:?}",
            cs.iter().map(|c| &c.text).collect::<Vec<_>>()
        );
    }

    /// The binder forms the corpus writes, every one found by reading it.
    #[test]
    /// W710: the name asserted corpus-wide coverage that four hand-written
    /// cases never provided, and that false ruler is the structural reason a
    /// 297-clause defect survived a green suite for as long as it did. The
    /// multi-binder colon list is folded in here so the name is true.
    fn every_binder_form_in_the_corpus_is_read() {
        // `name : Type, name : Type` -- 297 clauses, the shape this test was
        // named after and did not contain.
        assert_eq!(
            parse_binders("a : i16, b : i16"),
            vec![
                ("a".to_string(), "i16".to_string()),
                ("b".to_string(), "i16".to_string())
            ]
        );
        // `Type name, name`
        assert_eq!(
            parse_binders("Trit a, b"),
            vec![
                ("a".to_string(), "Trit".to_string()),
                ("b".to_string(), "Trit".to_string())
            ]
        );
        // `name, name : Type`
        assert_eq!(
            parse_binders("exp, mant: u8, body()"),
            vec![
                ("exp".to_string(), "u8".to_string()),
                ("mant".to_string(), "u8".to_string())
            ]
        );
        // `name in Type where guard`
        assert_eq!(
            parse_binders("k in u8 where k <= 27"),
            vec![("k".to_string(), "u8".to_string())]
        );
        // `name in <collection>` -- a binder, with a domain `size_of` will call
        // unbounded. That is a different answer from "no binder".
        assert_eq!(parse_binders("r in results").len(), 1);
    }

    // ---- W710: the binder list is a LIST -------------------------------
    //
    // Every case below is a real corpus line, cited. `parse_binders` read the
    // first binder of each and dropped the rest, so a clause binding four
    // variables was sized as though it bound one.

    /// `cordic_top.t27:311` -- four binders, one colon each.
    #[test]
    fn a_colon_list_binds_every_name() {
        assert_eq!(
            parse_binders("clk : bool, rst_n : bool, angle : i16, valid_in : bool"),
            vec![
                ("clk".to_string(), "bool".to_string()),
                ("rst_n".to_string(), "bool".to_string()),
                ("angle".to_string(), "i16".to_string()),
                ("valid_in".to_string(), "bool".to_string()),
            ]
        );
    }

    /// The trap. 69 clauses put the BODY after the comma, and a fix that keeps
    /// walking segments would mint a binder out of it -- corrupting 69 rows to
    /// repair 297. `_tmp_pipeline_import.t27:1476`.
    #[test]
    fn a_body_after_the_binder_is_not_a_binder() {
        assert_eq!(
            parse_binders("p : PipelineResult, pipeline_token_count(p) >= 0"),
            vec![("p".to_string(), "PipelineResult".to_string())]
        );
    }

    /// `phi_split_optimality.t27:293` -- the body is a brace block.
    #[test]
    fn a_brace_body_after_the_binder_is_not_a_binder() {
        assert_eq!(
            parse_binders("bits: u8, { let (e, m) = f(bits); e + m == bits"),
            vec![("bits".to_string(), "u8".to_string())]
        );
    }

    /// PROBE: the keyword alone, binders absent, predicate on the next line.
    ///
    /// `specs/igla/coder/benchmark.t27:3827` writes exactly this. The old
    /// matcher was `find("forall ")` -- the spelling, with a trailing space --
    /// so this line matched nothing and never became a clause. The census read
    /// 922 where the corpus holds 923, and `no binder this can read` was one
    /// short of the bucket this clause belongs in.
    #[test]
    fn a_bare_forall_is_a_clause_with_no_binders() {
        let src = "invariant bounded:\nforall\nestimate() > 0.0\n";
        let cs = scan_clauses(&[(PathBuf::from("x.t27"), src.to_string())]);
        assert_eq!(cs.len(), 1, "the keyword alone still opens a clause");
        assert_eq!(cs[0].notation, "forall");
        assert!(
            cs[0].binders.is_empty(),
            "and it quantifies over nothing, which is what the bucket is for"
        );
    }

    /// COUNTER: the letters inside an identifier are not the keyword.
    ///
    /// Reading the keyword as a WORD is stricter on the left edge than
    /// `find("forall ")` was. Measured before changing it: zero lines in the
    /// corpus put an identifier character before the keyword, so the change
    /// could not drop a clause -- but the rule has to hold anyway, or the
    /// census starts inventing clauses out of variable names.
    #[test]
    fn the_letters_inside_an_identifier_are_not_the_keyword() {
        for src in [
            "myforall x : u8\n",
            "forall_count = 3\n",
            "let xforally : u8 = 1\n",
        ] {
            let cs = scan_clauses(&[(PathBuf::from("x.t27"), src.to_string())]);
            assert!(cs.is_empty(), "{src:?} is not a quantified clause");
        }
    }

    /// `systolic_array.t27:287` -- the second quantifier keyword sits INSIDE
    /// the binder text, because `scan_clauses` finds the first `forall ` only.
    #[test]
    fn a_second_forall_keyword_is_not_a_body() {
        assert_eq!(
            parse_binders("a : i16, forall b : i16,"),
            vec![
                ("a".to_string(), "i16".to_string()),
                ("b".to_string(), "i16".to_string()),
            ]
        );
    }

    /// `bench_proxy.t27:349` -- names separated by a space, not a comma. Filed
    /// as "no binder" today, which is a different lie from an undersized one.
    #[test]
    fn space_separated_names_before_the_colon() {
        assert_eq!(
            parse_binders(
                "a b : i32, a <= b, verilog_eval_problems(a) <= verilog_eval_problems(b)"
            ),
            vec![
                ("a".to_string(), "i32".to_string()),
                ("b".to_string(), "i32".to_string()),
            ]
        );
    }

    /// `gf16_bfloat16_nmse.t27:89` and `:96`. Here the colon means "such that",
    /// not "has type". The old reader FABRICATED a binder `[D: F,]` -- a name
    /// and a type neither of which is in the source. No binder is the truth.
    #[test]
    fn a_colon_that_is_not_a_type_ascription_yields_no_binder() {
        assert!(parse_binders("F, D : NMSE(F, D) >= 0").is_empty());
        assert!(parse_binders("D : if mean_sq_ref(D) == 0 then a else b").is_empty());
    }

    /// `jones_polynomial.t27:321` -- the example in this module's own doc
    /// comment, which the function has never actually parsed.
    #[test]
    fn grouped_names_share_a_membership_domain() {
        let b = parse_binders("a, b in {1, -1}");
        assert_eq!(b.len(), 2, "got {b:?}");
        assert_eq!(b[0].0, "a");
        assert_eq!(b[1].0, "b");
    }

    /// `phi_split_optimality.t27:301` -- the domain must stop at the binder
    /// list, not swallow the body that follows it.
    #[test]
    fn a_membership_domain_stops_at_the_binder_list() {
        assert_eq!(
            parse_binders("i in 0..verification.length(), verification[i].matches == true"),
            vec![("i".to_string(), "0..verification.length()".to_string())]
        );
    }

    /// A bracket type is one type. Pins the depth-aware comma split.
    #[test]
    fn a_bracket_type_is_one_binder_not_two() {
        assert_eq!(
            parse_binders("w : []TernaryWeight, b : WeightBank"),
            vec![
                ("w".to_string(), "[]TernaryWeight".to_string()),
                ("b".to_string(), "WeightBank".to_string()),
            ]
        );
    }

    /// `audio_overview.t27:144`. Guards are still not read -- that is the
    /// owner's semantics to choose, and #2774 is where it is chosen.
    #[test]
    fn a_where_guard_is_not_a_binder() {
        assert_eq!(
            parse_binders("lang : str where @langToCode(lang) != \"\","),
            vec![("lang".to_string(), "str".to_string())]
        );
    }

    // ---- W711: cost, not count ------------------------------------------

    /// A ceiling between two domain sizes is a synonym for the lower one. The
    /// sweep must report the SIZES, not the round numbers a reader would guess.
    #[test]
    fn only_the_distinct_sizes_are_real_ceilings() {
        let rows = plateaus(&[2, 2, 256, 256, 256, 65536]);
        assert_eq!(rows.len(), 3, "one row per distinct size, got {rows:?}");
        assert_eq!(rows[0], (2, 2, 4, 2));
        assert_eq!(rows[1], (256, 5, 772, 3));
        assert_eq!(rows[2], (65536, 6, 66308, 1));
    }

    /// The cumulative sum must saturate rather than panic. `primitive` maps
    /// u128 to u128::MAX and 46 corpus rows are overflowed products; before the
    /// saturating form this table killed the whole report on its last line.
    #[test]
    fn a_saturated_size_does_not_panic_the_sweep() {
        let rows = plateaus(&[u128::MAX, u128::MAX, 2]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].2, u128::MAX, "cumulative sum saturates");
    }

    #[test]
    fn an_empty_corpus_sweeps_to_nothing() {
        assert!(plateaus(&[]).is_empty());
    }

    /// 82% of this corpus's finite domains are exact powers of 256: it is a
    /// hardware-spec corpus and the quantifiers run over register widths.
    /// `18 446 744 073 709 551 616` hides that; `2^64` states it.
    #[test]
    fn a_register_width_prints_as_a_power() {
        assert_eq!(width(65536), "2^16");
        assert_eq!(width(1u128 << 64), "2^64");
        assert_eq!(width(256), "2^8");
        // Below 2^8 the digits are shorter than the exponent form and easier
        // to compare against a domain the reader can count.
        assert_eq!(width(16), "16");
        assert_eq!(width(27), "27");
        assert_eq!(width(u128::MAX), "saturated");
    }

    #[test]
    fn a_long_number_keeps_its_column() {
        assert_eq!(group(2768791), "2 768 791");
        assert_eq!(group(10), "10");
        assert_eq!(compact(16279), "16 279");
        assert!(
            compact(1u128 << 70).starts_with('~'),
            "{}",
            compact(1u128 << 70)
        );
        assert_eq!(compact(u128::MAX), ">= saturated");
    }

    // ---- W713: does the clause name functions that exist? --------------

    /// The stored clause text is ONE line -- `forall i : u8` -- and the
    /// predicate is below it. Without the window there is nothing to resolve.
    #[test]
    fn the_body_is_the_indented_lines_below_the_clause() {
        let src: Vec<&str> = vec![
            "    invariant cordic_pow2_neg_entry_bounded:",
            "        forall i : u8",
            "        pow2_neg_entry(i) > 0.0 && pow2_neg_entry(i) <= 1.0",
            "",
            "    invariant next_one:",
        ];
        let b = clause_body(&src, 2);
        assert!(b.contains("pow2_neg_entry"), "{b}");
        assert!(!b.contains("next_one"), "the blank line ends it: {b}");
    }

    /// A dedent ends the body even with no blank line between.
    #[test]
    fn a_dedent_ends_the_body() {
        let src: Vec<&str> = vec!["        forall i : u8", "        p(i)", "    fn other() {"];
        let b = clause_body(&src, 1);
        assert!(b.contains("p(i)"));
        assert!(!b.contains("other"), "{b}");
    }

    #[test]
    fn a_line_past_the_end_is_empty_not_a_panic() {
        assert_eq!(clause_body(&["a"], 9), "");
        assert_eq!(clause_body(&["a"], 0), "");
    }

    /// `let (s, c) = f(x)` is a call to `f`, not to `let`. The prototype of
    /// this check counted `let` as an undefined function.
    #[test]
    fn a_keyword_before_a_paren_is_not_a_call() {
        let n = called_names("let (s, c) = cordic_sin_cos(a, b)\nif (x) { g(y) }");
        assert!(n.contains("cordic_sin_cos"), "{n:?}");
        assert!(n.contains("g"), "{n:?}");
        assert!(!n.contains("let"), "{n:?}");
        assert!(!n.contains("if"), "{n:?}");
    }

    #[test]
    fn a_comment_is_not_a_body() {
        let n = called_names("p(x)  // see also q(y)");
        assert_eq!(n.iter().cloned().collect::<Vec<_>>(), vec!["p".to_string()]);
    }

    /// Scope is the file plus what it `use`s -- not the whole corpus. A name
    /// defined ten times corpus-wide and imported zero times does not resolve.
    #[test]
    fn scope_is_the_file_plus_what_it_uses() {
        let specs = vec![
            (
                PathBuf::from("specs/a/one.t27"),
                "use b::two;\nfn local() {}\n".to_string(),
            ),
            (
                PathBuf::from("specs/b/two.t27"),
                "fn shared() {}\n".to_string(),
            ),
            (
                PathBuf::from("specs/c/three.t27"),
                "fn hidden() {}\n".to_string(),
            ),
        ];
        let sc = scan_scope(&specs);
        let (u, a) = resolution(&sc, "specs/a/one.t27", "local(1) shared(2)");
        assert!(u.is_empty() && a.is_empty(), "u={u:?} a={a:?}");
        let (u, _) = resolution(&sc, "specs/a/one.t27", "hidden(3)");
        assert_eq!(
            u,
            vec!["hidden".to_string()],
            "not imported, so not resolved"
        );
    }

    /// Two visible definitions is a different answer from none, and the report
    /// must not collapse them: one is unwritten, the other is undecided.
    #[test]
    fn two_visible_definitions_is_ambiguous_not_undefined() {
        let specs = vec![
            (
                PathBuf::from("specs/a/one.t27"),
                "use b::two;\nfn dup() {}\n".to_string(),
            ),
            (
                PathBuf::from("specs/b/two.t27"),
                "fn dup() {}\n".to_string(),
            ),
        ];
        let sc = scan_scope(&specs);
        let (u, a) = resolution(&sc, "specs/a/one.t27", "dup(1)");
        assert!(u.is_empty(), "{u:?}");
        assert_eq!(a, vec!["dup".to_string()]);
    }

    // ---- W714: a clause that asserts nothing ---------------------------
    //
    // Every negative case below is a REAL corpus clause that a flattened
    // regex calls a tautology and that says something. Three false positives
    // in the first eight hits is how a check gets classified as noise in its
    // first pull request, so they are pinned here rather than in prose.

    #[test]
    fn a_call_compared_to_itself_is_a_tautology() {
        let v = vacuity(
            &["kw".into()],
            "forall kw : string\nencode_keyword(kw) == encode_keyword(kw)",
        );
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind(), "A == A");
    }

    /// `weights.t27:741` -- `depth == depth` is a SUBSTRING of a real claim
    /// that the bank preserves the depth it was given.
    #[test]
    fn a_reflexive_substring_is_not_a_tautology() {
        let v = vacuity(
            &["depth".into()],
            "forall depth : u8\nint4_dequantize_bank(codes, depth, width).depth == depth",
        );
        assert!(v.is_empty(), "{v:?}");
    }

    /// `systolic_array.t27:272` -- `b == b` inside real i16 commutativity.
    #[test]
    fn commutativity_is_not_a_tautology() {
        let v = vacuity(
            &["a".into(), "b".into()],
            "forall a : i16, b : i16\na * b == b * a",
        );
        assert!(v.is_empty(), "{v:?}");
    }

    /// `cordic_top.t27:848` -- `.0 == 0` is a tuple index, not `0 == 0`.
    #[test]
    fn a_tuple_index_is_not_a_reflexive_comparison() {
        let v = vacuity(&["x".into()], "forall x : i16\ncordic_step(x).0 == 0");
        assert!(v.is_empty(), "{v:?}");
    }

    /// `backend.t27:492` -- the one arithmetic identity the corpus writes.
    #[test]
    fn plus_zero_folds_before_the_comparison() {
        let v = vacuity(&["x".into()], "forall x : i32\nx + 0 == x");
        assert_eq!(v.len(), 1, "{v:?}");
    }

    /// The predicate is often on the SAME line as the binder list, so "does
    /// the name appear below the head line" is the wrong question. A
    /// line-based version emitted seven hits of which four were this shape.
    #[test]
    fn a_same_line_predicate_uses_its_binder() {
        let v = vacuity(
            &["g".into()],
            "invariant m: forall g : Gemm, g.M > 0 and g.N > 0",
        );
        assert!(v.is_empty(), "{v:?}");
    }

    /// `adder_tree.t27:1050` -- every argument a literal, so the quantifier
    /// over 2^32 values decorates a single ground equation.
    #[test]
    fn a_binder_that_never_appears_is_decoration() {
        let v = vacuity(
            &["a".into()],
            "forall a : i32\nadder_tree_4(0, 0, 0, 0) == 0",
        );
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind(), "binder never used in the predicate");
    }

    /// A binder must match as a WORD: `a` is not the `a` inside `arctan`.
    #[test]
    fn a_binder_matches_as_a_word_not_a_substring() {
        let v = vacuity(&["a".into()], "forall a : u8\narctan_table_entry(3) > 0.0");
        assert_eq!(
            v.len(),
            1,
            "the `a` in arctan does not count as a use: {v:?}"
        );
    }

    /// The compiler says it in its own words: this "constrain[s] the value
    /// not at all ... which is trivially true".
    #[test]
    fn not_undefined_constrains_nothing() {
        let v = vacuity(&["e".into()], "forall e : StepKind, e != undefined");
        assert_eq!(v.len(), 1, "{v:?}");
        assert_eq!(v[0].kind(), "X != undefined");
    }

    /// `!=` and `==>` must not be read as `==`.
    #[test]
    fn a_comparison_operator_is_not_a_prefix_of_another() {
        assert!(vacuity(&["x".into()], "forall x : u8\nf(x) != f(x)").is_empty());
        let v = vacuity(&["x".into()], "forall x : u8\nf(x) ==> g(x)");
        assert!(v.is_empty(), "an implication with different sides: {v:?}");
    }

    #[test]
    fn conjuncts_split_per_line_and_at_depth_zero() {
        let c = conjuncts("a == b && f(x, y) > 0\nc and d");
        assert_eq!(c, vec!["a == b", "f(x, y) > 0", "c", "d"], "{c:?}");
    }

    // ---- W716: arity in a clause the compiler never parses ---------------

    /// A comma split would say three. `f(g(a, b), c)` passes two, and 80
    /// clause windows in this corpus carry a nested group.
    #[test]
    fn a_nested_call_is_one_argument() {
        let c = calls("f(g(a, b), c)");
        assert_eq!(c[0].name, "f");
        assert_eq!(c[0].args, Some(2), "{c:?}");
        assert_eq!(c[1].name, "g");
        assert_eq!(c[1].args, Some(2));
    }

    /// A walk that only notices non-space characters scores `f([])` as zero.
    #[test]
    fn an_empty_bracket_is_still_an_argument() {
        assert_eq!(calls("f([])")[0].args, Some(1));
        assert_eq!(calls("f()")[0].args, Some(0));
        assert_eq!(calls("f(  )")[0].args, Some(0));
    }

    /// 306 of the 313 method-position calls in clause windows are `.len()`.
    /// The receiver IS the argument; 0-vs-1 there is a convention.
    #[test]
    fn a_method_call_is_marked_and_never_counted_against_a_declaration() {
        let c = calls("samples.len() == 0");
        assert_eq!(c[0].name, "len");
        assert!(c[0].method, "{c:?}");
        let specs = vec![(
            PathBuf::from("specs/a.t27"),
            "fn len(x: u8) -> u8 {}\n".to_string(),
        )];
        let sc = scan_scope(&specs);
        assert!(
            arity_mismatches(&sc, "specs/a.t27", "samples.len() == 0").is_empty(),
            "method position must abstain"
        );
    }

    /// A call whose paren never closes inside the window is not scored short.
    #[test]
    fn an_unclosed_paren_abstains() {
        assert_eq!(calls("f(a, b")[0].args, None);
    }

    /// `fn cordic_top(` opens and its four parameters follow below. Reading
    /// only the head line records arity 0 and fabricates a defect against
    /// every correct four-argument call -- 19 of 31 rows in this column's
    /// first run.
    #[test]
    fn a_declaration_that_wraps_is_not_read_as_zero_parameters() {
        let specs = vec![(
            PathBuf::from("specs/a.t27"),
            "fn wide(\n    a: u8,\n    b: u8,\n) -> u8 {}\n".to_string(),
        )];
        let sc = scan_scope(&specs);
        assert!(
            sc.arity
                .get(&("specs/a.t27".to_string(), "wide".to_string()))
                .is_none(),
            "a wrapped declaration must abstain, not record 0"
        );
        assert!(
            arity_mismatches(&sc, "specs/a.t27", "wide(1, 2)").is_empty(),
            "and nothing may be reported against it"
        );
    }

    /// Two visible declarations with different arities is an overload
    /// question t27 has not settled, not a mismatch.
    #[test]
    fn two_arities_in_scope_abstain() {
        let specs = vec![
            (
                PathBuf::from("specs/a.t27"),
                "use b;\nfn f(a: u8) -> u8 {}\n".to_string(),
            ),
            (
                PathBuf::from("specs/b.t27"),
                "fn f(a: u8, b: u8) -> u8 {}\n".to_string(),
            ),
        ];
        let sc = scan_scope(&specs);
        assert!(arity_mismatches(&sc, "specs/a.t27", "f(1, 2, 3)").is_empty());
    }

    /// The whole point: one arity in scope, a different count passed.
    #[test]
    fn one_arity_in_scope_and_a_different_count_is_reported() {
        let specs = vec![(
            PathBuf::from("specs/a.t27"),
            "fn booth_mul_u32(a: u32, b: u32) -> u32 {}\n".to_string(),
        )];
        let sc = scan_scope(&specs);
        let m = arity_mismatches(&sc, "specs/a.t27", "booth_mul_u32(a) != undefined");
        assert_eq!(m, vec![("booth_mul_u32".to_string(), 1, 2)], "{m:?}");
    }

    /// A name nothing in scope declares is the RESOLUTION column's question,
    /// not this one.
    #[test]
    fn an_undeclared_name_abstains() {
        let specs = vec![(
            PathBuf::from("specs/a.t27"),
            "fn g() -> u8 {}\n".to_string(),
        )];
        let sc = scan_scope(&specs);
        assert!(arity_mismatches(&sc, "specs/a.t27", "pow(2, 8) > 0").is_empty());
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
