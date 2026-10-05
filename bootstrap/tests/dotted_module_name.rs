//! #5978: a dotted module name or use path is read whole.
//!
//! `module sandbox.health;` parsed as the module `sandbox` followed by a stray
//! top-level expression `.health` with no line number, and `use std.testing;`
//! as the import `std` followed by `.testing`. Twelve specs hit it (found by
//! the t27b corpus run). Nothing reported it: the gen-verilog backend turned
//! the strays into `always @(*) begin health; end`, which is not Verilog.
//!
//! The name is kept verbatim (the seal tool's `extract_module_name` already
//! recorded `sandbox.health`), and every place the name becomes an identifier
//! sanitizes it: the C include guard, the Verilog `module` line on both the
//! AST and the HIR path, and the gen-testbench DUT instance. A use now named
//! for its last segment can meet a parameter of that name, so the Zig backend
//! no longer imports a use whose name is bound as a local.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "t27c-dotmod-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    d
}

fn run(cmd: &[&str], p: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(cmd)
        .arg(p)
        .output()
        .expect("t27c");
    assert!(
        out.status.success(),
        "t27c {:?} failed: {}",
        cmd,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn write_spec(spec: &str, tag: &str) -> PathBuf {
    let p = scratch(tag).join("in.t27");
    std::fs::write(&p, spec).expect("write");
    p
}

fn parse(spec: &str, tag: &str) -> serde_json::Value {
    let p = write_spec(spec, tag);
    serde_json::from_str(&run(&["parse", "--json"], &p)).expect("json")
}

fn kinds(ast: &serde_json::Value) -> Vec<String> {
    ast["children"]
        .as_array()
        .expect("children")
        .iter()
        .map(|c| c["kind"].as_str().unwrap_or("").to_string())
        .collect()
}

const SPEC: &str = "module sandbox.health;\n\
                    use sandbox.session_timeout;\n\
                    use std.testing;\n\
                    fn f() -> i32 { return 1; }\n\
                    test t { assert f() == 1; }\n";

#[test]
fn a_dotted_module_name_is_read_whole_with_no_stray_expression() {
    let ast = parse(SPEC, "name");
    assert_eq!(ast["name"], "sandbox.health");
    let k = kinds(&ast);
    assert!(
        !k.iter().any(|k| k == "StmtExpr"),
        "stray top-level expression: {k:?}"
    );
    assert_eq!(k, ["UseDecl", "UseDecl", "FnDecl", "TestBlock"]);
}

#[test]
fn a_dotted_use_path_is_a_module_path() {
    let ast = parse(SPEC, "use");
    let uses: Vec<(String, String)> = ast["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["kind"] == "UseDecl")
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_string(),
                c["value"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        uses,
        [
            (
                "session_timeout".to_string(),
                "sandbox::session_timeout".to_string()
            ),
            ("testing".to_string(), "std::testing".to_string()),
        ]
    );
}

#[test]
fn a_mixed_path_and_dotted_name_is_kept_verbatim() {
    // specs/port/trinity/src/tri/gen_image.t27 spells it this way.
    let ast = parse(
        "module port::trinity.src.tri.gen_image;\nfn f() -> i32 { return 1; }\n",
        "mixed",
    );
    assert_eq!(ast["name"], "port::trinity.src.tri.gen_image");
    assert_eq!(kinds(&ast), ["FnDecl"]);
}

#[test]
fn a_later_dotted_module_header_leaves_nothing_behind() {
    let ast = parse(
        "module a;\nfn f() -> i32 { return 1; }\nmodule b.c;\nfn g() -> i32 { return 2; }\n",
        "second",
    );
    assert_eq!(ast["name"], "a");
    assert_eq!(kinds(&ast), ["FnDecl", "FnDecl"]);
}

#[test]
fn a_field_access_after_the_header_is_still_a_field_access() {
    // Negative control: only the header and the use path read `.` as part of
    // a name. A `.` inside a body is untouched.
    let ast = parse(
        "module a.b;\nconst S = struct { x: i32 };\nfn f(s: S) -> i32 { return s.x; }\n",
        "field",
    );
    assert_eq!(ast["name"], "a.b");
    let txt = ast.to_string();
    assert!(
        txt.contains("\"extra_field\":\"x\"") || txt.contains("ExprFieldAccess"),
        "{txt}"
    );
}

#[test]
fn the_c_include_guard_is_a_macro_name() {
    let h = run(&["gen-c"], &write_spec(SPEC, "c"));
    assert!(h.contains("#ifndef SANDBOX_HEALTH_H"), "got:\n{h}");
    assert!(h.contains("#define SANDBOX_HEALTH_H"), "got:\n{h}");
    for l in h
        .lines()
        .filter(|l| l.starts_with("#ifndef") || l.starts_with("#define S"))
    {
        assert!(!l.contains('.'), "guard carries a dot: {l}");
    }
}

#[test]
fn the_verilog_module_line_is_an_identifier_on_both_paths() {
    let p = write_spec(SPEC, "v");
    let v = run(&["gen-verilog"], &p);
    assert!(v.contains("module sandbox_health ("), "got:\n{v}");
    assert!(
        !v.contains("always @(*) begin\n        health;"),
        "stray statement:\n{v}"
    );
    let h = run(&["gen-verilog-hir"], &p);
    assert!(h.contains("module sandbox_health ("), "got:\n{h}");
    let tb = run(&["gen-testbench"], &p);
    assert!(
        tb.contains("sandbox_health uut ("),
        "DUT must match the module:\n{tb}"
    );
}

#[test]
fn a_dotted_use_resolves_like_the_path_form() {
    // use_resolve read `use pkg.dep;` as `specs/pkg.dep.t27` and pulled
    // nothing, while the parser stores the same import as `pkg::dep`.
    let root = scratch("resolve");
    let pkg = root.join("specs").join("pkg");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(
        pkg.join("dep.t27"),
        "module pkg.dep;\npub const Thing = struct { x: i32 };\nfn k() -> i32 { return 0; }\ntest t { assert k() == 0; }\n",
    )
    .unwrap();
    let user = |sep: &str| {
        let p = pkg.join(format!(
            "user{}.t27",
            if sep == "." { "_dot" } else { "_path" }
        ));
        std::fs::write(
            &p,
            format!(
                "module pkg.user;\nuse pkg{sep}dep;\nfn f(t: Thing) -> i32 {{ return t.x; }}\n"
            ),
        )
        .unwrap();
        run(&["gen"], &p)
    };
    let dotted = user(".");
    let path = user("::");
    assert!(
        path.contains("pub const Thing"),
        "control: the path form pulls Thing:\n{path}"
    );
    assert!(
        dotted.contains("pub const Thing"),
        "the dotted form must pull it too:\n{dotted}"
    );
}

#[test]
fn a_use_name_bound_as_a_local_is_not_imported() {
    // specs/sandbox/orphan_detection.t27: once `use sandbox.session;` is named
    // `session`, the parameter `session` and its `session.status` looked like
    // a module reference, and the Zig backend emitted
    // `const session = @import("session.zig");`, which the parameter shadows.
    let shadowed = run(
        &["gen"],
        &write_spec(
            "module a.b;\nuse sandbox.session;\nconst S = struct { x: i32 };\n\
             fn f(session: S) -> i32 { return session.x; }\n",
            "shadow",
        ),
    );
    assert!(
        !shadowed.contains("@import(\"session.zig\")"),
        "got:\n{shadowed}"
    );
    assert!(
        shadowed.contains("// use session: no references in this module"),
        "got:\n{shadowed}"
    );
    // Control: a real qualified call with no local of that name still imports.
    let used = run(
        &["gen"],
        &write_spec(
            "module a.b;\nuse sandbox.session;\nfn f() -> i32 { return session.open(); }\n",
            "used",
        ),
    );
    assert!(
        used.contains("const session = @import(\"session.zig\");"),
        "got:\n{used}"
    );
}
