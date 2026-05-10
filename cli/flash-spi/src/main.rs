//! Persistent SPI flash programmer for QMTech Wukong V1 (XC7A100T).
//!
//! The grain agents missed for 3 months: `--write-flash`, NOT `--program`.
//!
//! Wraps `openFPGALoader --cable <CABLE> --write-flash <BIT> --verify`,
//! with pre-flight checks for bitstream existence, openFPGALoader presence,
//! and cable detection (IDCODE match).

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

/// Permanently program the QMTech Wukong V1 SPI flash so the FPGA boots
/// from flash on every power-up. After success, the JTAG cable can be
/// physically removed; the bitstream survives power-off forever.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the .bit file to flash.
    #[arg(default_value = "fpga/vsa/gf16_heartbeat_top.bit")]
    bit: PathBuf,

    /// JTAG cable name passed to openFPGALoader (e.g. dlc10, ft232, digilent).
    #[arg(long, env = "CABLE", default_value = "dlc10")]
    cable: String,

    /// Expected JTAG IDCODE (lowercase hex, no 0x). XC7A100T = 13631093.
    #[arg(long, default_value = "13631093")]
    expected_idcode: String,

    /// Skip cable detection (useful for dry-run or unusual setups).
    #[arg(long)]
    skip_detect: bool,

    /// Print the openFPGALoader command instead of running it.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("=== Step 1/4: pre-flight checks ===");
    let ofl = which::which("openFPGALoader")
        .context("openFPGALoader not found in PATH (try `brew install openfpgaloader`)")?;
    eprintln!("openFPGALoader: {}", ofl.display());

    if !cli.bit.is_file() {
        bail!("bitstream not found: {}", cli.bit.display());
    }
    let bit_size = std::fs::metadata(&cli.bit)?.len();
    eprintln!(
        "bitstream:      {} ({:.1} MiB)",
        cli.bit.display(),
        bit_size as f64 / 1024.0 / 1024.0
    );

    if !cli.skip_detect {
        eprintln!("\n=== Step 2/4: detect cable + IDCODE ===");
        let detect = run_capture(
            &ofl,
            &["--cable", &cli.cable, "--detect"],
            "openFPGALoader --detect",
        )?;
        let combined = format!("{}{}", detect.stdout, detect.stderr);

        if !detect.status.success() {
            bail!(
                "cable detection failed (exit {:?}). Is `{}` plugged in and JTAG ribbon connected?\n\nstdout/stderr:\n{}",
                detect.status.code(),
                cli.cable,
                combined.trim()
            );
        }

        if !combined.to_lowercase().contains(&cli.expected_idcode.to_lowercase()) {
            bail!(
                "IDCODE 0x{} not seen — wrong board or bad JTAG wiring.\n\nopenFPGALoader output:\n{}",
                cli.expected_idcode,
                combined.trim()
            );
        }
        eprintln!("IDCODE 0x{} confirmed.", cli.expected_idcode);
    } else {
        eprintln!("\n=== Step 2/4: detect cable + IDCODE [SKIPPED] ===");
    }

    eprintln!("\n=== Step 3/4: write bitstream to SPI flash (~60s) ===");
    let bit_str = cli
        .bit
        .to_str()
        .ok_or_else(|| anyhow!("non-UTF-8 path: {}", cli.bit.display()))?;
    let args = ["--cable", &cli.cable, "--write-flash", bit_str, "--verify"];

    if cli.dry_run {
        eprintln!("[dry-run] {} {}", ofl.display(), args.join(" "));
        return Ok(());
    }

    let status = Command::new(&ofl)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn openFPGALoader --write-flash")?;
    if !status.success() {
        bail!(
            "openFPGALoader --write-flash failed (exit {:?})",
            status.code()
        );
    }

    eprintln!("\n=== Step 4/4: success ===");
    eprintln!("Bitstream is now PERMANENT in M25P/N25Q SPI flash.");
    eprintln!("FPGA will auto-load it within ~100 ms after every power-on.");
    eprintln!();
    eprintln!("Next:");
    eprintln!("  1. Physically unplug the JTAG cable (no longer needed).");
    eprintln!("  2. Power-cycle the FPGA board.");
    eprintln!("  3. D5/D6 (R23/T23) must blink the 3-phase phi heartbeat");
    eprintln!("     without any cable connected — that proves flash is alive.");
    Ok(())
}

struct Captured {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_capture(prog: &std::path::Path, args: &[&str], label: &str) -> Result<Captured> {
    let out = Command::new(prog)
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn {}", label))?;
    Ok(Captured {
        status: out.status,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idcode_match_case_insensitive() {
        let out = "IDCODE 0x13631093 (XC7A100T)\n";
        assert!(out.to_lowercase().contains("13631093"));
    }

    #[test]
    fn idcode_mismatch_detected() {
        let out = "IDCODE 0xFFFFFFFF\n";
        assert!(!out.to_lowercase().contains("13631093"));
    }

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["flash-spi"]);
        assert_eq!(cli.cable, "dlc10");
        assert_eq!(cli.expected_idcode, "13631093");
        assert_eq!(cli.bit, PathBuf::from("fpga/vsa/gf16_heartbeat_top.bit"));
    }

    #[test]
    fn cli_overrides_work() {
        let cli = Cli::parse_from([
            "flash-spi",
            "--cable",
            "ft232",
            "--expected-idcode",
            "deadbeef",
            "--skip-detect",
            "--dry-run",
            "some.bit",
        ]);
        assert_eq!(cli.cable, "ft232");
        assert_eq!(cli.expected_idcode, "deadbeef");
        assert!(cli.skip_detect);
        assert!(cli.dry_run);
        assert_eq!(cli.bit, PathBuf::from("some.bit"));
    }
}
