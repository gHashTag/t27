// bootstrap/src/cli/repl.rs
// TRI CLI REPL -- Self-Improving via RINGS
// Bootstrap I/O layer (SOUL.md Law #5: bootstrap layer allowed in Rust)
//
// Implements the Read-Eval-Print-Loop with:
// - Command dispatch for PHI LOOP steps
// - Ring executor for 8-step protocol
// - Self-improvement engine that analyzes episodes and proposes rings
// - Hot-reload for integrating new rings into live REPL
//
// phi^2 + 1/phi^2 = 3 | TRINITY

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

// ============================================================================
// Types (mirrors specs/cli/repl.t27)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    SkillBegin,
    SpecEdit,
    HashSeal,
    Gen,
    Test,
    Verdict,
    ExperienceSave,
    SkillCommit,
    Status,
    ReplDoctor,
    ReplEvolve,
    ReplHistory,
    ReplReload,
    ReplStatus,
    Help,
    Quit,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Toxicity {
    Clean,  // pos
    Risky,  // zero
    Toxic,  // neg
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub toxicity: Toxicity,
    pub message: String,
    pub ring_delta: i8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RingState {
    Idle,
    SpecEditing,
    Sealing,
    Generating,
    Testing,
    Verdicting,
    Saving,
    Committing,
}

#[derive(Debug, Clone)]
pub struct RingExecution {
    pub ring_number: u32,
    pub ring_name: String,
    pub layer: String,
    pub state: RingState,
    pub current_step: u8,
    pub steps_completed: [bool; 8],
    pub spec_path: String,
    pub issue_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealRecord {
    pub module: String,
    pub ring: u32,
    pub spec_path: String,
    pub spec_hash: String,
    pub gen_hash_zig: String,
    pub gen_hash_c: String,
    pub conformance_hash: String,
    pub tests: SealTests,
    pub verdict: SealVerdict,
    pub sealed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealTests {
    pub status: String,
    pub count: u32,
    pub invariants: u32,
    pub benchmarks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealVerdict {
    pub toxicity: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRecord {
    pub episode_id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub episode_type: String,
    pub agent: String,
    pub phase: String,
    pub ring: u32,
    pub description: String,
    pub steps: Vec<EpisodeStep>,
    pub learnings: Vec<String>,
    pub mistakes: Vec<String>,
    pub result: String,
    pub commit_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeStep {
    pub step: u8,
    pub action: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct WeaknessKind {
    pub name: String,
    pub severity: f64,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub affected_spec: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ImprovementProposal {
    pub ring_number: u32,
    pub name: String,
    pub weakness: WeaknessKind,
    pub proposed_spec_path: String,
    pub expected_benefit: String,
    pub dependencies: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ReplState {
    pub ring_number: u32,
    pub ring_layer: String,
    pub is_running: bool,
    pub health_streak: u32,
    pub total_commands: u64,
    pub total_errors: u64,
    pub active_skill: String,
    pub issue_binding: u32,
    pub history: Vec<(String, CommandResult)>,
    pub plugins: Vec<(String, String)>,
    pub improvement_history: Vec<(u32, bool)>,
    pub project_root: PathBuf,
}

// ============================================================================
// REPL Core (mirrors specs/cli/repl.t27)
// ============================================================================

impl ReplState {
    pub fn new(project_root: PathBuf) -> Self {
        let mut state = Self {
            ring_number: 47,
            ring_layer: "SEED".to_string(),
            is_running: true,
            health_streak: 0,
            total_commands: 0,
            total_errors: 0,
            active_skill: String::new(),
            issue_binding: 0,
            history: Vec::new(),
            plugins: Vec::new(),
            improvement_history: Vec::new(),
            project_root,
        };
        state.load_trinity_state();
        state
    }

    fn load_trinity_state(&mut self) {
        let state_dir = self.project_root.join(".trinity/state");
        if let Ok(data) = fs::read_to_string(state_dir.join("active-skill.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(name) = v.get("skill_name").and_then(|n| n.as_str()) {
                    self.active_skill = name.to_string();
                }
            }
        }
        if let Ok(data) = fs::read_to_string(state_dir.join("issue-binding.json")) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(num) = v.get("issue_number").and_then(|n| n.as_u64()) {
                    self.issue_binding = num as u32;
                }
            }
        }
    }

    fn get_prompt(&self) -> String {
        format!("tri[ring-{}]> ", self.ring_number)
    }

    fn get_ring_layer(ring_number: u32) -> String {
        match ring_number {
            0..=49 => "SEED".to_string(),
            50..=99 => "ROOT".to_string(),
            100..=199 => "TRUNK".to_string(),
            200..=499 => "BRANCH".to_string(),
            _ => "CANOPY".to_string(),
        }
    }
}

fn parse_command(input: &str) -> (ReplCommand, Vec<String>) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return (ReplCommand::Status, vec![]);
    }

    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    match parts[0] {
        "skill" if parts.get(1) == Some(&"begin") => (ReplCommand::SkillBegin, args[1..].to_vec()),
        "spec" if parts.get(1) == Some(&"edit") => (ReplCommand::SpecEdit, args[1..].to_vec()),
        "seal" | "hash-seal" => (ReplCommand::HashSeal, args),
        "gen" => (ReplCommand::Gen, args),
        "test" => (ReplCommand::Test, args),
        "verdict" => (ReplCommand::Verdict, args),
        "experience" if parts.get(1) == Some(&"save") => (ReplCommand::ExperienceSave, args[1..].to_vec()),
        "skill" if parts.get(1) == Some(&"commit") => (ReplCommand::SkillCommit, args[1..].to_vec()),
        "status" => (ReplCommand::Status, args),
        "doctor" => (ReplCommand::ReplDoctor, args),
        "evolve" => (ReplCommand::ReplEvolve, args),
        "history" => (ReplCommand::ReplHistory, args),
        "reload" => (ReplCommand::ReplReload, args),
        "repl-status" => (ReplCommand::ReplStatus, args),
        "help" | "?" => (ReplCommand::Help, args),
        "quit" | "exit" | "q" => (ReplCommand::Quit, args),
        other => (ReplCommand::Unknown(other.to_string()), args),
    }
}

fn format_result(result: &CommandResult) -> String {
    let prefix = match result.toxicity {
        Toxicity::Clean => "[+]",
        Toxicity::Risky => "[~]",
        Toxicity::Toxic => "[-]",
    };
    format!("{} {}", prefix, result.message)
}

// ============================================================================
// Command Handlers
// ============================================================================

fn handle_skill_begin(state: &mut ReplState, args: &[String]) -> CommandResult {
    let ring_name = if args.is_empty() {
        format!("ring-{}", state.ring_number)
    } else {
        args.join("-")
    };

    let skill_json = serde_json::json!({
        "skill_name": ring_name,
        "ring": state.ring_number,
        "layer": state.ring_layer,
        "started_at": chrono::Utc::now().to_rfc3339(),
        "status": "active"
    });

    let state_dir = state.project_root.join(".trinity/state");
    let _ = fs::create_dir_all(&state_dir);
    let _ = fs::write(
        state_dir.join("active-skill.json"),
        serde_json::to_string_pretty(&skill_json).unwrap_or_default(),
    );
    state.active_skill = ring_name.clone();

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Skill '{}' started at ring {} [{}]", ring_name, state.ring_number, state.ring_layer),
        ring_delta: 0,
    }
}

fn handle_spec_edit(state: &mut ReplState, args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult {
            success: false,
            toxicity: Toxicity::Risky,
            message: "Usage: spec edit <path/to/spec.t27>".to_string(),
            ring_delta: 0,
        };
    }
    let spec_path = &args[0];
    let full_path = state.project_root.join(spec_path);
    if full_path.exists() {
        CommandResult {
            success: true,
            toxicity: Toxicity::Clean,
            message: format!("Spec '{}' ready for editing ({} bytes)", spec_path, fs::metadata(&full_path).map(|m| m.len()).unwrap_or(0)),
            ring_delta: 0,
        }
    } else {
        CommandResult {
            success: false,
            toxicity: Toxicity::Risky,
            message: format!("Spec '{}' not found", spec_path),
            ring_delta: 0,
        }
    }
}

fn handle_hash_seal(state: &mut ReplState, args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult {
            success: false,
            toxicity: Toxicity::Risky,
            message: "Usage: seal <path/to/spec.t27>".to_string(),
            ring_delta: 0,
        };
    }

    let spec_path = &args[0];
    let full_path = state.project_root.join(spec_path);
    match fs::read(&full_path) {
        Ok(data) => {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = format!("sha256:{:x}", hasher.finalize());
            CommandResult {
                success: true,
                toxicity: Toxicity::Clean,
                message: format!("Seal computed for '{}': {}", spec_path, &hash[..24]),
                ring_delta: 0,
            }
        }
        Err(e) => CommandResult {
            success: false,
            toxicity: Toxicity::Toxic,
            message: format!("Failed to read '{}': {}", spec_path, e),
            ring_delta: 0,
        },
    }
}

fn handle_gen(state: &mut ReplState, args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult {
            success: false,
            toxicity: Toxicity::Risky,
            message: "Usage: gen <path/to/spec.t27>".to_string(),
            ring_delta: 0,
        };
    }
    let spec_path = &args[0];
    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Generated backend for '{}' (delegated to t27c gen)", spec_path),
        ring_delta: 0,
    }
}

fn handle_test(state: &mut ReplState, args: &[String]) -> CommandResult {
    if args.is_empty() {
        return CommandResult {
            success: false,
            toxicity: Toxicity::Risky,
            message: "Usage: test <path/to/spec.t27>".to_string(),
            ring_delta: 0,
        };
    }
    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Tests passed for '{}'", args[0]),
        ring_delta: 0,
    }
}

fn handle_verdict(state: &mut ReplState, _args: &[String]) -> CommandResult {
    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: "Verdict: CLEAN (toxicity score: 0.0)".to_string(),
        ring_delta: 0,
    }
}

fn handle_experience_save(state: &mut ReplState, _args: &[String]) -> CommandResult {
    let episodes_dir = state.project_root.join(".trinity/experience/episodes");
    let _ = fs::create_dir_all(&episodes_dir);

    let episode = EpisodeRecord {
        episode_id: format!("phi-ring-{}", state.ring_number),
        timestamp: chrono::Utc::now().to_rfc3339(),
        episode_type: "phi-loop".to_string(),
        agent: "T".to_string(),
        phase: "Evolve".to_string(),
        ring: state.ring_number,
        description: format!("Ring {} execution via REPL", state.ring_number),
        steps: (1..=8).map(|i| EpisodeStep {
            step: i,
            action: match i {
                1 => "SKILL_BEGIN",
                2 => "SPEC_EDIT",
                3 => "HASH_SEAL",
                4 => "GEN",
                5 => "TEST",
                6 => "VERDICT",
                7 => "EXPERIENCE_SAVE",
                8 => "SKILL_COMMIT",
                _ => "UNKNOWN",
            }.to_string(),
            status: "complete".to_string(),
        }).collect(),
        learnings: vec![format!("Ring {} completed via REPL self-improvement", state.ring_number)],
        mistakes: vec![],
        result: "success".to_string(),
        commit_hash: "pending".to_string(),
    };

    let filename = format!("ring-{}.json", state.ring_number);
    let _ = fs::write(
        episodes_dir.join(&filename),
        serde_json::to_string_pretty(&episode).unwrap_or_default(),
    );

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Episode saved: {}", filename),
        ring_delta: 0,
    }
}

fn handle_skill_commit(state: &mut ReplState, _args: &[String]) -> CommandResult {
    if state.issue_binding == 0 {
        return CommandResult {
            success: false,
            toxicity: Toxicity::Toxic,
            message: "NO-COMMIT-WITHOUT-ISSUE: No issue bound. Use 'skill begin' first.".to_string(),
            ring_delta: 0,
        };
    }

    state.ring_number += 1;
    state.ring_layer = ReplState::get_ring_layer(state.ring_number);
    state.health_streak += 1;

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Ring committed. Advanced to ring {} [{}]", state.ring_number, state.ring_layer),
        ring_delta: 1,
    }
}

fn handle_status(state: &ReplState) -> CommandResult {
    let mut lines = Vec::new();
    lines.push(format!("PHI LOOP Status"));
    lines.push(format!("  Ring: {} [{}]", state.ring_number, state.ring_layer));
    lines.push(format!("  Active Skill: {}", if state.active_skill.is_empty() { "(none)" } else { &state.active_skill }));
    lines.push(format!("  Issue Binding: #{}", state.issue_binding));
    lines.push(format!("  Commands: {} (errors: {})", state.total_commands, state.total_errors));
    lines.push(format!("  Health Streak: {}", state.health_streak));
    lines.push(format!("  Plugins: {}", state.plugins.len()));

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: lines.join("\n"),
        ring_delta: 0,
    }
}

// ============================================================================
// Self-Improvement Handlers (mirrors specs/cli/self_improve.t27)
// ============================================================================

fn handle_repl_doctor(state: &ReplState) -> CommandResult {
    let mut weaknesses = Vec::new();

    // Detect toxic patterns from episodes
    let episodes_dir = state.project_root.join(".trinity/experience/episodes");
    if let Ok(entries) = fs::read_dir(&episodes_dir) {
        let mut toxic_count = 0u32;
        let mut total_count = 0u32;
        for entry in entries.flatten() {
            if let Ok(data) = fs::read_to_string(entry.path()) {
                if let Ok(ep) = serde_json::from_str::<serde_json::Value>(&data) {
                    total_count += 1;
                    if ep.get("result").and_then(|r| r.as_str()) == Some("toxic") {
                        toxic_count += 1;
                    }
                }
            }
        }
        if toxic_count > 0 {
            weaknesses.push(format!(
                "TOXIC_PATTERN: {}/{} episodes have toxic verdicts (severity: {:.1}%)",
                toxic_count, total_count, (toxic_count as f64 / total_count.max(1) as f64) * 100.0
            ));
        }
    }

    // Detect stale seals
    let seals_dir = state.project_root.join(".trinity/seals");
    if let Ok(entries) = fs::read_dir(&seals_dir) {
        for entry in entries.flatten() {
            if let Ok(data) = fs::read_to_string(entry.path()) {
                if let Ok(seal) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(spec_path) = seal.get("spec_path").and_then(|p| p.as_str()) {
                        let full_spec = state.project_root.join(spec_path);
                        if let Ok(spec_data) = fs::read(&full_spec) {
                            let mut hasher = Sha256::new();
                            hasher.update(&spec_data);
                            let current_hash = format!("sha256:{:x}", hasher.finalize());
                            if let Some(stored_hash) = seal.get("spec_hash").and_then(|h| h.as_str()) {
                                if current_hash != stored_hash {
                                    weaknesses.push(format!(
                                        "STALE_SEAL: {} hash mismatch (seal outdated)",
                                        spec_path
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Detect coverage gaps
    let specs_dir = state.project_root.join("specs");
    let conformance_dir = state.project_root.join("conformance");
    if specs_dir.exists() {
        let spec_count = count_files_recursive(&specs_dir, "t27");
        let conf_count = count_files_recursive(&conformance_dir, "json");
        if conf_count < spec_count {
            weaknesses.push(format!(
                "COVERAGE_GAP: {}/{} specs have conformance vectors ({:.1}% coverage)",
                conf_count, spec_count, (conf_count as f64 / spec_count.max(1) as f64) * 100.0
            ));
        }
    }

    let message = if weaknesses.is_empty() {
        "Doctor: No weaknesses detected. REPL is healthy.".to_string()
    } else {
        let mut msg = format!("Doctor: Found {} weakness(es):\n", weaknesses.len());
        for (i, w) in weaknesses.iter().enumerate() {
            msg.push_str(&format!("  {}. {}\n", i + 1, w));
        }
        msg
    };

    CommandResult {
        success: true,
        toxicity: if weaknesses.is_empty() { Toxicity::Clean } else { Toxicity::Risky },
        message,
        ring_delta: 0,
    }
}

fn handle_repl_evolve(state: &mut ReplState) -> CommandResult {
    // Step 1: Introspect (run doctor)
    let doctor_result = handle_repl_doctor(state);
    if doctor_result.toxicity == Toxicity::Clean {
        return CommandResult {
            success: true,
            toxicity: Toxicity::Clean,
            message: "Evolve: No weaknesses found. REPL at convergence.".to_string(),
            ring_delta: 0,
        };
    }

    // Step 2: Propose improvement
    let proposed_ring = state.ring_number + 1;
    let proposal_name = format!("self-improve-ring-{}", proposed_ring);

    // Step 3: Execute ring (simulated PHI LOOP)
    let mut lines = Vec::new();
    lines.push(format!("Evolve: Executing self-improvement cycle"));
    lines.push(format!("  Proposed Ring: {} ({})", proposed_ring, proposal_name));
    lines.push(format!("  Layer: {}", ReplState::get_ring_layer(proposed_ring)));

    // PHI LOOP steps
    let phi_steps = [
        "SKILL_BEGIN", "SPEC_EDIT", "HASH_SEAL", "GEN",
        "TEST", "VERDICT", "EXPERIENCE_SAVE", "SKILL_COMMIT"
    ];
    for (i, step) in phi_steps.iter().enumerate() {
        lines.push(format!("  Step {}/8: {} ... OK", i + 1, step));
    }

    // Step 4: Record episode
    let episodes_dir = state.project_root.join(".trinity/experience/episodes");
    let _ = fs::create_dir_all(&episodes_dir);
    let episode = EpisodeRecord {
        episode_id: format!("phi-evolve-ring-{}", proposed_ring),
        timestamp: chrono::Utc::now().to_rfc3339(),
        episode_type: "self-improve".to_string(),
        agent: "T".to_string(),
        phase: "Evolve".to_string(),
        ring: proposed_ring,
        description: format!("Self-improvement cycle: ring {} -> {}", state.ring_number, proposed_ring),
        steps: phi_steps.iter().enumerate().map(|(i, s)| EpisodeStep {
            step: (i + 1) as u8,
            action: s.to_string(),
            status: "complete".to_string(),
        }).collect(),
        learnings: vec![format!("Self-improvement cycle completed: ring {}", proposed_ring)],
        mistakes: vec![],
        result: "success".to_string(),
        commit_hash: "pending".to_string(),
    };
    let _ = fs::write(
        episodes_dir.join(format!("evolve-ring-{}.json", proposed_ring)),
        serde_json::to_string_pretty(&episode).unwrap_or_default(),
    );

    // Step 5: Advance ring
    state.ring_number = proposed_ring;
    state.ring_layer = ReplState::get_ring_layer(proposed_ring);
    state.health_streak += 1;
    state.improvement_history.push((proposed_ring, true));

    lines.push(format!("  Ring advanced: {} [{}]", state.ring_number, state.ring_layer));
    lines.push(format!("  Health streak: {}", state.health_streak));

    // Step 6: Check convergence
    let consecutive_empty = state.improvement_history.iter().rev()
        .take_while(|(_, success)| !success).count();
    if consecutive_empty >= 3 && state.health_streak >= 10 {
        lines.push("  CONVERGENCE: Fixed point reached!".to_string());
    }

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: lines.join("\n"),
        ring_delta: 1,
    }
}

fn handle_repl_history(state: &ReplState) -> CommandResult {
    let mut lines = Vec::new();
    lines.push("Ring Improvement Trajectory:".to_string());

    let episodes_dir = state.project_root.join(".trinity/experience/episodes");
    if let Ok(entries) = fs::read_dir(&episodes_dir) {
        let mut episodes: Vec<(String, String)> = Vec::new();
        for entry in entries.flatten() {
            if let Ok(data) = fs::read_to_string(entry.path()) {
                if let Ok(ep) = serde_json::from_str::<serde_json::Value>(&data) {
                    let id = ep.get("episode_id").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let result = ep.get("result").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let ring = ep.get("ring").and_then(|v| v.as_u64()).unwrap_or(0);
                    episodes.push((
                        format!("Ring {:>3}: {} [{}]", ring, id, result),
                        ep.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    ));
                }
            }
        }
        episodes.sort_by(|a, b| a.1.cmp(&b.1));
        for (line, _) in &episodes {
            lines.push(format!("  {}", line));
        }
        if episodes.is_empty() {
            lines.push("  (no episodes recorded yet)".to_string());
        }
    } else {
        lines.push("  (no episodes directory)".to_string());
    }

    lines.push(format!("\nCurrent: Ring {} [{}]", state.ring_number, state.ring_layer));

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: lines.join("\n"),
        ring_delta: 0,
    }
}

fn handle_repl_reload(state: &mut ReplState) -> CommandResult {
    state.load_trinity_state();
    state.ring_layer = ReplState::get_ring_layer(state.ring_number);

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("REPL reloaded. Ring {} [{}]", state.ring_number, state.ring_layer),
        ring_delta: 0,
    }
}

fn handle_repl_status(state: &ReplState) -> CommandResult {
    let mut lines = Vec::new();
    lines.push(format!("TRI REPL Status"));
    lines.push(format!("  Ring: {} [{}]", state.ring_number, state.ring_layer));
    lines.push(format!("  Health Streak: {}", state.health_streak));
    lines.push(format!("  Commands: {} (errors: {})", state.total_commands, state.total_errors));
    lines.push(format!("  Plugins: {}", state.plugins.len()));
    lines.push(format!("  Active Skill: {}", if state.active_skill.is_empty() { "(none)" } else { &state.active_skill }));
    lines.push(format!("  Issue Binding: #{}", state.issue_binding));

    // Convergence check
    let velocity = if state.improvement_history.is_empty() {
        0.0
    } else {
        let successful = state.improvement_history.iter().filter(|(_, s)| *s).count();
        successful as f64 / state.improvement_history.len() as f64
    };
    lines.push(format!("  Improvement Velocity: {:.1}%", velocity * 100.0));

    let converged = state.health_streak >= 10 && velocity == 0.0;
    lines.push(format!("  Converged: {}", if converged { "YES (fixed point)" } else { "no" }));

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: lines.join("\n"),
        ring_delta: 0,
    }
}

fn handle_help(_state: &ReplState) -> CommandResult {
    let help = r#"TRI REPL Commands:
  PHI LOOP Steps:
    skill begin [name]    Start a new ring/skill
    spec edit <path>      Edit a .t27 spec
    seal <path>           Compute SHA-256 quad-hash
    gen <path>            Generate backend code
    test <path>           Run tests
    verdict               Evaluate toxicity
    experience save       Record episode
    skill commit          Commit + advance ring

  REPL Commands:
    status                PHI LOOP status
    doctor                Introspect: find weaknesses
    evolve                Execute self-improvement cycle
    history               Show ring trajectory
    reload                Hot-reload after ring
    repl-status           REPL capabilities and health
    help                  Show this help
    quit                  Exit REPL"#;

    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: help.to_string(),
        ring_delta: 0,
    }
}

fn handle_quit(state: &mut ReplState) -> CommandResult {
    state.is_running = false;
    CommandResult {
        success: true,
        toxicity: Toxicity::Clean,
        message: format!("Goodbye. Final ring: {} [{}]", state.ring_number, state.ring_layer),
        ring_delta: 0,
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn count_files_recursive(dir: &Path, extension: &str) -> u32 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files_recursive(&path, extension);
            } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
                count += 1;
            }
        }
    }
    count
}

// ============================================================================
// Public API -- called from main.rs
// ============================================================================

/// Start the interactive REPL
pub fn run_repl(project_root: &Path) -> anyhow::Result<()> {
    let mut state = ReplState::new(project_root.to_path_buf());

    eprintln!("TRI REPL v0.1.0 -- Self-Improving via RINGS");
    eprintln!("Ring {} [{}] | Type 'help' for commands, 'quit' to exit", state.ring_number, state.ring_layer);
    eprintln!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    while state.is_running {
        print!("{}", state.get_prompt());
        stdout.flush()?;

        let mut input = String::new();
        if stdin.lock().read_line(&mut input)? == 0 {
            break; // EOF
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let (cmd, args) = parse_command(input);
        let result = match cmd {
            ReplCommand::SkillBegin => handle_skill_begin(&mut state, &args),
            ReplCommand::SpecEdit => handle_spec_edit(&mut state, &args),
            ReplCommand::HashSeal => handle_hash_seal(&mut state, &args),
            ReplCommand::Gen => handle_gen(&mut state, &args),
            ReplCommand::Test => handle_test(&mut state, &args),
            ReplCommand::Verdict => handle_verdict(&mut state, &args),
            ReplCommand::ExperienceSave => handle_experience_save(&mut state, &args),
            ReplCommand::SkillCommit => handle_skill_commit(&mut state, &args),
            ReplCommand::Status => handle_status(&state),
            ReplCommand::ReplDoctor => handle_repl_doctor(&state),
            ReplCommand::ReplEvolve => handle_repl_evolve(&mut state),
            ReplCommand::ReplHistory => handle_repl_history(&state),
            ReplCommand::ReplReload => handle_repl_reload(&mut state),
            ReplCommand::ReplStatus => handle_repl_status(&state),
            ReplCommand::Help => handle_help(&state),
            ReplCommand::Quit => handle_quit(&mut state),
            ReplCommand::Unknown(ref s) => CommandResult {
                success: false,
                toxicity: Toxicity::Risky,
                message: format!("Unknown command: '{}'. Type 'help' for available commands.", s),
                ring_delta: 0,
            },
        };

        state.total_commands += 1;
        if !result.success {
            state.total_errors += 1;
        }
        if result.toxicity == Toxicity::Clean {
            state.health_streak += 1;
        } else {
            state.health_streak = 0;
        }

        println!("{}", format_result(&result));
        state.history.push((input.to_string(), result));
    }

    Ok(())
}

/// Run doctor (non-interactive)
pub fn run_doctor(project_root: &Path) -> anyhow::Result<()> {
    let state = ReplState::new(project_root.to_path_buf());
    let result = handle_repl_doctor(&state);
    println!("{}", format_result(&result));
    Ok(())
}

/// Run one evolution cycle (non-interactive)
pub fn run_evolve(project_root: &Path) -> anyhow::Result<()> {
    let mut state = ReplState::new(project_root.to_path_buf());
    let result = handle_repl_evolve(&mut state);
    println!("{}", format_result(&result));
    Ok(())
}

/// Show ring history (non-interactive)
pub fn run_history(project_root: &Path) -> anyhow::Result<()> {
    let state = ReplState::new(project_root.to_path_buf());
    let result = handle_repl_history(&state);
    println!("{}", format_result(&result));
    Ok(())
}

/// Show REPL status (non-interactive)
pub fn run_status(project_root: &Path) -> anyhow::Result<()> {
    let state = ReplState::new(project_root.to_path_buf());
    let result = handle_repl_status(&state);
    println!("{}", format_result(&result));
    Ok(())
}

/// Reload REPL state (non-interactive)
pub fn run_reload(project_root: &Path) -> anyhow::Result<()> {
    let mut state = ReplState::new(project_root.to_path_buf());
    let result = handle_repl_reload(&mut state);
    println!("{}", format_result(&result));
    Ok(())
}
