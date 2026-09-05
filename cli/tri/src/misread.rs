//! `tri misread` — the specs the compiler reads WRONGLY, which no census covers.
//!
//! `tri unparsed` ranks the constructs that stop the parser and its population is
//! defined by *the compiler refused*. Everything the compiler accepted is outside
//! that census by construction, however wrong the result. This command asks the
//! other question, and it asks it of the GENERATED OUTPUT rather than of the
//! parser, because that is where being read wrongly becomes visible.
//!
//! The class is real and it was invisible. Measured on the corpus at #3225:
//! fourteen specs pass `parse`, pass `typecheck`, and emit a struct field with no
//! type at all; three appear in the debt ledger for other reasons and ELEVEN were
//! tracked by nothing. A four-line spec carries the whole mechanism:
//!
//! ```text
//! module probe { pub const Thing = struct { ok : u8, bad : 0, }; }
//! ```
//!
//! An integer literal sits in TYPE position. `parse` accepts it, `typecheck`
//! accepts it, the Rust backend writes `pub bad: 0,` and the C backend writes
//! `0 bad;`, both unparseable in their languages.
//!
//! The Zig backend is where it disappears, and not by dropping anything: it
//! writes `bad: 0,` and `empty: void`, and BOTH readings accept them --
//! `zig build-obj` and the deeper `zig test --test-no-exec` alike. So the Zig
//! column of the corpus counts these specs as generating AND accepting, and no
//! shape read from Zig output would flag the `void` one, since `void` is a
//! legitimate Zig type. That is why the shapes below are read from Rust and C.
//!
//! ERROR RECOVERY IS WHAT HIDES IT. A list-valued declaration the parser does not
//! implement has its items recovered as fields, so the spec comes out the other
//! side looking well-formed and every gate is green on it.
//!
//! THE POSITIVE CONTROL IS NOT OPTIONAL. A detector that finds nothing and a
//! detector that is broken print the same thing, and this whole command exists
//! because a green reading was not a result. So it runs the reproducer above
//! through the real compiler first and REFUSES to report on the corpus unless
//! every shape it claims to detect actually fired on a case known to contain it.

use anyhow::{Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args, Debug)]
pub struct Misread {
    /// Root of the spec tree.
    #[arg(long, default_value = "specs")]
    specs_dir: String,
    /// Name every spec under each shape rather than only counting.
    #[arg(long)]
    list: bool,
}

/// One way a generated file can be meaningless while every stage stayed green.
///
/// Named by what the OUTPUT shows, not by a guess at the spec's intent: the
/// intent is exactly what was lost, and naming it here would be inventing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// `pub field: ,` — the type slot is empty.
    RustEmptyType,
    /// `pub field: 0,` — an integer literal stands where a type must.
    RustLiteralType,
    /// `Vec<>` — a generic whose parameter was dropped.
    RustEmptyGeneric,
    /// `0 field;` — the C half of the literal-in-type-position defect.
    CLiteralType,
    /// `pub a: Floatb:String,` — a field whose TYPE carries a bare colon.
    ///
    /// Named for what the output SHOWS, not for a cause: measured on this corpus
    /// the colon arrives two ways, and the shape cannot tell them apart.
    RustColonInType,
}

impl Shape {
    pub fn label(self) -> &'static str {
        match self {
            Shape::RustEmptyType => "rust: `pub f: ,`      field with no type",
            Shape::RustLiteralType => "rust: `pub f: 0,`     literal in type position",
            Shape::RustEmptyGeneric => "rust: `Vec<>`         generic lost its parameter",
            Shape::CLiteralType => "c:    `0 f;`          literal in type position",
            Shape::RustColonInType => "rust: `pub a: Xb:Y,`  bare colon inside a type",
        }
    }

    /// Which backend's output this shape is read from.
    pub fn backend(self) -> &'static str {
        match self {
            Shape::CLiteralType => "gen-c",
            _ => "gen-rust",
        }
    }

    pub fn all() -> [Shape; 5] {
        [
            Shape::RustEmptyType,
            Shape::RustLiteralType,
            Shape::RustEmptyGeneric,
            Shape::CLiteralType,
            Shape::RustColonInType,
        ]
    }
}

/// Does this line of generated Rust declare a field whose type slot is empty?
///
/// Anchored on the whole line rather than on a substring: `pub f: ,` inside a
/// comment or a string would otherwise count, and a census that counts prose is
/// the defect this repository names most often.
pub fn rust_empty_type(line: &str) -> bool {
    let t = line.trim();
    let Some(rest) = t.strip_prefix("pub ") else {
        return false;
    };
    let Some((name, ty)) = rest.split_once(':') else {
        return false;
    };
    is_ident(name.trim()) && ty.trim() == ","
}

/// Does this line declare a field whose type is an integer literal?
///
/// `pub bad: 0,` — accepted by `parse` and by `typecheck`, and unparseable as
/// Rust. A negative literal counts: the sign does not make it a type.
pub fn rust_literal_type(line: &str) -> bool {
    let t = line.trim();
    let Some(rest) = t.strip_prefix("pub ") else {
        return false;
    };
    let Some((name, ty)) = rest.split_once(':') else {
        return false;
    };
    if !is_ident(name.trim()) {
        return false;
    }
    let ty = ty.trim().trim_end_matches(',').trim();
    is_int_literal(ty)
}

/// `Vec<>` — the element type was dropped on the way in.
pub fn rust_empty_generic(line: &str) -> bool {
    line.contains("Vec<>")
}

/// Does this line declare a field whose TYPE contains a bare colon?
///
/// A Rust type never does: a path spells its separator `::` and no other type
/// syntax admits a lone `:`. So the line is wreckage either way -- but the shape
/// does NOT say which wreck, and naming it for one cause was wrong. Measured on
/// this corpus it arrives two ways:
///
///   * a field swallowed by the one before it. An inline `#` comment on a field
///     consumes the rest of the line AND the next declaration, so
///     `id : U8  # note,` followed by three fields yields the single field
///     `pub id: U8command:Stringargs:Stringgroup_id:U8status:JobStatus`. Four
///     declarations gone, and `parse` and `typecheck` both accept the spec.
///   * a map type the emitter cannot spell. `[str: str]` comes out as
///     `Vec<str:str>`, which is a different defect with the same footprint.
///
/// The detector reports the shape and leaves the cause to whoever reads the spec,
/// which is the rule this module states at the top and which I broke by calling
/// this `RustSwallowedField` first.
pub fn rust_colon_in_type(line: &str) -> bool {
    let t = line.trim();
    let Some(rest) = t.strip_prefix("pub ") else {
        return false;
    };
    let Some((name, ty)) = rest.split_once(':') else {
        return false;
    };
    if !is_ident(name.trim()) {
        return false;
    }
    let ty = ty.trim().trim_end_matches(',');
    // Strip every `::` before looking for a lone `:`, so a legitimate path type
    // cannot be reported.
    let without_paths = ty.replace("::", "");
    without_paths.contains(':')
}

/// The C half: `0 field;` — a literal where a type specifier must stand.
pub fn c_literal_type(line: &str) -> bool {
    let t = line.trim().trim_end_matches(';');
    let mut parts = t.split_whitespace();
    let (Some(first), Some(second), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    is_int_literal(first) && is_ident(second)
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_int_literal(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Apply every shape to one generated file, returning the ones present.
pub fn shapes_in(text: &str, backend: &str) -> Vec<Shape> {
    let mut found = Vec::new();
    for shape in Shape::all() {
        if shape.backend() != backend {
            continue;
        }
        let hit = text.lines().any(|l| match shape {
            Shape::RustEmptyType => rust_empty_type(l),
            Shape::RustLiteralType => rust_literal_type(l),
            Shape::RustEmptyGeneric => rust_empty_generic(l),
            Shape::CLiteralType => c_literal_type(l),
            Shape::RustColonInType => rust_colon_in_type(l),
        });
        if hit {
            found.push(shape);
        }
    }
    found
}

/// The spec whose whole purpose is to contain every shape this command claims to
/// find. If the compiler stops producing it, the detectors are unproven and the
/// corpus reading is worthless.
const CONTROL_SPEC: &str = concat!(
    "module probe {\n",
    "    pub const Thing = struct {\n",
    "        ok : u8,\n",      // the one field that is well formed
    "        bad : 0,\n",      // -> RustLiteralType, CLiteralType
    "        empty : ,\n",     // -> RustEmptyType
    "        nogen : [],\n",   // -> RustEmptyGeneric
    "        eaten : u8  # a note,\n",  // swallows the NEXT line ->
    "        victim : u8,\n",           //    RustSwallowedField
    "    };\n",
    "}\n",
);

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

fn t27c(root: &Path) -> PathBuf {
    root.join("target/release/t27c")
}

fn generate(root: &Path, backend: &str, spec: &Path) -> Option<String> {
    let out = Command::new(t27c(root))
        .arg(backend)
        .arg(spec)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run the reproducer through the real compiler and require every shape to fire.
///
/// Returns the shapes that did NOT fire. An empty vec is the only result that
/// makes the corpus reading below mean anything.
fn control_failures(root: &Path) -> Result<Vec<Shape>> {
    let dir = std::env::temp_dir().join(format!("tri-misread-{}", std::process::id()));
    std::fs::create_dir_all(&dir).context("creating the control directory")?;
    let spec = dir.join("probe.t27");
    std::fs::write(&spec, CONTROL_SPEC).context("writing the control spec")?;

    let mut missed = Vec::new();
    for backend in ["gen-rust", "gen-c"] {
        let Some(text) = generate(root, backend, &spec) else {
            anyhow::bail!(
                "the control spec did not survive `{backend}`.\n\
                 It is four lines and the compiler must emit for it; a failure here\n\
                 is a broken build or a moved binary, not a clean corpus."
            );
        };
        let seen = shapes_in(&text, backend);
        for shape in Shape::all().into_iter().filter(|s| s.backend() == backend) {
            if !seen.contains(&shape) {
                missed.push(shape);
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(missed)
}

fn collect_specs(root: &Path, specs_dir: &str) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.join(specs_dir)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == "scratch") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "t27") {
                out.push(p);
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn run(args: &Misread) -> Result<()> {
    let root = repo_root()?;
    if !t27c(&root).exists() {
        anyhow::bail!(
            "target/release/t27c is not built. Nothing can be read from output that\n\
             was never produced; run `cargo build --release -p t27c` first."
        );
    }

    println!();
    println!("  the specs the compiler reads WRONGLY -- every gate green on all of them");
    println!("  {}", "-".repeat(66));

    // The control first, and its failure is fatal. A detector that finds nothing
    // and a detector that is broken print the same thing.
    let missed = control_failures(&root)?;
    if !missed.is_empty() {
        eprintln!("  CONTROL FAILED -- these shapes did not fire on a case built to contain them:");
        for s in &missed {
            eprintln!("      {}", s.label());
        }
        anyhow::bail!(
            "refusing to report on the corpus. An unproven detector reporting zero is\n\
             indistinguishable from a clean tree, and that is the defect this command\n\
             was written for."
        );
    }
    println!("  control: all {} shape(s) fired on the reproducer", Shape::all().len());

    let specs = collect_specs(&root, &args.specs_dir)?;
    if specs.is_empty() {
        anyhow::bail!(
            "no .t27 file under `{}`. A census with nothing to read is not a clean\n\
             census -- absence is not amnesty (T31).",
            args.specs_dir
        );
    }
    println!("  corpus:  {} spec(s) under {}", specs.len(), args.specs_dir);
    println!();

    let mut hits: Vec<(Shape, Vec<String>)> =
        Shape::all().into_iter().map(|s| (s, Vec::new())).collect();
    let mut generated = 0usize;

    for spec in &specs {
        let rel = spec
            .strip_prefix(&root)
            .unwrap_or(spec)
            .to_string_lossy()
            .replace('\\', "/");
        let mut any = false;
        for backend in ["gen-rust", "gen-c"] {
            let Some(text) = generate(&root, backend, spec) else {
                continue;
            };
            any = true;
            for shape in shapes_in(&text, backend) {
                if let Some(row) = hits.iter_mut().find(|(s, _)| *s == shape) {
                    row.1.push(rel.clone());
                }
            }
        }
        if any {
            generated += 1;
        }
    }

    println!("  generated for {} of {} spec(s)", generated, specs.len());
    println!();
    let total: usize = hits.iter().map(|(_, v)| v.len()).sum();
    for (shape, specs_hit) in &hits {
        println!("  {:>4}  {}", specs_hit.len(), shape.label());
        if args.list {
            for s in specs_hit {
                println!("           {}", s);
            }
        }
    }
    println!();
    if total == 0 {
        println!("  Nothing found, and the control above is why that reads as a result");
        println!("  rather than as a silence.");
    } else {
        println!("  Each of these parses and typechecks. The count above is a count of");
        println!("  SPEC-SHAPE pairs, not of specs: one spec can carry more than one.");
        println!("  `--list` names them.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_type_slot_is_found() {
        assert!(rust_empty_type("    pub variants: ,"));
        assert!(rust_empty_type("pub f: ,"));
    }

    #[test]
    fn a_real_field_is_not_an_empty_type_slot() {
        assert!(!rust_empty_type("    pub kind: u8,"));
        assert!(!rust_empty_type("    pub name: String,"));
        assert!(!rust_empty_type("// pub f: ,"));
    }

    #[test]
    fn a_literal_in_type_position_is_found_with_either_sign() {
        assert!(rust_literal_type("    pub success: 0,"));
        assert!(rust_literal_type("    pub offset: -1,"));
    }

    #[test]
    fn a_numeric_looking_type_name_is_not_a_literal() {
        assert!(!rust_literal_type("    pub v: u8,"));
        assert!(!rust_literal_type("    pub v: i32,"));
        assert!(!rust_literal_type("    pub v: f64,"));
    }

    #[test]
    fn the_c_half_needs_exactly_a_literal_and_a_name() {
        assert!(c_literal_type("    0 bad;"));
        assert!(!c_literal_type("    int32_t good;"));
        // Three tokens is a declaration with a qualifier, not this defect.
        assert!(!c_literal_type("    const int x;"));
    }

    #[test]
    fn a_bare_colon_in_a_type_is_found_whatever_wrecked_it() {
        // A swallowed field...
        assert!(rust_colon_in_type("    pub a: Floatb:String,"));
        // ...and a map type the emitter could not spell. Same shape, other cause.
        assert!(rust_colon_in_type("    pub env: Vec<str:str>,"));
    }

    #[test]
    fn a_path_type_is_not_a_bare_colon() {
        // `::` is how a Rust path spells its separator and must not be read as
        // the wreckage of two declarations.
        assert!(!rust_colon_in_type("    pub a: std::mem::Allocator,"));
        assert!(!rust_colon_in_type("    pub v: Vec<HashMap<K, V>>,"));
        assert!(!rust_colon_in_type("    pub p: *mut (),"));
    }

    #[test]
    fn an_empty_generic_is_found_anywhere_in_the_line() {
        assert!(rust_empty_generic("    pub adj: Vec<>,"));
        assert!(!rust_empty_generic("    pub adj: Vec<u8>,"));
    }

    #[test]
    fn shapes_are_read_only_from_the_backend_that_can_show_them() {
        // The C shape must not be sought in Rust output, or `0 bad;` written
        // inside a Rust comment would be reported against the wrong backend.
        let c_text = "    0 bad;\n";
        assert!(shapes_in(c_text, "gen-c").contains(&Shape::CLiteralType));
        assert!(!shapes_in(c_text, "gen-rust").contains(&Shape::CLiteralType));
    }

    #[test]
    fn the_control_spec_contains_the_defect_it_is_meant_to_carry() {
        // Guards the reproducer itself: if someone "tidies" the constant and
        // removes `bad : 0`, the control would pass vacuously and the command
        // would report a clean corpus it never proved.
        // One marker per shape. A shape whose case is missing makes the
        // control refuse at run time; this catches it at test time and
        // says which one.
        for needle in ["bad : 0", "empty : ,", "nogen : []", "eaten : u8  # a note", "victim : u8"] {
            assert!(CONTROL_SPEC.contains(needle), "control lost `{needle}`");
        }
    }

    #[test]
    fn every_shape_declares_a_backend_that_is_generated() {
        for s in Shape::all() {
            assert!(matches!(s.backend(), "gen-rust" | "gen-c"), "{:?}", s);
        }
    }
}
