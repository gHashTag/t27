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

    let (mut n_exec, mut n_debt, mut n_prose) = (0usize, 0usize, 0usize);
    let (mut c_total, mut c_data) = (0usize, 0usize);
    println!(
        "{:<38}{:>7}{:>7}  {}",
        "vector file", "cases", "data", "verdict"
    );
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_default();
        let (cases, data) = count_cases(&text);
        c_total += cases;
        c_data += data;
        let stem = f
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .trim_start_matches("fpga_")
            .to_string();
        // Executed only when the registry names the module as a key: the mere
        // presence of a vector file proves nothing about execution, and that
        // confusion is what this command exists to end.
        let key = format!("\"{}\": (", stem.trim_end_matches("_vectors"));
        let verdict = if registry.contains(&key) {
            n_exec += 1;
            "executed"
        } else if data > 0 {
            n_debt += 1;
            "debt (has data, no runner)"
        } else {
            n_prose += 1;
            "prose-only (no data at all)"
        };
        println!(
            "{:<38}{:>7}{:>7}  {}",
            f.file_name().unwrap().to_string_lossy(),
            cases,
            data,
            verdict
        );
    }
    println!();
    println!(
        "{} executed, {} debt, {} prose-only ({} files); {} cases, {} carrying data ({}%).",
        n_exec,
        n_debt,
        n_prose,
        files.len(),
        c_total,
        c_data,
        if c_total == 0 {
            0
        } else {
            c_data * 100 / c_total
        }
    );
    println!(
        "PROSE-ONLY is the majority and the important number: those cases carry\n\
         an id and a sentence, no inputs and no expected values, so no runner\n\
         can ever execute them as written -- they are documentation shaped like\n\
         tests. DEBT is executable in principle and blocked by numbered defects\n\
         (#2410 slices, #2413 test emission). Neither counts as coverage."
    );
    Ok(())
}

/// Count cases and how many carry any field beyond prose metadata.
///
/// String-aware by construction. The first version split objects on ',' and was
/// fooled by commas INSIDE description strings ("is_sync=true, is_async=false"),
/// reporting 147 data-carrying cases where an independent python pass reported
/// 100 -- the two instruments disagreed, and the text splitter was the liar.
/// This one tracks string state and collects only real KEYS (a string followed
/// by ':'), which reproduces the python count exactly.
fn count_cases(text: &str) -> (usize, usize) {
    const PROSE: [&str; 5] = ["id", "description", "note", "name", "comment"];
    let b = text.as_bytes();
    let (mut cases, mut data) = (0usize, 0usize);
    let mut i = 0usize;
    while let Some(rel) = text[i..].find("\"cases\"") {
        let mut j = i + rel + 7;
        // Find the opening bracket of the array.
        while j < b.len() && b[j] != b'[' {
            j += 1;
        }
        if j >= b.len() {
            break;
        }
        let mut depth = 0i32;
        let mut in_str = false;
        let mut esc = false;
        // Per-case state.
        let mut obj_depth = 0i32;
        let mut has_data = false;
        let mut key_start: Option<usize> = None;
        let mut last_string: Option<(usize, usize)> = None;
        while j < b.len() {
            let c = b[j];
            if in_str {
                if esc {
                    esc = false;
                } else if c == b'\\' {
                    esc = true;
                } else if c == b'"' {
                    in_str = false;
                    if let Some(st) = key_start.take() {
                        last_string = Some((st, j));
                    }
                }
                j += 1;
                continue;
            }
            match c {
                b'"' => {
                    in_str = true;
                    key_start = Some(j + 1);
                }
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                b'{' => {
                    if obj_depth == 0 {
                        has_data = false;
                    }
                    obj_depth += 1;
                }
                b'}' => {
                    obj_depth -= 1;
                    if obj_depth == 0 {
                        cases += 1;
                        if has_data {
                            data += 1;
                        }
                    }
                }
                b':' => {
                    // The string that just closed is a KEY.
                    if obj_depth >= 1 {
                        if let Some((a, z)) = last_string {
                            let k = &text[a..z];
                            if !PROSE.contains(&k) {
                                has_data = true;
                            }
                        }
                    }
                    last_string = None;
                }
                _ => {}
            }
            j += 1;
        }
        i = j.max(i + rel + 7);
    }
    (cases, data)
}
