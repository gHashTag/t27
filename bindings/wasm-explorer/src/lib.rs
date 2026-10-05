//! The bootstrap compiler, compiled for the browser.
//!
//! `apps/website/src/lib/t27Compiler.ts` in the trinity repository loads the
//! artifact this crate builds and drives it over a `.t27` spec to render the
//! spec explorer: tokens, AST, typecheck, HIR and every backend's output.
//!
//! Why this file exists at all. The vendored artifact it replaces was
//! committed as a binary and its source was committed NOWHERE -- `git log -S`
//! over the whole history of this repository finds no `t27_analyze`. The
//! sync script named `bindings/wasm-explorer` as its input and the directory
//! did not exist, so the script silently fell back to the vendored copy on
//! every run. The page therefore ran a compiler nobody could rebuild, and it
//! had drifted: probing it shows string literals still carrying their quotes
//! in `value`, which this lexer stopped doing.
//!
//! Three sources are pulled in by path rather than copied. `compiler.rs` is
//! sealed against `bootstrap/stage0/FROZEN_HASH`, so a copy here would be a
//! second home for the truth and the seal would not cover it.
//!
//! It is portable to `wasm32-unknown-unknown` as it stands: its only external
//! dependency is `serde::Serialize`, its two `std::fs` reads sit behind test
//! helpers, and every `std::env::var` is already written to tolerate absence.

// The three modules below are the CLI's, whole. This crate reaches a fraction
// of what they export -- the analysis needs six entry points out of hundreds --
// so `dead_code` here would report 270-odd functions that are alive in `t27c`
// and merely unreached from a browser. A warning list that long hides the one
// warning that matters.
#![allow(dead_code)]

// Testing this crate: `cargo test -- --skip compiler::` runs the ones that are
// ours. A bare `cargo test` runs several hundred more, because `#[path]` brings
// the compiler's own `#[cfg(test)]` modules along with its code -- that is the
// point of including rather than copying, and it is a feature until you reach
// these two:
//
//     compiler::tests_hir_roundtrip::test_roundtrip_uart_spec
//     compiler::tests_hir_roundtrip::test_roundtrip_bridge_spec
//
// (This paragraph used to print both counts. One was wrong the day it was
// written and both were wrong a commit later, because every test either crate
// gains moves them. A number nobody can keep true is worse than no number --
// the two names above are what a reader hitting the red needs anyway.)
//
// Both read `$CARGO_MANIFEST_DIR/../specs/fpga/*.t27`, so they pass from any
// crate exactly one level below the repository root and fail from anywhere
// else. From here that path is `bindings/specs/fpga/`, which does not exist.
// They pass in `bootstrap`, where they live and where they run.
//
// Not worked around. `compiler.rs` is sealed, so the fix is a reseal; a copy
// of the two fixtures under `bindings/` would be a second home for the truth;
// and moving this crate up a level to satisfy a relative path would separate
// it from `bindings/javascript` and `bindings/python`, which is where a
// binding belongs. The skip is narrow, it is named, and this comment is why.

#[path = "../../../bootstrap/src/compiler.rs"]
mod compiler;

#[path = "../../../bootstrap/src/use_resolve.rs"]
mod use_resolve;

#[path = "../../../bootstrap/src/codegen_js.rs"]
mod codegen_js;

#[path = "../../../bootstrap/src/codegen_ts.rs"]
mod codegen_ts;

// Whether this file is source at all -- asked before the parser, because a
// parser's answer about a Markdown document is not news. `t27c classify` reads
// the same function; see `source_kind.rs` for why it is not two functions.
#[path = "../../../bootstrap/src/source_kind.rs"]
mod source_kind;

use compiler::{Compiler, Lexer, Node};
use serde_json::{json, Map, Value};

// ---------------------------------------------------------------------------
// Memory, as the page's driver expects it
// ---------------------------------------------------------------------------

/// Reserve `len` bytes for the caller to write a spec into.
///
/// The allocation is exact -- `vec![0; len]` boxed to a slice -- because
/// `t27_free` reconstructs it from the same length, and `Vec::with_capacity`
/// is permitted to hand back more than it was asked for.
#[no_mangle]
pub extern "C" fn t27_alloc(len: usize) -> *mut u8 {
    let buf = vec![0u8; len].into_boxed_slice();
    Box::into_raw(buf) as *mut u8
}

/// Release a buffer from `t27_alloc` or the blob returned by `t27_analyze`.
///
/// # Safety
/// `ptr` must come from this module and `len` must be the length it was
/// created with -- for an analysis blob that is `4 + payload`, which is what
/// the driver passes.
#[no_mangle]
pub unsafe extern "C" fn t27_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    drop(Box::from_raw(std::slice::from_raw_parts_mut(ptr, len)));
}

/// Hand a JSON document back as `[u32 LE byte length][utf8 json]`.
fn into_blob(json: String) -> *mut u8 {
    let payload = json.into_bytes();
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);
    Box::into_raw(out.into_boxed_slice()) as *mut u8
}

/// # Safety
/// `ptr`/`len` must describe a buffer obtained from `t27_alloc`. Ownership of
/// it passes to this function, which is what the driver in `t27Compiler.ts`
/// relies on -- it frees only the returned blob.
#[no_mangle]
pub unsafe extern "C" fn t27_analyze(ptr: *mut u8, len: usize) -> *mut u8 {
    analyze_owned(ptr, len, None)
}

/// As `t27_analyze`, but told what the spec is CALLED.
///
/// Only the JavaScript backend needs this: its header names the file it was
/// generated from, and without a name the page would print a header no CLI run
/// could reproduce. The unnamed entry point is kept because the driver that
/// shipped with the vendored artifact calls it, and a page loading an older
/// bundle against a newer wasm must not break.
///
/// # Safety
/// Both buffers must come from `t27_alloc`; both are consumed.
#[no_mangle]
pub unsafe extern "C" fn t27_analyze_named(
    ptr: *mut u8,
    len: usize,
    name_ptr: *mut u8,
    name_len: usize,
) -> *mut u8 {
    let name = if name_ptr.is_null() || name_len == 0 {
        None
    } else {
        let raw = Box::from_raw(std::slice::from_raw_parts_mut(name_ptr, name_len));
        Some(String::from_utf8_lossy(&raw).into_owned())
    };
    analyze_owned(ptr, len, name)
}

unsafe fn analyze_owned(ptr: *mut u8, len: usize, name: Option<String>) -> *mut u8 {
    if ptr.is_null() {
        return into_blob(json!({"error": "null source pointer"}).to_string());
    }
    let raw = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
    let source = String::from_utf8_lossy(&raw).into_owned();
    into_blob(analyze_source(&source, name.as_deref()))
}

// ---------------------------------------------------------------------------
// The analysis itself
// ---------------------------------------------------------------------------

/// Every layer the spec explorer shows, as one JSON document.
///
/// Key order is alphabetical because `serde_json::Map` is a `BTreeMap` here;
/// that is incidental, and nothing downstream may depend on it.
fn analyze_source(source: &str, name: Option<&str>) -> String {
    let mut root = Map::new();

    root.insert("sourceBytes".into(), json!(source.len()));
    root.insert("sourceLines".into(), json!(source.lines().count()));

    // What this file IS, before anything asks whether it compiles. A `.t27`
    // extension is a filename: the corpus holds Markdown documents, TRI-27
    // assembly listings and fixtures that exist to be damaged, and each of them
    // makes the parser say "not a module" -- true, and not a defect. A catalog
    // that reads this can stop calling them broken specs.
    //
    // It is NOT a prediction about parsing. 5 non-`source` files parse anyway
    // (measured 2026-08-24), and plenty of `source` files do not.
    root.insert(
        "sourceKind".into(),
        json!(source_kind::classify(source).slug()),
    );

    // Tokens. The lexer is run on its own here rather than through the parser,
    // because the parser pulls tokens lazily and a spec that fails to parse
    // must still show its token stream.
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    root.insert("tokenCount".into(), json!(tokens.len()));
    root.insert(
        "tokens".into(),
        Value::Array(
            tokens
                .iter()
                .map(|t| {
                    json!({
                        "kind": format!("{:?}", t.kind),
                        "lexeme": t.lexeme,
                        "line": t.line,
                        "col": t.col,
                    })
                })
                .collect(),
        ),
    );

    // Characters the lexer did not recognise and threw away.
    //
    // `Lexer::dropped` is typed `(char, usize)` and records no column, so none
    // is invented here -- the vendored artifact printed one and the page never
    // read it; it reads only the length of this list.
    //
    // The entries are per BYTE, not per character: the lexer widens the byte it
    // is looking at, so one em dash arrives as three, `â` and two controls.
    // That is the compiler's accounting, unchanged by this crate, and it means
    // the page's loss figure overcounts every non-ASCII character threefold.
    // `compiler.rs` is sealed against `bootstrap/stage0/FROZEN_HASH`, so fixing
    // it is a reseal, not a line in a browser binding -- see the test below,
    // which pins the behaviour as it IS so the day it changes is visible.
    root.insert(
        "lexerDiscarded".into(),
        Value::Array(
            lexer
                .dropped
                .iter()
                .map(|(c, line)| json!({"char": c.to_string(), "line": line}))
                .collect(),
        ),
    );

    // The tree.
    match Compiler::parse_ast(source) {
        Ok(ast) => {
            root.insert("nodeCount".into(), json!(count_nodes(&ast)));
            root.insert("astDepth".into(), json!(depth(&ast)));
            root.insert("topLevel".into(), json!(ast.children.len()));
            root.insert("ast".into(), node_json(&ast));
        }
        Err(e) => {
            root.insert("astError".into(), json!(e));
        }
    }

    // What the parser threw away while still reporting a parse. Both lists are
    // empty when the parse failed outright -- there is no recovery to report,
    // and `astError` above is the whole story.
    root.insert(
        "discarded".into(),
        Value::Array(
            Compiler::parse_ast_bdd_fallbacks(source)
                .unwrap_or_default()
                .iter()
                .map(|(line, why, clause)| json!(format!("line {line}: {why} -- {clause}")))
                .collect(),
        ),
    );
    root.insert(
        "swallowed".into(),
        Value::Array(
            Compiler::parse_ast_dropped_records(source)
                .unwrap_or_default()
                .iter()
                .map(|(line, lexeme, channel)| json!({"what": format!("{lexeme} ({channel})"), "line": line}))
                .collect(),
        ),
    );

    root.insert(
        "typecheck".into(),
        match Compiler::typecheck(source) {
            Ok(tc) => json!({
                "ok": tc.ok,
                "errorCount": tc.error_count,
                "warnings": tc.warnings,
                "errors": tc.errors,
            }),
            Err(e) => json!({"fatal": e}),
        },
    );

    root.insert(
        "hir".into(),
        match Compiler::debug_hir(source) {
            Ok(text) => json!({"ok": true, "text": text}),
            Err(e) => json!({"ok": false, "error": e}),
        },
    );

    let mut targets = Map::new();
    targets.insert("zig".into(), target(Compiler::compile(source)));
    targets.insert("c".into(), target(Compiler::compile_c(source)));
    targets.insert("rust".into(), target(Compiler::compile_rust(source)));
    targets.insert("verilog".into(), target(Compiler::compile_verilog(source)));
    targets.insert(
        "verilog_hir".into(),
        target(Compiler::compile_verilog_hir(source)),
    );
    targets.insert("js".into(), declared_target(compile_js(source, name)));
    targets.insert("ts".into(), declared_target(compile_ts(source, name)));
    root.insert("targets".into(), Value::Object(targets));

    Value::Object(root).to_string()
}

/// What the declaration backends need, on the terms a browser can meet.
///
/// `use_resolve` is not run here: it reads the filesystem to splice imported
/// declarations in, and a browser has no checkout to read. A spec whose
/// declarations live behind a `use` therefore shows fewer of them on the page
/// than at the CLI -- which is a limit of the medium, not of the backend, and
/// is why the page says where its output came from.
///
/// Shared by both declaration layers so the page cannot show a `js` tab and a
/// `ts` tab built from two different readings of the same spec.
fn declarations_ast(source: &str, name: Option<&str>) -> Result<(Node, String), String> {
    let ast = Compiler::parse_ast(source)?;
    let name = match name {
        Some(n) if !n.is_empty() => n.to_string(),
        _ if !ast.name.is_empty() => format!("{}.t27", ast.name),
        _ => "spec.t27".to_string(),
    };
    Ok((ast, name))
}

/// The JavaScript backend, reached the way `t27c gen-js` reaches it.
fn compile_js(source: &str, name: Option<&str>) -> Result<(String, usize), String> {
    let (ast, name) = declarations_ast(source, name)?;
    codegen_js::generate_reported(&ast, &name)
}

/// The TypeScript backend, reached the way `t27c gen-ts` reaches it.
fn compile_ts(source: &str, name: Option<&str>) -> Result<(String, usize), String> {
    let (ast, name) = declarations_ast(source, name)?;
    codegen_ts::generate_reported(&ast, &name)
}

fn target(result: Result<String, String>) -> Value {
    match result {
        Ok(code) => json!({"ok": true, "bytes": code.len(), "code": code}),
        Err(e) => json!({"ok": false, "error": e}),
    }
}

/// A declaration backend's target, carrying what it left out.
///
/// The five compiled backends either produce a translation unit or fail. These
/// two have a third answer: an artifact that is real and incomplete, because a
/// const holding something JavaScript has no spelling for is announced rather
/// than allowed to take the other forty declarations down with it. `notEmitted`
/// is that count, so a catalog can mark the spec partial. Without it the only
/// way to learn the number is to parse `__NOT_EMITTED__` back out of the code --
/// a second, weaker implementation of something the backend already knows.
fn declared_target(result: Result<(String, usize), String>) -> Value {
    match result {
        Ok((code, not_emitted)) => {
            json!({"ok": true, "bytes": code.len(), "code": code, "notEmitted": not_emitted})
        }
        Err(e) => json!({"ok": false, "error": e}),
    }
}

fn count_nodes(n: &Node) -> usize {
    1 + n.children.iter().map(count_nodes).sum::<usize>()
}

fn depth(n: &Node) -> usize {
    1 + n.children.iter().map(depth).max().unwrap_or(0)
}

/// One AST node, in the shape the page's `T27Node` declares.
///
/// The `extra_*` fields are renamed to what the page calls them and are OMITTED
/// when empty, which is why a small spec's tree is readable at all. `pragma` is
/// carried through even though the page has no column for it: the alternative
/// is a field the compiler records and the page silently cannot show.
fn node_json(n: &Node) -> Value {
    let mut m = Map::new();
    m.insert("kind".into(), json!(format!("{:?}", n.kind)));
    m.insert("line".into(), json!(n.line));
    for (key, value) in [
        ("name", &n.name),
        ("value", &n.value),
        ("type", &n.extra_type),
        ("field", &n.extra_field),
        ("size", &n.extra_size),
        ("nodeKind", &n.extra_kind),
        ("op", &n.extra_op),
        ("pragma", &n.extra_pragma),
        ("returnType", &n.extra_return_type),
    ] {
        if !value.is_empty() {
            m.insert(key.into(), json!(value));
        }
    }
    if n.extra_pub {
        m.insert("pub".into(), json!(true));
    }
    if n.extra_mutable {
        m.insert("mutable".into(), json!(true));
    }
    if !n.params.is_empty() {
        m.insert(
            "params".into(),
            Value::Array(
                n.params
                    .iter()
                    .map(|(name, ty)| json!({"name": name, "type": ty}))
                    .collect(),
            ),
        );
    }
    m.insert(
        "children".into(),
        Value::Array(n.children.iter().map(node_json).collect()),
    );
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(src: &str) -> Value {
        serde_json::from_str(&analyze_source(src, Some("x.t27"))).unwrap()
    }

    #[test]
    fn the_javascript_layer_is_one_of_the_targets() {
        // The whole point of this crate's existence beside the vendored binary
        // it replaces: the page can only offer a language `t27_analyze` returns.
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        let js = &j["targets"]["js"];
        assert_eq!(js["ok"], json!(true), "{j}");
        assert!(
            js["code"].as_str().unwrap().contains("export const N = 7;"),
            "{js}"
        );
    }

    /// A spec the declaration backends can only half-print is neither broken nor
    /// whole, and the page can only say so if the count travels with the code.
    #[test]
    fn a_partial_artifact_says_how_much_it_left_out() {
        // `u8` names a type. A type alias has no value to export, so it is
        // announced -- and the const beside it still is.
        let j = analyze("module m;\npub const N: u32 = 7;\npub const PackedTrit = u8;\n");
        for lang in ["js", "ts"] {
            let t = &j["targets"][lang];
            assert_eq!(t["ok"], json!(true), "{t}");
            assert_eq!(t["notEmitted"], json!(1), "{lang}: {t}");
            // `= 7;` in JavaScript, `= 7 satisfies number;` in TypeScript --
            // the shared prefix is the part that says the whole spec survived.
            assert!(t["code"].as_str().unwrap().contains("export const N = 7"), "{t}");
        }
        // The compiled backends have no such answer, and must not grow a field
        // that would read as zero omissions rather than as no such question.
        assert_eq!(j["targets"]["rust"]["notEmitted"], Value::Null, "{j}");
    }

    #[test]
    fn a_whole_artifact_reports_no_omissions() {
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        assert_eq!(j["targets"]["js"]["notEmitted"], json!(0));
        assert_eq!(j["targets"]["ts"]["notEmitted"], json!(0));
    }

    #[test]
    fn the_name_reaches_the_generated_header() {
        // A header naming a file no one asked about is a header a CLI run
        // cannot reproduce, which is why the named entry point exists.
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        assert!(
            j["targets"]["js"]["code"]
                .as_str()
                .unwrap()
                .contains("from x.t27"),
            "{j}"
        );
    }

    #[test]
    fn a_spec_that_does_not_parse_still_shows_its_tokens() {
        // The reason the lexer is run separately from the parser. A spec that
        // fails to parse is exactly when someone needs the token stream.
        let j = analyze("module m;\npub const = ;\n");
        assert!(j["tokenCount"].as_u64().unwrap() > 3, "{j}");
        assert!(j.get("astError").is_some(), "{j}");
        assert!(j.get("ast").is_none(), "{j}");
    }

    #[test]
    fn every_layer_the_page_reads_is_present() {
        // The page destructures this document. A key that silently stops being
        // written renders as an empty panel, not as an error.
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        for key in [
            "sourceBytes",
            "sourceLines",
            "tokenCount",
            "tokens",
            "discarded",
            "swallowed",
            "lexerDiscarded",
            "typecheck",
            "hir",
            "targets",
        ] {
            assert!(j.get(key).is_some(), "missing {key}: {j}");
        }
        for layer in ["zig", "c", "rust", "verilog", "verilog_hir", "js", "ts"] {
            assert!(j["targets"].get(layer).is_some(), "missing target {layer}");
        }
    }

    #[test]
    fn the_typescript_layer_carries_the_declared_type() {
        // The one thing the `js` layer beside it cannot show. If this ever
        // prints `export const N = 7;` the tab has silently become a second
        // copy of the JavaScript one and has no reason to be on the page.
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        let ts = &j["targets"]["ts"];
        assert_eq!(ts["ok"], json!(true), "{j}");
        assert!(
            ts["code"]
                .as_str()
                .unwrap()
                .contains("export const N = 7 satisfies number;"),
            "{ts}"
        );
    }

    #[test]
    fn a_dropped_character_is_counted_once_per_byte() {
        // Written expecting 1 and corrected by running it. `Lexer::dropped` is
        // typed `(char, usize)`, which reads as one entry per character -- but
        // the byte is widened where it is pushed, so a three-byte em dash lands
        // as `â` and two controls. The page's loss figure therefore overcounts
        // every non-ASCII character threefold, and has since before this crate
        // existed: the vendored artifact did exactly the same.
        //
        // Pinned as it IS, not as it should be. `compiler.rs` is sealed, so the
        // repair is a reseal and does not belong in a browser binding; this
        // test is what makes the day it happens visible here.
        let j = analyze("module m;\n\u{2014}\npub const N: u32 = 7;\n");
        let dropped = j["lexerDiscarded"].as_array().unwrap();
        assert_eq!(dropped.len(), 3, "{j}");
        assert_eq!(dropped[0]["char"], json!("\u{e2}"));
        assert_eq!(dropped[0]["line"], json!(2));
    }

    #[test]
    fn an_empty_extra_is_left_out_of_a_node() {
        // Emitting every `extra_*` would make the tree unreadable and triple
        // the document. The omission is load-bearing, so it is tested.
        let j = analyze("module m;\npub const N: u32 = 7;\n");
        let decl = &j["ast"]["children"][0];
        assert_eq!(decl["kind"], json!("ConstDecl"));
        assert_eq!(decl["type"], json!("u32"));
        assert!(decl.get("op").is_none(), "{decl}");
        assert!(decl.get("field").is_none(), "{decl}");
    }
}
