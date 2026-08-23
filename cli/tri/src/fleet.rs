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
    // -r -c IOUSBHostDevice lists USB DEVICES rather than the whole tree, which
    // matters because a device's interfaces repeat its idVendor and serial: a
    // whole-tree walk counts one board three times.
    let out = Command::new("ioreg")
        .args(["-r", "-c", "IOUSBHostDevice", "-l", "-w0"])
        .output()
        .context("ioreg is not available")?;
    let text = String::from_utf8_lossy(&out.stdout);

    // IDENTIFY BY VENDOR, NOT BY LABEL. This probe used to match device names
    // against "ft232", "ftdi", "usb serial" and "jtag" -- and reported ZERO
    // bridges on a bench with three AX7203 boards plugged in, because macOS
    // calls them "Digilent USB Device". The board vendor brands the string;
    // only the vendor ID says what the chip is. 1027 == 0x0403 == FTDI, which
    // is the bridge every one of these carries.
    //
    // The old spelling failed in the direction that costs most: it said NO
    // HARDWARE while hardware was attached, which is the sentence this whole
    // command exists to get right.
    const FTDI: &str = "1027";

    let mut found: Vec<Bridge> = Vec::new();
    let mut name: Option<String> = None;
    let mut is_ftdi = false;
    let mut serial: Option<String> = None;

    // A device node opens with `+-o <name>@<addr>` and its properties follow
    // until the next node, so the vendor and the serial are collected across
    // that block and committed when the block ends.
    let mut flush = |name: &mut Option<String>,
                     is_ftdi: &mut bool,
                     serial: &mut Option<String>,
                     found: &mut Vec<Bridge>| {
        if *is_ftdi {
            if let Some(n) = name.clone() {
                found.push(Bridge {
                    kind: n,
                    serial: serial.clone(),
                });
            }
        }
        *name = None;
        *is_ftdi = false;
        *serial = None;
    };

    for line in text.lines() {
        // ioreg draws the tree with `|` and spaces, so a NESTED node reads
        // `| | +-o Name...`. Trimming whitespace alone leaves the pipes, and
        // `starts_with("+-o")` then matched only TOP-LEVEL nodes -- which is
        // every controller and hub, and no board: an FT232H on a hub port is
        // always nested. That is why this probe reported an empty bus with
        // three AX7203s plugged in.
        let l = line.trim_start_matches(|c| c == '|' || c == ' ').trim();
        if l.starts_with("+-o") {
            flush(&mut name, &mut is_ftdi, &mut serial, &mut found);
            // Only a DEVICE node counts. Its interfaces carry the same vendor
            // and serial, and an FT232H presents several -- so matching every
            // node reported one board as three.
            if l.contains("class IOUSBHostDevice") {
                let n = l.trim_start_matches("+-o").trim();
                name = Some(n.split('@').next().unwrap_or(n).trim().to_string());
            }
        } else if l.contains("\"idVendor\"") {
            if let Some(v) = l.split('=').nth(1) {
                if v.trim() == FTDI {
                    is_ftdi = true;
                }
            }
        } else if l.contains("\"USB Serial Number\"") {
            if let Some(v) = l.split('=').nth(1) {
                serial = Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    flush(&mut name, &mut is_ftdi, &mut serial, &mut found);
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
