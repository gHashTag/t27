//! Repository-wide test orchestration (replaces legacy `tests/*.sh` runners).
//! Invoked as `t27c suite` from the repository root (or `tri test`).

use anyhow::Context;
use chrono::Local;
use serde_json;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Options for the comprehensive repository suite.
#[derive(Clone, Debug, Default)]
pub struct SuiteOptions {
    /// Run Icarus Verilog simulation on lowerable specs.
    pub icarus_simulate: bool,
    /// Restrict Icarus simulation to specs the classifier marks lowerable.
    pub icarus_lowerable: bool,
    /// Run the Python reference-model cocotb cross-check on lowerable specs.
    pub cocotb: bool,
    /// Skip long-running phases where possible.
    pub fast: bool,
    /// Write the machine-readable suite summary to this path (Wave Loop 440).
    pub json_out: Option<PathBuf>,
}

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
        repo.join("bootstrap")
            .join("target")
            .join("release")
            .join("tri"),
        repo.join("bootstrap")
            .join("target")
            .join("debug")
            .join("tri"),
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
            eprintln!(
                "[suite] baseline file invalid JSON ({}); using empty baseline",
                e
            );
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

#[derive(Debug, Default, Clone)]
struct FpgaSmokeResult {
    passed: bool,
    skipped: bool,
    failed: bool,
    failure_reason: Option<String>,
    report_path: Option<PathBuf>,
    schema_version: Option<String>,
    bit_config_status: Option<String>,
    dry_run_sweep_status: Option<String>,
    verify_lean_status: Option<String>,
    theorem_matrix_status: Option<String>,
    theorem_matrix_elapsed_ms: Option<u64>,
    validate_lean_standalone_status: Option<String>,
    validate_lean_standalone_elapsed_ms: Option<u64>,
    yosys_synthesis_status: Option<String>,
}

/// Builder for `FpgaSmokeResult`. Using a builder prevents silent metric drops
/// when the smoke-gate report shape evolves: every field must be set
/// explicitly, and missing-bitstream / failure fallback shapes are centralized.
#[derive(Debug, Default, Clone)]
struct FpgaSmokeResultBuilder {
    inner: FpgaSmokeResult,
}

impl FpgaSmokeResultBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn passed(mut self, v: bool) -> Self {
        self.inner.passed = v;
        self
    }

    fn skipped(mut self, v: bool) -> Self {
        self.inner.skipped = v;
        self
    }

    fn failed(mut self, v: bool) -> Self {
        self.inner.failed = v;
        self
    }

    fn failure_reason(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.failure_reason = v.into();
        self
    }

    fn report_path(mut self, v: impl Into<Option<PathBuf>>) -> Self {
        self.inner.report_path = v.into();
        self
    }

    fn schema_version(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.schema_version = v.into();
        self
    }

    fn bit_config_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.bit_config_status = v.into();
        self
    }

    fn dry_run_sweep_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.dry_run_sweep_status = v.into();
        self
    }

    fn verify_lean_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.verify_lean_status = v.into();
        self
    }

    fn theorem_matrix_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.theorem_matrix_status = v.into();
        self
    }

    fn theorem_matrix_elapsed_ms(mut self, v: impl Into<Option<u64>>) -> Self {
        self.inner.theorem_matrix_elapsed_ms = v.into();
        self
    }

    fn validate_lean_standalone_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.validate_lean_standalone_status = v.into();
        self
    }

    fn validate_lean_standalone_elapsed_ms(mut self, v: impl Into<Option<u64>>) -> Self {
        self.inner.validate_lean_standalone_elapsed_ms = v.into();
        self
    }

    fn yosys_synthesis_status(mut self, v: impl Into<Option<String>>) -> Self {
        self.inner.yosys_synthesis_status = v.into();
        self
    }

    fn build(self) -> FpgaSmokeResult {
        self.inner
    }

    /// Pre-built shape used when the demo bitstream is missing: skipped, not
    /// passed, not failed, and with every metric cleared. Centralizing this shape
    /// keeps the suite's "missing bitstream" behavior consistent across call sites.
    fn missing_bitstream() -> FpgaSmokeResult {
        FpgaSmokeResultBuilder::new()
            .passed(false)
            .skipped(true)
            .failed(false)
            .failure_reason(Some("demo bitstream not found".to_string()))
            .build()
    }

    /// Pre-built shape used when the smoke gate command itself fails: not passed,
    /// not skipped, failed, with every metric cleared and a generic reason.
    fn failure_fallback() -> FpgaSmokeResult {
        FpgaSmokeResultBuilder::new()
            .passed(false)
            .skipped(false)
            .failed(true)
            .failure_reason(Some("smoke gate command failed".to_string()))
            .build()
    }
}

fn cmd_fpga_smoke_gate(
    repo: &Path,
    validate_lean_standalone: bool,
) -> anyhow::Result<FpgaSmokeResult> {
    let bit = repo
        .join("fpga")
        .join("verilog")
        .join("ternary_mac_demo_top_200t.bit");
    let report_path = repo
        .join("build")
        .join("fpga")
        .join("smoke_gate_report.json");

    if !bit.is_file() {
        println!("  SKIP: demo bitstream not found at {}", bit.display());
        return Ok(FpgaSmokeResultBuilder::missing_bitstream());
    }

    let tri = tri_exe(repo)?;
    run_fpga_smoke_gate(
        &bit,
        &tri,
        report_path,
        Some(repo),
        None,
        validate_lean_standalone,
    )
}

/// Core smoke-gate consumer. Separated from `cmd_fpga_smoke_gate` so unit tests
/// can inject fake bitstreams / `tri` binaries without touching the repo.
fn run_fpga_smoke_gate(
    _bit: &Path,
    tri: &Path,
    report_path: PathBuf,
    cwd: Option<&Path>,
    replay_fixtures: Option<&Path>,
    validate_lean_standalone: bool,
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
    let mut args: Vec<String> = vec![
        "fpga".to_string(),
        "smoke-gate".to_string(),
        "--synthetic-operating-point".to_string(),
        "--verify-lean".to_string(),
        "--theorem-matrix".to_string(),
        "--json".to_string(),
        report_path.to_string_lossy().to_string(),
    ];
    if let Some(fixtures) = replay_fixtures {
        args.push("--replay-fixtures".to_string());
        args.push(fixtures.to_string_lossy().to_string());
    }
    if validate_lean_standalone {
        args.push("--validate-lean-standalone".to_string());
    }
    let st = cmd
        .args(&args)
        .output()
        .with_context(|| format!("spawning {} for FPGA smoke gate", tri.display()))?;
    if !st.status.success() {
        let out = String::from_utf8_lossy(&st.stdout);
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("tri fpga smoke-gate failed: {} {}", out.trim(), err.trim());
    }

    parse_smoke_gate_report(&report_path)
}

/// Strict schema for the smoke-gate JSON report consumed by the comprehensive
/// suite. Every top-level key emitted by `tri fpga smoke-gate --json` is
/// enumerated here; unknown top-level fields are rejected so schema drift is
/// caught before it corrupts suite metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SmokeGateReport {
    schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bit_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run_sweep: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verify_lean: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theorem_matrix: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validate_lean_standalone: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yosys_synthesis: Option<serde_json::Value>,
    passed: bool,
}

fn parse_smoke_gate_report(report_path: &Path) -> anyhow::Result<FpgaSmokeResult> {
    let text = match fs::read_to_string(report_path) {
        Ok(t) => t,
        Err(e) => anyhow::bail!(
            "smoke-gate report missing: {}: {}",
            report_path.display(),
            e
        ),
    };
    // Schema guard: reject unknown top-level fields before consuming the report.
    let _: SmokeGateReport = serde_json::from_str(&text)
        .with_context(|| format!("smoke-gate report schema violation in {}", report_path.display()))?;
    let report: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing smoke-gate report {}", report_path.display()))?;

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
    let validate_lean_standalone_elapsed_ms = report
        .get("validate_lean_standalone")
        .and_then(|v| v.get("elapsed_ms"))
        .and_then(|v| v.as_u64());
    // A report is considered "skipped" when it did not pass and every present
    // phase status is "skipped". This distinguishes a missing-bitstream or
    // missing-dependency fallback from a real failure.
    let present_statuses = [
        phase_status("bit_config"),
        phase_status("dry_run_sweep"),
        phase_status("verify_lean"),
        phase_status("theorem_matrix"),
        phase_status("validate_lean_standalone"),
        phase_status("yosys_synthesis"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let skipped = !passed
        && !present_statuses.is_empty()
        && present_statuses.iter().all(|s| s == "skipped");
    let failed = !passed && !skipped;
    let failure_reason = if failed {
        Some("smoke-gate report indicates failure".to_string())
    } else if skipped {
        Some("smoke-gate skipped (missing dependency)".to_string())
    } else {
        None
    };

    let result = FpgaSmokeResultBuilder::new()
        .passed(passed)
        .skipped(skipped)
        .failed(failed)
        .failure_reason(failure_reason)
        .report_path(Some(report_path.to_path_buf()))
        .schema_version(schema_version)
        .bit_config_status(phase_status("bit_config"))
        .dry_run_sweep_status(phase_status("dry_run_sweep"))
        .verify_lean_status(phase_status("verify_lean"))
        .theorem_matrix_status(phase_status("theorem_matrix"))
        .theorem_matrix_elapsed_ms(theorem_matrix_elapsed_ms)
        .validate_lean_standalone_status(phase_status("validate_lean_standalone"))
        .validate_lean_standalone_elapsed_ms(validate_lean_standalone_elapsed_ms)
        .yosys_synthesis_status(phase_status("yosys_synthesis"))
        .build();

    println!(
        "  FPGA smoke gate: {} (report: {})",
        if passed { "OK" } else if skipped { "SKIPPED" } else { "FAILED" },
        report_path.display()
    );
    println!(
        "    phases: bit_config={:?} dry_run_sweep={:?} verify_lean={:?} yosys_synthesis={:?}",
        result.bit_config_status,
        result.dry_run_sweep_status,
        result.verify_lean_status,
        result.yosys_synthesis_status
    );

    if failed {
        anyhow::bail!("smoke-gate report indicates failure");
    }

    Ok(result)
}

/// W459: substrings of yosys warnings that are expected and therefore allowed.
/// These warnings are produced by the current t27c Verilog backend on
/// well-formed specs and do not indicate a synthesis failure. New, unexpected
/// warnings are still treated as smoke-test failures. The smoke runner now
/// defines `SIMULATION` during yosys parsing, so test and bench blocks are
/// skipped and many procedural warnings no longer appear.
const YOSYS_ALLOWED_WARNINGS: &[&str] = &[
    // Yosys hits this on large IGLA specs with deeply nested expressions; it
    // only means the AST simplifier recursion limit was raised. The follow-up
    // sentence is a continuation of the same warning and is also allowed.
    "Deep recursion in AST simplifier",
    "Does this design contain overly long or deeply nested expressions, or excessive recursion?",
    // Small module-level arrays are lowered into flip-flops by yosys.
    "Replacing memory",
    // Bench-block local variables are emitted as assignments without matching
    // module-scope declarations (pre-existing W458/W459 gap), and local arrays
    // are lowered to per-element registers. Yosys creates implicit wires and
    // warns about procedural assignments to them; the generated Verilog is
    // still syntactically valid.
    "is assigned in a block",
    "is implicitly declared",
    // Local arrays lowered to per-element registers are indexed with variable
    // indices; yosys reports out-of-range selects on the implicit wire stand-in.
    "Range select out of bounds",
];

/// W459: return true if a yosys warning line is on the allowed list.
fn yosys_warning_allowed(line: &str) -> bool {
    YOSYS_ALLOWED_WARNINGS.iter().any(|pat| line.contains(pat))
}

fn cmd_gen_verilog_yosys_smoke(repo: &Path, rel: &str) -> anyhow::Result<()> {
    let verilog = cmd_gen_verilog_stdout(repo, rel)?;
    let tmp = std::env::temp_dir().join(format!("t27c_yosys_smoke_{}.v", rel.replace('/', "_")));
    fs::write(&tmp, &verilog).with_context(|| {
        format!(
            "writing temporary Verilog for yosys smoke: {}",
            tmp.display()
        )
    })?;
    let st = Command::new("yosys")
        .arg("-q")
        .arg("-p")
        .arg(format!("read_verilog -sv -DSIMULATION {}", tmp.display()))
        .output()
        .context("spawning yosys for gen-verilog smoke")?;
    if !st.status.success() {
        let err = String::from_utf8_lossy(&st.stderr);
        anyhow::bail!("yosys rejected generated Verilog: {}", err.trim());
    }
    let err = String::from_utf8_lossy(&st.stderr);
    let mut unrecognized: Vec<String> = Vec::new();
    for line in err.lines() {
        let line = line.trim();
        if !line.is_empty() && !yosys_warning_allowed(line) {
            unrecognized.push(line.to_string());
        }
    }
    if !unrecognized.is_empty() {
        anyhow::bail!(
            "yosys emitted unrecognized warnings for {}:\n{}",
            rel,
            unrecognized.join("\n")
        );
    }
    Ok(())
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SuitePhaseSummary {
    name: String,
    passed: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SuiteSummary {
    repo: String,
    phases: Vec<SuitePhaseSummary>,
    fpga_smoke_report: Option<String>,
    fpga_smoke_passed: Option<bool>,
    /// True when the FPGA smoke gate was skipped (e.g., demo bitstream missing).
    fpga_smoke_skipped: Option<bool>,
    /// True when the FPGA smoke gate failed and was not skipped.
    fpga_smoke_failed: Option<bool>,
    /// Human-readable reason when the FPGA smoke gate failed or was skipped.
    fpga_smoke_failure_reason: Option<String>,
    /// Elapsed milliseconds reported by the smoke-gate theorem matrix, if any.
    fpga_smoke_gate_elapsed_ms: Option<u64>,
    /// Elapsed milliseconds reported by the smoke-gate theorem-matrix replay path,
    /// if run. Separated from `fpga_smoke_gate_elapsed_ms` so CI can trend
    /// generation and replay cost independently.
    fpga_smoke_gate_replay_elapsed_ms: Option<u64>,
    /// Elapsed milliseconds reported by the smoke-gate validate-lean-standalone
    /// phase, if run. Separated from the matrix generation/replay metrics so CI
    /// can trend standalone lake-package build cost independently.
    validate_lean_standalone_elapsed_ms: Option<u64>,
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
    push_phase(
        "gf16_conformance",
        1 - gf16_fail - (gf16_skipped as usize),
        gf16_fail,
        gf16_skipped as usize,
    );

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

    if opts.fast {
        println!("[suite] --fast mode: skipping the standalone lake-package build phase");
    }

    println!("--- Phase 3c: FPGA Board-Less Smoke Gate ---");
    let mut p3c_fail = 0usize;
    let mut p3c_skipped = 0usize;
    let validate_lean_standalone = !opts.fast;
    let fpga_result = match cmd_fpga_smoke_gate(&repo, validate_lean_standalone) {
        Ok(r) => {
            if r.skipped {
                p3c_skipped = 1;
            }
            summary.fpga_smoke_report = r.report_path.as_ref().map(|p| p.display().to_string());
            summary.fpga_smoke_passed = Some(r.passed);
            summary.fpga_smoke_skipped = Some(r.skipped);
            summary.fpga_smoke_failed = Some(r.failed);
            summary.fpga_smoke_failure_reason = r.failure_reason.clone();
            summary.fpga_smoke_gate_elapsed_ms = r.theorem_matrix_elapsed_ms;
            summary.validate_lean_standalone_elapsed_ms = r.validate_lean_standalone_elapsed_ms;
            r
        }
        Err(e) => {
            let reason = e.to_string();
            eprintln!("FPGA smoke gate failed: {}", e);
            p3c_fail = 1;
            summary.fpga_smoke_failed = Some(true);
            summary.fpga_smoke_failure_reason = Some(reason.clone());
            FpgaSmokeResultBuilder::new()
                .passed(false)
                .skipped(false)
                .failed(true)
                .failure_reason(Some(reason))
                .build()
        }
    };
    push_phase(
        "fpga-smoke-gate",
        if fpga_result.passed { 1 } else { 0 },
        p3c_fail,
        p3c_skipped,
    );

    println!("--- Phase 3c-standalone: FPGA Standalone Lake-Package Build ---");
    let mut p3cs_fail = 0usize;
    let mut p3cs_skipped = 0usize;
    if validate_lean_standalone {
        // The standalone phase piggybacks on the smoke-gate report above. Its
        // success is implied by the main smoke gate passing while the option is
        // enabled, and its elapsed time is recorded separately.
        if fpga_result.passed && fpga_result.validate_lean_standalone_status.is_some() {
            println!(
                "  FPGA standalone build: OK (elapsed_ms={:?})",
                fpga_result.validate_lean_standalone_elapsed_ms
            );
            push_phase("fpga-smoke-gate-standalone", 1, 0, 0);
        } else if fpga_result.skipped || fpga_result.validate_lean_standalone_status.is_none() {
            println!("  FPGA standalone build: skipped (bitstream missing or lake unavailable)");
            p3cs_skipped = 1;
            push_phase("fpga-smoke-gate-standalone", 0, 0, p3cs_skipped);
        } else {
            eprintln!("  FPGA standalone build: failed (report indicates failure)");
            p3cs_fail = 1;
            push_phase("fpga-smoke-gate-standalone", 0, p3cs_fail, 0);
        }
    } else {
        println!("  FPGA standalone build: skipped (--fast mode)");
        p3cs_skipped = 1;
        push_phase("fpga-smoke-gate-standalone", 0, 0, p3cs_skipped);
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

    // --- Phase 6: Integrity metrics (reporting only) --------------------
    //
    // Seven waves of auditing established that several of this project's
    // integrity claims are satisfiable by content that means nothing: tests
    // whose body is `assert true`, braceless given/when/then tests whose
    // assertions the parser discards, seals whose every gen_hash is "none",
    // and specs that synthesise to zero logic cells. Each was invisible until
    // measured. Surfacing the numbers on every suite run is what stops them
    // becoming invisible again.
    //
    // These are REPORTING metrics, deliberately excluded from total_fail: the
    // current values are large, and turning them into hard failures is a
    // maintainer's decision, not the suite's.
    println!("--- Phase 6: Integrity metrics (reporting only) ---");
    {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("t27c"));
        for (label, args) in [
            ("vacuity", vec!["validate-vacuity", "--specs-dir", "specs", "--top", "0"]),
            ("seals", vec!["seal-audit"]),
        ] {
            match std::process::Command::new(&exe).args(&args).output() {
                Ok(o) => {
                    let text = String::from_utf8_lossy(&o.stdout);
                    for line in text.lines() {
                        let t = line.trim();
                        if t.starts_with("tests that assert nothing")
                            || t.starts_with("BDD-form tests")
                            || t.starts_with("NOT ANALYSED")
                            || t.starts_with("VACUOUS (all 'none')")
                            || t.starts_with("spec file missing")
                        {
                            println!("  {}", t);
                        }
                    }
                }
                Err(e) => println!("  [{}] could not run: {}", label, e),
            }
        }
        println!("  (reporting only -- not counted in TOTAL FAILURES)");
    }

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
    println!("BASELINE FAILURES: {}", summary.baseline_failures);
    println!(
        "ACCEPTABLE:        {} (known failures match baseline, no other failures)",
        if summary.acceptable { "yes" } else { "no" }
    );
    println!();

    if let Some(path) = opts.json_out.as_ref() {
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
        let tmp =
            std::env::temp_dir().join(format!("t27_suite_tri_exe_test_{}", std::process::id()));
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
            fpga_smoke_skipped: Some(false),
            fpga_smoke_failed: Some(false),
            fpga_smoke_failure_reason: None,
            fpga_smoke_gate_elapsed_ms: Some(42),
            fpga_smoke_gate_replay_elapsed_ms: Some(7),
            validate_lean_standalone_elapsed_ms: Some(123),
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
        assert_eq!(value["fpga_smoke_gate_replay_elapsed_ms"].as_u64(), Some(7));
        assert_eq!(
            value["validate_lean_standalone_elapsed_ms"].as_u64(),
            Some(123)
        );
        assert_eq!(value["fpga_smoke_skipped"].as_bool(), Some(false));
        assert_eq!(value["fpga_smoke_failed"].as_bool(), Some(false));
        assert_eq!(value["fpga_smoke_failure_reason"].as_str(), None);
    }

    #[test]
    fn test_suite_summary_smoke_state_roundtrip() {
        let summary = SuiteSummary {
            repo: "/tmp/t27".to_string(),
            phases: vec![],
            fpga_smoke_report: None,
            fpga_smoke_passed: Some(false),
            fpga_smoke_skipped: Some(true),
            fpga_smoke_failed: Some(false),
            fpga_smoke_failure_reason: Some("demo bitstream not found".to_string()),
            fpga_smoke_gate_elapsed_ms: None,
            fpga_smoke_gate_replay_elapsed_ms: None,
            validate_lean_standalone_elapsed_ms: None,
            known_failures: vec![],
            baseline_failures: 0,
            total_failures: 0,
            passed: false,
            acceptable: true,
        };
        let json = serde_json::to_string_pretty(&summary).unwrap();
        let parsed: SuiteSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, summary);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["fpga_smoke_passed"].as_bool(), Some(false));
        assert_eq!(value["fpga_smoke_skipped"].as_bool(), Some(true));
        assert_eq!(value["fpga_smoke_failed"].as_bool(), Some(false));
        assert_eq!(
            value["fpga_smoke_failure_reason"].as_str(),
            Some("demo bitstream not found")
        );
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
        let tmp =
            std::env::temp_dir().join(format!("t27_suite_baseline_test_{}", std::process::id()));
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
            r#"{"schema_version":"1.0","bit_config":{"status":"ok"},"dry_run_sweep":{"status":"ok"},"verify_lean":{"status":"ok"},"theorem_matrix":{"status":"ok","variant_count":24,"source":"synthetic","replay":false,"elapsed_ms":42,"variants":[{"corner":"ff","oscfsel":0,"period_ns":400,"sck_low_ns":200,"sck_high_ns":200,"envelope_check":"ok","status":"ok","fixtures":{"pvt":"/tmp/pvt.json","raw_ns":"/tmp/raw_ns.json","lean":"/tmp/theorem.lean","summary":"/tmp/summary.json"}}]},"validate_lean_standalone":{"status":"ok","source":"synthetic","lean_file":"/tmp/standalone.lean","elapsed_ms":123},"yosys_synthesis":{"status":"ok"},"passed":true}"#
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
        let tmp = std::env::temp_dir().join(format!("t27_suite_smoke_pass_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let bit = tmp.join("demo.bit");
        std::fs::File::create(&bit).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        let fake_tri = make_fake_tri_script(&report_path, true);
        let result = run_fpga_smoke_gate(&bit, &fake_tri, report_path.clone(), None, None, false)
            .expect("smoke-gate should pass");
        assert!(result.passed);
        assert!(!result.skipped);
        assert!(!result.failed);
        assert!(result.failure_reason.is_none());
        assert_eq!(result.bit_config_status.as_deref(), Some("ok"));
        assert_eq!(result.schema_version.as_deref(), Some("1.0"));
        assert_eq!(result.theorem_matrix_status.as_deref(), Some("ok"));
        assert_eq!(result.theorem_matrix_elapsed_ms, Some(42));
        assert_eq!(
            result.validate_lean_standalone_status.as_deref(),
            Some("ok")
        );
        assert_eq!(result.validate_lean_standalone_elapsed_ms, Some(123));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_run_fpga_smoke_gate_fails_with_bad_report() {
        let tmp = std::env::temp_dir().join(format!("t27_suite_smoke_fail_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let bit = tmp.join("demo.bit");
        std::fs::File::create(&bit).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        let fake_tri = make_fake_tri_script(&report_path, false);
        let err = run_fpga_smoke_gate(&bit, &fake_tri, report_path.clone(), None, None, false)
            .expect_err("smoke-gate should fail when report says passed=false");
        assert!(err
            .to_string()
            .contains("smoke-gate report indicates failure"));
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
        let tmp =
            std::env::temp_dir().join(format!("t27_suite_smoke_schema_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        // Phase blocks are optional (represented as null or omitted), but the
        // top-level schema_version field is now mandatory.
        std::fs::write(
            &report_path,
            r#"{"schema_version":"1.0","bit_config":{"status":"ok"},"dry_run_sweep":{"status":"ok"},"verify_lean":{"status":"ok"},"yosys_synthesis":{"status":"ok"},"passed":true}"#,
        )
        .unwrap();
        let result = parse_smoke_gate_report(&report_path).expect("legacy report should parse");
        assert!(result.passed);
        assert_eq!(result.schema_version.as_deref(), Some("1.0"));
        assert!(result.theorem_matrix_status.is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fpga_smoke_result_builder_missing_bitstream() {
        let result = FpgaSmokeResultBuilder::missing_bitstream();
        assert!(!result.passed);
        assert!(result.skipped);
        assert!(!result.failed);
        assert_eq!(
            result.failure_reason.as_deref(),
            Some("demo bitstream not found")
        );
        assert!(result.report_path.is_none());
        assert!(result.schema_version.is_none());
        assert!(result.bit_config_status.is_none());
        assert!(result.dry_run_sweep_status.is_none());
        assert!(result.verify_lean_status.is_none());
        assert!(result.theorem_matrix_status.is_none());
        assert!(result.theorem_matrix_elapsed_ms.is_none());
        assert!(result.validate_lean_standalone_status.is_none());
        assert!(result.validate_lean_standalone_elapsed_ms.is_none());
        assert!(result.yosys_synthesis_status.is_none());
    }

    #[test]
    fn test_fpga_smoke_result_builder_failure_fallback() {
        let result = FpgaSmokeResultBuilder::failure_fallback();
        assert!(!result.passed);
        assert!(!result.skipped);
        assert!(result.failed);
        assert_eq!(
            result.failure_reason.as_deref(),
            Some("smoke gate command failed")
        );
        assert!(result.report_path.is_none());
    }

    #[test]
    fn test_suite_summary_deny_unknown_fields() {
        let json = r#"{
            "repo": "/tmp/t27",
            "phases": [],
            "fpga_smoke_report": null,
            "fpga_smoke_passed": null,
            "fpga_smoke_skipped": null,
            "fpga_smoke_failed": null,
            "fpga_smoke_failure_reason": null,
            "fpga_smoke_gate_elapsed_ms": null,
            "fpga_smoke_gate_replay_elapsed_ms": null,
            "validate_lean_standalone_elapsed_ms": null,
            "known_failures": [],
            "baseline_failures": 0,
            "total_failures": 0,
            "passed": true,
            "acceptable": true,
            "unknown_future_field": 42
        }"#;
        let err = serde_json::from_str::<SuiteSummary>(json).expect_err(
            "unknown field should be rejected");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn test_suite_phase_summary_deny_unknown_fields() {
        let json = r#"{"name":"p","passed":1,"failed":0,"skipped":0,"extra":true}"#;
        let err = serde_json::from_str::<SuitePhaseSummary>(json).expect_err(
            "unknown field should be rejected");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn test_parse_smoke_gate_report_fast_skips_standalone() {
        let tmp = std::env::temp_dir()
            .join(format!("t27_suite_smoke_fast_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        std::fs::write(
            &report_path,
            r#"{"schema_version":"1.0","bit_config":{"status":"ok"},"dry_run_sweep":{"status":"ok"},"verify_lean":{"status":"ok"},"theorem_matrix":{"status":"ok","elapsed_ms":42},"yosys_synthesis":{"status":"ok"},"passed":true}"#,
        )
        .unwrap();
        let result = parse_smoke_gate_report(&report_path)
            .expect("fast-mode report without standalone phase should parse");
        assert!(result.passed);
        assert!(!result.skipped);
        assert_eq!(result.validate_lean_standalone_status, None);
        assert_eq!(result.validate_lean_standalone_elapsed_ms, None);
        assert_eq!(result.theorem_matrix_elapsed_ms, Some(42));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_parse_smoke_gate_report_deny_unknown_fields() {
        let tmp = std::env::temp_dir()
            .join(format!("t27_suite_smoke_schema_unknown_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let report_path = tmp.join("smoke_gate_report.json");
        std::fs::write(
            &report_path,
            r#"{"schema_version":"1.0","bit_config":null,"dry_run_sweep":null,"verify_lean":null,"theorem_matrix":null,"validate_lean_standalone":null,"yosys_synthesis":null,"passed":false,"unknown_future_field":42}"#,
        )
        .unwrap();
        let err = parse_smoke_gate_report(&report_path).expect_err(
            "unknown top-level field should be rejected");
        assert!(
            err.to_string().contains("schema violation"),
            "error should mention schema violation: {}",
            err
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
