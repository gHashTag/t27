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
        && x.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_') == Some(true)
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
    !["==", "!=", ">=", "<=", "&&", "||", " > ", " < ", " + ", " = "]
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
            let (notation, binder_text) = if let Some(idx) = t.find("forall ") {
                ("forall", t[idx + 7..].to_string())
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
            parse_binders("a b : i32, a <= b, verilog_eval_problems(a) <= verilog_eval_problems(b)"),
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
