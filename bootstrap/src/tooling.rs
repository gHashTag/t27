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

/// Validate a JSON schema has Draft-07 structure (basic validation)
pub fn validate_schema(schema_path: &str) -> anyhow::Result<()> {
    use serde_json::Value;

    // Load schema
    let schema_raw = fs::read_to_string(schema_path)
        .with_context(|| format!("read {}", schema_path))?;
    let schema: Value = serde_json::from_str(&schema_raw)
        .with_context(|| format!("parse {}", schema_path))?;

    // Basic Draft-07 validation
    if let Some(s) = schema.get("$schema").and_then(|v| v.as_str()) {
        if !s.contains("draft-07") && !s.contains("draft/07") {
            eprintln!("Warning: schema does not declare Draft-07: {}", s);
        }
    } else {
        eprintln!("Warning: schema missing $schema declaration");
    }

    // Verify it's a valid object or boolean
    if !schema.is_object() && !schema.is_boolean() {
        anyhow::bail!("schema must be an object or boolean");
    }

    // If it's an object, check for common required fields
    if let Some(obj) = schema.as_object() {
        // Check for circular $ref issues (basic detection)
        if obj.contains_key("$ref") {
            let ref_val = obj.get("$ref").and_then(|v| v.as_str());
            if let Some(r) = ref_val {
                if !r.starts_with("#/") && !r.starts_with("http") && !r.starts_with("https") {
                    eprintln!("Warning: non-local reference: {}", r);
                }
            }
        }
    }

    println!("validate-schema: {} is valid JSON structure", schema_path);
    Ok(())
}

/// Validate a JSON instance against a schema
pub fn validate_instance(instance_path: &str, schema_path: &str) -> anyhow::Result<()> {
    use serde_json::Value;

    // Load schema
    let schema_raw = fs::read_to_string(schema_path)
        .with_context(|| format!("read {}", schema_path))?;
    let schema: Value = serde_json::from_str(&schema_raw)
        .with_context(|| format!("parse {}", schema_path))?;

    // Load instance
    let instance_raw = fs::read_to_string(instance_path)
        .with_context(|| format!("read {}", instance_path))?;
    let instance: Value = serde_json::from_str(&instance_raw)
        .with_context(|| format!("parse {}", instance_path))?;

    // Basic validation: check type if specified
    if let Some(obj) = schema.as_object() {
        if let Some(type_spec) = obj.get("type") {
            let type_match = match type_spec {
                Value::String(t) => match t.as_str() {
                    "object" => instance.is_object(),
                    "array" => instance.is_array(),
                    "string" => instance.is_string(),
                    "number" => instance.is_number(),
                    "integer" => instance.is_i64(),
                    "boolean" => instance.is_boolean(),
                    "null" => instance.is_null(),
                    _ => true,
                },
                Value::Array(types) => {
                    types.iter().any(|t| {
                        if let Some(t_str) = t.as_str() {
                            match t_str {
                                "object" => instance.is_object(),
                                "array" => instance.is_array(),
                                "string" => instance.is_string(),
                                "number" => instance.is_number(),
                                "integer" => instance.is_i64(),
                                "boolean" => instance.is_boolean(),
                                "null" => instance.is_null(),
                                _ => true,
                            }
                        } else {
                            false
                        }
                    })
                }
                _ => true,
            };

            if !type_match {
                eprintln!("validate-instance: type mismatch");
                eprintln!("  Expected type: {}", type_spec);
                eprintln!("  Instance type: {}", match instance {
                    Value::Object(_) => "object",
                    Value::Array(_) => "array",
                    Value::String(_) => "string",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Null => "null",
                });
                anyhow::bail!("instance type does not match schema");
            }
        }

        // Check required fields if object
        if instance.is_object() {
            if let Some(required) = obj.get("required").and_then(|v| v.as_array()) {
                if let Some(inst_obj) = instance.as_object() {
                    for req in required {
                        if let Some(req_str) = req.as_str() {
                            if !inst_obj.contains_key(req_str) {
                                anyhow::bail!("missing required field: {}", req_str);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("validate-instance: {} validates against {} (basic checks)", instance_path, schema_path);
    Ok(())
}

/// Check claim tiers consistency - validates schemas and reports valid tiers
pub fn check_claim_tiers(repo_root: &Path) -> anyhow::Result<()> {
    use serde_json::Value;

    println!("=== Claim Tiers Check ===");

    // Load EXPERIENCE_SCHEMA.json to get valid claim tiers for experience episodes
    let schema_path = repo_root.join("conformance/EXPERIENCE_SCHEMA.json");
    let schema_raw = fs::read_to_string(&schema_path)
        .with_context(|| format!("read {}", schema_path.display()))?;
    let schema: Value = serde_json::from_str(&schema_raw)
        .with_context(|| format!("parse {}", schema_path.display()))?;

    let claim_tier_enum = schema.get("properties")
        .and_then(|p| p.get("claim_tier"))
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("enum"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("EXPERIENCE_SCHEMA missing claim_tier enum"))?;

    let mut valid_tiers = Vec::new();
    for item in claim_tier_enum {
        if let Some(s) = item.as_str() {
            valid_tiers.push(s.to_string());
        }
    }

    println!("\nValid claim_tier values (for experience episodes):");
    for tier in &valid_tiers {
        println!("  - {}", tier);
    }

    // Extract status values used in RESEARCH_CLAIMS.md
    let claims_path = repo_root.join("docs/nona-03-manifest/RESEARCH_CLAIMS.md");
    let claims_raw = fs::read_to_string(&claims_path)
        .with_context(|| format!("read {}", claims_path.display()))?;

    let mut research_statuses = std::collections::BTreeSet::new();

    // Look for status values in claims document (backtick-wrapped status values)
    for line in claims_raw.lines() {
        if line.contains("`") {
            // Extract backtick-wrapped words
            let start = line.find('`');
            let end = line.rfind('`');
            if let (Some(s), Some(e)) = (start, end) {
                if s < e {
                    let status = &line[s+1..e];
                    // Check if it looks like a status (uppercase or underscore)
                    if status.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()) {
                        research_statuses.insert(status.to_string());
                    }
                }
            }
        }
    }

    println!("\nStatus values used in RESEARCH_CLAIMS.md:");
    for status in &research_statuses {
        println!("  - {}", status);
    }

    // Note: These are different tier systems for different purposes
    println!("\nNote: EXPERIENCE_SCHEMA.claim_tier and RESEARCH_CLAIMS.md Status use different systems.");
    println!("  - claim_tier: confidence level for experience episodes (PROVEN, TESTED, etc.)");
    println!("  - Status: verification status for research claims (EXACT, EMPIRICAL_FIT, etc.)");

    println!("\ncheck-claim-tiers: OK");
    Ok(())
}

/// Run experience aggregation script to refresh brain seals (Ring 059)
pub fn brain_seal_refresh(repo_root: &Path) -> anyhow::Result<()> {
    use std::process::Command;

    let script_path = repo_root.join("scripts/aggregate-experience.sh");

    if !script_path.exists() {
        anyhow::bail!("experience aggregation script not found: {}", script_path.display());
    }

    println!("=== Running brain seal refresh ===");
    println!("Script: {}", script_path.display());

    let output = Command::new("bash")
        .arg(&script_path)
        .arg(repo_root)
        .output()
        .context("execute aggregate-experience.sh")?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("aggregate-experience.sh failed: {}", stderr);
    }

    println!("=== Brain seal refresh complete ===");
    Ok(())
}
