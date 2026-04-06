//! Repo tooling in Rust (NO-PYTHON policy): doc language gate, φ f64 validation.

use anyhow::Context;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

const DOC_DIRS: &[&str] = &["docs", "specs", "architecture", "clara-bridge", "conformance"];
const ROOT_MD: &[&str] = &[
    "README.md",
    "AGENTS.md",
    "CLAUDE.md",
    "NOW.md",
    "SOUL.md",
    "OWNERS.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CODE_OF_CONDUCT.md",
];

fn is_cyrillic(ch: char) -> bool {
    matches!(ch, '\u{0400}'..='\u{04FF}')
}

fn load_allow_list(repo_root: &Path) -> anyhow::Result<BTreeSet<String>> {
    let path = repo_root.join("docs/.legacy-non-english-docs");
    if !path.is_file() {
        return Ok(BTreeSet::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.is_empty() {
            out.insert(line.replace('\\', "/"));
        }
    }
    Ok(out)
}

fn md_has_cyrillic(path: &Path) -> anyhow::Result<bool> {
    let mut f = fs::File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf.chars().any(is_cyrillic))
}

/// Same rules as former `scripts/check_first_party_doc_language.py` / `build.rs` policy.
pub fn run_lint_docs(repo_root: &Path) -> anyhow::Result<()> {
    let allowed = load_allow_list(repo_root)?;
    let mut errors: Vec<String> = Vec::new();

    for d in DOC_DIRS {
        let base = repo_root.join(d);
        if !base.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&base).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            if p.extension().and_then(|x| x.to_str()) != Some("md") {
                continue;
            }
            let rel = p
                .strip_prefix(repo_root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            if allowed.contains(&rel.to_string()) {
                continue;
            }
            if md_has_cyrillic(p)? {
                errors.push(rel.to_string());
            }
        }
    }

    for name in ROOT_MD {
        let p = repo_root.join(name);
        if !p.is_file() {
            continue;
        }
        if allowed.contains(*name) {
            continue;
        }
        if md_has_cyrillic(&p)? {
            errors.push((*name).to_string());
        }
    }

    errors.sort();
    if errors.is_empty() {
        println!("lint-docs: OK (no Cyrillic in first-party Markdown outside allowlist)");
        return Ok(());
    }
    for rel in &errors {
        eprintln!(
            "ERROR: Cyrillic in first-party Markdown (not in docs/.legacy-non-english-docs): {}",
            rel
        );
    }
    anyhow::bail!("lint-docs: {} file(s) failed", errors.len());
}

fn f64_params(x: f64, name: &str) -> anyhow::Result<(u64, i32)> {
    let bits = x.to_bits();
    let exp_biased = ((bits >> 52) & 0x7FF) as i32;
    let mantissa_bits = bits & 0xF_FFFF_FFFF_FFFF;
    let mantissa_full = (1u64 << 52) | mantissa_bits;
    let exp_flocq = exp_biased - 1023 - 52;
    println!("--- {} ---", name);
    println!("  mantissa = {}  (Coq positive)", mantissa_full);
    println!("  exponent = {}  (Coq Z)", exp_flocq);
    println!("  raw_bits = 0x{:016X}", bits);
    Ok((mantissa_full, exp_flocq))
}

/// Former `scripts/validate_phi_f64.py` — Flocq / IEEE binary64 cross-check for φ literals.
pub fn run_validate_phi() -> anyhow::Result<()> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    f64_params(phi, "phi")?;
    f64_params(phi * phi, "phi_sq")?;
    f64_params(phi + 1.0, "phi_plus_one")?;

    let residual = (phi * phi - (phi + 1.0)).abs();
    let tolerance = 5.0 * 2.0_f64.powi(-53) * phi.powi(2);
    println!();
    println!("|phi^2 - (phi+1)| = {:.20e}", residual);
    println!("PHI_TOLERANCE     = {:.20e}", tolerance);
    println!("residual < tol    = {}", residual < tolerance);
    println!("phi_sq == phi_po  = {}", phi * phi == phi + 1.0);
    if residual >= tolerance {
        anyhow::bail!("validate-phi: residual >= PHI_TOLERANCE");
    }
    println!("validate-phi: OK");
    Ok(())
}

/// Validate L5 IDENTITY: φ² = φ + 1 using FORMAT-SPEC-001.json (issue #163)
pub fn validate_phi_identity(repo_root: &Path) -> anyhow::Result<()> {
    use serde_json::Value;

    let path = repo_root.join("conformance/FORMAT-SPEC-001.json");
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let json: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;

    let phi_identity = json.get("phi_identity")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("FORMAT-SPEC-001.json missing phi_identity section"))?;

    let tolerance = phi_identity.get("tolerance")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow::anyhow!("missing tolerance"))?;

    // Compute φ and verify identity
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let residual = (phi * phi - (phi + 1.0)).abs();
    let verdict = residual < tolerance;

    println!("=== L5 IDENTITY Validation ===");
    println!("φ:           {}", phi);
    println!("φ²:          {}", phi * phi);
    println!("φ + 1:       {}", phi + 1.0);
    println!("|φ² - (φ+1)|: {:.20e}", residual);
    println!("Tolerance:   {:.20e}", tolerance);
    println!("Verdict:     {}", if verdict { "PASS" } else { "FAIL" });
    println!("Ring proven: {}", phi_identity.get("ring_proven").and_then(|v| v.as_i64()).unwrap_or(0));

    if !verdict {
        anyhow::bail!("validate-phi-identity: FAILED - residual >= tolerance");
    }

    println!("validate-phi-identity: OK");
    Ok(())
}
