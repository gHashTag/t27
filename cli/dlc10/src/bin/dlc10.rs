//! `dlc10` CLI: read IDCODE, program SRAM, program SPI flash, read JEDEC ID,
//! and a `debug` subcommand for decoding 7-series configuration registers.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dlc10::{cfg_reg, Dlc10, FlashOpts, StatBits};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Pure-Rust driver for Xilinx DLC10 (Platform Cable USB II)"
)]
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
    FlashId {
        #[arg(long)]
        verbose: bool,
    },
    /// Read the (raw) configuration STATUS register via plain CFG_OUT.
    Status,
    /// Decode the FPGA configuration state: STAT, CTL0, CTL1, BOOT_STS,
    /// IDCODE registers via the correct CFG_IN → CFG_OUT protocol.
    /// Use this after a failing `sram` attempt to diagnose DONE=LOW.
    Debug {
        /// Read STAT *without* trying any JSTART/BYPASS toggle first.
        /// Useful to confirm whether `program_sram` is leaving the chip
        /// in DONE=HIGH state while only the post-JSTART readback path
        /// is broken.
        #[arg(long)]
        no_jstart: bool,
    },
    /// Self-test the Type-1 read protocol by reading the configuration
    /// IDCODE register (addr 0x0C) via CFG_IN+CFG_OUT. On a healthy
    /// XC7A100T this MUST return 0x13631093 — same as the JTAG IDCODE.
    /// If JTAG IDCODE matches but this reads 0x00000000, the bug is in
    /// our read protocol (e.g. missing RTI parking), not in the chip.
    IdcodeCfg {
        /// Dump the exact wire-format DR payload (host-order packet words
        /// AND the bytes shifted on the wire after `reverse_32` + LE
        /// byte-split) for hand-comparison with xc3sprog / openFPGALoader,
        /// plus a 64-bit CFG_OUT shift to test the dummy-pipeline-word
        /// hypothesis (some 7-series parts return the value on the SECOND
        /// 32-bit word, not the first).
        #[arg(long)]
        raw: bool,
    },
    /// Read IR capture byte (DONE, INIT_B, ISC_ENABLED, ISC_DONE).
    IrCapture,
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
            let bytes = std::fs::read(&bit).with_context(|| format!("read {}", bit.display()))?;
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
            let bytes = std::fs::read(&bit).with_context(|| format!("read {}", bit.display()))?;
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
        Cmd::FlashId { verbose } => {
            let id = cable.read_flash_id_verbose(verbose)?;
            println!("JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);
        }
        Cmd::Status => {
            let s = cable.read_status()?;
            println!("STATUS: 0x{:08X}", s);
        }
        Cmd::IdcodeCfg { raw } => {
            let jtag_id = cable.read_idcode()?;
            println!(
                "JTAG IDCODE        : 0x{:08X}{}",
                jtag_id,
                if jtag_id == 0x13631093 {
                    "  (XC7A100T)"
                } else {
                    "  (UNEXPECTED)"
                }
            );

            if raw {
                // Wire-format dump + 64-bit CFG_OUT for the dummy-pipeline
                // hypothesis. Two consecutive read attempts so the user
                // can see whether the value migrates between words.
                let diag = cable.read_cfg_reg_diag(dlc10::cfg_reg::IDCODE, 64)?;
                println!();
                println!("== Wire-format diagnostic (Type-1 read for IDCODE addr 0x0C) ==");
                println!("Host-order packet words ([0]=SYNC … [4]=NOP2):");
                for (i, w) in diag.packets_host_order.iter().enumerate() {
                    let tag = match i {
                        0 => "SYNC",
                        1 => "NOP",
                        2 => "READ_HDR",
                        3 => "NOP",
                        4 => "NOP",
                        _ => "?",
                    };
                    println!("  [{i}] {tag:>8} = 0x{w:08X}");
                }
                println!();
                println!("Wire bytes (per-word reverse_bits, then LE byte-split — 4 bytes/word, 20 bytes total):");
                for chunk in diag.wire_bytes_per_word.chunks(4) {
                    println!("  {}", hex::encode(chunk));
                }
                println!();
                println!(
                    "Concatenated wire bytes: {}",
                    hex::encode(&diag.wire_bytes_per_word)
                );
                println!();
                println!("CFG_OUT 64-bit shift (2 × 32-bit words clocked out, already reverse_bits-applied):");
                for (i, w) in diag.result_words.iter().enumerate() {
                    let tag = if w == &0x13631093 {
                        "  ← XC7A100T IDCODE"
                    } else {
                        ""
                    };
                    println!("  word[{i}] = 0x{w:08X}{tag}");
                }
                println!();
                if diag.result_words.contains(&0x13631093) {
                    if diag.result_words.first() == Some(&0x13631093) {
                        println!("=> Type-1 read OK on first CFG_OUT word.");
                    } else {
                        println!("=> Type-1 read OK on SECOND CFG_OUT word — there IS a 1-word dummy pipeline.");
                        println!("   The driver should drop the first CFG_OUT word.");
                    }
                } else {
                    println!("=> Type-1 read FAILED — no word matched 0x13631093.");
                    println!("   Check wire bytes above against `openFPGALoader xilinx.cpp::dumpRegister` step-by-step.");
                }
            } else {
                let cfg_id = cable.read_cfg_idcode()?;
                println!(
                    "CFG IDCODE (0x0C)  : 0x{:08X}{}",
                    cfg_id,
                    if cfg_id == 0x13631093 {
                        "  (XC7A100T)"
                    } else {
                        "  (mismatch!)"
                    }
                );
                println!();
                if jtag_id == 0x13631093 && cfg_id == 0x13631093 {
                    println!("=> Type-1 read protocol OK (CFG IDCODE matches JTAG IDCODE).");
                } else if jtag_id == 0x13631093 && cfg_id != 0x13631093 {
                    println!("=> JTAG bus is healthy but Type-1 read protocol is BROKEN.");
                    println!("   Re-run with `--raw` to dump exact wire bytes for comparison.");
                } else {
                    println!("=> JTAG IDCODE itself is wrong — TAP walk / cable issue.");
                }
            }
        }
        Cmd::Debug { no_jstart } => {
            let idcode = cable.read_idcode()?;
            println!("== JTAG IDCODE ==");
            println!(
                "  IDCODE              : 0x{:08X}{}",
                idcode,
                if idcode == 0x13631093 {
                    "  (XC7A100T)"
                } else {
                    "  (UNEXPECTED)"
                }
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
            println!(
                "  IDCODE  (0x0C)      : 0x{:08X}{}",
                cfg_idcode,
                if cfg_idcode == 0x13631093 {
                    "  (XC7A100T)"
                } else {
                    "  (mismatch!)"
                }
            );
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
        Cmd::IrCapture => {
            let cap = cable.shift_ir_capture(dlc10::ir::BYPASS)?;
            let done = (cap >> 5) & 1;
            let init_b = (cap >> 4) & 1;
            let isc_en = (cap >> 3) & 1;
            let isc_done = (cap >> 2) & 1;
            let low2 = cap & 0x03;
            println!("IR capture: 0x{:02X}", cap);
            println!("  DONE={} INIT_B={} ISC_EN={} ISC_DONE={} low2=0x{:02X}", done, init_b, isc_en, isc_done, low2);
        }
    }
    cable.close();
    Ok(())
}
