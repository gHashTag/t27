//! Repository-wide test orchestration (replaces legacy `tests/*.sh` runners).
//! Invoked as `t27c suite` from the repository root (or `tri test`).

use anyhow::Context;
use chrono::Local;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Options for the comprehensive repository suite.
#[derive(Clone, Copy, Debug, Default)]
pub struct SuiteOptions {
    /// Run Icarus Verilog simulation on lowerable specs.
    pub icarus_simulate: bool,
    /// Restrict Icarus simulation to specs the classifier marks lowerable.
    pub icarus_lowerable: bool,
    /// Run the Python reference-model cocotb cross-check on lowerable specs.
    pub cocotb: bool,
    /// Skip long-running phases where possible.
    pub fast: bool,
}

fn t27c_exe() -> anyhow::Result<PathBuf> {
    std::env::current_exe().context("current_exe failed (expected t27c binary)")
}

fn rel_arg(repo: &Path, file: &Path) -> anyhow::Result<String> {
    let rel = file.strip_prefix(repo).with_context(|| {
        format!(
            "path {} not under repo root {}",
            file.display(),
            repo.display()
        )
    })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn collect_t27(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut v: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "t27"))
        .map(|e| e.path().to_path_buf())
        .collect();
    v.sort();
    Ok(v)
}

fn run_phase(
    repo: &Path,
    label: &str,
    f: impl Fn(&Path, &str) -> anyhow::Result<()>,
    files: &[PathBuf],
) -> anyhow::Result<(usize, usize)> {
    let mut pass = 0usize;
    let mut fail = 0usize;
    for file in files {
        let rel = match rel_arg(repo, file) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("FAIL {}: {}", file.display(), e);
                fail += 1;
                continue;
            }
        };
        if let Err(e) = f(repo, &rel) {
            eprintln!("FAIL {} ({}): {}", label, rel, e);
            fail += 1;
        } else {
            pass += 1;
        }
    }
    Ok((pass, fail))
}

fn cmd_parse(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["parse", rel])
        .output()?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("parse failed: {}", err.trim());
    }
    Ok(())
}

fn cmd_typecheck(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["typecheck", rel])
        .output()?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        let out = String::from_utf8_lossy(&st.stdout);
        anyhow::bail!("typecheck failed: {} {}", out.trim(), err.trim());
    }
    Ok(())
}

fn cmd_gen(repo: &Path, rel: &str, sub: &str) -> anyhow::Result<()> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args([sub, rel])
        .output()?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("{} failed: {}", sub, err.trim());
    }
    Ok(())
}

fn cmd_seal_verify(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["seal", rel, "--verify"])
        .output()?;
    if !st.status.success() {
        let out = String::from_utf8_lossy(&st.stdout);
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("seal verify: {} {}", out.trim(), err.trim());
    }
    Ok(())
}

fn cmd_gen_stdout(repo: &Path, rel: &str) -> anyhow::Result<Vec<u8>> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["gen", rel])
        .output()?;
    if !st.status.success() {
        anyhow::bail!("gen failed");
    }
    Ok(st.stdout)
}

fn cmd_gen_verilog_stdout(repo: &Path, rel: &str) -> anyhow::Result<Vec<u8>> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["gen-verilog", rel])
        .output()?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("gen-verilog failed: {}", err.trim());
    }
    Ok(st.stdout)
}

fn yosys_available() -> bool {
    Command::new("yosys")
        .arg("-q")
        .arg("-p")
        .arg("echo on")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn icarus_tools_available() -> bool {
    let iverilog = Command::new("iverilog")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let vvp = Command::new("vvp")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    iverilog && vvp
}

fn cmd_icarus_simulate(repo: &Path, rel: &str) -> anyhow::Result<String> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["icarus-simulate", rel])
        .output()?;
    let out = String::from_utf8_lossy(&st.stdout).to_string();
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("{}", format!("Icarus simulation failed: {} {}", err.trim(), out.trim()).trim());
    }
    Ok(out)
}

fn cmd_icarus_cocotb(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let exe = t27c_exe()?;
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["icarus-cocotb", rel])
        .output()?;
    let out = String::from_utf8_lossy(&st.stdout).to_string();
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!(
            "{}",
            format!(
                "cocotb reference-model failed: {} {}",
                err.trim(),
                out.trim()
            )
            .trim()
        );
    }
    Ok(())
}

fn icarus_regression_specs(repo: &Path) -> Vec<PathBuf> {
    // W531: include W5xx structural/struct witnesses and W3xx primitive-array
    // witnesses that are now lowered with unpacked Verilog arrays.
    collect_t27(&repo.join("specs/scratch"))
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("w5") || s.starts_with("w3"))
                .unwrap_or(false)
        })
        .collect()
}

fn icarus_baseline_path(repo: &Path, rel: &str) -> PathBuf {
    repo.join(".trinity")
        .join("icarus-baselines")
        .join(rel)
        .with_extension("json")
}

fn normalize_icarus_output(text: &str) -> Vec<String> {
    // W538/W551: the Icarus simulation baseline tracks [TEST] and [BENCH]
    // status lines. VCD capture diagnostics and scalar probe debug lines are
    // intentionally omitted so the baseline stays focused on pass/fail transitions.
    text.lines()
        .map(|l| l.trim_end().to_string())
        .filter(|l| {
            if l.is_empty() {
                return false;
            }
            if l.starts_with("VCD info:") || l.starts_with("VCD warning:") {
                return false;
            }
            if l.starts_with("[PROBE]") {
                return false;
            }
            true
        })
        .collect()
}

fn load_icarus_baseline(path: &Path) -> anyhow::Result<Vec<String>> {
    let raw = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let lines = json
        .get("lines")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(lines)
}

fn save_icarus_baseline(path: &Path, lines: &[String]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::json!({ "lines": lines });
    fs::write(path, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn is_icarus_lowerable(repo: &Path, rel: &str) -> bool {
    // W534: structural classifier is the authoritative lowerability predicate.
    // The Verilog backend is then used as a cross-check: any spec whose
    // generated Verilog contains `UNSUPPORTED_ICARUS` or is rejected by
    // `iverilog -g2012` is excluded from Icarus simulation.
    let path = repo.join(rel);
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let structural = match crate::compiler::Compiler::is_icarus_lowerable(&source) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if !structural {
        return false;
    }

    let exe = match t27c_exe() {
        Ok(e) => e,
        Err(_) => return false,
    };
    let st = Command::new(&exe)
        .current_dir(repo)
        .args(["gen-verilog", rel])
        .output();
    let Ok(st) = st else { return false };
    if !st.status.success() {
        return false;
    }
    let out = String::from_utf8_lossy(&st.stdout);
    if out.contains("UNSUPPORTED_ICARUS") {
        return false;
    }
    let tmp = std::env::temp_dir().join(format!("t27c_icarus_lowerable_{}.v", rel.replace('/', "_")));
    if fs::write(&tmp, out.as_bytes()).is_err() {
        return false;
    }
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", "/dev/null", &tmp.to_string_lossy()])
        .output();
    let Ok(compile) = compile else { return false };
    compile.status.success()
}

fn cmd_icarus_simulate_with_baseline(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let out = cmd_icarus_simulate(repo, rel)?;
    let baseline = icarus_baseline_path(repo, rel);
    let actual = normalize_icarus_output(&out);
    if baseline.exists() {
        let expected = load_icarus_baseline(&baseline)?;
        if actual != expected {
            anyhow::bail!(
                "Icarus output does not match baseline {}\nexpected: {:?}\nactual:   {:?}",
                baseline.display(),
                expected,
                actual
            );
        }
    } else {
        save_icarus_baseline(&baseline, &actual)?;
        println!("  recorded Icarus baseline: {}", baseline.display());
    }
    Ok(())
}

/// IGLA specs known to be yosys-clean through `t27c gen-verilog`.
/// All 27 IGLA specs are now clean after W378 fixed Defect 6 (`let`
/// destructuring lowering).
fn igla_clean_specs() -> Vec<String> {
    vec![
        "specs/igla/coder/arch.t27".into(),
        "specs/igla/coder/bench_proxy.t27".into(),
        "specs/igla/coder/benchmark.t27".into(),
        "specs/igla/coder/dataset.t27".into(),
        "specs/igla/coder/eval.t27".into(),
        "specs/igla/coder/pipeline.t27".into(),
        "specs/igla/coder/prm.t27".into(),
        "specs/igla/coder/tokenizer.t27".into(),
        "specs/igla/coder/training.t27".into(),
        "specs/igla/coder/weights.t27".into(),
        "specs/igla/race/adder_tree.t27".into(),
        "specs/igla/race/backend.t27".into(),
        "specs/igla/race/bram_weights.t27".into(),
        "specs/igla/race/cordic.t27".into(),
        "specs/igla/race/cordic_fixed.t27".into(),
        "specs/igla/race/cordic_top.t27".into(),
        "specs/igla/race/eda.t27".into(),
        "specs/igla/race/formal.t27".into(),
        "specs/igla/race/gemm.t27".into(),
        "specs/igla/race/opcodes.t27".into(),
        "specs/igla/race/rtl.t27".into(),
        "specs/igla/race/systolic_array.t27".into(),
        "specs/igla/race/systolic_ternary.t27".into(),
        "specs/igla/race/ternary_gemm.t27".into(),
        "specs/igla/race/ternary_inference.t27".into(),
        "specs/igla/race/ternary_mac.t27".into(),
        "specs/igla/race/yosys.t27".into(),
    ]
}

fn cmd_fpga_smoke_gate(repo: &Path) -> anyhow::Result<()> {
    let bit = repo.join("fpga").join("verilog").join("ternary_mac_demo_top_200t.bit");
    if !bit.is_file() {
        println!("  SKIP: demo bitstream not found at {}", bit.display());
        return Ok(());
    }

    let script = repo.join("scripts").join("dump_bit_config.py");
    let st = Command::new("python3")
        .arg(&script)
        .arg(&bit)
        .output()
        .with_context(|| format!("spawning {} for FPGA smoke gate", script.display()))?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("dump_bit_config.py failed: {}", err.trim());
    }

    let v_paths: Vec<PathBuf> = [
        repo.join("fpga").join("verilog").join("ternary_mac_synth.v"),
        repo.join("fpga").join("verilog").join("ternary_mac_demo_top.v"),
    ]
    .into_iter()
    .filter(|p| p.is_file())
    .collect();

    if !v_paths.is_empty() && yosys_available() {
        let reads: Vec<String> = v_paths
            .iter()
            .map(|p| format!("read_verilog -sv {}", p.display()))
            .collect();
        let st = Command::new("yosys")
            .arg("-q")
            .arg("-p")
            .arg(format!(
                "{}; synth_xilinx -top ternary_mac_demo_top -family xc7; stat",
                reads.join("; ")
            ))
            .output()
            .context("spawning yosys for FPGA smoke gate")?;
        if !st.status.success() {
            let err = String::from_utf8_lossy(&st.stderr);
            anyhow::bail!("yosys rejected demo Verilog: {}", err.trim());
        }
        println!("  yosys synthesis smoke: OK");
    } else {
        println!("  SKIP: yosys or demo Verilog unavailable");
    }

    Ok(())
}

fn cmd_gen_verilog_yosys_smoke(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let verilog = cmd_gen_verilog_stdout(repo, rel)?;
    let tmp = std::env::temp_dir().join(format!("t27c_yosys_smoke_{}.v", rel.replace('/', "_")));
    fs::write(&tmp, &verilog)
        .with_context(|| format!("writing temporary Verilog for yosys smoke: {}", tmp.display()))?;
    let st = Command::new("yosys")
        .arg("-q")
        .arg("-p")
        .arg(format!("read_verilog -sv {}", tmp.display()))
        .output()
        .context("spawning yosys for gen-verilog smoke")?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("yosys rejected generated Verilog: {}", err.trim());
    }
    let err = String::from_utf8_lossy(&st.stderr);
    if !err.trim().is_empty() {
        eprintln!("WARN yosys warnings for {}: {}", rel, err.trim());
    }
    Ok(())
}

/// Phases 1–6: same coverage as legacy `tests/run_all.sh`.
pub fn run_comprehensive(repo_root: &Path, opts: SuiteOptions) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)
        .with_context(|| format!("cannot canonicalize repo root {}", repo_root.display()))?;

    println!("=== T27 Comprehensive Test Suite ===");
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");
    println!("repo: {}", repo.display());
    println!();

    let specs_compiler: Vec<PathBuf> = {
        let mut v = collect_t27(&repo.join("specs"))?;
        v.sort();
        v.dedup();
        v
    };

    let specs_only = collect_t27(&repo.join("specs"))?;
    let specs_scratch = collect_t27(&repo.join("specs/scratch"))?;

    println!("--- Phase 1: Parse ---");
    let (p1p, p1f) = run_phase(&repo, "parse", cmd_parse, &specs_compiler)?;
    println!("Parse: {} passed, {} failed", p1p, p1f);

    println!("--- Phase 1b: Typecheck ---");
    let (p1bp, p1bf) = run_phase(&repo, "typecheck", cmd_typecheck, &specs_compiler)?;
    println!("Typecheck: {} passed, {} failed", p1bp, p1bf);

    println!("--- Phase 1c: GF16 Conformance ---");
    let mut gf16_fail = 0usize;
    let gf16_path = repo.join("specs/numeric/gf16.t27");
    if gf16_path.exists() {
        let rel = rel_arg(&repo, &gf16_path)?;
        if let Err(e) = cmd_typecheck(&repo, &rel) {
            eprintln!("GF16 CONFORMANCE FAIL: {}", e);
            gf16_fail = 1;
        } else {
            println!("GF16: conformance OK (typecheck clean)");
        }
    } else {
        println!("GF16: skipped (spec not found)");
    }

    println!("--- Phase 2: Gen Zig ---");
    let (p2p, p2f) = run_phase(
        &repo,
        "gen-zig",
        |r, rel| cmd_gen(r, rel, "gen"),
        &specs_compiler,
    )?;
    println!("Gen Zig: {} passed, {} failed", p2p, p2f);

    println!("--- Phase 2b: Gen Rust ---");
    let (p2bp, p2bf) = run_phase(
        &repo,
        "gen-rust",
        |r, rel| cmd_gen(r, rel, "gen-rust"),
        &specs_compiler,
    )?;
    println!("Gen Rust: {} passed, {} failed", p2bp, p2bf);

    println!("--- Phase 3: Gen Verilog ---");
    let (p3p, p3f) = run_phase(
        &repo,
        "gen-verilog",
        |r, rel| cmd_gen(r, rel, "gen-verilog"),
        &specs_only,
    )?;
    println!("Gen Verilog: {} passed, {} failed", p3p, p3f);

    println!("--- Phase 3b: Gen Verilog Yosys Smoke ---");
    let mut p3b_fail = 0usize;
    if yosys_available() {
        let mut smoke_targets = specs_scratch.clone();
        for rel in igla_clean_specs() {
            smoke_targets.push(repo.join(&rel));
        }
        smoke_targets.sort();
        smoke_targets.dedup();
        let (p3bp, p3bf) = run_phase(
            &repo,
            "gen-verilog-yosys-smoke",
            cmd_gen_verilog_yosys_smoke,
            &smoke_targets,
        )?;
        println!("Gen Verilog Yosys Smoke: {} passed, {} failed", p3bp, p3bf);
        p3b_fail = p3bf;
    } else {
        println!("Yosys not available; skipping gen-verilog yosys smoke gate");
    }

    println!("--- Phase 3c: FPGA Board-Less Smoke Gate ---");
    let mut p3c_fail = 0usize;
    if let Err(e) = cmd_fpga_smoke_gate(&repo) {
        eprintln!("FPGA smoke gate failed: {}", e);
        p3c_fail = 1;
    }

    println!("--- Phase 3d: Icarus Verilog Simulation Gate ---");
    let mut p3d_fail = 0usize;
    if opts.icarus_simulate || opts.icarus_lowerable {
        if icarus_tools_available() {
            // W530 first regression suite: W493–W529 lowerable scratch witnesses.
            let mut sim_targets = icarus_regression_specs(&repo);
            if opts.icarus_lowerable {
                sim_targets.retain(|f| {
                    let rel = match rel_arg(&repo, f) {
                        Ok(r) => r,
                        Err(_) => return false,
                    };
                    is_icarus_lowerable(&repo, &rel)
                });
            }
            let (p3dp, p3df) = run_phase(
                &repo,
                "icarus-simulate",
                cmd_icarus_simulate_with_baseline,
                &sim_targets,
            )?;
            println!("Icarus Simulation: {} passed, {} failed", p3dp, p3df);
            p3d_fail = p3df;
        } else {
            println!("iverilog/vvp not available; skipping Icarus simulation gate");
        }
    } else {
        println!("Icarus simulation gate disabled (use --icarus-simulate or --icarus-lowerable)");
    }

    println!("--- Phase 3e: Cocotb Reference-Model Cross-Check Gate ---");
    let mut p3e_fail = 0usize;
    if opts.cocotb {
        if icarus_tools_available() {
            let mut cocotb_targets = icarus_regression_specs(&repo);
            if opts.icarus_lowerable {
                cocotb_targets.retain(|f| {
                    let rel = match rel_arg(&repo, f) {
                        Ok(r) => r,
                        Err(_) => return false,
                    };
                    is_icarus_lowerable(&repo, &rel)
                });
            }
            let (p3ep, p3ef) = run_phase(
                &repo,
                "icarus-cocotb",
                cmd_icarus_cocotb,
                &cocotb_targets,
            )?;
            println!("Cocotb Reference Model: {} passed, {} failed", p3ep, p3ef);
            p3e_fail = p3ef;
        } else {
            println!("iverilog/vvp not available; skipping cocotb reference-model gate");
        }
    } else {
        println!("Cocotb reference-model gate disabled (use --cocotb)");
    }

    println!("--- Phase 4: Gen C ---");
    let (p4p, p4f) = run_phase(
        &repo,
        "gen-c",
        |r, rel| cmd_gen(r, rel, "gen-c"),
        &specs_only,
    )?;
    println!("Gen C: {} passed, {} failed", p4p, p4f);

    println!("--- Phase 5: Seal Verify ---");
    let (p5p, p5f) = run_phase(&repo, "seal-verify", cmd_seal_verify, &specs_only)?;
    println!("Seal Verify: {} passed, {} failed", p5p, p5f);

    println!("--- Phase 6: Fixed Point ---");
    let mut fp_diff = 0usize;
    for file in &specs_compiler {
        let rel = rel_arg(&repo, file)?;
        let a = match cmd_gen_stdout(&repo, &rel) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let b = match cmd_gen_stdout(&repo, &rel) {
            Ok(x) => x,
            Err(_) => continue,
        };
        if a != b {
            fp_diff += 1;
        }
    }
    println!("Fixed Point: {} divergences", fp_diff);

    println!();
    println!("=== SUMMARY ===");
    let total_fail = p1f + p1bf + gf16_fail + p2f + p2bf + p3f + p3b_fail + p3c_fail + p3d_fail + p3e_fail + p4f + p5f + fp_diff;
    println!("Parse failures:           {}", p1f);
    println!("Typecheck fails:          {}", p1bf);
    println!("GF16 conformance:         {}", gf16_fail);
    println!("Gen Zig failures:         {}", p2f);
    println!("Gen Rust failures:        {}", p2bf);
    println!("Gen Verilog fails:        {}", p3f);
    println!("Gen Verilog smoke fails:  {}", p3b_fail);
    println!("FPGA smoke fails:         {}", p3c_fail);
    println!("Icarus simulation fails:  {}", p3d_fail);
    println!("Cocotb reference fails:   {}", p3e_fail);
    println!("Gen C failures:           {}", p4f);
    println!("Seal mismatches:          {}", p5f);
    println!("FP divergences:           {}", fp_diff);
    println!("TOTAL FAILURES:    {}", total_fail);
    println!();
    if total_fail == 0 {
        println!("ALL TESTS PASSED");
        println!("phi^2 + 1/phi^2 = 3 | TRINITY");
        Ok(())
    } else {
        anyhow::bail!("SOME TESTS FAILED");
    }
}

/// Validate `conformance/*.json` files (structure + non-empty vectors when present).
pub fn validate_conformance(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)?;
    let dir = repo.join("conformance");
    println!("=== Conformance Validation ===");
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut entries: Vec<PathBuf> = fs::read_dir(&dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort();

    for p in entries {
        let raw = fs::read_to_string(&p)?;
        let json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("FAIL: {} invalid JSON: {}", p.display(), e);
                fail += 1;
                continue;
            }
        };
        let vec_len = json
            .get("vectors")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .or_else(|| {
                json.get("test_vectors")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
            })
            .or_else(|| {
                json.get("constants")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
            })
            .unwrap_or(0);
        if vec_len == 0 {
            let module = json
                .get("module")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            println!("WARN: {} has no vectors (module={})", p.display(), module);
        }
        pass += 1;
    }

    println!();
    println!(
        "Conformance files: {} total, {} valid, {} invalid",
        pass + fail,
        pass,
        fail
    );
    if fail == 0 {
        println!("ALL CONFORMANCE VALID");
        Ok(())
    } else {
        anyhow::bail!("CONFORMANCE FAILURES DETECTED");
    }
}

fn header_ok(first_lines: &str) -> bool {
    first_lines.contains("Auto-generated")
        || first_lines.contains("DO NOT EDIT")
        || first_lines.contains("TRINITY")
}

/// Validate generated file headers under `gen/`.
pub fn validate_gen_headers(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)?;
    println!("=== Gen Header Validation ===");

    let patterns: [(&str, &str); 4] = [
        ("gen/zig", "zig"),
        ("gen/c", "c"),
        ("gen/c", "h"),
        ("gen/verilog", "v"),
    ];

    let mut pass = 0usize;
    let mut fail = 0usize;

    for (base, ext) in patterns {
        let root = repo.join(base);
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            if p.extension().and_then(|e| e.to_str()) != Some(ext) {
                continue;
            }
            let content = fs::read_to_string(p)?;
            let head: String = content.lines().take(8).collect::<Vec<_>>().join("\n");
            if header_ok(&head) {
                pass += 1;
            } else {
                eprintln!("FAIL: {} missing required header", p.display());
                fail += 1;
            }
        }
    }

    println!(
        "Gen files: {} total, {} valid headers, {} missing",
        pass + fail,
        pass,
        fail
    );
    if fail == 0 {
        println!("ALL GEN HEADERS VALID");
        Ok(())
    } else {
        anyhow::bail!("HEADER FAILURES DETECTED");
    }
}

fn char_boundary_indices(line: &str) -> Vec<usize> {
    line.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(line.len()))
        .collect()
}

fn first_yyyy_mm_dd_in_line(line: &str) -> Option<String> {
    let idx = char_boundary_indices(line);
    for &i in &idx {
        if i + 10 > line.len() {
            continue;
        }
        let Some(slice) = line.get(i..i + 10) else {
            continue;
        };
        if !slice.is_ascii() {
            continue;
        }
        if !slice.as_bytes()[0].is_ascii_digit() {
            continue;
        }
        if chrono::NaiveDate::parse_from_str(slice, "%Y-%m-%d").is_ok() {
            return Some(slice.to_string());
        }
    }
    None
}

/// First RFC3339 timestamp on the line (UTC `…Z` or numeric offset `…+07:00`), if any.
fn optional_rfc3339_stamp(line: &str) -> Option<String> {
    let idx = char_boundary_indices(line);
    for (k, &i) in idx.iter().enumerate() {
        if i + 10 > line.len() {
            continue;
        }
        let date = match line.get(i..i + 10) {
            Some(s) if s.is_ascii() => s,
            _ => continue,
        };
        if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
            continue;
        }
        let mut longest: Option<String> = None;
        for &j in idx.iter().skip(k + 1) {
            if j < i + 19 {
                continue;
            }
            let Some(cand) = line.get(i..j) else {
                continue;
            };
            if chrono::DateTime::parse_from_rfc3339(cand).is_ok() {
                longest = Some(cand.to_string());
            }
        }
        if let Some(s) = longest {
            return Some(s);
        }
    }
    None
}

/// Gate: `docs/NOW.md` must contain `Last updated:` with today's calendar date (local timezone).
/// Used by `tri` before gen/compile and by CI (see `phi-loop-ci.yml`).
pub fn check_now_sync(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)?;
    let now_file = repo.join("docs/NOW.md");
    let today = Local::now().format("%Y-%m-%d").to_string();

    if !now_file.is_file() {
        eprintln!("tri/CI: docs/NOW.md not found at {}", now_file.display());
        anyhow::bail!("NOW.md missing");
    }

    let content = fs::read_to_string(&now_file)?;
    let line = content
        .lines()
        .find(|l| l.contains("Last updated:"))
        .unwrap_or("");
    let last = first_yyyy_mm_dd_in_line(line);

    if last.as_deref() != Some(today.as_str()) {
        eprintln!(
            r#"

╔═══════════════════════════════════════════════════════════════╗
║              ⛔  BUILD BLOCKED: SYNC REQUIRED                  ║
╠═══════════════════════════════════════════════════════════════╣
║  docs/NOW.md is STALE. All agents must be synchronized       ║
║  before any build can proceed.                               ║
╠═══════════════════════════════════════════════════════════════╣
║  STEPS TO UNBLOCK:                                            ║
║                                                               ║
║  1. Read coordination anchor:                                 ║
║     https://github.com/gHashTag/t27/issues/141               ║
║                                                               ║
║  2. Read agent sync state:                                    ║
║     cat .trinity/state/github-sync.json                      ║
║                                                               ║
║  3. Update docs/NOW.md:                                       ║
║     - Set calendar date YYYY-MM-DD (must match today locally) ║
║     - Use your local wall time (see NOW.md header template)   ║
║     - Update sprint status + what you build and why           ║
║                                                               ║
║  4. Stage and commit NOW.md with your changes:               ║
║     git add docs/NOW.md && git commit --amend                ║
╚═══════════════════════════════════════════════════════════════╝
"#
        );
        eprintln!(
            "(Expected Last updated: {}; found: {})",
            today,
            last.as_deref().unwrap_or("<none>")
        );
        anyhow::bail!("NOW.md stale");
    }

    if let Some(ts) = optional_rfc3339_stamp(line) {
        let human = chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|dt| {
                let local = dt.with_timezone(&Local);
                local
                    .format("%A, %d %B %Y · %H:%M local time (%:z)")
                    .to_string()
            })
            .unwrap_or_else(|_| ts.clone());
        println!(
            "✅ NOW.md synced — gate date {} — doc time {} [{}] — build authorized",
            today, human, ts
        );
    } else {
        println!("✅ NOW.md synced ({}) — build authorized", today);
    }
    Ok(())
}
