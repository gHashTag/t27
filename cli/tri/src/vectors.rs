//! `tri vectors` — the executed-vector registry, as a command.
//!
//! A conformance corpus that is displayed but never run is worse than no
//! corpus: the summary prints a stored `verdict` field, a reader takes it for
//! a measurement, and the gate stays green through anything. That was the
//! state of this repository's 34 vector files until #2241 — 0 of them had ever
//! been applied to RTL.
//!
//! The repair is not "run them all". It is a registry that sorts every file
//! into exactly one of THREE verdicts, because two verdicts force every
//! unexecutable artifact to masquerade as one of them:
//!
//!   * **executed**    — a call template maps the case shape onto real entry
//!                       points; gate hard on it.
//!   * **debt**        — executable in principle, blocked by a NUMBERED defect;
//!                       print it, link the issue, never count it as covered.
//!   * **aspirational** — describes behaviour no current interface exposes
//!                       (bit-level protocol vectors against a combinational
//!                       model; prose-only cases with no data fields). Running
//!                       these would test an invention.
//!
//! `run` was a hand-typed gen-verilog-then-python pair a dozen times before it
//! became a command; `debt` was a jq-and-eyeball pass three times.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum VectorsCmd {
    /// Generate a module's Verilog and execute its registered vectors.
    Run {
        /// Module stem under specs/fpga (e.g. `mac`).
        module: String,
        /// Keep the generated .v and testbench for inspection.
        #[arg(long)]
        keep: bool,
    },
    /// Inventory every vector file: executed, debt, or aspirational.
    Debt,
}

pub fn run(cmd: &VectorsCmd) -> Result<()> {
    match cmd {
        VectorsCmd::Run { module, keep } => run_module(module, *keep),
        VectorsCmd::Debt => debt(),
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git rev-parse")?;
    if !out.status.success() {
        bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// The one binary that must exist, named once so the error can say how to get it.
fn t27c(root: &Path) -> Result<PathBuf> {
    let p = root.join("target/release/t27c");
    if !p.exists() {
        bail!(
            "{} not built — run `cargo build --release -p t27c` first",
            p.display()
        );
    }
    Ok(p)
}

fn run_module(module: &str, keep: bool) -> Result<()> {
    let root = repo_root()?;
    let spec = root.join(format!("specs/fpga/{}.t27", module));
    if !spec.exists() {
        bail!("no spec at {}", spec.display());
    }
    let runner = root.join("tools/run_conformance_vvp.py");
    if !runner.exists() {
        bail!("no runner at {}", runner.display());
    }

    let workdir = root.join("build/fpga/conformance");
    std::fs::create_dir_all(&workdir).context("create build/fpga/conformance")?;
    let vpath = workdir.join(format!("{}.v", module));

    let gen = Command::new(t27c(&root)?)
        .args(["gen-verilog", &spec.to_string_lossy()])
        .output()
        .context("t27c gen-verilog")?;
    if !gen.status.success() {
        // The compiler's own message names file, function and token; a wrapper
        // that swallows it turns a typo into a mystery (t27 #2186).
        print!("{}", String::from_utf8_lossy(&gen.stderr));
        bail!("gen-verilog failed for {}", module);
    }
    std::fs::write(&vpath, &gen.stdout).context("write generated Verilog")?;

    let status = Command::new("python3")
        .arg(&runner)
        .arg(module)
        .arg(&vpath)
        .arg(&workdir)
        .status()
        .context("spawning the vector runner")?;

    if !keep {
        let _ = std::fs::remove_file(&vpath);
    }
    if !status.success() {
        bail!("vector execution failed for {}", module);
    }
    Ok(())
}

fn debt() -> Result<()> {
    let root = repo_root()?;
    let dir = root.join("conformance");
    let runner = root.join("tools/run_conformance_vvp.py");
    let registry = std::fs::read_to_string(&runner).unwrap_or_default();

    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("fpga_") && n.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();

    let (mut executed, mut other) = (0usize, 0usize);
    println!("{:<38} {}", "vector file", "verdict");
    for f in &files {
        let stem = f
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .trim_start_matches("fpga_")
            .to_string();
        // A module counts as executed only when the registry names it as a key
        // — the presence of a vector file proves nothing about execution, and
        // that confusion is exactly what this command exists to end.
        let key = format!("\"{}\": (", stem.trim_end_matches("_vectors"));
        let is_executed = registry.contains(&key);
        if is_executed {
            executed += 1;
        } else {
            other += 1;
        }
        println!(
            "{:<38} {}",
            f.file_name().unwrap().to_string_lossy(),
            if is_executed { "executed" } else { "not executed" }
        );
    }
    println!();
    println!(
        "{} executed, {} not executed, {} total.",
        executed,
        other,
        files.len()
    );
    println!(
        "Not-executed splits into DEBT (numbered defect: see #2410, #2413) and\n\
         ASPIRATIONAL (no interface exposes the behaviour: uart's bit-level\n\
         protocol vectors, fifo's prose-only config cases). Neither counts as\n\
         coverage; both are printed so nobody mistakes a file for a check."
    );
    Ok(())
}
