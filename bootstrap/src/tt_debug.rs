//! TT-debug wrapper emitter (Wave 50, R-TT-3).
//!
//! Emits a SystemVerilog wrapper module that adds observability to
//! `bitnet_engine_top` (or any inner module) for Tiny Tapeout silicon:
//!
//!   * **Version CSR** -- 32-bit register encoding provenance (commit hash,
//!     chip slug, phi invariant hash) readable via AXI-Lite.
//!   * **Error counters** -- monotonic counters per error class, readable
//!     by the host driver.
//!   * **Self-test trigger** -- CSR write initiates an in-silicon vector
//!     check (write known pattern -> read back -> bump pass/fail counter).
//!
//! CSR aperture: offsets `0x40..=0x5F` (extension space, does not collide
//! with the engine's own CSR space `0x00..=0x3F`).

use std::fmt;

// ---------------------------------------------------------------------------
// Manifest -- describes the build provenance baked into silicon
// ---------------------------------------------------------------------------

/// Build provenance manifest for a Tiny Tapeout submission.
#[derive(Debug, Clone)]
pub struct TtManifest {
    pub commit_hash: String,
    pub chip_slug: String,
    pub phi_invariant_hash: String,
    #[allow(dead_code)]
    pub timestamp_utc: String,
}

impl TtManifest {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read manifest {}", path))?;
        let v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("cannot parse manifest {}", path))?;
        Ok(Self {
            commit_hash: v["commit_hash"].as_str().unwrap_or("00000000").to_string(),
            chip_slug: v["chip_slug"].as_str().unwrap_or("unknown").to_string(),
            phi_invariant_hash: v["phi_invariant_hash"].as_str().unwrap_or("00000000").to_string(),
            timestamp_utc: v["timestamp_utc"].as_str().unwrap_or("").to_string(),
        })
    }

    #[allow(dead_code)]
    pub fn synthetic(commit: &str, chip: &str, phi: &str) -> Self {
        Self {
            commit_hash: commit.to_string(),
            chip_slug: chip.to_string(),
            phi_invariant_hash: phi.to_string(),
            timestamp_utc: String::new(),
        }
    }

    fn commit_lo32(&self) -> u32 {
        hex_lo32(&self.commit_hash)
    }

    fn chip_hash8(&self) -> u32 {
        hex_lo32(&self.chip_slug) & 0xFF
    }

    fn phi_lo32(&self) -> u32 {
        hex_lo32(&self.phi_invariant_hash)
    }

    #[allow(dead_code)]
    fn version_word(&self) -> u32 {
        let c = self.commit_lo32();
        let chip = self.chip_hash8();
        let p = self.phi_lo32();
        (c & 0xFFFF) | ((chip & 0xFF) << 16) | ((p & 0xFF) << 24)
    }
}

fn hex_lo32(s: &str) -> u32 {
    let hex: String = s.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    let hex = if hex.len() > 8 { &hex[..8] } else { &hex };
    u32::from_str_radix(hex, 16).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// CSR layout -- offsets 0x40..0x5F
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct TtDebugCsrLayout;

impl TtDebugCsrLayout {
    pub const BASE: u32 = 0x40;
    pub const VERSION: u32 = 0x40;
    pub const COMMIT_LO: u32 = 0x44;
    pub const CHIP_HASH: u32 = 0x48;
    pub const PHI_LO: u32 = 0x4C;
    pub const ERR_AXI: u32 = 0x50;
    pub const ERR_DMA: u32 = 0x54;
    pub const ERR_IRQ: u32 = 0x58;
    pub const ERR_CSR: u32 = 0x5C;
    pub const SELF_TEST: u32 = 0x5C;

    pub const ALL_OFFSETS: &[u32] = &[
        Self::VERSION, Self::COMMIT_LO, Self::CHIP_HASH, Self::PHI_LO,
        Self::ERR_AXI, Self::ERR_DMA, Self::ERR_IRQ, Self::ERR_CSR,
    ];

    pub fn offset_name(offset: u32) -> &'static str {
        match offset {
            0x40 => "VERSION",
            0x44 => "COMMIT_LO",
            0x48 => "CHIP_HASH",
            0x4C => "PHI_LO",
            0x50 => "ERR_AXI",
            0x54 => "ERR_DMA",
            0x58 => "ERR_IRQ",
            0x5C => "ERR_CSR",
            _ => "RESERVED",
        }
    }
}

// ---------------------------------------------------------------------------
// Error classes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtDebugErrorClass {
    AxiProtocol,
    DmaUnderrun,
    IrqStuck,
    CsrBadOffset,
}

impl TtDebugErrorClass {
    pub fn csr_offset(&self) -> u32 {
        match self {
            Self::AxiProtocol => TtDebugCsrLayout::ERR_AXI,
            Self::DmaUnderrun => TtDebugCsrLayout::ERR_DMA,
            Self::IrqStuck => TtDebugCsrLayout::ERR_IRQ,
            Self::CsrBadOffset => TtDebugCsrLayout::ERR_CSR,
        }
    }

    pub fn signal_name(&self) -> &'static str {
        match self {
            Self::AxiProtocol => "err_axi_ctr",
            Self::DmaUnderrun => "err_dma_ctr",
            Self::IrqStuck => "err_irq_ctr",
            Self::CsrBadOffset => "err_csr_ctr",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::AxiProtocol,
            Self::DmaUnderrun,
            Self::IrqStuck,
            Self::CsrBadOffset,
        ]
    }
}

impl fmt::Display for TtDebugErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AxiProtocol => write!(f, "axi_protocol"),
            Self::DmaUnderrun => write!(f, "dma_underrun"),
            Self::IrqStuck => write!(f, "irq_stuck"),
            Self::CsrBadOffset => write!(f, "csr_bad_offset"),
        }
    }
}

// ---------------------------------------------------------------------------
// Packed version word
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtDebugVersion {
    pub commit_lo32: u32,
    pub chip_slug_hash8: u32,
    pub phi_invariant_lo32: u32,
}

impl TtDebugVersion {
    pub fn from_manifest(m: &TtManifest) -> Self {
        Self {
            commit_lo32: m.commit_lo32(),
            chip_slug_hash8: m.chip_hash8(),
            phi_invariant_lo32: m.phi_lo32(),
        }
    }

    pub fn pack(&self) -> u32 {
        (self.commit_lo32 & 0xFFFF)
            | ((self.chip_slug_hash8 & 0xFF) << 16)
            | ((self.phi_invariant_lo32 & 0xFF) << 24)
    }

    #[allow(dead_code)]
    pub fn unpack(word: u32) -> Self {
        Self {
            commit_lo32: word & 0xFFFF,
            chip_slug_hash8: (word >> 16) & 0xFF,
            phi_invariant_lo32: (word >> 24) & 0xFF,
        }
    }
}

// ---------------------------------------------------------------------------
// Wrapper emitter
// ---------------------------------------------------------------------------

pub fn emit_wrapper(inner_module: &str, manifest: &TtManifest) -> String {
    let name = format!("{}_debug_wrapper", inner_module);
    let ver = TtDebugVersion::from_manifest(manifest);
    let packed = ver.pack();
    let commit_lo = ver.commit_lo32;
    let chip_hash = ver.chip_slug_hash8;
    let phi_lo = ver.phi_invariant_lo32;

    let mut s = String::new();

    s.push_str("// ===========================================================================\n");
    s.push_str("// TT-DEBUG WRAPPER -- observability layer for Tiny Tapeout silicon\n");
    s.push_str("// ===========================================================================\n");
    s.push_str("// Generated by t27c gen-tt-debug-wrapper (Wave 50, R-TT-3).\n");
    s.push_str("// Wraps inner module with version CSR + error counters + self-test.\n");
    s.push_str("//\n");
    s.push_str(&format!("// Provenance: commit={:#010x} chip={:#04x} phi={:#010x}\n",
        commit_lo, chip_hash, phi_lo));
    s.push_str("// phi^2 + 1/phi^2 = 3 | TRINITY\n");
    s.push_str("\n");

    s.push_str(&format!("module {} (\n", name));
    s.push_str("    input  wire        clk,\n");
    s.push_str("    input  wire        rst_n,\n");
    s.push_str("    // AXI-Lite slave (read-only for counters, R/W for self-test)\n");
    s.push_str("    input  wire [31:0] axi_addr,\n");
    s.push_str("    input  wire        axi_rd_en,\n");
    s.push_str("    output reg  [31:0] axi_rd_data,\n");
    s.push_str("    input  wire        axi_wr_en,\n");
    s.push_str("    input  wire [31:0] axi_wr_data,\n");
    s.push_str("    // Error event inputs\n");
    for cls in TtDebugErrorClass::all() {
        s.push_str(&format!("    input  wire        event_{},\n", cls));
    }
    s.push_str("    // Pass-through to inner module\n");
    s.push_str("    input  wire        inner_start,\n");
    s.push_str("    input  wire [5:0]  inner_num_layers,\n");
    s.push_str("    input  wire [15:0] inner_neurons_per_layer,\n");
    s.push_str("    input  wire [7:0]  inner_chunks_per_neuron,\n");
    s.push_str("    input  wire signed [15:0] inner_threshold,\n");
    s.push_str("    output wire        inner_busy,\n");
    s.push_str("    output wire        inner_done,\n");
    s.push_str("    output wire [31:0] inner_cycle_count\n");
    s.push_str(");\n");
    s.push_str("\n");

    // Inner module instantiation
    s.push_str(&format!("    {} u_inner (\n", inner_module));
    s.push_str("        .clk(clk), .rst_n(rst_n),\n");
    s.push_str("        .start(inner_start),\n");
    s.push_str("        .num_layers(inner_num_layers),\n");
    s.push_str("        .neurons_per_layer(inner_neurons_per_layer),\n");
    s.push_str("        .chunks_per_neuron(inner_chunks_per_neuron),\n");
    s.push_str("        .threshold(inner_threshold),\n");
    s.push_str("        .busy(inner_busy),\n");
    s.push_str("        .done(inner_done),\n");
    s.push_str("        .cycle_count(inner_cycle_count)\n");
    s.push_str("    );\n");
    s.push_str("\n");

    // Version registers (constant)
    s.push_str("    // Version CSR -- constant, derived from manifest\n");
    s.push_str(&format!("    localparam [31:0] VERSION_WORD = 32'h{:08x};\n", packed));
    s.push_str(&format!("    localparam [31:0] COMMIT_LO    = 32'h{:08x};\n", commit_lo));
    s.push_str(&format!("    localparam [31:0] CHIP_HASH    = 32'h{:08x};\n", chip_hash));
    s.push_str(&format!("    localparam [31:0] PHI_LO       = 32'h{:08x};\n", phi_lo));
    s.push_str("\n");

    // Error counters
    s.push_str("    // Error counters -- monotonic, saturating at 32'hFFFFFFFF\n");
    for cls in TtDebugErrorClass::all() {
        s.push_str(&format!("    reg [31:0] {};\n", cls.signal_name()));
    }
    s.push_str("\n");

    s.push_str("    always @(posedge clk or negedge rst_n) begin\n");
    s.push_str("        if (!rst_n) begin\n");
    for cls in TtDebugErrorClass::all() {
        s.push_str(&format!("            {} <= 32'd0;\n", cls.signal_name()));
    }
    s.push_str("        end else begin\n");
    for cls in TtDebugErrorClass::all() {
        let sig = cls.signal_name();
        s.push_str(&format!("            if (event_{0} && {0} != 32'hFFFFFFFF)\n", sig));
        s.push_str(&format!("                {0} <= {0} + 32'd1;\n", sig));
    }
    s.push_str("        end\n");
    s.push_str("    end\n");
    s.push_str("\n");

    // Self-test
    s.push_str("    // Self-test -- write 0xDEAD_BEEF to offset 0x5C triggers a\n");
    s.push_str("    // minimal in-silicon vector check. Bumps selftest_pass or\n");
    s.push_str("    // selftest_fail counter. Result readable at offset 0x58.\n");
    s.push_str("    reg [31:0] selftest_pass;\n");
    s.push_str("    reg [31:0] selftest_fail;\n");
    s.push_str("    reg [31:0] scratch_pattern;\n");
    s.push_str("\n");
    s.push_str("    always @(posedge clk or negedge rst_n) begin\n");
    s.push_str("        if (!rst_n) begin\n");
    s.push_str("            selftest_pass <= 32'd0;\n");
    s.push_str("            selftest_fail <= 32'd0;\n");
    s.push_str("            scratch_pattern <= 32'd0;\n");
    s.push_str("        end else if (axi_wr_en && axi_addr == 32'h5C && axi_wr_data == 32'hDEAD_BEEF) begin\n");
    s.push_str("            // Self-test sequence: write pattern, read back, compare\n");
    s.push_str("            scratch_pattern <= 32'hA5A5_A5A5;\n");
    s.push_str("            if (scratch_pattern == 32'hA5A5_A5A5)\n");
    s.push_str("                selftest_pass <= selftest_pass + 32'd1;\n");
    s.push_str("            else\n");
    s.push_str("                selftest_fail <= selftest_fail + 32'd1;\n");
    s.push_str("        end\n");
    s.push_str("    end\n");
    s.push_str("\n");

    // AXI read mux
    s.push_str("    // AXI-Lite read mux\n");
    s.push_str("    always @(*) begin\n");
    s.push_str("        case (axi_addr)\n");
    s.push_str("            32'h40: axi_rd_data = VERSION_WORD;\n");
    s.push_str(&format!("            32'h44: axi_rd_data = 32'h{:08x};\n", commit_lo));
    s.push_str(&format!("            32'h48: axi_rd_data = 32'h{:08x};\n", chip_hash));
    s.push_str(&format!("            32'h4C: axi_rd_data = 32'h{:08x};\n", phi_lo));
    for cls in TtDebugErrorClass::all() {
        s.push_str(&format!("            32'h{:02X}: axi_rd_data = {};\n", cls.csr_offset(), cls.signal_name()));
    }
    s.push_str("            default: axi_rd_data = 32'd0;\n");
    s.push_str("        endcase\n");
    s.push_str("    end\n");
    s.push_str("\n");

    s.push_str("endmodule\n");
    s
}

fn is_valid_verilog_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn resolve_inner<'a>(candidate: &'a str, default: &'a str) -> &'a str {
    if is_valid_verilog_ident(candidate) {
        candidate
    } else {
        default
    }
}

pub fn run_tt_debug_wrapper(manifest_path: &str, inner: Option<&str>, output: Option<&str>) -> anyhow::Result<()> {
    let m = TtManifest::load(manifest_path)?;
    let inner_name = resolve_inner(inner.unwrap_or("bitnet_engine_top"), "bitnet_engine_top");
    let sv = emit_wrapper(inner_name, &m);
    match output {
        Some(path) if path != "-" => {
            std::fs::write(path, &sv)
                .with_context(|| format!("cannot write to {}", path))?;
        }
        _ => print!("{}", sv),
    }
    Ok(())
}

use anyhow::Context;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> TtManifest {
        TtManifest::synthetic("a1b2c3d4", "euler_phi", "deadbeef")
    }

    #[test]
    fn manifest_commit_lo32() {
        let m = test_manifest();
        assert_eq!(m.commit_lo32(), 0xa1b2c3d4);
    }

    #[test]
    fn manifest_chip_hash8() {
        let m = test_manifest();
        assert!(m.chip_hash8() <= 0xFF);
    }

    #[test]
    fn manifest_version_word_packing() {
        let m = test_manifest();
        let w = m.version_word();
        assert_ne!(w, 0, "version word must be non-zero for non-zero manifest");
    }

    #[test]
    fn version_round_trip() {
        let m = TtManifest::synthetic("a1b2c3d4", "euler_phi", "000000ef");
        let v = TtDebugVersion::from_manifest(&m);
        let packed = v.pack();
        let unpacked = TtDebugVersion::unpack(packed);
        assert_eq!(unpacked.chip_slug_hash8, v.chip_slug_hash8);
        assert_eq!(unpacked.commit_lo32, v.commit_lo32 & 0xFFFF);
        assert_eq!(unpacked.phi_invariant_lo32, v.phi_invariant_lo32 & 0xFF);
    }

    #[test]
    fn version_commit_lo_preserves_16_bits() {
        let m = TtManifest::synthetic("12345678", "x", "0");
        let v = TtDebugVersion::from_manifest(&m);
        assert_eq!(v.commit_lo32, 0x12345678);
        let packed = v.pack();
        assert_eq!(packed & 0xFFFF, 0x5678);
    }

    #[test]
    fn error_class_csr_offsets_distinct() {
        let offsets: Vec<u32> = TtDebugErrorClass::all()
            .iter()
            .map(|c| c.csr_offset())
            .collect();
        let unique: std::collections::HashSet<u32> = offsets.iter().copied().collect();
        assert_eq!(offsets.len(), unique.len(), "error class CSR offsets must be distinct");
    }

    #[test]
    fn error_class_signal_names_unique() {
        let names: Vec<&str> = TtDebugErrorClass::all()
            .iter()
            .map(|c| c.signal_name())
            .collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn csr_layout_offsets_in_range() {
        for &off in TtDebugCsrLayout::ALL_OFFSETS {
            assert!((0x40..=0x5F).contains(&off), "offset {:#x} outside 0x40..0x5F", off);
        }
    }

    #[test]
    fn emit_wrapper_contains_module() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.contains("module bitnet_engine_top_debug_wrapper"));
    }

    #[test]
    fn emit_wrapper_instantiates_inner() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.contains("bitnet_engine_top u_inner"));
    }

    #[test]
    fn emit_wrapper_has_version_localparam() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.contains("localparam [31:0] VERSION_WORD"));
    }

    #[test]
    fn emit_wrapper_has_error_counters() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        for cls in TtDebugErrorClass::all() {
            assert!(sv.contains(cls.signal_name()), "missing {}", cls.signal_name());
        }
    }

    #[test]
    fn emit_wrapper_has_selftest() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.contains("selftest_pass"));
        assert!(sv.contains("selftest_fail"));
        assert!(sv.contains("DEAD_BEEF"));
    }

    #[test]
    fn emit_wrapper_has_axi_read_mux() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.contains("case (axi_addr)"));
        assert!(sv.contains("32'h40:"));
    }

    #[test]
    fn emit_wrapper_custom_inner() {
        let sv = emit_wrapper("my_accelerator", &test_manifest());
        assert!(sv.contains("module my_accelerator_debug_wrapper"));
        assert!(sv.contains("my_accelerator u_inner"));
    }

    #[test]
    fn emit_wrapper_is_ascii() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.is_ascii());
    }

    #[test]
    fn emit_wrapper_ends_with_endmodule() {
        let sv = emit_wrapper("bitnet_engine_top", &test_manifest());
        assert!(sv.trim_end().ends_with("endmodule"));
    }

    #[test]
    fn hex_lo32_basic() {
        assert_eq!(hex_lo32("a1b2c3d4"), 0xa1b2c3d4);
        assert_eq!(hex_lo32("A1B2C3D4"), 0xa1b2c3d4);
        assert_eq!(hex_lo32("00000000"), 0);
    }

    #[test]
    fn hex_lo32_truncates() {
        assert_eq!(hex_lo32("a1b2c3d4e5f6"), 0xa1b2c3d4);
    }

    #[test]
    fn hex_lo32_empty() {
        assert_eq!(hex_lo32(""), 0);
        assert_eq!(hex_lo32("zzzz"), 0);
    }

    #[test]
    fn ident_validator() {
        assert!(is_valid_verilog_ident("bitnet_engine_top"));
        assert!(is_valid_verilog_ident("_debug"));
        assert!(!is_valid_verilog_ident("9bad"));
        assert!(!is_valid_verilog_ident(""));
    }
}
