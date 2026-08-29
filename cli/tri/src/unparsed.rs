//! Specs the compiler cannot read, ranked by the CONSTRUCT that stops it --
//! and every construct backed by a live probe.
//!
//! WHY A PROBE
//! -----------
//! The obvious census groups by the compiler's message, and that census was
//! shipped once and was wrong: `import x`, `algorithm y {`, `type T = T` and an
//! English sentence all print "unexpected token after expression statement".
//! The message names the state the parser recovered INTO, not what stopped it.
//!
//! Grouping by what the line CONTAINS is better and still not enough, because a
//! pattern can name a construct the compiler already supports. Measured the day
//! this was written, SIX candidates -- read off real failing lines, and every
//! one plausible -- compile in isolation:
//!
//!     @trim("x", "y")          builtin call             ACCEPTED
//!     .anthropic               enum literal             ACCEPTED
//!     if (c) { 1 } else { 2 }  if-expression            ACCEPTED
//!     *Foo   &Foo              pointer / reference      ACCEPTED
//!     []const u8               const-qualified slice    ACCEPTED
//!     for (s) |v| { }          capture in a for-loop    ACCEPTED
//!
//! An earlier fan-out named `[]const u8` as a cause. A census that repeated it
//! would have sent someone to implement a feature that is already there.
//!
//! So every construct here carries a MINIMAL SOURCE, and the census names it
//! only while the compiler rejects that source today. It is self-invalidating
//! on purpose: when a construct gains support its probe starts passing and the
//! row disappears without anyone editing a list.
//!
//! WHAT IT ABSTAINS ON
//! -------------------
//! A failing line carrying no probed construct, whose head is a construct the
//! parser accepts, is a SYMPTOM -- the cause is earlier in the file. Naming one
//! would be inventing it. Files under `fixtures/` are broken ON PURPOSE as
//! detector inputs and are counted on their own line, never as debt.
use anyhow::Result;
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum UnparsedCmd {
    /// Rank the constructs that stop the compiler, most specs first.
    Report {
        /// Name the specs under each construct.
        #[arg(long)]
        list: bool,
    },
    /// For each unreadable spec, the ITEM whose presence causes the failure.
    ///
    /// The compiler names a line. When that line is a construct it accepts, the
    /// cause is earlier, and this finds it by feeding growing prefixes of the
    /// module body back to the compiler. Every answer is checked by commenting
    /// the item out and demanding the reported line move; the ones that fail
    /// that check are reported as refuted rather than dropped.
    Locate {
        /// Also print the candidates causality refuted.
        #[arg(long)]
        refuted: bool,
    },
    /// Run every construct's minimal source and say which the compiler rejects.
    ///
    /// This is the census's own control. A row it names must fail here; a row
    /// that passes here is a feature the compiler has, and naming it would send
    /// someone to build what already exists.
    Probe,
}

/// A construct, the shape that spots it, and TWO minimal sources.
///
/// `probe` is what the compiler must reject for the row to be named.
/// `counter` is a near-identical source it must ACCEPT, and on which the
/// matcher must stay silent. The counter is what makes the row honest: without
/// it, `is_use` fired on every `use` line while `use a::b;` compiles fine and
/// only `use a::b as C;` does not. The probe alone could not catch that -- the
/// probe was the aliased form and it did fail.
///
/// `deliberate` names a refusal the repository DECIDED on, with its citation.
/// Such a row is not work; it is a position. Casts to `float` are refused
/// because no backend lowers float arithmetic, argued at length in
/// `bootstrap/src/compiler.rs` beside `VALID_CAST_TYPES`. A work queue that
/// listed it would send someone to undo a decision.
struct Construct {
    name: &'static str,
    probe: &'static str,
    counter: Option<&'static str>,
    deliberate: Option<&'static str>,
    matches: fn(&str) -> bool,
}

/// Types the cast operator already accepts. Measured: `1 as u32` compiles,
/// `1 as float` and `1 as gf16::GF16` do not -- the defect is the TARGET type,
/// and a rule keyed on `as` alone would name every cast in the tree.
const PRIMITIVE: [&str; 14] = [
    "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64", "bool", "str", "usize",
    "trit",
];

fn head(line: &str) -> &str {
    let t = line.trim();
    t.strip_prefix("pub ").unwrap_or(t)
}

/// `if (opt) |v| ..` -- a payload capture in an IF.
///
/// Not `|` alone: `1 | 2` compiles. Not a capture anywhere: `for (s) |v|`
/// compiles too. It is the if-expression form and only that.
fn is_if_capture(l: &str) -> bool {
    l.contains("if (") && l.contains(") |")
}

/// `for (xs, 0..) |v|` -- an open-ended range in a for-header.
fn is_for_range(l: &str) -> bool {
    l.contains("for (") && l.contains("..") && l.contains(") |")
}

/// ` as T` where T is not one of the primitives the cast already accepts.
fn is_cast_to_non_primitive(l: &str) -> bool {
    let Some(i) = l.find(" as ") else {
        return false;
    };
    let word: String = l[i + 4..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
        .collect();
    !word.is_empty() && !PRIMITIVE.contains(&word.as_str())
}

/// `fn name<T>(..)` -- type parameters on a FUNCTION.
///
/// Not a generic type in a signature: `fn a(k: Result<T, E>)` compiles. The
/// angle bracket has to sit between the name and the parameter list.
fn is_generic_fn(l: &str) -> bool {
    let Some(rest) = head(l).strip_prefix("fn ") else {
        return false;
    };
    match (rest.find('<'), rest.find('(')) {
        (Some(lt), Some(lp)) => lt < lp,
        _ => false,
    }
}

/// `[K: V]` -- a map TYPE. Not `[T]`, an array type, which compiles.
fn is_map_type(l: &str) -> bool {
    let b = l.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'[' {
            if let Some(off) = l[i..].find(']') {
                let inner = &l[i + 1..i + off];
                if inner.contains(':') && !inner.contains(',') && !inner.trim().is_empty() {
                    return true;
                }
                i += off;
            }
        }
        i += 1;
    }
    false
}

fn is_prototype(l: &str) -> bool {
    head(l).starts_with("fn ") && l.trim_end().ends_with(';')
}
fn is_tuple_struct(l: &str) -> bool {
    head(l).starts_with("struct ") && l.trim_end().ends_with(");")
}
fn is_pub_module(l: &str) -> bool {
    l.trim().starts_with("pub module ")
}
fn is_path_module(l: &str) -> bool {
    head(l).starts_with("module ") && head(l).contains("::")
}
fn is_import(l: &str) -> bool {
    head(l).starts_with("import ")
}
/// `use a::b as C;` -- an ALIASED use. Plain `use a::b;` and `using a::b;`
/// both compile, so a rule keyed on `use` alone names a feature that exists.
fn is_use(l: &str) -> bool {
    head(l).starts_with("use ") && l.contains(" as ")
}
fn is_trait(l: &str) -> bool {
    head(l).starts_with("trait ")
}
fn is_impl(l: &str) -> bool {
    head(l).starts_with("impl ")
}
fn is_algorithm(l: &str) -> bool {
    head(l).starts_with("algorithm ")
}
fn is_type_alias(l: &str) -> bool {
    head(l).starts_with("type ") && l.contains('=')
}
fn is_zig_block(l: &str) -> bool {
    l.trim_start().starts_with("\\\\")
}
// NO ROW FOR PROSE, and the reason is worth keeping.
//
// A paragraph where a declaration belongs does stop the parser -- the probe
// `module m { Some words here. fn a() ... }` is rejected. But acceptance is
// POSITION-DEPENDENT: `specs/api/sdk_contract.t27` parses while containing
//
//     fn random(dim: usize, seed: u64) -> Hypervector
//         Create random hypervector
//
// so the same words after a body-less signature are fine. A line-level matcher
// cannot be faithful to that, and the numbers said so before this was written:
// the loosest rule fired on 8925 lines inside specs that PARSE, the tightest
// still fired on 42 while losing 2 of the 5 real cases.
//
// `tri prose report` answers this correctly by asking the compiler line by
// line. A pattern was tried, measured, and dropped rather than shipped loose.

/// `pub use NAME;` -- visibility on a use. Plain `use NAME;` compiles.
fn is_pub_use(l: &str) -> bool {
    l.trim().starts_with("pub use ")
}

/// `fn(a: T) R { }` in expression position -- an anonymous function literal.
/// Not a declaration: `fn name(..)` has a name between `fn` and `(`.
fn is_fn_literal(l: &str) -> bool {
    l.contains("fn(")
}

/// `&[_]T{}` / `&[0]T{}` -- an anonymous array literal. `[ 1, 2 ]` compiles.
fn is_anon_array(l: &str) -> bool {
    let Some(i) = l.find("&[") else { return false };
    let rest = &l[i..];
    rest.contains("]{}") || rest.contains("]{ }") || (rest.contains(']') && rest.contains("{}"))
}

/// A statement terminated twice. One semicolon compiles.
fn is_double_semicolon(l: &str) -> bool {
    l.trim_end().ends_with(";;")
}

/// A closing brace at module level with nothing open.
fn is_stray_close(l: &str) -> bool {
    matches!(l.trim(), "}" | "};")
}

fn is_macro(l: &str) -> bool {
    let Some(i) = l.find("!(") else {
        return false;
    };
    l[..i]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// Ordered most specific first: a body-less `fn` prototype has to be matched
/// before anything keyed on `fn`, or the largest actionable row vanishes into
/// the abstention.
const CONSTRUCTS: &[Construct] = &[
    Construct {
        name: "fn NAME(..) -> T;    body-less prototype",
        probe: "fn a(x: u32) -> u32;\n",
        counter: Some("fn a(x: u32) -> u32 { return x; }\n"),
        deliberate: None,
        matches: is_prototype,
    },
    Construct {
        name: "struct NAME(T);      tuple / newtype struct",
        probe: "struct Id(str);\n",
        counter: Some("struct Id { x: str }\n"),
        deliberate: None,
        matches: is_tuple_struct,
    },
    Construct {
        name: "fn NAME<T>(..)       type parameters on a function",
        probe: "fn a<T>(k: T) -> u32 { return 1; }\n",
        counter: Some("fn a(k: Result<T, E>) -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_generic_fn,
    },
    Construct {
        name: "[K: V]               map type",
        probe: "fn a() -> u32 {\n    var m: [str: str] = { \"a\": \"b\" };\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    var m: [str] = [ \"a\" ];\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_map_type,
    },
    Construct {
        name: "if (o) |v| ..        payload capture in an if",
        probe: "fn a() -> u32 {\n    const x = if (o) |v| v else 0;\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    for (s) |v| { }\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_if_capture,
    },
    Construct {
        name: "for (xs, 0..) |v|    open-ended range in a for",
        probe: "fn a() -> u32 {\n    for (s, 0..) |op| { }\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    for (s) |v| { }\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_for_range,
    },
    Construct {
        name: "x as T               cast to a non-primitive type",
        probe: "fn a() -> u32 { return 1 as float; }\n",
        counter: Some("fn a() -> u32 { return 1 as u32; }\n"),
        deliberate: Some("no backend lowers float arithmetic -- see VALID_CAST_TYPES in bootstrap/src/compiler.rs"),
        matches: is_cast_to_non_primitive,
    },
    Construct {
        name: "pub module N;        visibility on a module",
        probe: "pub module test;\nfn a() -> u32 { return 1; }\n",
        counter: Some("module test;\nfn a() -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_pub_module,
    },
    Construct {
        name: "module a::b          path-qualified module name",
        probe: "module a::b {\n    fn x() -> u32 { return 1; }\n}\n",
        counter: None,
        deliberate: None,
        matches: is_path_module,
    },
    Construct {
        name: "import ..            import statement",
        probe: "module m {\n    import a::b;\n}\n",
        counter: None,
        deliberate: None,
        matches: is_import,
    },
    Construct {
        name: "use ..               use declaration",
        probe: "use a::b as C;\nfn a() -> u32 { return 1; }\n",
        counter: Some("use a::b;\nfn a() -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_use,
    },
    Construct {
        name: "trait NAME           trait declaration",
        probe: "trait T { fn a() -> u32; }\n",
        counter: None,
        deliberate: None,
        matches: is_trait,
    },
    Construct {
        name: "impl NAME            impl block",
        probe: "impl T { fn a() -> u32 { return 1; } }\n",
        counter: None,
        deliberate: None,
        matches: is_impl,
    },
    Construct {
        name: "algorithm NAME {     algorithm block",
        probe: "algorithm foo {\n    x: 1\n}\n",
        counter: None,
        deliberate: None,
        matches: is_algorithm,
    },
    Construct {
        name: "type T = U           type alias",
        probe: "type T = u32;\nfn a() -> u32 { return 1; }\n",
        counter: None,
        deliberate: None,
        matches: is_type_alias,
    },
    Construct {
        name: "\\\\ ...               Zig multiline string block",
        probe: "fn a() -> u32 {\n    const s =\n        \\\\pub fn test() {}\n    ;\n    return 1;\n}\n",
        counter: None,
        deliberate: None,
        matches: is_zig_block,
    },
    Construct {
        name: "pub use NAME;        visibility on a use declaration",
        probe: "pub use session_timeout;\nfn a() -> u32 { return 1; }\n",
        counter: Some("use session_timeout;\nfn a() -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_pub_use,
    },
    Construct {
        name: "fn(a: T) R { }       anonymous function literal",
        probe: "fn a() -> u32 {\n    const h = fn(e: E) void { };\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    const h = handler;\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_fn_literal,
    },
    Construct {
        name: "&[_]T{}              anonymous array literal",
        probe: "fn a() -> u32 {\n    const t = &[_][]const u8{};\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    const t = [ 1, 2 ];\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_anon_array,
    },
    Construct {
        name: ";;                   an empty statement after a semicolon",
        probe: "fn a() -> u32 {\n    print(\"hi\");;\n    return 1;\n}\n",
        counter: Some("fn a() -> u32 {\n    print(\"hi\");\n    return 1;\n}\n"),
        deliberate: None,
        matches: is_double_semicolon,
    },
    Construct {
        name: "};  at module level  a closing brace with nothing open",
        probe: "fn a() -> u32 { return 1; }\n};\n",
        counter: Some("fn a() -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_stray_close,
    },
    Construct {
        name: "name!(..)            Rust-style macro invocation",
        probe: "fn a() -> u32 { assert_eq!(1, 1); return 1; }\n",
        counter: Some("fn a() -> u32 { return 1; }\n"),
        deliberate: None,
        matches: is_macro,
    },
];

/// Constructs the parser accepts, at the top level or inside a body. A failing
/// line that starts with one of these and carries no probed construct is a
/// symptom; the cause is earlier in the file.
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

fn accepted_head(line: &str) -> bool {
    let t = head(line);
    is_assignment(line)
        || ACCEPTED.iter().any(|k| {
            t.strip_prefix(k)
                .is_some_and(|r| r.starts_with(|c: char| c.is_whitespace() || c == '('))
        })
}

/// A file under `fixtures/` is BROKEN ON PURPOSE -- the reference input for a
/// detector, not debt. `tools/specs_generate_baseline.txt` omits all of them; a
/// census that counts them disagrees with the repository's own ledger.
fn is_fixture(path: &str) -> bool {
    path.contains("/fixtures/")
}

/// Which STAGE refused the file.
///
/// The census used to call every non-zero `check` "the compiler cannot read".
/// Measured: of 97 such specs, 13 PARSE PERFECTLY and fail type checking, 4
/// die in the lexer on an unterminated string, and 1 is a semantic refusal.
/// Reading a construct off the failing line of a TYPE error is nonsense --
/// `specs/numeric/gf8.t27` stops at `exp = exp + 1;`, which parses fine and is
/// rejected as `cannot assign F64 to F32`.
///
/// The discriminator is checked both ways: no typecheck output contains a
/// parse word, and no parse output contains "Typecheck".
#[derive(PartialEq, Clone, Copy)]
enum Stage {
    Lex,
    Parse,
    Typecheck,
    Semantic,
}

fn stage_of(text: &str) -> Stage {
    if text.contains("Typecheck FAILED") {
        Stage::Typecheck
    } else if text.contains("unterminated string literal") {
        Stage::Lex
    } else if text.contains("parse error")
        || text.contains("Unexpected token")
        || text.contains("Unexpected top-level")
        || text.contains("Expected ")
    {
        Stage::Parse
    } else {
        Stage::Semantic
    }
}

/// `x = expr;` -- a plain assignment. It COMPILES (probed), so a failing line
/// that is one is a symptom like `return x;` is.
fn is_assignment(l: &str) -> bool {
    let t = l.trim();
    let Some(eq) = t.find('=') else { return false };
    if t[eq..].starts_with("==") || eq == 0 {
        return false;
    }
    let lhs = t[..eq].trim_end();
    !lhs.is_empty()
        && lhs
            .chars()
            .all(|c| c.is_alphanumeric() || "_.[]* ".contains(c))
        && lhs.starts_with(|c: char| c.is_alphabetic() || c == '_' || c == '*')
}

fn line_of(text: &str) -> Option<usize> {
    let i = text.find("line ")?;
    let rest = &text[i + 5..];
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Run every probe. `true` means the compiler REJECTS it today, so the row may
/// be named.
fn run_probes(t27c: &Path, root: &Path) -> Vec<bool> {
    let dir = std::env::temp_dir().join("tri-unparsed-probes");
    let _ = std::fs::create_dir_all(&dir);
    CONSTRUCTS
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let f = dir.join(format!("probe{i}.t27"));
            if std::fs::write(&f, c.probe).is_err() {
                return false;
            }
            let rejected = std::process::Command::new(t27c)
                .arg("check")
                .arg(&f)
                .current_dir(root)
                .output()
                .map(|o| !o.status.success())
                .unwrap_or(false);
            let _ = std::fs::remove_file(&f);
            rejected
        })
        .collect()
}

/// Run every counter. `true` means the compiler still ACCEPTS it, which is the
/// state the row's boundary depends on.
fn run_counters(t27c: &Path, root: &Path) -> Vec<bool> {
    let dir = std::env::temp_dir().join("tri-unparsed-counters");
    let _ = std::fs::create_dir_all(&dir);
    CONSTRUCTS
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let Some(src) = c.counter else { return true };
            let f = dir.join(format!("counter{i}.t27"));
            if std::fs::write(&f, src).is_err() {
                return true;
            }
            let ok = std::process::Command::new(t27c)
                .arg("check")
                .arg(&f)
                .current_dir(root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            let _ = std::fs::remove_file(&f);
            ok
        })
        .collect()
}

fn compiler(root: &Path) -> Result<PathBuf> {
    ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no compiler -- every row here is a claim about what it rejects,\n  \
                 and its absence is not a clean bill.\n  cargo build --release -p t27c"
            )
        })
}

pub fn run(cmd: &UnparsedCmd, root: PathBuf) -> Result<()> {
    let t27c = compiler(&root)?;

    if let UnparsedCmd::Locate { refuted } = cmd {
        let out = std::process::Command::new("git")
            .args(["ls-files", "*.t27"])
            .current_dir(&root)
            .output()?;
        let specs: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| root.join(s).is_file() && !is_fixture(s))
            .collect();
        let (mut found, mut refused, mut silent) = (Vec::new(), Vec::new(), 0usize);
        let (mut tc, mut lex, mut sem) = (0usize, 0usize, 0usize);
        for spec in &specs {
            let p = root.join(spec);
            let quick = std::process::Command::new(&t27c)
                .arg("check")
                .arg(&p)
                .current_dir(&root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(true);
            if quick {
                continue;
            }
            match locate_one(&t27c, &root, &p) {
                Located::Item(a, b, alone) => found.push((spec.clone(), a, b, alone)),
                Located::Refuted(a, b) => refused.push((spec.clone(), a, b)),
                Located::WrongStage(Stage::Typecheck) => tc += 1,
                Located::WrongStage(Stage::Lex) => lex += 1,
                Located::WrongStage(_) => sem += 1,
                Located::None(_) => silent += 1,
            }
        }
        let alone = found.iter().filter(|(_, _, _, a)| *a).count();
        println!("  located AND causally confirmed   {}", found.len());
        println!("  ... the item ALONE reproduces    {alone}  <- a minimal case, not a coordinate");
        println!("  candidate REFUTED by causality   {}", refused.len());
        println!("  nothing claimed                  {silent}");
        if tc + lex + sem > 0 {
            println!();
            println!("  not a PARSE failure, so not this command's question:");
            if tc > 0 {
                println!(
                    "      {tc:>4}  typecheck   (the error already names its line AND reason)"
                );
            }
            if lex > 0 {
                println!("      {lex:>4}  lex         (unterminated string)");
            }
            if sem > 0 {
                println!("      {sem:>4}  semantics");
            }
        }
        println!();
        println!("  A confirmed item is one whose removal MOVES the reported error.");
        println!("  Prefix bisection alone is unsound -- a truncated prefix can fail");
        println!("  for a reason the whole file does not have -- so a candidate that");
        println!("  does not survive that check is not an answer.");
        if !found.is_empty() {
            println!();
            for (s, a, b, alone) in found.iter().take(40) {
                let span = if b > a {
                    format!("..{b}")
                } else {
                    String::new()
                };
                let note = if *alone { "" } else { "   (only in context)" };
                println!("      {s}:{a}{span}{note}");
            }
            if found.len() > 40 {
                println!("      ... and {} more", found.len() - 40);
            }
        }
        if *refuted && !refused.is_empty() {
            println!();
            println!("  refuted candidates -- the prefix failed, the item is innocent");
            for (s, a, b) in refused.iter() {
                println!("      {s}:{a}..{b}");
            }
        }
        return Ok(());
    }

    if matches!(cmd, UnparsedCmd::Probe) {
        let rejected = run_probes(&t27c, &root);
        let counters = run_counters(&t27c, &root);
        let n = rejected.iter().filter(|r| **r).count();
        let bad_counters: Vec<&str> = CONSTRUCTS
            .iter()
            .zip(&counters)
            .filter(|(c, ok)| c.counter.is_some() && !**ok)
            .map(|(c, _)| c.name)
            .collect();
        println!("  constructs probed              {}", CONSTRUCTS.len());
        println!("  ... the compiler REJECTS       {n}");
        println!("  ... the compiler ACCEPTS       {}", CONSTRUCTS.len() - n);
        println!(
            "  counters that still compile    {} of {}",
            counters
                .iter()
                .zip(CONSTRUCTS)
                .filter(|(ok, c)| **ok && c.counter.is_some())
                .count(),
            CONSTRUCTS.iter().filter(|c| c.counter.is_some()).count()
        );
        println!();
        for (c, r) in CONSTRUCTS.iter().zip(&rejected) {
            let tag = if *r { "rejected" } else { "ACCEPTED" };
            let note = match c.deliberate {
                Some(_) => "  <- refused ON PURPOSE, not work",
                None => "",
            };
            println!("      {tag}  {}{note}", c.name);
        }
        if !bad_counters.is_empty() {
            println!();
            println!("  A COUNTER NO LONGER COMPILES. The row claims the compiler accepts");
            println!("  a near-identical source, and it does not -- so the row's boundary");
            println!("  is wrong, not the compiler:");
            for b in &bad_counters {
                println!("      {b}");
            }
            return Err(anyhow::anyhow!(
                "{} counter(s) stopped compiling",
                bad_counters.len()
            ));
        }
        if n < CONSTRUCTS.len() {
            println!();
            println!("  An ACCEPTED row is a construct this repository already supports.");
            println!("  `report` will not name it, whatever the failing line looks like:");
            println!("  naming it would send someone to build what is already there.");
        }
        return Ok(());
    }

    let UnparsedCmd::Report { list } = cmd else {
        unreachable!()
    };
    let rejected = run_probes(&t27c, &root);
    let live: Vec<&Construct> = CONSTRUCTS
        .iter()
        .zip(&rejected)
        .filter(|(_, r)| **r)
        .map(|(c, _)| c)
        .collect();
    let retired = CONSTRUCTS.len() - live.len();

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
    let (mut upstream, mut fixtures, mut unlocated, mut failing) = (0usize, 0usize, 0usize, 0usize);
    let mut unnamed: Vec<(String, String)> = Vec::new();
    let (mut lex, mut typecheck, mut semantic) = (0usize, 0usize, 0usize);

    for spec in &specs {
        let Ok(o) = std::process::Command::new(&t27c)
            .arg("check")
            .arg(spec)
            .current_dir(&root)
            .output()
        else {
            continue;
        };
        if o.status.success() {
            continue;
        }
        if is_fixture(spec) {
            fixtures += 1;
            continue;
        }
        let text = String::from_utf8_lossy(&o.stderr) + String::from_utf8_lossy(&o.stdout);
        match stage_of(&text) {
            Stage::Lex => {
                lex += 1;
                continue;
            }
            Stage::Typecheck => {
                typecheck += 1;
                continue;
            }
            Stage::Semantic => {
                semantic += 1;
                continue;
            }
            Stage::Parse => {}
        }
        failing += 1;
        let (Some(n), Ok(src)) = (line_of(&text), std::fs::read_to_string(root.join(spec))) else {
            unlocated += 1;
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        if n == 0 || n > lines.len() {
            unlocated += 1;
            continue;
        }
        let line = lines[n - 1];
        match live.iter().find(|c| (c.matches)(line)) {
            Some(c) => by.entry(c.name).or_default().push(spec.clone()),
            None if accepted_head(line) => upstream += 1,
            None => unnamed.push((spec.clone(), line.trim().chars().take(52).collect())),
        }
    }

    let named: usize = by.values().map(|v| v.len()).sum();
    println!("  specs tracked                       {}", specs.len());
    println!("  ... refused at PARSE                {failing}   <- this census");
    if typecheck > 0 {
        println!("  ... refused at TYPECHECK            {typecheck}   (they parse; a construct");
        println!("                                            read off their failing line");
        println!("                                            would be nonsense)");
    }
    if lex > 0 {
        println!("  ... refused at LEX                  {lex}   (unterminated string)");
    }
    if semantic > 0 {
        println!("  ... refused on SEMANTICS            {semantic}");
    }
    println!("  ... construct NAMED and PROBED      {named}");
    println!("  ... cause is UPSTREAM, not named    {upstream}");
    println!("  ... not decided, nothing claimed    {}", unnamed.len());
    if unlocated > 0 {
        println!("  ... error names no readable line    {unlocated}");
    }
    if fixtures > 0 {
        println!("  broken ON PURPOSE under fixtures/   {fixtures}  (detector inputs, not debt)");
    }
    if retired > 0 {
        println!("  constructs the compiler now ACCEPTS {retired}  (probed; not named)");
    }

    if by.is_empty() {
        println!();
        println!("  No failing line carries a construct whose probe still fails.");
        return Ok(());
    }

    let deliberate: std::collections::BTreeMap<&str, &str> = CONSTRUCTS
        .iter()
        .filter_map(|c| c.deliberate.map(|d| (c.name, d)))
        .collect();
    let mut rows: Vec<(&&str, &Vec<String>)> = by.iter().collect();
    rows.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    let (work, decided): (Vec<_>, Vec<_>) = rows
        .iter()
        .partition(|(k, _)| !deliberate.contains_key(**k));

    println!();
    println!("  work queue -- every row proved unsupported by its own probe");
    for (k, v) in &work {
        println!("      {:>4}  {k}", v.len());
        if *list {
            for s in v.iter() {
                println!("            {s}");
            }
        }
    }

    if !decided.is_empty() {
        println!();
        println!("  refused ON PURPOSE -- a position, not a gap. Listed so it is not");
        println!("  mistaken for work, and so the reason is at hand when it is revisited.");
        for (k, v) in &decided {
            println!("      {:>4}  {k}", v.len());
            println!("            {}", deliberate[**k]);
            if *list {
                for s in v.iter() {
                    println!("            {s}");
                }
            }
        }
    }

    if *list && !unnamed.is_empty() {
        println!();
        println!("  not decided -- the census's own blind spot, printed so it can be");
        println!("  closed rather than carried. Each of these needs a probe and a");
        println!("  counter before it may become a row.");
        for (sp, l) in unnamed.iter() {
            println!("      {sp}");
            println!("            {l}");
        }
    }

    println!();
    println!("  The UPSTREAM count is not a residue to be reduced: those lines are");
    println!("  `fn`, `struct`, `const` carrying no probed construct. The defect is");
    println!("  earlier in the file and this census will not guess it.");
    println!();
    println!("  `tri unparsed probe` runs the minimal source behind each row.");
    if !*list {
        println!("  `--list` names the specs under each row.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each negative here is a MEASUREMENT: the construct compiles in isolation,
    // so a matcher that fires on it would name a feature the compiler has.
    #[test]
    fn bitwise_or_is_not_a_capture() {
        assert!(!is_if_capture("    return 1 | 2;"));
        assert!(is_if_capture("    const x = if (o) |v| v else 0;"));
    }

    #[test]
    fn a_capture_in_a_for_loop_is_not_the_if_form() {
        // `for (s) |v| { }` compiles; only the if-expression form does not.
        assert!(!is_if_capture("    for (s) |v| { }"));
        assert!(!is_for_range("    for (s) |v| { }"));
        assert!(is_for_range("    for (delta.operations, 0..) |op| {"));
    }

    #[test]
    fn a_primitive_cast_is_not_named() {
        // `1 as u32` compiles; `1 as float` and `1 as gf16::GF16` do not.
        assert!(!is_cast_to_non_primitive("return 1 as u32;"));
        assert!(is_cast_to_non_primitive("return 1 as float;"));
        assert!(is_cast_to_non_primitive("return x as gf16::GF16;"));
    }

    #[test]
    fn a_generic_type_in_a_signature_is_not_a_generic_function() {
        // `fn a(k: Result<T, E>)` compiles; `fn a<T>(k: T)` does not.
        assert!(!is_generic_fn("fn a(k: Result<T, E>) -> u32 {"));
        assert!(is_generic_fn("fn read<T>(key: [str]) -> Result<T, E> {"));
    }

    #[test]
    fn an_array_type_is_not_a_map_type() {
        // `[str]` compiles; `[str: str]` does not.
        assert!(!is_map_type("    var m: [str] = [ \"a\" ];"));
        assert!(is_map_type(
            "    var env: [str: str] = { \"PATH\": \"/usr/bin\" };"
        ));
        // A list of two things is not a key-value pair.
        assert!(!is_map_type("    const v = arr[a, b];"));
    }

    #[test]
    fn a_prototype_is_named_before_the_abstention_swallows_it() {
        assert!(is_prototype("pub fn poll(x: [str]) -> Result;"));
        assert!(accepted_head("pub fn poll(x: [str]) -> Result;"));
        assert!(!is_prototype("pub fn poll(x: u32) -> bool {"));
    }

    #[test]
    fn a_sentence_containing_a_keyword_is_not_that_keyword() {
        assert!(!is_import("importantly, the bridge is read-only."));
        assert!(!is_type_alias("typed values flow through the VM."));
        assert!(!accepted_head("constant folding is described here"));
    }

    #[test]
    fn a_macro_needs_a_name_in_front_of_the_bang() {
        assert!(is_macro("assert_eq!(a, b);"));
        assert!(!is_macro("if x != (a) {"));
    }

    #[test]
    fn a_fixture_is_not_debt() {
        assert!(is_fixture(
            "bootstrap/tests/fixtures/damage/damage_class_01.t27"
        ));
        assert!(!is_fixture("specs/github/auth.t27"));
        assert!(!is_fixture("specs/tools/fixtures_report.t27"));
    }

    #[test]
    fn a_statement_keyword_abstains_like_a_top_level_one() {
        for l in ["return x;", "let y = 1;", "    for (a) |b| {"] {
            assert!(accepted_head(l), "{l}");
        }
    }

    // The stage split, both directions. Measured on 97 refusals: no typecheck
    // output contains a parse word and no parse output contains "Typecheck".
    #[test]
    fn the_stage_split_reads_both_ways() {
        assert!(matches!(
            stage_of("Typecheck FAILED (4 errors, 16 warnings):\n  - type mismatch at line 168"),
            Stage::Typecheck
        ));
        assert!(matches!(
            stage_of("Error: parse error at module level near line 2: Unexpected token"),
            Stage::Parse
        ));
        assert!(matches!(
            stage_of("Error: unterminated string literal opened at line 148:13"),
            Stage::Lex
        ));
        assert!(matches!(
            stage_of("Error: nested fn 'inner' inside 'outer' captures enclosing locals"),
            Stage::Semantic
        ));
    }

    // `x = x + 1;` compiles, so a failing line that is one is a symptom. Seven
    // gf* specs sat in "not decided" on exactly that line -- and their real
    // failure was a TYPE error, not a parse error at all.
    #[test]
    fn an_assignment_abstains() {
        for l in ["exp = exp + 1;", "    x = x / 2.0;", "self.pos = 0;"] {
            assert!(is_assignment(l), "{l}");
            assert!(accepted_head(l), "{l}");
        }
        assert!(!is_assignment("    if (a == b) {"));
        assert!(!is_assignment("= 1;"));
    }

    #[test]
    fn the_three_new_rows_have_the_boundary_right() {
        // `[ 1, 2 ]` compiles; `&[_]T{}` does not.
        assert!(is_anon_array("    const t = &[_][]const u8{};"));
        assert!(is_anon_array("    .exports = &[0][]const u8{},"));
        assert!(!is_anon_array("    const t = [ 1, 2 ];"));
        // one semicolon compiles, two do not.
        assert!(is_double_semicolon("    print(\"hi\");;"));
        assert!(!is_double_semicolon("    print(\"hi\");"));
        // a brace with nothing open.
        assert!(is_stray_close("};"));
        assert!(is_stray_close("  }"));
        assert!(!is_stray_close("} else {"));
        // `use X;` compiles; `pub use X;` does not.
        assert!(is_pub_use("pub use session_timeout;"));
        assert!(!is_pub_use("use session_timeout;"));
        // a named declaration is not a literal.
        assert!(is_fn_literal("    .on_message = fn(e: SSEEvent) void { },"));
        assert!(!is_fn_literal("pub fn a(x: u32) -> u32 {"));
    }

    // Three bugs lived in this function, each found by reading an ANSWER and
    // not by any control. Braces inside a comment or a string are not braces.
    #[test]
    fn depth_ignores_comments_and_strings() {
        assert_eq!(
            code_only(&["fn a() { // } not a brace"])[0].trim(),
            "fn a() {"
        );
        // The quotes go too; what matters is that no brace survives.
        let stripped = code_only(&["let s = \"{ { {\";"]).remove(0);
        assert!(!stripped.contains('{'), "{stripped}");
        // A block comment carrying JSON moved every boundary in a 500-line file.
        let src = ["/*", "{ \"a\": 1 }", "*/", "fn a() { }"];
        let d = depths(&src);
        assert_eq!(d[2], 0, "the JSON brace must not count");
        assert_eq!(*d.last().unwrap(), 0);
    }

    #[test]
    fn a_string_with_a_brace_does_not_open_a_block() {
        let d = depths(&["const s = \"}\";", "fn a() { }"]);
        assert_eq!(d[0], 0);
        assert_eq!(d[1], 0);
    }

    // The module closer is found by DEPTH, not by matching the text `}`.
    // A backward text scan hit a nested `};` and made the tail 40 lines of
    // orphan code, which failed for its own reason on every prefix.
    #[test]
    fn the_module_closer_is_found_by_depth() {
        let src = [
            "module m {",
            "    fn a() {",
            "        x();",
            "    };",
            "    fn b() { }",
            "}",
        ];
        let (start, end) = split_module(&src);
        assert_eq!(start, 1);
        assert_eq!(end, 5, "the closer is the last line, not the nested `}};`");
    }

    // `report` learned the stage split and `locate` did not: 8 of its first 40
    // answers were typecheck failures. A type error already names its line AND
    // its reason, so a bisection has nothing to add.
    #[test]
    fn locate_answers_only_for_parse_failures() {
        assert!(stage_of("Typecheck FAILED (1 errors, 0 warnings):") != Stage::Parse);
        assert!(stage_of("Error: unterminated string literal opened at line 5:1") != Stage::Parse);
        assert!(stage_of("Error: parse error at module level near line 2") == Stage::Parse);
    }

    // `module NAME;` wraps nothing, so the body is everything after it. Taking
    // the first braced line as a wrapper instead made the rest of the file a
    // "tail" that was glued onto every truncated prefix -- 32 of 37 refuted
    // answers were that, against 4 of 37 confirmed.
    #[test]
    fn a_semicolon_module_wraps_nothing() {
        let src = [
            "// header",
            "module config-load;",
            "struct A { x: u32 }",
            "fn b() -> u32 { return 1; }",
        ];
        let (start, end) = split_module(&src);
        assert_eq!(start, 2, "body starts after the `module NAME;` line");
        assert_eq!(end, 4, "and runs to the end -- there is no closing brace");
    }

    #[test]
    fn a_braced_module_still_splits_at_its_brace() {
        let src = ["module m {", "    fn a() { }", "}"];
        assert_eq!(split_module(&src), (1, 2));
    }

    // A file with no module wrapper still has a body: the whole file.
    #[test]
    fn split_module_handles_a_bare_file() {
        let src = ["fn a() { }", "fn b() { }"];
        let (start, end) = split_module(&src);
        assert_eq!((start, end), (0, 2));
    }

    // Every probe must be distinct: two rows sharing a source would report the
    // same reading twice and hide one of them.
    #[test]
    fn every_probe_is_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for c in CONSTRUCTS {
            assert!(seen.insert(c.probe), "duplicate probe: {}", c.name);
        }
    }

    // A matcher must fire on its own probe, or the row can never be named.
    #[test]
    fn every_matcher_fires_on_its_own_probe() {
        for c in CONSTRUCTS {
            let hit = c.probe.lines().any(|l| (c.matches)(l));
            assert!(hit, "matcher never fires on its own probe: {}", c.name);
        }
    }

    // ...and must stay SILENT on the counter, which the compiler accepts.
    //
    // This is the half the probe cannot check. `is_use` fired on every `use`
    // line while only the aliased form fails; its probe WAS the aliased form,
    // so the probe passed and the matcher was still wrong.
    #[test]
    fn no_matcher_fires_on_its_counter() {
        for c in CONSTRUCTS {
            let Some(ctr) = c.counter else { continue };
            for l in ctr.lines() {
                assert!(
                    !(c.matches)(l),
                    "matcher fires on a source the compiler ACCEPTS: {} -- {l}",
                    c.name
                );
            }
        }
    }

    // A deliberate refusal must cite where the decision is written down.
    #[test]
    fn a_deliberate_refusal_carries_its_citation() {
        for c in CONSTRUCTS {
            if let Some(why) = c.deliberate {
                assert!(
                    why.contains(".rs") || why.contains(".md"),
                    "refusal without a citation: {}",
                    c.name
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// LOCATE: the item whose presence causes the failure, when it is not the line
// the compiler names.
//
// Method: the file is `module X { <items> }`. Feed the compiler
// `module X { <first k items> }` for increasing k and binary-search the first k
// that fails. The last item added is the suspect.
//
// It is UNSOUND on its own, and the numbers say by how much: of 45 files it
// located, a causality check REFUTED 16. A truncated prefix can fail for a
// reason the whole file does not have -- later declarations the tail refers to
// are simply absent -- so "first failing prefix" is a candidate, never an
// answer.
//
// Two controls, and only the second one bites:
//
//   fidelity  -- head+body+tail must reproduce the original failure, same line.
//                Passed 46 of 46 while the answer was wrong for 16 of them: the
//                concatenation is the whole file no matter where the
//                boundaries fall, so this control cannot see a bad split.
//   causality -- comment the located item out; the reported error line must
//                CHANGE. This is the one that found the 16.
//
// The first version of the causality check asked whether the error moved PAST
// the item. For an item at line 5 and an error at line 2215 that is true by
// arithmetic, and it passed 45 of 45. A control that cannot fail is not a
// control.
// ---------------------------------------------------------------------------

/// Source with comments and string bodies blanked, so brace depth is countable.
///
/// Three bugs lived here, each found by reading an answer rather than by a
/// control: line comments (`//`, `#`), the module's closing brace located by
/// TEXT rather than by depth, and `/* */` blocks -- one of which wrapped a JSON
/// schema whose `{` moved every boundary in the file.
fn code_only(lines: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(lines.len());
    let mut in_block = false;
    for l in lines {
        let b: Vec<char> = l.chars().collect();
        let (mut buf, mut i, mut in_str, mut esc) = (String::new(), 0usize, false, false);
        while i < b.len() {
            let c = b[i];
            if in_block {
                if c == '*' && i + 1 < b.len() && b[i + 1] == '/' {
                    in_block = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            if in_str {
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == '"' {
                    in_str = false;
                }
                i += 1;
                continue;
            }
            match c {
                '"' => {
                    in_str = true;
                    i += 1;
                }
                '/' if i + 1 < b.len() && b[i + 1] == '*' => {
                    in_block = true;
                    i += 2;
                }
                '/' if i + 1 < b.len() && b[i + 1] == '/' => break,
                '#' => break,
                _ => {
                    buf.push(c);
                    i += 1;
                }
            }
        }
        out.push(buf);
    }
    out
}

fn depths(lines: &[&str]) -> Vec<i32> {
    let mut d = 0i32;
    code_only(lines)
        .iter()
        .map(|c| {
            for ch in c.chars() {
                match ch {
                    '{' | '(' | '[' => d += 1,
                    '}' | ')' | ']' => d -= 1,
                    _ => {}
                }
            }
            d
        })
        .collect()
}

fn check_text(t27c: &Path, root: &Path, text: &str) -> (bool, Option<usize>) {
    let f = std::env::temp_dir().join("tri-locate-probe.t27");
    if std::fs::write(&f, format!("{text}\n")).is_err() {
        return (false, None);
    }
    let out = std::process::Command::new(t27c)
        .arg("check")
        .arg(&f)
        .current_dir(root)
        .output();
    let _ = std::fs::remove_file(&f);
    match out {
        Ok(o) => {
            let msg = String::from_utf8_lossy(&o.stderr) + String::from_utf8_lossy(&o.stdout);
            (o.status.success(), line_of(&msg))
        }
        Err(_) => (false, None),
    }
}

/// `module X {` header, its body, and its closing brace.
fn split_module(lines: &[&str]) -> (usize, usize) {
    let d = depths(lines);
    let c = code_only(lines);
    // `module NAME;` -- the SEMICOLON form, which wraps nothing. The scan below
    // looks for the first line that opens a brace at depth 1 and calls it the
    // module header; in a semicolon-form file that is the first `struct` or
    // `fn`, so everything after its closing brace became the "tail" and the
    // reconstruction glued a large orphan chunk onto a truncated body. Every
    // prefix then failed for the chunk's own reasons.
    //
    // Measured before the fix, with the base rate that makes it mean something:
    //
    //             tail > 10 lines   tail = 1 line
    //   refuted        32                4
    //   confirmed       4               33
    //
    // A one-line tail is the closing brace of a real braced module. Anything
    // longer was this.
    for i in 0..lines.len() {
        let t = c[i].trim();
        let t = t.strip_prefix("pub ").unwrap_or(t);
        if t.starts_with("module ") && t.ends_with(';') && d[i] == 0 {
            return (i + 1, lines.len());
        }
    }
    for i in 0..lines.len() {
        if d[i] == 1 && c[i].contains('{') {
            for j in i + 1..lines.len() {
                if d[j] == 0 {
                    return (i + 1, j);
                }
            }
            return (i + 1, lines.len());
        }
    }
    (0, lines.len())
}

enum Located {
    /// Item [a, b] (1-based, inclusive), causality confirmed. The flag says
    /// whether the item ALONE -- wrapped in a bare module -- reproduces a
    /// failure, which is the difference between "here is your bug, in four
    /// lines" and "here is where it starts".
    Item(usize, usize, bool),
    /// A candidate the causality check refuted, with the line it named.
    Refuted(usize, usize),
    /// The file does not fail at PARSE, so there is no item to find.
    ///
    /// `report` learned this and `locate` did not: 8 of its first 40 answers
    /// were typecheck failures, where "the item whose presence causes the
    /// failure" is a category error. A type error already names its line AND
    /// its reason -- `cannot assign F64 to F32` -- so there is nothing for a
    /// bisection to add.
    WrongStage(Stage),
    /// Nothing claimed, and why.
    None(&'static str),
}

fn locate_one(t27c: &Path, root: &Path, path: &Path) -> Located {
    let Ok(src) = std::fs::read_to_string(path) else {
        return Located::None("unreadable");
    };
    let lines: Vec<&str> = src.lines().collect();
    let (ok0, orig) = check_text(t27c, root, &src);
    if ok0 {
        return Located::None("parses");
    }
    let Some(orig) = orig else {
        return Located::None("error names no line");
    };
    // Same stage split `report` uses. Without it this command answers a
    // question about parsing with evidence from type checking.
    let stage = {
        let out = std::process::Command::new(t27c)
            .arg("check")
            .arg(path)
            .current_dir(root)
            .output();
        match out {
            Ok(o) => {
                stage_of(&(String::from_utf8_lossy(&o.stderr) + String::from_utf8_lossy(&o.stdout)))
            }
            Err(_) => Stage::Parse,
        }
    };
    if stage != Stage::Parse {
        return Located::WrongStage(stage);
    }
    let (bstart, bend) = split_module(&lines);
    if bstart == 0 || bend <= bstart {
        return Located::None("no module body");
    }
    let head = &lines[..bstart];
    let body = &lines[bstart..bend];
    let tail = &lines[bend..];

    // FIDELITY -- necessary, and far from sufficient; see the note above.
    let whole = format!(
        "{}\n{}\n{}",
        head.join("\n"),
        body.join("\n"),
        tail.join("\n")
    );
    let (ok, line) = check_text(t27c, root, &whole);
    if ok || line != Some(orig) {
        return Located::None("the split does not reproduce the failure");
    }

    let bd = depths(body);
    let bounds: Vec<usize> = bd
        .iter()
        .enumerate()
        .filter(|(_, x)| **x <= 0)
        .map(|(i, _)| i + 1)
        .collect();
    if bounds.len() < 2 {
        return Located::None("fewer than two items");
    }
    let pref = |k: usize| -> bool {
        let t = format!(
            "{}\n{}\n{}",
            head.join("\n"),
            body[..bounds[k]].join("\n"),
            tail.join("\n")
        );
        check_text(t27c, root, &t).0
    };
    let (mut lo, mut hi) = (0usize, bounds.len() - 1);
    if pref(hi) {
        return Located::None("every prefix parses");
    }
    let (a, b) = if !pref(lo) {
        (bstart + 1, bstart + bounds[0])
    } else {
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if pref(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (bstart + bounds[lo] + 1, bstart + bounds[hi])
    };

    // CAUSALITY -- comment the item out and demand the error line CHANGE.
    let muted: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if a <= i + 1 && i + 1 <= b {
                format!("// {l}")
            } else {
                (*l).to_string()
            }
        })
        .collect();
    // PROGRESS, not merely change. Commenting a block-comment opener changes
    // the error too -- it breaks the file further. Confirmed means the file
    // parses, or the new error is LATER than the original. The comparison is
    // against the ORIGINAL error line, never against the item, so it cannot be
    // satisfied by arithmetic the way "the error moved past the item" was.
    let (parsed, moved) = check_text(t27c, root, &muted.join("\n"));
    let progressed = parsed || moved.is_some_and(|m| m > orig);
    if !progressed {
        return Located::Refuted(a, b);
    }
    // Does the item alone reproduce? Measured over the first 37 confirmed
    // answers: all 37 did. That is what makes the output a set of minimal
    // reproducers rather than a set of coordinates.
    let item = lines[a - 1..b.min(lines.len())].join("\n");
    let (alone_ok, _) = check_text(t27c, root, &format!("module m {{\n{item}\n}}"));
    Located::Item(a, b, !alone_ok)
}
