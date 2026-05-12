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
        FpgaCmd::ProxyLoad { bit } => proxy_load(bit.as_ref()),
        FpgaCmd::ProxyStatus => proxy_status(),
        FpgaCmd::SpiRaw { hex, rx } => spi_raw(hex, *rx),
        FpgaCmd::IrProbe { ir_hex } => ir_probe(ir_hex),
        FpgaCmd::FlashIdDebug => flash_id_debug(),
        FpgaCmd::BuildProxy { install, src, out } => build_proxy(*install, src.as_ref(), out.as_ref()),
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
    use std::path::Path;
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
    run_step(
        "nextpnr-himbaechel",
        &[
            "--device",
            "xc7a100t-fgg676-2",
            "--json",
            json_path.to_str().unwrap(),
            "-o",
            &format!("xdc={}", xdc.display()),
            "-o",
            &format!("fasm={}", fasm_path.display()),
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
