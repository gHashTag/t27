//! `tri fpga ...` — centralised FPGA programming via the in-tree `dlc10`
//! crate. Replaces `tools/dlc10_jtag.py` and `tools/tri_fpga/cli.py`.
//!
//! All operations use pure-Rust paths through `rusb`; no external tools
//! (Vivado / openFPGALoader) and no Python dependencies are required.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use dlc10::{cfg_reg, Dlc10, FlashOpts, StatBits};

#[derive(Subcommand, Debug)]
pub enum FpgaCmd {
    /// Read and print the JTAG IDCODE of the attached DLC10 cable target.
    Idcode,
    /// Program FPGA SRAM (volatile — lost on power-cycle).
    Sram {
        bit: PathBuf,
        /// Emit detailed instrumentation.
        #[arg(long)]
        verbose: bool,
    },
    /// Program the on-board SPI flash (non-volatile / persistent).
    Program {
        bit: PathBuf,
        /// Skip read-back verification.
        #[arg(long)]
        no_verify: bool,
    },
    /// Read the SPI flash JEDEC ID via the JTAG-to-SPI bridge.
    FlashId,
    /// Read the raw CFG_OUT status register.
    Status,
    /// Decode 7-series configuration registers for DONE=LOW diagnosis.
    Debug {
        /// Skip any JSTART/BYPASS pulse before reading STAT.
        #[arg(long)]
        no_jstart: bool,
    },
}

pub fn run(cmd: &FpgaCmd) -> Result<()> {
    match cmd {
        FpgaCmd::Idcode => idcode(),
        FpgaCmd::Sram { bit, verbose } => sram(bit, *verbose),
        FpgaCmd::Program { bit, no_verify } => program(bit, !*no_verify),
        FpgaCmd::FlashId => flash_id(),
        FpgaCmd::Status => status(),
        FpgaCmd::Debug { no_jstart } => debug(*no_jstart),
    }
}

fn open_cable() -> Result<Dlc10> {
    Dlc10::open().context("open DLC10 cable (is it plugged in?)")
}

fn idcode() -> Result<()> {
    let mut cable = open_cable()?;
    let id = cable.read_idcode()?;
    println!("IDCODE: 0x{:08X}", id);
    if id != 0x13631093 {
        eprintln!("note: expected 0x13631093 (XC7A100T), got 0x{:08X}", id);
    }
    cable.close();
    Ok(())
}

fn sram(bit: &PathBuf, verbose: bool) -> Result<()> {
    let bytes = std::fs::read(bit)
        .with_context(|| format!("read {}", bit.display()))?;
    let mut cable = open_cable()?;
    let status = cable.program_sram_verbose(&bytes, verbose)?;
    println!("CFG_OUT raw (BYPASS+CFG_OUT): 0x{:08X}", status);
    eprintln!(
        "note: raw CFG_OUT is not a valid STAT decode. \
         Run `tri fpga debug` for register-by-register diagnosis."
    );
    cable.close();
    Ok(())
}

fn program(bit: &PathBuf, verify: bool) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let bytes = std::fs::read(bit)
        .with_context(|| format!("read {}", bit.display()))?;
    let total = bytes.len() as u64;
    eprintln!(
        "Programming SPI flash: {} ({:.1} MiB)",
        bit.display(),
        total as f64 / 1024.0 / 1024.0
    );

    let mut cable = open_cable()?;
    let id = cable.read_idcode()?;
    if id != 0x13631093 {
        bail!(
            "IDCODE mismatch: got 0x{:08X}, expected 0x13631093 (XC7A100T)",
            id
        );
    }
    eprintln!("IDCODE 0x{:08X} confirmed.", id);

    let opts = FlashOpts {
        verify,
        progress: Some(Box::new(move |w, t| {
            if w == t || w % (1 << 18) < 256 {
                eprintln!("  {} / {} ({}%)", w, total, 100 * w / total.max(1));
            }
        })),
    };
    cable.program_flash(&bytes, opts)?;
    eprintln!("Flash write OK — bitstream is now persistent.");
    cable.close();
    Ok(())
}

fn flash_id() -> Result<()> {
    let mut cable = open_cable()?;
    let id = cable.read_flash_id()?;
    println!("JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);
    cable.close();
    Ok(())
}

fn status() -> Result<()> {
    let mut cable = open_cable()?;
    let s = cable.read_status()?;
    println!("STATUS: 0x{:08X}", s);
    cable.close();
    Ok(())
}

fn debug(no_jstart: bool) -> Result<()> {
    let mut cable = open_cable()?;
    let idcode = cable.read_idcode()?;
    println!("== JTAG IDCODE ==");
    println!(
        "  IDCODE              : 0x{:08X}{}",
        idcode,
        if idcode == 0x13631093 { "  (XC7A100T)" } else { "  (UNEXPECTED)" }
    );
    println!();

    if no_jstart {
        println!("(--no-jstart: skipping any JSTART/BYPASS pulse before reading STAT)");
        println!();
    }

    let stat_raw = cable.read_cfg_reg(cfg_reg::STAT)?;
    let stat = StatBits::from_raw(stat_raw);
    println!("== STAT register (addr 0x07, UG470 Table 5-25) ==");
    println!("  raw                 : 0x{:08X}", stat.raw);
    println!("  DONE       [14]     : {}", stat.done as u8);
    println!("  INIT_COMPL [11]     : {}", stat.init_complete as u8);
    println!("  EOS        [4]      : {}", stat.eos as u8);
    println!("  CRC_ERROR  [0]      : {}", stat.crc_error as u8);
    println!("  ID_ERROR   [15]     : {}", stat.id_error as u8);
    println!("  diagnosis           : {}", stat.diagnose());
    println!();

    if stat.done {
        println!("=> FPGA is configured. DONE=HIGH.");
    } else {
        println!("=> FPGA is NOT configured. {}", stat.diagnose());
    }
    cable.close();
    Ok(())
}
