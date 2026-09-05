use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod abandoned;
mod census;
mod cibase;
mod competitors;
mod depin;
mod discard;
mod elab;
mod fleet;
mod fmtmine;
mod fpga;
mod gates;
mod gendet;
mod hooks;
mod inflight;
mod issues;
mod jumps;
mod kinddrift;
mod leanreach;
mod leanvac;
mod ledgers;
mod misread;
mod modreach;
mod mutate;
mod nownote;
mod orphaned;
mod prcheck;
mod prose;
mod quant;
mod red;
mod renum;
mod reseal;
mod rtl;
mod scratch;
mod seals;
mod skillnum;
mod sweep;
mod synth;
mod topic;
mod trees;
mod types_dup;
mod unparsed;
mod vectors;
mod vsim;

#[derive(Parser)]
#[command(name = "tri", about = "PHI LOOP CLI wrapper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum HarnessCmd {
    /// Test binaries whose tests share one scratch directory.
    Scratch {
        /// Exit non-zero when any binary shares a scratch directory.
        #[arg(long)]
        gate: bool,
        /// Negative control: prove this gate can see a planted collision.
        #[arg(long)]
        self_check: bool,
    },
}

#[derive(Subcommand)]
pub enum ModsCmd {
    /// List them.
    Orphan {
        /// Compare against docs/reports/orphan_modules.json and exit non-zero on a change.
        #[arg(long)]
        gate: bool,
        /// Negative control: prove this gate can see a planted orphan.
        #[arg(long)]
        self_check: bool,
    },
}

#[derive(Subcommand)]
enum Commands {
    Status,
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Match arms whose own comment names a case the pattern omits.
    Kinds {
        #[command(subcommand)]
        action: kinddrift::KindsCmd,
    },
    /// Completeness theorems whose Lean model is empty.
    Lean {
        #[command(subcommand)]
        action: leanvac::LeanCmd,
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
    Serve {
        #[arg(long, default_value = "0.0.0.0:3000")]
        addr: String,
    },
    /// FPGA programming via the in-tree DLC10 driver (pure Rust).
    Fpga {
        #[command(subcommand)]
        action: fpga::FpgaCmd,
    },
    /// Find the constants in a checker that nothing actually checks.
    Mutate {
        #[command(subcommand)]
        action: mutate::MutateCmd,
    },
    /// Recompute the FROZEN_HASH seal from the file it seals.
    Reseal {
        #[command(subcommand)]
        action: reseal::ResealCmd,
    },
    /// Write a docs/now/ entry without hand-writing the frame.
    Now {
        #[command(subcommand)]
        action: nownote::NowCmd,
    },
    /// Is the hardware this plan assumes actually attached?
    /// CI questions about the repository's own gates.
    Ci {
        #[command(subcommand)]
        action: cibase::CiCmd,
    },
    Fleet {
        #[command(subcommand)]
        action: fleet::FleetCmd,
    },
    /// Is this pull request actually safe to merge?
    Pr {
        #[command(subcommand)]
        action: prcheck::PrCmd,
    },
    /// Is a merge in flight here, and does this branch carry the base?
    Merging(inflight::Merging),
    /// The specs the compiler reads WRONGLY. Every gate is green on them.
    Misread(misread::Misread),
    /// What the checkouts on this disk are holding. Deletes nothing.
    Worktrees(trees::Worktrees),
    /// Synthesise across a parameter and check the area actually moves.
    Sweep {
        #[command(subcommand)]
        action: sweep::SweepCmd,
    },
    /// Synthesise a top module and report area, with the instrument named.
    Synth {
        #[command(subcommand)]
        action: synth::SynthCmd,
    },
    /// What is failing on the default branch right now, and since when.
    Red {
        #[command(subcommand)]
        action: red::RedCmd,
    },
    /// Find workflows that have never once succeeded.
    Gates {
        #[command(subcommand)]
        action: gates::GatesCmd,
    },
    /// Source files no crate root reaches, and so nothing compiles.
    /// Test-harness hygiene.
    Harness {
        #[command(subcommand)]
        action: HarnessCmd,
    },
    Mods {
        #[command(subcommand)]
        action: ModsCmd,
    },
    /// Classify a compiler's error output before quoting a number from it.
    Elab {
        #[command(subcommand)]
        action: elab::ElabCmd,
    },
    /// The structural check t27.ai offers, run locally: five verdicts, the
    /// yosys version beside the numbers, and no claim about correctness.
    Rtl {
        #[command(subcommand)]
        action: rtl::RtlCmd,
    },
    /// Recovery sites whose own comment names the construct they discard.
    Abandoned {
        #[command(subcommand)]
        action: abandoned::AbandonedCmd,
    },
    /// Type names with more than one definition in the spec tree.
    /// Run the formatter and restore every file it rewrote that you had not touched.
    Fmt {
        /// One package instead of the whole workspace.
        #[arg(short, long)]
        package: Option<String>,
        /// Report what is dirty and whether a workflow formats, and stop.
        #[arg(long)]
        dry_run: bool,
    },
    Types {
        #[command(subcommand)]
        action: types_dup::TypesCmd,
    },
    /// Has anyone already done this, or are they doing it now?
    Topic {
        /// Words to look for. A row matches on any; rows carrying more come first.
        #[arg(required = true)]
        keywords: Vec<String>,
        /// How many recent commits on the base branch to read.
        #[arg(long, default_value = "40")]
        commits: usize,
    },
    /// Every quantified clause, its binders, and the size of its domain.
    Quantifiers {
        #[command(subcommand)]
        action: quant::QuantCmd,
    },
    /// Checks whose input is not in the tree.
    Orphaned {
        #[command(subcommand)]
        action: orphaned::OrphanedCmd,
    },
    /// What the parser reads and throws away, ranked against its pinned bound.
    Discard {
        #[command(subcommand)]
        action: discard::DiscardCmd,
    },
    /// Specs a literate author left prose in, and how far that prose is from code.
    Prose {
        #[command(subcommand)]
        action: prose::ProseCmd,
    },
    /// Does each ledger's gate notice when one of its entries stops being true?
    Ledgers {
        #[command(subcommand)]
        action: ledgers::LedgersCmd,
    },
    /// Specs the compiler cannot read, ranked by the construct that stops it.
    /// Every census's printed population, against one counted another way.
    Census {
        #[command(subcommand)]
        action: census::CensusCmd,
    },
    /// The competitor table against its own contract: how many of its published
    /// scores were never published, and how many papers it counts twice.
    Competitors {
        #[command(subcommand)]
        action: competitors::CompetitorsCmd,
    },
    /// Open issues whose headline calls a workflow red, against what that
    /// workflow concludes on master today.
    Issues {
        #[command(subcommand)]
        action: issues::IssuesCmd,
    },
    Unparsed {
        #[command(subcommand)]
        action: unparsed::UnparsedCmd,
    },
    /// Does the same binary produce the same generated code twice?
    ///
    /// Every byte-comparison over emitted output -- seals, emit-bitexact,
    /// `corpus --per-spec` diffing -- assumes it does. Three of the four
    /// emitters do not; see #3006. (`Gen` is taken by the single-spec emitter,
    /// so this is `Emit`.)
    Emit {
        #[command(subcommand)]
        action: gendet::EmitCmd,
    },
    /// What happened to every `break` and `continue` the Verilog emitter had
    /// to lower?
    ///
    /// `break` was `disable fork;` and `continue` was `/* continue */;`. Both
    /// parse, both synthesise, both are no-ops -- so every instrument that
    /// asks whether the output PARSES said yes for as long as the emitter has
    /// existed. See #2988.
    Jumps {
        #[command(subcommand)]
        action: jumps::JumpsCmd,
    },
    /// How far each spec gets when its generated Verilog is actually RUN.
    ///
    /// The one arm that can catch a defect whose nature is that it compiles,
    /// and the one arm whose gate has had no targets since #2283 -- see #2987.
    Vsim {
        #[command(subcommand)]
        action: vsim::VsimCmd,
    },
    /// What `.trinity/seals` says about a spec, when it says it twice.
    Seals {
        #[command(subcommand)]
        action: seals::SealsCmd,
    },
    /// Pure-Rust ports of repository commit / push gates.
    Hooks {
        #[command(subcommand)]
        action: hooks::HooksCmd,
    },
    /// The executed-vector registry: run a module's vectors, or inventory
    /// which files are executed and which are only displayed.
    Vectors {
        #[command(subcommand)]
        action: vectors::VectorsCmd,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// Write a new lesson to the spool, unnumbered.
    ///
    /// The collision this removes: two branches each append `## N.` to SKILL.md
    /// numbered from their OWN base, both merge, and the number appears twice.
    /// It happened twice in two passes, and the two repairs then raced each
    /// other. No branch-side check can see it: `tri skill check` passes on both
    /// sides and fails only on the merge result.
    ///
    /// A spooled lesson has a unique path and no number, so two branches write
    /// two paths and there is nothing to conflict on -- the shape `docs/now/`
    /// already uses, adopted there after the same defect.
    Add {
        /// The section title, as it will read in SKILL.md.
        title: String,
        /// Which skill to file it under.
        #[arg(long, default_value = "ci-gates")]
        skill: String,
    },
    /// Fold every spooled lesson into SKILL.md, numbering them on the way in.
    ///
    /// The number is assigned HERE, against the SKILL.md in front of it, which
    /// is what makes this the step that cannot collide.
    Fold {
        /// Report what would move and write nothing.
        #[arg(long)]
        check: bool,
        /// Which skill's spool to fold.
        #[arg(long, default_value = "ci-gates")]
        skill: String,
    },
    /// Check every SKILL.md's section numbering for collisions.
    Check {
        /// Also print the gaps, which are reported but never fail.
        #[arg(long)]
        gaps: bool,
    },
    /// Sections whose body was cut short at some point in the file's history.
    Lost {
        /// Which document.
        #[arg(long, default_value = ".claude/skills/ci-gates/SKILL.md")]
        file: String,
        /// Compare against this ref instead of `origin/master`.
        #[arg(long, default_value = "origin/master")]
        base: String,
        /// Exit 1 if anything is found, for use as a gate.
        #[arg(long)]
        gate: bool,
    },
    /// Every cross-reference in the skills, and whether it resolves.
    Refs {
        /// Print every reference counted, not only the ones that dangle.
        #[arg(long)]
        list: bool,
    },
    /// Sections that state a FIGURE, and which of those a reader can re-take.
    Claims {
        /// Print every section in the free population, one line each.
        #[arg(long)]
        list: bool,
        /// Print `<skill>:<number>` for every counted section and nothing else.
        #[arg(long)]
        numbers: bool,
        /// List the sections whose figure stands over a SLIDING population.
        #[arg(long)]
        windowed: bool,
    },
    /// Move sections you appended to the numbers the base branch left free.
    Renumber {
        /// The branch whose numbering yours must follow. Use the shared base,
        /// not a peer branch: numbering against a sibling also rebuilds your
        /// file on it, and your PR would then carry the sibling's sections.
        #[arg(long, default_value = "origin/master")]
        base: String,
        /// Which skill file.
        #[arg(long, default_value = ".claude/skills/ci-gates/SKILL.md")]
        file: String,
        /// Report the moves and write nothing.
        #[arg(long)]
        check: bool,
        /// Remove the sections the base withdrew, instead of refusing.
        #[arg(long)]
        drop_withdrawn: bool,
        /// Start at this number instead of one past the base's highest. For a
        /// second open branch numbering against the same base -- pass a number,
        /// not a different --base.
        #[arg(long)]
        first: Option<usize>,
    },
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

pub fn find_trinity_root() -> Result<PathBuf> {
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
        Commands::Kinds { action } => kinddrift::run(action)?,
        Commands::Lean { action } => leanvac::run(action)?,
        Commands::Skill { action } => {
            let root = find_trinity_root()?;
            match action {
                SkillAction::Add { title, skill } => skillnum::run(&skillnum::SkillCmd::Add {
                    title: title.clone(),
                    skill: skill.clone(),
                })?,
                SkillAction::Fold { check, skill } => skillnum::run(&skillnum::SkillCmd::Fold {
                    check: *check,
                    skill: skill.clone(),
                })?,
                SkillAction::Check { gaps } => {
                    skillnum::run(&skillnum::SkillCmd::Check { gaps: *gaps })?
                }
                SkillAction::Lost { file, base, gate } => {
                    skillnum::run(&skillnum::SkillCmd::Lost {
                        file: file.clone(),
                        base: base.clone(),
                        gate: *gate,
                    })?
                }
                SkillAction::Refs { list } => {
                    skillnum::run(&skillnum::SkillCmd::Refs { list: *list })?
                }
                SkillAction::Claims {
                    list,
                    numbers,
                    windowed,
                } => skillnum::run(&skillnum::SkillCmd::Claims {
                    list: *list,
                    numbers: *numbers,
                    windowed: *windowed,
                })?,
                SkillAction::Renumber {
                    base,
                    file,
                    check,
                    first,
                    drop_withdrawn,
                } => renum::run(base, file, *check, *first, *drop_withdrawn)?,
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
        Commands::Serve { addr } => cmd_serve(addr)?,
        Commands::Fpga { action } => fpga::run(action)?,
        Commands::Mutate { action } => mutate::run(action)?,
        Commands::Reseal { action } => reseal::run(action)?,
        Commands::Now { action } => nownote::run(action)?,
        Commands::Ci { action } => cibase::run(action)?,
        Commands::Fleet { action } => fleet::run(action)?,
        Commands::Pr { action } => prcheck::run(action)?,
        Commands::Merging(a) => inflight::run(a)?,
        Commands::Misread(a) => misread::run(a)?,
        Commands::Worktrees(a) => trees::run(a)?,
        Commands::Sweep { action } => sweep::run(action)?,
        Commands::Synth { action } => synth::run(action)?,
        Commands::Red { action } => red::run(action)?,
        Commands::Gates { action } => gates::run(action)?,
        Commands::Vectors { action } => vectors::run(action)?,
        Commands::Harness { action } => match action {
            HarnessCmd::Scratch { gate, self_check } => scratch::run(*gate, *self_check)?,
        },
        Commands::Mods { action } => match action {
            ModsCmd::Orphan { gate, self_check } => {
                if *self_check {
                    modreach::self_check()?
                } else {
                    modreach::run(*gate)?
                }
            }
        },
        Commands::Elab { action } => elab::run(action)?,
        Commands::Rtl { action } => rtl::run(action)?,
        Commands::Abandoned { action } => abandoned::run(action)?,
        Commands::Fmt { package, dry_run } => {
            fmtmine::run(&find_trinity_root()?, package.as_deref(), *dry_run)?
        }
        Commands::Types { action } => types_dup::run(action)?,
        Commands::Topic { keywords, commits } => topic::run(keywords, *commits)?,
        Commands::Quantifiers { action } => quant::run(action)?,
        Commands::Orphaned { action } => orphaned::run(action)?,
        Commands::Discard { action } => discard::run(action)?,
        Commands::Prose { action } => prose::run(action, std::env::current_dir()?)?,
        Commands::Ledgers { action } => ledgers::run(action, std::env::current_dir()?)?,
        Commands::Census { action } => census::run(action)?,
        Commands::Competitors { action } => competitors::run(action)?,
        Commands::Issues { action } => issues::run(action)?,
        Commands::Unparsed { action } => unparsed::run(action, std::env::current_dir()?)?,
        Commands::Emit { action } => gendet::run(action)?,
        Commands::Jumps { action } => jumps::run(action)?,
        Commands::Vsim { action } => vsim::run(action)?,
        Commands::Seals { action } => seals::run(action)?,
        Commands::Hooks { action } => hooks::run(action)?,
    }

    Ok(())
}

fn cmd_serve(addr: &str) -> Result<()> {
    use axum::routing::{get, post};
    use axum::Router;
    use depin::prove;
    use depin::types::AppState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let state = Arc::new(RwLock::new(AppState::new()));

    let app = Router::new()
        .route("/prove", post(prove::post_prove))
        .route("/epoch-challenge", get(prove::get_epoch_challenge))
        .route("/health", get(prove::health_check))
        .with_state(state);

    println!("trinity depin v0.1.0 — listening on {}", addr);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
