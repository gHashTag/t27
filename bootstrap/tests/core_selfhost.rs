//! The self-hosted compiler core (#5981, epic #5980) against the compiler it
//! replaces.
//!
//! `specs/compiler/core/t27core.t27` is a t27 compiler written in t27. Inside
//! the subset it accepts, it must write exactly the bytes `t27c gen-c` writes,
//! and outside that subset it must refuse. The checks, cheapest claim first:
//!
//! 1. its own `test` blocks pass when gen-c's output is built with
//!    `-DT27_TEST_MAIN`;
//! 2. **fixpoint**: the core compiled by gen-c, run on its own source, writes
//!    gen-c's output byte for byte -- `core(core.t27) == gen-c(core.t27)`;
//! 3. every fixture (below, and the `.t27` data under `fixtures/core_selfhost`)
//!    compiles to gen-c's bytes, and those bytes are C
//!    that builds under `-std=c99 -Werror=implicit-function-declaration`;
//!    a fixture with `test` blocks is linked with `-DT27_TEST_MAIN` and run,
//!    so a value gen-c drops is a failed assertion, not a quiet zero;
//! 4. every refusal below is refused, with the stated code -- each one is a
//!    shape gen-c lowers with loss, so agreeing with gen-c there would be
//!    agreeing with a defect;
//!    Where gen-c refuses the same shape, `BOTH_REFUSE` says so too;
//! 5. over the whole `specs/` corpus, every file the core accepts compiles to
//!    gen-c's bytes, and the core accepts at least `CORPUS_FLOOR` of them, so
//!    the check cannot pass by refusing everything.
//!
//! The C driver below is plumbing: it reads stdin into the core's `src`, calls
//! `compile`, and writes `out`. Nothing in it decides an output byte.
//!
//! Needs a C compiler (`cc`); without one the test says so and passes, as the
//! other cc-dependent tests here do.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;

const DRIVER: &str = r#"#include <stdio.h>
#include "core.c"
int main(void) {
    size_t n = fread(src, 1, SRC_MAX, stdin);
    if (n == SRC_MAX && fgetc(stdin) != EOF) {
        fprintf(stderr, "t27core: input exceeds SRC_MAX\n");
        return 2;
    }
    int64_t r = compile((int64_t)n);
    if (r < 0) {
        fprintf(stderr, "t27core: error %lld at byte %lld\n", (long long)err_code, (long long)err_pos);
        return 1;
    }
    fwrite(out, 1, (size_t)r, stdout);
    return 0;
}
"#;

/// The floor on corpus files the core accepts; 58 when this test landed.
const CORPUS_FLOOR: usize = 50;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn work_dir() -> PathBuf {
    let d = std::env::temp_dir().join(format!("t27-core-selfhost-{}", std::process::id()));
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

fn have_cc() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(path: &Path) -> Option<Vec<u8>> {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(path)
        .output()
        .expect("run t27c gen-c");
    if out.status.success() { Some(out.stdout) } else { None }
}

/// Runs the built core on `src`: Ok(C bytes) or Err(error code).
fn run_core(core: &Path, src: &[u8]) -> Result<Vec<u8>, i64> {
    let mut child = Command::new(core)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run core");
    // The core refuses a spec longer than SRC_MAX after reading SRC_MAX + 1
    // bytes and exits, so the rest of a large spec meets a closed pipe. That
    // refusal is the verdict; a failed write is not (#6508).
    let _ = child.stdin.take().unwrap().write_all(src);
    let out = child.wait_with_output().expect("core output");
    if out.status.success() {
        return Ok(out.stdout);
    }
    let msg = String::from_utf8_lossy(&out.stderr);
    let code = msg
        .split_whitespace()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(-1);
    Err(code)
}

fn cc(args: &[&str], dir: &Path) {
    let st = Command::new("cc").args(args).current_dir(dir).status().expect("run cc");
    assert!(st.success(), "cc {:?} failed", args);
}

/// Programs inside the subset, chosen to reach every emitter path.
const FIXTURES: &[&str] = &[
    "module m;\n",
    "module m;\nendmodule\n",
    // #6047: `endmodule` last still compiles, and keeps what comes before it.
    "module kept;\nfn kept() -> i64 {\n    return 1;\n}\nendmodule\nendmodule\n// trailing comment\n",
    "// header\nmodule shapes;\n\
     const A: i64 = 5;\nconst B: i64 = -5;\nconst C: i64 = (-5);\nconst D: i64 = (7);\n\
     pub const E: bool = true;\nconst F: i64 = A + B * 2;\nconst G: u8 = 0x1F;\n\
     let H: i64 = F;\nvar v: u8 = 'x';\nvar n: i64 = -A;\n\
     var buf: [16]u8 = [_]u8{0} ** 16;\nvar big: [A]i64 = [_]i64{0} ** A;\n",
    "module flow;\n\
     fn empty() -> void {\n}\n\
     pub fn sign(x: i64) -> i64 {\n    if (x > 0) {\n        return 1;\n    } else if (x < 0) {\n        return -1;\n    } else {\n        return 0;\n    }\n}\n\
     fn loop_(n: usize, k: u16) -> bool {\n    var i: usize = 0;\n    while (i < n) {\n        if (i == 3) {\n            break;\n        }\n        if (i == 1) {\n            continue;\n        } else {\n        }\n        i = i + 1;\n    }\n    return !(i >= n) and ~k != 0 or false;\n}\n\
     fn ops(a: i32, b: u64) -> u32 {\n    var c: i16 = (a << 2 >> 1 | 3 ^ 5 & 7) as i16;\n    empty();\n    return (a % 3 + b / 2 - c * 1) as u32;\n}\n",
    "module tested;\n\
     fn twice(x: i64) -> i64 {\n    return x * 2;\n}\n\
     test doubles {\n    var r: i64 = twice(4);\n    r = r + 1;\n    assert(r == 9);\n    assert_eq(twice(1), 2);\n}\n\
     test todo {\n}\n",
    // #6046: a unary over the same unary keeps its space, `- -x`, never `--x`.
    "module neg;\n\
     fn f(x: i64) -> i64 {\n    var y: i64 = - -x;\n    y = y + - -1;\n    return y;\n}\n\
     fn g(b: bool, k: u8) -> bool {\n    return ! !b and ~ ~k == ~ ~k;\n}\n\
     test double_unary {\n    assert(f(4) == 5);\n    assert(g(true, 3));\n}\n",
    // #6048, #6049: a const initializer is the whole expression, and a char
    // const is emitted; the test reads every value back in C.
    "module consts;\n\
     const A: u8 = 5 as u8;\nconst B: bool = true and false;\nconst C: i64 = ~5;\n\
     const D: bool = !true;\nconst E: i64 = -5 as i64;\nconst S: bool = false or true;\n\
     const Q: u8 = 'a';\nconst NL: u8 = '\\n';\nconst AP: u8 = '\\'';\nconst Z: u8 = 'z' - 25;\n\
     test values {\n    assert(A == 5);\n    assert(!B);\n    assert(C == -6);\n    assert(!D);\n\
     \x20   assert(E == -5);\n    assert(S);\n    assert(Q == 97);\n    assert(NL == 10);\n\
     \x20   assert(AP == 39);\n    assert(Z == 'a');\n}\n",
    // #6051: `assert` outside a test still gets `<assert.h>`; built with
    // -Werror=implicit-function-declaration like every fixture.
    "module asrt;\nfn f(x: i64) -> i64 {\n    assert(x > 0);\n    assert_eq(x, x);\n    return x;\n}\n",
    // #6052: a test's write to a module `var` reaches the global; a
    // function reading it sees the new value when the test runs in C.
    "module gw;\nvar g: i64 = 0;\nvar flag: bool = false;\n\
     fn get() -> i64 {\n    return g;\n}\n\
     test writes_global {\n    g = 5;\n    flag = true;\n    assert(get() == 5);\n    assert(flag);\n}\n",
    // #6050: a global repeat keeps its value; `{0}` only for a zero one.
    "module rep;\n\
     const N: i64 = 4;\nvar sev: [N]u8 = [_]u8{7} ** N;\nvar five: [4]u8 = [_]u8{5} ** 4;\n\
     var z3: [3]u8 = [_]u8{0} ** 3;\nvar hx: [2]u16 = [_]u16{0x10} ** 2;\nvar hz: [2]u16 = [_]u16{0x00} ** 2;\n\
     test filled {\n    assert(sev[0] == 7);\n    assert(sev[3] == 7);\n    assert(five[3] == 5);\n\
     \x20   assert(z3[2] == 0);\n    assert(hx[1] == 16);\n    assert(hz[1] == 0);\n}\n",
    "module c;\n; prose line at column 1\n/* block /* nested */ still comment */\n# hash comment\n\
     fn f(a: u8) -> u8 {\n    return a && 1 || '\\n' == '\\'';\n}\n",
];

/// Shapes gen-c lowers with loss, and the code the core refuses them with.
const REFUSALS: &[(&str, i64)] = &[
    ("module m;\nfn f() -> u8 { return \"s\"; }\n", 1),
    ("module m;\nconst X: i64 = 2.5;\n", 1),
    ("module m;\nstruct S { a: u8 }\n", 1),
    ("module m;\nendmodule\nfn lost() {}\n", 2),
    ("module m;\nfn f() { g(); }\n", 4),
    ("module m;\nfn f() { assert(true, true); }\n", 4),
    ("module m;\nfn f() { assert_eq(1); }\n", 4),
    ("module m;\nfn _x() {}\nfn f() { _ = 1; }\n", 5),
    ("module m;\nfn assert_eq() {}\n", 5),
    ("module m;\ntest t { x = 1; }\n", 6),
    ("module m;\nvar b: [2]u8 = [_]u8{0} ** 2;\ntest t { b = 1; }\n", 6),
    ("module m;\nconst B: u8 = 1;\nconst A: u8 = B[0];\n", 7),
    ("module m;\nvar a: [4]u8 = [_]u8{x} ** 4;\n", 9),
];

/// Fixtures and refusals kept as t27 data, so a new shape needs no new Rust:
/// `fixtures/core_selfhost/accept/*.t27` join FIXTURES, and
/// `fixtures/core_selfhost/refuse/e<code>_<name>.t27` join REFUSALS.
/// `genc/*.t27` are gen-c repros the core need not take: built and run, not compared.
/// `both_refuse/*.t27` join BOTH_REFUSE.
fn data_files(sub: &str) -> Vec<(String, String)> {
    let d = repo_root().join("bootstrap/tests/fixtures/core_selfhost").join(sub);
    let mut v: Vec<(String, String)> = std::fs::read_dir(&d)
        .unwrap_or_else(|_| panic!("missing {}", d.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("t27"))
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()))
        .collect();
    v.sort();
    v
}

/// Refusals gen-c makes too: agreeing with gen-c here is agreeing with a
/// refusal, not with a loss.
const BOTH_REFUSE: &[&str] = &[
    // #6047: text after `endmodule` used to vanish with exit 0.
    "module m;\nendmodule\nfn lost() {}\n",
    "module m;\nendmodule\nfn lost() -> i64 {\n    return 1;\n}\n",
];

#[test]
fn core_compiles_itself_and_agrees_with_gen_c() {
    if !have_cc() {
        eprintln!("core_selfhost: no `cc` on PATH; skipped");
        return;
    }
    let root = repo_root();
    let spec = root.join("specs/compiler/core/t27core.t27");
    let dir = work_dir();

    // 1. gen-c builds the core; its own tests pass.
    let core_c = gen_c(&spec).expect("gen-c compiles t27core.t27");
    std::fs::write(dir.join("core.c"), &core_c).unwrap();
    std::fs::write(dir.join("driver.c"), DRIVER).unwrap();
    cc(&["-O1", "-w", "-DT27_TEST_MAIN", "-o", "coretest", "core.c"], &dir);
    let st = Command::new(dir.join("coretest")).status().expect("run core tests");
    assert!(st.success(), "t27core's own test blocks failed");
    cc(&["-O1", "-w", "-o", "core", "driver.c"], &dir);
    let core = dir.join("core");

    // 2. Fixpoint.
    let own = std::fs::read(&spec).unwrap();
    let self_c = run_core(&core, &own).expect("t27core accepts its own source");
    assert!(self_c == core_c, "fixpoint broken: core(t27core.t27) != gen-c(t27core.t27)");

    // 3. Fixtures.
    let fixtures: Vec<(bool, String)> = FIXTURES.iter().map(|s| (true, s.to_string()))
        .chain(data_files("accept").into_iter().map(|(_, s)| (true, s)))
        .chain(data_files("genc").into_iter().map(|(_, s)| (false, s)))
        .collect();
    for (i, (core_too, src)) in fixtures.iter().enumerate() {
        let path = dir.join(format!("fixture{}.t27", i));
        std::fs::write(&path, src).unwrap();
        let want = gen_c(&path).unwrap_or_else(|| panic!("gen-c refused fixture {}", i));
        if *core_too {
            let got = run_core(&core, src.as_bytes())
                .unwrap_or_else(|c| panic!("core refused fixture {} with code {}:\n{}", i, c, src));
            assert!(got == want, "fixture {} differs from gen-c:\n{}", i, src);
        }
        let cname = format!("fixture{}.c", i);
        std::fs::write(dir.join(&cname), &want).unwrap();
        let strict = ["-std=c99", "-Werror=implicit-function-declaration"];
        if src.contains("\ntest ") {
            let exe = format!("fixture{}t", i);
            let mut args = strict.to_vec();
            args.extend(["-DT27_TEST_MAIN", "-o", exe.as_str(), cname.as_str()]);
            cc(&args, &dir);
            let st = Command::new(dir.join(&exe)).status().expect("run fixture tests");
            assert!(st.success(), "fixture {}'s tests failed:\n{}", i, src);
        } else {
            let obj = format!("fixture{}.o", i);
            let mut args = strict.to_vec();
            args.extend(["-c", "-o", obj.as_str(), cname.as_str()]);
            cc(&args, &dir);
        }
    }

    // 4. Refusals.
    let refusals: Vec<(String, i64)> = REFUSALS.iter().map(|(s, c)| (s.to_string(), *c))
        .chain(data_files("refuse").into_iter().map(|(n, s)| {
            let code: i64 = n.split('_').next().unwrap().trim_start_matches('e').parse()
                .unwrap_or_else(|_| panic!("refusal {} is not named e<code>_<name>.t27", n));
            (s, code)
        }))
        .collect();
    for (src, code) in &refusals {
        match run_core(&core, src.as_bytes()) {
            Ok(_) => panic!("core accepted a lossy shape:\n{}", src),
            Err(c) => assert_eq!(c, *code, "wrong refusal code for:\n{}", src),
        }
    }

    let both: Vec<String> = BOTH_REFUSE.iter().map(|s| s.to_string())
        .chain(data_files("both_refuse").into_iter().map(|(_, s)| s))
        .collect();
    for (i, src) in both.iter().enumerate() {
        let path = dir.join(format!("refused{}.t27", i));
        std::fs::write(&path, src).unwrap();
        assert!(gen_c(&path).is_none(), "gen-c accepted a refused shape:\n{}", src);
        assert!(run_core(&core, src.as_bytes()).is_err(), "core accepted:\n{}", src);
    }

    // 5. Corpus differential.
    let mut stack = vec![root.join("specs")];
    let mut accepted = 0usize;
    let mut mismatched = Vec::new();
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) != Some("t27") {
                continue;
            }
            let src = std::fs::read(&p).unwrap();
            if let Ok(got) = run_core(&core, &src) {
                accepted += 1;
                if gen_c(&p).as_deref() != Some(&got[..]) {
                    mismatched.push(p.strip_prefix(&root).unwrap().display().to_string());
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(mismatched.is_empty(), "core and gen-c disagree on: {:?}", mismatched);
    assert!(
        accepted >= CORPUS_FLOOR,
        "core accepted only {} corpus specs (floor {})",
        accepted,
        CORPUS_FLOOR
    );
}
