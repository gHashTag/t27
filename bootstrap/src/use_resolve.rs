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

/// The module path a `use` line names, read the way the resolver reads it, or
/// `None` for a line that is not an import. `use a::b::c;   // note` -> `a::b::c`.
fn use_path_expr(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("use ")?;
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
        return None;
    }
    Some(path_expr)
}

/// `a::b::c` -> `<specs>/a/b/c.t27`, whether or not that file exists.
fn use_path(specs_root: &Path, path_expr: &str) -> PathBuf {
    let mut p = specs_root.to_path_buf();
    // #5978: `use sandbox.session_timeout;` is the same path as
    // `use sandbox::session_timeout;` -- the parser now reads both, and
    // stores the dotted one as `sandbox::session_timeout`. Splitting on
    // `::` alone looked for `specs/sandbox.session_timeout.t27`, so the
    // parser and this resolver disagreed about one import.
    for seg in path_expr.split("::").flat_map(|s| s.split('.')) {
        p.push(seg);
    }
    p.set_extension("t27");
    p
}

/// `a::b::{X, Y}` -> `a::b`: the module a brace list takes its items from.
fn brace_module(path_expr: &str) -> Option<&str> {
    let i = path_expr.find('{')?;
    Some(path_expr[..i].trim_end().trim_end_matches("::").trim_end_matches('.'))
}

/// The spec one `use` line names, and whether the line names that whole module.
/// `use a::b;` -> `<specs>/a/b.t27`, whole. Two item forms name the module that
/// holds the items, not whole: `use a::b::{X, Y};` (#2537) and `use a::b::Item;`
/// when `<specs>/a/b/Item.t27` is not a spec (#5552). Only one level is tried:
/// `use a::b::c::Item;` with neither `c.t27` nor `c/Item.t27` names nothing,
/// rather than a guess two levels up. A module named through an item is spliced
/// like any other -- the referenced names it declares are pulled, not only the
/// listed ones -- but its basename is not a qualifier: after `use a::b::K;`,
/// `b::K` is not rewritten, as `b` is not in scope in Rust either.
fn use_target(specs_root: &Path, path_expr: &str) -> Option<(PathBuf, bool)> {
    let p = use_path(specs_root, path_expr);
    if p.is_file() {
        return Some((p, true));
    }
    let module = match brace_module(path_expr) {
        Some(m) if !m.is_empty() => use_path(specs_root, m),
        Some(_) => return None,
        None => p.parent()?.with_extension("t27"),
    };
    (module.starts_with(specs_root) && module.is_file()).then_some((module, false))
}

/// The spec each `use` line names (`use_target`), for each line that names one.
fn use_targets(source: &str, specs_root: &Path) -> Vec<PathBuf> {
    source
        .lines()
        .filter_map(use_path_expr)
        .filter_map(|e| use_target(specs_root, e))
        .map(|(p, _)| p)
        .collect()
}

/// #7176: one warning per `use` line `use_targets` drops. A target that is not
/// a file was skipped without a word, `gen` exited 0, and the first error came
/// from zig, about an identifier (`use of undeclared identifier 'LIST_END'`),
/// never about the `use` line. `tri mutate spec` read that as a spec that does
/// not pass (#7148): its copy sat in the system temp dir, with no `specs/`
/// above it.
///
/// A warning, not an error, and the exit code is unchanged: 119 `use` lines in
/// 70 tracked specs name neither a spec nor a module to take an item from (7 of
/// them brace lists), and the zig backend still emits an `@import` for a
/// qualified reference through such a line (`tests/dotted_module_name.rs`).
/// Making it an error is #7176's next step.
/// The note goes to stderr; the generated code on stdout does not change.
pub fn missing_use_notes(input_path: &Path, source: &str) -> Vec<String> {
    missing_uses(
        &input_path.display().to_string(),
        source,
        find_specs_root(input_path).as_deref(),
    )
}

fn missing_uses(label: &str, source: &str, specs_root: Option<&Path>) -> Vec<String> {
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let expr = match use_path_expr(line) {
            Some(e) => e,
            None => continue,
        };
        let why = match specs_root {
            None => format!("no specs/ directory above {}", label),
            Some(root) => {
                if use_target(root, expr).is_some() {
                    continue;
                }
                let p = use_path(root, expr);
                let module = p.parent().map(|d| d.with_extension("t27")).filter(|m| m.starts_with(root));
                match (brace_module(expr), module) {
                    (Some(m), _) => format!(
                        "no spec at {} (the module a brace list takes its items from)",
                        use_path(root, m).display()
                    ),
                    (None, Some(m)) => format!(
                        "no spec at {}, nor a module at {} to take the item from",
                        p.display(),
                        m.display()
                    ),
                    (None, None) => format!("no spec at {}", p.display()),
                }
            }
        };
        out.push(format!(
            "warning: {}:{}: use {} resolves to no spec: {}; nothing is spliced from it (#7176)",
            label,
            i + 1,
            expr,
            why
        ));
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
        let mut paren = 0i32;
        loop {
            let (b, k, p) = depth_change(lines[i]);
            brace += b;
            bracket += k;
            paren += p;
            // A header split over lines (`fn f(` ... `) -> T {`) is open
            // until its `)`: without the paren count the declaration was its
            // first line alone, and the importer got `fn f(` glued to the
            // next declaration (fpga/mac.t27's mac_parallel_multiply in
            // mac_tb).
            if (brace <= 0 && bracket <= 0 && paren <= 0) || i + 1 >= lines.len() {
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

/// How far one line moves the `{}`, `[]` and `()` depth, outside strings, char
/// literals, `//` comments and `;` prose lines. A char literal `'"'` read as
/// the start of a string hid the rest of its line, `)` included
/// (`specs/policy/own_language.t27`), and the declaration around it never
/// closed: every later declaration of the module went unspliced.
fn depth_change(line: &str) -> (i32, i32, i32) {
    let (mut brace, mut bracket, mut paren) = (0i32, 0i32, 0i32);
    if line.trim_start().starts_with(';') {
        return (0, 0, 0);
    }
    let c: Vec<char> = line.chars().collect();
    let mut in_str = false;
    let mut i = 0usize;
    while i < c.len() {
        if in_str {
            match c[i] {
                '\\' => i += 1, // the escaped char is not a closing quote
                '"' => in_str = false,
                _ => {}
            }
            i += 1;
            continue;
        }
        match c[i] {
            '"' => in_str = true,
            '/' if c.get(i + 1) == Some(&'/') => break, // line comment
            // A char literal, `'\x'` or `'x'`; a lone apostrophe is not one.
            '\'' if c.get(i + 1) == Some(&'\\') && c.get(i + 3) == Some(&'\'') => i += 3,
            '\'' if c.get(i + 2) == Some(&'\'') => i += 2,
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            _ => {}
        }
        i += 1;
    }
    (brace, bracket, paren)
}

/// Names the importer declares at module level, found by brace depth: depth 0,
/// or depth 1 inside a `module M {` block. `split_decls` takes the smallest
/// indent as the module's, so a nested body followed by one column-0 `fn`
/// (`specs/fpga/testbench/spi_tb.t27`) hid every declaration in the body, and
/// the splice pulled a second `spi_transfer` beside the spec's own: zig said
/// "duplicate struct member name". A name in this set is never pulled.
fn module_level_names(source: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut depth = 0i32;
    let mut module_open = false;
    for line in source.lines() {
        let at_module_level = depth == 0 || (depth == 1 && module_open);
        if at_module_level {
            if let Some(name) = decl_name(line) {
                out.insert(name);
            }
        }
        let t = line.trim_start();
        if depth == 0 && t.starts_with("module ") && t.contains('{') {
            module_open = true;
        }
        depth += depth_change(line).0;
        if depth <= 0 {
            depth = 0;
            module_open = false;
        }
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
fn qualified_refs(text: &str, modules: &[String]) -> Vec<(String, String, bool)> {
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
                    // #5574: a reference followed by `(` is a CALL; the other
                    // spellings (struct literals, enum values, types) keep the
                    // old flattening, which is only wrong for calls.
                    let is_call = text[after + name.len()..]
                        .chars()
                        .skip_while(|c| c.is_whitespace())
                        .next()
                        == Some('(');
                    out.push((format!("{}{}", needle, name), name, is_call));
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
    let first = normalised(&candidates[0].text);
    candidates.iter().all(|d| normalised(&d.text) == first)
}

/// Drop a line comment that is not inside a string literal.
///
/// `base/types.t27` writes `pub const ONE : i8 = 1;      // Trit = +1` and
/// `base/ops.t27` writes `pub const ONE : i8 = 1;`. Those are the same
/// declaration; only one of them is annotated. Comparing raw text calls that a
/// conflict and refuses the import.
fn without_comment(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < b.len() {
        match in_str {
            Some(q) => {
                if b[i] == b'\\' {
                    i += 1;
                } else if b[i] == q {
                    in_str = None;
                }
            }
            None => {
                if b[i] == b'"' || b[i] == b'\'' {
                    in_str = Some(b[i]);
                } else if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
                    return &line[..i];
                }
            }
        }
        i += 1;
    }
    line
}

/// A declaration reduced to what it states: no comments, no blank lines, no
/// indentation.
fn normalised(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| without_comment(l).trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// If this declaration is nothing but `= <module>::<name>;`, the module it
/// names.
///
/// `specs/math/sacred_physics.t27` declares
/// `const PHI : f64 = constants::PHI;` -- a re-export. It is not a second
/// opinion about PHI, it is a pointer to the first one.
fn alias_target(d: &Decl, name: &str) -> Option<String> {
    let joined = normalised(&d.text).join(" ");
    let rhs = joined.split_once('=')?.1.trim().trim_end_matches(';').trim();
    let (module, target) = rhs.split_once("::")?;
    let module = module.trim();
    if target.trim() != name || module.is_empty() {
        return None;
    }
    if !module
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    Some(module.to_string())
}

/// Which candidate to splice when they are not all the same declaration.
///
/// An alias -- `const PHI : f64 = constants::PHI;` -- names another candidate
/// rather than competing with it. When every candidate but one is an alias
/// pointing at that one's module, there is a single definition and the others
/// say so themselves.
///
/// Splicing the ALIAS would be wrong: its text is `constants::PHI`, and the
/// flat output has no `constants` namespace. The target is what carries the
/// value.
fn alias_resolved<'a>(candidates: &[&'a Decl], name: &str) -> Option<&'a Decl> {
    let mut real: Vec<&Decl> = Vec::new();
    let mut aliases: Vec<(&Decl, String)> = Vec::new();
    for d in candidates {
        match alias_target(d, name) {
            Some(m) => aliases.push((d, m.to_string())),
            None => real.push(d),
        }
    }
    if real.len() != 1 || aliases.is_empty() {
        return None;
    }
    let target = real[0];
    let target_module = target
        .origin
        .rsplit('/')
        .next()
        .unwrap_or(&target.origin)
        .trim_end_matches(".t27");
    if aliases.iter().all(|(_, m)| m == target_module) {
        Some(target)
    } else {
        None
    }
}
/// The names `resolve` refused to splice, as it explained them.
///
/// The refusal is already written down -- `resolve` puts an
/// `// UNRESOLVED <name>: declared in A and B -- ambiguous, not spliced`
/// comment in the source it returns. Every backend then strips comments, so
/// the compiler's own account of why a type is missing reaches nobody, and
/// `cc` reports `unknown type name 'Trit'` with no cause attached (#2764).
///
/// Returning them lets the gen commands put the explanation on stderr, where
/// the person reading the error is. stdout stays exactly the generated code.
pub fn unresolved_notes(resolved: &str) -> Vec<String> {
    resolved
        .lines()
        .map(str::trim)
        // Both halves of the shape, not just the prefix: a line reading
        // `// UNRESOLVED is a word` is prose, and matching on the prefix alone
        // reports it as a refusal.
        .filter(|l| l.starts_with("// UNRESOLVED ") && l.contains(": declared in "))
        .map(|l| l.trim_start_matches("// ").to_string())
        .collect()
}


/// Whether a qualified reference must be flattened to its bare name in the
/// spliced output (#5574). A qualified CALL whose bare name collides with a
/// local declaration binds to that local after flattening; that is correct
/// only when some imported declaration of the name agrees with the local one
/// (a faithful inline copy, `specs/igla/coder/dataset.t27`), and wrong when
/// the local is a namesake of a different arity (`specs/ml/transformer/mha_block.t27`).
/// Non-call spellings and pulled names keep the W606 flattening.
fn flatten_qualified(
    name: &str,
    is_call: bool,
    pulled_names: &HashSet<String>,
    local: &HashSet<String>,
    local_decls: &[Decl],
    available: &HashMap<String, Vec<Decl>>,
) -> bool {
    if pulled_names.contains(name) {
        return true;
    }
    if !local.contains(name) {
        return false;
    }
    if !is_call {
        return true;
    }
    local_decls
        .iter()
        .find(|d| d.name == name)
        .map(|ld| {
            available.get(name).map_or(false, |cands| {
                cands
                    .iter()
                    .any(|d| normalised(&d.text) == normalised(&ld.text))
            })
        })
        .unwrap_or(false)
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

    let local_decls: Vec<Decl> = split_decls(source, "self");
    let local: HashSet<String> = local_decls
        .iter()
        .map(|d| d.name.clone())
        .chain(module_level_names(source))
        .collect();

    // Basenames of the modules this spec imports whole, for the
    // qualified-reference rewrite (`use_target`: an item's module is not one).
    let modules: Vec<String> = source
        .lines()
        .filter_map(use_path_expr)
        .filter_map(|e| use_target(&specs_root, e))
        .filter(|(_, whole)| *whole)
        .filter_map(|(p, _)| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    let qualified = qualified_refs(source, &modules);

    // Fixpoint: pull a needed declaration, then look at what IT references.
    let mut pulled: Vec<Decl> = Vec::new();
    let mut pulled_names: HashSet<String> = HashSet::new();
    let mut ambiguous: Vec<(String, Vec<String>)> = Vec::new();
    let mut frontier = identifiers(source);
    for (_, name, _) in &qualified {
        frontier.insert(name.clone());
    }
    while !frontier.is_empty() {
        let mut next: HashSet<String> = HashSet::new();
        // The frontier is walked in HashSet order, which Rust randomises per
        // process. Sorting it here was tried and REMOVED: with the `distinct`
        // sort below in place and this one gone, 0 of the 492 importing specs
        // are non-deterministic over four runs each. A guard whose removal
        // changes nothing measurable is decoration, and the comment justifying
        // it would outlive the reason for it.
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
                // Same reason as the frontier above: `into_values()` on a
                // HashMap is randomised, and `distinct[0]` below is the
                // declaration that actually gets emitted whenever the
                // candidates agree. Agreement is judged on NORMALISED text, so
                // two agreeing declarations can still differ byte for byte.
                let mut v: Vec<&Decl> = by_origin.into_values().collect();
                v.sort_by(|a, b| (&a.origin, &a.name).cmp(&(&b.origin, &b.name)));
                v
            };
            let chosen: Option<&Decl> = if distinct.len() == 1 || all_agree(&distinct) {
                Some(distinct[0])
            } else {
                alias_resolved(&distinct, &name)
            };
            let decl = match chosen {
                Some(d) => d.clone(),
                None => {
                    let mut origins: Vec<String> =
                        distinct.iter().map(|d| d.origin.clone()).collect();
                    origins.sort();
                    ambiguous.push((name.clone(), origins));
                    continue;
                }
            };
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
    // A name can enter the frontier again on a later round -- it is never added
    // to `pulled_names`, since nothing was pulled -- so without this the same
    // refusal is written twice. It printed `UNRESOLVED PHI` twice for
    // `specs/physics/sacred_verification.t27`.
    ambiguous.dedup();

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
    // #5574: the W606 rewrite flattens a qualified reference to the bare name
    // the splice declares. For a name that is local as well, the bare spelling
    // binds to the LOCAL declaration -- correct only when the two declarations
    // are the same function (specs/igla/coder/dataset.t27 keeps inline copies
    // of eval.t27), and wrong when the local is a namesake of a DIFFERENT
    // arity (specs/ml/transformer/mha_block.t27's own `forward` vs
    // `multi_head_attn::forward`). A wrong flattening makes the typechecker
    // report the imported call against the local namesake. So a qualified CALL
    // whose bare name collides with a local declaration is flattened only when
    // some imported declaration of that name agrees (normalised text) with the
    // local one; otherwise the qualifier stays, the call binds to no local
    // function, and the arity check is skipped by non-match rather than run
    // against a namesake. Non-call spellings keep the old flattening.
    let mut rewrites: Vec<&(String, String, bool)> = qualified
        .iter()
        .filter(|(_, name, is_call)| {
            flatten_qualified(name, *is_call, &pulled_names, &local, &local_decls, &available)
        })
        .collect();
    rewrites.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    for (qual, name, _) in rewrites {
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
            refs
                .iter()
                .any(|(q, n, call)| q == "eval::has_substring" && n == "has_substring" && *call),
            "qualified_refs must pair the qualified spelling with the bare name: {:?}",
            refs
        );
    }

    #[test]
    fn a_qualified_call_is_marked_as_a_call() {
        let refs = qualified_refs(
            "let r = multi_head_attn::forward(a, b, c);",
            &["multi_head_attn".to_string()],
        );
        assert!(refs.iter().any(|(q, _, call)| q == "multi_head_attn::forward" && *call));
    }

    #[test]
    fn a_qualified_struct_literal_is_not_marked_as_a_call() {
        let refs = qualified_refs(
            "let m = multi_head_attn::AttentionMask{ .mask = [] };",
            &["multi_head_attn".to_string()],
        );
        assert!(
            refs.iter()
                .any(|(q, _, call)| q == "multi_head_attn::AttentionMask" && !*call)
        );
    }

    fn decl(origin: &str, text: &str, name: &str) -> Decl {
        Decl { name: name.into(), text: text.into(), origin: origin.into() }
    }

    #[test]
    fn a_qualified_call_to_a_namesake_of_different_arity_is_not_flattened() {
        // mha_block.t27 declares its own `forward` of two parameters and calls
        // `multi_head_attn::forward(a, b, c)` -- flattening binds the call to
        // the local namesake and the arity check reports a false positive.
        let local = ["forward".to_string()].into_iter().collect();
        let local_decls = vec![decl("self", "pub fn forward(s: usize, i: usize) -> usize { return i; }", "forward")];
        let mut available: HashMap<String, Vec<Decl>> = HashMap::new();
        available.insert(
            "forward".into(),
            vec![decl("multi_head_attn", "pub fn forward(s: usize, i: usize, m: usize) -> usize { return i; }", "forward")],
        );
        let pulled: HashSet<String> = HashSet::new();
        assert!(!flatten_qualified("forward", true, &pulled, &local, &local_decls, &available));
    }

    #[test]
    fn a_qualified_call_to_a_faithful_local_copy_is_still_flattened() {
        // dataset.t27 declares an inline copy of eval.t27's has_substring and
        // calls eval::has_substring(...): the two declarations agree, so the
        // bare spelling binds to the same function.
        let text = "pub fn has_substring(s: str, n: str, i: usize) -> bool { return true; }";
        let local = ["has_substring".to_string()].into_iter().collect();
        let local_decls = vec![decl("self", text, "has_substring")];
        let mut available: HashMap<String, Vec<Decl>> = HashMap::new();
        available.insert("has_substring".into(), vec![decl("eval", text, "has_substring")]);
        let pulled: HashSet<String> = HashSet::new();
        assert!(flatten_qualified("has_substring", true, &pulled, &local, &local_decls, &available));
    }

    #[test]
    fn a_non_call_qualified_reference_is_still_flattened() {
        let local = ["AttentionMask".to_string()].into_iter().collect();
        let local_decls = vec![decl("self", "pub const AttentionMask = struct { mask: usize };", "AttentionMask")];
        let mut available: HashMap<String, Vec<Decl>> = HashMap::new();
        available.insert(
            "AttentionMask".into(),
            vec![decl("multi_head_attn", "pub const AttentionMask = struct { mask: usize, shape: usize };", "AttentionMask")],
        );
        let pulled: HashSet<String> = HashSet::new();
        assert!(flatten_qualified("AttentionMask", false, &pulled, &local, &local_decls, &available));
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
    fn split_decls_keeps_a_header_split_over_lines_with_its_body() {
        let src = "fn f(\n    a: []u8,\n    n: usize,\n) -> u8 {\n    return a[n];\n}\nfn g() -> u8 {\n    return 2;\n}\n";
        let d = split_decls(src, "m");
        let names: Vec<&str> = d.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["f", "g"]);
        assert!(d[0].text.ends_with("    return a[n];\n}"), "{}", d[0].text);
        assert!(!d[0].text.contains("return 2;"));
    }

    #[test]
    fn depth_change_skips_char_literals_strings_and_prose() {
        assert_eq!(depth_change("    if (t > f and s[f] == '\"') { f += 1; }"), (0, 0, 0));
        assert_eq!(depth_change("    if (c == '(') {"), (1, 0, 0));
        assert_eq!(depth_change("    if (c == '\\'') {"), (1, 0, 0));
        assert_eq!(depth_change("    const S = \"a\\\"(\"; // (x"), (0, 0, 0));
        assert_eq!(depth_change("; prose (see the table"), (0, 0, 0));
        assert_eq!(depth_change("fn f("), (0, 0, 1));
    }

    #[test]
    fn split_decls_ends_a_body_whose_line_holds_a_quote_char() {
        let src = "fn trim(s: []u8, f: usize) -> usize {\n    if (s[f] == '\"') { return 1; }\n    return 0;\n}\nfn g() -> u8 {\n    return 2;\n}\n";
        let d = split_decls(src, "m");
        let names: Vec<&str> = d.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["trim", "g"]);
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

#[cfg(test)]
mod notes_tests {
    use super::*;

    #[test]
    fn a_refusal_is_reported_without_its_comment_marker() {
        let src = "// UNRESOLVED PHI: declared in a and b -- ambiguous, not spliced\nconst X = 1;\n";
        assert_eq!(
            unresolved_notes(src),
            vec!["UNRESOLVED PHI: declared in a and b -- ambiguous, not spliced".to_string()]
        );
    }

    #[test]
    fn an_ordinary_comment_is_not_a_refusal() {
        assert!(unresolved_notes("// UNRESOLVED is a word\n// note\nconst X = 1;\n").is_empty());
    }

    #[test]
    fn a_resolved_source_reports_nothing() {
        assert!(unresolved_notes("const X = 1;\nfn f() {}\n").is_empty());
    }

    #[test]
    fn an_indented_refusal_is_still_found() {
        // The splice writes at column zero today, but the corpus indents
        // everything under `module M;` and a future writer may match it.
        let src = "module M;\n    // UNRESOLVED T: declared in a and b -- ambiguous, not spliced\n";
        assert_eq!(unresolved_notes(src).len(), 1);
    }
}

#[cfg(test)]
mod alias_tests {
    use super::*;

    fn d(origin: &str, text: &str) -> Decl {
        Decl { name: "X".into(), text: text.into(), origin: origin.into() }
    }

    #[test]
    fn a_trailing_comment_is_not_part_of_a_declaration() {
        // base/types.t27 annotates `pub const ONE : i8 = 1;` and base/ops.t27
        // does not. Same declaration, one of them documented.
        let a = d("base/ops.t27", "pub const ONE : i8 = 1;");
        let b = d("base/types.t27", "pub const ONE : i8 = 1;      // Trit = +1");
        assert!(all_agree(&[&a, &b]));
    }

    #[test]
    fn a_slash_inside_a_string_does_not_start_a_comment() {
        assert_eq!(without_comment(r#"const U = "http://x"; // note"#), r#"const U = "http://x"; "#);
    }

    #[test]
    fn an_alias_names_its_target_module() {
        let a = d("math/sacred_physics.t27", "const PHI : f64 = constants::PHI;");
        assert_eq!(alias_target(&a, "PHI").as_deref(), Some("constants"));
    }

    #[test]
    fn a_value_is_not_an_alias() {
        let a = d("math/constants.t27", "const PHI : f64 = 1.618033988749895;");
        assert!(alias_target(&a, "PHI").is_none());
    }

    #[test]
    fn an_alias_to_a_different_name_is_not_an_alias_for_this_one() {
        // `const PHI = constants::PHI_INV;` is a re-export of something else,
        // and splicing the PHI definition for it would be wrong.
        let a = d("m/x.t27", "const PHI : f64 = constants::PHI_INV;");
        assert!(alias_target(&a, "PHI").is_none());
    }

    #[test]
    fn the_target_is_spliced_and_the_alias_is_not() {
        // Splicing the alias would emit `constants::PHI` into a flat file with
        // no `constants` namespace. The target carries the value.
        let real = d("math/constants.t27", "const PHI : f64 = 1.618033988749895;");
        let alias = d("math/sacred_physics.t27", "const PHI : f64 = constants::PHI;");
        let got = alias_resolved(&[&real, &alias], "PHI").expect("resolved");
        assert_eq!(got.origin, "math/constants.t27");
    }

    #[test]
    fn two_real_definitions_stay_ambiguous() {
        // TRINITY is 3.0 in constants and PHI_SQ + PHI_INV_SQ in
        // sacred_physics. Equal by this project's own identity, and still two
        // declarations: the choice is not mine.
        let a = d("math/constants.t27", "const TRINITY : f64 = 3.0;");
        let b = d("math/sacred_physics.t27", "const TRINITY : f64 = PHI_SQ + PHI_INV_SQ;");
        assert!(alias_resolved(&[&a, &b], "TRINITY").is_none());
    }

    #[test]
    fn an_alias_pointing_somewhere_else_stays_ambiguous() {
        let a = d("math/constants.t27", "const PHI : f64 = 1.618;");
        let b = d("math/sacred_physics.t27", "const PHI : f64 = elsewhere::PHI;");
        assert!(alias_resolved(&[&a, &b], "PHI").is_none());
    }
}


#[cfg(test)]
mod missing_use_tests {
    use super::*;

    /// A scratch `specs/` holding `a/b.t27`, so a target can exist or not.
    fn specs_with_a_b(tag: &str) -> PathBuf {
        let d = std::env::temp_dir()
            .join(format!("t27c-7176-{}-{}", std::process::id(), tag))
            .join("specs");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("a")).expect("dir");
        std::fs::write(d.join("a/b.t27"), "module b;\npub const K : u8 = 3;\n").expect("write");
        d
    }

    const SRC: &str = "module m;\n\
                       use a::b;\n\
                       use a::gone;   // a note\n\
                       use a::b::Item;\n\
                       use a::b::{X, Y};\n\
                       // use a::commented;\n\
                       use a.b;\n\
                       use a::gone::{X};\n\
                       use gone;\n";

    #[test]
    fn a_use_line_is_read_the_way_the_resolver_reads_it() {
        assert_eq!(use_path_expr("use a::b::c;"), Some("a::b::c"));
        assert_eq!(use_path_expr("    use igla::race::cordic;   // note"), Some("igla::race::cordic"));
        assert_eq!(use_path_expr("use sandbox.session;"), Some("sandbox.session"));
        assert_eq!(use_path_expr("use = 24,"), None);
        assert_eq!(use_path_expr("use ;"), None);
        assert_eq!(use_path_expr("// use a::b;"), None);
        assert_eq!(use_path_expr("user::x;"), None);
    }

    #[test]
    fn each_use_line_the_resolver_drops_is_named_by_line() {
        let root = specs_with_a_b("lines");
        let notes = missing_uses("m.t27", SRC, Some(&root));
        let lines: Vec<&str> = notes
            .iter()
            .map(|n| n.split(": use ").next().unwrap_or(""))
            .collect();
        assert_eq!(lines, vec!["warning: m.t27:3", "warning: m.t27:8", "warning: m.t27:9"], "{:#?}", notes);
        assert_eq!(
            notes[0],
            format!(
                "warning: m.t27:3: use a::gone resolves to no spec: no spec at {}, nor a module at {} to take the item from; nothing is spliced from it (#7176)",
                root.join("a/gone.t27").display(),
                root.join("a.t27").display()
            )
        );
        assert_eq!(
            notes[1],
            format!(
                "warning: m.t27:8: use a::gone::{{X}} resolves to no spec: no spec at {} (the module a brace list takes its items from); nothing is spliced from it (#7176)",
                root.join("a/gone.t27").display()
            )
        );
        assert_eq!(
            notes[2],
            format!(
                "warning: m.t27:9: use gone resolves to no spec: no spec at {}; nothing is spliced from it (#7176)",
                root.join("gone.t27").display()
            )
        );
    }

    /// #5552 and #2537: an item and a brace list name the module that holds
    /// them, one level up and no further, and only a whole-module line is whole.
    #[test]
    fn an_item_or_a_brace_list_names_its_module() {
        let root = specs_with_a_b("kinds");
        let ab = root.join("a/b.t27");
        assert_eq!(use_target(&root, "a::b"), Some((ab.clone(), true)));
        assert_eq!(use_target(&root, "a.b"), Some((ab.clone(), true)));
        assert_eq!(use_target(&root, "a::b::Item"), Some((ab.clone(), false)));
        assert_eq!(use_target(&root, "a.b.Item"), Some((ab.clone(), false)));
        assert_eq!(use_target(&root, "a::b::{X, Y}"), Some((ab.clone(), false)));
        assert_eq!(use_target(&root, "a::b::c::Item"), None);
        assert_eq!(use_target(&root, "a::gone::{X}"), None);
        assert_eq!(use_target(&root, "{X}"), None);
        assert_eq!(use_target(&root, "b"), None);
        assert_eq!(brace_module("a::b::{X, Y}"), Some("a::b"));
        assert_eq!(brace_module("a.b.{X}"), Some("a.b"));
        assert_eq!(brace_module("a::b"), None);
    }

    /// The splice reads an item's module; the module's basename stays out of
    /// the qualified-reference rewrite unless the whole module is imported.
    #[test]
    fn an_item_use_splices_from_its_module_without_making_it_a_qualifier() {
        let root = specs_with_a_b("splice");
        let input = root.join("m.t27");
        let item = "module m;\nuse a::b::K;\n\npub fn k() -> u8 {\n    return K;\n}\n";
        let out = resolve(&input, item);
        assert!(out.contains("pub const K : u8 = 3;"), "{}", out);
        let brace = "module m;\nuse a::b::{K};\n\npub fn k() -> u8 {\n    return K;\n}\n";
        assert!(resolve(&input, brace).contains("pub const K : u8 = 3;"));
        let through_item = "module m;\nuse a::b::K;\n\npub fn k() -> u8 {\n    return b::K;\n}\n";
        assert!(resolve(&input, through_item).contains("return b::K;"));
        let through_module = "module m;\nuse a::b;\n\npub fn k() -> u8 {\n    return b::K;\n}\n";
        let out = resolve(&input, through_module);
        assert!(out.contains("return K;") && !out.contains("b::K"), "{}", out);
    }

    /// Listing a name does not pick between two modules that declare it: it
    /// stays UNRESOLVED, as with whole-module imports. Rust's rule (an explicit
    /// import outranks a glob) was tried and measured on the corpus: in
    /// `specs/base/ring_32.t27` it chose math/sacred_physics' TRINITY, whose
    /// closure declares `PHI` as `constants::PHI`, and the splice rewrites
    /// qualifiers in the importer only, so zig failed on `constants` where the
    /// unspliced output passed its test.
    #[test]
    fn a_listed_item_two_modules_declare_stays_unresolved() {
        let root = specs_with_a_b("listed");
        std::fs::write(root.join("a/c.t27"), "module c;\npub const K : u8 = 4;\n").expect("write");
        let input = root.join("m.t27");
        let body = "\n\npub fn k() -> u8 {\n    return K;\n}\n";
        for line in ["use a::b::{ K };", "use a::b::K;", "use a::b;"] {
            let out = resolve(&input, &format!("module m;\nuse a::c;\n{}{}", line, body));
            assert!(out.contains("UNRESOLVED K"), "{}: {}", line, out);
            assert!(!out.contains("= 3;") && !out.contains("= 4;"), "{}: {}", line, out);
        }
    }

    /// `specs/fpga/testbench/spi_tb.t27`'s shape: a module body at four spaces
    /// and one `fn` at column 0. The body's own `K` is the importer's, so the
    /// module's `K` is not pulled beside it.
    #[test]
    fn a_name_the_importer_declares_in_a_nested_body_is_not_pulled() {
        let root = specs_with_a_b("nested");
        let input = root.join("m.t27");
        let src = "module m {\n    use a::b::K;\n\n    const K : u8 = 9;\n\n    fn k() -> u8 {\n        const J : u8 = 1;\n        return K + J;\n    }\n}\nfn tail() -> u8 { return K; }\n";
        let out = resolve(&input, src);
        assert!(out.contains("const K : u8 = 9;") && !out.contains("= 3;"), "{}", out);
        let names = module_level_names(src);
        let mut sorted: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        sorted.sort();
        assert_eq!(sorted, ["K", "k", "tail"]);
        let flat = "module m;\nconst A : u8 = 1;\ntest t {\n    const B : u8 = 2;\n}\nfn f() -> u8 {\n    const C : u8 = 3;\n    return C;\n}\n";
        let mut sorted: Vec<String> = module_level_names(flat).into_iter().collect();
        sorted.sort();
        assert_eq!(sorted, ["A", "f"]);
    }

    #[test]
    fn with_no_specs_directory_every_use_line_is_named() {
        let notes = missing_uses("/tmp/q/m.t27", SRC, None);
        assert_eq!(notes.len(), 7, "{:#?}", notes);
        assert_eq!(
            notes[0],
            "warning: /tmp/q/m.t27:2: use a::b resolves to no spec: no specs/ directory above /tmp/q/m.t27; nothing is spliced from it (#7176)"
        );
        assert!(missing_uses("m.t27", "module m;\npub const N : u8 = 1;\n", None).is_empty());
    }

    /// The splice and the warning read one line the same way: every `use`
    /// line is either a target `use_targets` returns or a warning, never both
    /// and never neither.
    #[test]
    fn every_use_line_is_spliced_or_named() {
        let root = specs_with_a_b("agree");
        let read = SRC.lines().filter_map(use_path_expr).count();
        let targets = use_targets(SRC, &root);
        let notes = missing_uses("m.t27", SRC, Some(&root));
        assert_eq!(read, 7);
        assert_eq!(targets.len(), 4, "{:?}", targets);
        assert_eq!(targets.len() + notes.len(), read);
    }
}
