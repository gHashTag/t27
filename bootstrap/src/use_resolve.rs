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

    // Fixpoint: pull a needed declaration, then look at what IT references.
    let mut pulled: Vec<Decl> = Vec::new();
    let mut pulled_names: HashSet<String> = HashSet::new();
    let mut ambiguous: Vec<(String, Vec<String>)> = Vec::new();
    let mut frontier = identifiers(source);
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
            if distinct.len() > 1 {
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

    let mut out = String::from(source);
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

#[cfg(test)]
mod tests {
    use super::*;

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

