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
    ACCEPTED.iter().any(|k| {
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
        failing += 1;
        let text = String::from_utf8_lossy(&o.stderr) + String::from_utf8_lossy(&o.stdout);
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
    println!("  ... the compiler cannot read        {failing}");
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
