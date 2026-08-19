//! `tri rtl` — the structural check t27.ai offers, run locally.
//!
//! This is the same five checks the reusable workflow performs
//! (`gHashTag/trinity .github/workflows/rtl-check.yml`), in one command, so a
//! design can be checked without pushing anything and a CI number can be
//! reproduced on demand.
//!
//! Two things it does that the hand-typed yosys invocation kept getting wrong:
//!
//! 1. **It names the instrument.** The cell count depends on which yosys ran.
//!    Measured on one design with this exact script, 0.33 reports 45 cells
//!    where 0.65 reports 49 — while wires and flip-flops agree exactly. A cell
//!    count without its version is not reproducible, so the version is printed
//!    beside it rather than left in the shell history.
//!
//! 2. **It counts its own verdicts.** Five checks must emit five lines. A
//!    check that silently does not run looks exactly like one that passed, and
//!    that has happened here before — so the count is asserted, not assumed.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum RtlCmd {
    /// Run the five structural checks on a design directory.
    Check {
        /// Directory holding info.yaml and src/ (defaults to the current one).
        #[arg(default_value = ".")]
        path: String,
        /// Top module. Read from info.yaml when omitted.
        #[arg(long)]
        top: Option<String>,
        /// Emit JSON instead of the human report.
        #[arg(long)]
        json: bool,
        /// Fail if the flip-flop count differs from this.
        #[arg(long)]
        expect_flops: Option<u64>,
    },
}

struct Verdict {
    name: &'static str,
    pass: bool,
    detail: String,
    command: String,
}

pub fn run(cmd: &RtlCmd) -> Result<()> {
    match cmd {
        RtlCmd::Check {
            path,
            top,
            json,
            expect_flops,
        } => check(Path::new(path), top.as_deref(), *json, *expect_flops),
    }
}

/// yosys prints its banner on stdout for `-V`; an absent tool is a hard error
/// rather than an empty string, because an empty version silently becomes an
/// unlabelled number in the report.
fn yosys_version() -> Result<String> {
    let out = Command::new("yosys")
        .arg("-V")
        .output()
        .context("yosys is not installed or not on PATH")?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    let v = s
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");
    if v.trim().is_empty() {
        bail!("yosys -V printed nothing; refusing to report numbers from an unnamed tool");
    }
    Ok(v)
}

/// The `source_files:` list from info.yaml, as paths under `<dir>/src/`.
///
/// Deliberately a small hand parser rather than a YAML dependency: the block is
/// a flat list of quoted scalars, and the failure mode that matters — a file
/// declared but absent — is about the filesystem, not about YAML.
fn declared_sources(dir: &Path) -> Result<Vec<PathBuf>> {
    let info = dir.join("info.yaml");
    let text = std::fs::read_to_string(&info)
        .with_context(|| format!("no info.yaml at {}", info.display()))?;
    let mut out = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("source_files:") {
            in_block = true;
            continue;
        }
        if in_block {
            if let Some(rest) = t.strip_prefix("- ") {
                let f = rest.trim().trim_matches('"').trim_matches('\'');
                if !f.is_empty() {
                    out.push(dir.join("src").join(f));
                }
                continue;
            }
            if !t.is_empty() && !t.starts_with('#') {
                break;
            }
        }
    }
    if out.is_empty() {
        bail!("info.yaml declares no source_files");
    }
    Ok(out)
}

fn top_from_info(dir: &Path) -> Result<String> {
    let text = std::fs::read_to_string(dir.join("info.yaml"))?;
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("top_module:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Ok(v.to_string());
            }
        }
    }
    bail!("info.yaml has no top_module and --top was not given")
}

/// Any cell type containing "dff" is a flop, and yosys prints the count in
/// either field order depending on version — so take whichever field is a
/// number rather than assuming a column.
fn count_flops(stat: &str) -> u64 {
    let mut total = 0u64;
    for line in stat.lines() {
        let low = line.to_lowercase();
        if !low.contains("dff") {
            continue;
        }
        let mut fields = line.split_whitespace();
        let a = fields.next().unwrap_or("");
        let b = fields.next().unwrap_or("");
        if let Ok(n) = a.parse::<u64>() {
            total += n;
        } else if let Ok(n) = b.parse::<u64>() {
            total += n;
        }
    }
    total
}

fn count_named(stat: &str, what: &str) -> Option<u64> {
    for line in stat.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_suffix(what) {
            if let Ok(n) = rest.trim().parse::<u64>() {
                return Some(n);
            }
        }
        if let Some(rest) = t.strip_prefix(&format!("Number of {what}:")) {
            if let Ok(n) = rest.trim().parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

fn check(dir: &Path, top: Option<&str>, json: bool, expect_flops: Option<u64>) -> Result<()> {
    let version = yosys_version()?;
    let top = match top {
        Some(t) => t.to_string(),
        None => top_from_info(dir)?,
    };
    let sources = declared_sources(dir)?;
    let mut v: Vec<Verdict> = Vec::new();

    // 1. Every declared file is present. Stated on the way through, not only on
    //    failure: a check that speaks only when it fails leaves the reader
    //    unable to tell "checked and passed" from "never ran".
    let missing: Vec<String> = sources
        .iter()
        .filter(|p| !p.is_file())
        .map(|p| p.display().to_string())
        .collect();
    v.push(Verdict {
        name: "sources resolve",
        pass: missing.is_empty(),
        detail: if missing.is_empty() {
            format!("every file info.yaml declares is present ({} of them)", sources.len())
        } else {
            format!("declared but absent: {}", missing.join(", "))
        },
        command: "info.yaml source_files -> src/".to_string(),
    });

    // 2/3/4/5. One yosys pass. flatten before stat, because per-module counts
    // under-report anything hierarchical.
    let read = sources
        .iter()
        .map(|p| format!("read_verilog -sv {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");
    let script = format!(
        "{read}\nhierarchy -top {top}\nproc; opt; fsm; opt; memory; opt\ntechmap; opt\nflatten; opt\nstat\nselect -assert-none t:$_DLATCH_* t:$_DLATCHSR_*\n"
    );
    let sp = dir.join(".tri-rtl-check.ys");
    std::fs::write(&sp, &script)?;
    let out = Command::new("yosys")
        .arg("-s")
        .arg(&sp)
        .output()
        .context("failed to run yosys")?;
    let _ = std::fs::remove_file(&sp);
    let stat = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let ok = out.status.success();
    let cmd = format!("yosys -s <script>  # top {top}");

    v.push(Verdict {
        name: "elaborates",
        pass: ok || !stat.contains("is not defined"),
        detail: if stat.contains("is not defined") {
            "a module it instantiates is not defined in the files given".into()
        } else {
            "every module it instantiates is defined in the files given".into()
        },
        command: cmd.clone(),
    });
    v.push(Verdict {
        name: "no latch inferred",
        pass: ok,
        detail: if ok {
            "no latch inferred by this flow".into()
        } else {
            "a latch was inferred, or synthesis stopped".into()
        },
        command: "select -assert-none t:$_DLATCH_*".into(),
    });

    let cells = count_named(&stat, "cells");
    let wires = count_named(&stat, "wires");
    v.push(Verdict {
        name: "synthesises",
        pass: cells.is_some(),
        detail: match (cells, wires) {
            (Some(c), Some(w)) => format!("{c} cells, {w} wires, flattened — {version}"),
            (Some(c), None) => format!("{c} cells, flattened — {version}"),
            _ => "synthesis produced no cell count".into(),
        },
        command: cmd.clone(),
    });

    let flops = count_flops(&stat);
    let flops_ok = expect_flops.map(|e| e == flops).unwrap_or(flops > 0);
    v.push(Verdict {
        name: "has clocked logic",
        pass: flops_ok,
        detail: match expect_flops {
            Some(e) if e != flops => format!("{flops} flip-flops, asserted {e}"),
            _ => format!(
                "{flops} flip-flops, so a clock frequency is a meaningful question"
            ),
        },
        command: "stat, any cell type containing \"dff\"".into(),
    });

    // The report is counted, not trusted. Five checks, five lines: a verdict
    // that never got pushed would otherwise leave a shorter report that reads
    // exactly like a complete one.
    if v.len() != 5 {
        bail!("the report carries {} verdicts where five were expected", v.len());
    }

    if json {
        let arr: Vec<serde_json::Value> = v
            .iter()
            .map(|x| {
                serde_json::json!({
                    "name": x.name,
                    "status": if x.pass { "PASS" } else { "FAIL" },
                    "detail": x.detail,
                    "command": x.command,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "top": top,
                "yosys": version,
                "checks": arr,
            }))?
        );
    } else {
        println!("RTL structural check — {top}");
        for x in &v {
            println!(
                "  {} {} — {}",
                if x.pass { "PASS" } else { "FAIL" },
                x.name,
                x.detail
            );
            println!("       $ {}", x.command);
        }
        println!();
        println!("What this does not establish: nothing above compares the design");
        println!("against a specification. No frequency is claimed — the flip-flop");
        println!("count says the question is meaningful, not what the answer is.");
    }

    if v.iter().any(|x| !x.pass) {
        bail!("{} of 5 checks failed", v.iter().filter(|x| !x.pass).count());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checked against real yosys output in BOTH field orders, because an
    // earlier version of this counter was checked against fixtures typed out
    // in the order the author assumed — so the pattern and the fixture shared
    // one wrong assumption and agreed with each other.
    #[test]
    fn flops_counted_in_either_field_order() {
        let a = "     17 $_DFF_P_\n      4 $_NAND_\n";
        let b = "  $_DFF_P_ 17\n  $_NAND_ 4\n";
        assert_eq!(count_flops(a), 17);
        assert_eq!(count_flops(b), 17);
    }

    #[test]
    fn dff_variants_with_digits_are_counted() {
        // `[A-Z_]*` after DFF cannot match `$_DFFE_PN0P_`; it contains digits.
        let s = "      9 $_DFFE_PN0P_\n      8 $_SDFFCE_PP0P_\n";
        assert_eq!(count_flops(s), 17);
    }

    #[test]
    fn a_stat_with_no_flops_counts_zero_not_garbage() {
        assert_eq!(count_flops("     45 $_NAND_\n     12 $_XOR_\n"), 0);
    }

    #[test]
    fn cells_and_wires_are_read_from_the_stat_block() {
        let s = "   Number of wires:                 57\n   Number of cells:                 49\n";
        assert_eq!(count_named(s, "cells"), Some(49));
        assert_eq!(count_named(s, "wires"), Some(57));
    }
}
