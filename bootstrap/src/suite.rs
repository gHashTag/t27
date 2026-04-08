//! Repository-wide test orchestration (replaces legacy `tests/*.sh` runners).
//! Invoked as `t27c suite` from the repository root (or `tri test`).

use anyhow::Context;
use chrono::Local;
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

/// Phases 1–6: same coverage as legacy `tests/run_all.sh`.
pub fn run_comprehensive(repo_root: &Path) -> anyhow::Result<()> {
    let repo = fs::canonicalize(repo_root)
        .with_context(|| format!("cannot canonicalize repo root {}", repo_root.display()))?;

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
    let total_fail = p1f + p1bf + gf16_fail + p2f + p2bf + p3f + p4f + p5f + fp_diff;
    println!("Parse failures:    {}", p1f);
    println!("Typecheck fails:   {}", p1bf);
    println!("GF16 conformance:  {}", gf16_fail);
    println!("Gen Zig failures:  {}", p2f);
    println!("Gen Rust failures: {}", p2bf);
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
