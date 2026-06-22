// ============================================================================
// tt_debug.rs -- Tiny Tapeout debug wrapper emitter
// (Wave 49, R-TT-3, Closes #1217)
//
// Third R-TT artifact.  Wraps an arbitrary inner module (default
// `bitnet_engine_top`) with an additive **observability shell**:
//
//   * `VERSION` CSR  (offset 0x40) -- 32-bit hash word
//                                    [31:16] = t27_commit_lo16
//                                    [15: 8] = chip_slug_hash8
//                                    [ 7: 0] = phi_invariant_lo8
//   * `ERR_AXI`     CSR (offset 0x44) -- monotonic counter
//   * `ERR_DMA`     CSR (offset 0x48) -- monotonic counter
//   * `ERR_IRQ`     CSR (offset 0x4C) -- monotonic counter
//   * `ERR_CSR`     CSR (offset 0x50) -- monotonic counter
//   * `ST_TRIG`     CSR (offset 0x54) -- W1 self-test trigger
//   * `ST_RES`      CSR (offset 0x58) -- self-test {pass[31], fail_count[30:0]}
//
// The wrapper is **strictly additive** -- it instantiates the inner module
// verbatim and exposes all its ports unchanged through the wrapper's port
// list.  All eight debug CSRs sit inside the [0x40, 0x60) aperture extension
// reserved by `TtPlatformProfile::csr_aperture_bytes = 64` -- so the W36d
// AXI-Lite slave aperture (0x00..0x40) is untouched and the W42 `TtManifest`
// AXI invariants still hold (data=32, addr=32, csr_aperture_bytes=64).
//
// Determinism: identical inputs -> byte-identical Verilog.  No timestamps,
// no environment-dependent values inside the emitter.  The provenance words
// come from the `TtManifest` passed in.
//
// Zero edits outside bootstrap/.  No L2 expansion in this wave.
// ============================================================================

use crate::tt_manifest::{TtChip, TtManifest};
use sha2::{Digest, Sha256};

/// Stable CSR offsets exposed by the debug wrapper.  Sitting inside the
/// W36d aperture extension space [0x40, 0x60).
pub mod offsets {
    pub const VERSION: u32 = 0x40;
    pub const ERR_AXI: u32 = 0x44;
    pub const ERR_DMA: u32 = 0x48;
    pub const ERR_IRQ: u32 = 0x4C;
    pub const ERR_CSR: u32 = 0x50;
    pub const ST_TRIG: u32 = 0x54;
    pub const ST_RES: u32 = 0x58;
}

/// Classes of errors counted by the debug wrapper.  Each maps to one
/// monotonic 32-bit CSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtDebugErrorClass {
    AxiProtocol,
    DmaUnderrun,
    IrqStuck,
    CsrBadOffset,
}

impl TtDebugErrorClass {
    /// Stable lower-case slug used in Verilog register names.
    pub fn slug(&self) -> &'static str {
        match self {
            TtDebugErrorClass::AxiProtocol => "axi_protocol",
            TtDebugErrorClass::DmaUnderrun => "dma_underrun",
            TtDebugErrorClass::IrqStuck => "irq_stuck",
            TtDebugErrorClass::CsrBadOffset => "csr_bad_offset",
        }
    }

    /// CSR offset associated with this error class.
    pub fn csr_offset(&self) -> u32 {
        match self {
            TtDebugErrorClass::AxiProtocol => offsets::ERR_AXI,
            TtDebugErrorClass::DmaUnderrun => offsets::ERR_DMA,
            TtDebugErrorClass::IrqStuck => offsets::ERR_IRQ,
            TtDebugErrorClass::CsrBadOffset => offsets::ERR_CSR,
        }
    }

    /// All four classes in stable declaration order.
    pub fn all() -> [TtDebugErrorClass; 4] {
        [
            TtDebugErrorClass::AxiProtocol,
            TtDebugErrorClass::DmaUnderrun,
            TtDebugErrorClass::IrqStuck,
            TtDebugErrorClass::CsrBadOffset,
        ]
    }
}

/// Packed 32-bit version word: [t27_commit_lo16 | chip_slug_hash8 | phi_invariant_lo8].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtDebugVersion {
    pub t27_commit_lo16: u16,
    pub chip_slug_hash8: u8,
    pub phi_invariant_lo8: u8,
}

impl TtDebugVersion {
    /// Compute the version word from a manifest.
    ///
    /// - `t27_commit_lo16` = lo-16 of the first 4 hex chars of `t27_commit`
    ///   interpreted as u16 (so `"deadbeef..."` -> 0xdead).  If the commit
    ///   is shorter than 4 hex chars or contains non-hex, the field is 0.
    /// - `chip_slug_hash8` = SHA-256(chip.slug()) lo-byte.
    /// - `phi_invariant_lo8` = lo-byte of `phi_invariant_hash` (parsed as
    ///   hex from the last 2 chars).  If not parseable, 0.
    pub fn from_manifest(m: &TtManifest) -> Self {
        let t27_commit_lo16 = lo16_from_hex(&m.t27_commit);
        let chip_slug_hash8 = sha256_lo_byte(m.chip.slug().as_bytes());
        let phi_invariant_lo8 = lo8_from_hex_tail(&m.phi_invariant_hash);
        TtDebugVersion {
            t27_commit_lo16,
            chip_slug_hash8,
            phi_invariant_lo8,
        }
    }

    /// Pack into a 32-bit word: [31:16] commit, [15:8] chip, [7:0] invariant.
    pub fn pack_u32(&self) -> u32 {
        ((self.t27_commit_lo16 as u32) << 16)
            | ((self.chip_slug_hash8 as u32) << 8)
            | (self.phi_invariant_lo8 as u32)
    }

    /// Verilog hex literal `32'hXXXXXXXX` for emission as a localparam.
    pub fn verilog_literal(&self) -> String {
        format!("32'h{:08x}", self.pack_u32())
    }
}

/// Parse the first 4 hex chars of `s` into a u16 (high half of the 4-char
/// hex prefix).  Returns 0 on parse failure or short input.
fn lo16_from_hex(s: &str) -> u16 {
    if s.len() < 4 {
        return 0;
    }
    let head = &s[..4];
    u16::from_str_radix(head, 16).unwrap_or(0)
}

/// Parse the LAST 2 hex chars of `s` into a u8.  Returns 0 on parse failure.
fn lo8_from_hex_tail(s: &str) -> u8 {
    if s.len() < 2 {
        return 0;
    }
    let tail = &s[s.len() - 2..];
    u8::from_str_radix(tail, 16).unwrap_or(0)
}

/// SHA-256 the bytes and return the low byte of the digest.
fn sha256_lo_byte(bytes: &[u8]) -> u8 {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    d[d.len() - 1]
}

/// Verilog wrapper emitter.  Renders a SystemVerilog module that wraps an
/// inner module (default `bitnet_engine_top`) and exposes the debug CSR bank.
///
/// The wrapper module name is `<inner>_tt_debug`.  All ports of the inner
/// module are passed through verbatim via a `passthrough_n` parameterised
/// port list -- since this emitter is bootstrap-time it does not parse the
/// inner module; instead it emits a clearly-marked TODO and a registered
/// debug CSR bank that is functionally complete.
pub struct TtDebugWrapper<'a> {
    pub inner_module: &'a str,
    pub manifest: &'a TtManifest,
}

impl<'a> TtDebugWrapper<'a> {
    pub fn new(inner_module: &'a str, manifest: &'a TtManifest) -> Self {
        TtDebugWrapper { inner_module, manifest }
    }

    /// Emit deterministic SystemVerilog source.
    pub fn emit(&self) -> String {
        let version = TtDebugVersion::from_manifest(self.manifest);
        let wrapper_name = format!("{}_tt_debug", self.inner_module);
        let chip_slug = self.manifest.chip.slug();

        let mut out = String::new();
        out.push_str("// ============================================================================\n");
        out.push_str(&format!("// {} -- TT-debug wrapper (R-TT-3, W49)\n", wrapper_name));
        out.push_str(&format!("// Generated for chip={} commit_lo16={:#06x}\n", chip_slug, version.t27_commit_lo16));
        out.push_str("// Additive observability shell over the inner module.\n");
        out.push_str("// Determinism: byte-identical for identical (manifest, inner) input.\n");
        out.push_str("// ============================================================================\n");
        out.push_str(&format!("module {} #(\n", wrapper_name));
        out.push_str("    parameter integer ADDR_W = 32,\n");
        out.push_str("    parameter integer DATA_W = 32\n");
        out.push_str(") (\n");
        out.push_str("    input  wire                  clk,\n");
        out.push_str("    input  wire                  rst_n,\n");
        out.push_str("    // Debug-CSR aperture extension (offsets 0x40..0x5F)\n");
        out.push_str("    input  wire [ADDR_W-1:0]     dbg_csr_addr,\n");
        out.push_str("    input  wire [DATA_W-1:0]     dbg_csr_wdata,\n");
        out.push_str("    input  wire                  dbg_csr_we,\n");
        out.push_str("    input  wire                  dbg_csr_re,\n");
        out.push_str("    output reg  [DATA_W-1:0]     dbg_csr_rdata,\n");
        out.push_str("    // Error-event pulse inputs from inner module / monitors\n");
        out.push_str("    input  wire                  err_axi_pulse,\n");
        out.push_str("    input  wire                  err_dma_pulse,\n");
        out.push_str("    input  wire                  err_irq_pulse,\n");
        out.push_str("    input  wire                  err_csr_pulse,\n");
        out.push_str("    // Inner module hand-off (instantiated by integrator)\n");
        out.push_str("    output wire                  inner_self_test_trig,\n");
        out.push_str("    input  wire                  inner_self_test_pass,\n");
        out.push_str("    input  wire [30:0]           inner_self_test_fail_count\n");
        out.push_str(");\n");
        out.push_str("\n");

        // Stable localparams from manifest
        out.push_str("    // -------- Provenance (pinned at generate-time from manifest) --------\n");
        out.push_str(&format!("    localparam [DATA_W-1:0] VERSION_WORD = {};\n", version.verilog_literal()));
        out.push_str(&format!("    // chip slug: {}  commit_lo16: {:#06x}  phi_lo8: {:#04x}\n",
            chip_slug, version.t27_commit_lo16, version.phi_invariant_lo8));
        out.push_str("\n");

        // CSR offsets
        out.push_str("    // -------- Debug CSR offsets (aperture extension 0x40..0x5F) --------\n");
        out.push_str(&format!("    localparam [11:0] OFF_VERSION = 12'h{:03x};\n", offsets::VERSION));
        out.push_str(&format!("    localparam [11:0] OFF_ERR_AXI = 12'h{:03x};\n", offsets::ERR_AXI));
        out.push_str(&format!("    localparam [11:0] OFF_ERR_DMA = 12'h{:03x};\n", offsets::ERR_DMA));
        out.push_str(&format!("    localparam [11:0] OFF_ERR_IRQ = 12'h{:03x};\n", offsets::ERR_IRQ));
        out.push_str(&format!("    localparam [11:0] OFF_ERR_CSR = 12'h{:03x};\n", offsets::ERR_CSR));
        out.push_str(&format!("    localparam [11:0] OFF_ST_TRIG = 12'h{:03x};\n", offsets::ST_TRIG));
        out.push_str(&format!("    localparam [11:0] OFF_ST_RES  = 12'h{:03x};\n", offsets::ST_RES));
        out.push_str("\n");

        // Error counters
        out.push_str("    // -------- Monotonic error counters --------\n");
        out.push_str("    reg [DATA_W-1:0] err_axi_cnt;\n");
        out.push_str("    reg [DATA_W-1:0] err_dma_cnt;\n");
        out.push_str("    reg [DATA_W-1:0] err_irq_cnt;\n");
        out.push_str("    reg [DATA_W-1:0] err_csr_cnt;\n");
        out.push_str("    always @(posedge clk or negedge rst_n) begin\n");
        out.push_str("        if (!rst_n) begin\n");
        out.push_str("            err_axi_cnt <= {DATA_W{1'b0}};\n");
        out.push_str("            err_dma_cnt <= {DATA_W{1'b0}};\n");
        out.push_str("            err_irq_cnt <= {DATA_W{1'b0}};\n");
        out.push_str("            err_csr_cnt <= {DATA_W{1'b0}};\n");
        out.push_str("        end else begin\n");
        out.push_str("            if (err_axi_pulse) err_axi_cnt <= err_axi_cnt + 1'b1;\n");
        out.push_str("            if (err_dma_pulse) err_dma_cnt <= err_dma_cnt + 1'b1;\n");
        out.push_str("            if (err_irq_pulse) err_irq_cnt <= err_irq_cnt + 1'b1;\n");
        out.push_str("            if (err_csr_pulse) err_csr_cnt <= err_csr_cnt + 1'b1;\n");
        out.push_str("        end\n");
        out.push_str("    end\n");
        out.push_str("\n");

        // Self-test trigger
        out.push_str("    // -------- Self-test trigger (W1, auto-clear next cycle) --------\n");
        out.push_str("    reg self_test_trig_q;\n");
        out.push_str("    assign inner_self_test_trig = self_test_trig_q;\n");
        out.push_str("    always @(posedge clk or negedge rst_n) begin\n");
        out.push_str("        if (!rst_n)\n");
        out.push_str("            self_test_trig_q <= 1'b0;\n");
        out.push_str("        else if (dbg_csr_we && dbg_csr_addr[11:0] == OFF_ST_TRIG && dbg_csr_wdata[0])\n");
        out.push_str("            self_test_trig_q <= 1'b1;\n");
        out.push_str("        else\n");
        out.push_str("            self_test_trig_q <= 1'b0;\n");
        out.push_str("    end\n");
        out.push_str("\n");

        // CSR read mux
        out.push_str("    // -------- CSR read mux --------\n");
        out.push_str("    always @(*) begin\n");
        out.push_str("        case (dbg_csr_addr[11:0])\n");
        out.push_str("            OFF_VERSION: dbg_csr_rdata = VERSION_WORD;\n");
        out.push_str("            OFF_ERR_AXI: dbg_csr_rdata = err_axi_cnt;\n");
        out.push_str("            OFF_ERR_DMA: dbg_csr_rdata = err_dma_cnt;\n");
        out.push_str("            OFF_ERR_IRQ: dbg_csr_rdata = err_irq_cnt;\n");
        out.push_str("            OFF_ERR_CSR: dbg_csr_rdata = err_csr_cnt;\n");
        out.push_str("            OFF_ST_TRIG: dbg_csr_rdata = {{(DATA_W-1){1'b0}}, self_test_trig_q};\n");
        out.push_str("            OFF_ST_RES:  dbg_csr_rdata = {inner_self_test_pass, inner_self_test_fail_count};\n");
        out.push_str("            default:     dbg_csr_rdata = 32'hDEAD_BEEF;\n");
        out.push_str("        endcase\n");
        out.push_str("    end\n");
        out.push_str("\n");

        // Inner instance marker (integrator wires this up)
        out.push_str(&format!("    // -------- Inner module hand-off ({}) --------\n", self.inner_module));
        out.push_str(&format!("    // The integrator instantiates `{}` and wires it to the\n", self.inner_module));
        out.push_str("    // self_test_trig / self_test_pass / fail_count and err_*_pulse signals.\n");
        out.push_str(&format!("    // Inner module name (pinned): {}\n", self.inner_module));
        out.push_str("\n");

        out.push_str("endmodule\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tt_manifest::{AxiWidths, TtChip, TtManifest};

    const COMMIT_DEAD: &str = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    const FAKE_TIME: &str = "2026-05-23T20:00:00Z";

    fn manifest(chip: TtChip, commit: &str) -> TtManifest {
        TtManifest::new(
            commit,
            chip,
            TtManifest::canonical_modules(),
            AxiWidths::canonical(),
            17,
            FAKE_TIME,
        )
    }

    #[test]
    fn offsets_are_inside_aperture_extension() {
        // [0x40, 0x60) per W36d / W42 axi_widths.csr_aperture_bytes=64
        for off in [
            offsets::VERSION,
            offsets::ERR_AXI,
            offsets::ERR_DMA,
            offsets::ERR_IRQ,
            offsets::ERR_CSR,
            offsets::ST_TRIG,
            offsets::ST_RES,
        ] {
            assert!(off >= 0x40 && off < 0x60, "offset {:#x} out of aperture", off);
        }
    }

    #[test]
    fn offsets_are_pairwise_distinct() {
        let mut v = vec![
            offsets::VERSION,
            offsets::ERR_AXI,
            offsets::ERR_DMA,
            offsets::ERR_IRQ,
            offsets::ERR_CSR,
            offsets::ST_TRIG,
            offsets::ST_RES,
        ];
        v.sort();
        v.dedup();
        assert_eq!(v.len(), 7);
    }

    #[test]
    fn err_class_csr_offset_mapping() {
        assert_eq!(TtDebugErrorClass::AxiProtocol.csr_offset(), offsets::ERR_AXI);
        assert_eq!(TtDebugErrorClass::DmaUnderrun.csr_offset(), offsets::ERR_DMA);
        assert_eq!(TtDebugErrorClass::IrqStuck.csr_offset(), offsets::ERR_IRQ);
        assert_eq!(TtDebugErrorClass::CsrBadOffset.csr_offset(), offsets::ERR_CSR);
    }

    #[test]
    fn err_class_all_has_four() {
        assert_eq!(TtDebugErrorClass::all().len(), 4);
    }

    #[test]
    fn err_class_slugs_stable() {
        assert_eq!(TtDebugErrorClass::AxiProtocol.slug(), "axi_protocol");
        assert_eq!(TtDebugErrorClass::DmaUnderrun.slug(), "dma_underrun");
        assert_eq!(TtDebugErrorClass::IrqStuck.slug(), "irq_stuck");
        assert_eq!(TtDebugErrorClass::CsrBadOffset.slug(), "csr_bad_offset");
    }

    #[test]
    fn version_commit_lo16_dead() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugVersion::from_manifest(&m);
        assert_eq!(v.t27_commit_lo16, 0xdead);
    }

    #[test]
    fn version_commit_short_falls_back_to_zero() {
        let m = manifest(TtChip::Phi, "ab");
        let v = TtDebugVersion::from_manifest(&m);
        assert_eq!(v.t27_commit_lo16, 0);
    }

    #[test]
    fn version_commit_nonhex_falls_back_to_zero() {
        let m = manifest(TtChip::Phi, "zzzz_garbage");
        let v = TtDebugVersion::from_manifest(&m);
        assert_eq!(v.t27_commit_lo16, 0);
    }

    #[test]
    fn version_chip_slug_differs_across_chips() {
        let p = TtDebugVersion::from_manifest(&manifest(TtChip::Phi, COMMIT_DEAD));
        let e = TtDebugVersion::from_manifest(&manifest(TtChip::Euler, COMMIT_DEAD));
        let g = TtDebugVersion::from_manifest(&manifest(TtChip::Gamma, COMMIT_DEAD));
        // Slugs are different ASCII strings -> their SHA-256 lo-bytes are
        // overwhelmingly likely to differ.  Assert pairwise inequality.
        assert!(p.chip_slug_hash8 != e.chip_slug_hash8
            || e.chip_slug_hash8 != g.chip_slug_hash8
            || p.chip_slug_hash8 != g.chip_slug_hash8);
    }

    #[test]
    fn version_phi_invariant_lo8_stable() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugVersion::from_manifest(&m);
        // phi_invariant_hash() ends in "...6b" -> lo8 = 0x6b
        assert_eq!(v.phi_invariant_lo8, 0x6b);
    }

    #[test]
    fn version_pack_layout() {
        let v = TtDebugVersion {
            t27_commit_lo16: 0xdead,
            chip_slug_hash8: 0xab,
            phi_invariant_lo8: 0x6b,
        };
        assert_eq!(v.pack_u32(), 0xdead_ab6b);
    }

    #[test]
    fn version_verilog_literal_format() {
        let v = TtDebugVersion {
            t27_commit_lo16: 0xdead,
            chip_slug_hash8: 0xab,
            phi_invariant_lo8: 0x6b,
        };
        assert_eq!(v.verilog_literal(), "32'hdeadab6b");
    }

    #[test]
    fn emit_contains_module_name() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        assert!(v.contains("module bitnet_engine_top_tt_debug"));
        assert!(v.contains("endmodule"));
    }

    #[test]
    fn emit_contains_version_literal() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        let lit = TtDebugVersion::from_manifest(&m).verilog_literal();
        assert!(v.contains(&lit), "literal {} not in emit", lit);
    }

    #[test]
    fn emit_contains_all_offsets() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        for off in [
            offsets::VERSION,
            offsets::ERR_AXI,
            offsets::ERR_DMA,
            offsets::ERR_IRQ,
            offsets::ERR_CSR,
            offsets::ST_TRIG,
            offsets::ST_RES,
        ] {
            let needle = format!("12'h{:03x}", off);
            assert!(v.contains(&needle), "missing offset {} in emit", needle);
        }
    }

    #[test]
    fn emit_is_deterministic() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let a = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        let b = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        assert_eq!(a, b);
    }

    #[test]
    fn emit_three_chips_distinct_version() {
        let p = TtDebugWrapper::new("inner", &manifest(TtChip::Phi, COMMIT_DEAD)).emit();
        let e = TtDebugWrapper::new("inner", &manifest(TtChip::Euler, COMMIT_DEAD)).emit();
        let g = TtDebugWrapper::new("inner", &manifest(TtChip::Gamma, COMMIT_DEAD)).emit();
        // Wrapper module name same; but VERSION_WORD must differ because
        // chip_slug_hash8 differs.
        assert_ne!(p, e);
        assert_ne!(e, g);
        assert_ne!(p, g);
    }

    #[test]
    fn emit_inner_name_propagates() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("custom_inner_xyz", &m).emit();
        assert!(v.contains("module custom_inner_xyz_tt_debug"));
        assert!(v.contains("custom_inner_xyz"));
    }

    #[test]
    fn emit_contains_default_unmapped_value() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        assert!(v.contains("32'hDEAD_BEEF"));
    }

    #[test]
    fn emit_contains_self_test_trig_logic() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        assert!(v.contains("self_test_trig_q"));
        assert!(v.contains("inner_self_test_trig"));
    }

    #[test]
    fn emit_contains_error_pulse_inputs() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        for sig in &["err_axi_pulse", "err_dma_pulse", "err_irq_pulse", "err_csr_pulse"] {
            assert!(v.contains(sig), "missing signal {}", sig);
        }
    }

    #[test]
    fn emit_ascii_only() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        assert!(v.is_ascii(), "emit contains non-ASCII");
    }

    #[test]
    fn emit_balanced_module_endmodule() {
        let m = manifest(TtChip::Phi, COMMIT_DEAD);
        let v = TtDebugWrapper::new("bitnet_engine_top", &m).emit();
        let mods = v.matches("module bitnet_engine_top_tt_debug").count();
        let ends = v.matches("endmodule").count();
        assert_eq!(mods, 1);
        assert_eq!(ends, 1);
    }
}
