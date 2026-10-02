//! The JavaScript backend: `t27c gen-js`.
//!
//! Why it exists. Our MCP servers run on JavaScript, and a .t27 spec that
//! describes one had no way to reach them -- so every such spec grew a
//! hand-written generator in a foreign language beside it, and the generator,
//! not the compiler, became the thing that decided what the artifact said.
//! That is a hole in the compiler wearing the costume of a build script. With
//! this backend the artifact is printed by t27c, and the bespoke generator
//! does not exist.
//!
//! Scope, stated plainly: this lowers *declarations* -- consts, enums and
//! structs -- into an ES module. It does not lower function bodies. A `fn` in
//! the input is not dropped in silence; it is announced in a comment in the
//! output, because an artifact that is quietly missing something is worse than
//! one that says what it left out.
//!
//! The emitted module is data. Nothing in it runs, so a spec cannot smuggle
//! behaviour into a server through its own description.
//!
//! This file is also the *value* layer of `t27c gen-ts`. TypeScript's syntax
//! for a value is JavaScript's, to the character, so `codegen_ts` calls the
//! helpers below rather than carrying a second copy of them: the reserved-word
//! list, the array-length check, the escape table and the constant-expression
//! walk have one home. Only the words in their error messages differ, and that
//! difference is a `Target` passed in, not a duplicated function.

use crate::compiler::{Node, NodeKind};
use std::collections::BTreeSet;

/// Which backend is driving the shared helpers below.
///
/// It exists so an error a caller sees names the subcommand they actually ran.
/// `gen-ts` reusing this layer must not report itself as `gen-js`, and the fix
/// for that is one argument -- not a parallel set of functions that drift.
pub(crate) struct Target {
    /// The subcommand to blame in an error: `gen-js` or `gen-ts`.
    pub tag: &'static str,
    /// The language to name in prose: `JavaScript` or `TypeScript`.
    pub lang: &'static str,
}

pub(crate) const JS: Target = Target { tag: "gen-js", lang: "JavaScript" };
pub(crate) const TS: Target = Target { tag: "gen-ts", lang: "TypeScript" };

impl Target {
    /// `as const` where TypeScript needs it to keep a literal literal, and
    /// nothing at all where JavaScript has no such spelling.
    fn as_const(&self) -> &'static str {
        if self.lang == "TypeScript" {
            " as const"
        } else {
            ""
        }
    }
}

/// The names already bound in the emitted module.
///
/// A spec may declare `test_value` three times, once inside each of three test
/// blocks, and in t27 that reads as three scopes. By the time this backend sees
/// them they are three module-level declarations, and a second
/// `export const test_value` is not a shadow -- it is a SyntaxError that stops
/// the WHOLE artifact from parsing, taking the other forty names with it. C,
/// Rust and Zig have block scope and never meet the problem.
///
/// So the first binding stands and the rest are announced. Neither one is more
/// right than the other -- they were never meant to be the same name -- and
/// saying so is better than picking silently.
///
/// It also answers the other half of the question: whether a name an expression
/// READS exists. Every backend but this one and `gen-ts` resolves that in a
/// compiler of its own; here the artifact is loaded as an ES module, and a name
/// that is not bound is a `ReferenceError` at import -- which takes the whole
/// module with it, exactly like the duplicate above.
///
/// Three shapes in the corpus reached that error, and all three came through one
/// line that assumed an identifier was `a reference to a const declared above`:
///
/// - `pub const PackedTrit = u8;` -- a type alias. `u8` names a type, and a type
///   has no value to export.
/// - `pub const sym = ast.Symbol;` -- a member of another module.
/// - `const PHI : f64 = constants::PHI; const PHI_SQ : f64 = PHI * PHI;` --
///   where `PHI` was correctly announced as not emitted, and `PHI_SQ` then read
///   it anyway. A refusal has to propagate, or announcing the first one merely
///   moves the failure two lines down.
#[derive(Default)]
pub(crate) struct Bound {
    /// Names this module has actually exported, in their JavaScript spelling.
    emitted: BTreeSet<String>,
    /// Declarations that were reached and announced instead of printed.
    refused: BTreeSet<String>,
    /// Every top-level declaration in the spec, whether or not it is reached
    /// yet. What separates `declared further down` from `not here at all`.
    declared: BTreeSet<String>,
}

impl Bound {
    /// The spec's own declaration names, read before anything is emitted.
    pub(crate) fn in_spec(ast: &Node) -> Self {
        let declared = ast
            .children
            .iter()
            .filter(|n| {
                matches!(n.kind, NodeKind::ConstDecl | NodeKind::EnumDecl | NodeKind::StructDecl)
            })
            .filter(|n| !n.name.is_empty())
            .map(|n| n.name.clone())
            .collect();
        Self { declared, ..Default::default() }
    }

    pub(crate) fn claim(&mut self, t: &Target, name: &str) -> Result<(), String> {
        if self.emitted.insert(name.to_string()) {
            return Ok(());
        }
        Err(format!(
            "{}: {:?} is declared more than once in this spec, and a second `export const {}` would stop the whole module from parsing",
            t.tag, name, name
        ))
    }

    /// Record a declaration that was announced, so anything reading it later is
    /// announced too rather than emitted against a name that will not exist.
    pub(crate) fn refuse(&mut self, name: &str) {
        if !name.is_empty() {
            self.refused.insert(name.to_string());
        }
    }

    /// Whether `name` has a value here, and if not, precisely why not.
    pub(crate) fn resolve(&self, t: &Target, name: &str, line: u32) -> Result<(), String> {
        if js_name(t, name).is_ok_and(|js| self.emitted.contains(&js)) {
            return Ok(());
        }
        let tag = t.tag;
        if self.refused.contains(name) {
            return Err(format!(
                "{tag}: {:?} at line {} was itself not emitted -- the announcement above it says why, and there is no value here to read",
                name, line
            ));
        }
        if self.declared.contains(name) {
            return Err(format!(
                "{tag}: {:?} at line {} is declared further down this spec, and a `const` cannot be read before its declaration",
                name, line
            ));
        }
        if is_primitive_type(name) {
            return Err(format!(
                "{tag}: {:?} at line {} names a type, and a type is not a value -- this reads as a type alias, which the typed backends spell and this one has nothing to stand for",
                name, line
            ));
        }
        Err(format!(
            "{tag}: {:?} at line {} is not declared in this spec, so there is nothing for it to name -- it may come from an import, which this backend does not read (one spec is compiled at a time)",
            name, line
        ))
    }
}

/// A built-in type name, so the refusal can say `a type is not a value` instead
/// of the vaguer `not declared here`. Widths are not enumerated: `u3` and `i27`
/// are ordinary in a ternary spec, so the shape is matched rather than a list.
fn is_primitive_type(name: &str) -> bool {
    if matches!(name, "bool" | "void" | "type" | "str" | "char" | "trit" | "tryte") {
        return true;
    }
    let Some(width) = name
        .strip_prefix('u')
        .or_else(|| name.strip_prefix('i'))
        .or_else(|| name.strip_prefix('f'))
        .or_else(|| name.strip_prefix('t'))
    else {
        return false;
    };
    !width.is_empty() && width.bytes().all(|b| b.is_ascii_digit())
}

/// What a backend could not print, recorded rather than thrown away.
///
/// The rule this file already states for a `fn` -- announced, never dropped in
/// silence -- used to stop at declarations. A const holding something beyond
/// this backend failed the whole module instead, which cost the forty consts
/// that were fine to save the reader from the one that was not. Six other
/// backends emit those specs, so the artifact went missing and the spec read as
/// broken.
///
/// So: the name is announced in a comment, collected here, and re-exported as
/// data. Two things then stay true at once. The omission is loud -- an ES module
/// fails at LINK time for a missing named export, so an importer of the absent
/// name gets an error rather than `undefined`. And it is legible to a tool: the
/// corpus catalog reads this list and marks such a spec partial, never whole.
#[derive(Default)]
pub(crate) struct NotEmitted {
    entries: Vec<(String, String)>,
}

impl NotEmitted {
    /// Announce one omission in the artifact and remember it for the map.
    pub(crate) fn record(&mut self, out: &mut String, t: &Target, what: &str, why: &str) {
        // The reason arrives as the error the value layer raised, which already
        // opens with the backend's own name. Printing that after `t27c gen-js:`
        // says it twice.
        let flat = why.replace('\n', " ");
        let why = flat.strip_prefix(&format!("{}: ", t.tag)).unwrap_or(&flat);
        out.push_str(&format!("// t27c {}: {} was not emitted -- {}\n", t.tag, what, why));
        self.entries.push((what.to_string(), why.to_string()));
    }

    /// How many declarations this artifact announced instead of printing.
    ///
    /// Returned alongside the code by `generate_reported` so a reader learns it
    /// by being told, not by parsing the artifact back. A catalog that had to
    /// regex its way to this number would be a second, weaker implementation of
    /// the thing the backend already knows exactly.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn finish(&self, t: &Target) -> String {
        let mut out = String::new();
        out.push_str("\n// What this spec holds and this backend did not print. Empty is the whole\n");
        out.push_str("// story most of the time; an entry here is a promise the artifact does not\n");
        out.push_str("// keep, and reading it is how a tool tells a partial module from a complete\n");
        out.push_str("// one without parsing comments.\n");
        out.push_str("//\n");
        out.push_str("// A LIST, not a map keyed by name. `specs/numeric/formats.t27` declares four\n");
        out.push_str("// separate consts called `result`, one per test block, and under an object\n");
        out.push_str("// literal three of the four omissions vanished into the fourth -- a record of\n");
        out.push_str("// what went missing that itself went missing. A spec is free to reuse a\n");
        out.push_str("// name; this file is not free to lose the second one.\n");
        let items: Vec<String> = self
            .entries
            .iter()
            .map(|(what, why)| {
                format!("{{ what: {}, why: {} }}", js_string(what), js_string(why))
            })
            .collect();
        out.push_str(&format!(
            "export const __NOT_EMITTED__ = Object.freeze([{}]{});\n",
            if items.is_empty() { String::new() } else { format!(" {} ", items.join(", ")) },
            t.as_const()
        ));
        out
    }
}

/// Reserved in a module context; `export const` with one of these is a syntax
/// error, so it is refused here with a sentence rather than there with a stack.
///
/// TypeScript shares this list exactly. Its own additions (`type`, `interface`,
/// `readonly`, ...) are *contextual* keywords and remain legal as value names,
/// so widening the list for `gen-ts` would refuse specs that compile.
const RESERVED: &[&str] = &[
    "await", "break", "case", "catch", "class", "const", "continue", "debugger", "default",
    "delete", "do", "else", "enum", "export", "extends", "false", "finally", "for", "function",
    "if", "import", "in", "instanceof", "let", "new", "null", "return", "static", "super",
    "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while", "with", "yield",
];

pub fn generate(ast: &Node, source_name: &str) -> Result<String, String> {
    generate_reported(ast, source_name).map(|(code, _)| code)
}

/// The artifact, and how many declarations it announced instead of printing.
///
/// Callers that only want the file use `generate`. The Spec Explorer wants both:
/// an artifact with omissions is not a broken spec and not a whole one either,
/// and a corpus catalog can only say so if it is told the count.
pub fn generate_reported(ast: &Node, source_name: &str) -> Result<(String, usize), String> {
    let mut out = String::new();
    out.push_str("// Generated by `t27c gen-js` from ");
    out.push_str(source_name);
    out.push_str(". Do not edit.\n");
    out.push_str("//\n");
    out.push_str("// Every name and every value here comes from that spec. Change the spec and\n");
    out.push_str("// regenerate; an edit made here is lost the next time anyone builds.\n\n");

    let mut struct_order: Vec<String> = Vec::new();
    let mut decl_order: Vec<String> = Vec::new();
    let mut missing = NotEmitted::default();
    let mut bound = Bound::in_spec(ast);

    for node in &ast.children {
        match node.kind {
            NodeKind::ConstDecl => {
                match js_name(&JS, &node.name)
                    .and_then(|name| Ok((name, const_value(&JS, node, &bound)?)))
                    .and_then(|(name, value)| {
                        bound.claim(&JS, &name)?;
                        Ok((name, value))
                    })
                {
                    Ok((name, value)) => {
                        out.push_str(&format!("export const {} = {};\n", name, value));
                        decl_order.push(node.name.clone());
                    }
                    Err(why) => {
                        bound.refuse(&node.name);
                        missing.record(&mut out, &JS, &format!("const {}", describe(node)), &why)
                    }
                }
            }
            NodeKind::EnumDecl => {
                // The body renders before the name is claimed, for the reason
                // the const arm states: a name claimed against a declaration
                // that then fails to print is a name that resolves here and
                // does not exist in the artifact.
                match js_name(&JS, &node.name)
                    .and_then(|name| Ok((name, enum_variants(&JS, node, &bound)?)))
                    .and_then(|(name, body)| {
                        bound.claim(&JS, &name)?;
                        Ok((name, body))
                    }) {
                    Ok((name, body)) => {
                        out.push_str(&format!(
                            "export const {} = Object.freeze({{ {} }});\n",
                            name, body.body
                        ));
                        for (what, why) in &body.dropped {
                            missing.record(&mut out, &JS, what, why);
                        }
                        decl_order.push(node.name.clone());
                    }
                    Err(why) => {
                        bound.refuse(&node.name);
                        missing.record(&mut out, &JS, &format!("enum {}", describe(node)), &why)
                    }
                }
            }
            NodeKind::StructDecl => {
                let name = match js_name(&JS, &node.name)
                    .and_then(|name| {
                        bound.claim(&JS, &name)?;
                        Ok(name)
                    }) {
                    Ok(name) => name,
                    Err(why) => {
                        bound.refuse(&node.name);
                        missing.record(&mut out, &JS, &format!("struct {}", describe(node)), &why);
                        continue;
                    }
                };
                let fields = struct_fields(node);
                out.push_str(&format!(
                    "export const {} = Object.freeze({{ __struct__: {}, fields: [{}] }});\n",
                    name,
                    js_string(&node.name),
                    fields.join(", ")
                ));
                struct_order.push(node.name.clone());
                decl_order.push(node.name.clone());
            }
            NodeKind::UseDecl => {}
            // Announced, never dropped in silence.
            NodeKind::FnDecl => out.push_str(&format!(
                "// t27c gen-js: fn {} was not emitted -- this backend lowers declarations, not bodies.\n",
                node.name
            )),
            NodeKind::TestBlock | NodeKind::BenchBlock | NodeKind::InvariantBlock => {
                out.push_str(&format!(
                    "// t27c gen-js: a {:?} was not emitted -- it is checked by the compiler, not by the artifact.\n",
                    node.kind
                ))
            }
            // A statement at module level -- `x = 1;`, a bare call, an `if`.
            // Some specs open with a few, and refusing the file over them threw
            // away every declaration underneath.
            _ => missing.record(
                &mut out,
                &JS,
                &format!("a {:?} {}", node.kind, describe(node)),
                "this backend lowers declarations, and a statement is not one",
            ),
        }
    }

    out.push_str("\n// Declaration order, which the spec's own laws depend on.\n");
    out.push_str(&format!("export const __STRUCT_ORDER__ = [{}];\n", list(&struct_order)));
    out.push_str(&format!("export const __DECL_ORDER__ = [{}];\n", list(&decl_order)));
    out.push_str(&missing.finish(&JS));
    Ok((out, missing.len()))
}

/// How to refer to a declaration in an announcement: its own name when it has
/// one, and its position when it does not, so two omissions never collapse into
/// one line and one key.
pub(crate) fn describe(node: &Node) -> String {
    if node.name.is_empty() {
        format!("at line {}", node.line)
    } else {
        node.name.clone()
    }
}

/// The body of an enum: `Info: 0, Warn: 5, Error: 6`.
///
/// Both backends print this, and they must not disagree about what a
/// discriminant is -- so it is written once here rather than twice.
/// An enum body, and whatever the body could not carry.
///
/// The second field exists because `compiler/ast.t27` declares `And` twice in
/// one `TokenType` -- once for the operator `and` (16) and once for the Gherkin
/// keyword `And` (99). An object literal keeps the last of a repeated key, so
/// the artifact silently answered 99 to both questions and the operator's
/// discriminant was gone: a wrong number wearing the shape of a right one,
/// which is the one thing this backend must never emit. TypeScript says so out
/// loud (TS1117); JavaScript does not, which is exactly why it is announced
/// here rather than left to the type-checker to notice.
pub(crate) struct EnumBody {
    pub body: String,
    /// `(what, why)` per displaced variant, for `NotEmitted`.
    pub dropped: Vec<(String, String)>,
}

pub(crate) fn enum_variants(t: &Target, node: &Node, scope: &Bound) -> Result<EnumBody, String> {
    let mut next = 0i64;
    let mut parts: Vec<String> = Vec::new();
    let mut dropped: Vec<(String, String)> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for variant in &node.children {
        if variant.kind != NodeKind::EnumVariant {
            continue;
        }
        // `Warn = 5` puts the 5 on the VARIANT NODE, in `value` -- not in a
        // child, which is where an earlier draft of this backend looked.
        // Reading only the children made an explicit discriminant vanish
        // without a word: `Warn = 5` came out `Warn: 1`, the position it
        // happened to sit in. A wrong number that looks like a right one is the
        // worst thing this backend could emit, so the node is asked first and
        // the counter is only the fallback.
        let value = if !variant.value.is_empty() {
            variant.value.clone()
        } else if let Some(child) = variant.children.first() {
            expr(t, child, scope)?
        } else {
            next.to_string()
        };
        // And the count resumes FROM the explicit value, so `Error` after
        // `Warn = 5` is 6 rather than 2 -- the rule C, Rust and Zig all share.
        next = value.parse::<i64>().unwrap_or(next) + 1;
        // The counter still advances past a displaced variant: the numbering
        // belongs to the spec, and renumbering what follows would change values
        // that are correct to compensate for one that is not.
        if !seen.insert(key(t, &variant.name)) {
            dropped.push((
                format!("variant {}.{} = {}", node.name, variant.name, value),
                format!(
                    "{}: {:?} appears twice in this enum, and an object literal keeps only the last -- the first is the one emitted, so this discriminant is absent rather than silently overwriting it",
                    t.tag, variant.name
                ),
            ));
            continue;
        }
        parts.push(format!("{}: {}", key(t, &variant.name), value));
    }
    Ok(EnumBody { body: parts.join(", "), dropped })
}

/// The `fields:` descriptor of a struct: `["name", "declared-t27-type"]` per
/// field, carrying the spec's OWN spelling of the type rather than the target
/// language's. Both backends emit this list, and they must agree on it -- a
/// consumer that reads the JS descriptor and the TS descriptor of the same spec
/// and gets two different answers has been lied to by one of them.
pub(crate) fn struct_fields(node: &Node) -> Vec<String> {
    node.children
        .iter()
        .filter(|field| !field.name.is_empty())
        .map(|field| format!("[{}, {}]", js_string(&field.name), js_string(&field.extra_type)))
        .collect()
}

pub(crate) fn list(names: &[String]) -> String {
    names.iter().map(|n| js_string(n)).collect::<Vec<_>>().join(", ")
}

/// A const's value, read with its declared type in hand.
pub(crate) fn const_value(t: &Target, node: &Node, scope: &Bound) -> Result<String, String> {
    let tag = t.tag;
    let child = node
        .children
        .first()
        .ok_or_else(|| format!("{tag}: const {} has no value", node.name))?;

    // `[4]str = [POST, PUT, PATCH, DELETE]` -- the parser hands back the whole
    // bracketed text as one bare word, so the element type has to come from the
    // declaration. The declared length is then a real check: a literal that
    // does not have the length its own type claims is refused here rather than
    // shipped as a short array.
    if let Some((count, elem)) = array_type(&node.extra_type) {
        let items = match string_units(child, &elem) {
            Some(units) => units,
            None => {
                let raw = array_items(t, child)?;
                let rendered: Result<Vec<String>, String> =
                    raw.iter().map(|item| array_element(t, node, count, &elem, item, scope)).collect();
                rendered?
            }
        };
        if items.len() != count {
            return Err(format!(
                "{tag}: {} is declared [{}]{} but its literal has {} element(s)",
                node.name,
                count,
                elem,
                items.len()
            ));
        }
        return Ok(format!("[{}]", items.join(", ")));
    }

    expr_typed(t, child, &node.extra_type, scope)
}

/// `pub const METHOD_GET : [3]u8 = "GET";` -- a string standing in for an array
/// of bytes, which is how most of the corpus writes a short fixed tag.
///
/// This is the one place where this backend deliberately spells a value
/// differently from its siblings. C emits `"GET"`, Rust and Zig emit their own
/// string forms, and all three are right: in those languages a string literal
/// IS an array of bytes. In JavaScript it is not, and `gen-ts` makes the
/// difference impossible to paper over -- `[3]u8` becomes the type
/// `readonly [number, number, number]`, so emitting `"GET"` there produces an
/// artifact that does not type-check against the spec's own declaration.
///
/// So the bytes are printed. `[71, 69, 84]` is the same three bytes the C
/// backend writes, said in the only way JavaScript can hold them, and the
/// declared length checks it the way it checks any other literal.
fn string_units(child: &Node, elem: &str) -> Option<Vec<String>> {
    let (bits, _) = int_width(elem)?;
    if child.extra_kind != "string" {
        return None;
    }
    Some(if bits <= 8 {
        // UTF-8 bytes, because that is what a `[N]u8` in a spec means and what
        // every other backend puts in the file.
        child.value.bytes().map(|b| b.to_string()).collect()
    } else {
        child.value.chars().map(|c| (c as u32).to_string()).collect()
    })
}

/// One element of an array whose type the declaration states.
fn array_element(
    t: &Target,
    node: &Node,
    count: usize,
    elem: &str,
    item: &Item<'_>,
    scope: &Bound,
) -> Result<String, String> {
    let tag = t.tag;
    let wants_text = elem == "str" || elem == "string";
    let js = match item {
        Item::Node(n) => expr_typed(t, n, elem, scope)?,
        // The bare-word path: the parser handed back `[POST, PUT]` with the
        // element quotes already gone, so the declared element type is the only
        // thing left that can say whether `POST` is a string or a reference to a
        // const above.
        Item::Word(w) if wants_text => js_string(w),
        Item::Word(w) if is_number(w) || w == "true" || w == "false" => w.clone(),
        Item::Word(w) => {
            scope.resolve(t, w, node.line)?;
            js_name(t, w)?
        }
    };
    // A string where a number belongs is the mismatch worth refusing: it is the
    // one that changes what the artifact means rather than merely how it looks,
    // and in `gen-ts` it is also the one that fails the emitted `satisfies`.
    if wants_text != js.starts_with('"') {
        return Err(format!(
            "{tag}: {} is declared [{}]{} but {} is not a {}",
            node.name, count, elem, js, elem
        ));
    }
    Ok(js)
}

pub(crate) fn array_type(decl: &str) -> Option<(usize, String)> {
    let rest = decl.strip_prefix('[')?;
    let (count, elem) = rest.split_once(']')?;
    Some((count.trim().parse().ok()?, elem.trim().to_string()))
}

/// An element of an array literal, as the parser left it.
///
/// The two arms are not interchangeable and the difference is the whole point.
/// A parsed element is a NODE and has to be lowered like any other expression;
/// an element the parser kept as text is a WORD and all that can be done with it
/// is to read it under the declared type. An earlier draft flattened both to a
/// string by reading `c.value` off the node -- which is empty on every node that
/// is not a bare literal, so `[-1, 0, 1]` was refused with
/// `"" is not a i32`: the unary minus made a node, and the node had no text.
pub(crate) enum Item<'a> {
    Node(&'a Node),
    Word(String),
}

pub(crate) fn array_items<'a>(t: &Target, child: &'a Node) -> Result<Vec<Item<'a>>, String> {
    let tag = t.tag;
    if child.kind == NodeKind::ExprArrayLiteral {
        return Ok(child.children.iter().map(Item::Node).collect());
    }
    // The bare-word path: the parser hands the whole bracketed text back as one
    // word with the element quotes already gone, so there is nothing to strip.
    let text = if child.name.is_empty() { &child.value } else { &child.name };
    let inner = text
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| format!("{tag}: {:?} is not an array literal", text))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(inner.split(',').map(|p| Item::Word(p.trim().to_string())).collect())
}

pub(crate) fn expr(t: &Target, node: &Node, scope: &Bound) -> Result<String, String> {
    expr_typed(t, node, "", scope)
}

/// A constant expression, lowered with the declared type of the const it
/// belongs to in hand.
///
/// The type is not decoration. `UART_CLOCK_HZ / UART_BAUD_RATE` is 868 when the
/// const is a `u32` and 868.0555... when it is an `f64`, because t27 divides
/// integers the way C, Rust and Zig do and JavaScript has only one `/`. Without
/// the declared type this backend would have to guess, and a wrong number that
/// looks like a right one is the worst thing it could emit.
///
/// The type propagates INTO operands, because every step of an integer
/// expression is an integer step. It does not propagate into a struct literal's
/// fields, which carry types of their own that this node does not know.
pub(crate) fn expr_typed(t: &Target, node: &Node, ty: &str, scope: &Bound) -> Result<String, String> {
    let (tag, lang) = (t.tag, t.lang);
    match &node.kind {
        NodeKind::ExprLiteral => {
            // `extra_kind == "string"` asks the one question that matters: was
            // this written in quotes? The answer is the difference between a
            // string and a reference to something with the same spelling. The
            // Zig backend asks it the same way, for the same reason.
            //
            // `node.value` is the DECODED text, already without its quotes.
            // Stripping a leading and trailing quote here as well -- which an
            // earlier draft of this backend did, to tolerate an older parser --
            // eats the real ones off a value like `"\"wrapped\""` and emits
            // `"wrapped"`: the string without the quotes that were its content.
            if node.extra_kind == "string" || node.extra_kind == "char" {
                return Ok(char_or_text(&node.value, ty));
            }
            // A char literal inside an array literal arrives WITH its quotes and
            // WITHOUT the `char` mark -- `['T','R','K','G']` gives four nodes
            // whose value is `'T'`, `'R'` and so on. So the quotes are the mark.
            if let Some(decoded) = char_literal(&node.value) {
                return Ok(char_or_text(&decoded, ty));
            }
            if is_number(&node.value) {
                return Ok(node.value.clone());
            }
            match node.value.as_str() {
                "true" | "false" | "null" => Ok(node.value.clone()),
                other => Err(format!(
                    "{tag}: literal {:?} at line {} has no {lang} spelling",
                    other, node.line
                )),
            }
        }
        NodeKind::ExprIdentifier => match node.name.as_str() {
            "true" | "false" | "null" => Ok(node.name.clone()),
            name if name.starts_with('[') => {
                // An array without a declared length: the elements are taken as
                // written, which is all the parser preserved of them.
                let items = array_items(t, node)?;
                let rendered: Result<Vec<String>, String> = items
                    .iter()
                    .map(|i| match i {
                        Item::Node(n) => expr_typed(t, n, "", scope),
                        Item::Word(w) if is_number(w) => Ok(w.clone()),
                        Item::Word(w) => Ok(js_string(w)),
                    })
                    .collect();
                Ok(format!("[{}]", rendered?.join(", ")))
            }
            name if is_number(name) => Ok(name.to_string()),
            // `gf16::GF16::ONE` -- the parser keeps a qualified name whole, and
            // the value it names lives in a spec this backend did not read. An
            // importless `GF16.ONE` in the artifact is a ReferenceError at load,
            // so the const is announced instead of guessed at.
            name if name.contains("::") => Err(format!(
                "{tag}: {:?} at line {} names a value in another module, which this backend does not read -- one spec is compiled at a time",
                name, node.line
            )),
            // A reference to something declared above -- checked, not assumed.
            // The assumption used to live in this comment alone, and three
            // shapes in the corpus walked straight through it into a
            // `ReferenceError` at import. See `Bound::resolve`.
            name => {
                scope.resolve(t, name, node.line)?;
                Ok(js_name(t, name)?)
            }
        },
        NodeKind::ExprEnumValue => {
            let holder = if node.name.is_empty() { &node.extra_type } else { &node.name };
            let variant = if node.extra_field.is_empty() { &node.value } else { &node.extra_field };
            if holder.is_empty() || variant.is_empty() {
                return Err(format!("{tag}: an enum value at line {} names no variant", node.line));
            }
            scope.resolve(t, holder, node.line)?;
            Ok(format!("{}[{}]", js_name(t, holder)?, js_string(variant)))
        }
        NodeKind::ExprArrayLiteral => {
            let items: Result<Vec<String>, String> =
                node.children.iter().map(|c| expr_typed(t, c, ty, scope)).collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        NodeKind::ExprUnary => {
            let child = node
                .children
                .first()
                .ok_or_else(|| format!("{tag}: an operator at line {} has no operand", node.line))?;
            let v = expr_typed(t, child, ty, scope)?;
            match node.extra_op.trim() {
                "-" => Ok(format!("(-{})", v)),
                "!" => Ok(format!("(!{})", v)),
                "~" => match int_width(ty) {
                    Some((bits, signed)) if bits <= 32 => {
                        Ok(if signed { format!("(~{})", v) } else { format!("((~{}) >>> 0)", v) })
                    }
                    _ => Err(bitwise_refusal(t, "~", ty, node.line)),
                },
                other => Err(format!(
                    "{tag}: the unary operator {:?} at line {} has no {lang} spelling this backend will guess at",
                    other, node.line
                )),
            }
        }
        NodeKind::ExprBinary => {
            let (left, right) = match (node.children.first(), node.children.get(1)) {
                (Some(l), Some(r)) => (l, r),
                _ => {
                    return Err(format!(
                        "{tag}: the operator {:?} at line {} is missing an operand",
                        node.extra_op, node.line
                    ))
                }
            };
            let (a, b) = (expr_typed(t, left, ty, scope)?, expr_typed(t, right, ty, scope)?);
            // Everything is fully parenthesised. The spec's precedence is
            // already in the shape of the tree, so reproducing it with
            // JavaScript's own precedence rules would be a second chance to get
            // it wrong for no gain in readability.
            let op = node.extra_op.trim();
            match op {
                "+" | "-" | "*" | "%" => Ok(format!("({} {} {})", a, op, b)),
                "/" => match int_width(ty) {
                    // t27 truncates integer division, as C, Rust and Zig do.
                    // JavaScript's `/` is always real division: without this,
                    // `UART_CLOCK_HZ / UART_BAUD_RATE` ships 868.0555... where
                    // every other backend writes 868.
                    Some(_) => Ok(format!("Math.trunc({} / {})", a, b)),
                    None => Ok(format!("({} / {})", a, b)),
                },
                "==" => Ok(format!("({} === {})", a, b)),
                "!=" => Ok(format!("({} !== {})", a, b)),
                "<" | "<=" | ">" | ">=" => Ok(format!("({} {} {})", a, op, b)),
                "and" | "&&" => Ok(format!("({} && {})", a, b)),
                "or" | "||" => Ok(format!("({} || {})", a, b)),
                "&" | "|" | "^" | "<<" | ">>" => bitwise(t, op, ty, &a, &b, node.line),
                other => Err(format!(
                    "{tag}: the operator {:?} at line {} has no {lang} spelling this backend will guess at",
                    other, node.line
                )),
            }
        }
        NodeKind::ExprStructLit => {
            let mut parts: Vec<String> = Vec::new();
            for field in &node.children {
                if field.name.is_empty() {
                    continue;
                }
                let value = field.children.first().ok_or_else(|| {
                    format!("{tag}: field {} at line {} has no value", field.name, node.line)
                })?;
                // A field's type is the struct's business, not this const's, so
                // the value is lowered without a declared width rather than
                // under a borrowed one.
                parts.push(format!("{}: {}", key(t, &field.name), expr_typed(t, value, "", scope)?));
            }
            Ok(format!("Object.freeze({{ {} }})", parts.join(", ")))
        }
        NodeKind::ExprFieldAccess => {
            let object = node.children.first().ok_or_else(|| {
                format!("{tag}: the field {} at line {} has nothing to read it from", node.name, node.line)
            })?;
            let base = expr_typed(t, object, "", scope)?;
            Ok(match js_name(t, &node.name) {
                Ok(field) => format!("{}.{}", base, field),
                Err(_) => format!("{}[{}]", base, js_string(&node.name)),
            })
        }
        other => Err(format!(
            "{tag}: {:?} at line {} is not a constant expression. This backend emits data, never code.",
            other, node.line
        )),
    }
}

/// A `char` under an integer type is its code point, and text anywhere else.
///
/// `pub const FILE_MAGIC : [4]u8 = ['T', 'R', 'K', 'G'];` means four bytes, and
/// the C backend writes them as `'T'` because C says a char is one. JavaScript
/// has no such type -- a one-character string is a string -- so under a `u8` the
/// number is what the spec actually meant.
fn char_or_text(decoded: &str, ty: &str) -> String {
    let mut chars = decoded.chars();
    match (int_width(ty), chars.next(), chars.next()) {
        (Some(_), Some(c), None) => (c as u32).to_string(),
        _ => js_string(decoded),
    }
}

/// `'T'` -> `T`, `'\n'` -> a newline. `None` for anything that is not a char
/// literal, which is how this is told apart from an identifier.
fn char_literal(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut chars = inner.chars();
    let decoded = match (chars.next()?, chars.next()) {
        ('\\', Some(esc)) => match esc {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            _ => return None,
        },
        (c, None) => c,
        _ => return None,
    };
    if chars.next().is_some() {
        return None;
    }
    Some(decoded.to_string())
}

/// The width of an integer type, and whether it is signed. `None` for a float,
/// a bool, a string, a struct, or a type this backend cannot classify.
fn int_width(ty: &str) -> Option<(u32, bool)> {
    let ty = ty.trim().trim_end_matches('?').trim();
    let (signed, rest) = match ty.strip_prefix('i') {
        Some(rest) => (true, rest),
        None => (false, ty.strip_prefix('u')?),
    };
    if rest == "size" {
        // Whatever the target machine says, and this backend may not assume 32.
        return Some((64, signed));
    }
    let bits: u32 = rest.parse().ok()?;
    matches!(bits, 8 | 16 | 32 | 64 | 128).then_some((bits, signed))
}

/// Bitwise arithmetic, which JavaScript does in 32 signed bits and nothing else.
///
/// Inside 32 bits the answer is exact once it is read back unsigned: `1 << 31`
/// on a `u32` is -2147483648 as JavaScript computes it and 2147483648 after
/// `>>> 0`, which is what the spec means. A right shift needs `>>>` outright,
/// because `>>` propagates the sign bit it should not have.
///
/// Beyond 32 bits there is no such repair -- `1 << 40` is 256 in JavaScript --
/// so the const is refused and announced rather than quietly made wrong. No spec
/// in the corpus takes that path today; the guard is there so that the first one
/// to do so is told.
fn bitwise(t: &Target, op: &str, ty: &str, a: &str, b: &str, line: u32) -> Result<String, String> {
    match int_width(ty) {
        Some((bits, signed)) if bits <= 32 => Ok(match (op, signed) {
            (">>", false) => format!("({} >>> {})", a, b),
            (_, false) => format!("(({} {} {}) >>> 0)", a, op, b),
            (_, true) => format!("({} {} {})", a, op, b),
        }),
        _ => Err(bitwise_refusal(t, op, ty, line)),
    }
}

fn bitwise_refusal(t: &Target, op: &str, ty: &str, line: u32) -> String {
    let (tag, lang) = (t.tag, t.lang);
    let width = if ty.is_empty() { "a value of no declared width" } else { ty };
    format!(
        "{tag}: {:?} at line {} is bitwise arithmetic on {}, and {lang} does bitwise arithmetic in 32 signed bits -- the result would be wrong rather than merely absent",
        op, line, width
    )
}

pub(crate) fn is_number(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    for (prefix, radix) in [("0x", 16u32), ("0b", 2), ("0o", 8)] {
        if let Some(digits) = lower.strip_prefix(prefix) {
            return !digits.is_empty()
                && digits.chars().all(|c| c == '_' || c.is_digit(radix));
        }
    }
    // Exponent form is accepted because the corpus is full of it -- `1e-8`,
    // `6.24984990176514e-11` -- and JavaScript spells it identically. Falling
    // through on the `e` refused a number that needed no translation at all.
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut seen_exp = false;
    let mut exp_digit = false;
    let mut after_e = false;
    for c in s.chars() {
        match c {
            '0'..='9' => {
                if seen_exp {
                    exp_digit = true;
                } else {
                    seen_digit = true;
                }
                after_e = false;
            }
            '_' => after_e = false,
            '.' if !seen_dot && !seen_exp => {
                seen_dot = true;
                after_e = false;
            }
            'e' | 'E' if seen_digit && !seen_exp => {
                seen_exp = true;
                after_e = true;
            }
            // A sign is part of a number only directly after the `e`.
            '+' | '-' if after_e => after_e = false,
            _ => return false,
        }
    }
    // `1e` is not a number, and neither is `1e-`.
    seen_digit && (!seen_exp || exp_digit)
}

pub(crate) fn js_name(t: &Target, name: &str) -> Result<String, String> {
    let (tag, lang) = (t.tag, t.lang);
    let mut chars = name.chars();
    let valid = match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        }
        _ => false,
    };
    if !valid {
        return Err(format!("{tag}: {:?} is not a name {lang} can bind", name));
    }
    if RESERVED.contains(&name) {
        return Err(format!("{tag}: {:?} is a {lang} keyword; rename it in the spec", name));
    }
    Ok(name.to_string())
}

/// An object key: quoted unless it is plainly an identifier, so a variant named
/// `default` is legal in the output.
pub(crate) fn key(t: &Target, name: &str) -> String {
    match js_name(t, name) {
        Ok(n) => n,
        Err(_) => js_string(name),
    }
}

/// Quote a decoded string for JavaScript.
///
/// The lexer unescapes as it reads -- `compiler.rs` says so where the Zig
/// backend does the same thing -- so `node.value` holds a real newline, not a
/// backslash and an `n`. Everything that has to survive a round trip is put
/// back here.
///
/// Two of these arms exist for JavaScript alone. U+2028 and U+2029 terminate a
/// line inside a JS string literal *even between quotes*, so a description
/// carrying one would be a syntax error in the emitted module rather than a
/// character in it. The `\u{:04x}` spelling is exactly four digits and cannot
/// swallow a following hex digit the way C's `\x` can.
///
/// This is the compiler's third escape table, beside `zig_escape` and
/// `escape_for_fmt`. It is not a copy of either -- the JS-only arms above are
/// the reason -- but the overlap (`\\`, `"`, `\n`, `\r`, `\t`) is real, and
/// folding all three into one table with a target parameter is worth doing on
/// its own, away from a new backend.
pub(crate) fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scope holding exactly the names a test says the spec declared above.
    ///
    /// Spelled out per test rather than made permissive on purpose: a scope that
    /// resolves anything would pass the very artifacts this check exists to
    /// refuse, and the suite would stop measuring the thing it is for.
    fn scope(names: &[&str]) -> Bound {
        let mut b = Bound::default();
        for n in names {
            b.claim(&JS, n).unwrap();
        }
        b
    }

    fn lit(value: &str, kind: &str) -> Node {
        Node {
            kind: NodeKind::ExprLiteral,
            value: value.to_string(),
            extra_kind: kind.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn a_string_keeps_its_quotes() {
        // The defect this backend was written beside: every other emitter
        // printed `habr_search` where `"habr_search"` belonged.
        assert_eq!(expr(&JS, &lit("habr_search", "string"), &scope(&[])).unwrap(), "\"habr_search\"");
    }

    #[test]
    fn a_value_wrapped_in_real_quotes_keeps_them() {
        // `pub const W: str = "\"wrapped\"";` decodes to a value whose first and
        // last characters are quotes. The tolerate-an-older-parser strip this
        // backend used to do ate them and emitted `"wrapped"` -- the string
        // without the quotes that were its content.
        assert_eq!(
            expr(&JS, &lit("\"wrapped\"", "string"), &scope(&[])).unwrap(),
            "\"\\\"wrapped\\\"\""
        );
    }

    #[test]
    fn a_line_separator_cannot_break_the_emitted_module() {
        // U+2028 ends a line inside a JS string literal even between quotes, so
        // an unescaped one is a syntax error in the artifact, not a character.
        assert_eq!(
            expr(&JS, &lit("a\u{2028}b", "string"), &scope(&[])).unwrap(),
            "\"a\\u2028b\""
        );
    }

    #[test]
    fn a_number_is_not_a_string() {
        assert_eq!(expr(&JS, &lit("20", ""), &scope(&[])).unwrap(), "20");
        assert_eq!(expr(&JS, &lit("0x1f", ""), &scope(&[])).unwrap(), "0x1f");
    }

    #[test]
    fn an_unquoted_word_is_a_reference_not_a_string() {
        let node = Node {
            kind: NodeKind::ExprIdentifier,
            name: "PAGE_MIN".to_string(),
            ..Default::default()
        };
        assert_eq!(expr(&JS, &node, &scope(&["PAGE_MIN"])).unwrap(), "PAGE_MIN");
    }

    #[test]
    fn quotes_and_newlines_survive_a_description() {
        assert_eq!(js_string("say \"no\"\n"), "\"say \\\"no\\\"\\n\"");
    }

    #[test]
    fn a_keyword_is_refused_rather_than_emitted() {
        assert!(js_name(&JS, "class").is_err());
        assert!(js_name(&JS, "2fast").is_err());
        assert!(js_name(&JS, "HabrSearch").is_ok());
    }

    #[test]
    fn a_short_array_is_refused_by_its_own_declared_length() {
        let node = Node {
            kind: NodeKind::ConstDecl,
            name: "WRITE_METHODS".to_string(),
            extra_type: "[4]str".to_string(),
            children: vec![Node {
                kind: NodeKind::ExprIdentifier,
                name: "[POST,PUT,DELETE]".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = const_value(&JS, &node, &scope(&[])).unwrap_err();
        assert!(err.contains("3 element"), "{}", err);
    }

    #[test]
    fn an_array_of_strings_is_quoted_elementwise() {
        let node = Node {
            kind: NodeKind::ConstDecl,
            name: "WRITE_METHODS".to_string(),
            extra_type: "[2]str".to_string(),
            children: vec![Node {
                kind: NodeKind::ExprIdentifier,
                name: "[POST, PUT]".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(const_value(&JS, &node, &scope(&[])).unwrap(), "[\"POST\", \"PUT\"]");
    }

    #[test]
    fn an_explicit_discriminant_is_the_one_emitted() {
        // `pub enum Sev { Info, Warn = 5, Error }`. The 5 arrives on the
        // variant node itself; reading only the children silently printed the
        // position instead, and the count afterwards must resume from 5.
        let variant = |name: &str, value: &str| Node {
            kind: NodeKind::EnumVariant,
            name: name.to_string(),
            value: value.to_string(),
            ..Default::default()
        };
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![Node {
                kind: NodeKind::EnumDecl,
                name: "Sev".to_string(),
                children: vec![variant("Info", ""), variant("Warn", "5"), variant("Error", "")],
                ..Default::default()
            }],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(
            out.contains("{ Info: 0, Warn: 5, Error: 6 }"),
            "{}",
            out
        );
    }

    fn decl(name: &str, ty: &str, value: Node) -> Node {
        Node {
            kind: NodeKind::ConstDecl,
            name: name.to_string(),
            extra_type: ty.to_string(),
            children: vec![value],
            ..Default::default()
        }
    }

    fn binary(op: &str, left: Node, right: Node) -> Node {
        Node {
            kind: NodeKind::ExprBinary,
            extra_op: op.to_string(),
            children: vec![left, right],
            ..Default::default()
        }
    }

    fn ident(name: &str) -> Node {
        Node { kind: NodeKind::ExprIdentifier, name: name.to_string(), ..Default::default() }
    }

    #[test]
    fn an_element_that_is_a_node_is_lowered_not_read_as_text() {
        // `pub const TRIT_VALUES : [3]i32 = [-1, 0, 1];`. The minus makes a
        // node, and an earlier draft read `.value` off it -- which is empty --
        // and refused the spec with `"" is not a i32`. Rust prints
        // `[-(1), 0, 1]` for the same array.
        let node = decl(
            "TRIT_VALUES",
            "[3]i32",
            Node {
                kind: NodeKind::ExprArrayLiteral,
                children: vec![
                    Node {
                        kind: NodeKind::ExprUnary,
                        extra_op: "-".to_string(),
                        children: vec![lit("1", "")],
                        ..Default::default()
                    },
                    lit("0", ""),
                    lit("1", ""),
                ],
                ..Default::default()
            },
        );
        assert_eq!(const_value(&JS, &node, &scope(&[])).unwrap(), "[(-1), 0, 1]");
    }

    #[test]
    fn a_string_standing_for_bytes_becomes_its_bytes() {
        // `pub const METHOD_GET : [3]u8 = "GET";`. C, Rust and Zig all print the
        // string, because in those languages a string literal IS an array of
        // bytes. In JavaScript it is not, and `gen-ts` gives `[3]u8` the type
        // `readonly [number, number, number]` -- so `"GET"` would emit an
        // artifact that does not type-check against the spec's own declaration.
        let node = decl("METHOD_GET", "[3]u8", lit("GET", "string"));
        assert_eq!(const_value(&JS, &node, &scope(&[])).unwrap(), "[71, 69, 84]");
    }

    #[test]
    fn a_char_under_a_byte_type_is_its_code_point() {
        // `pub const FILE_MAGIC : [4]u8 = ['T', 'R', 'K', 'G'];`. The parser
        // keeps the quotes on these and drops the `char` mark, so the quotes are
        // what tells a char from an identifier.
        let node = decl(
            "FILE_MAGIC",
            "[4]u8",
            Node {
                kind: NodeKind::ExprArrayLiteral,
                children: vec![lit("'T'", ""), lit("'R'", ""), lit("'K'", ""), lit("'G'", "")],
                ..Default::default()
            },
        );
        assert_eq!(const_value(&JS, &node, &scope(&[])).unwrap(), "[84, 82, 75, 71]");
    }

    #[test]
    fn integer_division_truncates_and_float_division_does_not() {
        // `const UART_BIT_PERIOD : u32 = UART_CLOCK_HZ / UART_BAUD_RATE;` is 868
        // in C and Zig. JavaScript's `/` alone would ship 868.0555...
        let int = decl("P", "u32", binary("/", ident("CLK"), ident("BAUD")));
        assert_eq!(const_value(&JS, &int, &scope(&["CLK", "BAUD"])).unwrap(), "Math.trunc(CLK / BAUD)");

        // And an `f64` divides the way it reads: `360.0 / (PHI * PHI)`.
        let float = decl("GA", "f64", binary("/", lit("360.0", ""), binary("*", ident("PHI"), ident("PHI"))));
        assert_eq!(const_value(&JS, &float, &scope(&["PHI"])).unwrap(), "(360.0 / (PHI * PHI))");
    }

    #[test]
    fn an_exponent_is_a_number_javascript_already_spells() {
        assert!(is_number("1e-8"));
        assert!(is_number("6.24984990176514e-11"));
        assert!(is_number("1.0E9"));
        // And the edges, so the widening does not let a word through.
        assert!(!is_number("1e"));
        assert!(!is_number("1e-"));
        assert!(!is_number("e9"));
        assert!(!is_number("PHI"));
        // The hex path is untouched: the `e` in `0x1e` is a digit.
        assert!(is_number("0x1e"));
    }

    #[test]
    fn bitwise_arithmetic_is_refused_where_javascript_would_get_it_wrong() {
        // Inside 32 bits the answer is exact once it is read back unsigned.
        let ok = decl("MASK", "u32", binary("<<", lit("1", ""), lit("31", "")));
        assert_eq!(const_value(&JS, &ok, &scope(&[])).unwrap(), "((1 << 31) >>> 0)");

        // A right shift needs `>>>`, because `>>` propagates a sign bit a u32
        // does not have.
        let shr = decl("HALF", "u32", binary(">>", ident("X"), lit("1", "")));
        assert_eq!(const_value(&JS, &shr, &scope(&["X"])).unwrap(), "(X >>> 1)");

        // Beyond 32 bits there is no repair -- `1 << 40` is 256 in JavaScript --
        // so it is announced rather than quietly made wrong.
        let wrong = decl("BIG", "u64", binary("<<", lit("1", ""), lit("40", "")));
        let err = const_value(&JS, &wrong, &scope(&["X"])).unwrap_err();
        assert!(err.contains("32 signed bits"), "{}", err);
    }

    #[test]
    fn a_struct_literal_is_a_frozen_object() {
        let field = |name: &str, value: Node| Node {
            kind: NodeKind::ExprFieldAccess,
            name: name.to_string(),
            children: vec![value],
            ..Default::default()
        };
        let node = decl(
            "CLK",
            "PinSpec",
            Node {
                kind: NodeKind::ExprStructLit,
                name: "PinSpec".to_string(),
                children: vec![
                    field("port_name", lit("clk", "string")),
                    field("is_clock", lit("true", "")),
                ],
                ..Default::default()
            },
        );
        assert_eq!(
            const_value(&JS, &node, &scope(&[])).unwrap(),
            "Object.freeze({ port_name: \"clk\", is_clock: true })"
        );
    }

    #[test]
    fn a_value_from_another_module_is_announced_not_guessed_at() {
        // `gf16::GF16::ONE` is not in this file, and an importless `GF16.ONE` in
        // the artifact is a ReferenceError at load rather than a number.
        let node = decl("ONE", "gf16::GF16", ident("gf16::GF16::ONE"));
        let err = const_value(&JS, &node, &scope(&[])).unwrap_err();
        assert!(err.contains("another module"), "{}", err);
    }

    #[test]
    fn one_unprintable_const_does_not_cost_the_rest_of_the_module() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![
                decl("PAGE_MIN", "i32", lit("20", "")),
                decl("ONE", "gf16::GF16", ident("gf16::GF16::ONE")),
            ],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(out.contains("export const PAGE_MIN = 20;"), "{}", out);
        assert!(out.contains("// t27c gen-js: const ONE was not emitted"), "{}", out);
        // The omission is data too, so a tool can tell a partial module from a
        // whole one without reading comments.
        assert!(out.contains("__NOT_EMITTED__ = Object.freeze([ { what: \"const ONE\","), "{}", out);
        // And nothing claims the missing const was declared.
        assert!(out.contains("__DECL_ORDER__ = [\"PAGE_MIN\"];"), "{}", out);
    }

    #[test]
    fn a_whole_module_says_so_when_it_left_nothing_out() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![decl("PAGE_MIN", "i32", lit("20", ""))],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        // Emitted even when empty: the shape of the module is the same either
        // way, so a consumer can always import it.
        assert!(out.contains("export const __NOT_EMITTED__ = Object.freeze([]);"), "{}", out);
    }

    /// The defect this whole check exists for, in the shape the corpus had it:
    /// `sacred_physics.t27` opens `const PHI : f64 = constants::PHI;` and then
    /// `const PHI_SQ : f64 = PHI * PHI;`. Announcing only the first one left an
    /// artifact that parsed, imported, and threw `PHI is not defined` -- the
    /// failure moved two lines down rather than going away.
    /// `specs/numeric/formats.t27` declares four consts named `result`, one per
    /// test block, and none of them could be printed. Keyed by name, the record
    /// of four omissions was one entry -- and TypeScript said so, because
    /// `{ "const result": ..., "const result": ... }` is TS1117 in an artifact
    /// that is supposed to type-check itself.
    #[test]
    fn two_omissions_under_one_name_are_both_recorded() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![
                decl("result", "i32", ident("a::X")),
                decl("result", "i32", ident("b::Y")),
            ],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        let listed = out.matches("what: \"const result\"").count();
        assert_eq!(listed, 2, "{}", out);
    }

    #[test]
    fn a_refusal_reaches_everything_that_reads_it() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![
                decl("PHI", "f64", ident("constants::PHI")),
                decl("PHI_SQ", "f64", binary("*", ident("PHI"), ident("PHI"))),
                decl("PAGE_MIN", "i32", lit("20", "")),
            ],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(!out.contains("export const PHI_SQ"), "{}", out);
        assert!(out.contains("const PHI_SQ was not emitted -- \"PHI\""), "{}", out);
        assert!(out.contains("was itself not emitted"), "{}", out);
        // The cascade stops where it should: a const that reads neither of them
        // is still printed.
        assert!(out.contains("export const PAGE_MIN = 20;"), "{}", out);
        assert!(out.contains("__DECL_ORDER__ = [\"PAGE_MIN\"];"), "{}", out);
    }

    /// `pub const PackedTrit = u8;` in `specs/base/types.t27`. Six typed
    /// backends spell a type alias; this one has no value to put on the right
    /// of the `=`, and `export const PackedTrit = u8` is a ReferenceError.
    #[test]
    fn a_type_alias_is_announced_because_a_type_is_not_a_value() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![decl("PackedTrit", "", ident("u8"))],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(!out.contains("export const PackedTrit ="), "{}", out);
        assert!(out.contains("names a type, and a type is not a value"), "{}", out);
    }

    /// `pub const sym = ast.Symbol;` -- a member of a module this backend never
    /// read. The `::` spelling was already refused; the dotted one walked past
    /// the same guard because it arrives as a field access over an identifier.
    #[test]
    fn a_member_of_another_module_is_announced_whichever_way_it_is_spelled() {
        let field = Node {
            kind: NodeKind::ExprFieldAccess,
            name: "Symbol".to_string(),
            children: vec![ident("ast")],
            ..Default::default()
        };
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![decl("sym", "", field)],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(!out.contains("export const sym ="), "{}", out);
        assert!(out.contains("is not declared in this spec"), "{}", out);
    }

    /// `const` is not hoisted into a usable state: reading one before its
    /// declaration is a `ReferenceError` at load, not `undefined`. A spec may
    /// order its declarations freely, so this is a real difference between t27
    /// and the module it lowers to, and it is announced rather than reordered --
    /// reordering would change which line the artifact blames.
    #[test]
    fn a_name_declared_further_down_cannot_be_read_yet() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![
                decl("AREA", "i32", binary("*", ident("SIDE"), ident("SIDE"))),
                decl("SIDE", "i32", lit("3", "")),
            ],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(out.contains("is declared further down this spec"), "{}", out);
        assert!(out.contains("export const SIDE = 3;"), "{}", out);
    }

    /// An enum whose body cannot be printed must not leave its name bound: the
    /// name would resolve for everything below it and be absent from the
    /// artifact, which is the same ReferenceError one indirection away.
    #[test]
    fn an_enum_that_prints_nothing_does_not_hold_its_name() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![
                Node {
                    kind: NodeKind::EnumDecl,
                    name: "Level".to_string(),
                    // A discriminant that reads a name from somewhere else, so
                    // the body is what fails rather than the name.
                    children: vec![Node {
                        kind: NodeKind::EnumVariant,
                        name: "Warn".to_string(),
                        children: vec![ident("limits::WARN")],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                decl("FIRST", "", ident("Level")),
            ],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(!out.contains("export const Level"), "{}", out);
        assert!(!out.contains("export const FIRST"), "{}", out);
        assert!(out.contains("const FIRST was not emitted"), "{}", out);
    }

    /// `compiler/ast.t27` declares `And` twice in one `TokenType`: the operator
    /// `and` at 16 and the Gherkin keyword `And` at 99. An object literal keeps
    /// the last of a repeated key, so the artifact used to answer 99 to both
    /// questions -- a wrong number wearing the shape of a right one. The first
    /// stands, the rest are announced, and the counter does not renumber what
    /// follows.
    #[test]
    fn a_variant_declared_twice_keeps_the_first_and_announces_the_second() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![Node {
                kind: NodeKind::EnumDecl,
                name: "TokenType".to_string(),
                children: vec![
                    Node {
                        kind: NodeKind::EnumVariant,
                        name: "And".to_string(),
                        value: "16".to_string(),
                        ..Default::default()
                    },
                    Node {
                        kind: NodeKind::EnumVariant,
                        name: "And".to_string(),
                        value: "99".to_string(),
                        ..Default::default()
                    },
                    Node {
                        kind: NodeKind::EnumVariant,
                        name: "But".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(out.contains("And: 16"), "{}", out);
        assert!(!out.contains("And: 99"), "{}", out);
        // The numbering belongs to the spec: `But` follows 99, not 16.
        assert!(out.contains("But: 100"), "{}", out);
        assert!(
            out.contains("variant TokenType.And = 99"),
            "the displaced variant is recorded, not dropped: {}",
            out
        );
    }

    #[test]
    fn a_function_is_announced_not_dropped() {
        let ast = Node {
            kind: NodeKind::Module,
            children: vec![Node {
                kind: NodeKind::FnDecl,
                name: "helper".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let out = generate(&ast, "x.t27").unwrap();
        assert!(out.contains("fn helper was not emitted"), "{}", out);
    }
}
