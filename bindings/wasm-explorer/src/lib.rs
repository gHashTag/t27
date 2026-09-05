//! WebAssembly bridge: run the real t27 compiler in a browser.
//!
//! `compiler.rs` is included verbatim rather than vendored or reimplemented,
//! so every layer this returns is produced by the same code path the CLI uses.
//! It carries exactly one external `use` (`std::default::Default`) and its only
//! filesystem calls sit inside `#[cfg(test)]`, which is why it crosses to
//! `wasm32-unknown-unknown` untouched.
//!
//! ABI is deliberately plain `extern "C"` instead of wasm-bindgen: it needs no
//! JS glue generator in the build, so `cargo build --target wasm32-unknown-unknown`
//! is the whole toolchain. Strings cross the boundary as a length-prefixed blob
//! (`[u32 little-endian byte length][utf8 payload]`).

#[path = "../../../bootstrap/src/compiler.rs"]
pub mod compiler;

use compiler::{Compiler, Lexer, Node};

/// Reserve `len` bytes the caller can write UTF-8 source into.
///
/// Leaks deliberately: ownership passes to JS, which hands the pointer back to
/// `analyze`, which reclaims it.
#[no_mangle]
pub extern "C" fn t27_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

/// Release a blob previously returned by `analyze`.
#[no_mangle]
pub extern "C" fn t27_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// Hand a length-prefixed UTF-8 blob back to JS.
fn into_blob(s: String) -> *mut u8 {
    let bytes = s.into_bytes();
    let mut out = Vec::with_capacity(4 + bytes.len());
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&bytes);
    let ptr = out.as_mut_ptr();
    core::mem::forget(out);
    ptr
}

/// One AST node, shaped for the browser.
///
/// `line` is carried on every node so the UI can map a node back to the source
/// that produced it; `children` is always present (possibly empty) so the
/// consumer never has to branch on its absence.
fn node_json(n: &Node) -> serde_json::Value {
    let mut o = serde_json::Map::new();
    o.insert("kind".into(), format!("{:?}", n.kind).into());
    o.insert("line".into(), n.line.into());

    // Only non-empty scalars are emitted -- an AST of 5k nodes each carrying
    // eight empty strings is mostly padding, and the UI renders whatever is
    // present rather than a fixed field list.
    let mut put = |k: &str, v: &str| {
        if !v.is_empty() {
            o.insert(k.into(), v.into());
        }
    };
    put("name", &n.name);
    put("value", &n.value);
    put("type", &n.extra_type);
    put("field", &n.extra_field);
    put("size", &n.extra_size);
    put("nodeKind", &n.extra_kind);
    put("op", &n.extra_op);
    put("returnType", &n.extra_return_type);

    if n.extra_pub {
        o.insert("pub".into(), true.into());
    }
    if n.extra_mutable {
        o.insert("mutable".into(), true.into());
    }
    if !n.params.is_empty() {
        let ps: Vec<serde_json::Value> = n
            .params
            .iter()
            .map(|(name, ty)| serde_json::json!({ "name": name, "type": ty }))
            .collect();
        o.insert("params".into(), ps.into());
    }
    let kids: Vec<serde_json::Value> = n.children.iter().map(node_json).collect();
    o.insert("children".into(), kids.into());
    serde_json::Value::Object(o)
}

fn count_nodes(n: &Node) -> usize {
    1 + n.children.iter().map(count_nodes).sum::<usize>()
}

fn depth_of(n: &Node) -> usize {
    1 + n.children.iter().map(depth_of).max().unwrap_or(0)
}

/// Run one source through every layer and return the whole pipeline as JSON.
///
/// Each codegen target is captured independently: one backend failing must not
/// blank the others, since "Verilog fails here but Zig succeeds" is exactly the
/// kind of thing this page exists to show.
#[no_mangle]
pub extern "C" fn t27_analyze(ptr: *mut u8, len: usize) -> *mut u8 {
    let source = unsafe {
        let v = Vec::from_raw_parts(ptr, len, len);
        match String::from_utf8(v) {
            Ok(s) => s,
            Err(_) => return into_blob(r#"{"error":"source was not valid UTF-8"}"#.into()),
        }
    };

    let mut root = serde_json::Map::new();

    // ---- Layer 1: tokens -------------------------------------------------
    // The normal compile path never materialises a token vector (the parser
    // pulls one at a time), so this is a separate lex purely for display.
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    let toks: Vec<serde_json::Value> = tokens
        .iter()
        .map(|t| {
            serde_json::json!({
                "kind": format!("{:?}", t.kind),
                "lexeme": t.lexeme,
                "line": t.line,
                "col": t.col,
            })
        })
        .collect();
    root.insert("tokenCount".into(), toks.len().into());
    root.insert("tokens".into(), toks.into());

    // ---- Layer 2: AST, with what the parser silently dropped -------------
    // `parse_ast_full` rather than `parse_ast`: error recovery can discard
    // declarations while still reporting a clean parse, and a tree that
    // quietly omits half its file would be the wrong thing to draw.
    let (ast_res, discarded, swallowed, lex_discarded) = Compiler::parse_ast_full(&source);
    match &ast_res {
        Ok(ast) => {
            root.insert("ast".into(), node_json(ast));
            root.insert("nodeCount".into(), count_nodes(ast).into());
            root.insert("astDepth".into(), depth_of(ast).into());
            root.insert("topLevel".into(), ast.children.len().into());
        }
        Err(e) => {
            root.insert("astError".into(), e.clone().into());
        }
    }
    root.insert("discarded".into(), discarded.clone().into());
    let sw: Vec<serde_json::Value> = swallowed
        .iter()
        .map(|(what, line)| serde_json::json!({ "what": what, "line": line }))
        .collect();
    root.insert("swallowed".into(), sw.into());
    let lex_bad: Vec<serde_json::Value> = lex_discarded
        .iter()
        .map(|(ch, line, col)| {
            serde_json::json!({ "char": ch.to_string(), "line": line, "col": col })
        })
        .collect();
    root.insert("lexerDiscarded".into(), lex_bad.into());

    // ---- Layer 3: type check --------------------------------------------
    // Reported even though no codegen path runs it -- `t27c compile` gates on
    // it, `t27c gen` does not, and that difference is worth surfacing.
    match Compiler::typecheck(&source) {
        Ok(tc) => {
            root.insert(
                "typecheck".into(),
                serde_json::json!({
                    "ok": tc.ok,
                    "errorCount": tc.error_count,
                    "warnings": tc.warnings,
                    "errors": tc.errors,
                }),
            );
        }
        Err(e) => {
            root.insert("typecheck".into(), serde_json::json!({ "fatal": e }));
        }
    }

    // ---- Layer 4: HIR ----------------------------------------------------
    let hir = match Compiler::debug_hir(&source) {
        Ok(h) => serde_json::json!({ "ok": true, "text": h }),
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    };
    root.insert("hir".into(), hir);

    // ---- Layer 5: codegen targets ---------------------------------------
    let mut targets = serde_json::Map::new();
    let mut emit = |name: &str, r: Result<String, String>| {
        targets.insert(
            name.into(),
            match r {
                Ok(code) => serde_json::json!({ "ok": true, "code": code, "bytes": code.len() }),
                Err(e) => serde_json::json!({ "ok": false, "error": e }),
            },
        );
    };
    emit("zig", Compiler::compile(&source));
    emit("verilog", Compiler::compile_verilog(&source));
    emit("verilog_hir", Compiler::compile_verilog_hir(&source));
    emit("c", Compiler::compile_c(&source));
    emit("rust", Compiler::compile_rust(&source));
    root.insert("targets".into(), serde_json::Value::Object(targets));

    root.insert("sourceBytes".into(), source.len().into());
    root.insert("sourceLines".into(), source.lines().count().into());

    into_blob(serde_json::Value::Object(root).to_string())
}
