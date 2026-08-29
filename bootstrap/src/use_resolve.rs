//! W569: resolve `use a::b::c` across specs.
//!
//! Until now `use` was parsed and then ignored: every backend emitted one
//! self-contained file per spec and nothing crossed a module boundary. So
//! `specs/igla/race/systolic_ternary.t27`, which declares
//! `use igla::race::ternary_mac;` and calls `ternary_mul(a_in, w)`, generated
//! Zig that failed with "use of undeclared identifier 'TernaryWeight'" --
//! against a type declared in exactly the module it had just imported.
//!
//! Measured in W568: 7 specs, **993 substantive assertion clauses**, three of
//! them the heaviest IGLA RACE kernels.
//!
//! ## Why splicing, and why SELECTIVE splicing
//!
//! The obvious design -- paste each dependency's declarations into the
//! generated file -- does not survive contact with the corpus. The import
//! closure of those 7 specs is 15 files with **38 colliding top-level names**;
//! `PHI` alone is declared in four of them. Pasting whole modules would pick a
//! winner silently.
//!
//! So only the names the importer actually *needs* are pulled in: referenced,
//! not declared locally, and not already pulled. A name found in two
//! dependencies is left UNRESOLVED with a comment naming both, because a wrong
//! silent choice is worse than the undeclared-identifier error it replaces.
//!
//! This runs as a source-to-source pass before the compiler, so `t27c gen`
//! keeps its "one spec in, one self-contained file out" contract.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// One top-level declaration lifted out of a spec, with the module it came from.
#[derive(Clone)]
struct Decl {
    name: String,
    text: String,
    origin: String,
}

/// Locate the repository's `specs/` directory by walking up from the input.
fn find_specs_root(input: &Path) -> Option<PathBuf> {
    let mut dir = input.parent()?.to_path_buf();
    loop {
        let candidate = dir.join("specs");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if dir.file_name().map(|n| n == "specs").unwrap_or(false) && dir.is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// `use a::b::c;` -> `<specs>/a/b/c.t27`
fn use_targets(source: &str, specs_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        let rest = match t.strip_prefix("use ") {
            Some(r) => r,
            None => continue,
        };
        // W587: strip a trailing comment BEFORE the semicolon. A line like
        // `use igla::race::cordic;   // note` left the whole comment inside the
        // module path, so the import silently resolved to nothing -- and the
        // comment in question was one I added in W571 to explain the import.
        let rest = match rest.find("//") {
            Some(i) => &rest[..i],
            None => rest,
        };
        let path_expr = rest.trim().trim_end_matches(';').trim();
        if path_expr.is_empty() || !path_expr.contains("::") && path_expr.contains(' ') {
            continue;
        }
        let mut p = specs_root.to_path_buf();
        for seg in path_expr.split("::") {
            p.push(seg);
        }
        p.set_extension("t27");
        if p.is_file() {
            out.push(p);
        }
    }
    out
}

/// Leading-space count, used to tell a module's own declarations from
/// statements inside a function or test body.
fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// The indentation at which this file writes its top-level declarations. Specs
/// are written both flat (`module M;` then column 0) and nested (`module M;`
/// then a four-space body), so it is measured rather than assumed.
fn top_level_indent(lines: &[&str]) -> usize {
    lines
        .iter()
        .filter(|l| decl_name(l).is_some())
        .map(|l| indent_of(l))
        .min()
        .unwrap_or(0)
}

/// Name of a top-level declaration opening on this line, if any.
fn decl_name(line: &str) -> Option<String> {
    let mut t = line.trim();
    if let Some(r) = t.strip_prefix("pub ") {
        t = r.trim_start();
    }
    for kw in ["fn ", "struct ", "enum ", "const ", "type "] {
        if let Some(rest) = t.strip_prefix(kw) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
            return None;
        }
    }
    None
}

/// Split a spec into its top-level declarations, keeping each one's source text
/// verbatim. A declaration runs until its brace and bracket depth return to
/// zero at an end of line -- the same rule for `fn f() { ... }` and for a
/// multi-line `const A : [3]u32 = [ 1, 2, 3 ]`.
fn split_decls(source: &str, origin: &str) -> Vec<Decl> {
    let lines: Vec<&str> = source.lines().collect();
    let top = top_level_indent(&lines);
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        // `const a: PackedTrit = 0xFF;` inside a test body is not a module
        // declaration; without this the resolver spliced statement fragments
        // into the importer and the whole file stopped parsing.
        let name = match decl_name(lines[i]).filter(|_| indent_of(lines[i]) == top) {
            Some(n) => n,
            None => {
                i += 1;
                continue;
            }
        };
        let start = i;
        let mut brace = 0i32;
        let mut bracket = 0i32;
        loop {
            let mut in_str = false;
            let mut prev = '\0';
            let mut chars = lines[i].chars().peekable();
            while let Some(c) = chars.next() {
                if in_str {
                    if c == '"' && prev != '\\' {
                        in_str = false;
                    }
                    prev = c;
                    continue;
                }
                match c {
                    '"' => in_str = true,
                    '/' if chars.peek() == Some(&'/') => break, // line comment
                    '{' => brace += 1,
                    '}' => brace -= 1,
                    '[' => bracket += 1,
                    ']' => bracket -= 1,
                    _ => {}
                }
                prev = c;
            }
            if (brace <= 0 && bracket <= 0) || i + 1 >= lines.len() {
                break;
            }
            i += 1;
        }
        out.push(Decl {
            name,
            text: lines[start..=i].join("\n"),
            origin: origin.to_string(),
        });
        i += 1;
    }
    out
}

/// Qualified references to an imported module: `eval::has_substring`,
/// `constants.PHI`. The generated file is FLAT -- every spliced declaration
/// lands in one scope -- so such a reference must both (a) mark the trailing
/// name as needed and (b) be rewritten to that bare name.
///
/// W588: the resolver collected only BARE identifiers, so a spec that referred
/// to an imported function by module name pulled nothing and then failed on the
/// qualified spelling.
fn qualified_refs(text: &str, modules: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for m in modules {
        for sep in ["::", "."] {
            let needle = format!("{}{}", m, sep);
            let mut from = 0usize;
            while let Some(i) = text[from..].find(&needle) {
                let start = from + i;
                // The module name must stand alone, not end another identifier.
                let ok_before = start == 0
                    || !text.as_bytes()[start - 1].is_ascii_alphanumeric()
                        && text.as_bytes()[start - 1] != b'_';
                let after = start + needle.len();
                let name: String = text[after..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if ok_before && !name.is_empty() {
                    out.push((format!("{}{}", needle, name), name));
                }
                from = after.max(start + 1);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every identifier-shaped token in the text, with `//` comments removed so a
/// doc line cannot invent a dependency.
fn identifiers(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in text.lines() {
        let code = match line.find("//") {
            Some(p) => &line[..p],
            None => line,
        };
        let mut cur = String::new();
        for c in code.chars() {
            if c.is_alphanumeric() || c == '_' {
                cur.push(c);
            } else {
                if !cur.is_empty() && !cur.chars().next().unwrap().is_ascii_digit() {
                    out.insert(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
            }
        }
        if !cur.is_empty() && !cur.chars().next().unwrap().is_ascii_digit() {
            out.insert(cur);
        }
    }
    out
}

/// Resolve `use` for one spec, returning the source with the needed foreign
/// declarations appended. On any failure -- no `specs/` root, no imports,
/// nothing missing -- the source is returned untouched, so this can never make
/// a spec that compiled stop compiling.
/// Do these candidate declarations say the same thing?
///
/// The refusal to splice an ambiguous name exists because "a wrong silent
/// choice is worse than the undeclared-identifier error it replaces". That
/// reasoning needs two candidates to disagree. When they are the same
/// declaration there is no choice to get wrong, and refusing costs the import
/// for nothing.
///
/// `Trit` is the case: `pub const Trit = enum(i8) { neg = -1, zero = 0,
/// pos = 1, };` appears verbatim in both `base/types.t27` and `base/ops.t27`,
/// and six specs import both. Each of them generated C using `Trit` 141 times
/// while declaring it zero times, and `cc` said `unknown type name 'Trit'`.
///
/// Compared line-by-line with each line trimmed, because the corpus writes two
/// indentation conventions -- a declaration at column 0 in one file and the
/// same declaration indented under `module M;` in another are the same
/// declaration. Comparing raw text would call those different and keep
/// refusing.
///
/// Measured over the corpus: 30 ambiguous (spec, name) pairs, of which 10
/// agree and 20 genuinely differ. The 20 stay unresolved -- `PHI` in
/// `math/constants.t27` against `math/sacred_physics.t27` is a real conflict
/// and a silent pick would be exactly the mistake this guard was built for.
fn all_agree(candidates: &[&Decl]) -> bool {
    let norm = |t: &str| -> Vec<String> {
        t.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };
    let first = norm(&candidates[0].text);
    candidates.iter().all(|d| norm(&d.text) == first)
}

pub fn resolve(input_path: &Path, source: &str) -> String {
    let specs_root = match find_specs_root(input_path) {
        Some(r) => r,
        None => return source.to_string(),
    };

    // Transitive closure of imports, so a pulled declaration's own dependencies
    // are available too.
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut queue: Vec<PathBuf> = use_targets(source, &specs_root);
    let mut available: HashMap<String, Vec<Decl>> = HashMap::new();
    while let Some(dep) = queue.pop() {
        let canonical = dep.canonicalize().unwrap_or_else(|_| dep.clone());
        if !seen.insert(canonical) {
            continue;
        }
        let text = match std::fs::read_to_string(&dep) {
            Ok(t) => t,
            Err(_) => continue,
        };
        // A dependency that does not parse on its own cannot be a source of
        // valid declarations. `specs/base/types.t27` is exactly this case: it
        // is imported by most of the corpus and it does not parse, and
        // splicing from it broke the importer. Its own `use` targets are still
        // followed -- an unparsable file can still name a parsable one.
        let dep_parses = crate::compiler::Compiler::parse_ast(&text).is_ok();
        queue.extend(use_targets(&text, &specs_root));
        if !dep_parses {
            continue;
        }
        let origin = dep
            .strip_prefix(&specs_root)
            .unwrap_or(&dep)
            .to_string_lossy()
            .to_string();
        for d in split_decls(&text, &origin) {
            available.entry(d.name.clone()).or_default().push(d);
        }
    }
    if available.is_empty() {
        return source.to_string();
    }

    let local: HashSet<String> = split_decls(source, "self")
        .into_iter()
        .map(|d| d.name)
        .collect();

    // Module basenames this spec imports, for the qualified-reference rewrite.
    let modules: Vec<String> = use_targets(source, &specs_root)
        .iter()
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    let qualified = qualified_refs(source, &modules);

    // Fixpoint: pull a needed declaration, then look at what IT references.
    let mut pulled: Vec<Decl> = Vec::new();
    let mut pulled_names: HashSet<String> = HashSet::new();
    let mut ambiguous: Vec<(String, Vec<String>)> = Vec::new();
    let mut frontier = identifiers(source);
    for (_, name) in &qualified {
        frontier.insert(name.clone());
    }
    while !frontier.is_empty() {
        let mut next: HashSet<String> = HashSet::new();
        for name in frontier {
            if local.contains(&name) || pulled_names.contains(&name) {
                continue;
            }
            let candidates = match available.get(&name) {
                Some(c) => c,
                None => continue,
            };
            let distinct: Vec<&Decl> = {
                let mut by_origin: HashMap<&str, &Decl> = HashMap::new();
                for d in candidates {
                    by_origin.entry(d.origin.as_str()).or_insert(d);
                }
                by_origin.into_values().collect()
            };
            if distinct.len() > 1 && !all_agree(&distinct) {
                let mut origins: Vec<String> =
                    distinct.iter().map(|d| d.origin.clone()).collect();
                origins.sort();
                ambiguous.push((name.clone(), origins));
                continue;
            }
            let decl = distinct[0].clone();
            next.extend(identifiers(&decl.text));
            pulled_names.insert(name);
            pulled.push(decl);
        }
        frontier = next;
    }

    if pulled.is_empty() && ambiguous.is_empty() {
        return source.to_string();
    }

    // Deterministic order: by origin, then by name, so regenerating a spec twice
    // produces byte-identical output.
    pulled.sort_by(|a, b| (&a.origin, &a.name).cmp(&(&b.origin, &b.name)));
    ambiguous.sort();

    // Rewrite `module::name` / `module.name` to the bare name the splice
    // declares. Longest first, so `a::bc` is not damaged by rewriting `a::b`.
    let mut out = String::from(source);
    // W606: `|| local.contains(name)`.
    //
    // The filter used to accept only names the splice PULLED, so a qualified
    // reference to something the importing file also declares itself was left
    // spelled `eval::has_substring` -- and codegen lowers `::` to `.`, which is
    // an undeclared namespace in the flat output.
    //
    // `specs/igla/coder/dataset.t27` is exactly that shape: it declares its own
    // `has_substring` (its header says "inline copies of eval.t27 templates to
    // avoid circular imports") AND calls `eval::has_substring(...)`. The
    // fixpoint skips local names by design, so the name never entered
    // `pulled_names`, so the rewrite never fired -- while three OTHER qualified
    // references in the same file, whose declarations were pulled, rewrote
    // correctly. One file, two outcomes, from one missing disjunct.
    //
    // Rewriting to the bare name is safe precisely BECAUSE the fixpoint skips
    // locals: a name that is local is never also pulled, so the bare spelling
    // has exactly one definition to bind to.
    let mut rewrites: Vec<&(String, String)> = qualified
        .iter()
        .filter(|(_, name)| pulled_names.contains(name) || local.contains(name))
        .collect();
    rewrites.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    for (qual, name) in rewrites {
        out = out.replace(qual.as_str(), name.as_str());
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\n// ---- resolved from `use` (t27c) ----\n");
    for (name, origins) in &ambiguous {
        out.push_str(&format!(
            "// UNRESOLVED {}: declared in {} -- ambiguous, not spliced\n",
            name,
            origins.join(" and ")
        ));
    }
    let mut current_origin = String::new();
    for d in &pulled {
        if d.origin != current_origin {
            out.push_str(&format!("\n// from {}\n", d.origin));
            current_origin = d.origin.clone();
        }
        out.push_str(&d.text);
        out.push('\n');
    }
    out
}

/// The enums declared by the specs this one imports, as
/// `(enum, [(variant, value)])` in `use` order.
///
/// This is deliberately NOT `resolve`. Splicing pulls whole declarations --
/// functions, structs, constants -- and the Verilog backend cannot lower most
/// of them, so widening its input is a change of behaviour for 492 specs. An
/// enum is different: the backend ALREADY lowers `Enum.variant` to the
/// identifier `Enum_variant`, and it already declares a `localparam` for every
/// enum a spec declares itself. The only thing missing when the enum arrives
/// through `use` is the declaration. That is what this returns, and nothing
/// else.
///
/// Direct imports only, and only dependencies that parse on their own -- the
/// same contract `resolve` carries. `specs/base/types.t27` does not parse and
/// declares `Trit`; a spec that also imports `base::ops` still gets `Trit`,
/// because ops declares the same enum. The first declaration of a name wins,
/// so one file can never resolve one name two ways.
pub fn imported_enums(input_path: &Path, source: &str) -> Vec<(String, Vec<(String, String)>)> {
    let specs_root = match find_specs_root(input_path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for dep in use_targets(source, &specs_root) {
        let text = match std::fs::read_to_string(&dep) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let ast = match crate::compiler::Compiler::parse_ast(&text) {
            Ok(a) => a,
            Err(_) => continue,
        };
        for decl in &ast.children {
            if decl.kind != crate::compiler::NodeKind::EnumDecl || decl.name.is_empty() {
                continue;
            }
            if !seen.insert(decl.name.clone()) {
                continue;
            }
            // The value is carried verbatim, including the empty string, so the
            // backend applies the same "no value means the ordinal" rule to an
            // imported enum that it applies to a local one.
            let variants: Vec<(String, String)> = decl
                .children
                .iter()
                .filter(|v| v.kind == crate::compiler::NodeKind::EnumVariant)
                .map(|v| (v.name.clone(), v.value.clone()))
                .collect();
            out.push((decl.name.clone(), variants));
        }
    }
    out
}

/// The struct declarations of every direct `use` dependency, in the same
/// `(name, fields)` shape `struct_decls` stores for a local struct. Mirrors
/// `imported_enums` (#2275): `word.raw` on an imported-struct param used to
/// fall past the part-select branch (struct_field_offset had no entry) and
/// flatten to the unbound identifier `word_raw`.
pub fn imported_structs(input_path: &Path, source: &str) -> Vec<(String, Vec<(String, String)>)> {
    let specs_root = match find_specs_root(input_path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for dep in use_targets(source, &specs_root) {
        let text = match std::fs::read_to_string(&dep) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let ast = match crate::compiler::Compiler::parse_ast(&text) {
            Ok(a) => a,
            Err(_) => continue,
        };
        for decl in &ast.children {
            if decl.kind != crate::compiler::NodeKind::StructDecl || decl.name.is_empty() {
                continue;
            }
            if !seen.insert(decl.name.clone()) {
                continue;
            }
            let fields: Vec<(String, String)> = decl
                .children
                .iter()
                .map(|f| (f.name.clone(), f.extra_type.clone()))
                .collect();
            out.push((decl.name.clone(), fields));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W606: a qualified reference to a name the importing file declares
    /// ITSELF must still be rewritten to the bare name. The filter used to
    /// accept only PULLED names, so `dataset.t27` -- which declares its own
    /// `has_substring` and also writes `eval::has_substring(...)` -- kept the
    /// qualified spelling and generated an undeclared `eval.` namespace.
    #[test]
    fn a_qualified_ref_to_a_local_name_is_still_rewritten() {
        let refs = qualified_refs("x = eval::has_substring(s, n, 0);", &["eval".to_string()]);
        assert!(
            refs.iter().any(|(q, n)| q == "eval::has_substring" && n == "has_substring"),
            "qualified_refs must pair the qualified spelling with the bare name: {:?}",
            refs
        );
    }

    #[test]
    fn decl_name_reads_the_declared_name() {
        assert_eq!(decl_name("pub const Foo = struct {").as_deref(), Some("Foo"));
        assert_eq!(decl_name("fn ternary_mul(a: i8) -> i8 {").as_deref(), Some("ternary_mul"));
        assert_eq!(decl_name("    let x = 1;"), None);
    }

    #[test]
    fn split_decls_keeps_a_multi_line_body_together() {
        let src = "fn f() -> i32 {\n    return 1;\n}\nfn g() -> i32 {\n    return 2;\n}\n";
        let d = split_decls(src, "m");
        assert_eq!(d.len(), 2);
        assert!(d[0].text.contains("return 1;"));
        assert!(!d[0].text.contains("return 2;"));
    }

    #[test]
    fn identifiers_ignores_comments() {
        let ids = identifiers("let x = y; // mentions zzz\n");
        assert!(ids.contains("y"));
        assert!(!ids.contains("zzz"));
    }
}

#[cfg(test)]
mod agree_tests {
    use super::*;

    fn d(origin: &str, text: &str) -> Decl {
        Decl { name: "X".into(), text: text.into(), origin: origin.into() }
    }

    #[test]
    fn identical_declarations_agree() {
        let a = d("types", "pub const Trit = enum(i8) {\n    neg = -1,\n};");
        let b = d("ops", "pub const Trit = enum(i8) {\n    neg = -1,\n};");
        assert!(all_agree(&[&a, &b]));
    }

    #[test]
    fn indentation_does_not_make_two_declarations_disagree() {
        // The corpus writes two conventions -- column 0 in one file, indented
        // under `module M;` in another. Raw text comparison calls these
        // different and keeps refusing an import that has no ambiguity in it.
        let a = d("types", "pub const T = enum(i8) {\n    neg = -1,\n};");
        let b = d("ops", "    pub const T = enum(i8) {\n        neg = -1,\n    };");
        assert!(all_agree(&[&a, &b]));
    }

    #[test]
    fn a_different_value_disagrees() {
        // PHI is 1.618... in math/constants.t27 and something else in
        // math/sacred_physics.t27. Twenty (spec, name) pairs are this shape and
        // every one must stay unresolved: a silent pick here is the mistake the
        // refusal was built to prevent.
        let a = d("constants", "pub const PHI : f64 = 1.618033988749895;");
        let b = d("sacred_physics", "pub const PHI : f64 = 1.6180339887;");
        assert!(!all_agree(&[&a, &b]));
    }

    #[test]
    fn a_missing_line_disagrees() {
        let a = d("x", "pub const E = enum(i8) {\n    a = 1,\n    b = 2,\n};");
        let b = d("y", "pub const E = enum(i8) {\n    a = 1,\n};");
        assert!(!all_agree(&[&a, &b]));
    }

    #[test]
    fn blank_lines_are_not_content() {
        let a = d("x", "pub const A = 1;");
        let b = d("y", "\npub const A = 1;\n\n");
        assert!(all_agree(&[&a, &b]));
    }

    #[test]
    fn a_single_candidate_agrees_with_itself() {
        let a = d("x", "pub const A = 1;");
        assert!(all_agree(&[&a]));
    }
}
