//! `t27b`: t27's own native backend (MVP), command line.
//!
//! ```text
//! t27b test   <file.t27> [--overflow trap|wrap] [--time] [--quiet] [--check]
//! t27b build  <file.t27> -o <out.o> [--overflow trap|wrap] [--time]
//! t27b asm    <file.t27> [--overflow trap|wrap]
//! t27b corpus <dir> [--timeout-ms N] [--jobs N] [--overflow trap|wrap] [--list]
//! ```
//!
//! Exit codes: 0 success; 1 a test failed; 2 unsupported construct;
//! 3 front-end (parse or typecheck) error; 4 JIT and interpreter disagree
//! (`--check`); 5 code generation limit; 64 usage; 74 I/O error.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use t27b::codegen::{self, TrapStyle};
use t27b::eval::{Interp, Stop};
use t27b::ir::{OverflowMode, Program, Ty};
use t27b::jit::Jit;
use t27b::timing::Phases;
use t27b::{a64, front, lower, macho};

const EXIT_FAIL: u8 = 1;
const EXIT_UNSUPPORTED: u8 = 2;
const EXIT_FRONTEND: u8 = 3;
const EXIT_MISMATCH: u8 = 4;
const EXIT_CODEGEN: u8 = 5;
const EXIT_USAGE: u8 = 64;
const EXIT_IO: u8 = 74;

/// Write to stdout, ignoring errors such as a closed pipe (`t27b asm | head`),
/// so output never panics.
fn out(s: &str) {
    use std::io::Write;
    let mut o = std::io::stdout().lock();
    let _ = o.write_all(s.as_bytes());
    let _ = o.flush();
}

macro_rules! outln {
    ($($t:tt)*) => { out(&format!("{}\n", format!($($t)*))) };
}

/// Like eprintln!, without the panic on a write error.
macro_rules! errln {
    ($($t:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stderr().lock(), $($t)*);
    }};
}

const USAGE: &str = "usage:
  t27b test   <file.t27> [--overflow trap|wrap] [--time] [--quiet] [--check]
  t27b build  <file.t27> -o <out.o> [--overflow trap|wrap] [--time]
  t27b asm    <file.t27> [--overflow trap|wrap]
  t27b corpus <dir> [--timeout-ms N] [--jobs N] [--overflow trap|wrap] [--list]";

struct Opts {
    cmd: String,
    input: Option<String>,
    out: Option<String>,
    mode: OverflowMode,
    time: bool,
    quiet: bool,
    check: bool,
    list: bool,
    timeout_ms: u64,
    jobs: usize,
}

fn usage(msg: &str) -> ExitCode {
    if !msg.is_empty() {
        errln!("t27b: {}", msg);
    }
    errln!("{}", USAGE);
    ExitCode::from(EXIT_USAGE)
}

fn parse_args() -> Result<Opts, String> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().ok_or("missing command")?;
    let mut o = Opts {
        cmd,
        input: None,
        out: None,
        mode: OverflowMode::Trap,
        time: false,
        quiet: false,
        check: false,
        list: false,
        timeout_ms: 10_000,
        jobs: std::thread::available_parallelism().map_or(4, |n| n.get()),
    };
    while let Some(a) = args.next() {
        match a.as_str() {
            "-o" => o.out = Some(args.next().ok_or("-o needs a path")?),
            "--overflow" => {
                o.mode = match args.next().as_deref() {
                    Some("trap") => OverflowMode::Trap,
                    Some("wrap") => OverflowMode::Wrap,
                    _ => return Err("--overflow takes trap or wrap".into()),
                }
            }
            "--time" => o.time = true,
            "--quiet" | "-q" => o.quiet = true,
            "--check" => o.check = true,
            "--list" => o.list = true,
            "--timeout-ms" => {
                o.timeout_ms = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--timeout-ms takes a number")?
            }
            "--jobs" | "-j" => {
                o.jobs = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .filter(|&n| n > 0)
                    .ok_or("--jobs takes a positive number")?
            }
            s if s.starts_with('-') => return Err(format!("unknown option {}", s)),
            s => {
                if o.input.is_some() {
                    return Err(format!("unexpected argument {}", s));
                }
                o.input = Some(s.to_string());
            }
        }
    }
    Ok(o)
}

fn main() -> ExitCode {
    let o = match parse_args() {
        Ok(o) => o,
        Err(e) => return usage(&e),
    };
    match o.cmd.as_str() {
        "test" | "build" | "asm" => {
            let Some(input) = o.input.clone() else {
                return usage("missing input file");
            };
            match compile_front(&input, &o) {
                Ok((prog, mut ph)) => match o.cmd.as_str() {
                    "test" => cmd_test(&prog, &mut ph, &o),
                    "build" => cmd_build(&prog, &mut ph, &o),
                    _ => cmd_asm(&prog),
                },
                Err(code) => ExitCode::from(code),
            }
        }
        "corpus" => {
            let Some(dir) = o.input.clone() else {
                return usage("missing directory");
            };
            cmd_corpus(Path::new(&dir), &o)
        }
        "help" | "--help" | "-h" => {
            outln!("{}", USAGE);
            ExitCode::SUCCESS
        }
        other => usage(&format!("unknown command {}", other)),
    }
}

/// read -> parse -> typecheck -> lower. Prints diagnostics; returns the exit
/// code on failure.
fn compile_front(input: &str, o: &Opts) -> Result<(Program, Phases), u8> {
    let mut ph = Phases::default();
    let path = PathBuf::from(input);
    let src = ph.time("read", || std::fs::read_to_string(&path));
    let src = match src {
        Ok(s) => s,
        Err(e) => {
            errln!("t27b: cannot read {}: {}", input, e);
            return Err(EXIT_IO);
        }
    };
    let parsed = ph.time("parse", || front::parse(&path, &src));
    let parsed = match parsed {
        Ok(p) => p,
        Err(e) => {
            errln!("t27b: parse error in {}: {}", input, e);
            return Err(EXIT_FRONTEND);
        }
    };
    let tc = ph.time("typecheck", || front::typecheck(&parsed.ast));
    if let Err(errs) = tc {
        for e in &errs {
            errln!("t27b: typecheck error in {}: {}", input, e);
        }
        return Err(EXIT_FRONTEND);
    }
    let lowered = ph.time("lower", || lower::lower_src(&parsed.ast, o.mode, Some(&src)));
    match lowered {
        Ok(p) => Ok((p, ph)),
        Err(rejects) => {
            for r in &rejects {
                errln!("{}", r.message());
            }
            if o.time {
                errln!("{}", ph.report());
            }
            Err(EXIT_UNSUPPORTED)
        }
    }
}

fn codegen_error(e: &codegen::CodegenError) -> ExitCode {
    errln!(
        "t27b: unsupported construct {} at line {} (fn {}: {})",
        e.construct, e.line, e.func, e.detail
    );
    ExitCode::from(EXIT_CODEGEN)
}

/// Render a raw register image of type `ty` for messages.
fn show(ty: Ty, raw: u64) -> String {
    let v = ty.from_raw(raw);
    if ty == Ty::Bool {
        (v != 0).to_string()
    } else {
        v.to_string()
    }
}

fn cmd_test(prog: &Program, ph: &mut Phases, o: &Opts) -> ExitCode {
    let code = ph.time("codegen", || codegen::compile(prog, TrapStyle::Jit, true));
    let code = match code {
        Ok(c) => c,
        Err(e) => return codegen_error(&e),
    };
    let jit = ph.time("jit-map", || Jit::load(&code, prog.funcs.len(), &prog.data));
    let mut jit = match jit {
        Ok(j) => j,
        Err(e) => {
            errln!("t27b: {}", e);
            return ExitCode::from(EXIT_IO);
        }
    };
    let mut passed = 0usize;
    let mut failed = 0usize;
    let (mut held, mut broken) = (0usize, 0usize);
    let mut mismatches = 0usize;
    let mut lines: Vec<String> = Vec::new();
    let t0 = Instant::now();
    for (id, f) in prog.tests() {
        let r = jit.call(id as u32, &[]);
        // An invariant runs exactly like a test and is reported apart from
        // the tests, prefixed `INVARIANT`.
        let tag = if f.is_invariant { "INVARIANT " } else { "" };
        match r {
            Ok(_) => {
                if f.is_invariant {
                    held += 1;
                } else {
                    passed += 1;
                }
                if !o.quiet {
                    lines.push(format!("{}PASS {}", tag, f.name));
                }
            }
            Err(t) => {
                if f.is_invariant {
                    broken += 1;
                } else {
                    failed += 1;
                }
                let site = &prog.sites[t.site as usize];
                let mut msg = format!("{}FAIL {}: {} at line {}", tag, f.name, site.kind.describe(), site.line);
                if site.kind == t27b::ir::TrapKind::AssertEq {
                    msg.push_str(&format!(" (left {}, right {})", show(site.ty, t.a), show(site.ty, t.b)));
                } else if !site.what.is_empty() {
                    msg.push_str(&format!(" ({})", site.what));
                }
                lines.push(msg);
            }
        }
        if o.check {
            // Cross-check against the reference interpreter.
            let mut it = Interp::new(prog);
            let want = it.call(id, &[]);
            let agree = match (&r, &want) {
                (Ok(_), Ok(_)) => true,
                (Err(t), Err(Stop::Trap { site, a, b })) => {
                    let ty = prog.sites[*site as usize].ty;
                    t.site == *site
                        && (prog.sites[*site as usize].kind != t27b::ir::TrapKind::AssertEq
                            || (ty.from_raw(t.a) == *a && ty.from_raw(t.b) == *b))
                }
                (_, Err(Stop::Fuel)) | (_, Err(Stop::Depth)) => true,
                // An interpreter fault is a lowering defect: never agreement.
                (_, Err(Stop::Fault(_))) => false,
                _ => false,
            };
            if !agree {
                mismatches += 1;
                lines.push(format!("MISMATCH {}{}: jit {:?}, interpreter {:?}", tag, f.name, r, want));
            }
        }
    }
    if !o.quiet {
        for n in &prog.unchecked {
            lines.push(format!("INVARIANT NOT CHECKED {}: the front-end discarded its body", n));
        }
    }
    let run_ms = t0.elapsed().as_secs_f64() * 1e3;
    ph.entries.push(("run", run_ms));
    for l in &lines {
        outln!("{}", l);
    }
    // A spec without a `module` declaration is named after its file.
    let name = if prog.module.is_empty() {
        o.input
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        prog.module.clone()
    };
    if held + broken + prog.unchecked.len() > 0 {
        outln!(
            "{}: invariants {} held, {} broken, {} not checked",
            name,
            held,
            broken,
            prog.unchecked.len()
        );
    }
    // Tests only; this line is last, and `corpus` reads its total.
    outln!(
        "{}: {} passed, {} failed, {} total",
        name,
        passed,
        failed,
        passed + failed
    );
    if o.time {
        errln!("{}", ph.report());
        errln!("t27b code: {} words ({} bytes) in the JIT image", jit.code_words, jit.code_words * 4);
    }
    if mismatches > 0 {
        errln!("t27b: {} JIT/interpreter mismatches", mismatches);
        return ExitCode::from(EXIT_MISMATCH);
    }
    if failed + broken > 0 {
        ExitCode::from(EXIT_FAIL)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_build(prog: &Program, ph: &mut Phases, o: &Opts) -> ExitCode {
    let Some(out) = o.out.clone() else {
        return usage("build needs -o <out.o>");
    };
    let code = ph.time("codegen", || codegen::compile(prog, TrapStyle::Brk, false));
    let code = match code {
        Ok(c) => c,
        Err(e) => return codegen_error(&e),
    };
    // A function taking or returning a struct uses t27b's own convention
    // (a struct is passed by address), not the platform's, so no C caller
    // can link to it.
    let mut names: Vec<String> = prog.funcs.iter().map(|f| f.name.clone()).collect();
    for &id in &prog.internal_abi {
        errln!("t27b: note: `{}` takes or returns a struct and is not exported", names[id as usize]);
        names[id as usize].clear();
    }
    let obj = ph.time("emit", || {
        codegen::link(Vec::new(), None, &code, prog.funcs.len()).map(|l| (macho::object(&l, &names, &prog.data), l.code.len()))
    });
    let (obj, words) = match obj {
        Ok(x) => x,
        Err(e) => {
            errln!("t27b: {}", e);
            return ExitCode::from(EXIT_CODEGEN);
        }
    };
    let w = ph.time("write", || std::fs::write(&out, &obj));
    if let Err(e) = w {
        errln!("t27b: cannot write {}: {}", out, e);
        return ExitCode::from(EXIT_IO);
    }
    if o.time {
        errln!("{}", ph.report());
        errln!("t27b code: __text {} bytes, object {} bytes", words * 4, obj.len());
    }
    ExitCode::SUCCESS
}

fn cmd_asm(prog: &Program) -> ExitCode {
    let code = match codegen::compile(prog, TrapStyle::Brk, true) {
        Ok(c) => c,
        Err(e) => return codegen_error(&e),
    };
    let linked = match codegen::link(Vec::new(), None, &code, prog.funcs.len()) {
        Ok(l) => l,
        Err(e) => {
            errln!("t27b: {}", e);
            return ExitCode::from(EXIT_CODEGEN);
        }
    };
    let mut starts: Vec<(usize, usize)> = linked
        .offsets
        .iter()
        .enumerate()
        .filter_map(|(i, o)| o.map(|o| (o, i)))
        .collect();
    starts.sort();
    let mut text = String::new();
    for (k, &(start, fi)) in starts.iter().enumerate() {
        let f = &prog.funcs[fi];
        let end = starts.get(k + 1).map_or(linked.code.len(), |s| s.0);
        text.push_str(&format!(
            "{}{}:  ; line {}\n",
            if f.is_invariant { "invariant " } else if f.is_test { "test " } else { "_" },
            f.name,
            f.line
        ));
        for pc in start..end {
            let w = linked.code[pc];
            text.push_str(&format!("  {:6x}: {:08x}  {}\n", pc * 4, w, a64::disasm(w, pc * 4)));
        }
    }
    out(&text);
    ExitCode::SUCCESS
}

// ------------------------------------------------------------------ corpus

#[derive(Clone, Debug)]
enum Outcome {
    /// Tests run, invariants run.
    Pass(usize, usize),
    TestFail(String),
    Unsupported(Vec<String>),
    FrontEnd(String),
    Mismatch(String),
    Codegen(String),
    Timeout,
    Crash(String),
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().map_or(false, |e| e == "t27") {
            out.push(p);
        }
    }
}

/// Construct name from a reject line `t27b: unsupported construct X at line N (...)`.
fn construct_of(line: &str) -> Option<String> {
    let rest = line.strip_prefix("t27b: unsupported construct ")?;
    let end = rest.find(" at line ")?;
    Some(rest[..end].to_string())
}

fn run_one(exe: &Path, file: &Path, o: &Opts) -> Outcome {
    let mut cmd = Command::new(exe);
    cmd.arg("test").arg(file).arg("--quiet").arg("--check");
    if o.mode == OverflowMode::Wrap {
        cmd.arg("--overflow").arg("wrap");
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Outcome::Crash(format!("spawn: {}", e)),
    };
    // Drain the pipes on helper threads so a chatty child cannot block.
    let mut so = child.stdout.take().unwrap();
    let mut se = child.stderr.take().unwrap();
    let ho = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut so, &mut s);
        s
    });
    let he = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut se, &mut s);
        s
    });
    let deadline = Instant::now() + Duration::from_millis(o.timeout_ms);
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(_) => break None,
        }
    };
    let stdout = ho.join().unwrap_or_default();
    let stderr = he.join().unwrap_or_default();
    let Some(st) = status else { return Outcome::Timeout };
    let first_err = stderr.lines().next().unwrap_or("").to_string();
    match st.code() {
        Some(0) => {
            let total = stdout
                .lines()
                .last()
                .and_then(|l| l.rsplit(", ").next())
                .and_then(|t| t.strip_suffix(" total"))
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            let invariants = stdout
                .lines()
                .find_map(|l| l.split_once(": invariants ").map(|x| x.1))
                .and_then(|t| t.split(' ').next())
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(0);
            Outcome::Pass(total, invariants)
        }
        Some(1) => Outcome::TestFail(
            stdout
                .lines()
                .find(|l| l.starts_with("FAIL") || l.starts_with("INVARIANT FAIL"))
                .unwrap_or("")
                .to_string(),
        ),
        Some(2) => Outcome::Unsupported(stderr.lines().filter_map(construct_of).collect()),
        Some(3) => Outcome::FrontEnd(first_err),
        Some(4) => Outcome::Mismatch(stdout.lines().find(|l| l.starts_with("MISMATCH")).unwrap_or("").to_string()),
        Some(5) => Outcome::Codegen(first_err),
        Some(c) => Outcome::Crash(format!("exit {}: {}", c, first_err)),
        None => {
            use std::os::unix::process::ExitStatusExt;
            Outcome::Crash(format!("signal {:?}: {}", st.signal(), first_err))
        }
    }
}

fn cmd_corpus(dir: &Path, o: &Opts) -> ExitCode {
    let mut files = Vec::new();
    collect(dir, &mut files);
    if files.is_empty() {
        errln!("t27b: no .t27 files under {}", dir.display());
        return ExitCode::from(EXIT_IO);
    }
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            errln!("t27b: cannot locate own executable: {}", e);
            return ExitCode::from(EXIT_IO);
        }
    };
    let t0 = Instant::now();
    let files = Arc::new(files);
    let next = Arc::new(Mutex::new(0usize));
    let results: Arc<Mutex<Vec<(usize, Outcome)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    for _ in 0..o.jobs {
        let (files, next, results, exe) = (files.clone(), next.clone(), results.clone(), exe.clone());
        let opts = Opts {
            cmd: String::new(),
            input: None,
            out: None,
            mode: o.mode,
            time: false,
            quiet: true,
            check: true,
            list: false,
            timeout_ms: o.timeout_ms,
            jobs: 1,
        };
        handles.push(std::thread::spawn(move || loop {
            let i = {
                let mut n = next.lock().unwrap();
                let i = *n;
                *n += 1;
                i
            };
            if i >= files.len() {
                break;
            }
            let r = run_one(&exe, &files[i], &opts);
            results.lock().unwrap().push((i, r));
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let mut results = std::mem::take(&mut *results.lock().unwrap());
    results.sort_by_key(|r| r.0);

    let (mut pass, mut pass_tests, mut pass_inv, mut pass_zero) = (0, 0usize, 0usize, 0);
    let (mut fail, mut unsup, mut fe, mut mism, mut cg, mut tout, mut crash) = (0, 0, 0, 0, 0, 0, 0);
    let mut first: HashMap<String, usize> = HashMap::new();
    let mut all: HashMap<String, usize> = HashMap::new();
    for (i, r) in &results {
        let name = files[*i].display();
        match r {
            Outcome::Pass(n, inv) => {
                pass += 1;
                pass_tests += n;
                pass_inv += inv;
                if *n == 0 {
                    pass_zero += 1;
                }
                if o.list {
                    outln!("PASS        {} ({} tests, {} invariants)", name, n, inv);
                }
            }
            Outcome::TestFail(m) => {
                fail += 1;
                outln!("TESTFAIL    {}: {}", name, m);
            }
            Outcome::Unsupported(cs) => {
                unsup += 1;
                if let Some(c) = cs.first() {
                    *first.entry(c.clone()).or_default() += 1;
                }
                for c in cs {
                    *all.entry(c.clone()).or_default() += 1;
                }
                if o.list {
                    outln!("UNSUPPORTED {}: {}", name, cs.first().map_or("", |s| s.as_str()));
                }
            }
            Outcome::FrontEnd(m) => {
                fe += 1;
                if o.list {
                    outln!("FRONTEND    {}: {}", name, m);
                }
            }
            Outcome::Mismatch(m) => {
                mism += 1;
                outln!("MISMATCH    {}: {}", name, m);
            }
            Outcome::Codegen(m) => {
                cg += 1;
                outln!("CODEGEN     {}: {}", name, m);
            }
            Outcome::Timeout => {
                tout += 1;
                outln!("TIMEOUT     {}", name);
            }
            Outcome::Crash(m) => {
                crash += 1;
                outln!("CRASH       {}: {}", name, m);
            }
        }
    }
    let mut top: Vec<(String, usize)> = first.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    outln!("");
    outln!("t27b corpus: {} files under {} in {:.1} s", files.len(), dir.display(), t0.elapsed().as_secs_f64());
    outln!(
        "  supported, all tests pass : {} ({} tests, {} invariants; {} files have no test block)",
        pass, pass_tests, pass_inv, pass_zero
    );
    outln!("  supported, a test fails   : {} (a test or an invariant)", fail);
    outln!("  rejected (unsupported)    : {}", unsup);
    outln!("  front-end error           : {}", fe);
    outln!("  JIT/interpreter mismatch  : {}", mism);
    outln!("  codegen limit             : {}", cg);
    outln!("  {:26}: {}", format!("timeout ({} ms)", o.timeout_ms), tout);
    outln!("  crash                     : {}", crash);
    outln!("top rejecting constructs (first rejection per file; all occurrences):");
    for (c, n) in top.iter().take(15) {
        outln!("  {:5} {:5}  {}", n, all.get(c).copied().unwrap_or(0), c);
    }
    if crash > 0 || mism > 0 {
        ExitCode::from(EXIT_MISMATCH)
    } else {
        ExitCode::SUCCESS
    }
}
