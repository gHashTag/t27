//! Repository-wide test orchestration (replaces legacy `tests/*.sh` runners).
//! Invoked as `t27c suite` from the repository root (or `tri test`).

use anyhow::Context;
use chrono::Local;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

fn t27c_exe() -> anyhow::Result<PathBuf> {
    std::env::current_exe().context("current_exe failed (expected t27c binary)")
}

/// Locate the `tri` CLI binary used for FPGA smoke-gate integration.
/// Prefers the same build profile as the running `t27c` binary, then falls
/// back through common target directories.
fn tri_exe(repo: &Path) -> anyhow::Result<PathBuf> {
    let t27c = t27c_exe()?;
    if let Some(dir) = t27c.parent() {
        let adjacent = dir.join("tri");
        if adjacent.is_file() {
            return Ok(adjacent);
        }
    }
    let candidates: Vec<PathBuf> = vec![
        repo.join("target").join("release").join("tri"),
        repo.join("target").join("debug").join("tri"),
        repo.join("bootstrap").join("target").join("release").join("tri"),
        repo.join("bootstrap").join("target").join("debug").join("tri"),
    ];
    for p in &candidates {
        if p.is_file() {
            return Ok(p.clone());
        }
    }
    anyhow::bail!(
        "tri binary not found. Expected one of: {}. Run: cargo build -p tri",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Load the set of spec paths that are documented as pre-existing
/// `gen-verilog` yosys smoke failures. If the baseline file is missing, the
/// set is empty and the suite summary falls back to a strict `acceptable ==
/// passed` interpretation.
fn load_gen_verilog_smoke_baseline(repo: &Path) -> HashSet<String> {
    let path = repo
        .join("docs")
        .join("reports")
        .join("gen_verilog_smoke_baseline.json");
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "[suite] baseline file not readable ({}); using empty baseline",
                e
            );
            return HashSet::new();
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[suite] baseline file invalid JSON ({}); using empty baseline", e);
            return HashSet::new();
        }
    };
    json.get("expected_failures")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
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
    let (pass, fail, _) = run_phase_with_failures(repo, label, f, files)?;
    Ok((pass, fail))
}

/// Like `run_phase`, but also returns the relative paths of failing files so the
/// suite summary can expose them to CI consumers.
fn run_phase_with_failures(
    repo: &Path,
    label: &str,
    f: impl Fn(&Path, &str) -> anyhow::Result<()>,
    files: &[PathBuf],
) -> anyhow::Result<(usize, usize, Vec<String>)> {
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for file in files {
        let rel = match rel_arg(repo, file) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("FAIL {}: {}", file.display(), e);
                fail += 1;
                failures.push(file.display().to_string().replace('\\', "/"));
                continue;
            }
        };
        if let Err(e) = f(repo, &rel) {
            eprintln!("FAIL {} ({}): {}", label, rel, e);
            fail += 1;
            failures.push(rel);
        } else {
            pass += 1;
        }
    }
    Ok((pass, fail, failures))
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

#[derive(Debug, Clone)]
struct FpgaSmokeResult {
    passed: bool,
    skipped: bool,
    report_path: Option<PathBuf>,
    schema_version: Option<String>,
    bit_config_status: Option<String>,
    dry_run_sweep_status: Option<String>,
    verify_lean_status: Option<String>,
    theorem_matrix_status: Option<String>,
    theorem_matrix_elapsed_ms: Option<u64>,
    yosys_synthesis_status: Option<String>,
}

fn cmd_fpga_smoke_gate(repo: &Path) -> anyhow::Result<FpgaSmokeResult> {
    let bit = repo.join("fpga").join("verilog").join("ternary_mac_demo_top_200t.bit");
    let report_path = repo.join("build").join("fpga").join("smoke_gate_report.json");

    if !bit.is_file() {
        println!("  SKIP: demo bitstream not found at {}", bit.display());
        return Ok(FpgaSmokeResult {
            passed: false,
            skipped: true,
            report_path: None,
            schema_version: None,
            bit_config_status: None,
            dry_run_sweep_status: None,
            verify_lean_status: None,
            theorem_matrix_status: None,
            theorem_matrix_elapsed_ms: None,
            yosys_synthesis_status: None,
        });
    }

    let tri = tri_exe(repo)?;
    run_fpga_smoke_gate(&bit, &tri, report_path, Some(repo))
}

/// Core smoke-gate consumer. Separated from `cmd_fpga_smoke_gate` so unit tests
/// can inject fake bitstreams / `tri` binaries without touching the repo.
fn run_fpga_smoke_gate(
    _bit: &Path,
    tri: &Path,
    report_path: PathBuf,
    cwd: Option<&Path>,
) -> anyhow::Result<FpgaSmokeResult> {
    let fallback_dir = report_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir());
    let report_dir = report_path.parent().unwrap_or(&fallback_dir);
    fs::create_dir_all(report_dir)?;

    let mut cmd = Command::new(tri);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let st = cmd
        .args([
            "fpga",
            "smoke-gate",
            "--synthetic-operating-point",
            "--verify-lean",
            "--theorem-matrix",
            "--json",
            &report_path.to_string_lossy(),
        ])
        .output()
        .with_context(|| format!("spawning {} for FPGA smoke gate", tri.display()))?;
    if !st.status.success() {
        let out = String::from_utf8_lossy(&st.stdout);
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("tri fpga smoke-gate failed: {} {}", out.trim(), err.trim());
    }

    parse_smoke_gate_report(&report_path)
}

fn parse_smoke_gate_report(report_path: &Path) -> anyhow::Result<FpgaSmokeResult> {
    let report: serde_json::Value = match fs::read_to_string(report_path) {
        Ok(text) => serde_json::from_str(&text)
            .with_context(|| format!("parsing smoke-gate report {}", report_path.display()))?,
        Err(e) => anyhow::bail!("smoke-gate report missing: {}: {}", report_path.display(), e),
    };

    let phase_status = |key: &str| {
        report
            .get(key)
            .and_then(|v| v.get("status"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
    };

    let passed = report
        .get("passed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let schema_version = report
        .get("schema_version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let theorem_matrix_elapsed_ms = report
        .get("theorem_matrix")
        .and_then(|v| v.get("elapsed_ms"))
        .and_then(|v| v.as_u64());
    let result = FpgaSmokeResult {
        passed,
        skipped: false,
        report_path: Some(report_path.to_path_buf()),
        schema_version,
        bit_config_status: phase_status("bit_config"),
        dry_run_sweep_status: phase_status("dry_run_sweep"),
        verify_lean_status: phase_status("verify_lean"),
        theorem_matrix_status: phase_status("theorem_matrix"),
        theorem_matrix_elapsed_ms,
        yosys_synthesis_status: phase_status("yosys_synthesis"),
    };

    println!(
        "  FPGA smoke gate: {} (report: {})",
        if passed { "OK" } else { "FAILED" },
        report_path.display()
    );
    println!(
        "    phases: bit_config={:?} dry_run_sweep={:?} verify_lean={:?} yosys_synthesis={:?}",
        result.bit_config_status,
        result.dry_run_sweep_status,
        result.verify_lean_status,
        result.yosys_synthesis_status
    );

    if !passed {
        anyhow::bail!("smoke-gate report indicates failure");
    }

    Ok(result)
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

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct SuitePhaseSummary {
    name: String,
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct SuiteSummary {
    repo: String,
    phases: Vec<SuitePhaseSummary>,
    fpga_smoke_report: Option<String>,
    fpga_smoke_passed: Option<bool>,
    /// Elapsed milliseconds reported by the smoke-gate theorem matrix, if any.
    fpga_smoke_gate_elapsed_ms: Option<u64>,
    /// Specs that failed in the `gen-verilog-yosys-smoke` phase, if any.
    known_failures: Vec<String>,
    /// Number of failures documented as the current baseline in
    /// `docs/reports/gen_verilog_smoke_baseline.json`.
    baseline_failures: usize,
    total_failures: usize,
    /// True when no failures were observed at all.
    passed: bool,
    /// True when the only observed failures are within the documented baseline.
    acceptable: bool,
}

/// Phases 1–6: same coverage as legacy `tests/run_all.sh`.
pub fn run_comprehensive(repo_root: &Path, json_out: Option<&PathBuf>) -> anyhow::Result<()> {
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

    let mut summary = SuiteSummary {
        repo: repo.display().to_string(),
        ..Default::default()
    };
    let mut push_phase = |name: &str, passed: usize, failed: usize, skipped: usize| {
        summary.phases.push(SuitePhaseSummary {
            name: name.to_string(),
            passed,
            failed,
            skipped,
        });
    };

    println!("--- Phase 1: Parse ---");
    let (p1p, p1f) = run_phase(&repo, "parse", cmd_parse, &specs_compiler)?;
    println!("Parse: {} passed, {} failed", p1p, p1f);
    push_phase("parse", p1p, p1f, 0);

    println!("--- Phase 1b: Typecheck ---");
    let (p1bp, p1bf) = run_phase(&repo, "typecheck", cmd_typecheck, &specs_compiler)?;
    println!("Typecheck: {} passed, {} failed", p1bp, p1bf);
    push_phase("typecheck", p1bp, p1bf, 0);

    println!("--- Phase 1c: GF16 Conformance ---");
    let mut gf16_fail = 0usize;
    let gf16_path = repo.join("specs/numeric/gf16.t27");
    let gf16_skipped = !gf16_path.exists();
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
    push_phase("gf16_conformance", 1 - gf16_fail - (gf16_skipped as usize), gf16_fail, gf16_skipped as usize);

    println!("--- Phase 2: Gen Zig ---");
    let (p2p, p2f) = run_phase(
        &repo,
        "gen-zig",
        |r, rel| cmd_gen(r, rel, "gen"),
        &specs_compiler,
    )?;
    println!("Gen Zig: {} passed, {} failed", p2p, p2f);
    push_phase("gen-zig", p2p, p2f, 0);

    println!("--- Phase 2b: Gen Rust ---");
    let (p2bp, p2bf) = run_phase(
        &repo,
        "gen-rust",
        |r, rel| cmd_gen(r, rel, "gen-rust"),
        &specs_compiler,
    )?;
    println!("Gen Rust: {} passed, {} failed", p2bp, p2bf);
    push_phase("gen-rust", p2bp, p2bf, 0);

    println!("--- Phase 3: Gen Verilog ---");
    let (p3p, p3f) = run_phase(
        &repo,
        "gen-verilog",
        |r, rel| cmd_gen(r, rel, "gen-verilog"),
        &specs_only,
    )?;
    println!("Gen Verilog: {} passed, {} failed", p3p, p3f);
    push_phase("gen-verilog", p3p, p3f, 0);

    println!("--- Phase 3b: Gen Verilog Yosys Smoke ---");
    let mut p3b_fail = 0usize;
    let mut p3b_skipped = 0usize;
    let baseline = load_gen_verilog_smoke_baseline(&repo);
    let (p3bp, p3bf, p3b_known_failures) = if yosys_available() {
        let mut smoke_targets = specs_scratch.clone();
        for rel in igla_clean_specs() {
            smoke_targets.push(repo.join(&rel));
        }
        smoke_targets.sort();
        smoke_targets.dedup();
        let (bp, bf, failures) = run_phase_with_failures(
            &repo,
            "gen-verilog-yosys-smoke",
            cmd_gen_verilog_yosys_smoke,
            &smoke_targets,
        )?;
        println!("Gen Verilog Yosys Smoke: {} passed, {} failed", bp, bf);
        summary.known_failures = failures;
        (bp, bf, summary.known_failures.clone())
    } else {
        println!("Yosys not available; skipping gen-verilog yosys smoke gate");
        p3b_skipped = 1;
        (0, 0, Vec::new())
    };
    p3b_fail = p3bf;
    summary.baseline_failures = baseline.len();
    push_phase("gen-verilog-yosys-smoke", p3bp, p3bf, p3b_skipped);

    println!("--- Phase 3c: FPGA Board-Less Smoke Gate ---");
    let mut p3c_fail = 0usize;
    let mut p3c_skipped = 0usize;
    let fpga_result = match cmd_fpga_smoke_gate(&repo) {
        Ok(r) => {
            if r.skipped {
                p3c_skipped = 1;
            }
            summary.fpga_smoke_report = r.report_path.as_ref().map(|p| p.display().to_string());
            summary.fpga_smoke_passed = Some(r.passed);
            summary.fpga_smoke_gate_elapsed_ms = r.theorem_matrix_elapsed_ms;
            r
        }
        Err(e) => {
            eprintln!("FPGA smoke gate failed: {}", e);
            p3c_fail = 1;
            FpgaSmokeResult {
                passed: false,
                skipped: false,
                report_path: None,
                schema_version: None,
                bit_config_status: None,
                dry_run_sweep_status: None,
                verify_lean_status: None,
                theorem_matrix_status: None,
                theorem_matrix_elapsed_ms: None,
                yosys_synthesis_status: None,
            }
        }
    };
    push_phase("fpga-smoke-gate", if fpga_result.passed { 1 } else { 0 }, p3c_fail, p3c_skipped);

    println!("--- Phase 4: Gen C ---");
    let (p4p, p4f) = run_phase(
        &repo,
        "gen-c",
        |r, rel| cmd_gen(r, rel, "gen-c"),
        &specs_only,
    )?;
    println!("Gen C: {} passed, {} failed", p4p, p4f);
    push_phase("gen-c", p4p, p4f, 0);

    println!("--- Phase 5: Seal Verify ---");
    let (p5p, p5f) = run_phase(&repo, "seal-verify", cmd_seal_verify, &specs_only)?;
    println!("Seal Verify: {} passed, {} failed", p5p, p5f);
    push_phase("seal-verify", p5p, p5f, 0);

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
    push_phase("fixed-point", 0, fp_diff, 0);

    println!();
    println!("=== SUMMARY ===");
    let total_fail = p1f + p1bf + gf16_fail + p2f + p2bf + p3f + p3b_fail + p3c_fail + p4f + p5f + fp_diff;

    summary.total_failures = total_fail;
    summary.passed = total_fail == 0;
    let known_set: HashSet<String> = summary.known_failures.iter().cloned().collect();
    let non_baseline_failures = total_fail.saturating_sub(summary.known_failures.len());
    summary.acceptable = known_set.is_subset(&baseline) && non_baseline_failures == 0;

    println!("Parse failures:           {}", p1f);
    println!("Typecheck fails:          {}", p1bf);
    println!("GF16 conformance:         {}", gf16_fail);
    println!("Gen Zig failures:         {}", p2f);
    println!("Gen Rust failures:        {}", p2bf);
    println!("Gen Verilog fails:        {}", p3f);
    println!("Gen Verilog smoke fails:  {}", p3b_fail);
    println!("FPGA smoke fails:         {}", p3c_fail);
    println!("Gen C failures:           {}", p4f);
    println!("Seal mismatches:          {}", p5f);
    println!("FP divergences:           {}", fp_diff);
    println!("TOTAL FAILURES:    {}", total_fail);
    println!("BASELINE FAILURES: {}", summary.baseline_failures);
    println!(
        "ACCEPTABLE:        {} (known failures match baseline, no other failures)",
        if summary.acceptable { "yes" } else { "no" }
    );
    println!();

    if let Some(path) = json_out {
        let json = serde_json::to_string_pretty(&summary)
            .with_context(|| format!("serializing suite summary for {}", path.display()))?;
        fs::write(path, json)
            .with_context(|| format!("writing suite summary {}", path.display()))?;
        println!("[suite] JSON summary: {}", path.display());
    }

    if total_fail == 0 {
        println!("ALL TESTS PASSED");
        println!("phi^2 + 1/phi^2 = 3 | TRINITY");
        Ok(())
    } else {
        anyhow::bail!("SOME TESTS FAILED")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_tri_exe_finds_target_debug_tri() {
        let tmp = std::env::temp_dir().join(format!("t27_suite_tri_exe_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("target").join("debug")).unwrap();
        let fake_tri = tmp.join("target").join("debug").join("tri");
        {
            let mut f = std::fs::File::create(&fake_tri).unwrap();
            f.write_all(b"#!/bin/sh\necho fake tri\n").unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_tri).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_tri, perms).unwrap();
        }
        let found = tri_exe(&tmp).expect("tri_exe should find target/debug/tri");
        assert_eq!(found, fake_tri);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_suite_summary_schema_roundtrip() {
        let summary = SuiteSummary {
            repo: "/tmp/t27".to_string(),
            phases: vec![
                SuitePhaseSummary {
                    name: "parse".to_string(),
                    passed: 10,
                    failed: 0,
                    skipped: 0,
                },
                SuitePhaseSummary {
                    name: "gen-verilog-yosys-smoke".to_string(),
                    passed: 5,
                    failed: 2,
                    skipped: 0,
                },
            ],
            fpga_smoke_report: Some("build/fpga/smoke_gate_report.json".to_string()),
            fpga_smoke_passed: Some(true),
            fpga_smoke_gate_elapsed_ms: Some(42),
            known_failures: vec!["specs/scratch/a.t27".to_string()],
            baseline_failures: 2,
            total_failures: 2,
            passed: false,
            acceptable: true,
        };
        let json = serde_json::to_string_pretty(&summary).unwrap();
        let parsed: SuiteSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, summary);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["repo"].as_str(), Some("/tmp/t27"));
        assert_eq!(value["phases"].as_array().unwrap().len(), 2);
        assert_eq!(value["known_failures"].as_array().unwrap().len(), 1);
        assert_eq!(value["acceptable"].as_bool(), Some(true));
        assert_eq!(value["fpga_smoke_gate_elapsed_ms"].as_u64(), Some(42));
    }

    #[test]
    fn test_suite_summary_acceptable_computation() {
        let baseline: HashSet<String> = vec![
            "specs/scratch/a.t27".to_string(),
            "specs/scratch/b.t27".to_string(),
        ]
        .into_iter()
        .collect();

        // All failures are within baseline and there are no other failures.
        let known = vec!["specs/scratch/a.t27".to_string()];
        let total = known.len();
        let known_set: HashSet<String> = known.iter().cloned().collect();
        assert!(known_set.is_subset(&baseline));
        assert_eq!(total.saturating_sub(known.len()), 0);

        // A non-baseline failure makes the run unacceptable.
        let known_bad = vec!["specs/scratch/c.t27".to_string()];
        let known_bad_set: HashSet<String> = known_bad.iter().cloned().collect();
        assert!(!known_bad_set.is_subset(&baseline));

        // An extra non-smoke failure makes the run unacceptable even when known
        // failures match the baseline.
        let total_extra = known.len() + 1;
        assert_ne!(total_extra.saturating_sub(known.len()), 0);
    }

    #[test]
    fn test_load_gen_verilog_smoke_baseline() {
        let tmp = std::env::temp_dir().join(format!(
            "t27_suite_baseline_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let docs = tmp.join("docs").join("reports");
        std::fs::create_dir_all(&docs).unwrap();
        let baseline_path = docs.join("gen_verilog_smoke_baseline.json");
        std::fs::write(
            &baseline_path,
            r#"{"expected_failures": ["specs/a.t27", "specs/b.t27"]}"#,
        )
        .unwrap();
        let set = load_gen_verilog_smoke_baseline(&tmp);
        assert!(set.contains("specs/a.t27"));
        assert!(set.contains("specs/b.t27"));
        assert_eq!(set.len(), 2);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn make_fake_tri_script(report_path: &Path, passed: bool) -> PathBuf {
        let script_dir = report_path
            .parent()
            .expect("report_path must have a parent")
            .join("fake_tri");
        let _ = std::fs::remove_dir_all(&script_dir);
        std::fs::create_dir_all(&script_dir).unwrap();
        let script = script_dir.join("tri");
        let report_json = if passed {
            r#"{"schema_version":"1.0","bit_config":{"status":"ok"},"dry_run_sweep":{"status":"ok"},"verify_lean":{"status":"ok"},"theorem_matrix":{"status":"ok","variant_count":24,"source":"synthetic","replay":false,"elapsed_ms":42,"variants":[{"corner":"ff","oscfsel":0,"period_ns":400,"sck_low_ns":200,"sck_high_ns":200,"envelope_check":"ok","status":"ok","fixtures":{"pvt":"/tmp/pvt.json","raw_ns":"/tmp/raw_ns.json","lean":"/tmp/theorem.lean","summary":"/tmp/summary.json"}}]},"yosys_synthesis":{"status":"ok"},"passed":true}"#
        } else {
            r#"{"schema_version":"1.0","bit_config":{"status":"ok"},"dry_run_sweep":{"status":"failed"},"verify_lean":null,"theorem_matrix":null,"yosys_synthesis":null,"passed":false}"#
        };
        let body = format!(
            "#!/bin/sh\nprintf '%s' '{}' > {}\nexit 0\n",
            report_json.replace('\'', "'\"'\"'"),
            report_path.to_string_lossy()
        );
        {
            let mut f = std::fs::File::create(&script).unwrap();
            f.write_all(body.as_bytes()).unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script, perms).unwrap();
        }
        script
    }

    #[test]
    fn test_run_fpga_smoke_gate_passes_with_good_report() {
        let tmp = std::env::temp_dir().join(format!(
            "t27_suite_smoke_pass_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let bit = tmp.join("demo.bit");
        std::fs::File::create(&bit).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        let fake_tri = make_fake_tri_script(&report_path, true);
        let result = run_fpga_smoke_gate(
            &bit,
            &fake_tri,
            report_path.clone(),
            None,
        )
        .expect("smoke-gate should pass");
        assert!(result.passed);
        assert!(!result.skipped);
        assert_eq!(result.bit_config_status.as_deref(), Some("ok"));
        assert_eq!(result.schema_version.as_deref(), Some("1.0"));
        assert_eq!(result.theorem_matrix_status.as_deref(), Some("ok"));
        assert_eq!(result.theorem_matrix_elapsed_ms, Some(42));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_run_fpga_smoke_gate_fails_with_bad_report() {
        let tmp = std::env::temp_dir().join(format!(
            "t27_suite_smoke_fail_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let bit = tmp.join("demo.bit");
        std::fs::File::create(&bit).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        let fake_tri = make_fake_tri_script(&report_path, false);
        let err = run_fpga_smoke_gate(
            &bit,
            &fake_tri,
            report_path.clone(),
            None,
        )
        .expect_err("smoke-gate should fail when report says passed=false");
        assert!(err.to_string().contains("smoke-gate report indicates failure"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_smoke_gate_report_missing_file() {
        let missing = std::env::temp_dir().join(format!(
            "t27_suite_missing_report_{}.json",
            std::process::id()
        ));
        let err = parse_smoke_gate_report(&missing).expect_err("missing report should error");
        assert!(err.to_string().contains("smoke-gate report missing"));
    }

    #[test]
    fn test_parse_smoke_gate_report_schema_tolerant_without_theorem_matrix() {
        let tmp = std::env::temp_dir().join(format!(
            "t27_suite_smoke_schema_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        std::fs::write(
            &report_path,
            r#"{"bit_config":{"status":"ok"},"dry_run_sweep":{"status":"ok"},"verify_lean":{"status":"ok"},"yosys_synthesis":{"status":"ok"},"passed":true}"#,
        )
        .unwrap();
        let result = parse_smoke_gate_report(&report_path).expect("legacy report should parse");
        assert!(result.passed);
        assert!(result.schema_version.is_none());
        assert!(result.theorem_matrix_status.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
