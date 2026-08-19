//! `tri fleet` — is the hardware this plan assumes actually attached?
//!
//! Written after an audit found three present-tense capability claims in the
//! project's own notes — "3-board inference cluster PROVEN", "all three flash
//! from software without replugging", "on-chip training PROVEN" — while not a
//! single board was on the bus. Each was true when measured. None was true
//! that day.
//!
//! A measurement stays true because it happened; a capability quietly becomes
//! false when the environment regresses, and nothing announces it. The notes
//! even said "a configured fleet is a perishable measurement" — the instinct
//! was recorded and never turned into a check. This is that check.
//!
//! It refuses to guess: an empty bus is reported as an empty bus, with the
//! sentence to send the owner, not as a problem to code around.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::Command;

#[derive(Subcommand)]
pub enum FleetCmd {
    /// Scan the USB bus and say plainly what hardware is present.
    Scan {
        /// How many boards the plan expects. Non-zero exit if fewer are found.
        #[arg(long)]
        expect: Option<usize>,
    },
    /// Check the environments a claim depends on before repeating the claim.
    ///
    /// The bus is one environment; a deployed site is another. Both go stale
    /// the same way — the note says "works" because it worked, and nothing
    /// announces the regression. This checks each named URL and the bus, and
    /// reports which capability claims are currently unverifiable.
    Asof {
        /// URLs the claim depends on, repeatable. Checked with a HEAD request.
        #[arg(long = "url")]
        urls: Vec<String>,
        /// Also require hardware on the bus.
        #[arg(long)]
        needs_hardware: bool,
        /// Read the claims from a declaration instead of the command line.
        ///
        /// Remembering which URLs back which claim is exactly the step that
        /// gets skipped, so the claims live in the repository next to the
        /// code they describe. Format:
        ///
        /// {"claims":[{"name":"...","urls":["..."],"needs_hardware":false}]}
        #[arg(long)]
        from: Option<PathBuf>,
    },
}

#[derive(serde::Deserialize)]
struct Claim {
    name: String,
    #[serde(default)]
    urls: Vec<String>,
    #[serde(default)]
    needs_hardware: bool,
}

#[derive(serde::Deserialize)]
struct Claims {
    claims: Vec<Claim>,
}

pub fn run(cmd: &FleetCmd) -> Result<()> {
    match cmd {
        FleetCmd::Scan { expect } => scan(*expect),
        FleetCmd::Asof {
            urls,
            needs_hardware,
            from,
        } => match from {
            Some(path) => asof_declared(path),
            None => asof(urls, *needs_hardware),
        },
    }
}

/// Check every claim in a declaration and report per claim, because "is the
/// project fine" has no answer — each claim rests on its own environments and
/// they fail independently.
fn asof_declared(path: &PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let decl: Claims =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;

    let mut stale: Vec<String> = Vec::new();
    for c in &decl.claims {
        let mut missing: Vec<String> = Vec::new();
        for u in &c.urls {
            let (ok, code) = head(u);
            if !ok {
                missing.push(format!("{u} (HTTP {code})"));
            }
        }
        if c.needs_hardware {
            match probe_usb() {
                Ok(b) if !b.is_empty() => {}
                Ok(_) => missing.push("hardware (bus empty)".to_string()),
                Err(_) => missing.push("hardware (cannot tell)".to_string()),
            }
        }
        if missing.is_empty() {
            println!("SAYABLE  {}", c.name);
        } else {
            println!("STALE    {}", c.name);
            for m in &missing {
                println!("           needs {m}");
            }
            stale.push(c.name.clone());
        }
    }

    println!();
    if stale.is_empty() {
        println!(
            "VERDICT: all {} claim(s) rest on reachable environments.",
            decl.claims.len()
        );
        return Ok(());
    }
    println!(
        "VERDICT: {} of {} claim(s) may NOT be stated in the present tense today.",
        stale.len(),
        decl.claims.len()
    );
    anyhow::bail!("{} stale claim(s)", stale.len())
}

/// HEAD one URL. A timeout is a failure to verify, not a failure of the site —
/// the two are reported differently because only one of them is the owner's
/// problem.
fn head(url: &str) -> (bool, String) {
    let out = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-m",
            "15",
            "-L",
            "-I",
            url,
        ])
        .output();
    match out {
        Ok(o) => {
            let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let ok = code.starts_with('2') || code.starts_with('3');
            (ok, code)
        }
        Err(e) => (false, format!("curl failed: {e}")),
    }
}

fn asof(urls: &[String], needs_hardware: bool) -> Result<()> {
    let mut unverifiable: Vec<String> = Vec::new();

    for u in urls {
        let (ok, code) = head(u);
        println!("{:<7} {u}", if ok { "live" } else { "DOWN" });
        if !ok {
            unverifiable.push(format!("{u} (HTTP {code})"));
        }
    }

    if needs_hardware {
        match probe_usb() {
            Ok(b) if !b.is_empty() => println!("{:<7} {} USB bridge(s)", "live", b.len()),
            Ok(_) => {
                println!("{:<7} no board on the bus", "DOWN");
                unverifiable.push("hardware (bus empty)".to_string());
            }
            Err(e) => {
                println!("{:<7} {e}", "UNKNOWN");
                unverifiable.push("hardware (cannot tell)".to_string());
            }
        }
    }

    println!();
    if unverifiable.is_empty() {
        println!("VERDICT: every environment this claim depends on is reachable.");
        println!("The claim can be repeated in the present tense today.");
        return Ok(());
    }

    println!(
        "VERDICT: {} environment(s) unreachable:",
        unverifiable.len()
    );
    for u in &unverifiable {
        println!("  {u}");
    }
    println!();
    println!("Any note asserting this capability in the present tense is a MEASUREMENT");
    println!("of the past, not a capability of today. Re-verify before repeating it.");
    anyhow::bail!("{} environment(s) unverifiable", unverifiable.len())
}

/// One JTAG/UART bridge as the bus reports it.
struct Bridge {
    kind: String,
    serial: Option<String>,
}

/// Read the USB tree. macOS only — on anything else this says so rather than
/// reporting an empty fleet, because "no boards" and "cannot tell" are
/// different answers and only one of them is safe to act on.
fn probe_usb() -> Result<Vec<Bridge>> {
    if !cfg!(target_os = "macos") {
        anyhow::bail!("bus probe is implemented for macOS only; cannot tell what is attached");
    }
    let out = Command::new("ioreg")
        .args(["-p", "IOUSB", "-l", "-w0"])
        .output()
        .context("ioreg is not available")?;
    let text = String::from_utf8_lossy(&out.stdout);

    let mut found = Vec::new();
    let mut pending: Option<String> = None;
    for line in text.lines() {
        let l = line.trim();
        // Device nodes appear as `+-o <name>@<addr>`; the serial follows in
        // that node's property block a few lines later.
        if l.starts_with("+-o") {
            let name = l.trim_start_matches("+-o").trim();
            let lower = name.to_ascii_lowercase();
            pending = if lower.contains("ft232")
                || lower.contains("ftdi")
                || lower.contains("usb serial")
                || lower.contains("jtag")
            {
                Some(name.split('@').next().unwrap_or(name).trim().to_string())
            } else {
                None
            };
            if let Some(kind) = pending.clone() {
                found.push(Bridge { kind, serial: None });
            }
        } else if pending.is_some() && l.contains("\"USB Serial Number\"") {
            if let Some(v) = l.split('=').nth(1) {
                if let Some(b) = found.last_mut() {
                    b.serial = Some(v.trim().trim_matches('"').to_string());
                }
            }
            pending = None;
        }
    }
    Ok(found)
}

/// Serial device nodes, which is what a UART console actually needs. Bluetooth
/// and the debug console are always present and are not hardware.
fn probe_tty() -> Vec<String> {
    let mut nodes = Vec::new();
    if let Ok(dir) = std::fs::read_dir("/dev") {
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if (name.starts_with("cu.") || name.starts_with("tty."))
                && !name.contains("Bluetooth")
                && !name.contains("debug-console")
            {
                nodes.push(name);
            }
        }
    }
    nodes.sort();
    nodes
}

fn scan(expect: Option<usize>) -> Result<()> {
    let bridges = probe_usb()?;
    let ttys = probe_tty();

    println!("USB JTAG/UART bridges: {}", bridges.len());
    for b in &bridges {
        match &b.serial {
            Some(s) => println!("  {} (serial {s})", b.kind),
            None => println!("  {}", b.kind),
        }
    }
    println!("serial device nodes: {}", ttys.len());
    for t in &ttys {
        println!("  /dev/{t}");
    }
    println!();

    if bridges.is_empty() && ttys.is_empty() {
        println!("VERDICT: no hardware attached.");
        println!();
        println!("Any 'proven on hardware' note in this project is a MEASUREMENT of the past,");
        println!("not a capability of today. Do not write code for a board that is not there,");
        println!("and do not report readiness to flash. Tell the owner instead:");
        println!();
        println!("  \"No board is on the bus — the fleet needs to be plugged in before");
        println!("   anything hardware-side can run. This needs hands, not code.\"");
    } else {
        println!(
            "VERDICT: {} bridge(s), {} serial node(s) present.",
            bridges.len(),
            ttys.len()
        );
    }

    if let Some(n) = expect {
        if bridges.len() < n {
            anyhow::bail!(
                "expected {n} board(s), found {} — refusing to report a fleet that is not there",
                bridges.len()
            );
        }
    }
    Ok(())
}
