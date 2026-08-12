//! Persistent SPI flash programmer for QMTech Wukong V1 (XC7A100T).
//!
//! Now a thin wrapper around the in-tree `dlc10` crate — no shell-out to
//! `openFPGALoader`, no external dependencies on the host beyond `libusb`.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use dlc10::{Dlc10, FlashOpts};

/// Permanently program the QMTech Wukong V1 SPI flash so the FPGA boots
/// from flash on every power-up. After success, the JTAG cable can be
/// physically removed; the bitstream survives power-off forever.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the .bit file to flash.
    #[arg(default_value = "fpga/vsa/gf16_heartbeat_top.bit")]
    bit: PathBuf,

    /// Expected JTAG IDCODE (lowercase hex, no 0x). XC7A100T = 13631093.
    #[arg(long, default_value = "13631093")]
    expected_idcode: String,

    /// Skip cable detection (useful for unusual setups).
    #[arg(long)]
    skip_detect: bool,

    /// Skip read-back verification.
    #[arg(long)]
    no_verify: bool,

    /// Print intent and exit.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("=== Step 1/4: pre-flight checks ===");
    if !cli.bit.is_file() {
        bail!("bitstream not found: {}", cli.bit.display());
    }
    let bit_size = std::fs::metadata(&cli.bit)?.len();
    eprintln!(
        "bitstream:      {} ({:.1} MiB)",
        cli.bit.display(),
        bit_size as f64 / 1024.0 / 1024.0
    );

    if cli.dry_run {
        eprintln!("[dry-run] would call dlc10::Dlc10::program_flash on {}", cli.bit.display());
        return Ok(());
    }

    let bytes = std::fs::read(&cli.bit)
        .with_context(|| format!("read {}", cli.bit.display()))?;

    eprintln!("\n=== Step 2/4: detect cable + IDCODE ===");
    let mut cable = Dlc10::open().context("open DLC10 cable (is it plugged in?)")?;
    if cli.skip_detect {
        eprintln!("[skipped] (--skip-detect)");
    } else {
        let id = cable.read_idcode()?;
        let want = u32::from_str_radix(&cli.expected_idcode, 16)
            .with_context(|| format!("parse expected_idcode={}", cli.expected_idcode))?;
        if id != want {
            bail!(
                "IDCODE mismatch: got 0x{:08X}, expected 0x{:08X}",
                id,
                want
            );
        }
        eprintln!("IDCODE 0x{:08X} confirmed.", id);
    }

    eprintln!("\n=== Step 3/4: write bitstream to SPI flash (~60s) ===");
    let total = bytes.len() as u64;
    // Prop. 163: `FlashOpts` gained `bitswap` and `no_jprogram` and this call
    // site was never updated, so this crate stopped compiling -- and no
    // workflow builds it, so nothing said so. `..Default::default()` is what
    // would have prevented the drift: bitswap defaults to true, matching
    // Vivado's `write_cfgmem`, which is what this path wants.
    let opts = FlashOpts {
        verify: !cli.no_verify,
        progress: Some(Box::new(move |w, t| {
            if w == t || w % (1 << 18) < 256 {
                eprintln!("  {} / {} ({}%)", w, total, 100 * w / total.max(1));
            }
        })),
        ..Default::default()
    };
    cable.program_flash(&bytes, opts)?;

    eprintln!("\n=== Step 4/4: success ===");
    eprintln!("Bitstream is now PERMANENT in M25P/N25Q SPI flash.");
    eprintln!("FPGA will auto-load it within ~100 ms after every power-on.");
    eprintln!();
    eprintln!("Next:");
    eprintln!("  1. Physically unplug the JTAG cable (no longer needed).");
    eprintln!("  2. Power-cycle the FPGA board.");
    eprintln!("  3. D5/D6 (R23/T23) must blink the 3-phase phi heartbeat");
    eprintln!("     without any cable connected — that proves flash is alive.");
    cable.close();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::parse_from(["flash-spi"]);
        assert_eq!(cli.expected_idcode, "13631093");
        assert_eq!(cli.bit, PathBuf::from("fpga/vsa/gf16_heartbeat_top.bit"));
        assert!(!cli.no_verify);
        assert!(!cli.skip_detect);
    }

    #[test]
    fn cli_overrides_work() {
        let cli = Cli::parse_from([
            "flash-spi",
            "--expected-idcode",
            "deadbeef",
            "--skip-detect",
            "--no-verify",
            "--dry-run",
            "some.bit",
        ]);
        assert_eq!(cli.expected_idcode, "deadbeef");
        assert!(cli.skip_detect);
        assert!(cli.no_verify);
        assert!(cli.dry_run);
        assert_eq!(cli.bit, PathBuf::from("some.bit"));
    }
}
