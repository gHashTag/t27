//! `tri fpga ...` — centralised FPGA programming via the in-tree `dlc10`
//! crate. Replaces `tools/dlc10_jtag.py` and `tools/tri_fpga/cli.py`.
//!
//! All operations use pure-Rust paths through `rusb`; no external tools
//! (Vivado / openFPGALoader) and no Python dependencies are required.

use std::io::{BufRead, Read};
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
        /// With 0 the command waits for keyboard input; with a positive value it
        /// auto-continues after the timeout (operator may press ENTER early).
        #[arg(long, default_value_t = 0)]
        wait_seconds: u32,
        /// Optional PVT context JSON file to embed in the boot log. The context
        /// is not validated against the captured CCLK; it records ambient
        /// conditions for later comparison.
        #[arg(long)]
        pvt_context: Option<PathBuf>,
        /// JSON boot-log directory (default: <repo>/build/fpga).
        #[arg(long)]
        log_dir: Option<PathBuf>,
    },
    /// Deterministic cold-POR boot experiment with optional relay control.
    /// With `--relay-port MOCK` the command writes a deterministic, clearly
    /// labeled mock boot log so CI can exercise the cold-POR JSON path without
    /// touching hardware. Real relay ports are reserved for Variant A/B.
    ColdPor {
        /// Bitstream to associate with the cold-POR experiment.
        bit: PathBuf,
        /// Relay control port. `MOCK` produces a deterministic mock log; any
        /// other value is not yet implemented.
        #[arg(long, default_value = "MOCK")]
        relay_port: String,
        /// Number of consecutive STAT samples to report (default: 3).
        #[arg(long, default_value_t = 3)]
        repeat: u32,
        /// Seconds to wait for the relay/mock power-cycle before sampling STAT.
        /// In MOCK mode this simulates the operator delay and auto-continues
        /// after the timeout (operator may press ENTER early).
        #[arg(long, default_value_t = 0)]
        wait_seconds: u32,
        /// Optional PVT context JSON file to embed in the mock boot log.
        #[arg(long)]
        pvt_context: Option<PathBuf>,
        /// JSON boot-log directory (default: <repo>/build/fpga).
        #[arg(long)]
        log_dir: Option<PathBuf>,
    },
    /// Board-less smoke gate for the FPGA path. Runs `tri fpga bit-config`
    /// on the GF16 demo bitstream and, if yosys is available, a synthesis
    /// smoke check on the demo Verilog. Requires no physical board.
    ///
    /// Asserts the canonical 200T configuration: IDCODE 0x03636093, SPI x1,
    /// CCLK startup, OSCFSEL=0, and no CRC register writes.
    ///
    /// With `--require-cable`, also detect the Digilent FTDI cable, load the
    /// bitstream into FPGA SRAM via openFPGALoader, and assert DONE=HIGH.
    ///
    /// With `--flash-boot`, program the bitstream to SPI flash, prompt for a
    /// physical power-cycle, then capture cold-POR STAT and assert `boot_success`.
    /// `--flash-boot` implies `--require-cable`.
    SmokeGate {
        /// Bitstream to audit (default: fpga/verilog/ternary_mac_demo_top_200t.bit).
        #[arg(long)]
        bit: Option<PathBuf>,
        /// Verilog top module to synthesize (default: ternary_mac_demo_top).
        #[arg(long, default_value = "ternary_mac_demo_top")]
        top: String,
        /// Require a connected Digilent cable and load the bitstream into SRAM.
        /// If no device is detected, the gate fails. Board-less checks still run.
        #[arg(long)]
        require_cable: bool,
        /// Program flash and verify cold-POR boot instead of SRAM load.
        /// Implies `--require-cable` and asks the operator to power-cycle.
        #[arg(long)]
        flash_boot: bool,
        /// Seconds to wait for the operator to perform the cold-POR power-cycle
        /// before capturing STAT. When non-zero, the gate auto-continues after
        /// the timeout (the operator may press ENTER to continue early). A long
        /// wait (e.g. 120 s) is recommended so the FPGA has time to complete
        /// configuration after the cable is reconnected.
        #[arg(long, default_value_t = 0)]
        wait_seconds: u32,
        /// openFPGALoader cable profile for the cable-connected check.
        #[arg(long, default_value = "digilent_hs2")]
        cable: String,
        /// FPGA part/package for openFPGALoader (default: xc7a200tfgg676).
        #[arg(long, default_value = "xc7a200tfgg676")]
        part: String,
    },
    /// Print or interactively confirm the cold-POR boot protocol. This is the
    /// standalone version of the instructions embedded in `boot-log` and
    /// `cclk-sweep`; use it to verify the operator steps before a physical
    /// power-cycle.
    BootProtocol {
        /// Print the checklist and exit (default: interactive confirmation).
        #[arg(long)]
        checklist: bool,
    },
    /// Patch the raw OSCFSEL field (COR0[22:17]) of a 7-series .bit file.
    ///
    /// WARNING: the OSCFSEL-to-MHz mapping is not publicly documented, and
    /// modifying a bitstream with CRC enabled will produce a CRC_ERROR. This
    /// command is intended for experimental CCLK-sweep work only; verify
    /// results on real hardware before committing a default bitstream.
    PatchCor0 {
        /// Input Xilinx .bit file.
        bit: PathBuf,
        /// Output patched .bit file.
        out: PathBuf,
        /// Raw 6-bit OSCFSEL value for COR0[22:17].
        #[arg(long)]
        oscfsel: u8,
    },
    /// Generate a set of OSCFSEL variants from a single input .bit file.
    /// Outputs one file per requested value to the given directory.
    CclkVariants {
        /// Input Xilinx .bit file.
        bit: PathBuf,
        /// Output directory (default: <repo>/build/fpga/cclk_variants).
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Comma-separated list of raw 6-bit OSCFSEL values to produce.
        #[arg(long, value_delimiter = ',')]
        values: Vec<u8>,
    },
    /// Automated cold-POR CCLK sweep. Generates OSCFSEL variants, programs each
    /// to flash, prompts for the physical power-cycle, captures STAT, and writes
    /// JSON logs. The only manual step is disconnecting/reconnecting the cable
    /// and power-cycling the board.
    CclkSweep {
        /// Input Xilinx .bit file.
        bit: PathBuf,
        /// Comma-separated list of raw 6-bit OSCFSEL values to sweep.
        /// Defaults to 0,1,2,3,4,5,6,7.
        #[arg(long, value_delimiter = ',')]
        values: Vec<u8>,
        /// Variant output directory (default: <repo>/build/fpga/cclk_variants).
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// JSON boot-log directory (default: <repo>/build/fpga).
        #[arg(long)]
        log_dir: Option<PathBuf>,
        /// Stop the sweep on the first failure instead of continuing.
        #[arg(long)]
        stop_on_fail: bool,
        /// Do not touch hardware; generate synthetic logs for testing the report path.
        #[arg(long)]
        dry_run: bool,
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
        /// Seconds to wait for the user to power-cycle before auto-continuing
        /// (default: 0, wait forever). Use only when the board state is already
        /// known and you want to skip the interactive prompt.
        #[arg(long, default_value_t = 0)]
        wait_seconds: u32,
        /// Optional PVT context JSON file to embed in each sweep log entry.
        #[arg(long)]
        pvt_context: Option<PathBuf>,
        /// Sweep only a single specified OSCFSEL value and exit. Useful for
        /// testing one variant at a time or for scripting around manual
        /// power-cycles.
        #[arg(long)]
        single: Option<u8>,
    },
    /// Read all `build/fpga/boot-log-*.json` files and produce a markdown sweep
    /// report identifying the first working CCLK variant.
    SweepReport {
        /// Directory containing boot-log JSON files.
        #[arg(long)]
        log_dir: Option<PathBuf>,
        /// Output markdown report path.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Print DSLogic / oscilloscope instructions for measuring the FPGA CCLK
    /// output during Master SPI configuration. Optionally parse a DSView CSV
    /// export or run a live capture via `sigrok-cli` with a connected logic
    /// analyzer (e.g., the Digilent FTDI cable as `ftdi-la`).
    MeasureCclk {
        /// Path to a DSView / PulseView / Saleae CSV export of the CCLK trace.
        #[arg(long)]
        csv: Option<PathBuf>,
        /// Run a live capture using sigrok-cli instead of parsing a CSV.
        #[arg(long)]
        live: bool,
        /// sigrok driver to use for live capture (default: ftdi-la).
        #[arg(long, default_value = "ftdi-la")]
        driver: String,
        /// Logic-analyzer channel to capture (default: ADBUS4 for ftdi-la).
        #[arg(long, default_value = "ADBUS4")]
        channel: String,
        /// Sample rate for live capture, e.g. 10 MHz (default: 10000000).
        #[arg(long, default_value = "10000000")]
        samplerate: u32,
        /// Number of samples to capture (default: 1000000).
        #[arg(long, default_value_t = 1000000)]
        samples: u32,
        /// Fail if the measured CCLK is outside the N25Q128 standard-read spec.
        #[arg(long)]
        validate: bool,
        /// Optional PVT context JSON file. When supplied with --validate, the capture
        /// is checked against PVT-derated SCK low/high bounds instead of the nominal
        /// 6 ns bounds.
        #[arg(long)]
        pvt_context: Option<PathBuf>,
        /// Generate a synthetic 2.5 MHz logic CSV fixture and validate it.
        /// Useful for CI when P12 is not wired to a logic analyzer.
        #[arg(long)]
        synth: bool,
        /// Emit a JSON object with the measured frequency, duty cycle, and the
        /// conservative SCK low/high times used by the Lean formal link.
        #[arg(long)]
        json: bool,
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
    /// Generate a Lean 4 theorem from a `tri fpga measure-cclk --json` output.
    /// The theorem proves that the measured `(frequency, duty)` pair satisfies
    /// `measured_cclk_satisfies_flash_spec` and links it to
    /// `transaction_satisfies_flash_spec`. Useful for turning a real capture
    /// into a machine-checked proof without manual copy-paste.
    MeasuredToLean {
        /// Path to the JSON file emitted by `tri fpga measure-cclk --json`.
        /// Reads from stdin if omitted.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Output file for the generated Lean snippet. Prints to stdout if omitted.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Theorem name prefix (default: "measured_cclk").
        #[arg(long, default_value = "measured_cclk")]
        name: String,
        /// Use the PVT-margin predicate (`measured_cclk_with_margin_satisfies_flash_spec`)
        /// instead of the nominal predicate.
        #[arg(long)]
        margin: bool,
        /// Optional PVT context JSON file. When supplied, the generated theorem uses
        /// the PVT-aware predicate (`measured_cclk_*_with_pvt_satisfies_flash_spec`).
        /// Mutually exclusive with `--margin` and `--pvt-worstcase` because all three
        /// select a derated bound.
        #[arg(long, conflicts_with = "margin", conflicts_with = "pvt_worstcase")]
        pvt_context: Option<PathBuf>,
        /// Use the worst-case documented operating point for PVT validation and the
        /// generated theorem: max temperature, min VCCINT, slow-slow process corner.
        /// This is the corner proven by `pvt_half_ns_worst_case_bound` in Lean 4.
        /// Mutually exclusive with `--margin` and `--pvt-context`.
        #[arg(long, conflicts_with = "margin", conflicts_with = "pvt_context")]
        pvt_worstcase: bool,
        /// Emit a self-contained `.lean` file with imports and namespace instead
        /// of a bare snippet.
        #[arg(long)]
        standalone: bool,
        /// Read raw nanosecond timings (`period_ns`, `low_ns`, `high_ns`) instead
        /// of computing from `freq_hz`/`duty_pct`.
        #[arg(long)]
        raw_ns: bool,
        /// Reject captures that violate the flash timing spec (or PVT-margin spec
        /// if `--margin` is also set) before emitting the theorem. This keeps the
        /// formal pipeline from generating a false proof for an out-of-spec trace.
        #[arg(long)]
        validate: bool,
        /// Parse a sigrok/DSView/PulseView/Saleae logic or analog CSV export and
        /// convert it to a raw-ns theorem. Mutually exclusive with `--file` and `--vcd`.
        #[arg(long, conflicts_with = "file", conflicts_with = "vcd")]
        csv: Option<PathBuf>,
        /// For multi-channel CSV exports, select the active signal channel by
        /// column name (e.g. `voltage`, `cclk_v`, `channel0`). The column name
        /// is matched case-insensitively against the header row. If the header
        /// row names the channel, `--csv-channel` overrides the default voltage
        /// column heuristic.
        #[arg(long)]
        csv_channel: Option<String>,
        /// Sample rate (Hz) for CSV exports whose time column is sample-number
        /// only (0, 1, 2, ...) rather than seconds. Required when the parser
        /// detects a sample-number time column and no other unit is found.
        #[arg(long)]
        csv_samplerate: Option<u32>,
        /// Unit of the voltage column in an analog CSV export. Some instruments
        /// report millivolts (e.g. 0..3300) instead of volts (0..3.3). Use `mv`
        /// to scale the column by 1e-3 before threshold detection. Default: `v`.
        #[arg(long, value_name = "v|mv")]
        csv_voltage_unit: Option<String>,
        /// Parse a VCD file and convert the first (or `--vcd-signal`) scalar or
        /// multi-bit logic net transitions to a raw-ns theorem. Mutually exclusive
        /// with `--file` and `--csv`.
        #[arg(long, conflicts_with = "file", conflicts_with = "csv")]
        vcd: Option<PathBuf>,
        /// VCD signal name to measure (default: the first scalar `$var` net, or
        /// the first bit of the first bus if no scalar net exists).
        #[arg(long)]
        vcd_signal: Option<String>,
        /// For VCD buses, measure the clock on this bit index (default: 0).
        /// Real-valued VCD nets require `--vcd-threshold-v`.
        #[arg(long, default_value_t = 0)]
        vcd_bit: usize,
        /// Voltage threshold (volts) for treating a real-valued VCD net as logic.
        /// Without this, analog nets are rejected.
        #[arg(long)]
        vcd_threshold_v: Option<f64>,
        /// Minimum voltage change (volts) between two consecutive real-valued VCD
        /// samples for the crossing to count as a real transition. Filters noise
        /// near the threshold.
        #[arg(long)]
        vcd_slope_min_v: Option<f64>,
        /// Minimum time difference (seconds) between two accepted transitions.
        /// Filters ringing / bounce near the threshold.
        #[arg(long)]
        vcd_slope_min_s: Option<f64>,
    },
    /// Print the PVT-derated N25Q128_3V SCK low/high bound for a supplied
    /// operating context. Also prints the margin over the nominal 6 ns bound
    /// and warns if the context is outside the documented operating envelope.
    PvtEnvelope {
        /// PVT context JSON file: {"temp_c": ..., "vccint_mv": ...,
        /// "vccaux_mv": ..., "process_corner": "ff" | "tt" | "ss"}.
        /// If omitted, prints the operating envelope bounds and example contexts.
        #[arg(long)]
        pvt_context: Option<PathBuf>,
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
        FpgaCmd::BitConfig { bit } => bit_config(bit, &[]),
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
            pvt_context,
            log_dir,
        } => boot_log(
            bit,
            cable,
            part,
            bridge.as_ref(),
            *freq,
            *repeat,
            *wait_seconds,
            pvt_context.as_ref(),
            log_dir.as_ref(),
        ),
        FpgaCmd::SmokeGate {
            bit,
            top,
            require_cable,
            flash_boot,
            wait_seconds,
            cable,
            part,
        } => smoke_gate(
            bit.as_ref(),
            top,
            *require_cable || *flash_boot,
            *flash_boot,
            *wait_seconds,
            cable,
            part,
        ),
        FpgaCmd::BootProtocol { checklist } => boot_protocol(*checklist),
        FpgaCmd::PatchCor0 { bit, out, oscfsel } => patch_cor0(bit, out, *oscfsel),
        FpgaCmd::CclkVariants { bit, output_dir, values } => {
            cclk_variants(bit, output_dir.as_ref(), values)
        },
        FpgaCmd::CclkSweep {
            bit,
            values,
            output_dir,
            log_dir,
            stop_on_fail,
            dry_run,
            cable,
            part,
            bridge,
            freq,
            repeat,
            wait_seconds,
            pvt_context,
            single,
        } => {
            let results = cclk_sweep(
                bit,
                values,
                output_dir.as_ref(),
                log_dir.as_ref(),
                *stop_on_fail,
                *dry_run,
                cable,
                part,
                bridge.as_ref(),
                *freq,
                *repeat,
                *wait_seconds,
                pvt_context.as_ref(),
                *single,
            )?;
            if !results.iter().any(|r| r.done) {
                bail!("CCLK sweep did not find a working variant");
            }
            Ok(())
        }
        FpgaCmd::SweepReport { log_dir, out } => {
            sweep_report(log_dir.as_ref(), out.as_ref())
        },
        FpgaCmd::MeasureCclk {
            csv,
            live,
            driver,
            channel,
            samplerate,
            samples,
            validate,
            pvt_context,
            synth,
            json,
        } => measure_cclk(
            csv.as_ref(),
            *live,
            driver,
            channel,
            *samplerate,
            *samples,
            *validate,
            pvt_context.as_ref(),
            *synth,
            *json,
        ),
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
        FpgaCmd::MeasuredToLean {
            file,
            out,
            name,
            margin,
            pvt_context,
            pvt_worstcase,
            standalone,
            raw_ns,
            validate,
            csv,
            csv_channel,
            csv_samplerate,
            csv_voltage_unit,
            vcd,
            vcd_signal,
            vcd_bit,
            vcd_threshold_v,
            vcd_slope_min_v,
            vcd_slope_min_s,
        } => measured_to_lean(
            file.as_ref(),
            csv.as_ref(),
            csv_channel.as_deref(),
            *csv_samplerate,
            csv_voltage_unit.as_deref(),
            vcd.as_ref(),
            vcd_signal.as_deref(),
            *vcd_bit,
            vcd_threshold_v.as_ref(),
            vcd_slope_min_v.as_ref(),
            vcd_slope_min_s.as_ref(),
            out.as_ref(),
            name,
            *margin,
            pvt_context.as_ref(),
            *pvt_worstcase,
            *standalone,
            *raw_ns,
            *validate,
        ),
        FpgaCmd::PvtEnvelope { pvt_context } => pvt_envelope(pvt_context.as_ref()),
        FpgaCmd::ColdPor {
            bit,
            relay_port,
            repeat,
            wait_seconds,
            pvt_context,
            log_dir,
        } => cold_por(
            bit,
            relay_port,
            *repeat,
            *wait_seconds,
            pvt_context.as_ref(),
            log_dir.as_ref(),
        ),
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
    let samples = capture_stat(cable, pre_jtag_reset, repeat)?;
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

/// Capture multiple STAT samples via openFPGALoader and return the decoded
/// `StatBits` for each sample. This is used by `boot-log` to build a JSON
/// record of a cold-POR attempt.
fn capture_stat(cable: &str, pre_jtag_reset: bool, repeat: u32) -> Result<Vec<StatBits>> {
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
    Ok(samples)
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

/// Patch the raw OSCFSEL value (COR0[22:17]) in a 7-series `.bit` file.
///
/// The sync word is located, the Type-1 configuration packet stream is walked,
/// and the last Type-1 write to COR0 (register 0x09) is rewritten in place.
/// The ASCII `.bit` header and any frame data are left untouched.
fn patch_cor0(bit: &PathBuf, out: &PathBuf, oscfsel: u8) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    if oscfsel > 0x3F {
        bail!("OSCFSEL must be a 6-bit value (0..63), got {}", oscfsel);
    }

    let mut data = std::fs::read(bit)
        .with_context(|| format!("read {}", bit.display()))?;
    let sync_idx = find_sync_word(&data)
        .ok_or_else(|| anyhow!("sync word 0xAA995566 not found in {}", bit.display()))?;
    let payload_start = sync_idx + 4;
    let word_count = (data.len().saturating_sub(payload_start)) / 4;

    let mut cor0_word_idx: Option<usize> = None;
    let mut cor0_value: Option<u32> = None;
    let mut i = 0usize;
    while i < word_count {
        let w = read_word_be(&data, payload_start + i * 4);
        let pkt_type = ((w >> 29) & 0x7) as u32;
        let opcode = ((w >> 27) & 0x3) as u32;
        if pkt_type == 1 {
            let reg = ((w >> 13) & 0x3FFF) as u32;
            let count = (w & 0x07FF) as usize;
            if reg == 0x09 && opcode == 2 && count > 0 {
                cor0_word_idx = Some(i + count);
                cor0_value = Some(read_word_be(&data, payload_start + (i + count) * 4));
            }
            i += 1 + count;
        } else if pkt_type == 2 {
            let count = (w & 0x07FFFFFF) as usize;
            i += 1 + count;
        } else {
            i += 1;
        }
    }

    let (idx, old_val) = cor0_word_idx
        .zip(cor0_value)
        .ok_or_else(|| anyhow!("no Type-1 write to COR0 (0x09) found in {}", bit.display()))?;

    const OSCFSEL_MASK: u32 = 0x007E_0000;
    let new_val = (old_val & !OSCFSEL_MASK) | (((oscfsel as u32) & 0x3F) << 17);
    write_word_be(&mut data, payload_start + idx * 4, new_val);

    std::fs::write(out, &data)
        .with_context(|| format!("write {}", out.display()))?;

    println!(
        "[patch-cor0] {} -> {}",
        bit.display(),
        out.display()
    );
    println!("  COR0 0x{:08X} -> 0x{:08X}", old_val, new_val);
    println!("  OSCFSEL[22:17] = {}", oscfsel);
    eprintln!("⚠ Warning: OSCFSEL-to-MHz mapping is not publicly documented.");
    eprintln!("⚠ Warning: If CRC is enabled in CTL0, this patch may cause CRC_ERROR.");
    eprintln!("⚠ Verify the result on real hardware before using this bitstream as default.");
    Ok(())
}

/// Generate CCLK-variants of a bitstream by patching COR0[22:17] for each
/// requested raw OSCFSEL value.
fn cclk_variants(
    bit: &PathBuf,
    output_dir: Option<&PathBuf>,
    values: &Vec<u8>,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let dir = match output_dir {
        Some(d) => d.to_path_buf(),
        None => {
            let root = repo_root()?;
            root.join("build").join("fpga").join("cclk_variants")
        }
    };
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create {}", dir.display()))?;

    let values: Vec<u8> = if values.is_empty() {
        vec![0, 1, 2, 3, 4, 5, 6, 7]
    } else {
        values.clone()
    };

    let stem = bit
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("bitstream");

    eprintln!("[cclk-variants] generating {} variant(s) in {}", values.len(), dir.display());
    for v in &values {
        let out = dir.join(format!("{}_oscfsel{:02}.bit", stem, v));
        patch_cor0(bit, &out, *v)?;
    }
    println!(
        "[cclk-variants] {} variant(s) written to {}",
        values.len(),
        dir.display()
    );
    eprintln!("Next step: program each variant to flash and run a cold-POR sweep,");
    eprintln!("capturing STAT with `tri fpga stat --pre-jtag-reset --repeat 5`.");
    Ok(())
}

/// Run an automated cold-POR CCLK sweep over a set of OSCFSEL variants.
///
/// In normal mode this generates variants, programs each to flash, asks the user
/// to perform the cable-disconnect + power-cycle protocol, captures STAT, and
/// writes a JSON log entry for every variant.  In `--dry-run` mode it synthesises
/// log entries from a fake board so the `sweep-report` path can be tested without
/// hardware.
fn cclk_sweep(
    bit: &PathBuf,
    values: &Vec<u8>,
    output_dir: Option<&PathBuf>,
    log_dir: Option<&PathBuf>,
    stop_on_fail: bool,
    dry_run: bool,
    cable: &str,
    part: &str,
    bridge: Option<&PathBuf>,
    freq: u32,
    repeat: u32,
    wait_seconds: u32,
    pvt_context: Option<&PathBuf>,
    single: Option<u8>,
) -> Result<Vec<SweepResult>> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let pvt_ctx = load_optional_pvt_context(pvt_context)?;

    let values: Vec<u8> = if let Some(v) = single {
        if v > 0x3F {
            bail!("OSCFSEL must be a 6-bit value (0..63), got {}", v);
        }
        vec![v]
    } else if values.is_empty() {
        vec![0, 1, 2, 3, 4, 5, 6, 7]
    } else {
        values.clone()
    };
    for v in &values {
        if *v > 0x3F {
            bail!("OSCFSEL must be a 6-bit value (0..63), got {}", v);
        }
    }

    let root = repo_root()?;
    let variant_dir = match output_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("build").join("fpga").join("cclk_variants"),
    };
    std::fs::create_dir_all(&variant_dir)
        .with_context(|| format!("create {}", variant_dir.display()))?;
    let sweep_log_dir = match log_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("build").join("fpga"),
    };
    std::fs::create_dir_all(&sweep_log_dir)
        .with_context(|| format!("create {}", sweep_log_dir.display()))?;

    eprintln!(
        "[cclk-sweep] {} variant(s) will be swept from {}",
        values.len(),
        bit.display()
    );
    if dry_run {
        eprintln!("[cclk-sweep] DRY RUN: no hardware will be touched; synthetic logs will be written.");
    }

    let mut results: Vec<SweepResult> = Vec::with_capacity(values.len());
    let mut first_working_oscfsel: Option<u8> = None;

    for (idx, oscfsel) in values.iter().enumerate() {
        let variant_name = format!(
            "{}_oscfsel{:02}.bit",
            bit.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("bitstream"),
            oscfsel
        );
        let variant_path = variant_dir.join(&variant_name);

        if !dry_run {
            patch_cor0(bit, &variant_path, *oscfsel)?;
        }

        eprintln!();
        eprintln!(
            "[cclk-sweep] variant {}/{}: OSCFSEL={} => {}",
            idx + 1,
            values.len(),
            oscfsel,
            variant_path.display()
        );

        let start_time = chrono::Local::now();
        if dry_run {
            // Synthesise a log entry with a deterministic "first variant works"
            // pattern so the report path is exercisable without a board.
            let fake_done = *oscfsel == values.first().copied().unwrap_or(0);
            let conclusion = if fake_done {
                "DONE=HIGH: board boots from flash"
            } else {
                "H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants"
            };
            let samples = (0..repeat.max(1))
                .map(|i| SweepSample {
                    index: i as usize,
                    raw: if fake_done { 0x50001B8C } else { 0x5000190C },
                    done: fake_done,
                    eos: fake_done,
                    init_complete: fake_done,
                    crc_error: false,
                    id_error: false,
                    mode: 0b001,
                    diagnosis: if fake_done {
                        "FPGA configured".to_string()
                    } else {
                        "FPGA NOT configured".to_string()
                    },
                })
                .collect();
            if fake_done {
                first_working_oscfsel.get_or_insert(*oscfsel);
            }
            let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
            let log = SweepLog {
                timestamp: start_time.to_rfc3339(),
                bitstream: variant_path.to_string_lossy().to_string(),
                oscfsel: *oscfsel,
                cable: cable.to_string(),
                part: part.to_string(),
                freq_hz: freq,
                repeat: repeat.max(1),
                conclusion: conclusion.to_string(),
                samples,
                pvt_context: pvt_json,
                xadc: xadc_context_json("not_read", pvt_ctx.as_ref()),
                pvt_envelope_margin_ns: pvt_envelope_margin_ns(cclk_nominal_hz(*oscfsel)),
                recommendation: recommendation_from_conclusion(conclusion, Some(*oscfsel), first_working_oscfsel),
            };
            write_sweep_log(&log, &sweep_log_dir)?;
            results.push(SweepResult {
                oscfsel: log.oscfsel,
                bitstream: variant_path.clone(),
                done: log.samples.iter().any(|s| s.done),
                mode: Some(log.samples.first().map(|s| s.mode).unwrap_or(0b001)),
                crc_error: log.samples.iter().any(|s| s.crc_error),
                id_error: log.samples.iter().any(|s| s.id_error),
                conclusion: log.conclusion.clone(),
            });
            continue;
        }

        // Program flash and run the interactive cold-POR protocol.
        if let Err(e) = program_flash(
            &variant_path,
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
        ) {
            eprintln!("[cclk-sweep] program-flash failed: {e}");
            let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
            let conclusion = "PROGRAM_FLASH_FAILED";
            let log = SweepLog {
                timestamp: start_time.to_rfc3339(),
                bitstream: variant_path.to_string_lossy().to_string(),
                oscfsel: *oscfsel,
                cable: cable.to_string(),
                part: part.to_string(),
                freq_hz: freq,
                repeat: repeat.max(1),
                conclusion: conclusion.to_string(),
                samples: Vec::new(),
                pvt_context: pvt_json,
                xadc: xadc_context_json("not_read", pvt_ctx.as_ref()),
                pvt_envelope_margin_ns: pvt_envelope_margin_ns(cclk_nominal_hz(*oscfsel)),
                recommendation: recommendation_from_conclusion(conclusion, Some(*oscfsel), first_working_oscfsel),
            };
            write_sweep_log(&log, &sweep_log_dir)?;
            results.push(SweepResult {
                oscfsel: *oscfsel,
                bitstream: variant_path,
                done: false,
                mode: None,
                crc_error: false,
                id_error: false,
                conclusion: log.conclusion.clone(),
            });
            if stop_on_fail {
                bail!("stopping sweep because --stop-on-fail was set");
            }
            continue;
        }

        eprintln!();
        eprintln!("[cclk-sweep] PHYSICAL POWER-CYCLE REQUIRED");
        eprintln!("  1. Disconnect the JTAG/programming cable from the board.");
        eprintln!("     (An attached cable can hold TMS/TCK/PROGRAM_B and corrupt cold-POR");
        eprintln!("      mode sampling. See AR66954 / XAPP1188.)");
        eprintln!("  2. Disconnect the board's USB power / barrel jack.");
        eprintln!("  3. Wait at least 10 seconds for all rails to collapse.");
        eprintln!("  4. Reconnect power.");
        eprintln!("  5. Do NOT press the FPGA's PROG_B or RESET button.");
        eprintln!("  6. Wait at least 2 seconds, then reconnect the JTAG cable.");
        if wait_seconds > 0 {
            eprintln!(
                "  7. Auto-continuing after {} seconds (press ENTER to continue early).",
                wait_seconds
            );
        } else {
            eprintln!("  7. Press ENTER here when the board and cable are stable.");
        }
        eprintln!();

        wait_for_continue(wait_seconds, "cclk-sweep")?;

        match capture_stat(cable, true, repeat) {
            Ok(samples) => {
                let bits = samples.first().cloned().expect("at least one STAT sample");
                let conclusion = if samples.iter().any(|b| b.done) {
                    "DONE=HIGH: board boots from flash".to_string()
                } else if samples.iter().any(|b| b.mode != 0b001) {
                    "MODE_MISMATCH: mode-pin strapping issue".to_string()
                } else {
                    "H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants".to_string()
                };
                if samples.iter().any(|b| b.done) {
                    first_working_oscfsel.get_or_insert(*oscfsel);
                }
                let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
                let log = SweepLog {
                    timestamp: start_time.to_rfc3339(),
                    bitstream: variant_path.to_string_lossy().to_string(),
                    oscfsel: *oscfsel,
                    cable: cable.to_string(),
                    part: part.to_string(),
                    freq_hz: freq,
                    repeat: repeat.max(1),
                    conclusion: conclusion.clone(),
                    samples: samples
                        .iter()
                        .enumerate()
                        .map(|(i, b)| SweepSample {
                            index: i,
                            raw: b.raw,
                            done: b.done,
                            eos: b.eos,
                            init_complete: b.init_complete,
                            crc_error: b.crc_error,
                            id_error: b.id_error,
                            mode: b.mode,
                            diagnosis: b.diagnose(),
                        })
                        .collect(),
                    pvt_context: pvt_json,
                    xadc: xadc_context_json("not_read", pvt_ctx.as_ref()),
                    pvt_envelope_margin_ns: pvt_envelope_margin_ns(cclk_nominal_hz(*oscfsel)),
                    recommendation: recommendation_from_conclusion(&conclusion, Some(*oscfsel), first_working_oscfsel),
                };
                write_sweep_log(&log, &sweep_log_dir)?;
                results.push(SweepResult {
                    oscfsel: *oscfsel,
                    bitstream: variant_path,
                    done: samples.iter().any(|b| b.done),
                    mode: Some(bits.mode),
                    crc_error: samples.iter().any(|b| b.crc_error),
                    id_error: samples.iter().any(|b| b.id_error),
                    conclusion,
                });
            }
            Err(e) => {
                eprintln!("[cclk-sweep] STAT capture failed: {e}");
                let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
                let conclusion = "STAT_CAPTURE_FAILED";
                let log = SweepLog {
                    timestamp: start_time.to_rfc3339(),
                    bitstream: variant_path.to_string_lossy().to_string(),
                    oscfsel: *oscfsel,
                    cable: cable.to_string(),
                    part: part.to_string(),
                    freq_hz: freq,
                    repeat: repeat.max(1),
                    conclusion: conclusion.to_string(),
                    samples: Vec::new(),
                    pvt_context: pvt_json,
                    xadc: xadc_context_json("not_read", pvt_ctx.as_ref()),
                    pvt_envelope_margin_ns: pvt_envelope_margin_ns(cclk_nominal_hz(*oscfsel)),
                    recommendation: recommendation_from_conclusion(conclusion, Some(*oscfsel), first_working_oscfsel),
                };
                write_sweep_log(&log, &sweep_log_dir)?;
                results.push(SweepResult {
                    oscfsel: *oscfsel,
                    bitstream: variant_path,
                    done: false,
                    mode: None,
                    crc_error: false,
                    id_error: false,
                    conclusion: log.conclusion.clone(),
                });
                if stop_on_fail {
                    bail!("stopping sweep because --stop-on-fail was set");
                }
            }
        }
    }

    // Print summary table.
    println!();
    println!("== CCLK sweep summary ==");
    println!("{:-<70}", "");
    println!("{:>8}  {:<30}  {:>6}  {:>6}  {:<30}", "OSCFSEL", "bitstream", "DONE", "MODE", "conclusion");
    println!("{:-<70}", "");
    for r in &results {
        let mode_str = r
            .mode
            .map(|m| format!("0b{:03b}", m))
            .unwrap_or_else(|| "n/a".to_string());
        let bit_name = r
            .bitstream
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        println!(
            "{:>8}  {:<30}  {:>6}  {:>6}  {:<30}",
            r.oscfsel,
            bit_name,
            if r.done { "1" } else { "0" },
            mode_str,
            r.conclusion
        );
    }
    println!("{:-<70}", "");

    if let Some(first) = results.iter().find(|r| r.done) {
        println!();
        println!(
            "=> First working variant: OSCFSEL={} ({})",
            first.oscfsel,
            first.bitstream.display()
        );
        println!("   Next: measure actual CCLK with `tri fpga measure-cclk` and commit this variant as the default.");
    } else {
        println!();
        println!("=> No variant reached DONE=HIGH.");
        println!("   Next: expand the OSCFSEL range, check mode-pin straps, or capture CCLK with a logic analyser.");
    }

    Ok(results)
}

/// A measured CCLK frequency/duty pair, ready for the Lean formal link.
/// `sck_low_ns` and `sck_high_ns` are conservative integer nanoseconds derived
/// from the floating-point instrument estimate; they mirror the definitions in
/// `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct MeasuredCclk {
    freq_hz: u64,
    duty_pct: f64,
    period_ns: u64,
    sck_low_ns: u64,
    sck_high_ns: u64,
    source: String,
}

impl MeasuredCclk {
    /// Build a conservative measured-CCLK record from a frequency (Hz) and duty
    /// cycle (% high). The period is rounded down and the low time is rounded
    /// down; the high time is the remainder so that low + high exactly equals
    /// the conservative period.
    fn new(freq_hz: f64, duty_pct: f64, source: String) -> Self {
        let period_ns = (1.0e9 / freq_hz).floor() as u64;
        let low_ns = (period_ns as f64 * (100.0 - duty_pct) / 100.0).floor() as u64;
        let high_ns = period_ns.saturating_sub(low_ns);
        Self {
            freq_hz: freq_hz.floor() as u64,
            duty_pct,
            period_ns,
            sck_low_ns: low_ns,
            sck_high_ns: high_ns,
            source,
        }
    }
}

/// Raw nanosecond timing record for the `--raw-ns` mode. The user (or an
/// instrument export script) supplies the measured period and low/high times
/// directly; this avoids duty-cycle quantization when the instrument reports
/// timing in nanoseconds rather than as a frequency/duty pair.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct MeasuredCclkRawNs {
    period_ns: u64,
    sck_low_ns: u64,
    sck_high_ns: u64,
    source: String,
}

/// Process corner used for PVT-aware flash timing derating.
/// Mirrors `ProcessCorner` in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ProcessCorner {
    Tt,
    Ff,
    Ss,
}

/// PVT context used for N25Q128_3V timing derating.
/// Mirrors `PvtContext` in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct PvtContext {
    temp_c: i64,
    vccint_mv: u64,
    vccaux_mv: u64,
    process_corner: ProcessCorner,
}

/// Structs used to persist and report a single cold-POR sweep attempt.
#[derive(serde::Serialize, serde::Deserialize)]
struct SweepLog {
    timestamp: String,
    bitstream: String,
    oscfsel: u8,
    cable: String,
    part: String,
    freq_hz: u32,
    repeat: u32,
    conclusion: String,
    samples: Vec<SweepSample>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pvt_context: Option<serde_json::Value>,
    xadc: serde_json::Value,
    /// Nominal CCLK half-period margin over the documented PVT worst-case bound,
    /// in nanoseconds. Positive means the nominal timing is safe even at the
    /// worst-case operating point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pvt_envelope_margin_ns: Option<i64>,
    /// Machine-readable next action derived from the conclusion.
    recommendation: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SweepSample {
    index: usize,
    raw: u32,
    done: bool,
    eos: bool,
    init_complete: bool,
    crc_error: bool,
    id_error: bool,
    mode: u8,
    diagnosis: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct SweepResult {
    oscfsel: u8,
    bitstream: PathBuf,
    done: bool,
    mode: Option<u8>,
    crc_error: bool,
    id_error: bool,
    conclusion: String,
}

/// Wait for the operator to continue. With `wait_seconds == 0` the call blocks
/// until ENTER is pressed. With `wait_seconds > 0` it auto-continues after the
/// timeout, while a background stdin reader lets the operator press ENTER to
/// continue early. This is used by `boot-log`, `cold-por`, and `cclk-sweep`.
fn wait_for_continue(wait_seconds: u32, label: &str) -> Result<()> {
    if wait_seconds == 0 {
        eprintln!("  Press ENTER here when the board and cable are stable.");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context(format!("{} waiting for operator confirmation", label))?;
        return Ok(());
    }

    eprintln!(
        "  Auto-continuing after {} seconds (press ENTER to continue early).",
        wait_seconds
    );
    let timeout = std::time::Duration::from_secs(wait_seconds as u64);
    let start = std::time::Instant::now();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    std::thread::Builder::new()
        .name(format!("{}-stdin-wait", label))
        .spawn(move || {
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                let _ = tx.send(());
            }
        })
        .context(format!("{} spawn stdin watcher", label))?;

    loop {
        if rx.try_recv().is_ok() {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            eprintln!("[{}] auto-continuing after {} s", label, wait_seconds);
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// XADC context placeholder. Real XADC readout is not yet implemented; the
/// helper still records whether the values were read from the board or taken
/// from a supplied PVT context file, and the temperature / rail voltages when
/// available.
fn xadc_context_json(source: &str, ctx: Option<&PvtContext>) -> serde_json::Value {
    let temp_c = ctx.map(|c| c.temp_c);
    let vccint_mv = ctx.map(|c| c.vccint_mv);
    let vccaux_mv = ctx.map(|c| c.vccaux_mv);
    serde_json::json!({
        "source": source,
        "temp_c": temp_c,
        "vccint_mv": vccint_mv,
        "vccaux_mv": vccaux_mv,
    })
}

fn write_sweep_log(log: &SweepLog, log_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(log_dir)
        .with_context(|| format!("create {}", log_dir.display()))?;
    let name = format!(
        "boot-log-{}-oscfsel{:02}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S"),
        log.oscfsel
    );
    let path = log_dir.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(log)?)
        .with_context(|| format!("write {}", path.display()))?;
    eprintln!("[cclk-sweep] log written to {}", path.display());
    Ok(())
}

/// Produce a markdown sweep report from all `boot-log-*.json` files in the FPGA
/// build directory.
fn sweep_report(log_dir: Option<&PathBuf>, out: Option<&PathBuf>) -> Result<()> {
    let root = repo_root()?;
    let dir = match log_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("build").join("fpga"),
    };
    if !dir.is_dir() {
        bail!("log directory not found: {}", dir.display());
    }

    let mut entries: Vec<SweepLog> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if name.starts_with("boot-log-") && name.ends_with(".json") {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()))?;
            let log: SweepLog = serde_json::from_str(&text)
                .with_context(|| format!("parse {}", path.display()))?;
            entries.push(log);
        }
    }

    // Sort by OSCFSEL for the report.
    entries.sort_by_key(|e| e.oscfsel);

    let first_working = entries.iter().find(|e| {
        e.samples.iter().any(|s| s.done)
            || e.conclusion.starts_with("DONE=HIGH")
    });

    let mut md = String::new();
    md.push_str("# FPGA cold-POR CCLK sweep report\n\n");
    md.push_str(&format!("Generated: {}\n\n", chrono::Local::now().to_rfc3339()));
    md.push_str(&format!("Variants tested: {}\n\n", entries.len()));

    if let Some(w) = first_working {
        md.push_str(&format!(
            "**First working variant:** OSCFSEL={} (`{}`)\n\n",
            w.oscfsel,
            w.bitstream
        ));
    } else {
        md.push_str("**First working variant:** none reached DONE=HIGH\n\n");
    }

    md.push_str("| OSCFSEL | Bitstream | DONE | MODE | CRC | ID | Conclusion |\n");
    md.push_str("|---------|-----------|------|------|-----|----|------------|\n");

    for e in &entries {
        let any_done = e.samples.iter().any(|s| s.done);
        let mode = e
            .samples
            .first()
            .map(|s| format!("0b{:03b}", s.mode))
            .unwrap_or_else(|| "n/a".to_string());
        let crc = e.samples.iter().any(|s| s.crc_error);
        let id_err = e.samples.iter().any(|s| s.id_error);
        let bit_name = std::path::Path::new(&e.bitstream)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?");
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            e.oscfsel,
            bit_name,
            if any_done { "1" } else { "0" },
            mode,
            if crc { "1" } else { "0" },
            if id_err { "1" } else { "0" },
            e.conclusion.replace('|', "\\|")
        ));
    }

    md.push('\n');
    md.push_str("## Next steps\n\n");
    if first_working.is_some() {
        md.push_str("1. Measure actual CCLK with `tri fpga measure-cclk`.\n");
        md.push_str("2. Rename the working variant to the canonical default bitstream.\n");
        md.push_str("3. Update `fpga/HARDWARE_SSOT.md` with the measured frequency.\n");
    } else {
        md.push_str("1. Expand the OSCFSEL sweep range.\n");
        md.push_str("2. Verify mode-pin straps with `tri fpga stat --pre-jtag-reset`.\n");
        md.push_str("3. Capture CCLK with a logic analyser to confirm the FPGA is driving it.\n");
    }

    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => dir.join(format!(
            "sweep-report-{}.md",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        )),
    };
    std::fs::write(&out_path, &md)
        .with_context(|| format!("write {}", out_path.display()))?;
    println!("[sweep-report] wrote {} variant(s) to {}", entries.len(), out_path.display());
    Ok(())
}

/// Maximum SCK frequency (Hz) the Micron N25Q128_3V supports for the
/// standard SPI Read command used during 7-series Master SPI boot.
const N25Q128_MAX_SCK_HZ: f64 = 50_000_000.0;

/// Minimum sensible CCLK frequency (Hz). Below this the capture is likely noise
/// or the FPGA never drove the pin. Roughly 100 kHz.
const CCLK_MIN_SENSE_HZ: f64 = 100_000.0;

/// Minimum SCK clock-low time (seconds) for the N25Q128_3V standard Read
/// command. Datasheet value: 5.5 ns; rounded up to 6 ns to keep the model
/// integral and conservative.
const N25Q128_MIN_SCK_LOW_S: f64 = 6.0e-9;

/// Minimum SCK clock-high time (seconds) for the N25Q128_3V standard Read
/// command. Datasheet value: 5.5 ns; rounded up to 6 ns to keep the model
/// integral and conservative.
const N25Q128_MIN_SCK_HIGH_S: f64 = 6.0e-9;

/// Sensible absolute duty-cycle range for a valid CCLK. This is a last-resort
/// guard that rejects pathological captures (e.g., 1% pulses) when the
/// frequency-derived N25Q128 bound becomes very loose at low CCLK rates.
const CCLK_MIN_DUTY_PCT: f64 = 10.0;
const CCLK_MAX_DUTY_PCT: f64 = 90.0;

/// Print guidance for measuring the Master SPI CCLK output, run a live capture
/// via sigrok-cli, parse a CSV export, or generate a synthetic fixture to
/// estimate frequency / duty cycle.
fn measure_cclk(
    csv: Option<&PathBuf>,
    live: bool,
    driver: &str,
    channel: &str,
    samplerate: u32,
    samples: u32,
    validate: bool,
    pvt_context: Option<&PathBuf>,
    synth: bool,
    json: bool,
) -> Result<()> {
    println!("== CCLK measurement guide ==");
    println!();
    println!("Target board: QMTech Wukong V1 / XC7A200T-FGG676-1");
    println!("CCLK pin: P12 (CFGCLK / CCLK_0, bank 0, 3.3 V)");
    println!("Ground: any GND pin on the JTAG header or board");
    println!();
    println!("Live capture setup (sigrok-cli):");
    println!("  Driver: {} (use 'dreamsourcelab-dslogic' for DSLogic Plus)", driver);
    println!("  Channel: {} (for ftdi-la use ADBUS4..7, not ADBUS0..3 which are JTAG)", channel);
    println!("  Sample rate: {} Hz", samplerate);
    println!("  Samples: {}", samples);
    println!("  Expected CCLK: active only during FPGA configuration from flash.");
    println!();
    println!("CSV setup:");
    println!("  DSView / PulseView / Saleae export: one analog or logic channel.");
    println!();
    println!("Synthetic fixture setup:");
    println!("  --synth generates a 2.5 MHz square-wave logic CSV and validates it");
    println!("  (no hardware required; useful for CI).");
    println!();

    let (freq_hz, duty_pct, source) = if synth {
        println!("[measure-cclk] generating synthetic 2.5 MHz CCLK fixture ...");
        let tmp = std::env::temp_dir().join(format!("tri_cclk_synthetic_{}.csv", std::process::id()));
        generate_synth_cclk_csv(2_500_000.0, samplerate, 1000, &tmp)?;
        let (f, d) = parse_logic_csv(&tmp, samplerate)?;
        println!("[measure-cclk] wrote synthetic fixture to {}", tmp.display());
        (f, d, format!("synthetic ({} Hz samplerate)", samplerate))
    } else if live {
        println!("[measure-cclk] running live capture via sigrok-cli ...");
        let tmp = std::env::temp_dir().join(format!("tri_cclk_capture_{}.csv", std::process::id()));
        capture_cclk_live(driver, channel, samplerate, samples, &tmp)?;
        let (f, d) = parse_logic_csv(&tmp, samplerate)?;
        println!("[measure-cclk] captured {} samples to {}", samples, tmp.display());
        (f, d, format!("live ({}, {})", driver, channel))
    } else if let Some(path) = csv {
        if !path.is_file() {
            bail!("CSV not found: {}", path.display());
        }
        println!("[measure-cclk] parsing {} ...", path.display());
        // Auto-detect analog vs logic CSV by looking at the first non-comment row.
        let (f, d) = if is_logic_csv(path)? {
            let samplerate = detect_logic_csv_samplerate(path)?.unwrap_or(samplerate);
            parse_logic_csv(path, samplerate)?
        } else {
            parse_cclk_csv(path, None, None, None)?
        };
        (f, d, format!("csv {}", path.display()))
    } else {
        println!("Pass --csv <export.csv> to parse a saved capture, or --live to capture now.");
        return Ok(());
    };

    println!("  Source: {}", source);
    println!("  Estimated frequency: {:.3} MHz", freq_hz / 1e6);
    println!("  Estimated duty cycle: {:.1}%", duty_pct);

    if validate {
        if freq_hz > N25Q128_MAX_SCK_HZ {
            bail!(
                "measured CCLK {:.3} MHz exceeds N25Q128 standard-read limit {:.3} MHz",
                freq_hz / 1e6,
                N25Q128_MAX_SCK_HZ / 1e6
            );
        }
        if freq_hz < CCLK_MIN_SENSE_HZ {
            bail!(
                "measured CCLK {:.3} MHz is below {:.3} MHz; capture looks like noise or no signal",
                freq_hz / 1e6,
                CCLK_MIN_SENSE_HZ / 1e6
            );
        }

        let measured = MeasuredCclk::new(freq_hz, duty_pct, source.clone());

        if let Some(ctx_path) = pvt_context {
            let ctx = parse_pvt_context(ctx_path)?;
            let min_half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
            if !raw_ns_satisfies_flash_spec_pvt(
                measured.period_ns,
                measured.sck_low_ns,
                measured.sck_high_ns,
                &ctx,
            ) {
                bail!(
                    "measured CCLK violates PVT-aware flash spec (min half-period {} ns at {} °C, {} mV, {:?} corner)",
                    min_half_ns,
                    ctx.temp_c,
                    ctx.vccint_mv,
                    ctx.process_corner
                );
            }
            println!(
                "  Validation: OK (PVT-aware, min half-period {} ns at {} °C, {} mV, {:?} corner, {:.1}x below {:.3} MHz limit)",
                min_half_ns,
                ctx.temp_c,
                ctx.vccint_mv,
                ctx.process_corner,
                N25Q128_MAX_SCK_HZ / freq_hz,
                N25Q128_MAX_SCK_HZ / 1e6
            );
        } else {
            // N25Q128 t_CL / t_CH bound: for a measured frequency f and period T,
            // the high time must be ≥ t_CH and the low time must be ≥ t_CL.
            // High time = duty * T, low time = (1 - duty) * T.
            // => duty ∈ [t_CL / T, 1 - t_CH / T]
            // => duty_pct ∈ [100 * t_CL * f, 100 - 100 * t_CH * f].
            let period_s = 1.0 / freq_hz;
            let min_duty_pct = 100.0 * N25Q128_MIN_SCK_LOW_S / period_s;
            let max_duty_pct = 100.0 - 100.0 * N25Q128_MIN_SCK_HIGH_S / period_s;
            let clamped_min_duty_pct = min_duty_pct.max(CCLK_MIN_DUTY_PCT);
            let clamped_max_duty_pct = max_duty_pct.min(CCLK_MAX_DUTY_PCT);
            if duty_pct < clamped_min_duty_pct || duty_pct > clamped_max_duty_pct {
                bail!(
                    "measured duty cycle {:.1}% is outside N25Q128-derived range {:.1}%–{:.1}% (or sensible {:.1}%–{:.1}%)",
                    duty_pct,
                    min_duty_pct,
                    max_duty_pct,
                    CCLK_MIN_DUTY_PCT,
                    CCLK_MAX_DUTY_PCT
                );
            }
            println!(
                "  Validation: OK (CCLK within N25Q128 standard-read spec, {:.1}x below {:.3} MHz limit, duty {:.1}%, N25Q128-derived range {:.1}%–{:.1}%)",
                N25Q128_MAX_SCK_HZ / freq_hz,
                N25Q128_MAX_SCK_HZ / 1e6,
                duty_pct,
                min_duty_pct,
                max_duty_pct
            );
        }
    }

    let measured = MeasuredCclk::new(freq_hz, duty_pct, source);
    if json {
        println!("{}", serde_json::to_string_pretty(&measured)?);
    } else {
        println!("  Formal link: freq_hz={} duty_pct={:.1} sck_low_ns={} sck_high_ns={}",
            measured.freq_hz, measured.duty_pct, measured.sck_low_ns, measured.sck_high_ns);
    }

    Ok(())
}

/// Sanitize a string so it can be used as part of a Lean 4 identifier.
/// Replaces non-alphanumeric characters with underscores and collapses runs.
fn sanitize_lean_ident(s: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    out.trim_matches('_').to_string()
}

/// Validate a raw-ns triple against the N25Q128_3V standard-read timing bounds.
/// `margin` uses the conservative 2× PVT-derated limits (12 ns); otherwise the
/// nominal 6 ns bounds are used. Mirrors the formal predicates in
/// `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
fn raw_ns_satisfies_flash_spec(period_ns: u64, low_ns: u64, high_ns: u64, margin: bool) -> bool {
    if period_ns == 0 || low_ns + high_ns != period_ns {
        return false;
    }
    let min_half_ns: u64 = if margin { 12 } else { 6 };
    let max_freq_hz = 50_000_000_u64;
    let freq_hz = 1_000_000_000_u64 / period_ns;
    freq_hz > 0
        && freq_hz <= max_freq_hz
        && low_ns >= min_half_ns
        && high_ns >= min_half_ns
}

/// Operating-envelope bounds that match the Lean 4 PVT model.
const PVT_TEMP_MIN_C: i64 = -40;
const PVT_TEMP_MAX_C: i64 = 85;
const PVT_VCCINT_MIN_MV: u64 = 900;
const PVT_VCCINT_MAX_MV: u64 = 1100;

/// Conservative temperature derating in nanoseconds: 0.02 ns per °C above -40 °C.
fn n25q128_pvt_temp_derating_ns(temp_c: i64) -> u64 {
    ((temp_c - PVT_TEMP_MIN_C).max(0) as u64 * 2) / 100
}

/// Conservative voltage derating in nanoseconds: 0.005 ns per mV below 1100 mV.
fn n25q128_pvt_voltage_derating_ns(vccint_mv: u64) -> u64 {
    ((PVT_VCCINT_MAX_MV - vccint_mv.min(PVT_VCCINT_MAX_MV)) * 5) / 1000
}

/// Process-corner derating in nanoseconds.
fn n25q128_pvt_process_derating_ns(corner: &ProcessCorner) -> u64 {
    match corner {
        ProcessCorner::Ff => 0,
        ProcessCorner::Tt => 2,
        ProcessCorner::Ss => 4,
    }
}

/// Convert a hexadecimal string (e.g. "FF", "0a") into an equivalent
/// binary string with leading zeros so that bit-indexing matches the VCD
/// LSB-first convention. Returns `None` for invalid hex digits or x/z.
fn hex_to_binary_string(hex: &str) -> Option<String> {
    let mut out = String::new();
    for c in hex.chars() {
        let nibble = match c.to_ascii_lowercase() {
            '0' => "0000",
            '1' => "0001",
            '2' => "0010",
            '3' => "0011",
            '4' => "0100",
            '5' => "0101",
            '6' => "0110",
            '7' => "0111",
            '8' => "1000",
            '9' => "1001",
            'a' => "1010",
            'b' => "1011",
            'c' => "1100",
            'd' => "1101",
            'e' => "1110",
            'f' => "1111",
            _ => return None,
        };
        out.push_str(nibble);
    }
    Some(out)
}

/// PVT-aware minimum SCK low/high time in nanoseconds.
fn n25q128_min_sck_half_ns_pvt(ctx: &PvtContext) -> u64 {
    6 + n25q128_pvt_temp_derating_ns(ctx.temp_c)
        + n25q128_pvt_voltage_derating_ns(ctx.vccint_mv)
        + n25q128_pvt_process_derating_ns(&ctx.process_corner)
}

/// Worst-case documented operating point: max temperature, min VCCINT, slow-slow
/// process corner. This matches `OSCFSEL_WORST_CASE_PVT_CONTEXT` in Lean 4.
fn pvt_worst_case_context() -> PvtContext {
    PvtContext {
        temp_c: PVT_TEMP_MAX_C,
        vccint_mv: PVT_VCCINT_MIN_MV,
        vccaux_mv: 2700,
        process_corner: ProcessCorner::Ss,
    }
}

/// Nominal CCLK frequency in hertz for an Artix-7 Master SPI boot OSCFSEL
/// selection. Mirrors `cclk_nominal_hz` in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
fn cclk_nominal_hz(oscfsel: u8) -> u32 {
    match oscfsel {
        0 => 2_500_000,
        1 => 4_200_000,
        2 => 6_600_000,
        3 => 10_000_000,
        4 => 12_500_000,
        5 => 16_700_000,
        6 => 25_000_000,
        7 => 33_300_000,
        _ => 0,
    }
}

/// PVT envelope margin for a given nominal CCLK frequency: how many nanoseconds
/// the nominal half-period exceeds the worst-case PVT-aware minimum half-period.
/// Returns `None` when the frequency is zero.
fn pvt_envelope_margin_ns(freq_hz: u32) -> Option<i64> {
    if freq_hz == 0 {
        return None;
    }
    let period_ns = 1_000_000_000u64 / (freq_hz as u64);
    let half_ns = period_ns / 2;
    let worst_bound = n25q128_min_sck_half_ns_pvt(&pvt_worst_case_context());
    Some(half_ns as i64 - worst_bound as i64)
}

/// Build a machine-readable recommendation object from a sweep/boot/cold-por
/// conclusion. The action vocabulary is closed so downstream tooling can react
/// without parsing free-form strings.
fn recommendation_from_conclusion(
    conclusion: &str,
    oscfsel: Option<u8>,
    first_working_oscfsel: Option<u8>,
) -> serde_json::Value {
    let action = if conclusion.starts_with("DONE=HIGH") {
        "success"
    } else if conclusion.starts_with("H2_CCLK_TIMING") {
        "try_next_oscfsel"
    } else if conclusion.starts_with("MODE_MISMATCH") {
        "inspect_mode_straps"
    } else if conclusion == "PROGRAM_FLASH_FAILED" {
        "check_cable_and_flash"
    } else if conclusion == "STAT_CAPTURE_FAILED" {
        "retry_stat_capture"
    } else {
        "retry_or_debug"
    };
    let mut steps = Vec::new();
    if action == "try_next_oscfsel" {
        if let Some(current) = oscfsel {
            steps.push(format!(
                "Program and boot the next slower OSCFSEL variant (current = {})",
                current
            ));
        }
        if let Some(first) = first_working_oscfsel {
            steps.push(format!("Use the first working OSCFSEL variant: {}", first));
        }
        steps.push("See fpga/HARDWARE_SSOT.md §3.3 (H2 decision tree)".to_string());
    } else if action == "inspect_mode_straps" {
        steps.push(
            "Inspect board mode-pin straps and add external pull resistors if needed".to_string(),
        );
        steps.push("See fpga/HARDWARE_SSOT.md §3.2".to_string());
    } else if action == "check_cable_and_flash" {
        steps.push("Verify the JTAG cable is connected and the flash chip is detected".to_string());
        steps.push("Run `tri fpga stat` and `tri fpga flash-status`".to_string());
    } else if action == "retry_stat_capture" {
        steps.push("Reconnect the cable and retry STAT capture".to_string());
        steps.push("Ensure the board rails are stable before JTAG operations".to_string());
    }
    serde_json::json!({
        "action": action,
        "oscfsel": oscfsel,
        "first_working_oscfsel": first_working_oscfsel,
        "next_steps": steps,
    })
}

/// Print the PVT-aware SCK low/high bound for a user-supplied context.
fn pvt_envelope(pvt_context: Option<&PathBuf>) -> Result<()> {
    const NOMINAL_HALF_NS: u64 = 6;

    if let Some(path) = pvt_context {
        let ctx = parse_pvt_context(path)?;
        let half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
        let margin_ns = half_ns.saturating_sub(NOMINAL_HALF_NS);
        let corner_str = format!("{:?}", ctx.process_corner).to_lowercase();

        println!("PVT-aware N25Q128_3V SCK timing envelope");
        println!(
            "  context: temp = {} °C, vccint = {} mV, vccaux = {} mV, process corner = {}",
            ctx.temp_c, ctx.vccint_mv, ctx.vccaux_mv, corner_str
        );
        println!("  min SCK low / high = {} ns", half_ns);
        println!("  margin over nominal {} ns = {} ns", NOMINAL_HALF_NS, margin_ns);

        let temp_ok = ctx.temp_c >= PVT_TEMP_MIN_C && ctx.temp_c <= PVT_TEMP_MAX_C;
        let vccint_ok = ctx.vccint_mv >= PVT_VCCINT_MIN_MV && ctx.vccint_mv <= PVT_VCCINT_MAX_MV;
        if !temp_ok || !vccint_ok {
            eprintln!(
                "  WARNING: context is outside the documented operating envelope (temp {}..{} °C, vccint {}..{} mV).",
                PVT_TEMP_MIN_C, PVT_TEMP_MAX_C, PVT_VCCINT_MIN_MV, PVT_VCCINT_MAX_MV
            );
        }
        return Ok(());
    }

    println!("N25Q128_3V SCK timing envelope");
    println!(
        "  operating envelope: temp = {}..{} °C, vccint = {}..{} mV",
        PVT_TEMP_MIN_C, PVT_TEMP_MAX_C, PVT_VCCINT_MIN_MV, PVT_VCCINT_MAX_MV
    );
    println!("  nominal min SCK low / high = {} ns", NOMINAL_HALF_NS);

    let example_ctxs = [
        (
            "best-case (ff corner, 1100 mV, -40 °C)",
            PvtContext {
                temp_c: PVT_TEMP_MIN_C,
                vccint_mv: PVT_VCCINT_MAX_MV,
                vccaux_mv: 2700,
                process_corner: ProcessCorner::Ff,
            },
        ),
        (
            "typical (tt corner, 1000 mV, 25 °C)",
            PvtContext {
                temp_c: 25,
                vccint_mv: 1000,
                vccaux_mv: 2700,
                process_corner: ProcessCorner::Tt,
            },
        ),
        (
            "worst-case (ss corner, 900 mV, +85 °C)",
            PvtContext {
                temp_c: PVT_TEMP_MAX_C,
                vccint_mv: PVT_VCCINT_MIN_MV,
                vccaux_mv: 2700,
                process_corner: ProcessCorner::Ss,
            },
        ),
    ];

    for (label, ctx) in example_ctxs {
        let half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
        println!(
            "  {}: min SCK low / high = {} ns (margin {} ns)",
            label,
            half_ns,
            half_ns.saturating_sub(NOMINAL_HALF_NS)
        );
    }
    println!("\nUse --pvt-context <ctx.json> to compute the bound for a specific context.");
    Ok(())
}

/// Validate a raw-ns triple against the PVT-aware N25Q128_3V timing bounds.
/// `ctx` must be inside the operating envelope; the caller is responsible for
/// envelope preconditions. Mirrors `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec`.
fn raw_ns_satisfies_flash_spec_pvt(period_ns: u64, low_ns: u64, high_ns: u64, ctx: &PvtContext) -> bool {
    if period_ns == 0 || low_ns + high_ns != period_ns {
        return false;
    }
    let min_half_ns = n25q128_min_sck_half_ns_pvt(ctx);
    let max_freq_hz = 50_000_000_u64;
    let freq_hz = 1_000_000_000_u64 / period_ns;
    freq_hz > 0
        && freq_hz <= max_freq_hz
        && low_ns >= min_half_ns
        && high_ns >= min_half_ns
}

/// Helper to parse an optional PVT context JSON file.
fn load_optional_pvt_context(path: Option<&PathBuf>) -> Result<Option<PvtContext>> {
    match path {
        Some(p) => Ok(Some(parse_pvt_context(p)?)),
        None => Ok(None),
    }
}

/// Helper to parse a PVT context JSON file.
fn parse_pvt_context(path: &std::path::Path) -> Result<PvtContext> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read PVT context {}", path.display()))?;
    let ctx: PvtContext = serde_json::from_str(&text)
        .with_context(|| format!("parse PVT context JSON {}", path.display()))?;
    if ctx.temp_c < PVT_TEMP_MIN_C || ctx.temp_c > PVT_TEMP_MAX_C {
        bail!(
            "PVT temp_c {} is outside operating envelope [{}..{}] °C",
            ctx.temp_c, PVT_TEMP_MIN_C, PVT_TEMP_MAX_C
        );
    }
    if ctx.vccint_mv < PVT_VCCINT_MIN_MV || ctx.vccint_mv > PVT_VCCINT_MAX_MV {
        bail!(
            "PVT vccint_mv {} is outside operating envelope [{}..{}] mV",
            ctx.vccint_mv, PVT_VCCINT_MIN_MV, PVT_VCCINT_MAX_MV
        );
    }
    Ok(ctx)
}

/// Format a `PvtContext` as a Lean 4 record literal.
fn format_pvt_context_lean(ctx: &PvtContext) -> String {
    let corner = match ctx.process_corner {
        ProcessCorner::Tt => "ProcessCorner.tt",
        ProcessCorner::Ff => "ProcessCorner.ff",
        ProcessCorner::Ss => "ProcessCorner.ss",
    };
    format!(
        "{{ temp_c := ({} : Int), vccint_mv := {}, vccaux_mv := {}, process_corner := {} }}",
        ctx.temp_c, ctx.vccint_mv, ctx.vccaux_mv, corner
    )
}

/// Read a `MeasuredCclk` JSON record (from `--file` or stdin) and emit a Lean 4
/// theorem that proves the measured pair satisfies the flash spec and links it
/// to `transaction_satisfies_flash_spec`.
fn measured_to_lean(
    file: Option<&PathBuf>,
    csv: Option<&PathBuf>,
    csv_channel: Option<&str>,
    csv_samplerate: Option<u32>,
    csv_voltage_unit: Option<&str>,
    vcd: Option<&PathBuf>,
    vcd_signal: Option<&str>,
    vcd_bit: usize,
    vcd_threshold_v: Option<&f64>,
    vcd_slope_min_v: Option<&f64>,
    vcd_slope_min_s: Option<&f64>,
    out: Option<&PathBuf>,
    name: &str,
    margin: bool,
    pvt_context: Option<&PathBuf>,
    pvt_worstcase: bool,
    standalone: bool,
    raw_ns: bool,
    validate: bool,
) -> Result<()> {
    let mut pvt_ctx: Option<PvtContext> = match pvt_context {
        Some(path) => Some(parse_pvt_context(path)?),
        None => None,
    };
    if pvt_worstcase {
        pvt_ctx = Some(PvtContext {
            temp_c: PVT_TEMP_MAX_C,
            vccint_mv: PVT_VCCINT_MIN_MV,
            vccaux_mv: 2700,
            process_corner: ProcessCorner::Ss,
        });
    }
    let csv_volt_unit = csv_voltage_unit.map(parse_csv_voltage_unit).transpose()?;
    let text = if let Some(path) = csv {
        let (period_ns, low_ns, high_ns) = parse_csv_to_raw_ns(path, csv_channel, csv_samplerate, csv_volt_unit)?;
        if validate {
            if let Some(ref ctx) = pvt_ctx {
                if !raw_ns_satisfies_flash_spec_pvt(period_ns, low_ns, high_ns, ctx) {
                    bail!(
                        "CSV capture {} -> {} ns period / {} ns low / {} ns high violates the PVT-aware flash spec; refusing to generate a false theorem",
                        path.display(),
                        period_ns,
                        low_ns,
                        high_ns
                    );
                }
            } else if !raw_ns_satisfies_flash_spec(period_ns, low_ns, high_ns, margin) {
                bail!(
                    "CSV capture {} -> {} ns period / {} ns low / {} ns high violates the {}flash spec; refusing to generate a false theorem",
                    path.display(),
                    period_ns,
                    low_ns,
                    high_ns,
                    if margin { "PVT-margin " } else { "" }
                );
            }
        }
        let source = format!("csv {}", path.display());
        serde_json::to_string_pretty(&MeasuredCclkRawNs {
            period_ns,
            sck_low_ns: low_ns,
            sck_high_ns: high_ns,
            source,
        })?
    } else if let Some(path) = vcd {
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            path,
            vcd_signal,
            vcd_bit,
            vcd_threshold_v,
            vcd_slope_min_v,
            vcd_slope_min_s,
        )?;
        if validate {
            if let Some(ref ctx) = pvt_ctx {
                if !raw_ns_satisfies_flash_spec_pvt(period_ns, low_ns, high_ns, ctx) {
                    bail!(
                        "VCD capture {} -> {} ns period / {} ns low / {} ns high violates the PVT-aware flash spec; refusing to generate a false theorem",
                        path.display(),
                        period_ns,
                        low_ns,
                        high_ns
                    );
                }
            } else if !raw_ns_satisfies_flash_spec(period_ns, low_ns, high_ns, margin) {
                bail!(
                    "VCD capture {} -> {} ns period / {} ns low / {} ns high violates the {}flash spec; refusing to generate a false theorem",
                    path.display(),
                    period_ns,
                    low_ns,
                    high_ns,
                    if margin { "PVT-margin " } else { "" }
                );
            }
        }
        let source = format!(
            "vcd {} {}",
            path.display(),
            vcd_signal.unwrap_or("first")
        );
        serde_json::to_string_pretty(&MeasuredCclkRawNs {
            period_ns,
            sck_low_ns: low_ns,
            sck_high_ns: high_ns,
            source,
        })?
    } else {
        match file {
            Some(path) => std::fs::read_to_string(path)
                .with_context(|| format!("read {}", path.display()))?,
            None => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .context("read JSON from stdin")?;
                buf
            }
        }
    };

    // Early validation for JSON inputs (raw-ns or freq/duty).
    if validate {
        if raw_ns {
            let m: MeasuredCclkRawNs = serde_json::from_str(&text)
                .context("parse MeasuredCclkRawNs JSON for validation")?;
            if let Some(ref ctx) = pvt_ctx {
                if !raw_ns_satisfies_flash_spec_pvt(m.period_ns, m.sck_low_ns, m.sck_high_ns, ctx) {
                    bail!(
                        "JSON raw-ns capture -> {} ns period / {} ns low / {} ns high violates the PVT-aware flash spec; refusing to generate a false theorem",
                        m.period_ns,
                        m.sck_low_ns,
                        m.sck_high_ns
                    );
                }
            } else if !raw_ns_satisfies_flash_spec(m.period_ns, m.sck_low_ns, m.sck_high_ns, margin) {
                bail!(
                    "JSON raw-ns capture -> {} ns period / {} ns low / {} ns high violates the {}flash spec; refusing to generate a false theorem",
                    m.period_ns,
                    m.sck_low_ns,
                    m.sck_high_ns,
                    if margin { "PVT-margin " } else { "" }
                );
            }
        } else {
            let m: MeasuredCclk = serde_json::from_str(&text)
                .context("parse MeasuredCclk JSON for validation")?;
            let period_ns = 1_000_000_000_u64 / m.freq_hz.max(1);
            let low_ns = m.sck_low_ns;
            let high_ns = m.sck_high_ns;
            if let Some(ref ctx) = pvt_ctx {
                if !raw_ns_satisfies_flash_spec_pvt(period_ns, low_ns, high_ns, ctx) {
                    bail!(
                        "JSON capture -> {} Hz / {:.1}% duty violates the PVT-aware flash spec; refusing to generate a false theorem",
                        m.freq_hz,
                        m.duty_pct
                    );
                }
            } else if !raw_ns_satisfies_flash_spec(period_ns, low_ns, high_ns, margin) {
                bail!(
                    "JSON capture -> {} Hz / {:.1}% duty violates the {}flash spec; refusing to generate a false theorem",
                    m.freq_hz,
                    m.duty_pct,
                    if margin { "PVT-margin " } else { "" }
                );
            }
        }
    }

    let mut lean = String::new();

    if standalone {
        lean.push_str("import Trinity.TernaryFPGABoot\n\n");
        lean.push_str("namespace Trinity.BitstreamConfig\n");
        lean.push('\n');
    }

    if raw_ns {
        let m: MeasuredCclkRawNs = serde_json::from_str(&text)
            .context("parse MeasuredCclkRawNs JSON")?;
        let source_suffix = sanitize_lean_ident(&m.source);
        let theorem_base = if source_suffix.is_empty() {
            format!("{}_{}_{}_{}", name, m.period_ns, m.sck_low_ns, m.sck_high_ns)
        } else {
            format!(
                "{}_{}_{}_{}_{}",
                name, source_suffix, m.period_ns, m.sck_low_ns, m.sck_high_ns
            )
        };

        let (predicate, link_theorem, transaction_ctor) = if pvt_ctx.is_some() {
            (
                "measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec",
                "measured_cclk_from_raw_ns_with_pvt_implies_transaction_ok",
                "measured_boot_transaction_from_raw_ns_with_pvt",
            )
        } else {
            (
                "measured_cclk_from_raw_ns_satisfies_flash_spec",
                "measured_cclk_from_raw_ns_implies_transaction_ok",
                "measured_boot_transaction_from_raw_ns",
            )
        };

        lean.push_str(&format!(
            "/- Generated by `tri fpga measured-to-lean --raw-ns` from source: {} -/\n",
            m.source
        ));
        if let Some(ref ctx) = pvt_ctx {
            lean.push_str(&format!(
                "/- PVT context: {} -/\n",
                format_pvt_context_lean(ctx)
            ));
        }
        lean.push_str(&format!(
            "theorem {}_satisfies_flash_spec :\n",
            theorem_base
        ));
        if let Some(ref ctx) = pvt_ctx {
            lean.push_str(&format!(
                "  {} {} {} {} {} = true := by\n",
                predicate, m.period_ns, m.sck_low_ns, m.sck_high_ns, format_pvt_context_lean(ctx)
            ));
        } else {
            lean.push_str(&format!(
                "  {} {} {} {} = true := by\n",
                predicate, m.period_ns, m.sck_low_ns, m.sck_high_ns
            ));
        }
        lean.push_str("  decide\n");
        lean.push('\n');
        lean.push_str(&format!(
            "theorem {}_transaction_ok (bits : Nat) :\n",
            theorem_base
        ));
        lean.push_str(&format!(
            "  transaction_satisfies_flash_spec ({} {} {} {} bits) = true := by\n",
            transaction_ctor, m.period_ns, m.sck_low_ns, m.sck_high_ns
        ));
        lean.push_str(&format!("  apply {}\n", link_theorem));
        if pvt_ctx.is_some() {
            lean.push_str("  · decide\n");
            lean.push_str("  · decide\n");
            lean.push_str(&format!(
                "  · exact {}_satisfies_flash_spec\n",
                theorem_base
            ));
        } else {
            lean.push_str(&format!(
                "  exact {}_satisfies_flash_spec\n",
                theorem_base
            ));
        }
    } else {
        let m: MeasuredCclk = serde_json::from_str(&text)
            .context("parse MeasuredCclk JSON")?;

        // Round the duty cycle to one decimal place, matching the Rust/Lean
        // conservative integer period conversion. The Lean predicate takes a Nat
        // percentage, so we emit the integer-rounded value used by the formal model.
        let duty_pct_int = m.duty_pct.round() as u64;
        let source_suffix = sanitize_lean_ident(&m.source);
        let theorem_base = if source_suffix.is_empty() {
            format!("{}_{}_{}", name, m.freq_hz, duty_pct_int)
        } else {
            format!("{}_{}_{}_{}", name, source_suffix, m.freq_hz, duty_pct_int)
        };

        let (predicate, link_theorem) = if pvt_ctx.is_some() {
            (
                "measured_cclk_with_pvt_satisfies_flash_spec",
                "measured_cclk_with_pvt_implies_transaction_ok",
            )
        } else if margin {
            (
                "measured_cclk_with_margin_satisfies_flash_spec",
                "measured_cclk_with_margin_implies_transaction_ok",
            )
        } else {
            (
                "measured_cclk_satisfies_flash_spec",
                "measured_cclk_satisfies_flash_spec_implies_transaction_ok",
            )
        };

        lean.push_str(&format!(
            "/- Generated by `tri fpga measured-to-lean` from source: {} -/\n",
            m.source
        ));
        if let Some(ref ctx) = pvt_ctx {
            lean.push_str(&format!(
                "/- PVT context: {} -/\n",
                format_pvt_context_lean(ctx)
            ));
        }
        lean.push_str(&format!(
            "theorem {}_satisfies_flash_spec :\n",
            theorem_base
        ));
        if let Some(ref ctx) = pvt_ctx {
            lean.push_str(&format!(
                "  {} {} {} {} = true := by\n",
                predicate, m.freq_hz, duty_pct_int, format_pvt_context_lean(ctx)
            ));
        } else {
            lean.push_str(&format!(
                "  {} {} {} = true := by\n",
                predicate, m.freq_hz, duty_pct_int
            ));
        }
        lean.push_str("  decide\n");
        lean.push('\n');
        lean.push_str(&format!(
            "theorem {}_transaction_ok (bits : Nat) :\n",
            theorem_base
        ));
        lean.push_str(&format!(
            "  transaction_satisfies_flash_spec (measured_boot_transaction {} {} bits) = true := by\n",
            m.freq_hz, duty_pct_int
        ));
        lean.push_str(&format!("  apply {}\n", link_theorem));
        if pvt_ctx.is_some() {
            lean.push_str("  · decide\n");
            lean.push_str("  · decide\n");
            lean.push_str(&format!(
                "  · exact {}_satisfies_flash_spec\n",
                theorem_base
            ));
        } else {
            lean.push_str(&format!(
                "  exact {}_satisfies_flash_spec\n",
                theorem_base
            ));
        }
    }

    if standalone {
        lean.push('\n');
        lean.push_str("end Trinity.BitstreamConfig\n");
    }

    match out {
        Some(path) => {
            std::fs::write(path, &lean)
                .with_context(|| format!("write {}", path.display()))?;
            println!("[measured-to-lean] wrote Lean snippet to {}", path.display());
        }
        None => print!("{}", lean),
    }

    Ok(())
}

/// Generate a synthetic logic CSV representing a perfect square wave at
/// `freq_hz` with 50% duty cycle. `samples` is the number of logic samples to
/// emit. The CSV uses the sigrok logic format (`logic` header, then one `0` or
/// `1` per line) so `parse_logic_csv` can read it back.
fn generate_synth_cclk_csv(
    freq_hz: f64,
    samplerate: u32,
    samples: usize,
    out: &PathBuf,
) -> Result<()> {
    if samplerate == 0 {
        bail!("samplerate must be > 0");
    }
    if freq_hz <= 0.0 {
        bail!("freq_hz must be > 0");
    }
    let period_samples = samplerate as f64 / freq_hz;
    let mut buf = String::from("logic\n");
    for i in 0..samples {
        let phase = (i as f64 % period_samples) / period_samples;
        let bit = if phase < 0.5 { '1' } else { '0' };
        buf.push(bit);
        buf.push('\n');
    }
    std::fs::write(out, buf)
        .with_context(|| format!("write synthetic fixture {}", out.display()))?;
    Ok(())
}

/// Run a live sigrok-cli capture and write the logic CSV to `out`.
fn capture_cclk_live(
    driver: &str,
    channel: &str,
    samplerate: u32,
    samples: u32,
    out: &PathBuf,
) -> Result<()> {
    let mut cmd = std::process::Command::new("sigrok-cli");
    cmd.arg("--driver").arg(driver);
    cmd.arg("--config")
        .arg(format!("samplerate={}", samplerate));
    cmd.arg("--channels").arg(channel);
    cmd.arg("--samples").arg(samples.to_string());
    cmd.arg("--output-format").arg("csv");
    cmd.arg("--output-file").arg(out);
    eprintln!("[sigrok-cli] $ sigrok-cli {}",
        cmd.get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" "));
    let status = cmd.status().with_context(|| "spawn sigrok-cli")?;
    if !status.success() {
        bail!("sigrok-cli failed (is the logic analyzer connected and the driver correct?)");
    }
    Ok(())
}

/// Return true if the CSV looks like a sigrok logic export (header row is
/// "logic" followed by 0/1 samples).
fn is_logic_csv(path: &PathBuf) -> Result<bool> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        return Ok(trimmed.eq_ignore_ascii_case("logic"));
    }
    Ok(false)
}

/// Try to read the samplerate from a sigrok logic CSV comment line such as
/// `; Samplerate: 10 MHz`.
fn detect_logic_csv_samplerate(path: &PathBuf) -> Result<Option<u32>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let re = regex::Regex::new(r"(?i)samplerate:\s*([0-9]+\.?[0-9]*)\s*(Hz|kHz|MHz|GHz)?")
        .map_err(|e| anyhow::anyhow!("regex: {}", e))?;
    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let value: f64 = caps[1].parse().map_err(|_| anyhow!("invalid samplerate"))?;
            let mult: f64 = match caps.get(2).map(|m| m.as_str().to_lowercase()).as_deref() {
                Some("khz") => 1_000.0,
                Some("mhz") => 1_000_000.0,
                Some("ghz") => 1_000_000_000.0,
                _ => 1.0,
            };
            return Ok(Some((value * mult) as u32));
        }
    }
    Ok(None)
}

/// Parse a sigrok logic CSV (one sample per line, 0 or 1) and estimate frequency
/// and duty cycle given the sample rate in Hz.
fn parse_logic_csv(path: &PathBuf, samplerate: u32) -> Result<(f64, f64)> {
    if samplerate == 0 {
        bail!("samplerate must be > 0");
    }
    let file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let mut samples: Vec<bool> = Vec::new();
    let mut header_seen = false;
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if !header_seen {
            if trimmed.eq_ignore_ascii_case("logic") {
                header_seen = true;
            }
            continue;
        }
        match trimmed {
            "0" => samples.push(false),
            "1" => samples.push(true),
            _ => continue,
        }
    }
    if samples.len() < 4 {
        bail!("too few logic samples to estimate frequency");
    }

    let high_count = samples.iter().filter(|&&v| v).count();
    let low_count = samples.len() - high_count;
    let mut transitions = 0usize;
    for window in samples.windows(2) {
        if window[0] != window[1] {
            transitions += 1;
        }
    }

    let total_time = samples.len() as f64 / samplerate as f64;
    let duty_pct = 100.0 * high_count as f64 / samples.len() as f64;

    // A clean clock with N full periods has 2N transitions (rising + falling).
    let freq_hz = if transitions >= 2 {
        transitions as f64 / (2.0 * total_time)
    } else {
        0.0
    };

    println!(
        "  Logic samples: {} (high {}, low {}, transitions {})",
        samples.len(),
        high_count,
        low_count,
        transitions
    );
    Ok((freq_hz, duty_pct))
}

/// Parse a logic-analyser CSV export and estimate CCLK frequency and duty cycle.
///
/// Supported formats (auto-detected):
/// - DSView analog: two columns `Time,Voltage`.
/// - PulseView / Saleae: header row with time and voltage columns.
/// - Fractional-second, millisecond, microsecond, nanosecond, and sample-number
///   time columns are normalized to seconds using the header name or the data
///   shape. For sample-number exports, `samplerate_hz` must be supplied.
///
/// The first numeric column is treated as time (seconds) and the next numeric
/// column as the signal voltage (volts). Rows with non-numeric fields are
/// skipped.
fn parse_cclk_csv(
    path: &PathBuf,
    channel: Option<&str>,
    samplerate_hz: Option<u32>,
    voltage_unit: Option<CsvVoltageUnit>,
) -> Result<(f64, f64)> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    parse_cclk_csv_reader(reader, channel, samplerate_hz, voltage_unit)
}

/// Time-column unit detected from a logic-analyzer CSV header or data shape.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CsvTimeUnit {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    SampleNumber,
}

impl CsvTimeUnit {
    /// Multiplier to convert the raw time value to seconds.
    fn to_seconds_multiplier(self) -> f64 {
        match self {
            CsvTimeUnit::Seconds => 1.0,
            CsvTimeUnit::Milliseconds => 1.0e-3,
            CsvTimeUnit::Microseconds => 1.0e-6,
            CsvTimeUnit::Nanoseconds => 1.0e-9,
            CsvTimeUnit::SampleNumber => 1.0,
        }
    }
}

/// Voltage-column unit for analog CSV exports. Some instruments (e.g. scope
/// CSVs) report millivolts instead of volts; the multiplier normalises to
/// volts before threshold detection.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CsvVoltageUnit {
    V,
    Mv,
}

impl CsvVoltageUnit {
    /// Multiplier to convert the raw voltage value to volts.
    fn to_volts_multiplier(self) -> f64 {
        match self {
            CsvVoltageUnit::V => 1.0,
            CsvVoltageUnit::Mv => 1.0e-3,
        }
    }
}

fn parse_csv_voltage_unit(s: &str) -> Result<CsvVoltageUnit> {
    match s.to_lowercase().as_str() {
        "v" | "volts" | "volt" => Ok(CsvVoltageUnit::V),
        "mv" | "millivolts" | "millivolt" => Ok(CsvVoltageUnit::Mv),
        _ => bail!("unsupported --csv-voltage-unit '{}'; expected 'v' or 'mv'", s),
    }
}

/// Detect the time-column unit from a header name. Returns `None` when the
/// header is ambiguous (e.g. a bare "time" or "t"), letting the caller fall
/// back to data-shape detection.
fn detect_csv_time_unit_from_header(name: &str) -> Option<CsvTimeUnit> {
    let lower = name.to_lowercase();
    if lower.contains("sample") || lower.contains("index") || lower.contains("point") {
        return Some(CsvTimeUnit::SampleNumber);
    }
    if lower.contains("ms") || lower.contains("millisecond") {
        return Some(CsvTimeUnit::Milliseconds);
    }
    if lower.contains("us") || lower.contains("microsecond") || lower.contains("µs") {
        return Some(CsvTimeUnit::Microseconds);
    }
    if lower.contains("ns") || lower.contains("nanosecond") {
        return Some(CsvTimeUnit::Nanoseconds);
    }
    if lower.contains("sec") || lower.contains("second") || lower == "s" {
        return Some(CsvTimeUnit::Seconds);
    }
    None
}

/// True if the first few time values look like consecutive sample numbers
/// starting from 0 (0, 1, 2, ...). This is used as a fallback when the header
/// does not name the unit.
fn csv_times_look_like_sample_numbers(times: &[Option<f64>]) -> bool {
    let first: Vec<f64> = times
        .iter()
        .copied()
        .flatten()
        .take(4)
        .filter(|v| v.is_finite())
        .collect();
    if first.len() < 4 {
        return false;
    }
    for window in first.windows(2) {
        let expected = window[0] + 1.0;
        if (window[1] - expected).abs() > 1.0e-6 {
            return false;
        }
    }
    first[0].abs() <= 1.0e-6
}

fn parse_cclk_csv_reader<R: std::io::BufRead>(
    reader: R,
    channel: Option<&str>,
    samplerate_hz: Option<u32>,
    voltage_unit: Option<CsvVoltageUnit>,
) -> Result<(f64, f64)> {
    let volts_per_unit = voltage_unit.unwrap_or(CsvVoltageUnit::V).to_volts_multiplier();
    let mut raw_times: Vec<f64> = Vec::new();
    let mut values: Vec<f64> = Vec::new();
    let mut header_seen = false;
    let mut header_named_columns = false;
    let mut header_time_unit: Option<CsvTimeUnit> = None;
    let mut time_idx = 0usize;
    let mut value_idx = 1usize;

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            continue;
        }

        // Detect header row: it contains at least one non-numeric token and a
        // time-like column. A leading metadata row such as "samplerate,100000000"
        // must not be accepted as the header because it has no "time" column.
        let has_header_token = parts.iter().any(|p| {
            let lower = p.to_lowercase();
            lower.contains("time")
                || lower.contains("channel")
                || lower.contains("samplerate")
                || lower.contains("sample")
                || lower.contains("voltage")
                || lower.contains("analog")
                || lower.contains("cclk")
                || lower.contains("vccint")
                || lower.contains("vccaux")
        });
        let has_time_token = parts.iter().any(|p| {
            let lower = p.to_lowercase();
            (lower.contains("time") || lower.contains("timestamp") || lower == "t")
                && !lower.contains("samplerate")
        });
        if has_header_token && has_time_token && !header_seen {
            header_seen = true;
            let header_names: Vec<String> = parts.iter().map(|s| s.to_lowercase()).collect();
            // Prefer the voltage/analog column as the signal value if the header
            // names it explicitly. This fixes multi-channel CSVs where the first
            // numeric column after time is the wrong channel.
            if let Some(target) = channel {
                // Explicit channel selection: match the requested column name.
                let target_lower = target.to_lowercase();
                for (i, name) in header_names.iter().enumerate() {
                    if name == &target_lower || name.contains(&target_lower) {
                        value_idx = i;
                        header_named_columns = true;
                        break;
                    }
                }
            } else {
                // Auto-detect a signal column from common instrument headers.
                for (i, name) in header_names.iter().enumerate() {
                    if name.contains("voltage")
                        || name == "v"
                        || name.contains("analog")
                        || name.contains("cclk_v")
                        || name.contains("vccint")
                        || name.contains("vccaux")
                        || name.contains("ain")
                        || name.contains("a0")
                        || name.contains("channel0")
                    {
                        value_idx = i;
                        header_named_columns = true;
                        break;
                    }
                }
            }
            // Time column is usually named time/s/time_s/timestamp; default to 0.
            for (i, name) in header_names.iter().enumerate() {
                if name.contains("time") || name == "t" || name.contains("timestamp") {
                    time_idx = i;
                    header_named_columns = true;
                    break;
                }
            }
            // Detect time unit from the time-column header if possible.
            if let Some(name) = header_names.get(time_idx) {
                header_time_unit = detect_csv_time_unit_from_header(name);
            }
            continue;
        }

        // Try to parse all columns as f64.
        let parsed: Vec<Option<f64>> = parts
            .iter()
            .map(|p| p.parse::<f64>().ok())
            .collect();

        // If this is the first data row and we have at least two numeric columns,
        // lock the time/value indices only when the header did not name them.
        if !header_seen || (!header_named_columns && raw_times.is_empty() && parsed.iter().filter(|x| x.is_some()).count() >= 2) {
            let numeric_positions: Vec<usize> = parsed
                .iter()
                .enumerate()
                .filter(|(_, v)| v.is_some())
                .map(|(i, _)| i)
                .collect();
            if numeric_positions.len() >= 2 {
                time_idx = numeric_positions[0];
                value_idx = numeric_positions[1];
            }
        }

        if let (Some(t), Some(v)) = (parsed.get(time_idx).copied().flatten(), parsed.get(value_idx).copied().flatten()) {
            raw_times.push(t);
            values.push(v * volts_per_unit);
        }
    }

    // Normalize the time column to seconds. If the header named the unit, use it;
    // otherwise infer from the data shape (sample numbers vs. seconds).
    let time_unit = header_time_unit.unwrap_or_else(|| {
        if csv_times_look_like_sample_numbers(
            &raw_times.iter().map(|t| Some(*t)).collect::<Vec<_>>(),
        ) {
            CsvTimeUnit::SampleNumber
        } else {
            CsvTimeUnit::Seconds
        }
    });
    let seconds_per_unit = match time_unit {
        CsvTimeUnit::SampleNumber => {
            let sr = samplerate_hz.ok_or_else(|| {
                anyhow!(
                    "CSV time column looks like sample numbers but no --csv-samplerate was supplied"
                )
            })?;
            if sr == 0 {
                bail!("--csv-samplerate must be > 0");
            }
            1.0 / sr as f64
        }
        _ => time_unit.to_seconds_multiplier(),
    };
    let times: Vec<f64> = raw_times.iter().map(|t| t * seconds_per_unit).collect();

    if time_unit == CsvTimeUnit::SampleNumber {
        eprintln!(
            "[measured-to-lean] CSV time column treated as sample numbers at {} Hz",
            samplerate_hz.unwrap()
        );
    } else {
        eprintln!(
            "[measured-to-lean] CSV time-column unit detected as {:?}; converted to seconds",
            time_unit
        );
    }
    if voltage_unit == Some(CsvVoltageUnit::Mv) {
        eprintln!("[measured-to-lean] CSV voltage column scaled from mV to V");
    }

    if times.len() < 2 {
        bail!("CSV has too few samples to estimate frequency");
    }

    // Compute a simple threshold midpoint from the min/max observed voltage.
    let vmin = values.iter().copied().fold(f64::INFINITY, f64::min);
    let vmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let threshold = (vmin + vmax) / 2.0;

    // Find zero crossings (rising and falling) and accumulate high/low durations.
    let mut crossings: Vec<(f64, bool)> = Vec::new();
    for i in 1..values.len() {
        let prev_high = values[i - 1] > threshold;
        let curr_high = values[i] > threshold;
        if prev_high != curr_high {
            let t0 = times[i - 1];
            let t1 = times[i];
            let v0 = values[i - 1];
            let v1 = values[i];
            let frac = (threshold - v0) / (v1 - v0);
            let tc = t0 + frac * (t1 - t0);
            crossings.push((tc, curr_high)); // true = now high
        }
    }

    if crossings.len() < 4 {
        bail!("too few zero crossings; check threshold / signal quality");
    }

    let mut high_time = 0.0;
    let mut low_time = 0.0;
    for window in crossings.windows(2) {
        let dt = window[1].0 - window[0].0;
        if window[1].1 {
            // entering high
            low_time += dt;
        } else {
            high_time += dt;
        }
    }
    let total = high_time + low_time;
    if total <= 0.0 {
        bail!("could not measure high/low time");
    }

    let freq_hz = (crossings.len() as f64 - 1.0) / (2.0 * total);
    let duty_pct = 100.0 * high_time / total;
    Ok((freq_hz, duty_pct))
}

/// Convert a frequency/duty estimate into a conservative integer nanosecond
/// (period, low, high) triple. The period is rounded down; the low time is
/// rounded down; the high time is the remainder so `low + high = period`.
fn freq_duty_to_raw_ns(freq_hz: f64, duty_pct: f64) -> (u64, u64, u64) {
    if freq_hz <= 0.0 {
        return (0, 0, 0);
    }
    let period_ns = (1.0e9 / freq_hz).floor() as u64;
    let high_ns = (period_ns as f64 * duty_pct / 100.0).floor() as u64;
    let low_ns = period_ns.saturating_sub(high_ns);
    (period_ns, low_ns, high_ns)
}

/// Parse a logic-analyzer CSV export and return a raw-ns (period, low, high)
/// triple suitable for `tri fpga measured-to-lean --raw-ns`. Logic CSVs use
/// `parse_logic_csv`; analog CSVs use `parse_cclk_csv_reader`.
fn parse_csv_to_raw_ns(
    path: &PathBuf,
    channel: Option<&str>,
    samplerate: Option<u32>,
    voltage_unit: Option<CsvVoltageUnit>,
) -> Result<(u64, u64, u64)> {
    if !path.is_file() {
        bail!("CSV not found: {}", path.display());
    }
    let source = format!("csv {}", path.display());
    if is_logic_csv(path)? {
        let samplerate = detect_logic_csv_samplerate(path)?.unwrap_or(10_000_000);
        let (freq_hz, duty_pct) = parse_logic_csv(path, samplerate)?;
        let (period_ns, low_ns, high_ns) = freq_duty_to_raw_ns(freq_hz, duty_pct);
        println!(
            "[measured-to-lean] logic CSV {} -> {} ns period, {} ns low, {} ns high",
            source, period_ns, low_ns, high_ns
        );
        Ok((period_ns, low_ns, high_ns))
    } else {
        let file = std::fs::File::open(path)
            .with_context(|| format!("open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        let (freq_hz, duty_pct) = parse_cclk_csv_reader(reader, channel, samplerate, voltage_unit)?;
        let (period_ns, low_ns, high_ns) = freq_duty_to_raw_ns(freq_hz, duty_pct);
        println!(
            "[measured-to-lean] analog CSV {} -> {} ns period, {} ns low, {} ns high",
            source, period_ns, low_ns, high_ns
        );
        Ok((period_ns, low_ns, high_ns))
    }
}

/// Check whether a trimmed VCD line ends with the exact token `token`
/// (case-insensitive). Only the last whitespace-delimited token is compared;
/// substring matches inside a larger token do not count. This prevents a
/// `$comment` block that contains the literal text `$end` from being terminated
/// early by the heuristic `ends_with("$end")`.
fn vcd_line_ends_with_token(line: &str, token: &str) -> bool {
    line.split_whitespace()
        .last()
        .map(|t| t.eq_ignore_ascii_case(token))
        .unwrap_or(false)
}

/// Parse a minimal VCD file and return a raw-ns (period, low, high) triple for
/// the requested (or first) net. Supports scalar `$var` wires/regs, multi-bit
/// logic buses (selecting `vcd_bit`), and real-valued nets (with an optional
/// voltage threshold, or auto-threshold when omitted). Handles `$dumpoff` /
/// `$dumpon` by suspending sampling. For real-valued nets, a slope filter can
/// reject transitions whose voltage step or inter-transition time is too small.
fn parse_vcd_to_raw_ns(
    path: &PathBuf,
    signal: Option<&str>,
    vcd_bit: usize,
    vcd_threshold_v: Option<&f64>,
    vcd_slope_min_v: Option<&f64>,
    vcd_slope_min_s: Option<&f64>,
) -> Result<(u64, u64, u64)> {
    if !path.is_file() {
        bail!("VCD not found: {}", path.display());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut timescale_s: f64 = 1.0e-9; // default 1 ns
    // (id, name, size, is_real)
    let mut vars: Vec<(String, String, usize, bool)> = Vec::new();
    let mut in_var = false;
    let mut var_tokens: Vec<String> = Vec::new();
    let mut in_timescale = false;
    let mut ts_tokens: Vec<String> = Vec::new();
    let mut in_date = false;
    let mut in_version = false;
    let mut in_comment = false;
    let mut past_dumpvars = false;
    let mut dumpoff = false;
    let mut current_time_s: f64 = 0.0;
    let mut selected_id: Option<String> = None;
    let mut selected_is_real: bool = false;
    let mut transitions: Vec<(f64, bool)> = Vec::new();
    let mut real_samples: Vec<(f64, f64)> = Vec::new();
    let mut last_high: Option<bool> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();

        // Header sections that must be skipped entirely so their contents are
        // never mistaken for `$var` declarations or value changes. Terminators are
        // matched by exact token only, so embedded `$end`-like strings inside
        // comments do not close the section early.
        if trimmed.to_lowercase().starts_with("$date") {
            in_date = true;
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_date = false;
            }
            continue;
        }
        if in_date {
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_date = false;
            }
            continue;
        }
        if trimmed.to_lowercase().starts_with("$version") {
            in_version = true;
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_version = false;
            }
            continue;
        }
        if in_version {
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_version = false;
            }
            continue;
        }
        if trimmed.to_lowercase().starts_with("$comment") {
            in_comment = true;
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_comment = false;
            }
            continue;
        }
        if in_comment {
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_comment = false;
            }
            continue;
        }

        // Timescale parsing: `$timescale 1 ns $end` (possibly across lines).
        // The terminator is matched by exact token so embedded `$end`-like
        // strings in comments do not close the section early.
        if trimmed.to_lowercase().starts_with("$timescale") {
            in_timescale = true;
            ts_tokens.clear();
            ts_tokens.extend(tokens.iter().skip(1).map(|s| s.to_string()));
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_timescale = false;
                if let Some(num_str) = ts_tokens.first() {
                    if let Ok(num) = num_str.parse::<f64>() {
                        let unit_mult = match ts_tokens.get(1).map(|s| s.to_lowercase()).as_deref() {
                            Some("s") => 1.0,
                            Some("ms") => 1.0e-3,
                            Some("us") => 1.0e-6,
                            Some("ns") => 1.0e-9,
                            Some("ps") => 1.0e-12,
                            Some("fs") => 1.0e-15,
                            other => {
                                eprintln!(
                                    "[measured-to-lean] VCD unknown $timescale unit {:?}; defaulting to 1 ns",
                                    other
                                );
                                1.0e-9
                            }
                        };
                        timescale_s = num * unit_mult;
                    }
                }
            }
            continue;
        }
        if in_timescale {
            ts_tokens.extend(tokens.iter().map(|s| s.to_string()));
            if vcd_line_ends_with_token(trimmed, "$end") {
                in_timescale = false;
                if let Some(num_str) = ts_tokens.first() {
                    if let Ok(num) = num_str.parse::<f64>() {
                        let unit_mult = match ts_tokens.get(1).map(|s| s.to_lowercase()).as_deref() {
                            Some("s") => 1.0,
                            Some("ms") => 1.0e-3,
                            Some("us") => 1.0e-6,
                            Some("ns") => 1.0e-9,
                            Some("ps") => 1.0e-12,
                            Some("fs") => 1.0e-15,
                            other => {
                                eprintln!(
                                    "[measured-to-lean] VCD unknown $timescale unit {:?}; defaulting to 1 ns",
                                    other
                                );
                                1.0e-9
                            }
                        };
                        timescale_s = num * unit_mult;
                    }
                }
            }
            continue;
        }

        // Var declaration: `$var wire 1 ! cclk $end` or `$var real 32 ! v $end`.
        if trimmed.to_lowercase().starts_with("$var") {
            in_var = true;
            var_tokens.clear();
            var_tokens.extend(tokens.iter().skip(1).map(|s| s.to_string()));
            if trimmed.to_lowercase().contains(" $end") || trimmed.to_lowercase().ends_with(" $end") {
                // Single-line var; close below.
            } else if !trimmed.to_lowercase().ends_with("$end") {
                continue;
            }
        }
        if in_var {
            if !trimmed.to_lowercase().starts_with("$var") {
                var_tokens.extend(tokens.iter().map(|s| s.to_string()));
            }
            if trimmed.to_lowercase().ends_with("$end") || trimmed.eq_ignore_ascii_case("$end") {
                in_var = false;
                // Expected tokens: [type, size, id, name, ...attrs..., $end]
                // The name may be an escaped identifier split across tokens, e.g.
                // `\my sig`. Strip the leading backslash used by VCD escaping.
                if var_tokens.len() >= 4 {
                    let vtype = var_tokens[0].to_lowercase();
                    let is_real = vtype == "real" || vtype == "integer";
                    let size = var_tokens[1].parse::<usize>().unwrap_or(0);
                    let mut id = var_tokens[2].clone();
                    let name_end = if var_tokens.last()
                        .map(|s| s.eq_ignore_ascii_case("$end"))
                        .unwrap_or(false)
                    {
                        var_tokens.len().saturating_sub(1)
                    } else {
                        var_tokens.len()
                    };
                    let mut name = var_tokens[3..name_end].join(" ");
                    if name.starts_with('\\') {
                        name = name.trim_start_matches('\\').to_string();
                    }
                    if id.starts_with('\\') {
                        id = id.trim_start_matches('\\').to_string();
                    }
                    vars.push((id.clone(), name.clone(), size, is_real));
                    if selected_id.is_none() {
                        let matches = if let Some(target) = signal {
                            name == target
                        } else {
                            true
                        };
                        if matches {
                            // For scalar signals, accept size == 1. For explicitly
                            // named buses, accept any size and let the user pick
                            // the bit via --vcd-bit.
                            if signal.is_some() || size == 1 {
                                selected_id = Some(id);
                                selected_is_real = is_real;
                            }
                        }
                    }
                }
            } else {
                var_tokens.extend(tokens.iter().map(|s| s.to_string()));
            }
            continue;
        }

        if trimmed.eq_ignore_ascii_case("$dumpvars") {
            past_dumpvars = true;
            continue;
        }
        if trimmed.eq_ignore_ascii_case("$dumpoff") {
            dumpoff = true;
            continue;
        }
        if trimmed.eq_ignore_ascii_case("$dumpon") {
            dumpoff = false;
            continue;
        }
        if trimmed.eq_ignore_ascii_case("$enddefinitions") {
            continue;
        }

        // Data section: timestamp lines `#12345`, value lines `0!`, `1!`,
        // `b101 !`, `r1.23e-9 !`.
        if let Some(ts_str) = trimmed.strip_prefix('#') {
            if let Ok(ts) = ts_str.parse::<f64>() {
                current_time_s = ts * timescale_s;
            }
            continue;
        }
        if !past_dumpvars || dumpoff {
            continue;
        }

        if let Some(sel) = &selected_id {
            // Scalar value change: one token like "1!" or "0$".
            // x/z/X/Z values are indeterminate and must be ignored.
            if tokens.len() == 1 && tokens[0].len() >= 2 && !selected_is_real {
                let tok = tokens[0];
                let value_char = tok.chars().next().unwrap();
                if value_char != '0' && value_char != '1' {
                    continue;
                }
                let id: String = tok.chars().skip(1).collect();
                if id == *sel {
                    let high = value_char == '1';
                    if last_high != Some(high) {
                        transitions.push((current_time_s, high));
                        last_high = Some(high);
                    }
                }
                continue;
            }

            // Bus value change: `b<value> <id>` (e.g. `b0 !`, `b1 !`, `b0001_ !`).
            if tokens.len() == 2 && (tokens[0].starts_with('b') || tokens[0].starts_with('B')) && !selected_is_real {
                let value_str = &tokens[0][1..];
                let id = tokens[1];
                if id == *sel {
                    let bit = vcd_bit.min(value_str.len().saturating_sub(1));
                    let bit_char = value_str.chars().rev().nth(bit).unwrap_or('0');
                    // Only accept deterministic 0/1 bits; x/z skip the transition.
                    if bit_char == '0' || bit_char == '1' {
                        let high = bit_char == '1';
                        if last_high != Some(high) {
                            transitions.push((current_time_s, high));
                            last_high = Some(high);
                        }
                    }
                }
                continue;
            }

            // Hex bus value change: `h<value> <id>` (some tools emit hex).
            if tokens.len() == 2 && (tokens[0].starts_with('h') || tokens[0].starts_with('H')) && !selected_is_real {
                let value_str = &tokens[0][1..];
                let id = tokens[1];
                if id == *sel {
                    if let Some(bin) = hex_to_binary_string(value_str) {
                        let bit = vcd_bit.min(bin.len().saturating_sub(1));
                        let bit_char = bin.chars().rev().nth(bit).unwrap_or('0');
                        if bit_char == '0' || bit_char == '1' {
                            let high = bit_char == '1';
                            if last_high != Some(high) {
                                transitions.push((current_time_s, high));
                                last_high = Some(high);
                            }
                        }
                    }
                }
                continue;
            }

            // Real value change: `r<value> <id>`.
            if tokens.len() == 2 && tokens[0].starts_with('r') && selected_is_real {
                let value_str = &tokens[0][1..];
                let id = tokens[1];
                if id == *sel {
                    if let Ok(v) = value_str.parse::<f64>() {
                        // Collect all real samples; thresholding and slope filtering
                        // are applied uniformly after the full waveform is known.
                        real_samples.push((current_time_s, v));
                    }
                }
                continue;
            }
        }
    }

    // Thresholding and slope filtering for real-valued VCD nets. If no explicit
    // threshold was supplied, compute the midpoint of the observed voltage swing.
    // Then walk consecutive sample pairs, keep only threshold crossings whose
    // voltage step is at least `vcd_slope_min_v` and whose spacing from the last
    // accepted transition is at least `vcd_slope_min_s`.
    if selected_is_real && !real_samples.is_empty() {
        let threshold = vcd_threshold_v.copied().unwrap_or_else(|| {
            let vmin = real_samples.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
            let vmax = real_samples.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max);
            let t = (vmin + vmax) / 2.0;
            eprintln!(
                "[measured-to-lean] VCD real-valued signal auto-threshold: {:.3} V (swing {:.3} V .. {:.3} V)",
                t, vmin, vmax
            );
            t
        });
        let slope_min_v = vcd_slope_min_v.copied().unwrap_or(0.0);
        let slope_min_s = vcd_slope_min_s.copied().unwrap_or(0.0);
        let mut last_accepted_t: Option<f64> = None;
        for window in real_samples.windows(2) {
            let (t0, v0) = window[0];
            let (t1, v1) = window[1];
            let high0 = v0 > threshold;
            let high1 = v1 > threshold;
            if high0 == high1 {
                continue;
            }
            let dv = (v1 - v0).abs();
            if dv < slope_min_v {
                continue;
            }
            // Real-valued VCD samples are events at exact timestamps, so the
            // threshold crossing is associated with the second sample time. Use
            // `t1` directly rather than linear interpolation, which would place
            // the crossing in the middle of an instantaneous step.
            let tc = t1;
            if let Some(last_t) = last_accepted_t {
                if tc - last_t < slope_min_s {
                    continue;
                }
            }
            // A filtered-out intermediate state can leave `last_high` unchanged,
            // so a later unfiltered edge that returns to the same side must not
            // create a spurious duplicate transition.
            if last_high == Some(high1) {
                continue;
            }
            transitions.push((tc, high1));
            last_accepted_t = Some(tc);
            last_high = Some(high1);
        }
    }

    let selected_name = vars
        .iter()
        .find(|(id, _, _, _)| Some(id) == selected_id.as_ref())
        .map(|(_, name, _, _)| name.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let source = format!(
        "vcd {} {}",
        path.display(),
        signal.unwrap_or(&selected_name)
    );

    if selected_id.is_none() {
        if let Some(target) = signal {
            bail!("VCD signal '{}' not found", target);
        } else {
            bail!("VCD has no scalar or selectable logic net");
        }
    }

    if transitions.len() < 4 {
        bail!(
            "VCD has too few transitions for signal '{}' (found {}); need at least 4",
            signal.unwrap_or(&selected_name),
            transitions.len()
        );
    }

    // Compute average period, low, high from transitions.
    let mut low_time = 0.0;
    let mut high_time = 0.0;
    for window in transitions.windows(2) {
        let dt = window[1].0 - window[0].0;
        if window[0].1 {
            high_time += dt;
        } else {
            low_time += dt;
        }
    }
    let total = high_time + low_time;
    if total <= 0.0 {
        bail!("could not measure high/low time from VCD transitions");
    }
    let period_count = (transitions.len() - 1) as f64 / 2.0;
    let period_s = if period_count > 0.0 {
        total / period_count
    } else {
        0.0
    };
    let period_ns = (period_s * 1.0e9).floor() as u64;
    let high_ns = (high_time / total * period_s * 1.0e9).floor() as u64;
    let low_ns = period_ns.saturating_sub(high_ns);
    println!(
        "[measured-to-lean] VCD {} -> {} ns period, {} ns low, {} ns high",
        source, period_ns, low_ns, high_ns
    );
    Ok((period_ns, low_ns, high_ns))
}

fn bit_config(bit: &PathBuf, extra_args: &[&str]) -> Result<()> {
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
    cmd.arg(&script);
    for a in extra_args {
        cmd.arg(*a);
    }
    cmd.arg(bit_str);
    eprintln!("[bit-config] $ python3 {} {} {}", script.display(), extra_args.join(" "), bit_str);
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

fn read_word_be(data: &[u8], idx: usize) -> u32 {
    u32::from_be_bytes([data[idx], data[idx + 1], data[idx + 2], data[idx + 3]])
}

fn write_word_be(data: &mut [u8], idx: usize, value: u32) {
    let bytes = value.to_be_bytes();
    data[idx..idx + 4].copy_from_slice(&bytes);
}

fn boot_log(
    bit: &PathBuf,
    cable: &str,
    part: &str,
    bridge: Option<&PathBuf>,
    freq: u32,
    repeat: u32,
    wait_seconds: u32,
    pvt_context: Option<&PathBuf>,
    log_dir: Option<&PathBuf>,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let pvt_ctx = load_optional_pvt_context(pvt_context)?;
    let root = repo_root()?;
    let boot_log_dir = match log_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("build").join("fpga"),
    };
    std::fs::create_dir_all(&boot_log_dir)
        .with_context(|| format!("create {}", boot_log_dir.display()))?;
    let start_time = chrono::Local::now();

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
    eprintln!("  1. Disconnect the JTAG/programming cable from the board.");
    eprintln!("     (An attached cable can hold TMS/TCK/PROGRAM_B and corrupt cold-POR");
    eprintln!("      mode sampling. See AR66954 / XAPP1188.)");
    eprintln!("  2. Disconnect the board's USB power / barrel jack.");
    eprintln!("  3. Wait at least 10 seconds for all rails to collapse.");
    eprintln!("  4. Reconnect power.");
    eprintln!("  5. Do NOT press the FPGA's PROG_B or RESET button.");
    eprintln!("  6. Wait at least 2 seconds, then reconnect the JTAG cable.");
    let step7 = if wait_seconds == 0 {
        "Press ENTER here when the board and cable are stable".to_string()
    } else {
        format!("Auto-continuing after {} seconds (press ENTER to continue early)", wait_seconds)
    };
    eprintln!("  7. {}.", step7);
    eprintln!();

    wait_for_continue(wait_seconds, "boot-log")?;

    eprintln!("[boot-log] Step 3/4: capture STAT without JTAG reset ({} sample[s])", repeat.max(1));
    let stat_result = capture_stat(cable, true, repeat);

    let samples = match &stat_result {
        Ok(s) => s.clone(),
        Err(_) => Vec::new(),
    };
    let conclusion = if let Ok(s) = &stat_result {
        if s.iter().any(|b| b.done) {
            "DONE=HIGH: board boots from flash"
        } else if s.iter().any(|b| b.mode != 0b001) {
            "MODE_MISMATCH: mode-pin strapping issue"
        } else {
            "H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants"
        }
    } else {
        "STAT_CAPTURE_FAILED"
    };

    // Persist a JSON log entry for later comparison across variants.
    let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
    let log_entry = serde_json::json!({
        "timestamp": start_time.to_rfc3339(),
        "bitstream": bit.to_string_lossy().to_string(),
        "cable": cable,
        "part": part,
        "freq_hz": freq,
        "repeat": repeat.max(1),
        "conclusion": conclusion,
        "samples": samples.iter().enumerate().map(|(i, b)| serde_json::json!({
            "index": i,
            "raw": b.raw,
            "done": b.done,
            "eos": b.eos,
            "init_complete": b.init_complete,
            "crc_error": b.crc_error,
            "id_error": b.id_error,
            "mode": b.mode,
            "diagnosis": b.diagnose(),
        })).collect::<Vec<_>>(),
        "pvt_context": pvt_json,
        "xadc": xadc_context_json("not_read", pvt_ctx.as_ref()),
        "pvt_envelope_margin_ns": Option::<i64>::None,
        "recommendation": recommendation_from_conclusion(conclusion, None, None),
    });
    let log_path = boot_log_dir.join(format!(
        "boot-log-{}.json",
        start_time.format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&log_path, serde_json::to_string_pretty(&log_entry)?)
        .with_context(|| format!("write {}", log_path.display()))?;
    eprintln!("[boot-log] log written to {}", log_path.display());

    eprintln!();
    eprintln!("[boot-log] Step 4/4: decision tree — {}", conclusion);
    match stat_result {
        Ok(samples) if samples.iter().any(|b| b.done) => {
            eprintln!("  SUCCESS: cold-POR STAT shows DONE=1.");
            eprintln!("  => The board boots from flash. No further mode-pin/CCLK work needed.");
            Ok(())
        }
        Ok(samples) if samples.iter().any(|b| b.mode != 0b001) => {
            eprintln!("  MODE MISMATCH: cold-POR sampled M[2:0] != 001.");
            eprintln!("  => Inspect board mode-pin straps (resistors / jumpers) or add external");
            eprintln!("     pull resistors to M0/M1/M2. See fpga/HARDWARE_SSOT.md §3.2.");
            bail!("cold-POR mode mismatch — see decision tree")
        }
        Ok(_) => {
            eprintln!("  DONE=0 after cold-POR with MODE=001. Possible causes:");
            eprintln!("    A. CCLK/SPI-startup timing: the N25Q128 may not respond to the");
            eprintln!("       default CCLK rate. Generate variants with:");
            eprintln!("         tri fpga cclk-variants {}", bit.display());
            eprintln!("       then program each *_oscfsel*.bit and repeat this boot-log.");
            eprintln!("    B. Flash wake-up state: before the power-cycle, issue a software");
            eprintln!("       reset via `tri fpga spi-raw 66` + `tri fpga spi-raw 99`.");
            eprintln!("    C. Signal integrity: verify 3.3 V VCCO_0 and clean CCLK/MISO/MOSI/FCS_B.");
            bail!("cold-POR boot failed — see H2 decision tree")
        }
        Err(e) => {
            eprintln!("  STAT capture failed: {e}");
            bail!("cold-POR boot failed — STAT capture error")
        }
    }
}

/// Deterministic cold-POR experiment. `--relay-port MOCK` writes a labeled,
/// reproducible mock boot log so CI can exercise the JSON/log path without
/// hardware. Real relay ports are not implemented in Variant C.
fn cold_por(
    bit: &PathBuf,
    relay_port: &str,
    repeat: u32,
    wait_seconds: u32,
    pvt_context: Option<&PathBuf>,
    log_dir: Option<&PathBuf>,
) -> Result<()> {
    if !bit.is_file() {
        bail!("bitstream not found: {}", bit.display());
    }
    let pvt_ctx = load_optional_pvt_context(pvt_context)?;

    let root = repo_root()?;
    let boot_log_dir = match log_dir {
        Some(d) => d.to_path_buf(),
        None => root.join("build").join("fpga"),
    };
    std::fs::create_dir_all(&boot_log_dir)
        .with_context(|| format!("create {}", boot_log_dir.display()))?;
    let start_time = chrono::Local::now();

    if relay_port != "MOCK" {
        bail!(
            "real relay control on port '{}' is not implemented; use '--relay-port MOCK' for the deterministic CI mock",
            relay_port
        );
    }

    println!("== Cold-POR (relay MOCK) ==");
    println!("Bitstream: {}", bit.display());
    println!("Relay port: MOCK (deterministic, no hardware touched)");
    if wait_seconds > 0 {
        println!("Simulating operator delay: auto-continuing after {} seconds (press ENTER to continue early).", wait_seconds);
        wait_for_continue(wait_seconds, "cold-por")?;
    }

    // Deterministic mock outcome: a successful cold-POR with the canonical W400
    // STAT signature.
    let raw = 0x401079FCu32;
    let fake_done = true;
    let samples: Vec<serde_json::Value> = (0..repeat.max(1))
        .map(|i| {
            serde_json::json!({
                "index": i as usize,
                "raw": raw,
                "done": fake_done,
                "eos": fake_done,
                "init_complete": fake_done,
                "crc_error": false,
                "id_error": false,
                "mode": 0b001,
                "diagnosis": "FPGA configured (relay mock)",
            })
        })
        .collect();

    let conclusion = "DONE=HIGH: board boots from flash (relay mock)";
    let pvt_json = pvt_ctx.as_ref().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null));
    let log_entry = serde_json::json!({
        "timestamp": start_time.to_rfc3339(),
        "bitstream": bit.to_string_lossy().to_string(),
        "relay_port": relay_port,
        "relay_mock": true,
        "repeat": repeat.max(1),
        "conclusion": conclusion,
        "samples": samples,
        "pvt_context": pvt_json,
        "xadc": xadc_context_json("not_read", pvt_ctx.as_ref()),
        "pvt_envelope_margin_ns": Option::<i64>::None,
        "recommendation": recommendation_from_conclusion(conclusion, None, None),
    });
    let log_path = boot_log_dir.join(format!(
        "boot-log-cold-por-mock-{}.json",
        start_time.format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&log_path, serde_json::to_string_pretty(&log_entry)?)
        .with_context(|| format!("write {}", log_path.display()))?;
    println!("[cold-por] mock log written to {}", log_path.display());
    println!("[cold-por] conclusion: {}", conclusion);
    Ok(())
}

/// Detect whether a target FPGA is reachable on the given openFPGALoader
/// cable profile. Returns false when the chain is empty / cable missing.
fn cable_detected(cable: &str) -> bool {
    let Ok((_, Some(output))) = run_openfpgaloader(cable, &["--detect"], true) else {
        return false;
    };
    output.contains("idcode")&& output.contains("manufacturer")
}

/// Assert that the decoded STAT register shows a successful boot/config.
fn assert_stat_boot_success(bits: &StatBits, ctx: &str) -> Result<()> {
    if !bits.done {
        bail!("{}: DONE=LOW (raw=0x{:08X}, {})", ctx, bits.raw, bits.diagnose());
    }
    if bits.mode != 0b001 {
        bail!(
            "{}: MODE=0b{:03b} != 0b001 (Master SPI x1)",
            ctx,
            bits.mode
        );
    }
    if bits.crc_error {
        bail!("{}: CRC_ERROR=1", ctx);
    }
    if bits.id_error {
        bail!("{}: ID_ERROR=1", ctx);
    }
    if bits.dec_error {
        bail!("{}: DEC_ERROR=1", ctx);
    }
    Ok(())
}

fn smoke_gate(
    bit: Option<&PathBuf>,
    top: &str,
    require_cable: bool,
    flash_boot: bool,
    wait_seconds: u32,
    cable: &str,
    part: &str,
) -> Result<()> {
    let root = repo_root()?;
    let bit_path = bit.cloned().unwrap_or_else(|| {
        root.join("fpga")
            .join("verilog")
            .join("ternary_mac_demo_top_200t.bit")
    });

    println!("== FPGA smoke gate ==");

    // Cable-connected hardware check (optional, gated by --require-cable).
    // --flash-boot implies --require-cable and performs a cold-POR flash-boot
    // gate instead of a volatile SRAM load.
    if require_cable {
        println!("[smoke-gate] require-cable: detecting FPGA via {}...", cable);
        if !cable_detected(cable) {
            bail!(
                "no FPGA detected on cable {} (is the board powered and connected?)",
                cable
            );
        }
        println!("[smoke-gate] cable OK (FPGA detected)");

        if !bit_path.is_file() {
            bail!(
                "bitstream not found at {} (required for --require-cable)",
                bit_path.display()
            );
        }

        if flash_boot {
            println!(
                "[smoke-gate] flash-boot: verifying cold-POR boot via single-variant CCLK sweep"
            );
            // Reuse the cclk-sweep code path for the cold-POR flash-boot check.
            // Empirically this path produces DONE=HIGH on the Wukong 200T, while
            // the older direct program_flash + capture path returned H2_CCLK_TIMING
            // with identical operator actions. The sweep patches OSCFSEL=0, programs
            // flash, prompts for power-cycle, and captures STAT.
            let results = cclk_sweep(
                &bit_path,
                &vec![0u8],
                Some(&root.join("build").join("fpga").join("cclk_variants")),
                Some(&root.join("build").join("fpga")),
                false,
                false,
                cable,
                part,
                None,
                6_000_000,
                3,
                wait_seconds,
                None,
                None,
            )
            .with_context(|| "flash-boot CCLK sweep failed")?;
            if !results.iter().any(|r| r.done) {
                bail!(
                    "hardware smoke-gate failed: cold-POR flash boot did not reach DONE=HIGH (OSCFSEL=0)"
                );
            }
            println!(
                "[smoke-gate] flash-boot check OK (DONE=HIGH, mode=001, no errors)"
            );
        } else {
            println!(
                "[smoke-gate] loading SRAM: {}",
                bit_path.display()
            );
            load_sram(&bit_path, cable, part, false, false)?;

            println!("[smoke-gate] reading STAT after SRAM load...");
            let samples = capture_stat(cable, false, 1)?;
            let bits = samples.first().cloned().expect("at least one STAT sample");
            assert_stat_boot_success(&bits, "SRAM load")
                .with_context(|| "hardware smoke-gate failed after SRAM load")?;
            println!("[smoke-gate] hardware check OK (DONE=HIGH, mode=001, no errors)");
        }
    }

    // 1. bit-config audit if the bitstream exists.
    if bit_path.is_file() {
        println!("[smoke-gate] bit-config audit: {}", bit_path.display());
        let assert_args: [&str; 7] = [
            "--assert-idcode",
            "0x03636093",
            "--assert-spi-x1",
            "--assert-cclk-startup",
            "--assert-oscfsel",
            "0",
            "--assert-no-crc-writes",
        ];
        bit_config(&bit_path, &assert_args)?;
    } else {
        println!(
            "[smoke-gate] SKIP: bitstream not found at {} (run openXC7 flow first)",
            bit_path.display()
        );
    }

    // 2. Dry-run CCLK sweep + report path (no hardware required).
    if bit_path.is_file() {
        println!("[smoke-gate] dry-run CCLK sweep: {}", bit_path.display());
        let values = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let dry_log_dir = root.join("build").join("fpga").join("smoke-gate-dry-run");
        // Remove stale dry-run logs so the report counts only this run.
        if dry_log_dir.is_dir() {
            for entry in std::fs::read_dir(&dry_log_dir)
                .with_context(|| format!("read {}", dry_log_dir.display()))?
            {
                let entry = entry?;
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("boot-log-") && name_str.ends_with(".json") {
                    std::fs::remove_file(entry.path())
                        .with_context(|| format!("remove {}", entry.path().display()))?;
                }
            }
        }
        let _dry_results = cclk_sweep(
            &bit_path,
            &values,
            Some(&root.join("build").join("fpga").join("cclk_variants")),
            Some(&dry_log_dir),
            false,
            true,
            "digilent_hs2",
            "xc7a200tfgg676",
            None,
            6_000_000,
            3,
            0,
            None,
            None,
        )?;
        let dry_report = dry_log_dir.join("sweep-report-smoke-gate-dry-run.md");
        sweep_report(Some(&dry_log_dir), Some(&dry_report))?;
        let report_text = std::fs::read_to_string(&dry_report)
            .with_context(|| format!("read {}", dry_report.display()))?;
        let variant_count = report_text
            .lines()
            .filter(|l| l.starts_with("| ") && l.contains(".bit") && !l.contains("Bitstream"))
            .count();
        if variant_count != values.len() {
            bail!(
                "dry-run sweep report has {} variant rows, expected {}",
                variant_count,
                values.len()
            );
        }
        println!("[smoke-gate] dry-run sweep report OK ({} variants)", variant_count);
    }

    // 3. yosys synthesis smoke on the demo sources if available.
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

fn boot_protocol(checklist: bool) -> Result<()> {
    println!("== Cold-POR boot-from-flash protocol ==");
    println!();
    println!("Target: QMTech Wukong V1 / XC7A200T-FGG676-1");
    println!("Prerequisite: flash is already programmed with the desired .bit file.");
    println!();
    println!("1. Disconnect the JTAG/programming cable from the board.");
    println!("   (An attached cable can hold TMS/TCK/PROGRAM_B and corrupt POR");
    println!("    mode-pin sampling. See AR66954 / XAPP1188.)");
    println!();
    println!("2. Disconnect board power. Wait >= 10 s for all rails to collapse.");
    println!();
    println!("3. Reconnect board power. Wait >= 2 s for rails to stabilise.");
    println!();
    println!("4. Reconnect the JTAG/programming cable.");
    println!();
    println!("5. Capture STAT without a JTAG reset/PROGRAM_B pulse:");
    println!("   tri fpga stat --pre-jtag-reset --repeat 3");
    println!();
    println!("Success signature: STAT = 0x401079FC");
    println!("   DONE=1, INIT_B=1, EOS=1, MODE=0b001 (Master SPI x1),");
    println!("   BUS Width=x1, CRC_ERROR=0, ID_ERROR=0.");
    println!();
    println!("Failure signature: STAT = 0x5000190C");
    println!("   MODE=0b001 but DONE=0. Usually caused by an incomplete cold-POR");
    println!("   or the JTAG cable remaining attached during power-on.");
    println!("   Re-run from step 1; do not assume CCLK timing is wrong.");
    println!();
    println!("If all steps above are followed and DONE stays 0, then diagnose");
    println!("with tri fpga boot-log or tri fpga cclk-sweep.");

    if !checklist {
        println!();
        println!("Confirm each step when ready (y/n):");
        let steps = [
            "JTAG cable disconnected before power-off",
            "Board power disconnected and waited >= 10 s",
            "Board power reconnected and waited >= 2 s",
            "JTAG cable reconnected after rails stable",
        ];
        for step in &steps {
            loop {
                print!("  [ ] {} > ", step);
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)
                    .context("reading confirmation")?;
                match input.trim().to_lowercase().as_str() {
                    "y" | "yes" => break,
                    "n" | "no" => bail!("cold-POR protocol aborted by user"),
                    _ => println!("    please answer y or n"),
                }
            }
        }
        println!();
        println!("Protocol confirmed. Run: tri fpga stat --pre-jtag-reset --repeat 3");
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static PVT_CTX_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn square_wave_csv(period: f64, cycles: usize) -> String {
        let mut out = String::from("Time,Voltage\n");
        let dt = period / 20.0;
        for i in 0..(cycles * 20) {
            let t = i as f64 * dt;
            let v = if i % 20 < 10 { 0.0 } else { 3.3 };
            out.push_str(&format!("{:.12},{:.1}\n", t, v));
        }
        out
    }

    #[test]
    fn test_parse_cclk_csv_dsview_header() {
        let csv = square_wave_csv(1.0 / 3.0e6, 10); // 3 MHz, 50% duty
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 3.0e6).abs() < 50_000.0, "freq {} should be ~3 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    #[test]
    fn test_parse_cclk_csv_pulseview_header() {
        let mut csv = String::from("samplerate,100000000\nTime,Channel 0,Channel 1\n");
        let dt = (1.0 / 6.0e6) / 20.0;
        for i in 0..200 {
            let t = i as f64 * dt;
            let v0 = if i % 20 < 10 { 0.0 } else { 3.3 };
            csv.push_str(&format!("{:.12},{:.1},0.0\n", t, v0));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 6.0e6).abs() < 100_000.0, "freq {} should be ~6 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    #[test]
    fn test_parse_cclk_csv_saleae_header() {
        let mut csv = String::from("time, channel 0\n");
        let dt = (1.0 / 12.0e6) / 20.0;
        for i in 0..200 {
            let t = i as f64 * dt;
            let v = if i % 20 < 8 { 0.0 } else { 3.3 }; // 60% high duty
            csv.push_str(&format!("{:.12},{:.1}\n", t, v));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 12.0e6).abs() < 200_000.0, "freq {} should be ~12 MHz", freq);
        assert!((duty - 60.0).abs() < 5.0, "duty {} should be ~60%", duty);
    }

    /// Voltage column is explicitly named and is not the second column. The
    /// parser must use the named voltage column rather than the first numeric
    /// column after time.
    #[test]
    fn test_parse_cclk_csv_named_voltage_column() {
        let mut csv = String::from("time_s,counter,voltage\n");
        let dt = (1.0 / 8.0e6) / 20.0;
        for i in 0..200 {
            let t = i as f64 * dt;
            let counter = i as f64; // steadily increasing, not the signal
            let v = if i % 20 < 10 { 0.0 } else { 3.3 }; // 50% duty 8 MHz
            csv.push_str(&format!("{:.12},{},{}\n", t, counter, v));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 8.0e6).abs() < 150_000.0, "freq {} should be ~8 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    #[test]
    fn test_parse_cclk_csv_too_few_samples() {
        let csv = "Time,Voltage\n0.0,0.0\n1.0,3.3\n";
        assert!(parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).is_err());
    }

    #[test]
    fn test_parse_cclk_csv_explicit_channel_select() {
        // Multi-channel export: time, a flat 0.0 reference, and the real CCLK signal.
        // Without --csv-channel the flat column would be chosen as the second
        // numeric column and produce no transitions; with `cclk_v` we get ~1 MHz.
        let mut csv = String::from("time,ref,cclk_v\n");
        for i in 0..200 {
            let t = i as f64 * 1.0e-7; // 0.1 µs per sample
            let v = if i % 10 < 5 { 0.0 } else { 3.3 }; // 1 µs period
            csv.push_str(&format!("{:.6e},0.0,{:.1}\n", t, v));
        }
        let (freq, duty) =
            parse_cclk_csv_reader(std::io::Cursor::new(csv), Some("cclk_v"), None, None).unwrap();
        assert!((freq - 1.0e6).abs() < 100_000.0, "freq {} should be ~1 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    /// CSV with a milliseconds time column (`time_ms`). The parser must scale the
    /// raw values by 1e-3 before computing frequency/duty.
    #[test]
    fn test_parse_cclk_csv_ms_header() {
        let mut csv = String::from("time_ms,voltage\n");
        let dt = 0.5; // 0.5 ms per sample
        for i in 0..200 {
            let t = i as f64 * dt;
            let v = if i % 20 < 10 { 0.0 } else { 3.3 }; // 100 Hz, 50% duty
            csv.push_str(&format!("{},{:.1}\n", t, v));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 100.0).abs() < 10.0, "freq {} should be ~100 Hz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    /// CSV with a microseconds time column (`time_us`).
    #[test]
    fn test_parse_cclk_csv_us_header() {
        let mut csv = String::from("time_us,voltage\n");
        let dt = 5.0; // 5 µs per sample
        for i in 0..200 {
            let t = i as f64 * dt;
            let v = if i % 20 < 10 { 0.0 } else { 3.3 }; // 10 kHz, 50% duty
            csv.push_str(&format!("{},{:.1}\n", t, v));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 10_000.0).abs() < 1_000.0, "freq {} should be ~10 kHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    /// CSV with a nanoseconds time column (`time_ns`).
    #[test]
    fn test_parse_cclk_csv_ns_header() {
        let mut csv = String::from("time_ns,voltage\n");
        let dt = 50.0; // 50 ns per sample
        for i in 0..200 {
            let t = i as f64 * dt;
            let v = if i % 20 < 10 { 0.0 } else { 3.3 }; // 1 MHz, 50% duty
            csv.push_str(&format!("{},{:.1}\n", t, v));
        }
        let (freq, duty) = parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).unwrap();
        assert!((freq - 1.0e6).abs() < 100_000.0, "freq {} should be ~1 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    /// CSV whose time column is sample numbers (header `Sample`) must be scaled
    /// by the supplied samplerate.
    #[test]
    fn test_parse_cclk_csv_sample_numbers() {
        let mut csv = String::from("Sample,cclk_v\n");
        for i in 0..200 {
            let v = if i % 10 < 5 { 0.0 } else { 3.3 }; // 1 MHz at 10 Msps
            csv.push_str(&format!("{},{:.1}\n", i, v));
        }
        let (freq, duty) =
            parse_cclk_csv_reader(std::io::Cursor::new(csv), None, Some(10_000_000), None).unwrap();
        assert!((freq - 1.0e6).abs() < 100_000.0, "freq {} should be ~1 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
    }

    /// CSV with sample-number time column and no samplerate must error clearly.
    #[test]
    fn test_parse_cclk_csv_sample_numbers_require_samplerate() {
        let csv = "Sample,cclk_v\n0,0.0\n1,3.3\n2,0.0\n3,3.3\n";
        assert!(parse_cclk_csv_reader(std::io::Cursor::new(csv), None, None, None).is_err());
    }

    #[test]
    fn test_is_logic_csv_detects_sigrok() {
        let csv = "; Samplerate: 10 MHz\nlogic\n0\n1\n0\n";
        assert!(is_logic_csv(&std::env::temp_dir().join("tri_test_logic.csv")).is_err());
        // is_logic_csv requires a real file; write one.
        let tmp = std::env::temp_dir().join(format!("tri_test_logic_{}.csv", std::process::id()));
        std::fs::write(&tmp, csv).unwrap();
        assert!(is_logic_csv(&tmp).unwrap());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_is_logic_csv_rejects_analog() {
        let tmp = std::env::temp_dir().join(format!("tri_test_analog_{}.csv", std::process::id()));
        std::fs::write(&tmp, "Time,Voltage\n0.0,0.0\n1.0,3.3\n").unwrap();
        assert!(!is_logic_csv(&tmp).unwrap());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_parse_logic_csv_2_5mhz() {
        let samplerate = 100_000_000_u32;
        let tmp = std::env::temp_dir().join(format!("tri_test_logic_25mhz_{}.csv", std::process::id()));
        generate_synth_cclk_csv(2_500_000.0, samplerate, 1000, &tmp).unwrap();
        let (freq, duty) = parse_logic_csv(&tmp, samplerate).unwrap();
        assert!((freq - 2.5e6).abs() < 200_000.0, "freq {} should be ~2.5 MHz", freq);
        assert!((duty - 50.0).abs() < 5.0, "duty {} should be ~50%", duty);
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_generate_synth_cclk_csv_header() {
        let tmp = std::env::temp_dir().join(format!("tri_test_synth_header_{}.csv", std::process::id()));
        generate_synth_cclk_csv(1_000_000.0, 10_000_000, 20, &tmp).unwrap();
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.starts_with("logic\n"));
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 21); // header + 20 samples
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_measured_cclk_conservative_2_5mhz_50duty() {
        let m = MeasuredCclk::new(2_500_000.0, 50.0, "synthetic".to_string());
        assert_eq!(m.freq_hz, 2_500_000);
        assert_eq!(m.sck_low_ns, 200);
        assert_eq!(m.sck_high_ns, 200);
        assert_eq!(m.sck_low_ns + m.sck_high_ns, 400);
    }

    #[test]
    fn test_measured_cclk_25mhz_50duty() {
        let m = MeasuredCclk::new(25_000_000.0, 50.0, "synthetic".to_string());
        assert_eq!(m.freq_hz, 25_000_000);
        assert_eq!(m.sck_low_ns, 20);
        assert_eq!(m.sck_high_ns, 20);
        assert!(m.sck_low_ns >= 6);
        assert!(m.sck_high_ns >= 6);
    }

    #[test]
    fn test_measured_cclk_json_roundtrip() {
        let m = MeasuredCclk::new(33_300_000.0, 48.5, "live".to_string());
        let json = serde_json::to_string(&m).unwrap();
        let back: MeasuredCclk = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn test_sanitize_lean_ident() {
        assert_eq!(sanitize_lean_ident("synthetic (10000000 Hz samplerate)"), "synthetic_10000000_Hz_samplerate");
        assert_eq!(sanitize_lean_ident("live (ftdi-la, ADBUS4)"), "live_ftdi_la_ADBUS4");
        assert_eq!(sanitize_lean_ident("---__---abc---"), "abc");
        assert_eq!(sanitize_lean_ident(""), "");
    }

    #[test]
    fn test_measured_to_lean_output_nominal() {
        let m = MeasuredCclk::new(2_500_000.0, 50.0, "synthetic (10000000 Hz samplerate)".to_string());
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, None, false, false, false, false).unwrap();
        assert_eq!(out, ());
        // Clean up.
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_measured_to_lean_output_margin() {
        let m = MeasuredCclk::new(25_000_000.0, 50.0, "live".to_string());
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_margin_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", true, None, false, false, false, false).unwrap();
        assert_eq!(out, ());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_measured_to_lean_output_standalone() {
        let m = MeasuredCclk::new(2_500_000.0, 50.0, "synthetic".to_string());
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_standalone_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out_path = std::env::temp_dir().join(format!("tri_measured_to_lean_standalone_out_{}.lean", std::process::id()));
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, Some(&out_path), "measured_cclk", false, None, false, true, false, false).unwrap();
        assert_eq!(out, ());
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("import Trinity.TernaryFPGABoot"));
        assert!(content.contains("namespace Trinity.BitstreamConfig"));
        assert!(content.contains("end Trinity.BitstreamConfig"));
        std::fs::remove_file(&tmp).unwrap();
        std::fs::remove_file(&out_path).unwrap();
    }

    #[test]
    fn test_measured_to_lean_output_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 40,
            sck_low_ns: 20,
            sck_high_ns: 20,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_raw_ns_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, None, false, false, true, false).unwrap();
        assert_eq!(out, ());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_measured_to_lean_csv_raw_ns() {
        let samplerate = 100_000_000_u32;
        let csv_tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_csv_raw_ns_{}.csv", std::process::id()));
        generate_synth_cclk_csv(2_500_000.0, samplerate, 1000, &csv_tmp).unwrap();
        let out_path = std::env::temp_dir().join(format!("tri_measured_to_lean_csv_raw_ns_out_{}.lean", std::process::id()));
        let out = measured_to_lean(None, Some(&csv_tmp), None, None, None, None, None, 0, None, None, None, Some(&out_path), "measured_csv", false, None, false, true, true, false).unwrap();
        assert_eq!(out, ());
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("import Trinity.TernaryFPGABoot"));
        assert!(content.contains("namespace Trinity.BitstreamConfig"));
        assert!(content.contains("measured_cclk_from_raw_ns_satisfies_flash_spec"));
        assert!(content.contains("decide"));
        std::fs::remove_file(&csv_tmp).unwrap();
        std::fs::remove_file(&out_path).unwrap();
    }

    /// Generate a minimal scalar VCD with a single 25 MHz clock net.
    fn generate_vcd_clock(freq_hz: f64, cycles: usize) -> String {
        let period_s = 1.0 / freq_hz;
        let half_s = period_s / 2.0;
        let timescale_ps = 100; // 100 ps = 0.1 ns
        let mut out = String::new();
        out.push_str("$date today $end\n");
        out.push_str("$version tri test $end\n");
        out.push_str(&format!("$timescale {} ps $end\n", timescale_ps));
        out.push_str("$scope module top $end\n");
        out.push_str("$var wire 1 ! cclk $end\n");
        out.push_str("$upscope $end\n");
        out.push_str("$enddefinitions $end\n");
        out.push_str("$dumpvars\n");
        out.push_str("0!\n");
        out.push_str("$end\n");
        let mut t = 0.0;
        for i in 0..(2 * cycles) {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            let val = if i % 2 == 0 { '1' } else { '0' };
            out.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        out
    }

    #[test]
    fn test_parse_vcd_to_raw_ns_25mhz() {
        let vcd = generate_vcd_clock(25_000_000.0, 20);
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_raw_ns_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    #[test]
    fn test_measured_to_lean_vcd_raw_ns() {
        let vcd_tmp = std::env::temp_dir().join(format!("tri_measured_to_lean_vcd_raw_ns_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, generate_vcd_clock(25_000_000.0, 20)).unwrap();
        let out_path = std::env::temp_dir().join(format!("tri_measured_to_lean_vcd_raw_ns_out_{}.lean", std::process::id()));
        let out = measured_to_lean(None, None, None, None, None, Some(&vcd_tmp), None, 0, None, None, None, Some(&out_path), "measured_vcd", false, None, false, true, true, false).unwrap();
        assert_eq!(out, ());
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("import Trinity.TernaryFPGABoot"));
        assert!(content.contains("namespace Trinity.BitstreamConfig"));
        assert!(content.contains("measured_cclk_from_raw_ns_satisfies_flash_spec"));
        assert!(content.contains("decide"));
        std::fs::remove_file(&vcd_tmp).unwrap();
        std::fs::remove_file(&out_path).unwrap();
    }

    /// Generate a minimal VCD with a multi-bit logic bus where bit 0 toggles.
    fn generate_vcd_bus(freq_hz: f64, cycles: usize) -> String {
        let period_s = 1.0 / freq_hz;
        let half_s = period_s / 2.0;
        let timescale_ps = 100;
        let mut out = String::new();
        out.push_str("$date today $end\n");
        out.push_str("$version tri test $end\n");
        out.push_str(&format!("$timescale {} ps $end\n", timescale_ps));
        out.push_str("$scope module top $end\n");
        out.push_str("$var wire 4 ! cclk_bus $end\n");
        out.push_str("$upscope $end\n");
        out.push_str("$enddefinitions $end\n");
        out.push_str("$dumpvars\n");
        out.push_str("b0000 !\n");
        out.push_str("$end\n");
        let mut t = 0.0;
        for i in 0..(2 * cycles) {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            // bit 0 toggles; upper bits stay 0.
            let val = if i % 2 == 0 { "b0001" } else { "b0000" };
            out.push_str(&format!("#{}\n{} !\n", ts, val));
        }
        out
    }

    #[test]
    fn test_parse_vcd_bus_to_raw_ns_25mhz() {
        let vcd = generate_vcd_bus(25_000_000.0, 20);
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_bus_raw_ns_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk_bus"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// Generate a minimal VCD with a real-valued net crossing a threshold.
    fn generate_vcd_real(freq_hz: f64, cycles: usize) -> String {
        let period_s = 1.0 / freq_hz;
        let half_s = period_s / 2.0;
        let timescale_ps = 100;
        let mut out = String::new();
        out.push_str("$date today $end\n");
        out.push_str("$version tri test $end\n");
        out.push_str(&format!("$timescale {} ps $end\n", timescale_ps));
        out.push_str("$scope module top $end\n");
        out.push_str("$var real 32 ! cclk_analog $end\n");
        out.push_str("$upscope $end\n");
        out.push_str("$enddefinitions $end\n");
        out.push_str("$dumpvars\n");
        out.push_str("r0.0 !\n");
        out.push_str("$end\n");
        let mut t = 0.0;
        for i in 0..(2 * cycles) {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            let v = if i % 2 == 0 { 3.3 } else { 0.0 };
            out.push_str(&format!("#{}{}\nr{} !\n", ts, if i == cycles { "\n$dumpoff\n" } else { "" }, v));
        }
        out.push_str("$dumpon\n");
        out
    }

    #[test]
    fn test_parse_vcd_real_to_raw_ns_25mhz() {
        let vcd = generate_vcd_real(25_000_000.0, 20);
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_real_raw_ns_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk_analog"), 0, Some(&1.65), None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert!(low_ns.abs_diff(20) <= 1, "low {} should be within 1 ns of 20 ns", low_ns);
        assert!(high_ns.abs_diff(20) <= 1, "high {} should be within 1 ns of 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// Real-valued VCD without an explicit threshold: auto-threshold must pick
    /// the midpoint of the observed 0.0 V .. 3.3 V swing and recover the clock.
    #[test]
    fn test_parse_vcd_real_auto_threshold() {
        let vcd = generate_vcd_real(25_000_000.0, 20);
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_real_auto_threshold_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk_analog"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert!(low_ns.abs_diff(20) <= 1, "low {} should be within 1 ns of 20 ns", low_ns);
        assert!(high_ns.abs_diff(20) <= 1, "high {} should be within 1 ns of 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// A `$comment` block containing the literal substring `$end` before the
    /// real `$end` terminator must not corrupt the signal dictionary. The old
    /// heuristic `ends_with("$end")` terminated early and swallowed the following
    /// `$var` declaration, causing "VCD has no scalar or selectable logic net".
    #[test]
    fn test_parse_vcd_comment_with_embedded_end_token() {
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$comment\n");
        vcd.push_str("This comment mentions the $end token but is not finished yet.\n");
        vcd.push_str("Another line with $end embedded in the text.\n");
        vcd.push_str("$end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        vcd.push_str(&generate_vcd_clock(25_000_000.0, 20));
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_comment_embedded_end_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// A multi-line `$timescale` block containing the literal substring `$end`
    /// before the real `$end` terminator must not corrupt the timescale. With the
    /// old substring heuristic the embedded `$end` closed the section early and
    /// the following `$var` line was swallowed, producing a parse error.
    #[test]
    fn test_parse_vcd_timescale_with_embedded_end_token() {
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale\n");
        vcd.push_str("  1 us\n");
        vcd.push_str("  // note: $end appears in this comment line\n");
        vcd.push_str("$end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        // 25 kHz clock with 1 us timescale: period = 40 us, half = 20 us.
        // Timestamps are clean integer microseconds.
        let half_us = 20u64;
        for i in 0..40 {
            let ts = half_us * (i + 1);
            let val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_timescale_embedded_end_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40_000, "period {} should be 40_000 ns (1 us timescale)", period_ns);
        // Low/high may differ by 1 ns due to floating-point timescale conversion; accept a small tolerance.
        assert!(
            low_ns.abs_diff(20_000) <= 1,
            "low {} should be within 1 ns of 20_000 ns", low_ns
        );
        assert!(
            high_ns.abs_diff(20_000) <= 1,
            "high {} should be within 1 ns of 20_000 ns", high_ns
        );
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// Real-valued VCD with a non-default `$timescale 1 us $end` and no explicit
    /// threshold. Auto-threshold must still recover the 25 kHz clock from the
    /// observed 0.0 V .. 3.3 V swing.
    #[test]
    fn test_parse_vcd_real_auto_threshold_us_timescale() {
        let timescale_us = 1u64;
        let freq_hz = 25_000.0;
        let period_s = 1.0 / freq_hz;
        let half_s = period_s / 2.0;
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale 1 us $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var real 32 ! cclk_analog $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("r0.0 !\n");
        vcd.push_str("$end\n");
        let mut t = 0.0;
        // Emit 41 samples so the final value is high, producing an odd number of
        // transitions and equal high/low windows despite the 1 µs timescale.
        for i in 0..41 {
            t += half_s;
            let ts = (t / (timescale_us as f64 * 1.0e-6)).round() as u64;
            let v = if i % 2 == 0 { 3.3 } else { 0.0 };
            vcd.push_str(&format!("#{}\nr{} !\n", ts, v));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_real_auto_threshold_us_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("cclk_analog"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40_000, "period {} should be 40_000 ns (1 us timescale)", period_ns);
        // Low/high may differ by 1 ns due to floating-point timescale conversion; accept a small tolerance.
        assert!(
            low_ns.abs_diff(20_000) <= 1,
            "low {} should be within 1 ns of 20_000 ns", low_ns
        );
        assert!(
            high_ns.abs_diff(20_000) <= 1,
            "high {} should be within 1 ns of 20_000 ns", high_ns
        );
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with an unsupported `$timescale` unit. The parser must warn and fall
    /// back to the default 1 ns timescale, then recover the 25 MHz clock.
    #[test]
    fn test_parse_vcd_unknown_timescale_defaults_to_1ns() {
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale 1 xy $end\n"); // unsupported unit
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        // With the default 1 ns timescale, timestamps are in nanoseconds.
        // 25 MHz => 40 ns period, half = 20 ns.
        for i in 0..41 {
            let ts = (i + 1) * 20;
            let val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        let vcd_tmp = std::env::temp_dir()
            .join(format!("tri_test_vcd_unknown_timescale_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) =
            parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert!(low_ns.abs_diff(20) <= 1, "low {} should be within 1 ns of 20 ns", low_ns);
        assert!(high_ns.abs_diff(20) <= 1, "high {} should be within 1 ns of 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// Real-valued VCD with a 5 ns glitch placed 5 ns after a real edge. The
    /// inter-transition slope filter (`--vcd-slope-min-s`) must drop the glitch
    /// and recover the underlying 10 MHz clock.
    #[test]
    fn test_parse_vcd_real_slope_filter_rejects_glitch() {
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale 1 ns $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var real 32 ! cclk_analog $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("r0.0 !\n");
        vcd.push_str("$end\n");
        // 10 MHz square wave (100 ns period). A 5 ns glitch (55 ns .. 60 ns)
        // while the signal is high must be ignored.
        let samples = [
            (0.0, 0.0),
            (50.0, 3.3),
            (55.0, 0.0), // glitch start
            (60.0, 3.3), // glitch end
            (100.0, 0.0),
            (150.0, 3.3),
            (200.0, 0.0),
            (250.0, 3.3),
            (300.0, 0.0),
            (350.0, 3.3),
        ];
        for (t, v) in samples {
            vcd.push_str(&format!("#{}\nr{} !\n", t as u64, v));
        }
        let vcd_tmp = std::env::temp_dir()
            .join(format!("tri_test_vcd_real_slope_filter_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp,
            Some("cclk_analog"),
            0,
            Some(&1.65),
            None,
            Some(&15.0e-9), // reject transitions closer than 15 ns
        )
        .unwrap();
        assert_eq!(period_ns, 100, "period {} should be 100 ns", period_ns);
        assert!(
            low_ns.abs_diff(50) <= 1,
            "low {} should be within 1 ns of 50 ns",
            low_ns
        );
        assert!(
            high_ns.abs_diff(50) <= 1,
            "high {} should be within 1 ns of 50 ns",
            high_ns
        );
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with `$dumpoff` / `$dumpon` placed on lines that do not carry a
    /// timestamp. The parser must continue using the last known timestamp and
    /// ignore value changes that occur while dumping is suspended.
    #[test]
    fn test_parse_vcd_dumpoff_dumpon_without_timestamp() {
        let mut vcd = String::new();
        vcd.push_str("$timescale 1 ns $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        // 25 MHz clock: timestamps in nanoseconds.
        vcd.push_str("#20\n1!\n");
        vcd.push_str("#40\n0!\n"); // real falling edge at 40 ns
        vcd.push_str("#40\n$dumpoff\n");
        vcd.push_str("1!\n");      // ignored: dumpoff active and opposite to current low
        vcd.push_str("$dumpon\n");
        vcd.push_str("#60\n1!\n"); // normal rising edge at 60 ns
        for i in 3..42 {
            let ts = (i + 1) * 20;
            let val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        let vcd_tmp = std::env::temp_dir()
            .join(format!("tri_test_vcd_dumpoff_no_ts_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) =
            parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert!(low_ns.abs_diff(20) <= 1, "low {} should be within 1 ns of 20 ns", low_ns);
        assert!(high_ns.abs_diff(20) <= 1, "high {} should be within 1 ns of 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with a multi-line $var declaration (size and identifier on one line,
    /// name on the next). The parser must accumulate tokens until `$end`.
    #[test]
    fn test_parse_vcd_multiline_var_declaration() {
        let mut vcd = String::new();
        vcd.push_str("$date today $end\n");
        vcd.push_str("$version tri test $end\n");
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 !\n");
        vcd.push_str("cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        vcd.push_str(&generate_vcd_clock(25_000_000.0, 20));
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_multiline_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD containing both a scalar clock and a multi-bit bus; selecting the
    /// scalar by name must ignore the bus transitions.
    #[test]
    fn test_parse_vcd_mixed_scalar_and_bus() {
        let mut vcd = String::new();
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$var wire 8 @ data $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("b00000000 @\n");
        vcd.push_str("$end\n");
        // Append a clean 25 MHz scalar clock plus per-step bus noise.
        let timescale_ps = 100;
        let period_s = 1.0 / 25_000_000.0;
        let half_s = period_s / 2.0;
        let mut t = 0.0;
        for i in 0..40 {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            let cclk_val = if i % 2 == 0 { '1' } else { '0' };
            let bus = format!("b{:08b} @\n", i as u8);
            vcd.push_str(&format!("#{}\n{}!\n{}", ts, cclk_val, bus));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_mixed_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(&vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with a $dumpoff/$dumpon region containing spurious fast toggles.
    /// The parser must ignore the dumpoff region entirely so the measured
    /// period matches the clean 25 MHz clock present before it.
    #[test]
    fn test_parse_vcd_dumpoff_ignores_spurious_edges() {
        let mut vcd = generate_vcd_clock(25_000_000.0, 10);
        // $dumpoff in the middle of the capture, then inject spurious edges.
        vcd.push_str("$dumpoff\n");
        for i in 0..100 {
            let ts = 500 + i; // arbitrary fast toggles at ~1 GHz relative scale
            let val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        vcd.push_str("$dumpon\n");
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_dumpoff_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// Multi-line $date / $version / $comment header sections must be skipped so
    /// their free-form contents are not mistaken for $var declarations. This
    /// is a regression test for vendor VCDs that split the header across lines.
    #[test]
    fn test_parse_vcd_multiline_header_sections_skipped() {
        let timescale_ps = 100;
        let mut vcd = String::new();
        vcd.push_str("$date\n");
        vcd.push_str("  Thu Jan 01 00:00:00 2026\n");
        vcd.push_str("$end\n");
        vcd.push_str("$version\n");
        vcd.push_str("  SomeVendor Simulator 1.2.3\n");
        vcd.push_str("$end\n");
        vcd.push_str("$comment\n");
        vcd.push_str("  This is a multi-line comment that could contain words like wire or reg.\n");
        vcd.push_str("$end\n");
        vcd.push_str(&format!("$timescale {} ps $end\n", timescale_ps));
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        let period_s = 1.0 / 25_000_000.0;
        let half_s = period_s / 2.0;
        let mut t = 0.0;
        for i in 0..40 {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            let val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, val));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_multiline_header_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with an escaped identifier that contains a space in the name.
    /// The parser must join name tokens and strip the leading backslash.
    #[test]
    fn test_parse_vcd_escaped_identifier_with_space() {
        let mut vcd = String::new();
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$scope module top $end\n");
        // Escaped identifier: id=!, name="\my sig"
        vcd.push_str("$var wire 1 ! \\my sig $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        vcd.push_str(&generate_vcd_clock(25_000_000.0, 20));
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_escaped_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("my sig"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD scalar net with x/z transitions inserted between real edges.
    /// The parser must ignore x/z and still measure a clean 25 MHz clock.
    #[test]
    fn test_parse_vcd_scalar_xz_ignored() {
        let mut vcd = String::new();
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 1 ! cclk $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("0!\n");
        vcd.push_str("$end\n");
        let timescale_ps = 100;
        let period_s = 1.0 / 25_000_000.0;
        let half_s = period_s / 2.0;
        let mut t = 0.0;
        for i in 0..40 {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            let cclk_val = if i % 2 == 0 { '1' } else { '0' };
            vcd.push_str(&format!("#{}\n{}!\n", ts, cclk_val));
            // Insert an indeterminate transition right after each edge.
            let xz_ts = ts + 1;
            let xz_val = if i % 4 == 0 { 'x' } else { 'z' };
            vcd.push_str(&format!("#{}\n{}!\n", xz_ts, xz_val));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_xz_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("cclk"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    /// VCD with a 4-bit hex bus. Bit 0 toggles at 25 MHz.
    #[test]
    fn test_parse_vcd_hex_bus_to_raw_ns_25mhz() {
        let mut vcd = String::new();
        vcd.push_str("$timescale 100 ps $end\n");
        vcd.push_str("$scope module top $end\n");
        vcd.push_str("$var wire 4 ! data $end\n");
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        vcd.push_str("h0 !\n");
        vcd.push_str("$end\n");
        let timescale_ps = 100;
        let period_s = 1.0 / 25_000_000.0;
        let half_s = period_s / 2.0;
        let mut t = 0.0;
        for i in 0..40 {
            t += half_s;
            let ts = (t / (timescale_ps as f64 * 1.0e-12)).round() as u64;
            // bit 0 toggles; upper bits stay 0 => h0 / h1 alternating.
            let val = if i % 2 == 0 { "h1" } else { "h0" };
            vcd.push_str(&format!("#{}\n{} !\n", ts, val));
        }
        let vcd_tmp = std::env::temp_dir().join(format!("tri_test_vcd_hex_{}.vcd", std::process::id()));
        std::fs::write(&vcd_tmp, vcd).unwrap();
        let (period_ns, low_ns, high_ns) = parse_vcd_to_raw_ns(
            &vcd_tmp, Some("data"), 0, None, None, None).unwrap();
        assert_eq!(period_ns, 40, "period {} should be 40 ns", period_ns);
        assert_eq!(low_ns, 20, "low {} should be 20 ns", low_ns);
        assert_eq!(high_ns, 20, "high {} should be 20 ns", high_ns);
        std::fs::remove_file(&vcd_tmp).unwrap();
    }

    #[test]
    fn test_validate_accepts_in_spec_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 40,
            sck_low_ns: 20,
            sck_high_ns: 20,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_in_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, None, false, false, true, true).unwrap();
        assert_eq!(out, ());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_validate_rejects_out_of_spec_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 5,
            sck_low_ns: 2,
            sck_high_ns: 3,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_out_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, None, false, false, true, true);
        assert!(out.is_err(), "expected validation to reject out-of-spec raw-ns capture");
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_validate_margin_accepts_in_spec_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 30,
            sck_low_ns: 15,
            sck_high_ns: 15,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_margin_in_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", true, None, false, false, true, true).unwrap();
        assert_eq!(out, ());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_validate_margin_rejects_out_of_spec_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 20,
            sck_low_ns: 8,
            sck_high_ns: 12,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_margin_out_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", true, None, false, false, true, true);
        assert!(out.is_err(), "expected PVT-margin validation to reject 8 ns low time");
        std::fs::remove_file(&tmp).unwrap();
    }

    /// Helper: write a PVT context JSON file and return a unique path.
    fn write_pvt_context_json(name: &str, ctx: &serde_json::Value) -> PathBuf {
        let n = PVT_CTX_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "tri_pvt_ctx_{}_{}_{}.json",
            name,
            std::process::id(),
            n
        ));
        std::fs::write(&path, serde_json::to_string(ctx).unwrap()).unwrap();
        path
    }

    #[test]
    fn test_validate_pvt_worstcase_accepts_in_spec_raw_ns() {
        let m = MeasuredCclkRawNs {
            period_ns: 40,
            sck_low_ns: 20,
            sck_high_ns: 20,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_pvt_in_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let pvt = write_pvt_context_json(
            "worstcase",
            &serde_json::json!({"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}),
        );
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, Some(&pvt), false, false, true, true).unwrap();
        assert_eq!(out, ());
        std::fs::remove_file(&tmp).unwrap();
        std::fs::remove_file(&pvt).unwrap();
    }

    #[test]
    fn test_validate_pvt_worstcase_rejects_out_of_spec_raw_ns() {
        // 20 ns period / 8 ns low fails the 13 ns worst-case half-period bound.
        let m = MeasuredCclkRawNs {
            period_ns: 20,
            sck_low_ns: 8,
            sck_high_ns: 12,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_validate_pvt_out_spec_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let pvt = write_pvt_context_json(
            "worstcase",
            &serde_json::json!({"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}),
        );
        let out = measured_to_lean(Some(&tmp), None, None, None, None, None, None, 0, None, None, None, None, "measured_cclk", false, Some(&pvt), false, false, true, true);
        assert!(out.is_err(), "expected PVT worst-case validation to reject 8 ns low time");
        std::fs::remove_file(&tmp).unwrap();
        std::fs::remove_file(&pvt).unwrap();
    }

    #[test]
    fn test_measured_to_lean_raw_ns_pvt_emits_pvt_theorem() {
        let m = MeasuredCclkRawNs {
            period_ns: 40,
            sck_low_ns: 20,
            sck_high_ns: 20,
            source: "live".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!("tri_pvt_lean_in_{}.json", std::process::id()));
        std::fs::write(&tmp, json).unwrap();
        let pvt = write_pvt_context_json(
            "worstcase",
            &serde_json::json!({"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}),
        );
        let out_path = std::env::temp_dir().join(format!("tri_pvt_lean_out_{}.lean", std::process::id()));
        let out = measured_to_lean(
            Some(&tmp), None, None, None, None, None, None, 0, None, None, None, Some(&out_path), "measured_cclk", false, Some(&pvt), false, true, true, true,
        ).unwrap();
        assert_eq!(out, ());
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec"));
        assert!(content.contains("process_corner := ProcessCorner.ss"));
        assert!(content.contains("decide"));
        std::fs::remove_file(&tmp).unwrap();
        std::fs::remove_file(&pvt).unwrap();
        std::fs::remove_file(&out_path).unwrap();
    }

    #[test]
    fn test_pvt_envelope_worstcase_context() {
        let pvt = write_pvt_context_json(
            "worstcase",
            &serde_json::json!({"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}),
        );
        let out = pvt_envelope(Some(&pvt));
        assert!(out.is_ok(), "pvt_envelope should accept a valid worst-case context");
        // Worst-case bound is 6 + 2.5 (temp) + 1.0 (voltage) + 4 (ss) = 13 ns
        // (integer arithmetic: temp derating = (125*2)/100 = 2; voltage = (200*5)/1000 = 1).
        assert_eq!(n25q128_min_sck_half_ns_pvt(
            &parse_pvt_context(&pvt).unwrap()), 13,
            "worst-case half-period bound should be 13 ns");
        std::fs::remove_file(&pvt).unwrap();
    }

    #[test]
    fn test_pvt_envelope_no_context_prints_examples() {
        // Without a context file the command prints the operating envelope and
        // best/typical/worst example bounds. It should not error.
        let out = pvt_envelope(None);
        assert!(out.is_ok());
    }

    /// The PVT-aware half-period bound is monotone non-decreasing in temperature
    /// inside the operating envelope: raising temperature (or keeping it equal)
    /// never shrinks the bound.
    #[test]
    fn test_pvt_half_ns_monotone_in_temp() {
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        let corners = [ProcessCorner::Ff, ProcessCorner::Tt, ProcessCorner::Ss];
        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        for vccint in vccints {
            for corner in &corners {
                let mut prev = 0u64;
                for (i, temp) in temps.iter().enumerate() {
                    let ctx = PvtContext {
                        temp_c: *temp,
                        vccint_mv: vccint,
                        vccaux_mv: 2700,
                        process_corner: corner.clone(),
                    };
                    let half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
                    assert!(
                        half_ns >= prev,
                        "PVT half-period bound not monotone at step {}: {} ns < previous {} ns (temp={} -> {}, vccint={}, corner={:?})",
                        i, half_ns, prev, if i > 0 { temps[i - 1] } else { *temp }, *temp, vccint, corner
                    );
                    prev = half_ns;
                }
            }
        }
    }

    /// The PVT-aware half-period bound is antitone non-increasing in VCCINT
    /// inside the operating envelope: raising VCCINT (closer to the maximum)
    /// never increases the bound.
    #[test]
    fn test_pvt_half_ns_antitone_in_vccint() {
        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        let corners = [ProcessCorner::Ff, ProcessCorner::Tt, ProcessCorner::Ss];
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        for temp in temps {
            for corner in &corners {
                let mut prev = u64::MAX;
                for (i, vccint) in vccints.iter().enumerate() {
                    let ctx = PvtContext {
                        temp_c: temp,
                        vccint_mv: *vccint,
                        vccaux_mv: 2700,
                        process_corner: corner.clone(),
                    };
                    let half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
                    assert!(
                        half_ns <= prev,
                        "PVT half-period bound not antitone at step {}: {} ns > previous {} ns (vccint={} -> {}, temp={}, corner={:?})",
                        i, half_ns, prev, if i > 0 { vccints[i - 1] } else { *vccint }, *vccint, temp, corner
                    );
                    prev = half_ns;
                }
            }
        }
    }

    /// The PVT-aware half-period bound is monotone with the process-corner
    /// ordering ff ≤ tt ≤ ss: moving to a worse corner never shrinks the bound.
    #[test]
    fn test_pvt_half_ns_monotone_in_process_corner() {
        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        let corner_pairs = [
            (ProcessCorner::Ff, ProcessCorner::Tt),
            (ProcessCorner::Tt, ProcessCorner::Ss),
            (ProcessCorner::Ff, ProcessCorner::Ss),
        ];
        for temp in temps {
            for vccint in vccints {
                for (c1, c2) in &corner_pairs {
                    let ctx1 = PvtContext {
                        temp_c: temp,
                        vccint_mv: vccint,
                        vccaux_mv: 2700,
                        process_corner: c1.clone(),
                    };
                    let ctx2 = PvtContext {
                        temp_c: temp,
                        vccint_mv: vccint,
                        vccaux_mv: 2700,
                        process_corner: c2.clone(),
                    };
                    let half1 = n25q128_min_sck_half_ns_pvt(&ctx1);
                    let half2 = n25q128_min_sck_half_ns_pvt(&ctx2);
                    assert!(
                        half1 <= half2,
                        "PVT half-period bound not monotone in process corner: {} ns (ff) > {} ns ({:?}) at temp={}, vccint={}",
                        half1, half2, c2, temp, vccint
                    );
                }
            }
        }
    }

    /// The PVT-aware half-period bound is monotone in the combined ordering:
    /// higher temperature, lower VCCINT, and a worse process corner all increase
    /// (or keep) the bound. This is the shape property a worst-case operating
    /// point search relies on.
    #[test]
    fn test_pvt_half_ns_monotone_combined() {
        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        let corners = [ProcessCorner::Ff, ProcessCorner::Tt, ProcessCorner::Ss];
        // Iterate over every pair of contexts where ctx2 is "worse or equal" on
        // all three axes. This is not a full lattice pair enumeration; it checks
        // the monotone path property.
        for i in 0..temps.len() {
            for j in 0..vccints.len() {
                for k in 0..corners.len() {
                    let ctx_bestish = PvtContext {
                        temp_c: temps[i.min(temps.len() - 1)],
                        vccint_mv: vccints[j.max(0)],
                        vccaux_mv: 2700,
                        process_corner: corners[k.min(corners.len() - 1)].clone(),
                    };
                    let ctx_worstish = PvtContext {
                        temp_c: temps[temps.len() - 1],
                        vccint_mv: vccints[0],
                        vccaux_mv: 2700,
                        process_corner: ProcessCorner::Ss,
                    };
                    let half_bestish = n25q128_min_sck_half_ns_pvt(&ctx_bestish);
                    let half_worstish = n25q128_min_sck_half_ns_pvt(&ctx_worstish);
                    assert!(
                        half_bestish <= half_worstish,
                        "PVT half-period bound not monotone combined: bestish {} ns > worstish {} ns",
                        half_bestish, half_worstish
                    );
                }
            }
        }

        // Spot-check specific axis-combined pairs.
        let ctx_a = PvtContext {
            temp_c: -40,
            vccint_mv: 1100,
            vccaux_mv: 2700,
            process_corner: ProcessCorner::Ff,
        };
        let ctx_b = PvtContext {
            temp_c: 85,
            vccint_mv: 900,
            vccaux_mv: 2700,
            process_corner: ProcessCorner::Ss,
        };
        assert!(
            n25q128_min_sck_half_ns_pvt(&ctx_a) <= n25q128_min_sck_half_ns_pvt(&ctx_b),
            "combined PVT monotonicity failed on explicit best/worst pair"
        );
    }

    /// The PVT-aware half-period bound is maximized at the worst-case operating
    /// point (max temperature, min VCCINT, ss corner). This mirrors the Lean 4
    /// `pvt_half_ns_worst_case_bound` lemma and is the regression fact a finite
    /// grid-search validation relies on.
    #[test]
    fn test_pvt_half_ns_worst_case_bound() {
        let worst = PvtContext {
            temp_c: PVT_TEMP_MAX_C,
            vccint_mv: PVT_VCCINT_MIN_MV,
            vccaux_mv: 2700,
            process_corner: ProcessCorner::Ss,
        };
        let worst_bound = n25q128_min_sck_half_ns_pvt(&worst);

        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        let corners = [ProcessCorner::Ff, ProcessCorner::Tt, ProcessCorner::Ss];
        for temp in temps {
            for vccint in vccints {
                for corner in &corners {
                    let ctx = PvtContext {
                        temp_c: temp,
                        vccint_mv: vccint,
                        vccaux_mv: 2700,
                        process_corner: corner.clone(),
                    };
                    let bound = n25q128_min_sck_half_ns_pvt(&ctx);
                    assert!(
                        bound <= worst_bound,
                        "PVT half-period bound {} ns exceeds worst-case {} ns at temp={} °C, vccint={} mV, corner={:?}",
                        bound, worst_bound, temp, vccint, corner
                    );
                }
            }
        }
    }

    /// Regression: the PVT-aware minimum SCK half-period bound must be at least
    /// the nominal 6 ns across the entire operating rectangle. This mirrors the
    /// Lean 4 `pvt_half_ns_at_least_nominal` lemma and catches accidental
    /// coefficient changes that would shrink the envelope below the datasheet.
    #[test]
    fn test_pvt_half_ns_lower_bound_across_operating_rectangle() {
        let temps = [PVT_TEMP_MIN_C, -20_i64, 0_i64, 25_i64, PVT_TEMP_MAX_C];
        let vccints = [PVT_VCCINT_MIN_MV, 950_u64, 1000_u64, 1050_u64, PVT_VCCINT_MAX_MV];
        let corners = [ProcessCorner::Ff, ProcessCorner::Tt, ProcessCorner::Ss];
        for temp in temps {
            for vccint in vccints {
                for corner in &corners {
                    let ctx = PvtContext {
                        temp_c: temp,
                        vccint_mv: vccint,
                        vccaux_mv: 2700,
                        process_corner: corner.clone(),
                    };
                    let half_ns = n25q128_min_sck_half_ns_pvt(&ctx);
                    assert!(
                        half_ns >= 6,
                        "PVT half-period bound {} ns below nominal 6 ns at temp={} °C, vccint={} mV, corner={:?}",
                        half_ns, temp, vccint, corner
                    );
                }
            }
        }
    }

    #[test]
    fn test_parse_pvt_context_roundtrip() {
        let pvt = write_pvt_context_json(
            "roundtrip",
            &serde_json::json!({"temp_c":-40,"vccint_mv":1100,"vccaux_mv":2700,"process_corner":"ff"}),
        );
        let ctx = parse_pvt_context(&pvt).unwrap();
        assert_eq!(ctx.temp_c, -40);
        assert_eq!(ctx.vccint_mv, 1100);
        assert_eq!(ctx.vccaux_mv, 2700);
        assert_eq!(ctx.process_corner, ProcessCorner::Ff);
        std::fs::remove_file(&pvt).unwrap();
    }

    #[test]
    fn test_cclk_nominal_hz_matches_lean() {
        assert_eq!(cclk_nominal_hz(0), 2_500_000);
        assert_eq!(cclk_nominal_hz(1), 4_200_000);
        assert_eq!(cclk_nominal_hz(2), 6_600_000);
        assert_eq!(cclk_nominal_hz(3), 10_000_000);
        assert_eq!(cclk_nominal_hz(4), 12_500_000);
        assert_eq!(cclk_nominal_hz(5), 16_700_000);
        assert_eq!(cclk_nominal_hz(6), 25_000_000);
        assert_eq!(cclk_nominal_hz(7), 33_300_000);
        assert_eq!(cclk_nominal_hz(8), 0);
    }

    #[test]
    fn test_pvt_envelope_margin_ns_zero_freq() {
        assert_eq!(pvt_envelope_margin_ns(0), None);
        assert_eq!(pvt_envelope_margin_ns(cclk_nominal_hz(255)), None);
    }

    #[test]
    fn test_pvt_envelope_margin_ns_2_5mhz() {
        // Worst-case bound is 13 ns. 2.5 MHz period = 400 ns, half = 200 ns.
        // Margin = 200 - 13 = 187 ns.
        let margin = pvt_envelope_margin_ns(cclk_nominal_hz(0)).unwrap();
        assert_eq!(margin, 187, "2.5 MHz OSCFSEL should have 187 ns worst-case margin");
    }

    #[test]
    fn test_pvt_envelope_margin_ns_33mhz() {
        // 33.3 MHz period = 30 ns, half = 15 ns. Margin = 15 - 13 = 2 ns.
        let margin = pvt_envelope_margin_ns(cclk_nominal_hz(7)).unwrap();
        assert_eq!(margin, 2, "33.3 MHz OSCFSEL should have 2 ns worst-case margin");
    }

    #[test]
    fn test_recommendation_success() {
        let rec = recommendation_from_conclusion(
            "DONE=HIGH: board boots from flash",
            Some(3),
            Some(3),
        );
        assert_eq!(rec["action"], "success");
        assert_eq!(rec["oscfsel"], 3);
        assert_eq!(rec["first_working_oscfsel"], 3);
    }

    #[test]
    fn test_recommendation_try_next_without_first_working() {
        let rec = recommendation_from_conclusion(
            "H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants",
            Some(2),
            None,
        );
        assert_eq!(rec["action"], "try_next_oscfsel");
        let steps = rec["next_steps"].as_array().unwrap();
        assert!(steps.iter().any(|s| s.as_str().unwrap().contains("next slower")));
    }

    #[test]
    fn test_recommendation_try_next_with_first_working() {
        let rec = recommendation_from_conclusion(
            "H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants",
            Some(4),
            Some(3),
        );
        assert_eq!(rec["action"], "try_next_oscfsel");
        let steps = rec["next_steps"].as_array().unwrap();
        assert!(steps.iter().any(|s| s.as_str().unwrap().contains("Use the first working")));
    }

    #[test]
    fn test_recommendation_mode_mismatch() {
        let rec = recommendation_from_conclusion(
            "MODE_MISMATCH: mode-pin strapping issue",
            Some(1),
            None,
        );
        assert_eq!(rec["action"], "inspect_mode_straps");
    }

    #[test]
    fn test_cold_por_mock_relay() {
        let root = repo_root().unwrap();
        // Any existing bitstream will do for the mock; fall back to the demo.
        let bit = root
            .join("fpga")
            .join("verilog")
            .join("ternary_mac_demo_top_200t.bit");
        let log_dir = std::env::temp_dir().join(format!("tri_cold_por_mock_{}", std::process::id()));
        std::fs::create_dir_all(&log_dir).unwrap();
        let out = cold_por(
            &bit,
            "MOCK",
            3,
            0,
            None,
            Some(&log_dir),
        );
        if bit.is_file() {
            out.unwrap();
            let entries: Vec<_> = std::fs::read_dir(&log_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.starts_with("boot-log-cold-por-mock-") && name.ends_with(".json")
                })
                .collect();
            assert_eq!(entries.len(), 1, "expected one mock log file");
            let content = std::fs::read_to_string(entries[0].path()).unwrap();
            let log: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(log["relay_port"], "MOCK");
            assert_eq!(log["relay_mock"], true);
            assert!(log["conclusion"].as_str().unwrap().contains("DONE=HIGH"));
            assert_eq!(log["samples"].as_array().unwrap().len(), 3);
        } else {
            // If no bitstream exists, cold_por should fail with "not found".
            assert!(out.is_err());
        }
        let _ = std::fs::remove_dir_all(&log_dir);
    }

    /// Standalone Lean integration test: a synthetic raw-ns capture is exported
    /// with `--standalone`, then copied into a minimal temporary `lake` package
    /// that depends on the local Trinity library. The package must typecheck
    /// with `lake build`, proving the generated theorem is consumable outside
    /// the monorepo.
    #[test]
    fn test_measured_to_lean_standalone_lake_package_builds() {
        let root = repo_root().unwrap();
        let trinity_pkg = root.join("proofs").join("lean4");
        if !trinity_pkg.join("lakefile.lean").is_file() {
            // Skip if the Trinity package is not present in this checkout.
            return;
        }

        // Generate a synthetic raw-ns capture.
        let m = MeasuredCclkRawNs {
            period_ns: 40,
            sck_low_ns: 20,
            sck_high_ns: 20,
            source: "synthetic".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let tmp = std::env::temp_dir().join(format!(
            "tri_standalone_lake_in_{}.json",
            std::process::id()
        ));
        std::fs::write(&tmp, json).unwrap();

        let generated = std::env::temp_dir().join(format!(
            "tri_standalone_lake_generated_{}.lean",
            std::process::id()
        ));
        let out = measured_to_lean(
            Some(&tmp), None, None, None, None, None, None, 0, None, None, None, Some(&generated), "measured_cclk", false, None, false, true, true, false,
        );
        assert!(out.is_ok(), "measured-to-lean standalone should succeed: {:?}", out);
        assert!(generated.is_file(), "generated Lean file should exist");

        // Create a minimal temporary lake package that consumes the theorem.
        let pkg_dir = std::env::temp_dir().join(format!(
            "tri_standalone_lake_pkg_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&pkg_dir);
        std::fs::create_dir_all(pkg_dir.join(".lake")).unwrap();

        let trinity_path = trinity_pkg.canonicalize().unwrap_or_else(|_| trinity_pkg.clone());
        let lakefile = format!(
            "import Lake\nopen Lake DSL\n\npackage StandaloneTest where\n\nrequire Trinity from \"{}\"\n\n@[default_target]\nlean_lib StandaloneTest where\n",
            trinity_path.display().to_string().replace('\\', "/")
        );
        std::fs::write(pkg_dir.join("lakefile.lean"), lakefile).unwrap();
        std::fs::copy(&generated, pkg_dir.join("StandaloneTest.lean")).unwrap();

        // Build the temporary package. This reuses the local Trinity/.lake cache
        // because the dependency is a local path.
        let lake_status = std::process::Command::new("lake")
            .arg("build")
            .current_dir(&pkg_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status();

        // Clean up inputs before assertions so failures still remove temp files.
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&generated);
        let _ = std::fs::remove_dir_all(&pkg_dir);

        let status = lake_status.expect("lake command should be available");
        assert!(
            status.success(),
            "temporary lake package consuming standalone measured-to-lean output should build"
        );
    }
}
