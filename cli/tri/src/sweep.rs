//! `tri synth sweep` — synthesise across a parameter and check the area moves.
//!
//! Written after two synthesis folds in one afternoon, both of which reported a
//! number rather than an error.
//!
//! A layer whose operand memory had no write port: yosys propagated the
//! never-written memory as constant and pruned the design. It reported **12
//! logic cells** for a 64-tap multiplier layer, and the area did not grow with
//! fan-in at all.
//!
//! The same layer with one memory location per lane: reading N of them per
//! cycle is N read ports, which becomes muxes rather than block RAM. **23052
//! cells on a 7680-cell part**, zero RAM inferred.
//!
//! Neither is an error. Both are numbers, and both were caught only because a
//! human found them implausible. This checks the shape instead: sweep the
//! parameter, and say plainly when the area does not respond to it, or responds
//! far too violently.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum SweepCmd {
    /// Synthesise a top across values of one parameter and report the trend.
    Area {
        /// Verilog sources, in order.
        #[arg(required = true)]
        sources: Vec<String>,
        /// Top module name.
        #[arg(long)]
        top: String,
        /// Parameter to sweep, e.g. N
        #[arg(long)]
        param: String,
        /// Values to try, repeatable: --value 8 --value 16 --value 32
        #[arg(long = "value", required = true)]
        values: Vec<i64>,
        /// Extra fixed parameters, repeatable: --fixed ACC=16
        #[arg(long = "fixed")]
        fixed: Vec<String>,
        /// Forbid DSP inference.
        #[arg(long)]
        nodsp: bool,
    },
}

pub fn run(cmd: &SweepCmd) -> Result<()> {
    match cmd {
        SweepCmd::Area {
            sources,
            top,
            param,
            values,
            fixed,
            nodsp,
        } => sweep(sources, top, param, values, fixed, *nodsp),
    }
}

fn count(log: &str, needle: &str, exact: bool) -> u64 {
    let mut total = 0u64;
    for line in log.lines() {
        let mut it = line.split_whitespace();
        let (n, name) = match (it.next(), it.next()) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        let hit = if exact {
            name == needle
        } else {
            name.starts_with(needle)
        };
        if hit {
            if let Ok(v) = n.parse::<u64>() {
                total += v;
            }
        }
    }
    total
}

fn area_at(
    sources: &[String],
    top: &str,
    param: &str,
    value: i64,
    fixed: &[String],
    nodsp: bool,
) -> Result<u64> {
    let mut script = format!("read_verilog -sv {}\n", sources.join(" "));
    script.push_str(&format!("chparam -set {param} {value} {top}\n"));
    for f in fixed {
        let (k, v) = f
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("--fixed wants NAME=VALUE, got {f:?}"))?;
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
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !log.contains("Printing statistics") {
        bail!(
            "synthesis did not reach a stat block at {param}={value}. Last lines:\n{}",
            log.lines().rev().take(5).collect::<Vec<_>>().join("\n")
        );
    }
    Ok(["LUT1", "LUT2", "LUT3", "LUT4", "LUT5", "LUT6"]
        .iter()
        .map(|n| count(&log, n, true))
        .sum())
}

/// What the trend says about the design, in words rather than a number.
///
/// Constant area across a scaling parameter is the fold signature; growth far
/// steeper than the parameter is the wrong-memory-inference signature. Both are
/// reported as questions, because a design can legitimately be flat (the
/// parameter is unused on this path) or steep (it really is quadratic).
fn verdict(values: &[i64], areas: &[u64]) -> Vec<String> {
    let mut notes = Vec::new();
    if areas.windows(2).all(|w| w[0] == w[1]) && values.len() > 1 {
        notes.push(
            "AREA DOES NOT MOVE. A parameter the design scales with should change \
             the cell count. This is the shape a fold leaves: a never-written \
             memory, an unread output, a pruned branch."
                .to_string(),
        );
    }
    for i in 1..values.len() {
        if values[i - 1] <= 0 || areas[i - 1] == 0 {
            continue;
        }
        let pratio = values[i] as f64 / values[i - 1] as f64;
        let aratio = areas[i] as f64 / areas[i - 1] as f64;
        if aratio > pratio * 2.5 {
            notes.push(format!(
                "AREA GREW {:.1}x WHILE {} GREW {:.1}x, from {} to {}. Steeper than \
                 the parameter by more than 2.5x usually means a structure was \
                 inferred that should not have been -- N read ports instead of a \
                 memory, or an unrolled loop instead of a shared resource.",
                aratio,
                "the parameter",
                pratio,
                values[i - 1],
                values[i]
            ));
        }
    }
    notes
}

fn sweep(
    sources: &[String],
    top: &str,
    param: &str,
    values: &[i64],
    fixed: &[String],
    nodsp: bool,
) -> Result<()> {
    let version = Command::new("yosys")
        .arg("-V")
        .output()
        .context("yosys is not installed or not on PATH")?;
    let version = String::from_utf8_lossy(&version.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    let mut areas = Vec::new();
    println!("{top}, sweeping {param}\n");
    println!("  {:>10} {:>10} {:>10}", param, "LUT", "per unit");
    for &v in values {
        let a = area_at(sources, top, param, v, fixed, nodsp)?;
        let per = if v > 0 { a as f64 / v as f64 } else { 0.0 };
        println!("  {v:>10} {a:>10} {per:>10.1}");
        areas.push(a);
    }
    println!("\n  instrument: {version}");

    let notes = verdict(values, &areas);
    println!();
    if notes.is_empty() {
        println!("The area responds to {param} and does so no more steeply than {param}.");
        println!("That is not proof the design is right -- it is the absence of the two");
        println!("shapes a synthesis fold leaves behind.");
    } else {
        for n in &notes {
            println!("{n}\n");
        }
        println!("Read the netlist before trusting any number in this table.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fold that reported 12 logic cells for a 64-tap layer left exactly
    /// this shape: the parameter moved and the area did not.
    #[test]
    fn flat_area_across_a_scaling_parameter_is_flagged() {
        let notes = verdict(&[16, 32, 64], &[12, 12, 12]);
        assert!(!notes.is_empty(), "a flat sweep must be reported");
        assert!(notes[0].contains("DOES NOT MOVE"));
    }

    /// And the one that reported 23052 cells left the other shape: area far
    /// steeper than the parameter.
    #[test]
    fn area_far_steeper_than_the_parameter_is_flagged() {
        let notes = verdict(&[16, 32], &[1000, 23052]);
        assert!(notes.iter().any(|n| n.contains("AREA GREW")));
    }

    /// Growth in step with the parameter is the ordinary case and must stay
    /// quiet, or the command becomes noise and stops being read.
    #[test]
    fn proportional_growth_is_not_flagged() {
        assert!(verdict(&[16, 32, 64], &[1746, 2994, 5502]).is_empty());
    }
}
