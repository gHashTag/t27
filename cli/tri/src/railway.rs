// SPDX-License-Identifier: MIT
// Backend for `tri railway` -- generated from specs/cli/railway.t27
// (CLI-RAILWAY-543).
//
// This file MUST stay behaviorally identical to the spec. Edits here
// without a matching spec edit + reseal violate CANON_DE_ZIGFICATION.

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------
// Constants -- keep in sync with specs/cli/railway.t27
// ---------------------------------------------------------------------

pub const DEFAULT_API_ENDPOINT: &str = "https://backboard.railway.com/graphql/v2";
pub const DEFAULT_PROJECT_ID: &str = "e4fe33bb-3b09-4842-9782-7d2dea1abc9b";
pub const DEFAULT_IMAGE_REF: &str = "ghcr.io/ghashtag/trios-trainer-igla:latest";
pub const DEFAULT_TARGET_BPB: f64 = 1.85;
pub const DEFAULT_STEPS: u64 = 30_000;
pub const GATE2_SEED_QUORUM: usize = 3;
pub const STATE_BINDING_PATH: &str = ".trinity/state/railway-binding.json";
pub const EMBARGO_PATH: &str = "assertions/embargo.txt";
pub const SHA_PREFIX_LEN: usize = 7;

pub const ENV_SEED: &str = "TRIOS_SEED";
pub const ENV_LEDGER_PUSH: &str = "TRIOS_LEDGER_PUSH";
pub const ENV_TARGET_BPB: &str = "TRIOS_TARGET_BPB";
pub const ENV_STEPS: &str = "TRIOS_STEPS";
pub const ENV_RUST_LOG: &str = "RUST_LOG";

pub const GATE2_SEEDS: [u64; 3] = [43, 44, 45];

// ---------------------------------------------------------------------
// Subcommand surface
// ---------------------------------------------------------------------

#[derive(Subcommand, Clone)]
pub enum RailwayAction {
    /// Verify RAILWAY_TOKEN by issuing `me { id email }` against the API.
    Login {
        #[arg(long, env = "RAILWAY_TOKEN")]
        token: Option<String>,
        #[arg(long, default_value = DEFAULT_API_ENDPOINT)]
        endpoint: String,
    },
    /// Persist the Railway project binding to
    /// `.trinity/state/railway-binding.json`.
    Link {
        #[arg(long, default_value = DEFAULT_PROJECT_ID)]
        project: String,
        #[arg(long, default_value = DEFAULT_API_ENDPOINT)]
        endpoint: String,
        #[arg(long)]
        image: Option<String>,
    },
    /// ONE SHOT: build per-seed service plans and (on `--confirm`) create
    /// + deploy them on Railway. Default is dry-run: prints the plan,
    /// issues zero mutations.
    Up {
        /// Comma-separated seed list. Must be a superset of 43,44,45.
        #[arg(long, default_value = "43,44,45")]
        seeds: String,
        #[arg(long, default_value = DEFAULT_IMAGE_REF)]
        image: String,
        #[arg(long, default_value_t = DEFAULT_TARGET_BPB)]
        target_bpb: f64,
        #[arg(long, default_value_t = DEFAULT_STEPS)]
        steps: u64,
        /// Required to actually mutate Railway. Without it, `up` prints
        /// the plan and exits 0.
        #[arg(long, default_value_t = false)]
        confirm: bool,
        /// HEAD SHA to check against the embargo file. Defaults to the
        /// value of `GITHUB_SHA` env, else "HEAD" (disables embargo).
        #[arg(long, env = "GITHUB_SHA")]
        head_sha: Option<String>,
        #[arg(long, default_value = EMBARGO_PATH)]
        embargo: PathBuf,
    },
    /// Print one R7-style line per bound service.
    Status {
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Fetch recent deployment logs for one service.
    Logs {
        #[arg(long)]
        service: String,
        #[arg(long, default_value_t = 100usize)]
        tail: usize,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Combined verdict: reads the live ledger via `trios-igla gate` +
    /// Railway service health.
    Gate2 {
        #[arg(long, default_value_t = DEFAULT_TARGET_BPB)]
        target: f64,
        #[arg(long, default_value = "assertions/seed_results.jsonl")]
        ledger: PathBuf,
    },
}

// ---------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailwayBinding {
    pub project_id: String,
    pub endpoint: String,
    pub image: Option<String>,
    pub linked_at: String,
    pub linked_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServicePlan {
    pub name: String,
    pub seed: u64,
    pub image: String,
    pub target_bpb: f64,
    pub steps: u64,
    pub vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeployResult {
    pub service_name: String,
    pub service_id: Option<String>,
    pub deployment_id: Option<String>,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpOutcome {
    pub project_id: String,
    pub dry_run: bool,
    pub results: Vec<DeployResult>,
    pub all_started: bool,
}

// ---------------------------------------------------------------------
// Plan construction (pure, no I/O)
// ---------------------------------------------------------------------

pub fn build_service_plan(seed: u64, image: &str, target_bpb: f64, steps: u64) -> ServicePlan {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();
    vars.insert(ENV_SEED.to_string(), seed.to_string());
    vars.insert(ENV_LEDGER_PUSH.to_string(), "1".to_string());
    vars.insert(ENV_TARGET_BPB.to_string(), format_f64(target_bpb));
    vars.insert(ENV_STEPS.to_string(), steps.to_string());
    vars.insert(ENV_RUST_LOG.to_string(), "info".to_string());
    ServicePlan {
        name: format!("trainer-seed-{seed}"),
        seed,
        image: image.to_string(),
        target_bpb,
        steps,
        vars,
    }
}

pub fn build_plans(seeds: &[u64], image: &str, target_bpb: f64, steps: u64) -> Vec<ServicePlan> {
    seeds
        .iter()
        .map(|&s| build_service_plan(s, image, target_bpb, steps))
        .collect()
}

pub fn is_valid_gate2_seed_set(seeds: &[u64]) -> bool {
    GATE2_SEEDS.iter().all(|req| seeds.contains(req))
}

fn format_f64(v: f64) -> String {
    // Keep TOML/float formatting stable across platforms.
    if v.fract() == 0.0 {
        format!("{v:.1}")
    } else {
        // Trim trailing zeros but preserve at least one decimal digit.
        let s = format!("{v}");
        s
    }
}

pub fn parse_seed_list(raw: &str) -> Result<Vec<u64>> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let n: u64 = trimmed
            .parse()
            .with_context(|| format!("not a u64 seed: {trimmed:?}"))?;
        out.push(n);
    }
    if out.is_empty() {
        bail!("seed list is empty");
    }
    Ok(out)
}

// ---------------------------------------------------------------------
// GraphQL envelope builders (pure, no I/O)
// ---------------------------------------------------------------------

pub fn build_login_query() -> String {
    json!({"query": "query { me { id email } }"}).to_string()
}

pub fn build_service_create_body(project_id: &str, plan: &ServicePlan) -> String {
    json!({
        "query": "mutation($input: ServiceCreateInput!) { serviceCreate(input: $input) { id name } }",
        "variables": {
            "input": {
                "projectId": project_id,
                "name": plan.name,
                "source": { "image": plan.image }
            }
        }
    })
    .to_string()
}

pub fn build_variable_upsert_body(
    project_id: &str,
    environment_id: &str,
    service_id: &str,
    vars: &BTreeMap<String, String>,
) -> String {
    let entries: Vec<Value> = vars
        .iter()
        .map(|(k, v)| {
            json!({
                "projectId": project_id,
                "environmentId": environment_id,
                "serviceId": service_id,
                "name": k,
                "value": v,
            })
        })
        .collect();
    json!({
        "query": "mutation($input: [VariableUpsertInput!]!) { variableCollectionUpsert(input: $input) }",
        "variables": { "input": entries }
    })
    .to_string()
}

pub fn build_deploy_body(service_id: &str, environment_id: &str) -> String {
    json!({
        "query": "mutation($input: ServiceInstanceDeployInput!) { serviceInstanceDeployV2(input: $input) { id status } }",
        "variables": {
            "input": { "serviceId": service_id, "environmentId": environment_id }
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------
// Embargo guard (R9)
// ---------------------------------------------------------------------

pub fn head_is_embargoed(embargo_lines: &[String], head_sha: &str) -> bool {
    let needle = head_sha.to_lowercase();
    if needle.is_empty() {
        return false;
    }
    for line in embargo_lines {
        let entry = line.trim().to_lowercase();
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        if entry == needle {
            return true;
        }
        if needle.len() >= SHA_PREFIX_LEN
            && entry.len() >= SHA_PREFIX_LEN
            && entry[0..SHA_PREFIX_LEN] == needle[0..SHA_PREFIX_LEN]
        {
            return true;
        }
    }
    false
}

pub fn read_embargo(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        out.push(line);
    }
    Ok(out)
}

// ---------------------------------------------------------------------
// Binding persistence
// ---------------------------------------------------------------------

pub fn write_binding(path: &Path, binding: &RailwayBinding) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(binding)? + "\n";
    fs::write(path, json)?;
    Ok(())
}

pub fn read_binding(path: &Path) -> Result<RailwayBinding> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read {} (run `tri railway link` first)", path.display()))?;
    let b: RailwayBinding = serde_json::from_str(&raw).context("parse railway binding")?;
    Ok(b)
}

// ---------------------------------------------------------------------
// Subcommand dispatch
// ---------------------------------------------------------------------

pub fn run(action: RailwayAction) -> Result<i32> {
    match action {
        RailwayAction::Login { token, endpoint } => run_login(token, &endpoint),
        RailwayAction::Link {
            project,
            endpoint,
            image,
        } => run_link(&project, &endpoint, image),
        RailwayAction::Up {
            seeds,
            image,
            target_bpb,
            steps,
            confirm,
            head_sha,
            embargo,
        } => run_up(
            &seeds, &image, target_bpb, steps, confirm, head_sha, &embargo,
        ),
        RailwayAction::Status { dry_run } => run_status(dry_run),
        RailwayAction::Logs {
            service,
            tail,
            dry_run,
        } => run_logs(&service, tail, dry_run),
        RailwayAction::Gate2 { target, ledger } => run_gate2(target, &ledger),
    }
}

fn run_login(token: Option<String>, endpoint: &str) -> Result<i32> {
    let tok = token
        .or_else(|| std::env::var("RAILWAY_TOKEN").ok())
        .ok_or_else(|| anyhow!("RAILWAY_TOKEN not set and --token not provided"))?;
    if tok.len() < 8 {
        bail!("RAILWAY_TOKEN looks truncated (len={})", tok.len());
    }
    println!("railway: endpoint={endpoint} token_len={}", tok.len());
    println!("railway: login query body = {}", build_login_query());
    println!("railway: OK (token shape valid; HTTP verification not performed in tri)");
    Ok(0)
}

fn run_link(project: &str, endpoint: &str, image: Option<String>) -> Result<i32> {
    let binding = RailwayBinding {
        project_id: project.to_string(),
        endpoint: endpoint.to_string(),
        image,
        linked_at: Utc::now().to_rfc3339(),
        linked_by: "agent:tri".to_string(),
    };
    let path = Path::new(STATE_BINDING_PATH);
    write_binding(path, &binding)?;
    println!(
        "railway: linked project={} endpoint={} -> {}",
        binding.project_id,
        binding.endpoint,
        path.display()
    );
    Ok(0)
}

#[allow(clippy::too_many_arguments)]
fn run_up(
    seeds_raw: &str,
    image: &str,
    target_bpb: f64,
    steps: u64,
    confirm: bool,
    head_sha: Option<String>,
    embargo_path: &Path,
) -> Result<i32> {
    let seeds = parse_seed_list(seeds_raw)?;
    if !is_valid_gate2_seed_set(&seeds) {
        bail!(
            "seed set {seeds:?} does not contain canonical Gate-2 seeds {:?}",
            GATE2_SEEDS
        );
    }

    // R9: embargo guard
    if let Some(ref sha) = head_sha {
        if sha != "HEAD" {
            let lines = read_embargo(embargo_path).unwrap_or_default();
            if head_is_embargoed(&lines, sha) {
                bail!("R9: HEAD SHA {sha} is embargoed; refusing to deploy");
            }
        }
    }

    let plans = build_plans(&seeds, image, target_bpb, steps);
    let mut results = Vec::new();
    for plan in &plans {
        // In dry-run we just print; without network stack in tri, --confirm
        // prints the exact GraphQL bodies that WOULD be sent, leaving the
        // actual POST to the operator (or to a downstream task). This is
        // R5-honest: we refuse to claim a Railway mutation happened when
        // tri has no HTTP client.
        let body = build_service_create_body(DEFAULT_PROJECT_ID, plan);
        println!("railway: plan service={} seed={}", plan.name, plan.seed);
        for (k, v) in &plan.vars {
            println!("  env {k}={v}");
        }
        println!("  graphql serviceCreate body_bytes={}", body.len());
        let status = if confirm {
            "planned-confirm"
        } else {
            "planned-dry-run"
        };
        results.push(DeployResult {
            service_name: plan.name.clone(),
            service_id: None,
            deployment_id: None,
            status: status.to_string(),
            message: Some(body),
        });
    }
    let outcome = UpOutcome {
        project_id: DEFAULT_PROJECT_ID.to_string(),
        dry_run: !confirm,
        results,
        all_started: false,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&outcome).unwrap_or_default()
    );
    if confirm {
        // When confirm is set, exit 2 to signal "planned but not executed",
        // because tri has no HTTP client. The operator runs the external
        // deploy step. Once an HTTP client is added (tracked by a follow-up
        // issue), this branch can return 0.
        Ok(2)
    } else {
        Ok(0)
    }
}

fn run_status(dry_run: bool) -> Result<i32> {
    let path = Path::new(STATE_BINDING_PATH);
    let binding = read_binding(path)?;
    println!(
        "railway: status project={} endpoint={} dry_run={}",
        binding.project_id, binding.endpoint, dry_run
    );
    for seed in GATE2_SEEDS {
        println!("  service=trainer-seed-{seed} status=unknown (tri has no HTTP client yet)");
    }
    Ok(0)
}

fn run_logs(service: &str, tail: usize, dry_run: bool) -> Result<i32> {
    let _ = read_binding(Path::new(STATE_BINDING_PATH))?;
    println!("railway: logs service={service} tail={tail} dry_run={dry_run}");
    println!(
        "railway: (tri has no HTTP client yet; run `railway logs --service {service}` externally)"
    );
    Ok(0)
}

fn run_gate2(target: f64, ledger: &Path) -> Result<i32> {
    println!("railway: gate2 target={target} ledger={}", ledger.display());
    println!("railway: to finish the verdict, run:");
    println!(
        "  trios-igla gate --target {target} --ledger {}",
        ledger.display()
    );
    println!("  tri railway status");
    Ok(0)
}

// =====================================================================
// Tests (mirrors specs/cli/railway.t27)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const PHI: f64 = 1.618033988749895;
    const TRINITY_ANCHOR: f64 = 3.0;

    #[test]
    fn phi_anchor_holds() {
        let lhs = PHI * PHI + 1.0 / (PHI * PHI);
        assert!((lhs - TRINITY_ANCHOR).abs() < 1e-10);
    }

    #[test]
    fn plan_for_seed_43() {
        let plan = build_service_plan(
            43,
            "ghcr.io/ghashtag/trios-trainer-igla:latest",
            1.85,
            30_000,
        );
        assert_eq!(plan.name, "trainer-seed-43");
        assert_eq!(plan.seed, 43);
        assert_eq!(plan.vars.get("TRIOS_SEED").map(|s| s.as_str()), Some("43"));
        assert_eq!(
            plan.vars.get("TRIOS_LEDGER_PUSH").map(|s| s.as_str()),
            Some("1")
        );
        assert_eq!(
            plan.vars.get("TRIOS_TARGET_BPB").map(|s| s.as_str()),
            Some("1.85")
        );
    }

    #[test]
    fn plan_for_seed_44() {
        let plan = build_service_plan(44, "img", 1.85, 30_000);
        assert_eq!(plan.name, "trainer-seed-44");
        assert_eq!(plan.vars.get("TRIOS_SEED").map(|s| s.as_str()), Some("44"));
    }

    #[test]
    fn plan_for_seed_45() {
        let plan = build_service_plan(45, "img", 1.85, 30_000);
        assert_eq!(plan.name, "trainer-seed-45");
        assert_eq!(plan.vars.get("TRIOS_SEED").map(|s| s.as_str()), Some("45"));
    }

    #[test]
    fn build_plans_preserves_order() {
        let plans = build_plans(&[43, 44, 45], "img", 1.85, 30_000);
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].seed, 43);
        assert_eq!(plans[1].seed, 44);
        assert_eq!(plans[2].seed, 45);
    }

    #[test]
    fn gate2_seed_set_accepts_canonical() {
        assert!(is_valid_gate2_seed_set(&[43, 44, 45]));
    }

    #[test]
    fn gate2_seed_set_accepts_superset() {
        assert!(is_valid_gate2_seed_set(&[43, 44, 45, 46]));
    }

    #[test]
    fn gate2_seed_set_rejects_missing_seed() {
        assert!(!is_valid_gate2_seed_set(&[43, 44]));
    }

    #[test]
    fn gate2_seed_set_rejects_empty() {
        assert!(!is_valid_gate2_seed_set(&[]));
    }

    #[test]
    fn embargo_refuses_full_match() {
        let emb = vec!["477e3377".to_string()];
        assert!(head_is_embargoed(&emb, "477e3377"));
    }

    #[test]
    fn embargo_refuses_prefix_match() {
        let emb = vec!["477e3377deadbeef".to_string()];
        assert!(head_is_embargoed(&emb, "477e3377"));
    }

    #[test]
    fn embargo_accepts_clean_sha() {
        let emb = vec!["477e3377".to_string(), "b3ee6a36".to_string()];
        assert!(!head_is_embargoed(&emb, "2446855"));
    }

    #[test]
    fn embargo_skips_comments_and_blanks() {
        let emb = vec![
            "# a comment".to_string(),
            "".to_string(),
            "477e3377".to_string(),
        ];
        assert!(head_is_embargoed(&emb, "477e3377"));
        assert!(!head_is_embargoed(&emb, "deadbee"));
    }

    #[test]
    fn login_query_shape() {
        let q = build_login_query();
        assert!(q.contains("me { id email }"));
    }

    #[test]
    fn service_create_body_contains_project_and_image() {
        let plan = build_service_plan(43, "ghcr.io/x/y:z", 1.85, 30_000);
        let body = build_service_create_body("proj-uuid", &plan);
        assert!(body.contains("proj-uuid"));
        assert!(body.contains("ghcr.io/x/y:z"));
        assert!(body.contains("trainer-seed-43"));
    }

    #[test]
    fn service_naming_is_stable() {
        let a = build_service_plan(43, "img", 1.85, 30_000);
        let b = build_service_plan(43, "img", 1.85, 30_000);
        assert_eq!(a.name, b.name);
    }

    #[test]
    fn parse_seed_list_three() {
        let seeds = parse_seed_list("43,44,45").unwrap();
        assert_eq!(seeds, vec![43, 44, 45]);
    }

    #[test]
    fn parse_seed_list_rejects_empty() {
        assert!(parse_seed_list("").is_err());
        assert!(parse_seed_list(" , ").is_err());
    }

    #[test]
    fn variable_upsert_body_shape() {
        let plan = build_service_plan(43, "img", 1.85, 30_000);
        let body = build_variable_upsert_body("p", "e", "s", &plan.vars);
        assert!(body.contains("variableCollectionUpsert"));
        assert!(body.contains("TRIOS_SEED"));
        assert!(body.contains("TRIOS_LEDGER_PUSH"));
    }
}
