//! `dlc10` CLI: read IDCODE, program SRAM, program SPI flash, read JEDEC ID.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dlc10::{Dlc10, FlashOpts};

#[derive(Parser, Debug)]
#[command(version, about = "Pure-Rust driver for Xilinx DLC10 (Platform Cable USB II)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Read and print the JTAG IDCODE.
    Idcode,
    /// Program FPGA SRAM (volatile — lost on power-cycle).
    Sram {
        bit: PathBuf,
    },
    /// Program the on-board SPI flash (non-volatile).
    Flash {
        bit: PathBuf,
        #[arg(long, default_value_t = true)]
        verify: bool,
    },
    /// Read the SPI flash JEDEC ID via the JTAG-to-SPI bridge.
    FlashId,
    /// Read the configuration STATUS register.
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cable = Dlc10::open().context("open DLC10")?;
    match cli.cmd {
        Cmd::Idcode => {
            let id = cable.read_idcode()?;
            println!("IDCODE: 0x{:08X}", id);
            if id != 0x13631093 {
                eprintln!("note: expected 0x13631093 (XC7A100T), got 0x{:08X}", id);
            }
        }
        Cmd::Sram { bit } => {
            let bytes = std::fs::read(&bit)
                .with_context(|| format!("read {}", bit.display()))?;
            let status = cable.program_sram(&bytes)?;
            println!("STATUS: 0x{:08X}", status);
            let done = (status >> 14) & 1;
            println!(
                "DONE bit (status[14]): {}",
                if done == 1 { "HIGH (configured)" } else { "LOW (not configured)" }
            );
        }
        Cmd::Flash { bit, verify } => {
            let bytes = std::fs::read(&bit)
                .with_context(|| format!("read {}", bit.display()))?;
            let total = bytes.len() as u64;
            let opts = FlashOpts {
                verify,
                progress: Some(Box::new(move |w, t| {
                    if w == t || w % (1 << 18) < 256 {
                        eprintln!("  {} / {} ({}%)", w, total, 100 * w / total.max(1));
                    }
                })),
            };
            cable.program_flash(&bytes, opts)?;
            eprintln!("Flash write OK.");
        }
        Cmd::FlashId => {
            let id = cable.read_flash_id()?;
            println!("JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);
        }
        Cmd::Status => {
            let s = cable.read_status()?;
            println!("STATUS: 0x{:08X}", s);
        }
    }
    cable.close();
    Ok(())
}
