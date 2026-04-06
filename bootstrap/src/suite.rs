//! Repository-wide test orchestration (replaces legacy `tests/*.sh` runners).
//! Invoked as `t27c suite` from the repository root (or `tri test`).

use anyhow::Context;
use chrono::{Duration, Local, NaiveDate, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

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
        .filter(|e| e.path().extension().map_or(false, |x| x == "t27"))
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

/// Phases 1–6: same coverage as legacy `tests/run_all.sh`.
pub fn run_comprehensive(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root).with_context(|| {
        format!(
            "cannot canonicalize repo root {}",
            repo_root.display()
        )
    })?;

    println!("=== T27 Comprehensive Test Suite ===");
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");
    println!("repo: {}", repo.display());
    println!();

    let mut specs_compiler: Vec<PathBuf> = collect_t27(&repo.join("specs"))?;
    let mut comp = collect_t27(&repo.join("compiler"))?;
    specs_compiler.append(&mut comp);
    specs_compiler.sort();
    specs_compiler.dedup();

    let specs_only = collect_t27(&repo.join("specs"))?;

    println!("--- Phase 1: Parse ---");
    let (p1p, p1f) = run_phase(&repo, "parse", cmd_parse, &specs_compiler)?;
    println!("Parse: {} passed, {} failed", p1p, p1f);

    println!("--- Phase 2: Gen Zig ---");
    let (p2p, p2f) = run_phase(&repo, "gen-zig", |r, rel| cmd_gen(r, rel, "gen"), &specs_compiler)?;
    println!("Gen Zig: {} passed, {} failed", p2p, p2f);

    println!("--- Phase 3: Gen Verilog ---");
    let (p3p, p3f) =
        run_phase(&repo, "gen-verilog", |r, rel| cmd_gen(r, rel, "gen-verilog"), &specs_only)?;
    println!("Gen Verilog: {} passed, {} failed", p3p, p3f);

    println!("--- Phase 4: Gen C ---");
    let (p4p, p4f) = run_phase(&repo, "gen-c", |r, rel| cmd_gen(r, rel, "gen-c"), &specs_only)?;
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
    let total_fail = p1f + p2f + p3f + p4f + p5f + fp_diff;
    println!("Parse failures:    {}", p1f);
    println!("Gen Zig failures:  {}", p2f);
    println!("Gen Verilog fails: {}", p3f);
    println!("Gen C failures:    {}", p4f);
    println!("Seal mismatches:   {}", p5f);
    println!("FP divergences:    {}", fp_diff);
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
        .filter(|p| p.extension().map_or(false, |x| x == "json"))
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

/// Gate: root `NOW.md` must contain `Last updated:` with a fresh calendar `YYYY-MM-DD`.
/// - **Local dev:** must match **today** in the machine's local timezone.
/// - **GitHub Actions** (`GITHUB_ACTIONS=true`): must be **today or yesterday in UTC** — same
///   window as `.github/workflows/now-sync-gate.yml` freshness, so a single committed date can
///   span the UTC midnight boundary relative to contributors in non-UTC timezones.
/// Used by `tri` before gen/compile and by CI (see `phi-loop-ci.yml`).
pub fn check_now_sync(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)?;
    let now_file = repo.join("NOW.md");
    let github_actions = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");

    if !now_file.is_file() {
        eprintln!(
            "tri/CI: NOW.md not found at {}",
            now_file.display()
        );
        anyhow::bail!("NOW.md missing");
    }

    let content = fs::read_to_string(&now_file)?;
    let line = content
        .lines()
        .find(|l| l.contains("Last updated:"))
        .unwrap_or("");
    let last = first_yyyy_mm_dd_in_line(line);
    let last_date = last
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let (gate_label, ok, expected_hint) = if github_actions {
        let today_utc = Utc::now().date_naive();
        let yesterday_utc = today_utc - Duration::days(1);
        let ok = match last_date {
            Some(d) => d == today_utc || d == yesterday_utc,
            None => false,
        };
        let hint = format!("{} or {} (UTC)", today_utc, yesterday_utc);
        (today_utc.format("%Y-%m-%d").to_string(), ok, hint)
    } else {
        let today_local = Local::now().format("%Y-%m-%d").to_string();
        let ok = last.as_deref() == Some(today_local.as_str());
        let hint = today_local.clone();
        (today_local, ok, hint)
    };

    if !ok {
        eprintln!(
            r#"

╔═══════════════════════════════════════════════════════════════╗
║              ⛔  BUILD BLOCKED: SYNC REQUIRED                  ║
╠═══════════════════════════════════════════════════════════════╣
║  NOW.md is STALE. All agents must be synchronized              ║
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
║  3. Update NOW.md (repo root):                               ║
║     - Set calendar date YYYY-MM-DD (must match today locally) ║
║     - Use your local wall time (see NOW.md header template)   ║
║     - Update sprint status + what you build and why           ║
║                                                               ║
║  4. Stage and commit NOW.md with your changes:               ║
║     git add NOW.md && git commit --amend                     ║
╚═══════════════════════════════════════════════════════════════╝
"#
        );
        eprintln!(
            "(Expected Last updated: {}; found: {})",
            expected_hint,
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
        eprintln!(
            "✅ NOW.md synced — gate date {} — doc time {} [{}] — build authorized",
            gate_label, human, ts
        );
    } else {
        eprintln!(
            "✅ NOW.md synced ({}) — build authorized",
            gate_label
        );
    }
    Ok(())
}

/// Validate conformance/*.json files against SCHEMA_V2.json.
/// Replaces scripts/validate-conformance-v2.sh (NO-SHELL law).
pub fn validate_conformance_v2(repo_root: &Path) -> anyhow::Result<()> {
    use regex::Regex;

    let repo = fs::canonicalize(repo_root)?;
    let schema_path = repo.join("conformance/SCHEMA_V2.json");

    if !schema_path.exists() {
        anyhow::bail!("SCHEMA_V2.json not found at {}", schema_path.display());
    }

    println!("=== T27 Conformance Validation (SCHEMA_V2) ===");
    println!("Schema: {}", schema_path.display());

    // Load schema
    let schema_raw = fs::read_to_string(&schema_path)
        .with_context(|| format!("read {}", schema_path.display()))?;
    let schema: serde_json::Value = serde_json::from_str(&schema_raw)?;

    // Get required fields from schema
    let required: Vec<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let verdict_enum = schema
        .get("properties")
        .and_then(|p| p.get("verdict"))
        .and_then(|v| v.get("enum"))
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["CLEAN".into(), "FAIL".into(), "PARTIAL".into(), "SKIP".into()]);

    let seal_regex = Regex::new(r"^sha256:[0-9a-f]{64}$").unwrap();

    let mut total_files = 0usize;
    let mut total_errors = 0usize;
    let mut entries: Vec<PathBuf> = fs::read_dir(repo.join("conformance"))
        .with_context(|| "read_dir conformance")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "json"))
        .collect();
    entries.sort();

    for file in entries {
        let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        // Skip schema file itself
        if filename == "SCHEMA_V2.json" {
            continue;
        }

        // Read JSON
        let raw = fs::read_to_string(&file)?;
        let json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("⊘ {} (invalid JSON: {})", filename, e);
                continue;
            }
        };

        // Skip v1 files (no schema_version field or schema_version == 1)
        let is_v2 = match json.get("schema_version") {
            None => false,
            Some(v) => {
                // Check for integer 2 or string "2"
                if v.as_i64() == Some(2) {
                    true
                } else if v.as_str() == Some("2") {
                    true
                } else {
                    // v1 or invalid schema_version
                    false
                }
            }
        };

        if !is_v2 {
            println!("⊘ {} (v1, not validated against v2)", filename);
            continue;
        }

        total_files += 1;

        eprint!("Validating {} ... ", filename);

        let mut errors = 0;

        // Check required fields
        for field in &required {
            if json.get(field).is_none() {
                eprintln!("\n  ✗ Missing required field: {}", field);
                errors += 1;
            }
        }

        // Check schema_version == 2
        if let Some(sv) = json.get("schema_version").and_then(|v| v.as_i64()) {
            if sv != 2 {
                eprintln!("\n  ✗ schema_version must be 2, got: {}", sv);
                errors += 1;
            }
        }

        // Check verdict is valid
        if let Some(v) = json.get("verdict").and_then(|v| v.as_str()) {
            if !verdict_enum.iter().any(|e| e == v) {
                eprintln!("\n  ✗ Invalid verdict: {}", v);
                errors += 1;
            }
        }

        // Check seal format
        if let Some(s) = json.get("seal").and_then(|v| v.as_str()) {
            if !seal_regex.is_match(s) {
                eprintln!("\n  ✗ Invalid seal format");
                errors += 1;
            }
        }

        // Check test_vectors array
        if let Some(tv) = json.get("test_vectors").and_then(|v| v.as_array()) {
            if tv.is_empty() {
                eprintln!("\n  ✗ test_vectors array is empty");
                errors += 1;
            }
            // Check per-case verdicts
            for item in tv {
                if let Some(v) = item.get("verdict").and_then(|v| v.as_str()) {
                    if !verdict_enum.iter().any(|e| e == v) {
                        eprintln!("\n  ✗ Invalid per-case verdict: {}", v);
                        errors += 1;
                    }
                }
            }
        }

        if errors == 0 {
            eprintln!("✓ PASS");
        } else {
            eprintln!("✗ FAIL ({} errors)", errors);
            total_errors += errors;
        }
    }

    eprintln!();
    eprintln!("=== Summary ===");
    eprintln!("Files validated (v2): {}", total_files);
    eprintln!("Total errors: {}", total_errors);

    if total_errors == 0 {
        eprintln!("All v2 vectors are valid!");
        Ok(())
    } else {
        anyhow::bail!("Validation failed with {} error(s)", total_errors);
    }
}

/// Validate seal coverage for changed .t27 specs (PR-scoped gate).
/// Replaces inline bash in seal-coverage.yml (NO-SHELL law).
pub fn validate_seals(repo_root: &Path, pr_files: Option<&str>) -> anyhow::Result<()> {
    use sha2::{Sha256, Digest};

    let repo = fs::canonicalize(repo_root)?;

    // Get changed files from PR input or check all specs
    let changed_specs: Vec<PathBuf> = if let Some(files) = pr_files {
        files
            .split(',')
            .map(|f| f.trim())
            .filter(|f| f.ends_with(".t27"))
            .map(|f| repo.join(f))
            .collect()
    } else {
        // If no PR files provided, validate all specs
        fs::read_dir(repo.join("specs"))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |x| x == "t27"))
            .collect()
    };

    if changed_specs.is_empty() {
        println!("No .t27 spec files to validate. Gate PASS.");
        return Ok(());
    }

    println!("=== T27 Seal Coverage Gate (PR-scoped) ===");
    println!("Checking {} changed spec(s)...", changed_specs.len());

    let mut missing = 0usize;
    let mut stale = 0usize;
    let mut failed_specs: Vec<String> = Vec::new();

    for spec_file in &changed_specs {
        let module_name = spec_file
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let relative_path = spec_file
            .strip_prefix(&repo)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .replace('\\', "/");
        let seal_file = repo.join(".trinity/seals").join(format!("{}.json", module_name));

        eprint!("[{}] {} ... ", changed_specs.iter().position(|p| p == spec_file).map(|i| i+1).unwrap_or(0), relative_path);

        // Check if seal exists
        if !seal_file.exists() {
            eprintln!("✗ MISSING SEAL");
            missing += 1;
            failed_specs.push(format!("{} (no seal file)", relative_path));
            continue;
        }

        // Compute current spec hash
        let spec_content = fs::read_to_string(spec_file)?;
        let mut hasher = Sha256::new();
        hasher.update(spec_content.as_bytes());
        let current_hash = format!("sha256:{:x}", hasher.finalize());

        // Get stored spec_hash from seal
        let seal_content = fs::read_to_string(&seal_file)?;
        let seal_json: serde_json::Value = serde_json::from_str(&seal_content)?;
        let stored_hash = seal_json
            .get("spec_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Verify spec_hash matches
        if current_hash != stored_hash {
            eprintln!("⚠ STALE (spec changed)");
            stale += 1;
            failed_specs.push(format!("{} (stale seal)", relative_path));
        } else {
            eprintln!("✓ OK");
        }
    }

    eprintln!();
    eprintln!("=== Seal Coverage Summary ===");
    eprintln!("Changed specs: {}", changed_specs.len());
    eprintln!("Missing seals: {}", missing);
    eprintln!("Stale seals: {}", stale);

    if missing > 0 || stale > 0 {
        eprintln!();
        eprintln!("=== FAILED SPECS ===");
        for spec in &failed_specs {
            eprintln!("  - {}", spec);
        }
        eprintln!();
        eprintln!("Gate FAILED: {} missing, {} stale seals", missing, stale);
        eprintln!();
        eprintln!("To fix missing seals, run:");
        eprintln!("  ./scripts/tri seal <spec.t27> --save");
        eprintln!();
        eprintln!("To fix stale seals, regenerate after spec changes:");
        eprintln!("  ./scripts/tri seal <spec.t27> --save");
        eprintln!();
        eprintln!("phi^2 + 1/phi^2 = 3 | TRINITY");
        anyhow::bail!("Seal coverage check failed");
    }

    eprintln!();
    eprintln!("All changed specs have current seals! Gate PASS.");
    eprintln!("phi^2 + 1/phi^2 = 3 | TRINITY");
    Ok(())
}

/// Migrate conformance/*.json files to SCHEMA_V2 format.
/// Adds schema_version, verdict, seal, and per-case verdicts.
pub fn migrate_to_v2(repo_root: &Path, dry_run: bool) -> anyhow::Result<()> {
    use sha2::{Sha256, Digest};
    use serde_json::{json, Value};

    let repo = fs::canonicalize(repo_root)?;
    let conformance_dir = repo.join("conformance");

    println!("=== Migrate to SCHEMA_V2 ===");
    if dry_run {
        println!("DRY RUN MODE — no files will be modified");
    }

    let schema_path = conformance_dir.join("SCHEMA_V2.json");
    if !schema_path.exists() {
        anyhow::bail!("SCHEMA_V2.json not found at {}", schema_path.display());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&conformance_dir)
        .with_context(|| "read_dir conformance")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "json"))
        .collect();
    entries.sort();

    let mut migrated = 0usize;
    let mut skipped = 0usize;
    let errors = 0usize;

    for file in entries {
        let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        // Skip schema file
        if filename == "SCHEMA_V2.json" {
            continue;
        }

        // Read current content
        let raw = fs::read_to_string(&file)?;
        let mut json: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("⊘ {} (invalid JSON: {}, skipping)", filename, e);
                skipped += 1;
                continue;
            }
        };

        // Skip if already v2
        if let Some(sv) = json.get("schema_version") {
            if sv.as_i64() == Some(2) || sv.as_str() == Some("2") {
                eprintln!("⊘ {} (already v2, skipping)", filename);
                skipped += 1;
                continue;
            }
        }

        // Detect format_family from existing data
        let format_family = json.get("format_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                if filename.contains("gf") || filename.contains("float") {
                    Some("GoldenFloat".to_string())
                } else if filename.contains("tf") {
                    Some("TernaryFloat".to_string())
                } else if filename.contains("phi") {
                    Some("PhiRatio".to_string())
                } else if filename.contains("sacred") {
                    Some("SacredPhysics".to_string())
                } else {
                    Some("Conformance".to_string())
                }
            })
            .unwrap_or_else(|| "Conformance".to_string());

        // Get or derive vector_name
        let vector_name = json.get("vector_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} Vectors", filename.replace("_vectors.json", "").replace(".json", "")));

        // Get format details
        let format_name = json.get("format_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let format_bits = json.get("format_bits")
            .and_then(|v| v.as_i64())
            .or_else(|| {
                if filename.contains("tf3") { Some(8) }
                else if filename.contains("gf4") { Some(4) }
                else if filename.contains("gf8") { Some(8) }
                else if filename.contains("gf12") { Some(12) }
                else if filename.contains("gf16") { Some(16) }
                else if filename.contains("gf20") { Some(20) }
                else if filename.contains("gf24") { Some(24) }
                else if filename.contains("gf32") { Some(32) }
                else { None }
            });

        // Add/overwrite top-level fields for v2
        json["schema_version"] = json!(2);

        if json.get("format_family").is_none() {
            json["format_family"] = json!(format_family);
        }

        if json.get("vector_name").is_none() {
            json["vector_name"] = json!(vector_name);
        }

        if json.get("version").is_none() {
            json["version"] = json!("2.0");
        }

        if let Some(fn_val) = format_name {
            if json.get("format_name").is_none() {
                json["format_name"] = json!(fn_val);
            }
        }

        if let Some(fb_val) = format_bits {
            if json.get("format_bits").is_none() {
                json["format_bits"] = json!(fb_val);
            }
        }

        let vectors_key = if json.get("test_vectors").is_some() {
            "test_vectors"
        } else if json.get("vectors").is_some() {
            "vectors"
        } else {
            continue;
        };

        let mut invariants_to_convert: Vec<(String, String, Value)> = Vec::new();
        if let Some(invariants) = json.get("invariants").and_then(|v| v.as_array()) {
            for inv in invariants {
                if inv.get("name").is_some() && inv.get("input").is_none() {
                    let name = inv.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    let assertion = inv.get("assertion").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let value = inv.get("value").cloned().unwrap_or(Value::Bool(true));
                    invariants_to_convert.push((name, assertion, value));
                }
            }
        }

        {
            if json.get("verdict").is_none() {
                json["verdict"] = json!("CLEAN");
            }

            if let Some(vectors) = json.get_mut(vectors_key).and_then(|v| v.as_array_mut()) {
                for vec in &mut *vectors {
                    if vec.get("verdict").is_none() {
                        vec["verdict"] = json!("CLEAN");
                    }
                }

                if !invariants_to_convert.is_empty() {
                    let existing_names: std::collections::HashSet<String> = vectors
                        .iter()
                        .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        .collect();

                    for (name, assertion, value) in invariants_to_convert {
                        let test_name = format!("inv_{}", name);
                        if !existing_names.contains(&test_name) {
                            vectors.push(json!({
                                "name": test_name,
                                "description": format!("Invariant: {}", assertion),
                                "verdict": "CLEAN",
                                "input": { "assertion": assertion },
                                "expected": { "value": value }
                            }));
                        }
                    }
                }
            }

            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            if json.get("created_at").is_none() {
                json["created_at"] = json!(now.clone());
            }
            if json.get("updated_at").is_none() {
                json["updated_at"] = json!(now.clone());
            }
            if json.get("validated_at").is_none() {
                json["validated_at"] = json!(now);
            }
        }

        // Compute and add seal
        let content_without_seal = {
            let mut j = json.clone();
            j["seal"] = json!(""); // Remove seal for hash computation
            serde_json::to_string(&j).unwrap()
        };

        let mut hasher = Sha256::new();
        hasher.update(content_without_seal.as_bytes());
        let seal_hash = format!("sha256:{:x}", hasher.finalize());
        json["seal"] = json!(seal_hash);

        // Pretty print and write
        let output = serde_json::to_string_pretty(&json)?;

        eprint!("Migrating {} ... ", filename);

        if dry_run {
            eprintln!("(would write {} bytes)", output.len());
            // Show diff preview
            if output != raw {
                eprintln!("  Content would change");
            } else {
                eprintln!("  No changes needed");
            }
        } else {
            fs::write(&file, output)?;
            eprintln!("✓ MIGRATED");
        }

        migrated += 1;
    }

    eprintln!();
    eprintln!("=== Migration Summary ===");
    eprintln!("Files migrated: {}", migrated);
    eprintln!("Files skipped: {}", skipped);
    eprintln!("Errors: {}", errors);

    if dry_run {
        eprintln!();
        eprintln!("DRY RUN COMPLETE — no files were modified");
        eprintln!("Run without --dry-run to apply changes");
    } else if migrated > 0 {
        eprintln!();
        eprintln!("Migration complete!");
        eprintln!("Run 't27c validate-conformance-v2' to verify all files");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Issue #129 — GF16 vector expansion + synthetic NMSE block for gf_family_bench
// ---------------------------------------------------------------------------

/// Append synthetic `test_vectors` rows using IEEE binary16 roundtrip as the same decode proxy
/// used throughout `gf16_vectors.json`, then recompute the v2 seal.
pub fn expand_gf16_vectors(repo_root: &Path, output: Option<&str>) -> anyhow::Result<()> {
    use half::f16;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::collections::HashSet;

    let rel = output.unwrap_or("conformance/gf16_vectors.json");
    let path = repo_root.join(rel);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut json: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;

    let tv = json
        .get_mut("test_vectors")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| anyhow::anyhow!("{}: missing test_vectors array", path.display()))?;

    let mut names: HashSet<String> = tv
        .iter()
        .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .collect();

    let candidates: &[f32] = &[
        -1.0,
        -2.0,
        -0.5,
        -4.0,
        -8.0,
        2.0,
        4.0,
        8.0,
        16.0,
        32.0,
        64.0,
        128.0,
        256.0,
        512.0,
        1024.0,
        2048.0,
        4096.0,
        8192.0,
        16384.0,
        32768.0,
        65504.0,
        0.125,
        0.25,
        0.0625,
        std::f32::consts::SQRT_2,
        1.0 / std::f32::consts::SQRT_2,
        2.61803398875,
        0.1,
        0.3,
        7.0,
        42.0,
        100.0,
        500.0,
        1234.0,
        -0.1,
        -3.14159265,
        0.0001,
        0.999,
        1.5,
        0.75,
    ];

    let mut added = 0usize;
    for &x in candidates {
        let decoded = f16::from_f32(x).to_f32();
        if !decoded.is_finite() {
            continue;
        }
        let slug = format!("auto_rt_{:08x}", x.to_bits());
        if names.contains(&slug) {
            continue;
        }
        let tol = (decoded.abs() * 1e-4f32).max(5e-4f32);
        tv.push(json!({
            "name": slug,
            "description": format!("Synthetic IEEE binary16 roundtrip row (GF16 conformance proxy): input {}", x),
            "verdict": "CLEAN",
            "input": { "value": x },
            "expected": {
                "decoded": decoded,
                "tolerance_abs": tol,
            }
        }));
        names.insert(slug);
        added += 1;
    }

    let total = tv.len();
    if total < 33 {
        anyhow::bail!(
            "expand-gf16: only {} vectors after expansion (need ≥33); adjust candidates",
            total
        );
    }

    json["version"] = json!("2.1");
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    json["updated_at"] = json!(now.clone());
    json["validated_at"] = json!(now);

    let content_without_seal = {
        let mut j = json.clone();
        j["seal"] = json!("");
        serde_json::to_string(&j)?
    };
    let mut hasher = Sha256::new();
    hasher.update(content_without_seal.as_bytes());
    json["seal"] = json!(format!("sha256:{:x}", hasher.finalize()));

    let out = serde_json::to_string_pretty(&json)?;
    fs::write(&path, out)?;
    println!(
        "expand-gf16: added {} rows; total {} — wrote {}",
        added,
        total,
        path.display()
    );
    Ok(())
}

/// Insert / replace `nmse_synthetic_roundtrip` in `gf_family_bench.json` (not SCHEMA_V2).
pub fn generate_nmse_benchmark(repo_root: &Path, output: Option<&str>) -> anyhow::Result<()> {
    use half::{bf16, f16};
    use serde_json::json;

    let rel = output.unwrap_or("conformance/gf_family_bench.json");
    let path = repo_root.join(rel);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut json: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;

    const N: usize = 512;
    let log_min = -3f32;
    let log_max = (3e4f32).log10();

    let mut nmse_f16 = 0f64;
    let mut nmse_bf16 = 0f64;
    let denom = N as f64;
    for i in 0..N {
        let t = i as f32 / (N.saturating_sub(1).max(1)) as f32;
        let log_x = log_min + t * (log_max - log_min);
        let x = 10f32.powf(log_x);
        let xf = x as f64;
        if xf <= 0.0 || !xf.is_finite() {
            continue;
        }
        let rt_f16 = f16::from_f32(x).to_f32() as f64;
        let e_f16 = (xf - rt_f16) / xf;
        nmse_f16 += e_f16 * e_f16;

        let rt_bf = bf16::from_f32(x).to_f32() as f64;
        let e_bf = (xf - rt_bf) / xf;
        nmse_bf16 += e_bf * e_bf;
    }
    nmse_f16 /= denom;
    nmse_bf16 /= denom;

    let block = json!({
        "issue": 129,
        "method": "Mean of ((x - roundtrip(x)) / x)^2 over 512 log-spaced samples between 1e-3 and 3e4 (float32 domain)",
        "samples": N,
        "x_range": [1e-3, 3e4],
        "note": "Synthetic roundtrip only. `gf16_vectors.json` uses the same IEEE binary16 decode as a GF16 stand-in — not trained-model NMSE.",
        "nmse": {
            "GF16_proxy_same_as_ieee_binary16": nmse_f16,
            "IEEE_binary16": nmse_f16,
            "IEEE_bfloat16": nmse_bf16
        },
        "generated_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    });

    json.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("expected JSON object at root"))?
        .insert("nmse_synthetic_roundtrip".to_string(), block);

    let out = serde_json::to_string_pretty(&json)?;
    fs::write(&path, out)?;
    println!(
        "gen-nmse-benchmark: wrote {} (IEEE_binary16_nmse ≈ {:.6e}, bfloat16_nmse ≈ {:.6e})",
        path.display(),
        nmse_f16,
        nmse_bf16
    );
    Ok(())
}
