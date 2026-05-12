//! `dlc10` CLI: read IDCODE, program SRAM, program SPI flash, read JEDEC ID,
//! and a `debug` subcommand for decoding 7-series configuration registers.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dlc10::{cfg_reg, Dlc10, FlashOpts, StatBits};


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
        /// Emit detailed instrumentation: payload range, sync-word offset,
        /// first/last shifted bytes, chunk counts, raw CFG_OUT read.
        #[arg(long)]
        verbose: bool,
    },
    /// Program the on-board SPI flash (non-volatile).
    Flash {
        bit: PathBuf,
        #[arg(long, default_value_t = true)]
        verify: bool,
    },
    /// Read the SPI flash JEDEC ID via the JTAG-to-SPI bridge.
    FlashId,
    /// Read the (raw) configuration STATUS register via plain CFG_OUT.
    Status,
    /// Decode the FPGA configuration state: STAT, CTL0, CTL1, BOOT_STS,
    /// IDCODE registers via the correct CFG_IN → CFG_OUT protocol.
    /// Use this after a failing `sram` attempt to diagnose DONE=LOW.
    Debug,
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
        Cmd::Sram { bit, verbose } => {
            let bytes = std::fs::read(&bit)
                .with_context(|| format!("read {}", bit.display()))?;
            let status = cable.program_sram_verbose(&bytes, verbose)?;
            println!("CFG_OUT raw (BYPASS+CFG_OUT): 0x{:08X}", status);
            // The raw CFG_OUT after BYPASS does not implement the
            // Type-1 read protocol; the captured value is stale and
            // its bit order is shift-order (LSB-first). Run `dlc10
            // debug` for a faithful STAT decode.
            eprintln!(
                "note: this raw value is not a valid STAT decode. \
                 Run `dlc10 debug` for register-by-register diagnosis."
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
        Cmd::Debug => {
            let idcode = cable.read_idcode()?;
            println!("== JTAG IDCODE ==");
            println!("  IDCODE              : 0x{:08X}{}",
                idcode,
                if idcode == 0x13631093 { "  (XC7A100T)" } else { "  (UNEXPECTED)" });
            println!();

            let stat_raw = cable.read_cfg_reg(cfg_reg::STAT)?;
            let stat = StatBits::from_raw(stat_raw);
            println!("== STAT register (addr 0x07, UG470 Table 5-25) ==");
            println!("  raw                 : 0x{:08X}", stat.raw);
            println!("  CRC_ERROR  [0]      : {}", stat.crc_error as u8);
            println!("  PART_SECURED [1]    : {}", stat.part_secured as u8);
            println!("  MMCM_LOCK  [2]      : {}", stat.mmcm_lock as u8);
            println!("  DCI_MATCH  [3]      : {}", stat.dci_match as u8);
            println!("  EOS        [4]      : {}", stat.eos as u8);
            println!("  GTS_CFG_B  [5]      : {}", stat.gts_cfg_b as u8);
            println!("  GWE        [6]      : {}", stat.gwe as u8);
            println!("  GHIGH_B    [7]      : {}", stat.ghigh_b as u8);
            println!("  MODE       [10:8]   : {}", stat.mode);
            println!("  INIT_COMPL [11]     : {}", stat.init_complete as u8);
            println!("  INIT_B     [12]     : {}", stat.init_b as u8);
            println!("  RELEASE_DONE [13]   : {}", stat.release_done as u8);
            println!("  DONE       [14]     : {}", stat.done as u8);
            println!("  ID_ERROR   [15]     : {}", stat.id_error as u8);
            println!("  DEC_ERROR  [16]     : {}", stat.dec_error as u8);
            println!("  XADC_OT    [17]     : {}", stat.xadc_over_temp as u8);
            println!("  STARTUP_STATE [21:18]: 0x{:X}", stat.startup_state);
            println!("  BUS_WIDTH  [23:22]  : {}", stat.bus_width);
            println!("  CFGERR_B   [25]     : {}", stat.cfgerr_b as u8);
            println!("  diagnosis           : {}", stat.diagnose());
            println!();

            // Other registers for additional context.
            let ctl0 = cable.read_cfg_reg(cfg_reg::CTL0)?;
            let ctl1 = cable.read_cfg_reg(cfg_reg::CTL1)?;
            let boot_sts = cable.read_cfg_reg(cfg_reg::BOOTSTS)?;
            let cfg_idcode = cable.read_cfg_reg(cfg_reg::IDCODE)?;
            let wbstar = cable.read_cfg_reg(cfg_reg::WBSTAR)?;
            let cor0 = cable.read_cfg_reg(cfg_reg::COR0)?;
            let cor1 = cable.read_cfg_reg(cfg_reg::COR1)?;

            println!("== Other configuration registers ==");
            println!("  CTL0    (0x05)      : 0x{:08X}", ctl0);
            println!("  CTL1    (0x18)      : 0x{:08X}", ctl1);
            println!("  BOOTSTS (0x16)      : 0x{:08X}", boot_sts);
            println!("  IDCODE  (0x0C)      : 0x{:08X}{}",
                cfg_idcode,
                if cfg_idcode == 0x13631093 { "  (XC7A100T)" } else { "  (mismatch!)" });
            println!("  WBSTAR  (0x10)      : 0x{:08X}", wbstar);
            println!("  COR0    (0x09)      : 0x{:08X}", cor0);
            println!("  COR1    (0x0E)      : 0x{:08X}", cor1);
            println!();

            if stat.done {
                println!("=> FPGA is configured. DONE=HIGH.");
            } else {
                println!("=> FPGA is NOT configured. {}", stat.diagnose());
            }
        }
    }
    cable.close();
    Ok(())
}
