//! `tri synth area` — synthesise and report area, with the instrument named.
//!
//! Written after getting the same task wrong twice in one session.
//!
//! First: the stat parser matched `Number of cells:`, which is what yosys 0.33
//! prints. 0.65 prints `N cells`. The parser found nothing, reported zero, and
//! zero looks exactly like a small design.
//!
//! Second: the output was captured into a shell variable and printed with
//! `echo`. zsh's `echo` interprets backslash escapes, and yosys writes Verilog
//! identifiers with a leading backslash -- `\tern_node` became a tab, and a
//! `\c` further along truncated the stream. Every count came back zero again,
//! from data that was correct on disk.
//!
//! Both failures produced a plausible number rather than an error, which is the
//! dangerous shape. This runs the flow, parses the counts, and refuses to print
//! anything if synthesis did not reach a stat block.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum SynthCmd {
    /// Synthesise a top module and report LUT/DSP/FF/CARRY4 counts.
    Area {
        /// Verilog sources, in order.
        #[arg(required = true)]
        sources: Vec<String>,
        /// Top module name.
        #[arg(long)]
        top: String,
        /// Forbid DSP inference, to see the LUT-only cost.
        #[arg(long)]
        nodsp: bool,
        /// Override a top-level parameter, repeatable: --param N=64
        #[arg(long = "param")]
        params: Vec<String>,
    },
}

pub fn run(cmd: &SynthCmd) -> Result<()> {
    match cmd {
        SynthCmd::Area {
            sources,
            top,
            nodsp,
            params,
        } => area(sources, top, *nodsp, params),
    }
}

/// Counts of a cell family, summed over every line naming it.
///
/// Deliberately matches the cell NAME rather than a surrounding phrase: the
/// phrase changed between yosys versions and the name did not.
fn count(log: &str, needle: &str, exact: bool) -> u64 {
    let mut total = 0u64;
    for line in log.lines() {
        let mut it = line.split_whitespace();
        let (n, name) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let hit = if exact { name == needle } else { name.starts_with(needle) };
        if hit {
            if let Ok(v) = n.parse::<u64>() {
                total += v;
            }
        }
    }
    total
}

fn yosys_version() -> Result<String> {
    let out = Command::new("yosys")
        .arg("-V")
        .output()
        .context("yosys is not installed or not on PATH")?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        bail!("yosys -V printed nothing; refusing to report area from an unnamed tool");
    }
    Ok(s.lines().next().unwrap_or("").to_string())
}

fn area(sources: &[String], top: &str, nodsp: bool, params: &[String]) -> Result<()> {
    let version = yosys_version()?;
    let mut script = format!("read_verilog -sv {}\n", sources.join(" "));
    for p in params {
        let (k, v) = p
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("--param wants NAME=VALUE, got {p:?}"))?;
        script.push_str(&format!("chparam -set {k} {v} {top}\n"));
    }
    script.push_str(&format!(
        "synth_xilinx -family xc7 -top {top}{}\nflatten\nstat\n",
        if nodsp { " -nodsp" } else { "" }
    ));

    let out = Command::new("yosys")
        .arg("-p")
        .arg(&script)
        .output()
        .context("failed to run yosys")?;
    // Both streams: yosys writes the banner to one and the report to the other
    // depending on version, and reading only one of them is how a report goes
    // missing without an error.
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    if !log.contains("Printing statistics") {
        bail!(
            "synthesis did not reach a stat block for {top}. Last lines:\n{}",
            log.lines().rev().take(6).collect::<Vec<_>>().join("\n")
        );
    }

    let luts = ["LUT1", "LUT2", "LUT3", "LUT4", "LUT5", "LUT6"]
        .iter()
        .map(|n| count(&log, n, true))
        .sum::<u64>();
    let dsp = count(&log, "DSP48", false);
    let ff = count(&log, "FD", false);
    let carry = count(&log, "CARRY4", true);

    println!("{top}   {}", if nodsp { "(DSP inference off)" } else { "(DSP inference on)" });
    for p in params {
        println!("  param {p}");
    }
    println!("  LUT     {luts}");
    println!("  DSP48   {dsp}");
    println!("  FF      {ff}");
    println!("  CARRY4  {carry}");
    println!("  instrument: {version}");
    println!();
    println!("Area only. No frequency: that needs a place-and-route this reports");
    println!("nothing about, and yosys `ltp` is not a substitute -- it counts");
    println!("topological hops in a netlist whose structure differs between");
    println!("designs, so it does not compare across them.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two yosys stat formats seen on this machine. A parser tied to the
    /// surrounding phrase silently reported zero on the version it did not
    /// expect, and zero reads exactly like a small design.
    #[test]
    fn counts_survive_both_stat_layouts() {
        let modern = "        52   LUT2\n       129   LUT3\n         3   DSP48E1\n";
        assert_eq!(count(modern, "LUT2", true), 52);
        assert_eq!(count(modern, "LUT3", true), 129);
        assert_eq!(count(modern, "DSP48", false), 3);
    }

    /// LUT6 must not be counted by a LUT1 query, and FDRE must be counted by FD.
    #[test]
    fn exact_and_prefix_matching_are_kept_apart() {
        let s = "  10   LUT1\n  20   LUT6\n  30   FDRE\n  40   FDSE\n";
        assert_eq!(count(s, "LUT1", true), 10);
        assert_eq!(count(s, "FD", false), 70);
    }

    /// A line with no leading count must not contribute, or prose in the log
    /// becomes area.
    #[test]
    fn prose_lines_are_not_counted() {
        let s = "Printing statistics.\n   LUT2 is a lookup table\n  7   LUT2\n";
        assert_eq!(count(s, "LUT2", true), 7);
    }
}
