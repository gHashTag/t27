use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveSkill {
    skill_id: String,
    session_id: String,
    issue_id: String,
    description: String,
    started_at: String,
    started_by: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CellEntry {
    cell_id: String,
    skill_id: String,
    created_at: String,
    steps: Vec<CheckpointStep>,
    seal: Option<CellSeal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckpointStep {
    step: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CellSeal {
    hash: String,
    sealed_at: String,
    step_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AkashicEvent {
    ts: String,
    event: String,
    agent_id: String,
    trace_id: String,
    task_id: Option<String>,
    spec_path: Option<String>,
    result: String,
    error: Option<String>,
    metadata: Value,
}

fn find_repo_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    for _ in 0..8 {
        if dir.join(".trinity").is_dir() {
            return Some(dir.to_path_buf());
        }
        if dir.join("specs").is_dir() && dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
    None
}

fn trinity_path(root: &Path, segments: &[&str]) -> PathBuf {
    let mut p = root.join(".trinity");
    for s in segments {
        p = p.join(s);
    }
    p
}

fn read_json_file(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn write_json_file(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let formatted = serde_json::to_string_pretty(value)?;
    fs::write(path, formatted)?;
    Ok(())
}

fn append_akashic(root: &Path, event: &AkashicEvent) -> Result<()> {
    let path = trinity_path(root, &["events", "akashic-log.jsonl"]);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

fn make_akashic_event(event_type: &str, result: &str, metadata: Value) -> AkashicEvent {
    AkashicEvent {
        ts: Utc::now().to_rfc3339(),
        event: event_type.to_string(),
        agent_id: "tri-mcp".to_string(),
        trace_id: Uuid::new_v4().to_string(),
        task_id: None,
        spec_path: None,
        result: result.to_string(),
        error: None,
        metadata,
    }
}

fn tool_result_text(text: &str) -> Value {
    serde_json::json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    })
}

fn tool_error_text(text: &str) -> Value {
    serde_json::json!({
        "content": [{"type": "text", "text": text}],
        "isError": true
    })
}

fn handle_initialize() -> Result<Value> {
    Ok(serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": { "listChanged": false }
        },
        "serverInfo": {
            "name": "tri-mcp",
            "version": "0.1.0"
        }
    }))
}

fn build_tools_list() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "tri_status",
            "description": "Returns PHI LOOP status including active skill, cell, issue binding, and uncommitted changes.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "tri_skill_begin",
            "description": "Begins a new PHI LOOP skill session bound to an issue.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "issue": {
                        "type": "integer",
                        "description": "Issue number to bind the skill to"
                    },
                    "description": {
                        "type": "string",
                        "description": "Human-readable description of the skill session"
                    }
                },
                "required": ["issue", "description"]
            }
        }),
        serde_json::json!({
            "name": "tri_skill_end",
            "description": "Ends the active PHI LOOP skill session.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "tri_cell_checkpoint",
            "description": "Records a checkpoint step in the active cell.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "step": {
                        "type": "string",
                        "description": "Description of the checkpoint step"
                    }
                },
                "required": ["step"]
            }
        }),
        serde_json::json!({
            "name": "tri_cell_seal",
            "description": "Seals the active cell by computing a SHA-256 hash of its checkpoint history.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "tri_gen",
            "description": "Generates backends from a .t27 spec file using t27c.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "spec_path": {
                        "type": "string",
                        "description": "Path to the .t27 spec file"
                    }
                },
                "required": ["spec_path"]
            }
        }),
        serde_json::json!({
            "name": "tri_test",
            "description": "Runs tests for a .t27 spec file using t27c.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "spec_path": {
                        "type": "string",
                        "description": "Path to the .t27 spec file"
                    }
                },
                "required": ["spec_path"]
            }
        }),
        serde_json::json!({
            "name": "tri_verdict",
            "description": "Checks for toxic regressions by analyzing swarm health metrics.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "tri_experience_save",
            "description": "Saves an experience episode to the akashic log.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "episode_type": {
                        "type": "string",
                        "description": "Type of experience episode (e.g. success, failure, learning)"
                    },
                    "summary": {
                        "type": "string",
                        "description": "Summary of the experience"
                    },
                    "ring": {
                        "type": "integer",
                        "description": "Ring number associated with this experience"
                    }
                },
                "required": ["episode_type", "summary"]
            }
        }),
        serde_json::json!({
            "name": "tri_health",
            "description": "Returns queen and swarm health status from .trinity state.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
    ]
}

fn handle_tools_list() -> Result<Value> {
    Ok(serde_json::json!({
        "tools": build_tools_list()
    }))
}

fn handle_tools_call(request: Value, root: &Path) -> Result<Value> {
    let params = request
        .get("params")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let result = match name {
        "tri_status" => cmd_status(root),
        "tri_skill_begin" => cmd_skill_begin(root, &arguments),
        "tri_skill_end" => cmd_skill_end(root),
        "tri_cell_checkpoint" => cmd_cell_checkpoint(root, &arguments),
        "tri_cell_seal" => cmd_cell_seal(root),
        "tri_gen" => cmd_gen(root, &arguments),
        "tri_test" => cmd_test(root, &arguments),
        "tri_verdict" => cmd_verdict(root),
        "tri_experience_save" => cmd_experience_save(root, &arguments),
        "tri_health" => cmd_health(root),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    };

    match result {
        Ok(v) => Ok(v),
        Err(e) => Ok(tool_error_text(&e.to_string())),
    }
}

fn read_active_skill(root: &Path) -> Option<ActiveSkill> {
    let path = trinity_path(root, &["state", "active-skill.json"]);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_active_skill(root: &Path, skill: &ActiveSkill) -> Result<()> {
    let path = trinity_path(root, &["state", "active-skill.json"]);
    write_json_file(&path, &serde_json::to_value(skill)?)
}

fn read_cells_registry(root: &Path) -> Vec<CellEntry> {
    let path = trinity_path(root, &["cells", "registry.json"]);
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_cells_registry(root: &Path, cells: &[CellEntry]) -> Result<()> {
    let path = trinity_path(root, &["cells", "registry.json"]);
    write_json_file(&path, &serde_json::to_value(cells)?)
}

fn find_active_cell(cells: &[CellEntry], skill_id: &str) -> Option<usize> {
    cells
        .iter()
        .position(|c| c.skill_id == skill_id && c.seal.is_none())
}

fn git_status_short(root: &Path) -> String {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(root)
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unable to read git status".to_string(),
    }
}

fn cmd_status(root: &Path) -> Result<Value> {
    let skill = read_active_skill(root);
    let cells = read_cells_registry(root);
    let git = git_status_short(root);

    let active_cell = skill.as_ref().and_then(|s| {
        let idx = find_active_cell(&cells, &s.skill_id)?;
        Some(&cells[idx])
    });

    let issue_binding = read_json_file(&trinity_path(root, &["state", "issue-binding.json"]));

    let mut status = serde_json::json!({
        "active_skill": skill,
        "active_cell": active_cell.map(|c| serde_json::json!({
            "cell_id": c.cell_id,
            "step_count": c.steps.len(),
            "sealed": c.seal.is_some()
        })),
        "issue_binding": issue_binding,
        "uncommitted_changes": if git.is_empty() { "none" } else { &git },
    });

    let mut text_parts = Vec::new();
    text_parts.push("=== PHI LOOP Status ===".to_string());

    if let Some(ref s) = skill {
        text_parts.push(format!("Skill: {} [{}]", s.skill_id, s.status));
        text_parts.push(format!("  Issue: {}", s.issue_id));
        text_parts.push(format!("  Description: {}", s.description));
        text_parts.push(format!("  Started: {}", s.started_at));
    } else {
        text_parts.push("No active skill session.".to_string());
    }

    if let Some(ref c) = active_cell {
        text_parts.push(format!("Cell: {} ({} steps)", c.cell_id, c.steps.len()));
    }

    if git.is_empty() {
        text_parts.push("Git: clean".to_string());
    } else {
        let count = git.lines().count();
        text_parts.push(format!("Git: {} uncommitted change(s)", count));
    }

    if let Some(obj) = status.as_object_mut() {
        obj.insert("_display".to_string(), Value::String(text_parts.join("\n")));
    }

    Ok(tool_result_text(&serde_json::to_string_pretty(&status)?))
}

fn cmd_skill_begin(root: &Path, args: &Value) -> Result<Value> {
    let issue = args
        .get("issue")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("missing or invalid 'issue' parameter"))?;
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'description' parameter"))?;

    let now = Utc::now().to_rfc3339();
    let session_id = format!("{}#{}", &now[..19], Uuid::new_v4().as_simple());
    let skill_id = format!("issue-{}", issue);

    let existing = read_active_skill(root);
    if let Some(ref s) = existing {
        if s.status == "active" {
            return Ok(tool_error_text(&format!(
                "Active skill already exists: {} (status: {}). End it first with tri_skill_end.",
                s.skill_id, s.status
            )));
        }
    }

    let skill = ActiveSkill {
        skill_id: skill_id.clone(),
        session_id: session_id.clone(),
        issue_id: format!("#{}", issue),
        description: description.to_string(),
        started_at: now.clone(),
        started_by: "tri-mcp".to_string(),
        status: "active".to_string(),
    };

    write_active_skill(root, &skill)?;

    let cell = CellEntry {
        cell_id: format!("cell-{}", Uuid::new_v4().as_simple()),
        skill_id: skill_id.clone(),
        created_at: now.clone(),
        steps: vec![CheckpointStep {
            step: "skill.begin".to_string(),
            timestamp: now.clone(),
        }],
        seal: None,
    };

    let mut cells = read_cells_registry(root);
    cells.push(cell);
    write_cells_registry(root, &cells)?;

    append_akashic(
        root,
        &make_akashic_event(
            "skill.begin",
            "success",
            serde_json::json!({
                "skill_id": skill_id,
                "session_id": session_id,
                "issue": issue,
                "description": description,
                "origin": "mcp"
            }),
        ),
    )?;

    let result = serde_json::json!({
        "skill_id": skill_id,
        "session_id": session_id,
        "issue": issue,
        "status": "active",
        "message": format!("PHI LOOP skill session started for issue #{}", issue)
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_skill_end(root: &Path) -> Result<Value> {
    let mut skill =
        read_active_skill(root).ok_or_else(|| anyhow::anyhow!("No active skill session to end"))?;

    if skill.status != "active" {
        return Ok(tool_error_text(&format!(
            "Skill {} is not active (status: {})",
            skill.skill_id, skill.status
        )));
    }

    let now = Utc::now().to_rfc3339();
    skill.status = "complete".to_string();
    write_active_skill(root, &skill)?;

    let mut cells = read_cells_registry(root);
    if let Some(idx) = find_active_cell(&cells, &skill.skill_id) {
        cells[idx].steps.push(CheckpointStep {
            step: "skill.end".to_string(),
            timestamp: now.clone(),
        });
        write_cells_registry(root, &cells)?;
    }

    append_akashic(
        root,
        &make_akashic_event(
            "skill.end",
            "success",
            serde_json::json!({
                "skill_id": skill.skill_id,
                "session_id": skill.session_id,
                "origin": "mcp"
            }),
        ),
    )?;

    let result = serde_json::json!({
        "skill_id": skill.skill_id,
        "status": "complete",
        "message": format!("PHI LOOP skill session ended: {}", skill.skill_id)
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_cell_checkpoint(root: &Path, args: &Value) -> Result<Value> {
    let step = args
        .get("step")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'step' parameter"))?;

    let skill =
        read_active_skill(root).ok_or_else(|| anyhow::anyhow!("No active skill session"))?;

    if skill.status != "active" {
        return Ok(tool_error_text(&format!(
            "Skill is not active (status: {})",
            skill.status
        )));
    }

    let now = Utc::now().to_rfc3339();
    let mut cells = read_cells_registry(root);
    let idx = find_active_cell(&cells, &skill.skill_id)
        .ok_or_else(|| anyhow::anyhow!("No active cell for skill {}", skill.skill_id))?;

    cells[idx].steps.push(CheckpointStep {
        step: step.to_string(),
        timestamp: now.clone(),
    });

    let step_count = cells[idx].steps.len();
    write_cells_registry(root, &cells)?;

    append_akashic(
        root,
        &make_akashic_event(
            "cell.checkpoint",
            "success",
            serde_json::json!({
                "skill_id": skill.skill_id,
                "cell_id": cells[idx].cell_id,
                "step": step,
                "step_count": step_count
            }),
        ),
    )?;

    let result = serde_json::json!({
        "cell_id": cells[idx].cell_id,
        "step": step,
        "step_count": step_count,
        "message": format!("Checkpoint recorded: '{}' (step {})", step, step_count)
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_cell_seal(root: &Path) -> Result<Value> {
    let skill =
        read_active_skill(root).ok_or_else(|| anyhow::anyhow!("No active skill session"))?;

    if skill.status != "active" {
        return Ok(tool_error_text(&format!(
            "Skill is not active (status: {})",
            skill.status
        )));
    }

    let mut cells = read_cells_registry(root);
    let idx = find_active_cell(&cells, &skill.skill_id)
        .ok_or_else(|| anyhow::anyhow!("No active cell for skill {}", skill.skill_id))?;

    if cells[idx].seal.is_some() {
        return Ok(tool_error_text(&format!(
            "Cell {} is already sealed",
            cells[idx].cell_id
        )));
    }

    let now = Utc::now().to_rfc3339();
    let mut hasher = Sha256::new();
    for s in &cells[idx].steps {
        hasher.update(s.step.as_bytes());
        hasher.update(s.timestamp.as_bytes());
    }
    hasher.update(skill.skill_id.as_bytes());
    hasher.update(cells[idx].cell_id.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_hex = format!("{:x}", hash_bytes);

    cells[idx].seal = Some(CellSeal {
        hash: hash_hex.clone(),
        sealed_at: now.clone(),
        step_count: cells[idx].steps.len(),
    });

    let cell_id = cells[idx].cell_id.clone();
    let step_count = cells[idx].steps.len();
    write_cells_registry(root, &cells)?;

    let seal_path = trinity_path(root, &["seals", &format!("{}.json", skill.skill_id)]);
    let seal_data = serde_json::json!({
        "cell_id": cell_id,
        "skill_id": skill.skill_id,
        "hash": hash_hex,
        "sealed_at": now,
        "step_count": step_count,
        "session_id": skill.session_id
    });
    write_json_file(&seal_path, &seal_data)?;

    append_akashic(
        root,
        &make_akashic_event(
            "cell.seal",
            "success",
            serde_json::json!({
                "skill_id": skill.skill_id,
                "cell_id": cell_id,
                "hash": hash_hex,
                "step_count": step_count
            }),
        ),
    )?;

    let result = serde_json::json!({
        "cell_id": cell_id,
        "hash": hash_hex,
        "step_count": step_count,
        "sealed_at": now,
        "message": format!("Cell sealed with hash {}", &hash_hex[..16])
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_gen(root: &Path, args: &Value) -> Result<Value> {
    let spec_path = args
        .get("spec_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'spec_path' parameter"))?;

    let spec = Path::new(spec_path);
    if !spec.exists() {
        return Ok(tool_error_text(&format!(
            "Spec file not found: {}",
            spec_path
        )));
    }

    let output = Command::new("t27c")
        .args(["gen", spec_path])
        .current_dir(root)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let success = out.status.success();

            append_akashic(
                root,
                &make_akashic_event(
                    "tri.gen",
                    if success { "success" } else { "failure" },
                    serde_json::json!({
                        "spec_path": spec_path,
                        "exit_code": out.status.code()
                    }),
                ),
            )?;

            let result = serde_json::json!({
                "success": success,
                "exit_code": out.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "spec_path": spec_path
            });

            Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
        }
        Err(e) => {
            let msg = format!("Failed to execute t27c: {}. Is t27c on PATH?", e);
            Ok(tool_error_text(&msg))
        }
    }
}

fn cmd_test(root: &Path, args: &Value) -> Result<Value> {
    let spec_path = args
        .get("spec_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'spec_path' parameter"))?;

    let spec = Path::new(spec_path);
    if !spec.exists() {
        return Ok(tool_error_text(&format!(
            "Spec file not found: {}",
            spec_path
        )));
    }

    let output = Command::new("t27c")
        .args(["test", spec_path])
        .current_dir(root)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let success = out.status.success();

            append_akashic(
                root,
                &make_akashic_event(
                    "tri.test",
                    if success { "success" } else { "failure" },
                    serde_json::json!({
                        "spec_path": spec_path,
                        "exit_code": out.status.code()
                    }),
                ),
            )?;

            let result = serde_json::json!({
                "success": success,
                "exit_code": out.status.code(),
                "stdout": stdout,
                "stderr": stderr,
                "spec_path": spec_path
            });

            Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
        }
        Err(e) => {
            let msg = format!("Failed to execute t27c: {}. Is t27c on PATH?", e);
            Ok(tool_error_text(&msg))
        }
    }
}

fn cmd_verdict(root: &Path) -> Result<Value> {
    let swarm_health = read_json_file(&trinity_path(root, &["state", "swarm-health.json"]));
    let queen_health = read_json_file(&trinity_path(root, &["state", "queen-health.json"]));

    let mut toxic_found = false;
    let mut details = Vec::new();

    if let Some(ref swarm) = swarm_health {
        let toxic_rate = swarm
            .get("metrics")
            .and_then(|m| m.get("toxic_rate"))
            .and_then(|r| r.as_f64())
            .unwrap_or(0.0);
        let repeat_failures = swarm
            .get("metrics")
            .and_then(|m| m.get("repeat_failures"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0);
        let stuck_tasks = swarm
            .get("metrics")
            .and_then(|m| m.get("stuck_tasks"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0);

        if toxic_rate > 0.1 {
            toxic_found = true;
            details.push(format!("High toxic rate: {:.3}", toxic_rate));
        }
        if repeat_failures > 3 {
            toxic_found = true;
            details.push(format!("Repeat failures: {}", repeat_failures));
        }
        if stuck_tasks > 2 {
            toxic_found = true;
            details.push(format!("Stuck tasks: {}", stuck_tasks));
        }
    } else {
        details.push("No swarm health data available".to_string());
    }

    if let Some(ref queen) = queen_health {
        let score = queen.get("score").and_then(|s| s.as_f64()).unwrap_or(1.0);
        if score < 0.5 {
            toxic_found = true;
            details.push(format!("Low queen score: {:.3}", score));
        }
    }

    let verdict = if toxic_found { "TOXIC" } else { "CLEAN" };

    let result = serde_json::json!({
        "verdict": verdict,
        "toxic_regressions": toxic_found,
        "details": details,
        "swarm_health": swarm_health,
        "queen_health": queen_health,
        "message": if toxic_found {
            format!("TOXIC REGRESSIONS DETECTED: {}", details.join("; "))
        } else {
            "No toxic regressions detected. All systems nominal.".to_string()
        }
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_experience_save(root: &Path, args: &Value) -> Result<Value> {
    let episode_type = args
        .get("episode_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'episode_type' parameter"))?;
    let summary = args
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing 'summary' parameter"))?;
    let ring = args.get("ring").and_then(|v| v.as_i64());

    let now = Utc::now().to_rfc3339();
    let episode_id = format!("ep-{}", Uuid::new_v4().as_simple());

    let mut experience = serde_json::json!({
        "episode_id": episode_id,
        "episode_type": episode_type,
        "summary": summary,
        "saved_at": now,
        "source": "tri-mcp"
    });

    if let Some(r) = ring {
        experience["ring"] = serde_json::json!(r);
    }

    if let Some(skill) = read_active_skill(root) {
        experience["skill_id"] = serde_json::json!(skill.skill_id);
        experience["session_id"] = serde_json::json!(skill.session_id);
    }

    let exp_path = trinity_path(root, &["experience", "episodes.jsonl"]);
    if let Some(parent) = exp_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&exp_path)?;
    writeln!(file, "{}", serde_json::to_string(&experience)?)?;

    append_akashic(
        root,
        &make_akashic_event(
            "experience.save",
            "success",
            serde_json::json!({
                "episode_id": episode_id,
                "episode_type": episode_type,
                "ring": ring
            }),
        ),
    )?;

    let result = serde_json::json!({
        "episode_id": episode_id,
        "message": format!("Experience episode saved: {}", episode_id)
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn cmd_health(root: &Path) -> Result<Value> {
    let queen = read_json_file(&trinity_path(root, &["state", "queen-health.json"]));
    let swarm = read_json_file(&trinity_path(root, &["state", "swarm-health.json"]));

    let queen_verdict = queen
        .as_ref()
        .and_then(|q| q.get("verdict"))
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
    let swarm_status = swarm
        .as_ref()
        .and_then(|s| s.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");

    let overall = match (queen_verdict, swarm_status) {
        ("GREEN", "GREEN") => "GREEN",
        ("YELLOW", _) | (_, "YELLOW") => "YELLOW",
        ("RED", _) | (_, "RED") => "RED",
        _ => "UNKNOWN",
    };

    let result = serde_json::json!({
        "overall": overall,
        "queen": queen,
        "swarm": swarm,
        "message": format!("Overall health: {} (queen: {}, swarm: {})", overall, queen_verdict, swarm_status)
    });

    Ok(tool_result_text(&serde_json::to_string_pretty(&result)?))
}

fn main() -> Result<()> {
    let root = find_repo_root().ok_or_else(|| {
        anyhow::anyhow!("Cannot find .trinity/ directory. Run from within a t27 repository.")
    })?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": format!("Parse error: {}", e)}
                });
                writeln!(stdout, "{}", serde_json::to_string(&err)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let result = match method {
            "initialize" => handle_initialize(),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(request, &root),
            _ if method.starts_with("notifications/") => {
                continue;
            }
            _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
        };

        if let Some(id) = id {
            let response = match result {
                Ok(r) => serde_json::json!({"jsonrpc": "2.0", "id": id, "result": r}),
                Err(e) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {"code": -32603, "message": e.to_string()}
                }),
            };
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }

    Ok(())
}
