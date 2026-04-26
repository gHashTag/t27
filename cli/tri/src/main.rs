mod railway;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "tri", about = "PHI LOOP CLI wrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status,
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Cell {
        #[command(subcommand)]
        action: CellAction,
    },
    Gen {
        spec_path: String,
    },
    Test {
        spec_path: String,
    },
    Verdict {
        #[arg(long)]
        toxic: bool,
    },
    Experience {
        #[command(subcommand)]
        action: ExperienceAction,
    },
    Doctor {
        action: String,
    },
    Health {
        target: Option<String>,
    },
    Railway {
        #[command(subcommand)]
        action: railway::RailwayAction,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    Begin {
        #[arg(long)]
        issue: u64,
        #[arg(long)]
        desc: String,
    },
    End,
}

#[derive(Subcommand)]
enum CellAction {
    Checkpoint {
        #[arg(long)]
        step: String,
    },
    Seal,
}

#[derive(Subcommand)]
enum ExperienceAction {
    Save,
}

#[derive(Serialize, Deserialize, Default)]
struct ActiveSkill {
    skill_id: Option<String>,
    session_id: Option<String>,
    issue_id: Option<String>,
    issue_title: Option<String>,
    description: Option<String>,
    started_at: Option<String>,
    started_by: Option<String>,
    status: String,
    allowed_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Cell {
    id: String,
    skill: String,
    issue: Option<String>,
    issue_title: Option<String>,
    episode: String,
    agent: String,
    spec_path: Option<String>,
    started_at: String,
    checkpoints: Vec<Checkpoint>,
    state: String,
    verdict: Option<String>,
    commit: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Checkpoint {
    step: u32,
    name: String,
    hash: String,
    at: String,
}

#[derive(Serialize)]
struct AkashicEvent {
    at: String,
    event: String,
    skill_id: Option<String>,
    cell_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<serde_json::Value>,
}

fn find_trinity_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join(".trinity").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!("could not find .trinity/ directory in any parent");
        }
    }
}

fn trinity_path(root: &Path, sub: &str) -> PathBuf {
    root.join(".trinity").join(sub)
}

fn ensure_dirs(root: &Path) -> Result<()> {
    for sub in &["state", "cells", "events", "experience"] {
        fs::create_dir_all(trinity_path(root, sub))?;
    }
    Ok(())
}

fn load_active_skill(root: &Path) -> Result<ActiveSkill> {
    let p = trinity_path(root, "state/active-skill.json");
    if !p.exists() {
        return Ok(ActiveSkill {
            status: "none".into(),
            ..Default::default()
        });
    }
    let data = fs::read_to_string(&p)?;
    Ok(serde_json::from_str(&data)?)
}

fn save_active_skill(root: &Path, skill: &ActiveSkill) -> Result<()> {
    let p = trinity_path(root, "state/active-skill.json");
    let data = serde_json::to_string_pretty(skill)?;
    fs::write(&p, data)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct CellRegistry {
    cells: Vec<Cell>,
}

fn load_registry(root: &Path) -> Result<CellRegistry> {
    let p = trinity_path(root, "cells/registry.json");
    if !p.exists() {
        return Ok(CellRegistry::default());
    }
    let data = fs::read_to_string(&p)?;
    Ok(serde_json::from_str(&data)?)
}

fn save_registry(root: &Path, reg: &CellRegistry) -> Result<()> {
    let p = trinity_path(root, "cells/registry.json");
    let data = serde_json::to_string_pretty(reg)?;
    fs::write(&p, data)?;
    Ok(())
}

fn append_akashic(root: &Path, evt: &AkashicEvent) -> Result<()> {
    let p = trinity_path(root, "events/akashic-log.jsonl");
    let line = serde_json::to_string(evt)? + "\n";
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)?
        .write_all(line.as_bytes())?;
    Ok(())
}

use std::io::Write;

fn file_sha256(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

fn git_short_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn run_t27c(args: &[&str]) -> Result<()> {
    let status = Command::new("t27c")
        .args(args)
        .status()
        .context("failed to execute t27c")?;
    if !status.success() {
        bail!("t27c {} exited with {:?}", args.join(" "), status);
    }
    Ok(())
}

fn cmd_status(root: &Path) -> Result<()> {
    let skill = load_active_skill(root)?;
    let reg = load_registry(root)?;
    let git = git_short_hash();

    println!("=== PHI LOOP STATUS ===");
    println!(
        "git: {}",
        if git.is_empty() {
            "unknown".into()
        } else {
            git
        }
    );

    match skill.status.as_str() {
        "active" => {
            println!(
                "skill: {} ({})",
                skill.skill_id.as_deref().unwrap_or("?"),
                skill.description.as_deref().unwrap_or("?")
            );
        }
        _ => {
            println!("skill: none");
        }
    }

    let active_cells: Vec<&Cell> = reg.cells.iter().filter(|c| c.state == "active").collect();
    println!(
        "cells: {} active / {} total",
        active_cells.len(),
        reg.cells.len()
    );

    for c in &active_cells {
        println!(
            "  [{}] {} checkpoints={}",
            c.id,
            c.spec_path.as_deref().unwrap_or("-"),
            c.checkpoints.len()
        );
    }

    let health_p = trinity_path(root, "state/queen-health.json");
    if health_p.exists() {
        let data = fs::read_to_string(&health_p)?;
        println!("queen: {}", data.trim());
    }

    Ok(())
}

fn cmd_skill_begin(root: &Path, issue: u64, desc: &str) -> Result<()> {
    ensure_dirs(root)?;

    let mut skill = load_active_skill(root)?;
    if skill.status == "active" {
        bail!(
            "active skill already in progress: {}",
            skill.skill_id.as_deref().unwrap_or("?")
        );
    }

    let ts = Utc::now().to_rfc3339();
    let skill_id = format!("skill-{}-{}", issue, Utc::now().timestamp());
    let session_id = format!("{}#{}", ts, skill_id);

    skill.skill_id = Some(skill_id.clone());
    skill.session_id = Some(session_id.clone());
    skill.issue_id = Some(issue.to_string());
    skill.issue_title = Some(desc.to_string());
    skill.description = Some(desc.to_string());
    skill.started_at = Some(ts.clone());
    skill.started_by = Some("tri-cli".into());
    skill.status = "active".into();
    skill.allowed_paths = vec!["specs/".into(), "gen/".into(), "tests/".into()];

    save_active_skill(root, &skill)?;

    let cell_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let commit = git_short_hash();
    let cell = Cell {
        id: cell_id.clone(),
        skill: skill_id.clone(),
        issue: Some(issue.to_string()),
        issue_title: Some(desc.to_string()),
        episode: session_id.clone(),
        agent: "tri".into(),
        spec_path: None,
        started_at: ts.clone(),
        checkpoints: vec![],
        state: "active".into(),
        verdict: None,
        commit: if commit.is_empty() {
            None
        } else {
            Some(commit)
        },
    };

    let mut reg = load_registry(root)?;
    reg.cells.push(cell);
    save_registry(root, &reg)?;

    append_akashic(
        root,
        &AkashicEvent {
            at: ts,
            event: "skill.begin".into(),
            skill_id: Some(skill_id),
            cell_id: Some(cell_id),
            detail: Some(serde_json::json!({ "issue": issue, "desc": desc })),
        },
    )?;

    println!(
        "skill began: {} issue=#{}",
        skill.skill_id.as_deref().unwrap(),
        issue
    );
    Ok(())
}

fn cmd_skill_end(root: &Path) -> Result<()> {
    let mut skill = load_active_skill(root)?;
    if skill.status != "active" {
        bail!("no active skill");
    }

    let ts = Utc::now().to_rfc3339();
    let sid = skill.skill_id.clone();

    skill.status = "closed".into();
    save_active_skill(root, &skill)?;

    let mut reg = load_registry(root)?;
    for c in reg.cells.iter_mut() {
        if c.state == "active" && c.skill == sid.as_deref().unwrap_or("") {
            c.state = "closed".into();
        }
    }
    save_registry(root, &reg)?;

    append_akashic(
        root,
        &AkashicEvent {
            at: ts,
            event: "skill.end".into(),
            skill_id: sid,
            cell_id: None,
            detail: None,
        },
    )?;

    println!("skill ended");
    Ok(())
}

fn cmd_cell_checkpoint(root: &Path, step_name: &str) -> Result<()> {
    let skill = load_active_skill(root)?;
    if skill.status != "active" {
        bail!("no active skill");
    }

    let skill_id = skill.skill_id.as_deref().unwrap_or("");

    let mut reg = load_registry(root)?;
    let cell = reg
        .cells
        .iter_mut()
        .find(|c| c.state == "active" && c.skill == skill_id)
        .context("no active cell for current skill")?;

    let step_num = (cell.checkpoints.len() as u32) + 1;
    let hash = match &cell.spec_path {
        Some(p) if Path::new(p).exists() => file_sha256(Path::new(p))?,
        _ => "no-spec".into(),
    };
    let ts = Utc::now().to_rfc3339();

    cell.checkpoints.push(Checkpoint {
        step: step_num,
        name: step_name.into(),
        hash,
        at: ts.clone(),
    });

    let cell_id = cell.id.clone();
    save_registry(root, &reg)?;

    append_akashic(
        root,
        &AkashicEvent {
            at: ts,
            event: "cell.checkpoint".into(),
            skill_id: Some(skill_id.into()),
            cell_id: Some(cell_id),
            detail: Some(serde_json::json!({ "step": step_num, "name": step_name })),
        },
    )?;

    println!("checkpoint {} recorded", step_num);
    Ok(())
}

fn cmd_cell_seal(root: &Path) -> Result<()> {
    let skill = load_active_skill(root)?;
    if skill.status != "active" {
        bail!("no active skill");
    }

    let skill_id = skill.skill_id.as_deref().unwrap_or("");

    let mut reg = load_registry(root)?;
    let cell = reg
        .cells
        .iter_mut()
        .find(|c| c.state == "active" && c.skill == skill_id)
        .context("no active cell for current skill")?;

    let ts = Utc::now().to_rfc3339();
    let commit = git_short_hash();
    cell.state = "sealed".into();
    cell.verdict = Some("clean".into());
    cell.commit = if commit.is_empty() {
        cell.commit.clone()
    } else {
        Some(commit)
    };

    let cell_id = cell.id.clone();
    save_registry(root, &reg)?;

    append_akashic(
        root,
        &AkashicEvent {
            at: ts,
            event: "cell.seal".into(),
            skill_id: Some(skill_id.into()),
            cell_id: Some(cell_id.clone()),
            detail: None,
        },
    )?;

    println!("cell sealed: {}", cell_id);
    Ok(())
}

fn cmd_gen(spec_path: &str) -> Result<()> {
    run_t27c(&["gen-verilog", spec_path])?;
    run_t27c(&["gen-c", spec_path])?;
    run_t27c(&["gen-rust", spec_path])?;
    println!("generation complete: {}", spec_path);
    Ok(())
}

fn cmd_test(spec_path: &str) -> Result<()> {
    run_t27c(&["test", spec_path])?;
    println!("tests passed: {}", spec_path);
    Ok(())
}

fn cmd_verdict(toxic: bool) -> Result<()> {
    run_t27c(&["validate-seals"])?;
    run_t27c(&["validate-phi-identity"])?;
    if toxic {
        run_t27c(&["validate-toxicity"])?;
    }
    println!("verdict: clean");
    Ok(())
}

fn cmd_experience_save(root: &Path) -> Result<()> {
    ensure_dirs(root)?;

    let skill = load_active_skill(root)?;
    let reg = load_registry(root)?;
    let ts = Utc::now().to_rfc3339();

    let skill_cells: Vec<&Cell> = reg
        .cells
        .iter()
        .filter(|c| {
            skill
                .skill_id
                .as_deref()
                .map_or(false, |sid| c.skill == sid)
        })
        .collect();

    let episode = serde_json::json!({
        "at": ts,
        "skill_id": skill.skill_id,
        "session_id": skill.session_id,
        "cells": skill_cells.len(),
        "total_checkpoints": skill_cells.iter().map(|c| c.checkpoints.len()).sum::<usize>(),
    });

    let ep_path = trinity_path(
        root,
        &format!("experience/episode-{}.jsonl", Utc::now().timestamp()),
    );
    let line = serde_json::to_string(&episode)? + "\n";
    fs::write(&ep_path, line)?;

    append_akashic(
        root,
        &AkashicEvent {
            at: ts,
            event: "experience.save".into(),
            skill_id: skill.skill_id,
            cell_id: None,
            detail: Some(episode),
        },
    )?;

    println!("experience saved");
    Ok(())
}

fn cmd_doctor(root: &Path, action: &str) -> Result<()> {
    match action {
        "start" => {
            ensure_dirs(root)?;
            let ts = Utc::now().to_rfc3339();
            let state = serde_json::json!({ "status": "running", "started_at": ts });
            let p = trinity_path(root, "state/doctor.json");
            fs::write(&p, serde_json::to_string_pretty(&state)?)?;
            println!("doctor started");
        }
        "stop" => {
            let p = trinity_path(root, "state/doctor.json");
            if p.exists() {
                let data = fs::read_to_string(&p)?;
                let mut state: serde_json::Value = serde_json::from_str(&data)?;
                state["status"] = serde_json::Value::String("stopped".into());
                state["stopped_at"] = serde_json::Value::String(Utc::now().to_rfc3339());
                fs::write(&p, serde_json::to_string_pretty(&state)?)?;
            }
            println!("doctor stopped");
        }
        "status" => {
            let p = trinity_path(root, "state/doctor.json");
            if p.exists() {
                let data = fs::read_to_string(&p)?;
                println!("{}", data.trim());
            } else {
                println!("doctor: not started");
            }
        }
        _ => bail!("unknown doctor action: {} (start|stop|status)", action),
    }
    Ok(())
}

fn cmd_health(root: &Path, target: Option<&str>) -> Result<()> {
    match target {
        Some("queen") | None => {
            let p = trinity_path(root, "state/queen-health.json");
            if p.exists() {
                let data = fs::read_to_string(&p)?;
                println!("{}", data.trim());
            } else {
                println!("queen: no health data");
            }
        }
        Some(other) => bail!("unknown health target: {}", other),
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Status => {
            let root = find_trinity_root()?;
            cmd_status(&root)?;
        }
        Commands::Skill { action } => {
            let root = find_trinity_root()?;
            match action {
                SkillAction::Begin { issue, desc } => cmd_skill_begin(&root, *issue, desc)?,
                SkillAction::End => cmd_skill_end(&root)?,
            }
        }
        Commands::Cell { action } => {
            let root = find_trinity_root()?;
            match action {
                CellAction::Checkpoint { step } => cmd_cell_checkpoint(&root, step)?,
                CellAction::Seal => cmd_cell_seal(&root)?,
            }
        }
        Commands::Gen { spec_path } => cmd_gen(spec_path)?,
        Commands::Test { spec_path } => cmd_test(spec_path)?,
        Commands::Verdict { toxic } => cmd_verdict(*toxic)?,
        Commands::Experience { action } => {
            let root = find_trinity_root()?;
            match action {
                ExperienceAction::Save => cmd_experience_save(&root)?,
            }
        }
        Commands::Doctor { action } => {
            let root = find_trinity_root()?;
            cmd_doctor(&root, action)?;
        }
        Commands::Health { target } => {
            let root = find_trinity_root()?;
            cmd_health(&root, target.as_deref())?;
        }
        Commands::Railway { action } => {
            let code = railway::run(action.clone())?;
            if code != 0 {
                std::process::exit(code);
            }
        }
    }

    Ok(())
}
