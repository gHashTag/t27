//! `tri fpga ...` — centralised FPGA programming via the in-tree `dlc10`
//! crate. Replaces `tools/dlc10_jtag.py` and `tools/tri_fpga/cli.py`.
//!
//! All operations use pure-Rust paths through `rusb`; no external tools
//! (Vivado / openFPGALoader) and no Python dependencies are required.

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;
use dlc10::{cfg_reg, ir, Dlc10, FlashOpts, StatBits, BSCAN_SPI_XC7A100T};

#[derive(Subcommand, Debug)]
pub enum FpgaCmd {
    /// Read and print the JTAG IDCODE of the attached DLC10 cable target.
    Idcode,
    /// Read the configuration IDCODE register via the Type-1 CFG_IN/CFG_OUT
    /// protocol. On a healthy XC7A100T this must return 0x13631093 (same as
    /// the JTAG IDCODE). If 0x00000000 is returned while `idcode` works, the
    /// read_cfg_reg implementation is broken.
    IdcodeCfg,
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
        /// Disable the default per-byte bit-swap of the bitstream payload.
        /// Vivado's `write_cfgmem` bit-swaps by default for Master SPI boot;
        /// disable only if your bitstream is already pre-swapped.
        #[arg(long, default_value_t = false)]
        no_bitswap: bool,
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
    /// Load a bitstream into FPGA SRAM using openFPGALoader with a Digilent
    /// FTDI cable. This is the canonical path for the QMTech Wukong V1 /
    /// XC7A200T board because the in-tree `dlc10` driver only supports Xilinx
    /// DLC10 cables (VID=0x03FD), not the attached Digilent cable
    /// (VID=0x0403:0x6014).
    LoadSram {
        bit: PathBuf,
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for openFPGALoader (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
        /// Reset the FPGA after loading.
        #[arg(long)]
        reset: bool,
        /// Emit verbose openFPGALoader output.
        #[arg(long)]
        verbose: bool,
    },
    /// Read and decode the FPGA STAT register via openFPGALoader.
    Stat {
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// Skip the JTAG reset/PROGRAM_B pulse before reading STAT.
        /// Use this to capture cold-POR mode sampling before the FPGA
        /// is re-initialized by the cable.
        #[arg(long)]
        pre_jtag_reset: bool,
        /// Number of consecutive STAT samples to capture (default: 1).
        /// Useful to see transient mode-bit values right after power-on.
        #[arg(long, default_value_t = 1)]
        repeat: u32,
    },
    /// Synthesize the GF16 4x4 matrix design through the openXC7 toolchain.
    /// Output is written to `<build_dir>/gf16_matmul4x4_top.bit`.
    SynthGf16 {
        /// Build output directory (default: <repo>/build/fpga/gf16).
        #[arg(long)]
        build_dir: Option<PathBuf>,
        /// nextpnr-xilinx chipdb binary (default: <repo>/build/xc7a100tfgg676.bin).
        #[arg(long)]
        chipdb: Option<PathBuf>,
        /// Target part for fasm2frames/xc7frames2bit (default: xc7a200tfbg676-1).
        /// prjxray-db lacks xc7a200tfgg676-1; fbg676-1 shares the same idcode.
        #[arg(long, default_value = "xc7a200tfbg676-1")]
        part: String,
    },
    /// Program the on-board SPI flash (non-volatile) using openFPGALoader's
    /// JTAG-to-SPI bridge. Requires a matching `spiOverJtag` bitstream for the
    /// exact FPGA package; the XC7A200T-FGG676 bridge is present in recent
    /// openFPGALoader distributions but flash boot may still depend on board
    /// mode pins (see docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md).
    ProgramFlash {
        bit: PathBuf,
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for bridge selection (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
        /// Optional explicit path to a JTAG-to-SPI bridge bitstream.
        /// If omitted, openFPGALoader selects one from its installation.
        #[arg(long)]
        bridge: Option<PathBuf>,
        /// JTAG frequency in Hz (default: 6 MHz).
        #[arg(long, default_value = "6000000")]
        freq: u32,
        /// Bitstream file type: `bit` or `bin` (default: auto-detect).
        #[arg(long)]
        file_type: Option<String>,
        /// Skip the post-write FPGA reset.
        #[arg(long)]
        skip_reset: bool,
        /// Verify the flash contents after writing.
        #[arg(long)]
        verify: bool,
        /// Bulk-erase the flash before writing.
        #[arg(long)]
        bulk_erase: bool,
        /// Enable the SPI flash quad-enable (QE) bit before writing.
        /// Needed for some boards/flash to boot from x4 SPI.
        #[arg(long)]
        enable_quad: bool,
        /// Disable the SPI flash quad-enable (QE) bit.
        #[arg(long)]
        disable_quad: bool,
        /// SPI bus width expected by the bitstream: 1, 2, or 4.
        /// Logged for diagnosis; the real width is set in the bitstream.
        #[arg(long, value_name = "WIDTH", value_parser = clap::builder::PossibleValuesParser::new(["1", "2", "4"]))]
        spi_buswidth: Option<String>,
    },
    /// Dump SPI flash contents to a file via openFPGALoader's JTAG-to-SPI bridge.
    DumpFlash {
        /// Output file path.
        out: PathBuf,
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for bridge selection (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
        /// Size in bytes to dump (default: 16 MiB, the whole N25Q128).
        #[arg(long, default_value_t = 16777216)]
        size: usize,
    },
    /// Parse the .bit header and print the configuration registers that
    /// affect Master SPI boot (COR0, COR1, WBSTAR, TIMER, CTL0, CTL1,
    /// IDCODE, BSPI).
    BitConfig {
        /// Path to the Xilinx .bit file.
        bit: PathBuf,
    },
    /// Program the given .bit to SPI flash, dump the same region back, and
    /// compare the dumped bytes against the bitstream payload. This verifies
    /// the openFPGALoader write path is bit-perfect for the raw bitstream.
    RoundTripVerify {
        /// Bitstream to program and read back.
        bit: PathBuf,
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for bridge selection (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
        /// Optional explicit path to a JTAG-to-SPI bridge bitstream.
        #[arg(long)]
        bridge: Option<PathBuf>,
        /// JTAG frequency in Hz (default: 6 MHz).
        #[arg(long, default_value = "6000000")]
        freq: u32,
    },
    /// Guided cold-POR boot experiment. Programs flash, asks the user to
    /// physically power-cycle the board, then captures STAT without a JTAG
    /// reset and prints a decision-tree diagnosis.
    BootLog {
        /// Bitstream to program to flash.
        bit: PathBuf,
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for bridge selection (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
        /// Optional explicit path to a JTAG-to-SPI bridge bitstream.
        #[arg(long)]
        bridge: Option<PathBuf>,
        /// JTAG frequency in Hz (default: 6 MHz).
        #[arg(long, default_value = "6000000")]
        freq: u32,
        /// Number of consecutive STAT samples to capture after power-on
        /// (default: 3).
        #[arg(long, default_value_t = 3)]
        repeat: u32,
        /// Seconds to wait for the user to power-cycle before sampling STAT.
        /// Ignored; the command waits for keyboard input by default.
        #[arg(long, default_value_t = 0)]
        wait_seconds: u32,
    },
    /// Board-less smoke gate for the FPGA path. Runs `tri fpga bit-config`
    /// on the GF16 demo bitstream and, if yosys is available, a synthesis
    /// smoke check on the demo Verilog. Requires no physical board.
    SmokeGate {
        /// Bitstream to audit (default: fpga/verilog/ternary_mac_demo_top.bit).
        #[arg(long)]
        bit: Option<PathBuf>,
        /// Verilog top module to synthesize (default: ternary_mac_demo_top).
        #[arg(long, default_value = "ternary_mac_demo_top")]
        top: String,
    },
    /// Read the SPI flash status register via openFPGALoader's JTAG-to-SPI bridge.
    /// Decodes WIP, WEL, and QE bits to help diagnose boot-from-flash failures.
    FlashStatus {
        /// openFPGALoader cable profile (default: digilent_hs2).
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for bridge selection (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
    },
    /// Diagnostic: load *only* the proxy bridge bitstream (or any other
    /// bitstream) into FPGA SRAM, report STAT, leave TAP in RTI. Does NOT
    /// touch SPI flash. Useful for verifying the bridge actually reaches
    /// DONE=HIGH before debugging SPI semantics.
    ///
    /// With no argument, uses the embedded `bscan_spi_xc7a100t.bit`.
    ProxyLoad {
        /// Optional path to a different proxy bitstream.
        bit: Option<PathBuf>,
    },
    /// Diagnostic: read IDCODE + STAT *without* touching the FPGA. Run
    /// this immediately after `tri fpga proxy-load` to confirm the bridge
    /// is alive. Also confirms IR=USER1 select is accepted.
    ProxyStatus,
    /// Diagnostic: shift raw hex bytes through the USER1 BSCAN as a
    /// single SPI transaction. Reads `--rx N` MISO bytes after the TX
    /// stream. Requires the proxy bridge to be already loaded
    /// (`tri fpga proxy-load` first).
    ///
    /// Examples:
    ///   `tri fpga spi-raw 9F --rx 3`       # read JEDEC ID
    ///   `tri fpga spi-raw AB`              # release power-down
    ///   `tri fpga spi-raw 66`              # reset enable
    ///   `tri fpga spi-raw 99`              # reset device
    ///   `tri fpga spi-raw 05 --rx 1`       # read status register
    SpiRaw {
        /// Hex string of bytes to shift onto MOSI (e.g. `9F` or `0102FF`).
        hex: String,
        /// Number of MISO bytes to capture after the TX stream.
        #[arg(long, default_value_t = 0)]
        rx: usize,
    },
    /// Diagnostic: probe the IR capture pattern after selecting an IR.
    /// A healthy 7-series TAP always captures `0b000001` into the IR
    /// shift register. Anything else means the JTAG chain is broken or
    /// the cable is mis-driving TMS.
    IrProbe {
        /// IR opcode to select (hex, e.g. `02` for USER1, `3F` for BYPASS).
        ir_hex: String,
    },
    /// Diagnostic: drive the full JEDEC-read flow end-to-end with maximum
    /// instrumentation, **including** the 0xAB Release-Power-down and
    /// 0x66+0x99 software-reset recovery attempts. Equivalent to
    /// `flash-id --verbose` plus auto-recovery.
    FlashIdDebug,
    /// Build the QMTech XC7A100T-FGG676 JTAG-to-SPI proxy bitstream via the
    /// openXC7 open-source toolchain (yosys + nextpnr-himbaechel + prjxray).
    /// Requires `yosys`, `nextpnr-himbaechel`, `fasm2frames.py` and
    /// `xc7frames2bit` on PATH. With `--install`, the produced `.bit` is
    /// copied to `fpga/tools/bscan_spi_xc7a100t.bit` so the embedded
    /// `BSCAN_SPI_XC7A100T` constant picks it up on the next rebuild.
    BuildProxy {
        /// After a successful build, copy the bitstream to
        /// `fpga/tools/bscan_spi_xc7a100t.bit`.
        #[arg(long)]
        install: bool,
        /// Source directory (Verilog + XDC). Defaults to
        /// `fpga/bscan_spi_qmtech/` under the repo root.
        #[arg(long)]
        src: Option<PathBuf>,
        /// Output directory for intermediate artefacts and the final
        /// `bscan_spi_xc7a100tfgg676.bit`. Defaults to `<src>/build/`.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Explicit path to a pre-built nextpnr-himbaechel chipdb (`.bba`).
        /// If omitted, standard locations are scanned (see
        /// `tri fpga setup-openxc7-chipdb` for installation).
        #[arg(long)]
        chipdb: Option<PathBuf>,
    },
    /// Clone the openXC7 `nextpnr-xilinx` repo and build a chipdb (`.bba`)
    /// for the requested 7-series part, then install it into
    /// `~/.local/share/nextpnr/himbaechel-xilinx/`.
    ///
    /// The standard build takes 20–40 minutes on Apple Silicon and downloads
    /// the prjxray database as a submodule (~1 GiB). No Python is invoked
    /// from this CLI; the upstream CMake/ninja project is used as-is.
    SetupOpenxc7Chipdb {
        /// Installation prefix. Defaults to
        /// `~/.local/share/nextpnr/himbaechel-xilinx/`.
        #[arg(long)]
        prefix: Option<PathBuf>,
        /// 7-series family. Defaults to `xc7a100t`.
        #[arg(long, default_value = "xc7a100t")]
        family: String,
        /// Working directory for the clone + build. Defaults to
        /// `<repo>/target/nextpnr-xilinx/`.
        #[arg(long)]
        work_dir: Option<PathBuf>,
        /// Git ref (branch / tag / SHA) of `openXC7/nextpnr-xilinx` to use.
        #[arg(long, default_value = "master")]
        git_ref: String,
    },
    /// Build the QMTech XC7A100T-FGG676 proxy bitstream via a Docker
    /// container running Xilinx Vivado. Clones our `openFPGALoader` fork
    /// (`feat/qmtech-xc7a100t-board`) into `target/openfpgaloader-fork/`
    /// and runs `make` inside `spiOverJtag/`. On Apple Silicon (arm64),
    /// the container runs under x86_64 emulation via
    /// `--platform linux/amd64`. With `--install`, the produced
    /// `bscan_spi_xc7a100tfgg676.bit.gz` is decompressed and copied to
    /// `fpga/tools/bscan_spi_xc7a100t.bit` and its SHA256 is printed.
    ///
    /// This is an alternative to `build-proxy` (which uses the open-source
    /// openXC7 flow) for users who already have a Vivado-capable Docker
    /// image. See `docker/Dockerfile.vivado` for build instructions when
    /// no public image is available.
    BuildProxyDocker {
        /// Path to an already-cloned openFPGALoader fork. If omitted,
        /// the fork is cloned into `target/openfpgaloader-fork/`.
        #[arg(long)]
        fork_dir: Option<PathBuf>,
        /// Docker image providing Vivado on `linux/amd64`. Defaults to
        /// the locally-built `t27/vivado:webpack` (see
        /// `docker/Dockerfile.vivado`).
        #[arg(long)]
        image: Option<String>,
        /// After build, decompress and install the bitstream to
        /// `fpga/tools/bscan_spi_xc7a100t.bit`.
        #[arg(long)]
        install: bool,
        /// Skip the `--platform linux/amd64` flag (use when already on
        /// an x86_64 host or when the chosen image is multi-arch).
        #[arg(long)]
        no_platform: bool,
    },
}

pub fn run(cmd: &FpgaCmd) -> Result<()> {
    match cmd {
        FpgaCmd::Idcode => idcode(),
        FpgaCmd::IdcodeCfg => idcode_cfg(),
        FpgaCmd::Sram { bit, verbose } => sram(bit, *verbose),
        FpgaCmd::Program { bit, no_verify, no_bitswap } => program(bit, !*no_verify, !*no_bitswap),
        FpgaCmd::FlashId => flash_id(),
        FpgaCmd::Status => status(),
        FpgaCmd::Debug { no_jstart } => debug(*no_jstart),
        FpgaCmd::ProxyLoad { bit } => proxy_load(bit.as_ref()),
        FpgaCmd::ProxyStatus => proxy_status(),
        FpgaCmd::SpiRaw { hex, rx } => spi_raw(hex, *rx),
        FpgaCmd::IrProbe { ir_hex } => ir_probe(ir_hex),
        FpgaCmd::FlashIdDebug => flash_id_debug(),
        FpgaCmd::BuildProxy {
            install,
            src,
            out,
            chipdb,
        } => build_proxy(*install, src.as_ref(), out.as_ref(), chipdb.as_ref()),
        FpgaCmd::SetupOpenxc7Chipdb {
            prefix,
            family,
            work_dir,
            git_ref,
        } => setup_openxc7_chipdb(prefix.as_ref(), family, work_dir.as_ref(), git_ref),
        FpgaCmd::BuildProxyDocker {
            fork_dir,
            image,
            install,
            no_platform,
        } => build_proxy_docker(fork_dir.as_ref(), image.as_deref(), *install, *no_platform),
        FpgaCmd::LoadSram {
            bit,
            cable,
            part,
            reset,
            verbose,
        } => load_sram(bit, cable, part, *reset, *verbose),
        FpgaCmd::Stat {
            cable,
            pre_jtag_reset,
            repeat,
        } => stat(cable, *pre_jtag_reset, *repeat),
        FpgaCmd::BitConfig { bit } => bit_config(bit),
        FpgaCmd::RoundTripVerify { bit, cable, part, bridge, freq } => {
            round_trip_verify(bit, cable, part, bridge.as_ref(), *freq)
        }
        FpgaCmd::BootLog {
            bit,
            cable,
            part,
            bridge,
            freq,
            repeat,
            wait_seconds,
        } => boot_log(bit, cable, part, bridge.as_ref(), *freq, *repeat, *wait_seconds),
        FpgaCmd::SmokeGate { bit, top } => smoke_gate(bit.as_ref(), top),
        FpgaCmd::SynthGf16 {
            build_dir,
            chipdb,
            part,
        } => synth_gf16(build_dir.as_ref(), chipdb.as_ref(), part),
        FpgaCmd::ProgramFlash {
            bit,
            cable,
            part,
            bridge,
            freq,
            file_type,
            skip_reset,
            verify,
            bulk_erase,
            enable_quad,
            disable_quad,
            spi_buswidth,
        } => program_flash(
            bit,
            cable,
            part,
            bridge.as_ref(),
            *freq,
            file_type.as_deref(),
            *skip_reset,
            *verify,
            *bulk_erase,
            *enable_quad,
            *disable_quad,
            spi_buswidth.as_deref(),
        ),
        FpgaCmd::DumpFlash {
            out,
            cable,
            part,
            size,
        } => dump_flash(out, cable, part, *size),
        FpgaCmd::FlashStatus { cable, part } => flash_status(cable, part),
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

fn idcode_cfg() -> Result<()> {
    let mut cable = open_cable()?;
    let id = cable.read_cfg_idcode()?;
    println!("CFG IDCODE: 0x{:08X}", id);
    if id == 0x13631093 {
        println!("  (XC7A100T — correct)");
    } else if id == 0x00000000 {
        eprintln!("  ERROR: 0x00000000 — read_cfg_reg is broken (Update-DR issue?)");
    } else {
        eprintln!("  UNEXPECTED: expected 0x13631093 for XC7A100T");
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

fn program(bit: &PathBuf, verify: bool, bitswap: bool) -> Result<()> {
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
        no_jprogram: false,
        bitswap,
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

fn proxy_load(bit: Option<&PathBuf>) -> Result<()> {
    let bytes: Vec<u8> = match bit {
        Some(p) => {
            eprintln!("[debug] proxy-load: reading bitstream from {}", p.display());
            std::fs::read(p).with_context(|| format!("read {}", p.display()))?
        }
        None => {
            eprintln!(
                "[debug] proxy-load: using embedded bscan_spi_xc7a100t.bit ({} bytes)",
                BSCAN_SPI_XC7A100T.len()
            );
            BSCAN_SPI_XC7A100T.to_vec()
        }
    };
    let mut cable = open_cable()?;
    let raw = cable.proxy_load(&bytes)?;
    println!("CFG_OUT raw (BYPASS+CFG_OUT): 0x{:08X}", raw);
    eprintln!("[debug] proxy-load complete — now run `tri fpga proxy-status` to confirm DONE=HIGH");
    cable.close();
    Ok(())
}

fn proxy_status() -> Result<()> {
    let mut cable = open_cable()?;
    let s = cable.proxy_status()?;
    println!("STAT raw: 0x{:08X}", s.raw);
    println!("  DONE          : {}", s.done as u8);
    println!("  EOS           : {}", s.eos as u8);
    println!("  INIT_B        : {}", s.init_b as u8);
    println!("  INIT_COMPLETE : {}", s.init_complete as u8);
    println!("  MMCM_LOCK     : {}", s.mmcm_lock as u8);
    println!("  ID_ERROR      : {}", s.id_error as u8);
    println!("  CRC_ERROR     : {}", s.crc_error as u8);
    println!("  diagnosis     : {}", s.diagnose());
    if !s.done {
        eprintln!();
        eprintln!("⚠ proxy bridge is NOT running (DONE=LOW). SPI flash will return FF FF FF.");
        eprintln!("  Verify the proxy bitstream pinout matches this board (QMTech XC7A100T).");
        eprintln!("  See docs/fpga/SPI_FLASH_DEBUG.md.");
    } else {
        eprintln!();
        eprintln!("✓ proxy bridge looks alive. You can now run `tri fpga spi-raw 9F --rx 3`.");
    }
    cable.close();
    Ok(())
}

fn spi_raw(hex: &str, rx: usize) -> Result<()> {
    let clean: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    let tx = ::hex::decode(&clean)
        .map_err(|e| anyhow!("invalid hex {:?}: {}", clean, e))?;
    if tx.is_empty() {
        bail!("spi-raw: TX hex string is empty");
    }
    let mut cable = open_cable()?;
    let result = cable.spi_raw(&tx, rx)?;
    println!("TX  : {}", ::hex::encode_upper(&tx));
    println!("RX  : {}", ::hex::encode_upper(&result));
    cable.close();
    Ok(())
}

fn ir_probe(ir_hex: &str) -> Result<()> {
    let clean = ir_hex.trim_start_matches("0x");
    let ir = u8::from_str_radix(clean, 16)
        .map_err(|e| anyhow!("invalid IR hex {:?}: {}", ir_hex, e))?;
    let known = match ir {
        ir::BYPASS => " (BYPASS)",
        ir::IDCODE => " (IDCODE)",
        ir::CFG_IN => " (CFG_IN)",
        ir::CFG_OUT => " (CFG_OUT)",
        ir::USER1 => " (USER1)",
        ir::USER2 => " (USER2)",
        ir::JPROGRAM => " (JPROGRAM)",
        ir::JSTART => " (JSTART)",
        ir::JSHUTDOWN => " (JSHUTDOWN)",
        _ => "",
    };
    eprintln!("[debug] ir-probe: shifting IR=0x{:02X}{}", ir, known);
    let mut cable = open_cable()?;
    let cap = cable.probe_ir_capture(ir)?;
    println!("IR capture: 0x{:02X}", cap);
    if cap & 0x3F == 0x01 {
        println!("✓ TAP IR capture pattern is healthy (0x01 = '...000001' per IEEE 1149.1).");
    } else {
        println!("⚠ Unexpected IR capture (0x{:02X}). Healthy 7-series should read 0x01.", cap);
        println!("  Possible causes: chain length != 1, TMS routing fault, cable VREF off.");
    }
    cable.close();
    Ok(())
}

fn flash_id_debug() -> Result<()> {
    let mut cable = open_cable()?;
    let id = cable.read_flash_id_verbose(true)?;
    println!("JEDEC ID: {:02X} {:02X} {:02X}", id[0], id[1], id[2]);
    if id == [0xFF, 0xFF, 0xFF] || id == [0x00, 0x00, 0x00] {
        eprintln!();
        eprintln!("⚠ JEDEC still {:02X} {:02X} {:02X} after recovery — see docs/fpga/SPI_FLASH_DEBUG.md", id[0], id[1], id[2]);
    } else {
        eprintln!();
        eprintln!("✓ SPI flash is alive. Manufacturer 0x{:02X} ; device 0x{:02X}{:02X}", id[0], id[1], id[2]);
        match id[0] {
            0x20 => eprintln!("  → Micron (N25Q / MT25Q family)"),
            0xC2 => eprintln!("  → Macronix (MX25 family)"),
            0xEF => eprintln!("  → Winbond (W25Q family)"),
            0x01 => eprintln!("  → Spansion/Cypress"),
            _ => eprintln!("  → Unknown manufacturer code"),
        }
    }
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

// ---------------------------------------------------------------------------
// build-proxy: openXC7 (yosys + nextpnr-himbaechel + prjxray) flow for the
// QMTech XC7A100T-FGG676 JTAG-to-SPI proxy bitstream. No Vivado, no Python
// build glue — all stages are invoked as plain external commands.
// ---------------------------------------------------------------------------

fn which(tool: &str) -> Result<PathBuf> {
    let path_env = std::env::var_os("PATH")
        .ok_or_else(|| anyhow!("PATH not set"))?;
    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(tool);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("required tool not found on PATH: {}", tool)
}

fn run_step(tool: &str, args: &[&str], cwd: &std::path::Path) -> Result<()> {
    let bin = which(tool)?;
    eprintln!(
        "[build-proxy] $ {} {}",
        bin.display(),
        args.join(" ")
    );
    let status = std::process::Command::new(&bin)
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("spawn {}", tool))?;
    if !status.success() {
        bail!("{} exited with {:?}", tool, status);
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join(".git").exists() || dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!("could not locate repository root");
        }
    }
}

fn build_proxy(
    install: bool,
    src: Option<&PathBuf>,
    out: Option<&PathBuf>,
    chipdb: Option<&PathBuf>,
) -> Result<()> {
    let root = repo_root()?;
    let src_dir = match src {
        Some(p) => p.clone(),
        None => root.join("fpga").join("bscan_spi_qmtech"),
    };
    let out_dir = match out {
        Some(p) => p.clone(),
        None => src_dir.join("build"),
    };

    let chipdb_path = match chipdb {
        Some(p) => {
            if !p.is_file() {
                bail!("--chipdb path is not a file: {}", p.display());
            }
            p.clone()
        }
        None => detect_chipdb(&root, "xc7a100t")?
            .ok_or_else(|| anyhow!(
                "no nextpnr-himbaechel chipdb found for xc7a100t.\n  \
                 Searched:\n    \
                 ~/.local/share/nextpnr/himbaechel-xilinx/\n    \
                 /opt/homebrew/share/nextpnr/himbaechel-xilinx/\n    \
                 /usr/local/share/nextpnr/himbaechel-xilinx/\n    \
                 <repo>/build/fpga/\n  \
                 Run `tri fpga setup-openxc7-chipdb` first (≈20–40 min),\n  \
                 or pass an explicit `--chipdb <path>` to a pre-built `.bba`."
            ))?,
    };
    eprintln!("[build-proxy] chipdb : {}", chipdb_path.display());

    let verilog = src_dir.join("bscan_spi_qmtech.v");
    let xdc = src_dir.join("bscan_spi_qmtech.xdc");
    if !verilog.is_file() {
        bail!("missing source: {}", verilog.display());
    }
    if !xdc.is_file() {
        bail!("missing constraints: {}", xdc.display());
    }
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;

    let json_path = out_dir.join("bscan_spi_qmtech.json");
    let fasm_path = out_dir.join("bscan_spi_qmtech.fasm");
    let frames_path = out_dir.join("bscan_spi_qmtech.frames");
    let bit_path = out_dir.join("bscan_spi_xc7a100tfgg676.bit");

    eprintln!("[build-proxy] source : {}", verilog.display());
    eprintln!("[build-proxy] xdc    : {}", xdc.display());
    eprintln!("[build-proxy] out    : {}", out_dir.display());

    // ---- Stage 1: yosys synthesis -------------------------------------
    let yosys_script = format!(
        "read_verilog {v}\nsynth_xilinx -family xc7 -top bscan_spi_qmtech -flatten\nwrite_json {j}\n",
        v = verilog.display(),
        j = json_path.display()
    );
    let ys_path = out_dir.join("synth.ys");
    std::fs::write(&ys_path, yosys_script)?;
    run_step("yosys", &["-q", "-s", ys_path.to_str().unwrap()], &out_dir)?;

    // ---- Stage 2: nextpnr-himbaechel place & route --------------------
    let chipdb_str = chipdb_path.to_str()
        .ok_or_else(|| anyhow!("chipdb path is not valid UTF-8: {:?}", chipdb_path))?;
    let xdc_arg = format!("xdc={}", xdc.display());
    let fasm_arg = format!("fasm={}", fasm_path.display());
    run_step(
        "nextpnr-himbaechel",
        &[
            "--device",
            "xc7a100tfgg676-1",
            "--chipdb",
            chipdb_str,
            "--json",
            json_path.to_str().unwrap(),
            "-o",
            &xdc_arg,
            "-o",
            &fasm_arg,
        ],
        &out_dir,
    )?;

    // ---- Stage 3: fasm2frames + xc7frames2bit -------------------------
    // prjxray ships fasm2frames as either `fasm2frames.py` or `fasm2frames`;
    // try the wrapper first, then fall back to the Python script.
    let fasm2frames_tool = if which("fasm2frames").is_ok() {
        "fasm2frames"
    } else if which("fasm2frames.py").is_ok() {
        "fasm2frames.py"
    } else {
        bail!("neither `fasm2frames` nor `fasm2frames.py` found on PATH (install prjxray)");
    };
    // Both variants accept --part / positional FASM input and write frames
    // to stdout; capture to a file.
    let bin = which(fasm2frames_tool)?;
    eprintln!(
        "[build-proxy] $ {} --part xc7a100tfgg676-2 {} > {}",
        bin.display(),
        fasm_path.display(),
        frames_path.display()
    );
    let frames_file = std::fs::File::create(&frames_path)
        .with_context(|| format!("create {}", frames_path.display()))?;
    let status = std::process::Command::new(&bin)
        .args(["--part", "xc7a100tfgg676-2", fasm_path.to_str().unwrap()])
        .stdout(frames_file)
        .current_dir(&out_dir)
        .status()
        .context("spawn fasm2frames")?;
    if !status.success() {
        bail!("fasm2frames exited with {:?}", status);
    }

    run_step(
        "xc7frames2bit",
        &[
            "--part_file",
            // Allow prjxray to find the part_db; tools resolve via env XRAY_DATABASE_DIR.
            // Pass --part_name explicitly so the user only needs XRAY_DATABASE_DIR set.
            "",
            "--part_name",
            "xc7a100tfgg676-2",
            "--frm_file",
            frames_path.to_str().unwrap(),
            "--output_file",
            bit_path.to_str().unwrap(),
        ],
        &out_dir,
    )?;

    if !bit_path.is_file() {
        bail!("expected bitstream not produced: {}", bit_path.display());
    }
    let size = std::fs::metadata(&bit_path)?.len();
    println!(
        "[build-proxy] OK  {} ({:.1} KiB)",
        bit_path.display(),
        size as f64 / 1024.0
    );

    if install {
        let dst = root
            .join("fpga")
            .join("tools")
            .join("bscan_spi_xc7a100t.bit");
        std::fs::copy(&bit_path, &dst)
            .with_context(|| format!("install {} -> {}", bit_path.display(), dst.display()))?;
        println!("[build-proxy] installed -> {}", dst.display());
        eprintln!("[build-proxy] rebuild `cli/dlc10` to pick up the new embedded bitstream:");
        eprintln!("    cargo build -p tri --release");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Chipdb discovery + openXC7 nextpnr-xilinx setup helper.
// ---------------------------------------------------------------------------

/// Return `$HOME` as a `PathBuf` or `None` if unavailable.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Standard locations where a himbaechel-xilinx chipdb `.bba` may live.
/// Order matters: user-local first, then platform packages, then repo.
fn chipdb_search_dirs(repo: &std::path::Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(home) = home_dir() {
        dirs.push(home.join(".local/share/nextpnr/himbaechel-xilinx"));
        dirs.push(home.join(".local/share/nextpnr"));
    }
    dirs.push(PathBuf::from("/opt/homebrew/share/nextpnr/himbaechel-xilinx"));
    dirs.push(PathBuf::from("/opt/homebrew/share/nextpnr"));
    dirs.push(PathBuf::from("/usr/local/share/nextpnr/himbaechel-xilinx"));
    dirs.push(PathBuf::from("/usr/local/share/nextpnr"));
    dirs.push(repo.join("build").join("fpga"));
    dirs
}

/// Attempt to find a chipdb file for `family` (e.g. `xc7a100t`). Returns
/// `Ok(Some(path))` if one matches, `Ok(None)` if nothing was found and
/// `Err` only on I/O errors that are not "does not exist".
fn detect_chipdb(repo: &std::path::Path, family: &str) -> Result<Option<PathBuf>> {
    // Common filename variants produced by openXC7 / himbaechel-xilinx.
    let candidates: Vec<String> = vec![
        format!("{family}.bba"),
        format!("{family}-fgg676.bba"),
        format!("{family}-fgg676-2.bba"),
    ];
    for dir in chipdb_search_dirs(repo) {
        for name in &candidates {
            let p = dir.join(name);
            match p.try_exists() {
                Ok(true) if p.is_file() => return Ok(Some(p)),
                Ok(_) => continue,
                Err(e) => {
                    eprintln!(
                        "[chipdb] warning: cannot stat {}: {}",
                        p.display(),
                        e
                    );
                }
            }
        }
    }
    Ok(None)
}

fn setup_openxc7_chipdb(
    prefix: Option<&PathBuf>,
    family: &str,
    work_dir: Option<&PathBuf>,
    git_ref: &str,
) -> Result<()> {
    if family.is_empty() {
        bail!("--family must be non-empty (e.g. xc7a100t)");
    }
    let root = repo_root()?;
    let work = match work_dir {
        Some(p) => p.clone(),
        None => root.join("target").join("nextpnr-xilinx"),
    };
    let dest_dir = match prefix {
        Some(p) => p.clone(),
        None => home_dir()
            .ok_or_else(|| anyhow!("$HOME not set; pass --prefix explicitly"))?
            .join(".local/share/nextpnr/himbaechel-xilinx"),
    };

    eprintln!("[setup-chipdb] family   : {}", family);
    eprintln!("[setup-chipdb] git ref  : {}", git_ref);
    eprintln!("[setup-chipdb] workdir  : {}", work.display());
    eprintln!("[setup-chipdb] install  : {}", dest_dir.display());
    eprintln!("[setup-chipdb] note     : full chipdb build takes ≈20–40 min on Apple Silicon");

    // ---- Stage 1: clone (or update) openXC7/nextpnr-xilinx ------------
    if let Some(parent) = work.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    if work.join(".git").is_dir() {
        eprintln!("[setup-chipdb] existing checkout — fetching {}", git_ref);
        run_step("git", &["fetch", "--depth=1", "origin", git_ref], &work)?;
        run_step("git", &["checkout", "FETCH_HEAD"], &work)?;
    } else {
        run_step(
            "git",
            &[
                "clone",
                "--recurse-submodules",
                "--shallow-submodules",
                "--depth=1",
                "--branch",
                git_ref,
                "https://github.com/openXC7/nextpnr-xilinx",
                work.to_str()
                    .ok_or_else(|| anyhow!("workdir is not valid UTF-8"))?,
            ],
            &root,
        )?;
    }

    // ---- Stage 2: configure (cmake) ----------------------------------
    let build_dir = work.join("build");
    std::fs::create_dir_all(&build_dir)
        .with_context(|| format!("create {}", build_dir.display()))?;
    let cmake_arch = format!("-DARCH=xilinx");
    let cmake_family = format!("-DXILINX_FAMILY={family}");
    run_step(
        "cmake",
        &[
            "-S",
            work.to_str()
                .ok_or_else(|| anyhow!("workdir is not valid UTF-8"))?,
            "-B",
            build_dir.to_str()
                .ok_or_else(|| anyhow!("build dir is not valid UTF-8"))?,
            &cmake_arch,
            &cmake_family,
            "-DCMAKE_BUILD_TYPE=Release",
        ],
        &work,
    )?;

    // ---- Stage 3: build chipdb target ---------------------------------
    // openXC7 exposes a `chipdb-xc7a100t` (and similar) target that emits
    // a `.bba` next to the build tree.
    let target = format!("chipdb-{family}");
    let jobs = std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| String::from("2"));
    run_step(
        "cmake",
        &[
            "--build",
            build_dir.to_str()
                .ok_or_else(|| anyhow!("build dir is not valid UTF-8"))?,
            "--target",
            &target,
            "--parallel",
            &jobs,
        ],
        &work,
    )?;

    // ---- Stage 4: locate emitted .bba and install --------------------
    let bba_name = format!("{family}.bba");
    let candidates = [
        build_dir.join(&bba_name),
        build_dir.join("xilinx").join(&bba_name),
        build_dir.join("share").join("himbaechel").join("xilinx").join(&bba_name),
    ];
    let produced = candidates
        .iter()
        .find(|p| p.is_file())
        .cloned()
        .ok_or_else(|| anyhow!(
            "chipdb target succeeded but `{bba_name}` was not found in expected locations:\n  {}",
            candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n  ")
        ))?;

    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("create {}", dest_dir.display()))?;
    let installed = dest_dir.join(&bba_name);
    std::fs::copy(&produced, &installed)
        .with_context(|| format!("install {} -> {}", produced.display(), installed.display()))?;
    let size = std::fs::metadata(&installed)?.len();
    println!(
        "[setup-chipdb] OK  {} ({:.1} MiB)",
        installed.display(),
        size as f64 / 1024.0 / 1024.0
    );
    eprintln!("[setup-chipdb] next: `tri fpga build-proxy --install`");
    Ok(())
}

// ---------------------------------------------------------------------------
// build-proxy-docker: Vivado-in-Docker flow targeting the same proxy
// bitstream. Drives the openFPGALoader fork's `spiOverJtag/Makefile` inside
// a container so users on macOS / Apple Silicon (where Vivado is not
// natively available) can still produce a board-specific .bit without
// installing the 90 GiB Vivado toolchain on the host.
// ---------------------------------------------------------------------------

const OPENFPGALOADER_FORK_URL: &str = "https://github.com/gHashTag/openFPGALoader";
const OPENFPGALOADER_FORK_BRANCH: &str = "feat/qmtech-xc7a100t-board";
const DEFAULT_VIVADO_IMAGE: &str = "t27/vivado:webpack";

fn run_cmd(cmd: &mut std::process::Command, label: &str) -> Result<()> {
    eprintln!("[build-proxy-docker] $ {:?}", cmd);
    let status = cmd
        .status()
        .with_context(|| format!("spawn {}", label))?;
    if !status.success() {
        bail!("{} exited with {:?}", label, status);
    }
    Ok(())
}

fn ensure_fork(fork_dir: &std::path::Path) -> Result<()> {
    if fork_dir.join(".git").is_dir() {
        eprintln!(
            "[build-proxy-docker] fork already present at {}; running `git fetch`",
            fork_dir.display()
        );
        let mut fetch = std::process::Command::new("git");
        fetch
            .args(["fetch", "origin", OPENFPGALOADER_FORK_BRANCH])
            .current_dir(fork_dir);
        // Non-fatal — user may be offline; warn but proceed with whatever
        // is on disk.
        if let Err(e) = run_cmd(&mut fetch, "git fetch") {
            eprintln!("[build-proxy-docker] warning: git fetch failed: {e}");
        }
        let mut checkout = std::process::Command::new("git");
        checkout
            .args(["checkout", OPENFPGALOADER_FORK_BRANCH])
            .current_dir(fork_dir);
        run_cmd(&mut checkout, "git checkout")?;
        return Ok(());
    }
    if let Some(parent) = fork_dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    eprintln!(
        "[build-proxy-docker] cloning {} (branch {}) into {}",
        OPENFPGALOADER_FORK_URL,
        OPENFPGALOADER_FORK_BRANCH,
        fork_dir.display()
    );
    let mut clone = std::process::Command::new("git");
    clone.args([
        "clone",
        "--branch",
        OPENFPGALOADER_FORK_BRANCH,
        "--depth",
        "1",
        OPENFPGALOADER_FORK_URL,
        fork_dir
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 fork path"))?,
    ]);
    run_cmd(&mut clone, "git clone")?;
    Ok(())
}

fn sha256_hex(path: &std::path::Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(format!("{:x}", h.finalize()))
}

fn gunzip(gz_path: &std::path::Path, out_path: &std::path::Path) -> Result<()> {
    // Shell out to `gunzip -c` (POSIX, ships on macOS and every mainstream
    // Linux). Avoids pulling `flate2` into the `tri` crate for a one-shot
    // decompression on the user's host.
    let gunzip = which("gunzip")?;
    eprintln!(
        "[build-proxy-docker] $ {} -c {} > {}",
        gunzip.display(),
        gz_path.display(),
        out_path.display()
    );
    let out_file = std::fs::File::create(out_path)
        .with_context(|| format!("create {}", out_path.display()))?;
    let status = std::process::Command::new(&gunzip)
        .arg("-c")
        .arg(gz_path)
        .stdout(out_file)
        .status()
        .context("spawn gunzip")?;
    if !status.success() {
        bail!("gunzip exited with {:?}", status);
    }
    Ok(())
}

fn build_proxy_docker(
    fork_dir: Option<&PathBuf>,
    image: Option<&str>,
    install: bool,
    no_platform: bool,
) -> Result<()> {
    let root = repo_root()?;
    let fork_path: PathBuf = match fork_dir {
        Some(p) => p.clone(),
        None => root.join("target").join("openfpgaloader-fork"),
    };
    let image_name = image.unwrap_or(DEFAULT_VIVADO_IMAGE);

    // 1. docker available?
    let docker = which("docker")
        .context("`docker` not found on PATH — install Docker Desktop or Docker Engine")?;

    // 2. clone or refresh the fork
    ensure_fork(&fork_path)?;

    let spi_dir = fork_path.join("spiOverJtag");
    if !spi_dir.is_dir() {
        bail!(
            "expected {} after clone — fork layout changed?",
            spi_dir.display()
        );
    }

    // 3. run the container
    //
    //   docker run --rm \
    //     [--platform linux/amd64] \
    //     -v <fork>:/work -w /work/spiOverJtag \
    //     <image> \
    //     make spiOverJtag_xc7a100tfgg676.bit.gz
    //
    let fork_abs = std::fs::canonicalize(&fork_path)
        .with_context(|| format!("canonicalize {}", fork_path.display()))?;
    let mount = format!(
        "{}:/work",
        fork_abs
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 fork path"))?
    );
    let mut cmd = std::process::Command::new(&docker);
    cmd.arg("run").arg("--rm");
    if !no_platform {
        cmd.args(["--platform", "linux/amd64"]);
    }
    cmd.args(["-v", &mount, "-w", "/work/spiOverJtag", image_name]);
    cmd.args(["make", "spiOverJtag_xc7a100tfgg676.bit.gz"]);
    run_cmd(&mut cmd, "docker run")?;

    let bit_gz = spi_dir.join("spiOverJtag_xc7a100tfgg676.bit.gz");
    if !bit_gz.is_file() {
        bail!(
            "expected artefact not produced: {} (check container output)",
            bit_gz.display()
        );
    }
    let gz_size = std::fs::metadata(&bit_gz)?.len();
    println!(
        "[build-proxy-docker] OK  {} ({:.1} KiB, gzipped)",
        bit_gz.display(),
        gz_size as f64 / 1024.0
    );

    if install {
        let dst = root
            .join("fpga")
            .join("tools")
            .join("bscan_spi_xc7a100t.bit");
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        gunzip(&bit_gz, &dst)?;
        let bit_size = std::fs::metadata(&dst)?.len();
        let digest = sha256_hex(&dst)?;
        println!(
            "[build-proxy-docker] installed -> {} ({:.1} KiB)",
            dst.display(),
            bit_size as f64 / 1024.0
        );
        println!("[build-proxy-docker] sha256 : {}", digest);
        eprintln!("[build-proxy-docker] rebuild to pick up the new embedded bitstream:");
        eprintln!("    cargo build -p tri --release");
    }

    Ok(())
}


// ---------------------------------------------------------------------------
// openFPGALoader-based helpers for Digilent FTDI cables (VID=0x0403:0x6014).
// The in-tree dlc10 driver only supports Xilinx DLC10 (VID=0x03FD), so these
// subcommands use the external openFPGALoader tool.
// ---------------------------------------------------------------------------

fn run_openfpgaloader(
    cable: &str,
    extra_args: &[&str],
    capture: bool,
) -> Result<(std::process::ExitStatus, Option<String>)> {
    let bin = which("openFPGALoader")?;
    let mut cmd = std::process::Command::new(&bin);
    cmd.args(["-c", cable]);
    cmd.args(extra_args);

    eprintln!(
        "[openfpgaloader] $ {} -c {} {}",
        bin.display(),
        cable,
        extra_args.join(" ")
    );

    if capture {
        let output = cmd
            .output()
            .with_context(|| format!("spawn openFPGALoader"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stdout.is_empty() {
            eprintln!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        if !output.status.success() {
            bail!("openFPGALoader exited with {:?}", output.status);
        }
        Ok((output.status, Some(format!("{}\n{}", stdout, stderr))))
    } else {
        let status = cmd
            .status()
            .with_context(|| format!("spawn openFPGALoader"))?;
        if !status.success() {
            bail!("openFPGALoader exited with {:?}", status);
        }
        Ok((status, None))
    }
}

fn load_sram(bit: &PathBuf, cable: &str, part: &str, reset: bool, verbose: bool) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }

    let bit_str = bit.to_str().ok_or_else(|| anyhow!("bitstream path is not UTF-8"))?;
    let mut args: Vec<&str> = Vec::new();
    if verbose {
        args.push("-v");
    }
    if reset {
        args.push("-r");
    }
    args.push("--fpga-part");
    args.push(part);
    args.push(bit_str);

    let (_status, _output) = run_openfpgaloader(cable, &args, true)?;

    eprintln!();
    eprintln!("[load-sram] Load complete. Run `tri fpga stat` to verify DONE.");
    Ok(())
}

fn stat(cable: &str, pre_jtag_reset: bool, repeat: u32) -> Result<()> {
    let mut extra = vec!["--read-register", "STAT"];
    if pre_jtag_reset {
        // --skip-reset is documented for write-flash mode, but pass it
        // anyway; if ignored the tool still reads STAT without a reset in
        // --read-register path on tested openFPGALoader versions.
        extra.push("--skip-reset");
        eprintln!("[stat] reading STAT without JTAG reset/PROGRAM_B pulse");
    }

    let n = repeat.max(1);
    let mut samples = Vec::with_capacity(n as usize);
    for i in 0..n {
        let (_status, output) = run_openfpgaloader(cable, &extra, true)?;
        let text = output.unwrap_or_default();
        let raw = parse_stat_raw(&text)?;
        let bits = StatBits::from_raw(raw);
        samples.push(bits);
        if n > 1 {
            eprintln!("[stat] sample {}/{}: raw=0x{:08X}", i + 1, n, bits.raw);
        }
    }

    let bits = samples.first().cloned().expect("at least one STAT sample");
    println!("== STAT register (openFPGALoader --read-register STAT) ==");
    println!("  samples             : {}", samples.len());
    println!("  raw                 : 0x{:08X}", bits.raw);
    println!("  DONE       [14]     : {}", bits.done as u8);
    println!("  INIT_COMPL [11]     : {}", bits.init_complete as u8);
    println!("  EOS        [4]      : {}", bits.eos as u8);
    println!("  CRC_ERROR  [0]      : {}", bits.crc_error as u8);
    println!("  ID_ERROR   [15]     : {}", bits.id_error as u8);
    println!("  MODE       [2:0]    : 0b{:03b} ({})", bits.mode, mode_name(bits.mode));
    println!("  diagnosis           : {}", bits.diagnose());
    println!();

    if bits.done {
        println!("=> FPGA is configured. DONE=HIGH.");
        Ok(())
    } else {
        eprintln!("=> FPGA is NOT configured. {}", bits.diagnose());
        bail!("DONE=LOW")
    }
}

fn mode_name(mode: u8) -> &'static str {
    match mode {
        0b000 => "JTAG/boundary-scan (M[2:0]=000)",
        0b001 => "Master SPI x1 (M[2:0]=001) — expected for N25Q128 boot",
        0b010 => "Master SPI x2/x4 (M[2:0]=010)",
        0b011 => "Master BPI x8/x16 (M[2:0]=011)",
        0b100 => "NAND (M[2:0]=100)",
        0b101 => "reserved",
        0b110 => "reserved",
        0b111 => "JTAG/boundary-scan (M[2:0]=111)",
        _ => "unknown",
    }
}

fn parse_stat_raw(text: &str) -> Result<u32> {
    for line in text.lines() {
        // openFPGALoader prints:
        //   Register raw value: 0x5000890c
        if let Some(idx) = line.find("Register raw value:") {
            let rest = &line[idx + "Register raw value:".len()..];
            let hex = rest.trim().trim_start_matches("0x").trim();
            return u32::from_str_radix(hex, 16)
                .with_context(|| format!("cannot parse STAT raw value '{}'", hex));
        }
    }
    bail!("openFPGALoader output did not contain 'Register raw value:'")
}

fn bit_config(bit: &PathBuf) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let root = repo_root()?;
    let script = root.join("scripts").join("dump_bit_config.py");
    if !script.is_file() {
        bail!("bitstream config parser not found: {}", script.display());
    }
    let bit_str = bit.to_str().ok_or_else(|| anyhow!("bitstream path is not UTF-8"))?;
    let mut cmd = std::process::Command::new("python3");
    cmd.arg(&script).arg(bit_str);
    eprintln!("[bit-config] $ python3 {} {}", script.display(), bit_str);
    let status = cmd.status().with_context(|| "spawn dump_bit_config.py")?;
    if !status.success() {
        bail!("dump_bit_config.py exited with {:?}", status);
    }
    Ok(())
}

fn round_trip_verify(
    bit: &PathBuf,
    cable: &str,
    part: &str,
    bridge: Option<&PathBuf>,
    freq: u32,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }

    // 1. Determine the raw bitstream payload length after the .bit ASCII header.
    let bit_bytes = std::fs::read(bit)
        .with_context(|| format!("read {}", bit.display()))?;
    let sync_idx = find_sync_word(&bit_bytes)
        .ok_or_else(|| anyhow!("sync word 0xAA995566 not found in {}", bit.display()))?;
    let payload = &bit_bytes[sync_idx + 4..];
    let payload_len = payload.len();
    eprintln!(
        "[round-trip] .bit header ends at byte {}; raw payload length = {} bytes",
        sync_idx,
        payload_len
    );

    // 2. Program flash (openFPGALoader strips the .bit header automatically).
    program_flash(bit, cable, part, bridge, freq, None, false, true, false, false, false, Some("1"))?;

    // 3. Dump the same number of bytes back from flash address 0.
    let root = repo_root()?;
    let dump_path = root.join("build").join("fpga").join("gf16").join("round_trip_dump.bin");
    std::fs::create_dir_all(dump_path.parent().unwrap())
        .with_context(|| format!("create {}", dump_path.parent().unwrap().display()))?;
    let size_str = payload_len.to_string();
    let dump_args: Vec<&str> = vec![
        "--dump-flash",
        "--fpga-part",
        part,
        "--file-size",
        &size_str,
        dump_path.to_str().ok_or_else(|| anyhow!("dump path is not UTF-8"))?,
    ];
    let (_status, _output) = run_openfpgaloader(cable, &dump_args, true)?;

    // 4. Align both streams at the sync word and compare the raw configuration
    // payload. openFPGALoader strips the .bit ASCII header and may add the
    // 7-series bus-width auto-detection preamble (0xFF padding + 0x000000BB
    // 0x11220044) in front of the sync word, so a byte-0 comparison is wrong.
    let dump_bytes = std::fs::read(&dump_path)
        .with_context(|| format!("read dumped flash {}", dump_path.display()))?;
    let dump_sync = find_sync_word(&dump_bytes)
        .ok_or_else(|| anyhow!("sync word not found in flash dump {} — dump is all 0xFF?", dump_path.display()))?;
    let dump_tail = &dump_bytes[dump_sync + 4..];
    let bit_tail = payload; // payload already starts right after the .bit sync word
    let cmp_len = dump_tail.len().min(bit_tail.len());
    if cmp_len == 0 {
        bail!("no data after sync word to compare");
    }
    if dump_tail[..cmp_len] == bit_tail[..cmp_len] {
        println!(
            "[round-trip] OK  flash dump aligns at sync word 0x{:08X} and matches .bit payload ({} comparable bytes)",
            dump_sync,
            cmp_len
        );
        Ok(())
    } else {
        let first_diff = dump_tail[..cmp_len].iter().zip(bit_tail[..cmp_len].iter()).position(|(a, b)| a != b);
        let diff_offset = first_diff.unwrap_or(0);
        eprintln!(
            "[round-trip] MISMATCH at byte offset {} after flash sync word (dump 0x{:02X} != bit 0x{:02X})",
            diff_offset,
            dump_tail[diff_offset],
            bit_tail[diff_offset]
        );
        bail!("round-trip verify failed: flash contents differ from bitstream payload")
    }
}

fn find_sync_word(data: &[u8]) -> Option<usize> {
    const SYNC: &[u8] = b"\xaa\x99\x55\x66";
    data.windows(SYNC.len()).position(|w| w == SYNC)
}

fn boot_log(
    bit: &PathBuf,
    cable: &str,
    part: &str,
    bridge: Option<&PathBuf>,
    freq: u32,
    repeat: u32,
    _wait_seconds: u32,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }

    eprintln!("[boot-log] Step 1/4: program flash with {}", bit.display());
    program_flash(
        bit,
        cable,
        part,
        bridge,
        freq,
        None,
        false,
        true,
        false,
        false,
        false,
        Some("1"),
    )?;

    eprintln!();
    eprintln!("[boot-log] Step 2/4: PHYSICAL POWER-CYCLE REQUIRED");
    eprintln!("  1. Disconnect the board's USB power / barrel jack.");
    eprintln!("  2. Wait at least 10 seconds for all rails to collapse.");
    eprintln!("  3. Reconnect power.");
    eprintln!("  4. Do NOT press the FPGA's PROG_B or RESET button.");
    eprintln!("  5. Press ENTER here when the board is powered and stable.");
    eprintln!();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("waiting for user confirmation after power-cycle")?;

    eprintln!("[boot-log] Step 3/4: capture STAT without JTAG reset ({} sample[s])", repeat.max(1));
    let stat_result = stat(cable, true, repeat);

    eprintln!();
    eprintln!("[boot-log] Step 4/4: decision tree");
    match stat_result {
        Ok(()) => {
            eprintln!("  SUCCESS: cold-POR STAT shows DONE=1.");
            eprintln!("  => The board boots from flash. No further mode-pin work needed.");
            Ok(())
        }
        Err(_) => {
            eprintln!("  DONE=0 after cold-POR. Possible causes:");
            eprintln!("    A. Mode-pin strapping: check that M[2:0]=001 (Master SPI x1) is sampled.");
            eprintln!("       Run `tri fpga stat --pre-jtag-reset --repeat 5` immediately after power-on.");
            eprintln!("       If MODE != 001, inspect board resistors/jumpers or add external straps.");
            eprintln!("    B. CCLK/SPI-startup timing: if MODE=001 and still DONE=0, the N25Q128");
            eprintln!("       may not respond to the default CCLK rate. Next wave should audit");
            eprintln!("       COR0 CFGRATE and add a slow-startup bitstream variant.");
            eprintln!("    C. Signal integrity: verify 3.3 V VCCO_0 and clean CCLK/MISO/MOSI/FCS_B.");
            bail!("cold-POR boot failed — see decision tree above")
        }
    }
}

fn smoke_gate(bit: Option<&PathBuf>, top: &str) -> Result<()> {
    let root = repo_root()?;
    let bit_path = bit.cloned().unwrap_or_else(|| {
        root.join("fpga")
            .join("verilog")
            .join("ternary_mac_demo_top_200t.bit")
    });

    println!("== FPGA board-less smoke gate ==");

    // 1. bit-config audit if the bitstream exists.
    if bit_path.is_file() {
        println!("[smoke-gate] bit-config audit: {}", bit_path.display());
        bit_config(&bit_path)?;
    } else {
        println!(
            "[smoke-gate] SKIP: bitstream not found at {} (run openXC7 flow first)",
            bit_path.display()
        );
    }

    // 2. yosys synthesis smoke on the demo sources if available.
    let verilog_dir: PathBuf = bit_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.join("fpga").join("verilog"));
    let v_paths: Vec<PathBuf> = [
        "ternary_mac_synth.v",
        "ternary_mac_demo_top.v",
    ]
    .iter()
    .map(|f| verilog_dir.join(f))
    .filter(|p| p.is_file())
    .collect();

    if !v_paths.is_empty() && yosys_available() {
        let reads: Vec<String> = v_paths
            .iter()
            .map(|p| format!("read_verilog -sv {}", p.display()))
            .collect();
        let script = format!(
            "{}\nsynth_xilinx -top {} -family xc7\nstat\n",
            reads.join("\n"),
            top
        );
        let ys_path = root.join("build").join("fpga").join("smoke_gate.ys");
        std::fs::create_dir_all(ys_path.parent().unwrap())
            .with_context(|| format!("create {}", ys_path.parent().unwrap().display()))?;
        std::fs::write(&ys_path, script)
            .with_context(|| format!("write {}", ys_path.display()))?;
        let status = std::process::Command::new("yosys")
            .arg("-q")
            .arg("-s")
            .arg(&ys_path)
            .status()
            .context("spawning yosys for smoke gate")?;
        if !status.success() {
            bail!("yosys rejected demo Verilog");
        }
        println!("[smoke-gate] yosys synthesis OK");
    } else if v_paths.is_empty() {
        println!("[smoke-gate] SKIP: demo Verilog sources not found");
    } else {
        println!("[smoke-gate] SKIP: yosys not on PATH");
    }

    println!("[smoke-gate] complete");
    Ok(())
}

fn yosys_available() -> bool {
    std::process::Command::new("yosys")
        .arg("-q")
        .arg("-p")
        .arg("echo on")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn program_flash(
    bit: &PathBuf,
    cable: &str,
    part: &str,
    bridge: Option<&PathBuf>,
    freq: u32,
    file_type: Option<&str>,
    skip_reset: bool,
    verify: bool,
    bulk_erase: bool,
    enable_quad: bool,
    disable_quad: bool,
    spi_buswidth: Option<&str>,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    if enable_quad && disable_quad {
        bail!("--enable-quad and --disable-quad are mutually exclusive");
    }

    let bit_str = bit.to_str().ok_or_else(|| anyhow!("bitstream path is not UTF-8"))?;
    let mut args: Vec<String> = Vec::new();
    args.push("-f".to_string());
    args.push("--freq".to_string());
    args.push(freq.to_string());
    args.push("--fpga-part".to_string());
    args.push(part.to_string());

    if let Some(ty) = file_type {
        args.push("--file-type".to_string());
        args.push(ty.to_string());
    }
    if let Some(b) = bridge {
        let b_str = b.to_str().ok_or_else(|| anyhow!("bridge path is not UTF-8"))?;
        args.push("-B".to_string());
        args.push(b_str.to_string());
    }
    if skip_reset {
        args.push("--skip-reset".to_string());
    }
    if verify {
        args.push("--verify".to_string());
    }
    if bulk_erase {
        args.push("--bulk-erase".to_string());
    }
    if enable_quad {
        args.push("--enable-quad".to_string());
    }
    if disable_quad {
        args.push("--disable-quad".to_string());
    }
    args.push(bit_str.to_string());

    if let Some(w) = spi_buswidth {
        eprintln!(
            "[program-flash] bitstream expects SPI x{}; ensure the flash QE bit and board straps match",
            w
        );
    }

    let extra: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (_status, _output) = run_openfpgaloader(cable, &extra, true)?;

    eprintln!();
    eprintln!("[program-flash] Write complete. Reset or power-cycle the board to load from flash.");
    Ok(())
}

fn dump_flash(out: &PathBuf, cable: &str, part: &str, size: usize) -> Result<()> {
    let out_str = out.to_str().ok_or_else(|| anyhow!("output path is not UTF-8"))?;
    let size_str = size.to_string();
    let extra: Vec<&str> = vec![
        "--dump-flash",
        "--fpga-part",
        part,
        "--file-size",
        &size_str,
        out_str,
    ];
    let (_status, _output) = run_openfpgaloader(cable, &extra, true)?;
    eprintln!();
    eprintln!("[dump-flash] Dump complete: {}", out.display());
    Ok(())
}

fn flash_status(cable: &str, part: &str) -> Result<()> {
    eprintln!(
        "[flash-status] Probing SPI flash via openFPGALoader. \
         openFPGALoader does not expose a raw RDSR (0x05) read, so this command \
         reports the detected flash chip and guidance for reading the status register."
    );

    let extra: Vec<&str> = vec![
        "-f",
        "--detect",
        "--fpga-part",
        part,
    ];
    let (_status, output) = run_openfpgaloader(cable, &extra, true)?;
    let text = output.unwrap_or_default();

    // Best-effort parse of any JEDEC / flash-id line.
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("flash")
            || lower.contains("jedec")
            || lower.contains("manufacturer")
            || lower.contains("device")
            || lower.contains("idcode")
        {
            println!("{}", line.trim());
        }
    }

    println!();
    println!("Flash status register decode is not available through openFPGALoader.");
    println!("Recommended alternatives:");
    println!("  1. Use a dedicated SPI programmer (e.g. flashrom -p ft2232_spi:type=232H,port=A)");
    println!("     to read RDSR (0x05), RDSR2 (0x35), RDSR3 (0x15).");
    println!("  2. Build a 200T JTAG-to-SPI proxy for the in-tree dlc10 driver and run");
    println!("     `tri fpga spi-raw 05 --rx 1` to read SR1.");
    Ok(())
}

fn synth_gf16(
    build_dir: Option<&PathBuf>,
    chipdb: Option<&PathBuf>,
    part: &str,
) -> Result<()> {
    let root = repo_root()?;
    let build = build_dir.cloned().unwrap_or_else(|| root.join("build").join("fpga").join("gf16"));
    let chipdb_path = chipdb.cloned().unwrap_or_else(|| root.join("build").join("xc7a100tfgg676.bin"));
    let rtl_dir = root.join("fpga").join("vivado");

    if !chipdb_path.is_file() {
        bail!(
            "chipdb not found: {}. Build it first with the OpenXC7 flow.",
            chipdb_path.display()
        );
    }

    let nextpnr = root.join("build").join("nextpnr-xilinx");
    let fasm2frames = root.join("target").join("prjxray").join("utils").join("fasm2frames.py");
    let xc7frames2bit = root
        .join("target")
        .join("prjxray")
        .join("build")
        .join("tools")
        .join("xc7frames2bit");
    let prjxray_db = root.join("target").join("prjxray-db").join("artix7");

    for tool in [&nextpnr, &fasm2frames, &xc7frames2bit] {
        if !tool.is_file() {
            bail!("required tool not found: {}", tool.display());
        }
    }

    std::fs::create_dir_all(&build)
        .with_context(|| format!("create {}", build.display()))?;

    eprintln!("[synth-gf16] build dir : {}", build.display());
    eprintln!("[synth-gf16] chipdb    : {}", chipdb_path.display());
    eprintln!("[synth-gf16] part      : {}", part);

    // Stage 1: yosys synthesis
    let json_path = build.join("gf16_matmul4x4_top.json");
    let yosys_script = format!(
        "read_verilog {add} {mul} {dot4} {matmul} {top}\nsynth_xilinx -family xc7 -top gf16_matmul4x4_top -flatten\nwrite_json {json}\n",
        add = rtl_dir.join("gf16_add.v").display(),
        mul = rtl_dir.join("gf16_mul.v").display(),
        dot4 = rtl_dir.join("gf16_dot4.v").display(),
        matmul = rtl_dir.join("gf16_matmul4x4.v").display(),
        top = rtl_dir.join("gf16_matmul4x4_top.v").display(),
        json = json_path.display(),
    );
    let ys_path = build.join("synth.ys");
    std::fs::write(&ys_path, yosys_script)?;
    run_step("yosys", &["-q", "-s", ys_path.to_str().unwrap()], &build)?;

    // Stage 2: nextpnr-xilinx place & route
    let fasm_path = build.join("gf16_matmul4x4_top.fasm");
    run_step(
        nextpnr.to_str().unwrap(),
        &[
            "--chipdb",
            chipdb_path.to_str().unwrap(),
            "--xdc",
            rtl_dir.join("gf16_matmul4x4_top.xdc").to_str().unwrap(),
            "--json",
            json_path.to_str().unwrap(),
            "--fasm",
            fasm_path.to_str().unwrap(),
            "--ignore-loops",
        ],
        &build,
    )?;

    // Stage 3: fasm2frames
    let frames_path = build.join("gf16_matmul4x4_top.frames");
    let py_bin = root
        .join("target")
        .join("prjxray-venv")
        .join("bin")
        .join("python3");
    if !py_bin.is_file() {
        bail!(
            "prjxray venv python not found: {}. Create it with:\n  python3 -m venv target/prjxray-venv\n  target/prjxray-venv/bin/pip install fasm pyyaml simplejson intervaltree numpy",
            py_bin.display()
        );
    }
    let prjxray_repo = root.join("target").join("prjxray");
    let mut py_env = std::env::vars().collect::<Vec<_>>();
    let pythonpath = format!(
        "{}:{}",
        prjxray_repo.display(),
        prjxray_repo.join("utils").display()
    );
    py_env.push(("PYTHONPATH".to_string(), pythonpath));

    eprintln!(
        "[synth-gf16] $ {} {} --db-root {} --part {} {} > {}",
        py_bin.display(),
        fasm2frames.display(),
        prjxray_db.display(),
        part,
        fasm_path.display(),
        frames_path.display()
    );
    let frames_file = std::fs::File::create(&frames_path)
        .with_context(|| format!("create {}", frames_path.display()))?;
    let mut py_cmd = std::process::Command::new(&py_bin);
    py_cmd
        .arg(&fasm2frames)
        .args(["--db-root", prjxray_db.to_str().unwrap(), "--part", part])
        .arg(fasm_path.to_str().unwrap())
        .stdout(frames_file)
        .current_dir(&build)
        .envs(py_env);
    let py_status = py_cmd
        .status()
        .with_context(|| format!("spawn fasm2frames"))?;
    if !py_status.success() {
        bail!("fasm2frames exited with {:?}", py_status);
    }

    // Stage 4: xc7frames2bit
    let bit_path = build.join("gf16_matmul4x4_top.bit");
    run_step(
        xc7frames2bit.to_str().unwrap(),
        &[
            "--frm_file",
            frames_path.to_str().unwrap(),
            "--output_file",
            bit_path.to_str().unwrap(),
            "--part_file",
            prjxray_db.join(part).join("part.yaml").to_str().unwrap(),
            "--part_name",
            part,
        ],
        &build,
    )?;

    let size = std::fs::metadata(&bit_path)?.len();
    println!(
        "[synth-gf16] OK  {} ({:.1} MiB)",
        bit_path.display(),
        size as f64 / 1024.0 / 1024.0
    );
    Ok(())
}
