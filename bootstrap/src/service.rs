// bootstrap/src/service.rs -- the path from a specification to working silicon,
// as a command instead of a document.
//
// WHY THIS MODULE EXISTS.  docs/theory/PATH_TO_HARDWARE_RU.md measures where the
// time between an idea and a configured FPGA actually goes: writing the code is
// 0.5% of it, and on 2026-08-14 the path broke five times without a single
// defect in the code.  Four of those five were INVISIBLE at the moment of
// breakage -- a deleted binary presented as a stage that finished in 0.0 s, and
// a 9.7 MB bitstream was still produced from zero-length frames because
// bitstream size is fixed by the die, not by the content.
//
// The fifth break is the reason this is Rust and not a shell script or a
// paragraph: docs/fpga/LOCAL-BITSTREAM-FLOW.md recorded the correct diagnosis
// and the correct fix the day before, and it was applied backwards.  Knowledge
// that has to be remembered will eventually not be.
//
// Three rules are therefore enforced here rather than documented:
//
//   1. A stage is judged by its EXIT CODE and its ARTEFACT, never by elapsed
//      time.  A stage that "finished" in 0.0 s did not finish; it did not start.
//   2. Board addresses are read at run time.  All three Digilent cables share
//      serial 210512180081, so bus position is the only handle and it changes on
//      every replug.  A hardcoded --busdev-num flashes the wrong board.
//   3. A load is only believed when it TRANSITIONS.  `Done 0x1` reads the same
//      before and after any load, so `flash` drives Done to 0 with a wrong-part
//      bitstream first and requires the 0 -> 1 edge.
//
// Refs #1959

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// One stage of the path, with everything needed to judge it honestly.
struct Stage {
    name: &'static str,
    /// Seconds the stage took. Reported, never used as a verdict.
    secs: f64,
    /// Process exit status. `None` means the tool could not be spawned at all,
    /// which is the failure that most often masquerades as instant success.
    code: Option<i32>,
    /// Bytes of the artefact the stage was supposed to produce, if it names one.
    artefact: Option<(PathBuf, u64)>,
    /// A short measured fact worth printing (cell counts, test totals).
    note: String,
}

impl Stage {
    fn ok(&self) -> bool {
        if self.code != Some(0) {
            return false;
        }
        // An artefact that exists but is empty is the zero-length-frames
        // failure: every tool returned 0 and nothing was computed.
        match &self.artefact {
            Some((_, 0)) => false,
            _ => true,
        }
    }
}

fn run(cmd: &mut Command) -> (Option<i32>, String, String) {
    match cmd.output() {
        Ok(out) => (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ),
        // Spawn failure -- the tool is absent. This is the case that reads as
        // "0.0 s" to anything that times stages, so it is reported as a
        // distinct, loud absence rather than folded into a non-zero code.
        Err(e) => (None, String::new(), format!("could not spawn: {e}")),
    }
}

fn file_len(p: &Path) -> Option<(PathBuf, u64)> {
    std::fs::metadata(p).ok().map(|m| (p.to_path_buf(), m.len()))
}

/// Count LUTs and DSP48E1 INSTANCES from the final `Printing statistics` block.
///
/// Counting occurrences of the string "DSP48E1" anywhere in a yosys log is
/// WRONG and this function exists because the first version of it did exactly
/// that. The name appears in cell-library metadata that yosys reads regardless
/// of whether any instance is created, so a substring count reports DSPs in a
/// design that has none -- the same misreading that once inflated a repository
/// arXiv draft to "64 DSP48E1" for a netlist whose real DSP count was zero.
///
/// The statistics block is authoritative: it lists only instantiated cells, one
/// per line, as `   <count>   <TYPE>`. Absence from the block means zero.
fn cell_census(log: &str) -> String {
    let Some(start) = log.rfind("Printing statistics") else {
        return "no statistics block".into();
    };
    let block = &log[start..];
    let mut luts = 0u64;
    let mut dsp = 0u64;
    let mut carry = 0u64;
    for line in block.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() != 2 {
            continue;
        }
        let Ok(n) = f[0].parse::<u64>() else { continue };
        let ty = f[1];
        if ty.starts_with("LUT") && ty.len() == 4 && ty.as_bytes()[3].is_ascii_digit() {
            luts += n;
        } else if ty == "DSP48E1" {
            dsp += n;
        } else if ty == "CARRY4" {
            carry += n;
        }
    }
    format!("{luts} LUT, {carry} CARRY4, {dsp} DSP48E1")
}

fn print_table(stages: &[Stage]) -> bool {
    println!();
    println!("  {:<26} {:>8}  {:>6}  {:>12}  {}", "stage", "time", "rc", "artefact", "note");
    println!("  {}", "-".repeat(78));
    let mut all_ok = true;
    for s in stages {
        let rc = match s.code {
            Some(c) => c.to_string(),
            None => "ABSENT".to_string(),
        };
        let art = match &s.artefact {
            Some((_, n)) => format!("{n} B"),
            None => "-".to_string(),
        };
        let mark = if s.ok() { "OK  " } else { "FAIL" };
        if !s.ok() {
            all_ok = false;
        }
        println!(
            "  {mark} {:<21} {:>7.2}s  {:>6}  {:>12}  {}",
            s.name, s.secs, rc, art, s.note
        );
    }
    println!();
    all_ok
}

/// `t27c boards` -- report the Digilent cables attached RIGHT NOW.
///
/// Never cache this and never hardcode what it prints. The three cables in this
/// project share serial 210512180081, so `--ftdi-serial` cannot select one and
/// the bus position is the only handle -- and it changes on replug. Addresses
/// that were 0:4, 0:7, 0:10 became 1:4, 1:6, 1:8 across a single reconnect.
pub fn run_boards() -> anyhow::Result<()> {
    let (code, out, err) = run(Command::new("openFPGALoader").arg("--scan-usb"));
    if code.is_none() {
        println!("  openFPGALoader ABSENT -- cannot enumerate boards");
        println!("  {}", err.trim());
        std::process::exit(1);
    }
    let mut found = Vec::new();
    for line in out.lines() {
        if line.contains("0x0403:0x6014") {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() >= 2 {
                if let (Ok(b), Ok(d)) = (f[0].parse::<u32>(), f[1].parse::<u32>()) {
                    found.push(format!("{b}:{d}"));
                }
            }
        }
    }
    if found.is_empty() {
        println!("  no Digilent cables (0x0403:0x6014) on the bus");
        std::process::exit(1);
    }
    println!("  {} board(s):", found.len());
    for b in &found {
        // Read the IDCODE per board rather than trusting any document. On
        // 2026-08-14 CLAUDE.md named the part XC7A100T while all three boards
        // reported the 200T idcode.
        let (_, dout, _) = run(Command::new("openFPGALoader")
            .args(["-c", "digilent_hs2", "--busdev-num", b, "--detect"]));
        let idcode = dout
            .lines()
            .find(|l| l.contains("idcode"))
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| "idcode unreadable".into());
        let family = dout
            .lines()
            .find(|l| l.contains("family"))
            .map(|l| l.trim().to_string())
            .unwrap_or_default();
        println!("    --busdev-num {b}   {idcode}   {family}");
    }
    Ok(())
}

/// `t27c preflight` -- refuse to start place-and-route on a toolchain that
/// cannot produce a valid bitstream.
///
/// Every check states what it proves. The constids check is the one that
/// matters most: constids are ORDINAL, so a file with the same names in a
/// different order produces a database-wide off-by-N and an assertion whose
/// advice ("regenerate the chip database") costs 1.3 GB and hours, when the
/// fix is to copy in the file the database was built with.
pub fn run_preflight(repo_root: &Path, nextpnr_src: Option<String>) -> anyhow::Result<()> {
    let src = nextpnr_src.unwrap_or_else(|| {
        "/Users/playom/t27/build/fpga/openxc7/nextpnr-openxc7".to_string()
    });
    let src = PathBuf::from(src);
    let chipdb = repo_root.join("build/fpga/openxc7/xc7a200tfbg676-1.bin");
    let refc = repo_root.join("build/fpga/openxc7/constids.inc");

    let mut fails = 0usize;
    let mut ok = |cond: bool, msg: String| {
        if cond {
            println!("  OK   {msg}");
        } else {
            println!("  FAIL {msg}");
            fails += 1;
        }
    };

    match std::fs::metadata(&chipdb) {
        Ok(m) => ok(true, format!("chipdb present ({} MB)", m.len() / 1_048_576)),
        Err(_) => ok(false, format!("chipdb missing: {}", chipdb.display())),
    }

    // constids agreement, by content hash of the two files.
    let a = std::fs::read(&refc).ok();
    let b = std::fs::read(src.join("xilinx/constids.inc")).ok();
    match (a, b) {
        (Some(a), Some(b)) if a == b => {
            let n = a.iter().filter(|&&c| c == b'\n').count();
            ok(true, format!("constids match the database ({n} lines)"));
        }
        (Some(_), Some(_)) => ok(
            false,
            format!(
                "constids DIFFER -- P&R will abort. Fix: cp {} {}/xilinx/constids.inc && cmake --build {}/build -j8",
                refc.display(), src.display(), src.display()
            ),
        ),
        _ => ok(false, "constids file missing on one side".to_string()),
    }

    // The source must be the openXC7 fork. Matching constids is NOT enough:
    // build/fpga/openxc7/nextpnr-xilinx is a vendored copy inside t27 whose git
    // origin is the t27 repo itself. It accepts the reference constids and then
    // fails with `device does not have a pin named ''` on a known-good design,
    // which reads as a bad XDC and sends you to debug a correct file.
    let (_, origin, _) = run(Command::new("git")
        .args(["-C", &src.to_string_lossy(), "remote", "get-url", "origin"]));
    if origin.contains("openXC7") {
        ok(true, "source is the openXC7 fork".to_string());
    } else if origin.trim().is_empty() {
        ok(false, format!("cannot read git origin of {}", src.display()));
    } else {
        ok(false, format!(
            "source is NOT the openXC7 fork (origin: {}) -- clone: git clone --depth 1 -b stable-backports https://github.com/openXC7/nextpnr-xilinx.git",
            origin.trim()
        ));
    }

    let pnr = src.join("build/nextpnr-xilinx");
    let (c, _, _) = run(Command::new(&pnr).arg("--version"));
    ok(c == Some(0), format!("nextpnr-xilinx runs ({})", pnr.display()));

    for tool in ["yosys", "xc7frames2bit", "openFPGALoader", "iverilog", "zig"] {
        let (c, _, _) = run(Command::new(tool).arg("--version"));
        ok(c.is_some(), format!("{tool} on PATH"));
    }

    println!();
    if fails == 0 {
        println!("PASS -- toolchain can produce and load a bitstream");
        Ok(())
    } else {
        println!("FAIL -- {fails} check(s) failed; do not trust a bitstream built now");
        std::process::exit(1);
    }
}

/// `t27c prove <spec>` -- discharge the equivalence miter between the generated
/// multiplier-free RTL and a hand-written golden model that uses a real `*`.
///
/// The golden is written from the SPEC HEADER, never from the generated Verilog:
/// reading the output to write the reference would prove only that the compiler
/// agrees with itself.
///
/// `--mutate` is not a convenience. A proof that cannot fail is not evidence, so
/// this flag perturbs the golden and REQUIRES the proof to fail, exactly as the
/// on-silicon harness is required to latch on an injected fault.
pub fn run_prove(repo_root: &Path, spec: &str, mutate: bool) -> anyhow::Result<()> {
    let stem = Path::new(spec)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let formal = repo_root.join("fpga/formal");

    // The proof script names its own DUT and golden; find the script by spec.
    let script = ["prove_mvp_classifier.ys", "prove_ternary_mac.ys"]
        .iter()
        .map(|f| formal.join(f))
        .find(|p| {
            std::fs::read_to_string(p)
                .map(|s| s.contains(&stem) || stem.contains("mvp") && p.to_string_lossy().contains("mvp"))
                .unwrap_or(false)
        });

    let Some(script) = script else {
        println!("  no proof script in fpga/formal covers {spec}");
        println!("  write a golden model from the SPEC HEADER and a miter script first");
        std::process::exit(1);
    };

    // Regenerate the DUT so the proof can never drift from the specification it
    // claims to be about.
    let dut = formal.join("mvp_classifier_dut.v");
    if script.to_string_lossy().contains("mvp") {
        let (c, out, _) = run(Command::new(std::env::current_exe()?)
            .args(["gen-verilog", spec]));
        if c != Some(0) {
            println!("  FAIL could not regenerate the DUT from {spec}");
            std::process::exit(1);
        }
        std::fs::write(&dut, out)?;
    }

    // Under --mutate the golden is actually PERTURBED before the run. The first
    // version of this flag asked whether the unmutated proof failed, which it
    // never does -- a check that cannot fire, in the command whose entire
    // purpose is to prove that checks can fire.
    let golden = formal.join("mvp_classifier_golden.v");
    let saved = if mutate {
        let text = std::fs::read_to_string(&golden)?;
        // Flip the low bit of one weight template. One trit changes, so a
        // correct DUT must now disagree with the golden on at least one input.
        let perturbed = text.replacen(
            "localparam [15:0] W_B = 16'b0000100101011000;",
            "localparam [15:0] W_B = 16'b0000100101011001;",
            1,
        );
        if perturbed == text {
            println!("  could not perturb the golden -- the constant it edits has moved.");
            println!("  Fix this before trusting any passing run from this command.");
            std::process::exit(1);
        }
        std::fs::write(&golden, &perturbed)?;
        Some(text)
    } else {
        None
    };

    let t0 = Instant::now();
    let (code, out, err) = run(Command::new("yosys")
        .current_dir(&formal)
        .arg(script.file_name().unwrap()));
    let secs = t0.elapsed().as_secs_f64();
    let log = format!("{out}{err}");

    // Restore before any early exit below, so a failed mutation run cannot
    // leave a perturbed golden behind for the next caller to prove against.
    if let Some(text) = saved {
        std::fs::write(&golden, text)?;
    }

    let solved = log
        .lines()
        .find(|l| l.contains("Solving problem with"))
        .unwrap_or("")
        .trim()
        .to_string();
    // Yosys reports the verdict differently depending on the proof mode, and
    // reading only one form silently inverts the result. Measured strings:
    //
    //   bounded (-seq N)     PASS  "SAT proof finished - no model found: SUCCESS!"
    //                        FAIL  "SAT proof finished - model found: FAIL!"
    //   induction (-tempinduct)
    //                        PASS  "Induction step proven: SUCCESS!"
    //                        FAIL  "SAT temporal induction proof finished -
    //                               model found for base case: FAIL!"
    //
    // Switching the scripts from -seq to -tempinduct broke BOTH branches of the
    // old check at once: a passing proof was reported NOT PROVED, and a failing
    // one was reported as a mutation that did not fail. The `--mutate` path is
    // what surfaced it, which is the entire argument for having that flag.
    //
    // The exit code is the primary signal (`-verify` makes yosys exit non-zero
    // on a failed proof) and the strings only confirm it. When the two disagree,
    // or neither string appears, the answer is NO VERDICT -- never a guess.
    let says_proved = log.contains("Induction step proven") || log.contains("no model found");
    let says_refuted = log.contains("proof did fail")
        || log.contains("model found for base case")
        || log.contains("model found: FAIL");
    let proved = code == Some(0) && says_proved && !says_refuted;
    let refuted = code != Some(0) && says_refuted;
    if !proved && !refuted {
        println!("  NO VERDICT -- rc {code:?}, and yosys printed neither a proof nor a");
        println!("  refutation in a form this tool recognises. The miter probably did not");
        println!("  build. Read the log before believing anything about this design.");
        std::process::exit(1);
    }

    println!("  script   {}", script.display());
    println!("  {solved}");
    println!("  time     {secs:.2}s   rc {:?}", code);

    if mutate {
        // Required to FAIL. A miter that succeeds on a perturbed golden is not
        // connected to anything.
        if refuted {
            println!("  MUTATION OK -- the proof failed on a perturbed golden, as it must");
            return Ok(());
        }
        println!("  MUTATION FAILED -- the proof still passed on a perturbed golden.");
        println!("  The miter is not testing what it claims. Do not trust the passing run.");
        std::process::exit(1);
    }

    if proved {
        println!("  PROVED -- multiplier-free RTL == golden model with a real `*`, for ALL inputs");
        Ok(())
    } else {
        println!("  NOT PROVED -- a counterexample exists, or the miter did not build");
        println!("  This is a defect in the compiler or the spec, not a nuisance. Report it.");
        std::process::exit(1);
    }
}

/// `t27c path <spec>` -- the whole road from a specification to a bitstream,
/// each stage judged by its exit code and its artefact.
pub fn run_path(_repo_root: &Path, spec: &str, to_bitstream: bool) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join("t27-path");
    std::fs::create_dir_all(&tmp)?;
    let stem = Path::new(spec)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "design".into());
    let me = std::env::current_exe()?;
    let mut stages: Vec<Stage> = Vec::new();

    // ---- spec -> Zig ----
    let zig_path = tmp.join(format!("{stem}.zig"));
    let t = Instant::now();
    let (c, out, _) = run(Command::new(&me).args(["gen", spec]));
    if c == Some(0) {
        std::fs::write(&zig_path, out)?;
    }
    stages.push(Stage {
        name: "spec -> Zig",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: file_len(&zig_path),
        note: String::new(),
    });

    // ---- Zig tests ----
    let t = Instant::now();
    let (c, out, err) = run(Command::new("zig").args(["test", &zig_path.to_string_lossy()]));
    let log = format!("{out}{err}");
    let note = log
        .lines()
        .find(|l| l.contains("tests passed") || l.contains("passed"))
        .unwrap_or("")
        .trim()
        .to_string();
    stages.push(Stage {
        name: "Zig tests",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: None,
        note,
    });

    // ---- spec -> Verilog ----
    let v_path = tmp.join(format!("{stem}.v"));
    let t = Instant::now();
    // W692: `gen-verilog-for-simulation`, not `gen-verilog`. The next stage runs
    // iverilog + vvp and counts PASSED -- and after W692 the synthesis command no
    // longer emits a testbench, because emitting one into "synthesizable" output
    // was the defect (T167). Asking the synthesis output to report test results
    // would report zero, and the rule three lines below treats zero checks as
    // failure: a harness that reports nothing could not have failed.
    //
    // The other four call sites in this file KEEP `gen-verilog` deliberately:
    // `prove` miters the synthesizable RTL, `corpus` measures it, `depth`
    // diagnoses it, `silicon` places it.
    let (c, out, _) = run(Command::new(&me).args(["gen-verilog-for-simulation", spec]));
    if c == Some(0) {
        std::fs::write(&v_path, out)?;
    }
    stages.push(Stage {
        name: "spec -> Verilog",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: file_len(&v_path),
        note: String::new(),
    });

    // ---- iverilog + vvp ----
    let vvp_path = tmp.join(format!("{stem}.vvp"));
    let t = Instant::now();
    let (c, _, _) = run(Command::new("iverilog").args([
        "-g2012", "-o", &vvp_path.to_string_lossy(), &v_path.to_string_lossy(),
    ]));
    let mut note = String::new();
    let mut code = c;
    if c == Some(0) {
        let (c2, out2, _) = run(Command::new("vvp").arg(&vvp_path));
        let passed = out2.matches("PASSED").count();
        let failed = out2.matches("FAILED").count();
        note = format!("{passed} PASSED, {failed} FAILED");
        // A harness that reports zero checks is the 265-baseline failure: it
        // could not fail, so its silence proves nothing.
        code = if failed > 0 || passed == 0 { Some(1) } else { c2 };
    }
    stages.push(Stage {
        name: "iverilog + vvp",
        secs: t.elapsed().as_secs_f64(),
        code,
        artefact: file_len(&vvp_path),
        note,
    });

    if to_bitstream {
        // ---- yosys ----
        let json_path = tmp.join(format!("{stem}.json"));
        let t = Instant::now();
        let script = format!(
            "read_verilog -sv -DSIMULATION {}; synth_xilinx -family xc7 -flatten; write_json {}",
            v_path.display(), json_path.display()
        );
        let (c, out, err) = run(Command::new("yosys").args(["-p", &script]));
        let log = format!("{out}{err}");
        stages.push(Stage {
            name: "yosys",
            secs: t.elapsed().as_secs_f64(),
            code: c,
            artefact: file_len(&json_path),
            note: cell_census(&log),
        });
        println!("  (P&R and bitstream need a top wrapper and an XDC; see");
        println!("   fpga/verilog/mvp_ternary_classifier_top.v for the pattern)");
    }

    let all_ok = print_table(&stages);
    let total: f64 = stages.iter().map(|s| s.secs).sum();
    let gen: f64 = stages.iter().filter(|s| s.name.starts_with("spec ->")).map(|s| s.secs).sum();
    println!("  total {total:.2}s, of which code generation {gen:.2}s ({:.1}%)",
             if total > 0.0 { 100.0 * gen / total } else { 0.0 });
    println!();
    if all_ok {
        println!("PASS -- every stage returned 0 and produced a non-empty artefact");
        Ok(())
    } else {
        println!("FAIL -- a stage did not run or produced nothing. Time is not a verdict.");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// `t27c corpus` -- the ONLY corpus metric that does not lie.
//
// T119 (W659) measured why this command exists. A parser error count moves by
// three orders of magnitude from a single character -- two broken identifiers
// in arch.t27 were worth 1,865 reported errors -- and it RISES when a real
// defect is fixed, because the tool then parses far enough to find what the
// earlier bail-out masked. Across the 13 specs repaired in W659 the corpus
// error total fell 13,066 -> 3,765 while the number of specs that actually
// compile moved 0 -> 0.
//
// So this reports BINARY outcomes only: for each spec and each backend, does it
// generate, and does the generated artefact compile. Counts of diagnostics are
// deliberately absent from the headline; they are a measure of how early a tool
// gave up, not of how much is wrong.
//
// Every step carries its own timeout. A step that cannot be spawned is reported
// as ABSENT, never folded into "failed" -- the failure that reads as instant
// success has cost this project four separate false results.
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct SpecOutcome {
    zig_gen: bool,
    zig_build: bool,
    v_gen: bool,
    v_build: bool,
    /// W707: does `yosys synth_xilinx` accept it?
    ///
    /// Nothing measured this before. `corpus` compiles with `iverilog`, which
    /// ACCEPTS constructs yosys rejects -- a function called as a task warns and
    /// passes (T198b) -- so 327 has never been a statement about hardware.
    ///
    /// Off by default and for a reason: synthesis time is QUADRATIC in design
    /// size (T199a, exponent ~2), so the corpus's largest members take minutes
    /// each. `--synth` must be run ALONE: the sweeps that first tried this
    /// measured machine load rather than the specs, because they shared the
    /// machine with a five-agent fan-out (T199b).
    v_synth: bool,
    /// W693: does the generated module have a port that can carry a VALUE?
    ///
    /// T180 refuted the story W692 told about its own headline. `corpus` calls
    /// `gen-verilog`, so a change to `gen-verilog` moved this command's reading
    /// 156 -> 326 without changing a single design section -- 444 specs, 0
    /// additions, 0 modifications, byte-identical. All 170 newly-accepted specs
    /// carry the banner the compiler writes ITSELF:
    ///
    ///     // NO DATA PORTS -- this module cannot move a value across its boundary.
    ///
    /// A metric computed by the system under test cannot detect a change to the
    /// system under test. This column is the one the instrument does not
    /// control: it moved 57 -> 57 across the change that moved the headline by
    /// +170, and it is the number to quote.
    v_data_port: bool,
    timed_out: bool,
}

fn run_timed(cmd: &mut Command, secs: u64) -> Option<(Option<i32>, String)> {
    // std::process has no timeout, so spawn and poll. A wait_with_output() would
    // block forever on the hanging testbenches this corpus is known to contain
    // (four orphaned vvp processes at 98% CPU were found this way in W659).
    //
    // W661: OUTPUT GOES TO FILES, NOT PIPES.
    //
    // The first version of this function piped stdout and stderr and polled
    // try_wait(). A pipe holds about 64 KiB; a child whose output exceeds that
    // BLOCKS on the write, because nothing drains the pipe until after the child
    // exits -- and it never exits. try_wait() returns None forever and the
    // timeout fires.
    //
    // The corpus reported exactly 29 "hangs". Measured independently: exactly 29
    // specs generate more than 65,536 bytes of Verilog, the largest 479,261. The
    // match is not a coincidence -- there were no hangs. This function
    // manufactured them, and it undercounted `generates Verilog` by the same 29
    // (415 reported against 444 real).
    //
    // A file has no buffer limit, so the child never blocks and the timeout once
    // again means what it says.
    let dir = std::env::temp_dir().join("t27-runtimed");
    let _ = std::fs::create_dir_all(&dir);
    let pid = std::process::id();
    let op = dir.join(format!("{pid}.out"));
    let ep = dir.join(format!("{pid}.err"));
    let (Ok(of), Ok(ef)) = (std::fs::File::create(&op), std::fs::File::create(&ep)) else {
        return None;
    };

    let mut child = match cmd
        .stdout(std::process::Stdio::from(of))
        .stderr(std::process::Stdio::from(ef))
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return None,
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = std::fs::read_to_string(&op).unwrap_or_default();
                let err = std::fs::read_to_string(&ep).unwrap_or_default();
                return Some((status.code(), format!("{out}{err}")));
            }
            Ok(None) => {
                if start.elapsed().as_secs() >= secs {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Some((None, String::from("__TIMEOUT__")));
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(_) => return None,
        }
    }
}

pub fn run_corpus(
    repo_root: &Path,
    specs_dir: &str,
    limit: usize,
    json: bool,
    synth: bool,
    synth_secs: u64,
) -> anyhow::Result<()> {
    let me = std::env::current_exe()?;
    let tmp = std::env::temp_dir().join("t27-corpus");
    std::fs::create_dir_all(&tmp)?;

    let mut specs: Vec<PathBuf> = Vec::new();
    let mut stack = vec![repo_root.join(specs_dir)];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().map(|n| n == "scratch").unwrap_or(false) {
                    continue;
                }
                stack.push(p);
            } else if p.extension().map(|x| x == "t27").unwrap_or(false) {
                specs.push(p);
            }
        }
    }
    specs.sort();
    if limit > 0 && specs.len() > limit {
        specs.truncate(limit);
    }

    let mut out: Vec<(String, SpecOutcome)> = Vec::new();
    for (i, p) in specs.iter().enumerate() {
        let mut o = SpecOutcome::default();
        let sp = p.to_string_lossy().to_string();

        // ---- Zig ----
        if let Some((c, text)) = run_timed(Command::new(&me).args(["gen", &sp]), 15) {
            if text == "__TIMEOUT__" {
                o.timed_out = true;
            } else if c == Some(0) && !text.trim().is_empty() {
                o.zig_gen = true;
                let zp = tmp.join("c.zig");
                if std::fs::write(&zp, &text).is_ok() {
                    if let Some((zc, zt)) =
                        run_timed(Command::new("zig").args(["build-obj", "-fno-emit-bin",
                                                            &zp.to_string_lossy()]), 30)
                    {
                        if zt == "__TIMEOUT__" { o.timed_out = true; }
                        o.zig_build = zc == Some(0);
                    }
                }
            }
        }

        // ---- Verilog ----
        if let Some((c, text)) = run_timed(Command::new(&me).args(["gen-verilog", &sp]), 15) {
            // The compiler emits this banner itself when the module it wrote has
            // no port that can move a value. Reading its own verdict is cheaper
            // and more honest than re-deriving one.
            o.v_data_port = c == Some(0) && !text.contains("NO DATA PORTS");
            if synth && c == Some(0) && !text.is_empty() {
                // The top module is the first one the generator emits.
                let top = text
                    .lines()
                    .find_map(|l| l.strip_prefix("module ").map(|r| {
                        r.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>()
                    }))
                    .unwrap_or_default();
                if !top.is_empty() {
                    let vp2 = std::env::temp_dir().join(format!("t27-synth-{}.v", std::process::id()));
                    if std::fs::write(&vp2, &text).is_ok() {
                        let script = format!(
                            "read_verilog -sv {}; {}",
                            vp2.display(),
                            synth_xilinx_noshare(&top)
                        );
                        if let Some((yc, _)) =
                            run_timed(Command::new("yosys").args(["-p", &script]), synth_secs)
                        {
                            o.v_synth = yc == Some(0);
                        }
                    }
                }
            }
            if text == "__TIMEOUT__" {
                o.timed_out = true;
            } else if c == Some(0) && !text.trim().is_empty() {
                o.v_gen = true;
                let vp = tmp.join("c.v");
                if std::fs::write(&vp, &text).is_ok() {
                    if let Some((vc, vt)) = run_timed(
                        Command::new("iverilog").args(["-g2012", "-o", "/dev/null",
                                                       &vp.to_string_lossy()]), 30)
                    {
                        if vt == "__TIMEOUT__" { o.timed_out = true; }
                        o.v_build = vc == Some(0);
                    }
                }
            }
        }

        if !json && (i % 50 == 0 || i + 1 == specs.len()) {
            eprintln!("  ... {}/{}", i + 1, specs.len());
        }
        let rel = p.strip_prefix(repo_root).unwrap_or(p).to_string_lossy().to_string();
        out.push((rel, o));
    }

    let n = out.len();
    let c = |f: fn(&SpecOutcome) -> bool| out.iter().filter(|(_, o)| f(o)).count();
    let zg = c(|o| o.zig_gen);
    let zb = c(|o| o.zig_build);
    let vg = c(|o| o.v_gen);
    let vb = c(|o| o.v_build);
    // T180: the column the instrument does not control.
    let vdp = out.iter().filter(|(_, o)| o.v_build && o.v_data_port).count();
    let vsy = out.iter().filter(|(_, o)| o.v_synth).count();
    let both = out.iter().filter(|(_, o)| o.zig_build && o.v_build).count();
    let to = c(|o| o.timed_out);

    if json {
        println!("{{\"specs\":{n},\"zig_gen\":{zg},\"zig_build\":{zb},\"verilog_gen\":{vg},\"verilog_build\":{vb},\"verilog_build_with_data_port\":{vdp},\"verilog_synth\":{vsy},\"both_build\":{both},\"timed_out\":{to}}}");
        return Ok(());
    }

    let pct = |x: usize| if n == 0 { 0.0 } else { 100.0 * x as f64 / n as f64 };
    println!();
    println!("  corpus: {n} specs under {specs_dir}");
    println!("  {}", "-".repeat(52));
    println!("  {:<26} {:>5}  {:>6}", "generates Zig", zg, format!("{:.1}%", pct(zg)));
    println!("  {:<26} {:>5}  {:>6}", "  ... and Zig accepts it", zb, format!("{:.1}%", pct(zb)));
    println!("  {:<26} {:>5}  {:>6}", "generates Verilog", vg, format!("{:.1}%", pct(vg)));
    println!("  {:<26} {:>5}  {:>6}", "  ... and iverilog accepts", vb, format!("{:.1}%", pct(vb)));
    println!("  {:<26} {:>5}  {:>6}", "  ... AND has a data port", vdp, format!("{:.1}%", pct(vdp)));
    if synth {
        println!("  {:<26} {:>5}  {:>6}", "  ... AND yosys SYNTHESISES", vsy, format!("{:.1}%", pct(vsy)));
    }
    println!("  {:<26} {:>5}  {:>6}", "BOTH backends accept", both, format!("{:.1}%", pct(both)));
    if to > 0 {
        println!("  {:<26} {:>5}", "timed out (hang)", to);
    }
    println!();
    println!("  The gap between 'generates' and 'accepts' is the real backlog.");
    println!("  Diagnostic counts are deliberately not reported: see T119.");
    println!();
    println!("  W693/T180: QUOTE THE DATA-PORT LINE, not the iverilog line.");
    println!("  `corpus` calls `gen-verilog`, so a change to `gen-verilog` moves");
    println!("  the iverilog reading without changing a single design section --");
    println!("  measured: 156 -> 326 across 444 specs with 0 additions and 0");
    println!("  modifications to any design, while this line moved 57 -> 57.");
    println!("  A module with no data port cannot move a value across its");
    println!("  boundary; the compiler says so itself in the Verilog it writes.");
    Ok(())
}

/// Normalise one iverilog diagnostic into a CLASS.
///
/// Strips the file:line prefix and replaces every backquoted identifier with a
/// placeholder, so `No function named `forward' found` and `No function named
/// `init' found` collapse to one class. Two diagnostics of the same class are
/// evidence of one defect; two of different classes are evidence of two.
///
/// This is a PROXY and is labelled as one wherever it is reported. Two errors
/// of one class can still need separate fixes, and two classes can share a root.
/// It is used because the alternative -- reading every spec -- does not scale to
/// 617, and because T120 showed that the metric it replaces (frequency of the
/// FIRST error) ranks causes in an order that predicts nothing.
fn error_class(line: &str) -> String {
    let after = match line.find(": ") {
        Some(i) => &line[i + 2..],
        None => line,
    };
    let mut out = String::new();
    let mut in_tick = false;
    for c in after.chars() {
        match c {
            '`' if !in_tick => {
                in_tick = true;
                out.push_str("`X`");
            }
            '\'' if in_tick => in_tick = false,
            _ if in_tick => {}
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(72).collect()
}

/// `t27c depth` -- how many DISTINCT defect classes stand between each spec and
/// a clean compile.
///
/// T120 (W660) is why this exists. Removing the single most frequent cause --
/// 435 scaffold call sites across 140 specs, 133 of all iverilog failures --
/// moved the count of compiling specs from 151 to 151, because 132 of those 140
/// specs carry four or more distinct classes. A first-error histogram ranks by
/// EARLIEST occurrence and therefore cannot rank blocking power.
///
/// The specs worth fixing are the ones ONE class deep. This finds them.
///
/// It also separates the population T121 identified: a spec whose every
/// diagnostic is `No function named ...` is not broken, it is UNWRITTEN -- 159
/// specs and 667 bodiless functions -- and no compiler fix will repair it.
pub fn run_depth(repo_root: &Path, specs_dir: &str, limit: usize) -> anyhow::Result<()> {
    let me = std::env::current_exe()?;
    let tmp = std::env::temp_dir().join("t27-depth");
    std::fs::create_dir_all(&tmp)?;

    let mut specs: Vec<PathBuf> = Vec::new();
    let mut stack = vec![repo_root.join(specs_dir)];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().map(|n| n == "scratch").unwrap_or(false) { continue; }
                stack.push(p);
            } else if p.extension().map(|x| x == "t27").unwrap_or(false) {
                specs.push(p);
            }
        }
    }
    specs.sort();
    if limit > 0 && specs.len() > limit { specs.truncate(limit); }

    let mut clean = 0usize;
    let mut no_gen = 0usize;
    // (depth, spec, the one class) for defect specs
    let mut by_depth: Vec<(usize, String, String)> = Vec::new();
    let mut unwritten = 0usize;
    let mut partial = 0usize;
    let mut class_hist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (i, p) in specs.iter().enumerate() {
        let sp = p.to_string_lossy().to_string();
        let Some((c, text)) = run_timed(Command::new(&me).args(["gen-verilog", &sp]), 15) else {
            no_gen += 1;
            continue;
        };
        if text == "__TIMEOUT__" || c != Some(0) || text.trim().is_empty() {
            no_gen += 1;
            continue;
        }
        let vp = tmp.join("d.v");
        if std::fs::write(&vp, &text).is_err() { no_gen += 1; continue; }
        let Some((vc, vt)) = run_timed(
            Command::new("iverilog").args(["-g2012", "-o", "/dev/null", &vp.to_string_lossy()]), 30)
        else { no_gen += 1; continue };
        if vt == "__TIMEOUT__" { no_gen += 1; continue; }
        if vc == Some(0) { clean += 1; continue; }

        let classes: std::collections::BTreeSet<String> = vt
            .lines()
            .filter(|l| l.contains("error"))
            .map(error_class)
            .collect();
        if classes.is_empty() { continue; }

        // W662: classify UNWRITTEN from the AST, not from the diagnostics.
        //
        // The previous rule -- "every diagnostic is `No function named ...`" --
        // reported UNWRITTEN = 0 against T121's count of 159, because after the
        // W660 scaffold fix these specs still emit other malformed constructs
        // alongside the missing bodies. A diagnostic-shaped test cannot see a
        // missing function BODY; it can only see the downstream symptom, and the
        // symptom is drowned out by whatever else the module got wrong.
        //
        // `impl_status` already owns the real signal -- an FnDecl with no
        // statements, which is exactly what the Zig backend turns into
        // `@compileError("not yet implemented")` -- so the same function decides
        // it here. Both commands then agree on what "unwritten" means.
        let src = std::fs::read_to_string(p).unwrap_or_default();
        let (empty_fns, total_fns) = crate::impl_status::spec_body_counts(&src);
        if total_fns > 0 && empty_fns == total_fns {
            unwritten += 1;
            continue;
        }
        if empty_fns > 0 {
            partial += 1;
            continue;
        }

        let rel = p.strip_prefix(repo_root).unwrap_or(p).to_string_lossy().to_string();
        let one = classes.iter().next().cloned().unwrap_or_default();
        for c in &classes { *class_hist.entry(c.clone()).or_insert(0) += 1; }
        by_depth.push((classes.len(), rel, one));

        if i % 100 == 0 { eprintln!("  ... {}/{}", i + 1, specs.len()); }
    }

    by_depth.sort();
    let depth_of = |d: usize| by_depth.iter().filter(|(n, _, _)| *n == d).count();

    println!();
    println!("  {} specs under {specs_dir}", specs.len());
    println!("  {}", "-".repeat(64));
    println!("  {:<34} {:>5}", "iverilog accepts", clean);
    println!("  {:<34} {:>5}", "does not generate Verilog", no_gen);
    println!("  {:<34} {:>5}", "UNWRITTEN (every fn body empty)", unwritten);
    println!("  {:<34} {:>5}", "PARTIAL (some fn bodies empty)", partial);
    println!("  {:<34} {:>5}   <- the real defect backlog", "DEFECT specs", by_depth.len());
    println!();
    println!("  depth distribution of the defect backlog");
    for d in 1..=5 {
        // W662: the count and the bar must come from the SAME number. The first
        // version built the bar from `depth_of(5)` while printing the `>= 5`
        // total beside it, so the deepest row showed 45 specs behind an 8-wide
        // bar. A chart whose bar disagrees with its own label is worse than no
        // chart -- it is read at a glance and the glance is wrong.
        let n = if d == 5 {
            by_depth.iter().filter(|(k, _, _)| *k >= 5).count()
        } else {
            depth_of(d)
        };
        let bar = "#".repeat(n.min(60));
        println!("    {d}{} class(es) {n:>4}  {bar}", if d == 5 { "+" } else { " " });
    }

    println!();
    println!("  DEPTH-1 SPECS -- the only ones a single fix can move:");
    let d1: Vec<_> = by_depth.iter().filter(|(n, _, _)| *n == 1).collect();
    if d1.is_empty() {
        println!("    none. No single compiler fix can raise the compiling count.");
    } else {
        let mut h: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for (_, _, c) in &d1 { *h.entry(c.as_str()).or_insert(0) += 1; }
        let mut hv: Vec<_> = h.into_iter().collect();
        hv.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        for (c, n) in hv.iter().take(10) {
            println!("    {n:>3}  {c}");
        }
        println!();
        for (_, s, _) in d1.iter().take(12) { println!("      {s}"); }
    }

    println!();
    println!("  Depth is a PROXY: distinct normalised diagnostics, not verified");
    println!("  independent fixes. See T120 for why frequency is worse.");
    Ok(())
}

// ---------------------------------------------------------------------------
// `t27c silicon` -- the whole path, spec to a verdict READ BACK OFF THE DIE.
//
// W690 walked this path by hand and it worked: the MVP's answer left the die
// through JTAG USER3 and a machine read it, A/B/A, three boards, 27 reads. But
// everything that made it work lived in a scratch shell script -- the two yosys
// `delete` passes, the port-less top, the gate on the frames file, the
// `--busdev-num` addressing, the chain/site agreement, and the control
// bitstream on both sides of the read. A result that lives in /tmp is one
// `rm -rf` from being a claim again, so this is that script as a command.
//
// WHAT EACH STAGE GUARDS, and the measurement behind it:
//
//   delete t:$print        `gen-verilog` emits the spec's `test` blocks into
//                          what its own help calls synthesizable output -- 387
//                          of 444 specs, 43,053 `$display` calls corpus-wide
//                          (T167). yosys turns each into a `$print` cell and
//                          nextpnr cannot place one.
//   delete t:$scopeinfo    yosys 0.63 debug metadata; nextpnr-xilinx has no BEL
//                          for it either.
//   no --xdc               the only XDC in this tree targets CSG324, not our
//                          FGG676. A port-less top drives no package pin, so it
//                          needs no pin map at all.
//   chain/site agreement   THE DEFECT THAT HID THE READBACK FOR SIX WAVES.
//                          nextpnr places a lone BSCANE2 at site BSCAN3; a
//                          design asking for JTAG_CHAIN(1) then emits
//                          `BSCAN.JTAG_CHAIN_1` while routing
//                          `CFG_CENTER_BSCAN3_*`. Chain 1 selects an unwired
//                          site. EVERY TOOL RETURNS 0 (T172c) -- yosys, nextpnr,
//                          fasm2frames, xc7frames2bit, openFPGALoader -- and the
//                          mismatch is visible only in the read. This stage is
//                          the only thing in existence that checks it.
//   frames gate            `xc7frames2bit` turns a ZERO-BYTE frames file into a
//                          9,730,899-byte bitstream and returns 0 -- one byte
//                          from a real build (T169). `Stage::ok` fails an empty
//                          artefact, so the pipeline stops here instead of
//                          manufacturing a bitstream.
//   A/B/A on hardware      `Done 0x1` proves nothing: the boards boot from SPI
//                          flash and assert DONE unaided. Force it low with a
//                          wrong-part bitstream first, and bracket the read with
//                          a design containing no BSCANE2 at all.
//
// The verdict word is 32 bits: [31:4] magic 0xA5A5A5A, [3] 0, [2] 1, [1] beat,
// [0] ok. The 28-bit magic is not decoration -- W675 added it because a 4-bit
// read could not be told from a JTAG artefact (T139), and on this pipeline's
// first mismatched build USER1 returned a perfect-looking `ok=1, const=01, beat`
// with 28 zero bits above it (T172b). Without the magic that would have been
// recorded as success.
// ---------------------------------------------------------------------------


/// W711: `synth_xilinx` without the `share` pass.
///
/// MEASURED, on `gft_dot8`:
///
/// ```text
/// with share      31 s wall   28.86 s CPU
/// without share   11 s wall    9.73 s CPU     2.97x
/// cell census     IDENTICAL
/// ```
///
/// `share` is SAT-based resource sharing. On this corpus it merges NOTHING: the
/// SAT verdicts are 100% "can not be shared" -- 3/3, 21/21, 65/65, 32/32 across
/// the designs measured -- while consuming 64-72% of synthesis time. It performs
/// exactly C(N,2) pairwise SAT calls over cells with activation patterns, and N
/// here counts the conditional variable-distance shifts in `gft_add`
/// (`if (d < 10) { sb = (512 + lo_m) >> d; }`), one per instance. **Nothing can
/// ever be shared there**: a combinational reduction tree evaluates every branch
/// simultaneously, so no two shifts are mutually exclusive, and yosys spends a
/// multi-million-clause SAT call proving that, C(N,2) times.
///
/// `gft_softmax4` goes from exceeding 900 s to completing in 282 s.
///
/// `-run` cannot skip it alone -- `share` sits inside the `coarse` label, between
/// `alumacc` and `opt` -- so the flow is split and the block re-issued without
/// it. If a future yosys reorders `coarse`, this must be re-derived from
/// `yosys -h synth_xilinx`, not assumed.
fn synth_xilinx_noshare(top: &str) -> String {
    format!(
        "synth_xilinx -family xc7 -top {top} -flatten -run :coarse; \
         techmap -map +/cmp2lut.v -map +/cmp2lcu.v -D LUT_WIDTH=6; \
         alumacc; opt; memory -nomap; opt_clean; \
         synth_xilinx -family xc7 -top {top} -flatten -run map_memory:"
    )
}

/// The run of decimal digits at the start of `s`, or `None`.
fn leading_number(s: &str) -> Option<u32> {
    let d: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if d.is_empty() { None } else { d.parse().ok() }
}

/// Extract `(chain, site)` from a FASM file: the enabled JTAG chain and the
/// BSCAN site whose signals are actually routed. They must be equal.
fn fasm_bscan_chain_and_site(fasm: &str) -> (Option<u32>, Vec<u32>) {
    let mut chain = None;
    let mut sites: Vec<u32> = Vec::new();
    for line in fasm.lines() {
        // W693: take ALL leading digits, not the first one. A single
        // `chars().next()` reads site 12 as site 1 and reports "agree" -- a
        // false PASS on the one guard this project says nothing else checks.
        if let Some(i) = line.find("BSCAN.JTAG_CHAIN_") {
            if let Some(n) = leading_number(&line[i + "BSCAN.JTAG_CHAIN_".len()..]) {
                chain = Some(n);
            }
        }
        if let Some(i) = line.find("CFG_CENTER_BSCAN") {
            if let Some(n) = leading_number(&line[i + "CFG_CENTER_BSCAN".len()..]) {
                if !sites.contains(&n) {
                    sites.push(n);
                }
            }
        }
    }
    sites.sort_unstable();
    (chain, sites)
}

/// One `openFPGALoader` load. Returns the DONE bit the loader reports.
fn load_bitstream(bit: &Path, busdev: &str) -> (Option<i32>, Option<u8>, String) {
    let (c, out, err) = run(Command::new("openFPGALoader").args([
        "--cable",
        "digilent_hs2",
        "--busdev-num",
        busdev,
        &bit.to_string_lossy(),
    ]));
    let log = format!("{out}{err}");
    // The loader prints `done 1` on success and a decoded status block --
    // including `Done            0x0` -- when the part rejects the bitstream.
    let done = if log.contains("Done            0x0") || log.contains("ID Error") {
        Some(0u8)
    } else if log.contains("done 1") {
        Some(1u8)
    } else {
        None
    };
    (c, done, log)
}

/// Read the verdict word through JTAG. Returns (the libftdi indices whose cable
/// answered with the magic, the first such word).
///
/// WHY THIS RETURNS A SET AND NOT A COUNT.
///
/// `--busdev-num` addresses a cable for openFPGALoader; libftdi index addresses
/// it for this transport. **They are different enumerations and nothing maps
/// between them.** The first version of this command loaded the control
/// bitstream onto ONE board and then required the magic to vanish from ALL
/// cables -- so the two boards still holding the real design failed the control,
/// and the command reported FAIL on a working pipeline.
///
/// That was not a bug in the boards; it was a bug in the experiment, and W690's
/// manual run had hidden it by loading the control onto all three. Returning the
/// SET lets the caller do the honest thing: watch which index loses the magic
/// when a known board is reprogrammed, and thereby DERIVE the mapping instead of
/// assuming one.
fn read_verdict(repo_root: &Path, chain: u32) -> (Vec<usize>, Option<u32>, String) {
    let script = repo_root.join("tools/jtag/read_verdict.py");
    // W693: the chain is DERIVED from the FASM, never assumed. A wrong chain
    // reads all-zero, which is indistinguishable from a design that is not
    // there -- so guessing it turns a hardware fault and a software fault into
    // the same output.
    let (_, out, err) = run(Command::new("python3")
        .arg(&script)
        .args(["--chain", &chain.to_string()])
        .current_dir(repo_root));
    let log = format!("{out}{err}");
    let mut idxs = Vec::new();
    let mut current: Option<usize> = None;
    let mut word = None;
    for line in log.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("index ") {
            current = rest
                .split(':')
                .next()
                .and_then(|n| n.trim().parse::<usize>().ok());
            if word.is_none() {
                word = t.split_whitespace().find_map(|w| {
                    u32::from_str_radix(w, 16).ok().filter(|v| (v >> 4) == 0xA5A5A5A)
                });
            }
        }
        if t.starts_with("MAGIC PRESENT") {
            if let Some(i) = current {
                if !idxs.contains(&i) {
                    idxs.push(i);
                }
            }
        }
    }
    idxs.sort_unstable();
    (idxs, word, log)
}

pub fn run_silicon(
    repo_root: &Path,
    spec: &str,
    tops: Vec<String>,
    busdev: String,
    wrong_part: Option<String>,
    no_bscan_control: Option<String>,
    skip_hardware: bool,
) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join("t27-silicon");
    std::fs::create_dir_all(&tmp)?;
    let stem = Path::new(spec)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "design".into());
    let me = std::env::current_exe()?;
    let mut stages: Vec<Stage> = Vec::new();

    let db = repo_root.join("build/fpga/openxc7/prjxray-db/artix7");
    let chipdb = repo_root.join("build/fpga/openxc7/xc7a200tfbg676-1.bin");
    // These two live outside the worktree because they are multi-gigabyte
    // checkouts shared across worktrees; `t27c preflight` is what verifies them.
    let xr = PathBuf::from("/Users/playom/t27/build/fpga/openxc7/prjxray");
    let venv = PathBuf::from("/Users/playom/t27/build/fpga/openxc7/venv/bin/python");
    let pnr = PathBuf::from("/Users/playom/t27/build/fpga/openxc7/nextpnr-xilinx/build/nextpnr-xilinx");

    println!("=== t27c silicon: {spec} ===");
    for (what, p) in [("chipdb", &chipdb), ("prjxray-db", &db), ("nextpnr", &pnr)] {
        if !p.exists() {
            println!("  MISSING {what}: {}", p.display());
            println!("  run `t27c preflight` first -- this path cannot be faked.");
            std::process::exit(1);
        }
    }

    // ---- spec -> Verilog ----
    let v_path = tmp.join(format!("{stem}.v"));
    let t = Instant::now();
    let (c, out, _) = run(Command::new(&me).args(["gen-verilog", spec]));
    let displays = out.matches("$display").count();
    if c == Some(0) {
        std::fs::write(&v_path, &out)?;
    }
    stages.push(Stage {
        name: "spec -> Verilog",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: file_len(&v_path),
        note: format!("{displays} $display (T167: stripped below, not synthesizable)"),
    });

    // ---- yosys, then nextpnr, then the guard -- up to TWICE ----
    //
    // W693: THE CHAIN NUMBER IS DERIVED, NOT TYPED.
    //
    // W690 hardcoded `.JTAG_CHAIN(3)` because that build placed BSCANE2 at site
    // 3. W692's compiler change altered the netlist, nextpnr moved the cell to
    // site 2, and `t27c silicon` began failing its own guard -- correctly. The
    // guard caught a regression the wave that caused it did not look for.
    //
    // Retyping the constant is the wrong repair twice over: it would drift again
    // on the next netlist change, and a WRONG chain reads all-zero, which is
    // indistinguishable from a design that is not on the board. So: place once,
    // read the site out of the FASM, and if it disagrees with the parameter,
    // rebuild with `chparam` and place again. Two attempts, then fail.
    let json_path = tmp.join(format!("{stem}.json"));
    let fasm_path = tmp.join(format!("{stem}.fasm"));
    let mut sources = vec![v_path.to_string_lossy().to_string()];
    sources.extend(tops.iter().cloned());
    let top_name = tops
        .last()
        .and_then(|p| Path::new(p).file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| stem.clone());

    let mut chain_override: Option<u32> = None;
    let mut yosys_stage: Option<Stage> = None;
    let mut pnr_stage: Option<Stage> = None;
    let mut guard_stage: Option<Stage> = None;
    let mut derived_chain: Option<u32> = None;

    for attempt in 0..2u32 {
        let t = Instant::now();
        let chparam = match chain_override {
            Some(n) => format!("chparam -set JTAG_CHAIN_N {n} {top_name}; "),
            None => String::new(),
        };
        // `chparam` must run BEFORE any hierarchy pass -- an explicit
        // `hierarchy -top` here elaborates the top before its children are
        // known and dies with "Module ... is not part of the design".
        // `synth_xilinx` runs hierarchy itself, in the right order.
        let script = format!(
            "read_verilog {}; {}{}; \
             delete t:$print; delete t:$scopeinfo; write_json {}",
            sources.join(" "),
            chparam,
            synth_xilinx_noshare(&top_name),
            json_path.display()
        );
        let (c, out, err) = run(Command::new("yosys").args(["-p", &script]));
        let log = format!("{out}{err}");
        let bscan = log
            .rfind("Printing statistics")
            .map(|i| &log[i..])
            .and_then(|b| {
                b.lines().find_map(|l| {
                    let f: Vec<&str> = l.split_whitespace().collect();
                    (f.len() == 2 && f[1] == "BSCANE2").then(|| f[0].parse::<u64>().ok())?
                })
            })
            .unwrap_or(0);
        yosys_stage = Some(Stage {
            name: "yosys",
            secs: t.elapsed().as_secs_f64(),
            code: c,
            artefact: file_len(&json_path),
            note: format!(
                "{} | BSCANE2 x{bscan}{}",
                cell_census(&log),
                match chain_override { Some(n) => format!(" | chain forced to {n}"), None => String::new() }
            ),
        });
        if c != Some(0) {
            break;
        }

        let t = Instant::now();
        let (c, _, err) = run(Command::new(&pnr).args([
            "--chipdb", &chipdb.to_string_lossy(),
            "--json", &json_path.to_string_lossy(),
            "--fasm", &fasm_path.to_string_lossy(),
        ]));
        pnr_stage = Some(Stage {
            name: "nextpnr (no XDC)",
            secs: t.elapsed().as_secs_f64(),
            code: c,
            artefact: file_len(&fasm_path),
            note: err.lines().find(|l| l.starts_with("ERROR")).unwrap_or("").to_string(),
        });
        if c != Some(0) {
            break;
        }

        let fasm = std::fs::read_to_string(&fasm_path).unwrap_or_default();
        let (chain, sites) = fasm_bscan_chain_and_site(&fasm);
        let agree = match (chain, sites.as_slice()) {
            (Some(ch), [s]) => ch == *s,
            (None, []) => true,
            _ => false,
        };
        derived_chain = chain;
        guard_stage = Some(Stage {
            name: "BSCAN chain == site",
            secs: 0.0,
            code: if agree { Some(0) } else { Some(1) },
            artefact: None,
            note: match (chain, sites.as_slice()) {
                (Some(ch), [s]) if ch == *s => format!("JTAG_CHAIN({ch}) at BSCAN{s} -- agree"),
                (Some(ch), [s]) => format!("JTAG_CHAIN({ch}) enabled, BSCAN{s} wired -- rebuilding at {s}"),
                (None, []) => "no BSCANE2 in this design".into(),
                (ch, ss) => format!("ambiguous: chain={ch:?} sites={ss:?}"),
            },
        });
        if agree {
            break;
        }
        // Disagreed. Adopt the SITE nextpnr chose and place once more.
        match sites.as_slice() {
            [s] if attempt == 0 => chain_override = Some(*s),
            _ => break,
        }
    }

    if let Some(st) = yosys_stage { stages.push(st); }
    if let Some(st) = pnr_stage { stages.push(st); }
    if let Some(st) = guard_stage { stages.push(st); }


    // ---- fasm2frames ----
    let frames_path = tmp.join(format!("{stem}.frames"));
    let t = Instant::now();
    let (c, out, err) = run(
        Command::new(&venv)
            .env("PYTHONPATH", &xr)
            .arg(xr.join("utils/fasm2frames.py"))
            .args(["--db-root", &db.to_string_lossy(), "--part", "xc7a200tfbg676-1"])
            .arg(&fasm_path),
    );
    if c == Some(0) {
        std::fs::write(&frames_path, &out)?;
    }
    // The FIRST line of this traceback blames a circular import in fasm.parser;
    // the real cause is on the LAST line, usually a missing input from a stage
    // that already failed (T171).
    let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
    stages.push(Stage {
        name: "fasm2frames",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: file_len(&frames_path),
        note: if c == Some(0) { String::new() } else { last.chars().take(60).collect() },
    });

    // ---- bitstream, ONLY from non-empty frames ----
    let bit_path = tmp.join(format!("{stem}.bit"));
    let frames_ok = file_len(&frames_path).map(|(_, n)| n > 0).unwrap_or(false);
    let t = Instant::now();
    let c = if frames_ok {
        let (c, _, _) = run(Command::new("xc7frames2bit").args([
            "--part_file", &db.join("xc7a200tfbg676-1/part.yaml").to_string_lossy(),
            "--part_name", "xc7a200tfbg676-1",
            "--frm_file", &frames_path.to_string_lossy(),
            "--output_file", &bit_path.to_string_lossy(),
        ]));
        c
    } else {
        Some(1)
    };
    stages.push(Stage {
        name: "frames -> bitstream",
        secs: t.elapsed().as_secs_f64(),
        code: c,
        artefact: file_len(&bit_path),
        note: if frames_ok {
            String::new()
        } else {
            "SKIPPED: empty frames would still yield a 9.7 MB .bit (T169)".into()
        },
    });

    let build_ok = print_table(&stages);
    if !build_ok {
        println!("FAIL -- the build did not complete. Nothing was loaded.");
        std::process::exit(1);
    }

    if skip_hardware {
        println!("PASS (build only) -- rerun without --skip-hardware to load and read.");
        return Ok(());
    }

    // ---- A/B/A on real silicon ----
    println!("  --- hardware, board {busdev} ---");
    let mut hw_ok = true;

    if let Some(wp) = &wrong_part {
        let (_, done, _) = load_bitstream(Path::new(wp), &busdev);
        let ok = done == Some(0);
        hw_ok &= ok;
        println!("  {} A1 wrong part      Done {:?}  (must be 0 -- `done 1` alone proves nothing)",
                 if ok { "OK  " } else { "FAIL" }, done);
    }

    let (_, done, _) = load_bitstream(&bit_path, &busdev);
    let ok = done == Some(1);
    hw_ok &= ok;
    println!("  {} B1 our bitstream   Done {:?}  (must be 1)",
             if ok { "OK  " } else { "FAIL" }, done);

    let chain = derived_chain.unwrap_or(3);
    println!("  reading USER{chain}, derived from the FASM");
    let (before, word, _) = read_verdict(repo_root, chain);
    let ok = !before.is_empty();
    hw_ok &= ok;
    match word {
        Some(w) => println!(
            "  {} B2 read            0x{w:08x}  magic, ok={} beat={}  on index {before:?}",
            if ok { "OK  " } else { "FAIL" }, w & 1, (w >> 1) & 1
        ),
        None => println!("  FAIL B2 read            no magic on any cable"),
    }

    if let Some(nb) = &no_bscan_control {
        // The control goes onto ONE board, so require that exactly that board
        // stops answering -- not that every board does. The index that loses the
        // magic IS the libftdi handle for this --busdev-num, derived rather than
        // assumed.
        let (_, _, _) = load_bitstream(Path::new(nb), &busdev);
        let (during, _, _) = read_verdict(repo_root, chain);
        let lost: Vec<usize> = before.iter().copied().filter(|i| !during.contains(i)).collect();
        let ok = lost.len() == 1;
        hw_ok &= ok;
        println!(
            "  {} C  control         index {during:?} still answer; lost {lost:?}  (exactly one must fall silent)",
            if ok { "OK  " } else { "FAIL" }
        );
        if ok {
            println!("       -> --busdev-num {busdev} is libftdi index {}", lost[0]);
        }

        let (_, _, _) = load_bitstream(&bit_path, &busdev);
        let (after, _, _) = read_verdict(repo_root, chain);
        let returned = lost.iter().all(|i| after.contains(i));
        hw_ok &= returned;
        println!(
            "  {} A\' reload          index {after:?} answer  (the silenced one must return)",
            if returned { "OK  " } else { "FAIL" }
        );
    }

    println!();
    if hw_ok && word.map(|w| w & 1 == 1).unwrap_or(false) {
        println!("PASS -- the silicon answered, and its answer is ok=1.");
        Ok(())
    } else {
        println!("FAIL -- see the line above. A read without its control is not a result.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod w693_bscan_parser {
    use super::*;

    /// W693: `chars().next()` read site 12 as site 1 and called it agreement.
    #[test]
    fn a_two_digit_site_is_not_read_as_one_digit() {
        let fasm = "TILE.BSCAN.JTAG_CHAIN_1\nTILE.X.CFG_CENTER_BSCAN12_TDI\n";
        let (chain, sites) = fasm_bscan_chain_and_site(fasm);
        assert_eq!(chain, Some(1));
        assert_eq!(sites, vec![12], "site 12 must not collapse to 1");
    }

    #[test]
    fn agreement_and_mismatch_are_distinguished() {
        let ok = "A.BSCAN.JTAG_CHAIN_3\nB.CFG_CENTER_BSCAN3_TDI\nC.CFG_CENTER_BSCAN3_TDO\n";
        let (c, s) = fasm_bscan_chain_and_site(ok);
        assert_eq!((c, s.as_slice()), (Some(3), [3].as_slice()));

        let bad = "A.BSCAN.JTAG_CHAIN_3\nB.CFG_CENTER_BSCAN2_TDI\n";
        let (c, s) = fasm_bscan_chain_and_site(bad);
        assert_eq!((c, s.as_slice()), (Some(3), [2].as_slice()));
    }

    #[test]
    fn a_design_with_no_bscan_is_not_a_mismatch() {
        let (c, s) = fasm_bscan_chain_and_site("SOME.OTHER.FEATURE\n");
        assert_eq!(c, None);
        assert!(s.is_empty());
    }
}
