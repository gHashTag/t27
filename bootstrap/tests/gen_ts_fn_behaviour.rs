//! Behavioural tests for `t27c gen-ts --fn` (#6559): does a lowered fn DO what
//! the spec's own test blocks say it does?
//!
//! The unit tests in `src/codegen_ts_fn.rs` read the emitted TEXT. Text can be
//! right to the character and still answer wrongly -- an `||` printed where the
//! spec said `&&` reads fine. So the acceptance case runs the spec's laws:
//! `specs/automation/agent-hive-group.t27` is generated through the real binary,
//! every `assert(...)` line of every `test` block is lifted out of the spec
//! TEXT (not out of the compiler's AST, so a parser and a lowering that agree
//! on a mistake cannot pass each other), and a harness evaluates each one
//! against the generated module under `node --experimental-strip-types`, or
//! `bun` when node is absent.
//!
//! A test that finds neither runtime still checks the text, then skips the
//! evaluation loudly -- an absent runtime is not a passing test. When `tsc` is
//! on PATH the artifact is also type-checked under `--strict`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn tool_present(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Per-test and per-process, so a failing run leaves its artifact where it can
/// be read again and two runs never delete each other's directory.
fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("t27c-gen-ts-fn-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn repo_path(rel: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.join(rel)
}

/// Run `t27c gen-ts [--fn] <path>`; panics with stderr on a non-zero exit.
fn gen_ts(path: &Path, lower_fns: bool) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_t27c"));
    cmd.arg("gen-ts");
    if lower_fns {
        cmd.arg("--fn");
    }
    let out = cmd.arg(path).output().expect("run t27c");
    assert!(
        out.status.success(),
        "`t27c gen-ts` failed on {}: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Every `test "<name>" { ... }` block of a spec, with the expression inside
/// each of its `assert(<expr>);` lines. Read from the text on purpose.
fn spec_asserts(spec: &str) -> Vec<(String, Vec<String>)> {
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();
    let mut open = false;
    for line in spec.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("test \"") {
            let name = rest.split('"').next().unwrap_or_default().to_string();
            blocks.push((name, Vec::new()));
            open = true;
            continue;
        }
        if open && t == "}" {
            open = false;
            continue;
        }
        if open {
            if let Some(expr) = t.strip_prefix("assert(").and_then(|r| r.strip_suffix(");")) {
                blocks.last_mut().unwrap().1.push(expr.to_string());
            } else if !t.is_empty() && !t.starts_with("//") {
                panic!("a test block line this harness does not read: {t:?}");
            }
        }
    }
    blocks
}

/// A t27 assert expression, as a JavaScript one. The spec's asserts use calls,
/// literals, `!`, `&&`, `||` and `==`; only `==` and `!=` differ, and anything
/// that would need more than that is refused rather than guessed at.
fn assert_to_js(expr: &str) -> String {
    assert!(
        expr.chars().all(|c| c.is_ascii_alphanumeric() || " _(),!=<>&|".contains(c)),
        "an assert this harness does not translate: {expr:?}"
    );
    expr.replace("!=", "\u{1}").replace("==", "===").replace('\u{1}', "!==")
}

/// The JavaScript runtime to evaluate with, and the arguments that make it
/// load a `.ts` file. `None` when there is neither.
fn ts_runtime() -> Option<(&'static str, Vec<&'static str>)> {
    if tool_present("node") {
        // Type stripping arrived behind this flag in node 22.6; a newer node
        // accepts the flag and does the same thing.
        let probe = Command::new("node")
            .args(["--experimental-strip-types", "-e", "0"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if probe {
            return Some(("node", vec!["--no-warnings", "--experimental-strip-types"]));
        }
    }
    if tool_present("bun") {
        return Some(("bun", vec!["run"]));
    }
    None
}

/// Write the artifact and a harness beside it, run the harness, and return
/// (exit ok, stdout + stderr). `None` when no runtime is available.
fn run_harness(artifact: &str, harness: &str, tag: &str) -> Option<(bool, String)> {
    let Some((tool, args)) = ts_runtime() else {
        eprintln!("SKIP {tag}: neither node (with type stripping) nor bun on PATH -- the text was checked, the behaviour was not");
        return None;
    };
    let dir = tmp_dir(tag);
    std::fs::write(dir.join("artifact.ts"), artifact).expect("write artifact");
    std::fs::write(dir.join("harness.ts"), harness).expect("write harness");
    let out = Command::new(tool)
        .args(&args)
        .arg(dir.join("harness.ts"))
        .current_dir(&dir)
        .output()
        .expect("run harness");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Some((out.status.success(), text))
}

fn exported_fns(artifact: &str) -> Vec<String> {
    artifact
        .lines()
        .filter_map(|l| l.strip_prefix("export function "))
        .map(|l| l.split('(').next().unwrap().to_string())
        .collect()
}

/// `tsc --strict --noEmit` over the artifact, when tsc is present.
fn tsc_strict(artifact: &str, tag: &str) {
    if !tool_present("tsc") {
        eprintln!("SKIP {tag}: no tsc on PATH -- the artifact was not type-checked");
        return;
    }
    let dir = tmp_dir(&format!("{tag}-tsc"));
    let file = dir.join("artifact.ts");
    std::fs::write(&file, artifact).expect("write artifact");
    let out = Command::new("tsc")
        .args(["--strict", "--noEmit", "--target", "es2022", "--module", "es2022"])
        .arg(&file)
        .output()
        .expect("run tsc");
    assert!(
        out.status.success(),
        "{tag}: tsc --strict refused the artifact:\n{}{}\n--- artifact ---\n{artifact}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

const HIVE: &str = "specs/automation/agent-hive-group.t27";

#[test]
fn every_fn_of_agent_hive_group_lowers_and_only_the_tests_stay_listed() {
    let artifact = gen_ts(&repo_path(HIVE), true);
    let fns = exported_fns(&artifact);
    assert_eq!(fns.len(), 19, "agent-hive-group declares 19 fns; lowered: {fns:?}");
    assert!(!artifact.contains("{ what: \"fn "), "a fn is still listed as not emitted:\n{artifact}");
    let listed = artifact.matches("{ what: ").count();
    let tests = artifact.matches("{ what: \"test ").count();
    assert_eq!(tests, 16, "all 16 test blocks are listed as not emitted");
    assert_eq!(listed, tests, "only the test blocks remain in __NOT_EMITTED__:\n{artifact}");
    tsc_strict(&artifact, "hive");
}

#[test]
fn agent_hive_group_lowered_fns_pass_every_assert_of_the_spec() {
    let spec = std::fs::read_to_string(repo_path(HIVE)).expect("read spec");
    let blocks = spec_asserts(&spec);
    assert_eq!(blocks.len(), 16, "agent-hive-group has 16 test blocks");
    let total: usize = blocks.iter().map(|(_, a)| a.len()).sum();
    assert!(blocks.iter().all(|(_, a)| !a.is_empty()), "every test block holds an assert");

    let artifact = gen_ts(&repo_path(HIVE), true);
    // An assert may read a const as well as call a fn, so the harness imports
    // every name the artifact exports except its own `__...__` bookkeeping.
    let mut names = exported_fns(&artifact);
    names.extend(
        artifact
            .lines()
            .filter_map(|l| l.strip_prefix("export const "))
            .map(|l| l.split(' ').next().unwrap().to_string())
            .filter(|n| !n.starts_with("__")),
    );
    let mut harness = format!("import {{ {} }} from \"./artifact.ts\";\n", names.join(", "));
    harness.push_str("let passed = 0;\nconst failed: string[] = [];\n");
    harness.push_str(
        "function check(test: string, src: string, f: () => boolean): void {\n  \
         let ok = false;\n  \
         try { ok = f() === true; } catch (e) { failed.push(test + \": \" + src + \" threw \" + e); return; }\n  \
         if (ok) { passed += 1; } else { failed.push(test + \": \" + src); }\n}\n",
    );
    for (name, asserts) in &blocks {
        for a in asserts {
            harness.push_str(&format!(
                "check({:?}, {:?}, () => ({}));\n",
                name,
                a,
                assert_to_js(a)
            ));
        }
    }
    harness.push_str(
        "for (const f of failed) { console.log(\"FAIL \" + f); }\n\
         console.log(\"PASSED \" + passed + \" FAILED \" + failed.length);\n\
         if (failed.length > 0) { process.exit(1); }\n",
    );

    let Some((ok, said)) = run_harness(&artifact, &harness, "hive-asserts") else {
        return;
    };
    assert!(ok, "the lowered fns broke the spec's own laws:\n{said}\n--- harness ---\n{harness}");
    assert!(
        said.contains(&format!("PASSED {total} FAILED 0")),
        "expected all {total} asserts to run and pass, the harness said:\n{said}"
    );
}

#[test]
fn integer_arithmetic_throws_at_its_width_instead_of_wrapping() {
    let dir = tmp_dir("width");
    let spec = dir.join("in.t27");
    std::fs::write(
        &spec,
        "module width {\n\
         pub fn add(a: u8, b: u8) -> u8 {\n    return a + b;\n}\n\
         pub fn half(a: i32) -> i32 {\n    return a / 2;\n}\n\
         pub fn back(a: u32) -> u32 {\n    return a - 1;\n}\n}\n",
    )
    .expect("write spec");
    let artifact = gen_ts(&spec, true);
    assert_eq!(exported_fns(&artifact), ["add", "half", "back"], "{artifact}");
    assert!(artifact.contains("function __t27_int("), "{artifact}");
    tsc_strict(&artifact, "width");
    let harness = "import { add, half, back } from \"./artifact.ts\";\n\
        function throws(f: () => number): boolean { try { f(); return false; } catch (e) { return e instanceof RangeError; } }\n\
        const ok = add(200, 55) === 255 && throws(() => add(200, 56))\n  \
          && half(-7) === -3 && half(7) === 3\n  \
          && back(1) === 0 && throws(() => back(0));\n\
        console.log(ok ? \"WIDTH OK\" : \"WIDTH WRONG\");\n\
        if (!ok) { process.exit(1); }\n";
    let Some((ok, said)) = run_harness(&artifact, harness, "width") else {
        return;
    };
    assert!(ok && said.contains("WIDTH OK"), "{said}\n--- artifact ---\n{artifact}");
}

#[test]
fn a_fn_outside_the_subset_is_listed_with_its_reason_and_its_callers_too() {
    let dir = tmp_dir("refuse");
    let spec = dir.join("in.t27");
    std::fs::write(
        &spec,
        "module refuse {\n\
         pub fn count(n: u32) -> u32 {\n    var i : u32 = 0;\n    while (i < n) {\n        i = i + 1;\n    }\n    return i;\n}\n\
         pub fn calls(n: u32) -> bool {\n    return count(n) == n;\n}\n\
         pub fn fine(a: bool) -> bool {\n    return !a;\n}\n}\n",
    )
    .expect("write spec");
    let artifact = gen_ts(&spec, true);
    assert_eq!(exported_fns(&artifact), ["fine"], "{artifact}");
    assert!(artifact.contains("{ what: \"fn count\""), "{artifact}");
    assert!(artifact.contains("{ what: \"fn calls\""), "{artifact}");
    assert!(artifact.contains("names no lowered fn"), "{artifact}");
    assert!(!artifact.contains("__t27_int"), "no arithmetic was lowered, so no helper: {artifact}");
}

#[test]
fn without_the_flag_no_fn_is_lowered_and_every_one_is_listed() {
    let artifact = gen_ts(&repo_path(HIVE), false);
    assert!(exported_fns(&artifact).is_empty(), "{artifact}");
    assert_eq!(artifact.matches("{ what: \"fn ").count(), 19, "{artifact}");
    assert_eq!(artifact.matches("{ what: \"test ").count(), 16, "{artifact}");
    assert!(artifact.contains(
        "// t27c gen-ts: fn addressed was not emitted -- this backend lowers declarations, not bodies.\n"
    ));
}

/// Generate `src` with `--fn`, assert which fns lowered and that each refused
/// one is listed with `reason`, then load the artifact and call `half(7)`
/// when it lowered: the refusal must leave a module that still runs.
fn refused_and_still_runs(tag: &str, src: &str, lowered: &[&str], refused: &[&str], reason: &str) {
    let dir = tmp_dir(tag);
    let spec = dir.join("in.t27");
    std::fs::write(&spec, src).expect("write spec");
    let artifact = gen_ts(&spec, true);
    assert_eq!(exported_fns(&artifact), lowered, "{artifact}");
    for name in refused {
        let line = artifact
            .lines()
            .find(|l| l.contains(&format!("{{ what: \"fn {name}\"")))
            .unwrap_or_else(|| panic!("fn {name} is not listed: {artifact}"));
        assert!(line.contains(reason), "fn {name} is listed without {reason:?}: {line}");
    }
    tsc_strict(&artifact, tag);
    let call = if lowered.contains(&"half") {
        "import { half } from \"./artifact.ts\";\nconst ok = half(7) === 3 && half(-7) === -3;\n"
    } else {
        "import * as m from \"./artifact.ts\";\nconst ok = Array.isArray(m.__NOT_EMITTED__);\n"
    };
    let harness = format!(
        "{call}console.log(ok ? \"RUNS OK\" : \"RUNS WRONG\");\nif (!ok) {{ process.exit(1); }}\n"
    );
    let Some((ok, said)) = run_harness(&artifact, &harness, tag) else {
        return;
    };
    assert!(ok && said.contains("RUNS OK"), "{said}\n--- artifact ---\n{artifact}");
}

const HALF: &str = "pub fn half(a: i32) -> i32 {\n    return a / 2;\n}\n";

#[test]
fn a_param_named_after_a_global_the_code_uses_is_refused() {
    // Lowered, `Math / 2` became `Math.trunc(Math / 2)` on the parameter.
    let src = format!("module g {{\n{HALF}pub fn shadow(Math: i32) -> i32 {{\n    return Math / 2;\n}}\n}}\n");
    refused_and_still_runs("param-global", &src, &["half"], &["shadow"], "would hide the JavaScript global");
}

#[test]
fn a_param_with_the_helper_prefix_is_refused() {
    let src = format!(
        "module h {{\n{HALF}pub fn add(__t27_int: u8, b: u8) -> u8 {{\n    return __t27_int + b;\n}}\n}}\n"
    );
    refused_and_still_runs("param-helper", &src, &["half"], &["add"], "prefix __t27");
}

#[test]
fn a_fn_named_after_a_global_refuses_the_whole_module() {
    // A module-level `function Math` hides the global from every other fn.
    let src = format!("module f {{\n{HALF}pub fn Math(a: bool) -> bool {{\n    return !a;\n}}\n}}\n");
    refused_and_still_runs("fn-global", &src, &[], &["half", "Math"], "the module declares");
}

#[test]
fn a_const_named_after_a_global_refuses_the_whole_module() {
    for (tag, decl) in [
        ("const-number", "pub const Number : u8 = 1;\n"),
        ("const-range", "pub const RangeError : u8 = 1;\n"),
        ("const-helper", "pub const __t27_int : u8 = 1;\n"),
    ] {
        let src = format!(
            "module c {{\n{decl}{HALF}pub fn add(a: u8, b: u8) -> u8 {{\n    return a + b;\n}}\n}}\n"
        );
        refused_and_still_runs(tag, &src, &[], &["half", "add"], "the module declares");
    }
}

#[test]
fn a_param_with_a_module_declarations_name_is_refused() {
    let src = format!(
        "module p {{\npub const k : i32 = 3;\n{HALF}pub fn over(k: i32) -> i32 {{\n    return k / 2;\n}}\n}}\n"
    );
    refused_and_still_runs("param-const", &src, &["half"], &["over"], "has the name of a module declaration");
}

#[test]
fn an_integer_literal_beyond_two_to_the_53_is_refused() {
    let src = format!(
        "module b {{\n{HALF}pub fn same() -> bool {{\n    return 9007199254740993 == 9007199254740992;\n}}\n}}\n"
    );
    refused_and_still_runs("literal-53", &src, &["half"], &["same"], "at line 6 is beyond 2^53 - 1");
}

#[test]
fn a_fn_or_param_named_like_what_the_artifact_exports_is_refused() {
    // Each is a second binding of a name the artifact always exports:
    // node said `SyntaxError: Identifier '...' has already been declared`.
    let fn_named = format!("module e {{\n{HALF}pub fn __NOT_EMITTED__() -> bool {{\n    return true;\n}}\n}}\n");
    refused_and_still_runs("fn-not-emitted", &fn_named, &[], &["half", "__NOT_EMITTED__"], "the module declares");
    let fn_order = format!("module o {{\n{HALF}pub fn __DECL_ORDER__(x: u8) -> u8 {{\n    return x;\n}}\n}}\n");
    refused_and_still_runs("fn-decl-order", &fn_order, &[], &["half", "__DECL_ORDER__"], "the module declares");
    for name in ["__NOT_EMITTED__", "__DECL_ORDER__", "__STRUCT_ORDER__"] {
        let param = format!("module q {{\n{HALF}pub fn id({name}: u8) -> u8 {{\n    return {name};\n}}\n}}\n");
        refused_and_still_runs(
            &format!("param-{name}"),
            &param,
            &["half"],
            &["id"],
            "is a name every gen-ts artifact already exports",
        );
    }
}
