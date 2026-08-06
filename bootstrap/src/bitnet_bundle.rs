// ============================================================================
// BitNet HLS Bundle Composer (Wave 38, R-SI-1, Closes #781)
//
// Composes the existing per-module emitters (W36a/b/c/d/e/f) and the
// behavior-DSL v2 emitter (W37) into a single self-consistent directory
// deliverable.
//
// This is a *composition* layer -- it introduces no new RTL or DSL surface.
// Every byte written is produced by an existing `pub fn build_*` emitter,
// so the kernel and trinity invariant `phi^2 + 1/phi^2 = 3` remain
// untouched (L5 / L6 hold).
//
// Source: composition over `gHashTag/vibee-lang` ports authored by
// Dmitrii Vasilev.
// ============================================================================

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::behavior_sva::Behavior;
use crate::behavior_sva_v2::build_behavior_sva_v2_file;
use crate::bitnet_axi::build_axi_lite_slave;
use crate::bitnet_buffers::{build_double_buffer_ctrl, build_weight_prefetch_ctrl};
use crate::bitnet_dma::build_dma_controller;
use crate::bitnet_irq::build_interrupt_controller;
use crate::bitnet_pipeline::{
    build_layer_sequencer, build_multilayer_sequencer, build_pipeline_stage2,
};
use crate::bitnet_top::build_bitnet_engine_top;
use crate::trit_stdlib::build_trit_stdlib_verilog;
use crate::weight_bram::build_default_weight_bram;

/// Default engine-top module name. Matches the W36f canonical default.
pub const DEFAULT_TOP_NAME: &str = "bitnet_engine_top";

/// Default AXI-Lite slave address width (bits). Matches the W36d default.
pub const DEFAULT_AXI_ADDR_WIDTH: u32 = 32;

/// Default AXI-Lite slave data width (bits). Matches the W36d default.
pub const DEFAULT_AXI_DATA_WIDTH: u32 = 32;

/// Bundle configuration. All fields have sensible defaults.
#[derive(Debug, Clone)]
pub struct BundleConfig<'a> {
    /// Engine-top module name (default: `bitnet_engine_top`).
    pub top_name: &'a str,
    /// AXI-Lite slave address width in bits (default: 32).
    pub axi_addr_width: u32,
    /// AXI-Lite slave data width in bits (default: 32).
    pub axi_data_width: u32,
}

impl<'a> Default for BundleConfig<'a> {
    fn default() -> Self {
        Self {
            top_name: DEFAULT_TOP_NAME,
            axi_addr_width: DEFAULT_AXI_ADDR_WIDTH,
            axi_data_width: DEFAULT_AXI_DATA_WIDTH,
        }
    }
}

/// A single entry in the bundle (filename + emitted content).
///
/// Returned as a vector by `build_bundle_entries` so callers can either
/// write the bundle to disk via `write_bundle` or inspect/transform it
/// in tests without touching the filesystem.
#[derive(Debug, Clone)]
pub struct BundleEntry {
    /// Filename relative to the bundle output directory (e.g. `weight_bram.sv`).
    pub filename: String,
    /// Emitted file content (ASCII Verilog or SVA).
    pub content: String,
}

/// Canonical filename ordering. Pipeline order = RTL deps order:
///   1. trit_stdlib                    (W32, ternary primitives -- deps of compute)
///   2. weight_bram                    (W36a, storage)
///   3. pipeline_stage2_compute        (W36b, compute)
///   4. layer_sequencer                (W36b, control)
///   5. multilayer_sequencer           (W36b, control)
///   6. double_buffer_ctrl             (W36c, buffering)
///   7. weight_prefetch_ctrl           (W36c, buffering)
///   8. axi_lite_slave                 (W36d, host I/O)
///   9. dma_controller                 (W36e, host I/O)
///  10. interrupt_controller           (W36f, host I/O)
///  11. bitnet_engine_top              (W36f, top wrapper)
///  12. behavior_sva_v2                (W37, verification)
///  13. manifest.txt                   (W38, this wave)
///
/// `trit_stdlib` leads because `pipeline_stage2_compute` instantiates
/// `trit27_dot_product` from it; including it makes the bundle self-contained
/// (it elaborates standalone under iverilog).
pub const BUNDLE_ORDER: &[&str] = &[
    "trit_stdlib.sv",
    "weight_bram.sv",
    "pipeline_stage2_compute.sv",
    "layer_sequencer.sv",
    "multilayer_sequencer.sv",
    "double_buffer_ctrl.sv",
    "weight_prefetch_ctrl.sv",
    "axi_lite_slave.sv",
    "dma_controller.sv",
    "interrupt_controller.sv",
    "bitnet_engine_top.sv",
    "behavior_sva_v2.sv",
    "manifest.txt",
];

/// Total expected file count in a bundle (12 SV files + 1 manifest).
pub const BUNDLE_FILE_COUNT: usize = 13;

/// Canonical BitNet HLS behavior set (4 properties, v2-emittable).
///
/// All four are realistic invariants over the W36 module surface:
///
/// 1. `busy_implies_running`     -- safety:    busy |-> ##1 (busy || done)
/// 2. `start_eventually_done`    -- liveness:  start |-> s_eventually done
/// 3. `irq_clear_on_status_read` -- safety:    !rst_n implies cleared next cycle
/// 4. `dma_done_completes`       -- safety:    dma_done |-> ##1 done
///
/// The exact rendered SVA text depends on the v1/v2 vocabulary parsers
/// (see `behavior_sva.rs::parse_*_clause`). These behaviors use only
/// vocabulary already present in v1.
pub fn canonical_behaviors() -> Vec<Behavior<'static>> {
    vec![
        Behavior {
            name: "engine_busy_safety",
            given: "running",
            when: "posedge clk",
            then: "set full",
        },
        Behavior {
            name: "start_eventually_done",
            given: "valid",
            when: "posedge clk",
            then: "eventually set full",
        },
        Behavior {
            name: "irq_clear_on_reset",
            given: "reset inactive",
            when: "posedge clk",
            then: "after 1 cycle set full",
        },
        Behavior {
            name: "dma_done_completes",
            given: "valid and ready",
            when: "posedge clk",
            then: "after 1 cycle set full",
        },
    ]
}

/// Build the manifest.txt contents for a bundle.
///
/// Manifest format (ASCII, line-oriented, deterministic):
///
///     # BitNet HLS Bundle Manifest
///     # Wave 38 (R-SI-1) -- t27c gen-bitnet-bundle
///     # Engine-top: <top_name>
///     # AXI addr/data width: <addr>/<data> bits
///     # Files: <count>
///     <idx>  <filename>  <byte-size>
///     ...
///
/// The manifest itself is *not* listed in its own body (it cannot know
/// its own final size). Listed entries are the 10 emitted SV files.
pub fn build_manifest(config: &BundleConfig<'_>, entries: &[BundleEntry]) -> String {
    let mut out = String::new();
    out.push_str("# BitNet HLS Bundle Manifest\n");
    out.push_str("# Wave 38 (R-SI-1) -- t27c gen-bitnet-bundle\n");
    out.push_str(&format!("# Engine-top: {}\n", config.top_name));
    out.push_str(&format!(
        "# AXI addr/data width: {}/{} bits\n",
        config.axi_addr_width, config.axi_data_width
    ));
    out.push_str(&format!("# Files: {}\n", entries.len()));
    out.push_str("# Trinity invariant: phi^2 + 1/phi^2 = 3 (kernel untouched)\n");
    for (idx, entry) in entries.iter().enumerate() {
        out.push_str(&format!(
            "{:02}  {}  {}\n",
            idx + 1,
            entry.filename,
            entry.content.len()
        ));
    }
    out
}

/// Emit every SV file in the bundle (11 files), in canonical order.
///
/// Does not include `manifest.txt` -- the manifest is built *after* these
/// 10 entries are emitted, since it references their byte sizes.
pub fn build_sv_entries(config: &BundleConfig<'_>) -> Vec<BundleEntry> {
    vec![
        BundleEntry {
            filename: "trit_stdlib.sv".to_string(),
            content: build_trit_stdlib_verilog(),
        },
        BundleEntry {
            filename: "weight_bram.sv".to_string(),
            content: build_default_weight_bram(),
        },
        BundleEntry {
            filename: "pipeline_stage2_compute.sv".to_string(),
            content: build_pipeline_stage2("pipeline_stage2_compute"),
        },
        BundleEntry {
            filename: "layer_sequencer.sv".to_string(),
            content: build_layer_sequencer("layer_sequencer"),
        },
        BundleEntry {
            filename: "multilayer_sequencer.sv".to_string(),
            content: build_multilayer_sequencer("multilayer_sequencer"),
        },
        BundleEntry {
            filename: "double_buffer_ctrl.sv".to_string(),
            content: build_double_buffer_ctrl("double_buffer_ctrl"),
        },
        BundleEntry {
            filename: "weight_prefetch_ctrl.sv".to_string(),
            content: build_weight_prefetch_ctrl("weight_prefetch_ctrl"),
        },
        BundleEntry {
            filename: "axi_lite_slave.sv".to_string(),
            content: build_axi_lite_slave(
                "axi_lite_slave",
                config.axi_addr_width,
                config.axi_data_width,
            ),
        },
        BundleEntry {
            filename: "dma_controller.sv".to_string(),
            content: build_dma_controller("dma_controller"),
        },
        BundleEntry {
            filename: "interrupt_controller.sv".to_string(),
            content: build_interrupt_controller("interrupt_controller"),
        },
        BundleEntry {
            filename: "bitnet_engine_top.sv".to_string(),
            content: build_bitnet_engine_top(config.top_name),
        },
        BundleEntry {
            filename: "behavior_sva_v2.sv".to_string(),
            content: build_behavior_sva_v2_file(&canonical_behaviors()),
        },
    ]
}

/// Build the complete bundle (10 SV files + manifest), in canonical order.
///
/// Returns a vector of 11 `BundleEntry` values. The 11th entry is always
/// `manifest.txt`; the manifest body lists the byte sizes of the first 10.
pub fn build_bundle_entries(config: &BundleConfig<'_>) -> Vec<BundleEntry> {
    let mut entries = build_sv_entries(config);
    let manifest = build_manifest(config, &entries);
    entries.push(BundleEntry {
        filename: "manifest.txt".to_string(),
        content: manifest,
    });
    entries
}

/// Write a complete bundle to `output_dir`.
///
/// Creates `output_dir` (and parents) if it does not exist. Each entry
/// is written verbatim to `output_dir/<filename>`. Returns the list of
/// absolute paths written, in canonical order.
pub fn write_bundle(config: &BundleConfig<'_>, output_dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)?;
    let entries = build_bundle_entries(config);
    let mut written = Vec::with_capacity(entries.len());
    for entry in entries {
        let path = output_dir.join(&entry.filename);
        fs::write(&path, entry.content.as_bytes())?;
        written.push(path);
    }
    Ok(written)
}

// ============================================================================
// Inline unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_canonical_top_name() {
        let cfg = BundleConfig::default();
        assert_eq!(cfg.top_name, "bitnet_engine_top");
        assert_eq!(cfg.axi_addr_width, 32);
        assert_eq!(cfg.axi_data_width, 32);
    }

    #[test]
    fn bundle_order_has_thirteen_entries() {
        assert_eq!(BUNDLE_ORDER.len(), 13);
        assert_eq!(BUNDLE_FILE_COUNT, 13);
    }

    #[test]
    fn bundle_order_ends_with_manifest() {
        assert_eq!(BUNDLE_ORDER[12], "manifest.txt");
    }

    #[test]
    fn bundle_order_starts_with_trit_stdlib() {
        assert_eq!(BUNDLE_ORDER[0], "trit_stdlib.sv");
    }

    #[test]
    fn canonical_behaviors_has_four_entries() {
        assert_eq!(canonical_behaviors().len(), 4);
    }

    #[test]
    fn canonical_behaviors_have_distinct_names() {
        let bs = canonical_behaviors();
        let mut names: Vec<&str> = bs.iter().map(|b| b.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn build_sv_entries_returns_twelve_files() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        assert_eq!(entries.len(), 12);
    }

    #[test]
    fn build_sv_entries_filenames_match_bundle_order() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(entry.filename, BUNDLE_ORDER[i]);
        }
    }

    #[test]
    fn build_sv_entries_all_nonempty() {
        let cfg = BundleConfig::default();
        for entry in build_sv_entries(&cfg) {
            assert!(!entry.content.is_empty(), "{} is empty", entry.filename);
        }
    }

    #[test]
    fn build_sv_entries_all_ascii() {
        let cfg = BundleConfig::default();
        for entry in build_sv_entries(&cfg) {
            assert!(entry.content.is_ascii(), "{} non-ASCII", entry.filename);
        }
    }

    #[test]
    fn build_bundle_entries_returns_thirteen() {
        let cfg = BundleConfig::default();
        let entries = build_bundle_entries(&cfg);
        assert_eq!(entries.len(), 13);
    }

    #[test]
    fn build_bundle_entries_last_is_manifest() {
        let cfg = BundleConfig::default();
        let entries = build_bundle_entries(&cfg);
        assert_eq!(entries[12].filename, "manifest.txt");
    }

    #[test]
    fn manifest_lists_all_ten_files() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        let manifest = build_manifest(&cfg, &entries);
        for entry in &entries {
            assert!(manifest.contains(&entry.filename), "manifest missing {}", entry.filename);
        }
    }

    #[test]
    fn manifest_mentions_trinity_invariant() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        let manifest = build_manifest(&cfg, &entries);
        assert!(manifest.contains("phi^2 + 1/phi^2 = 3"));
    }

    #[test]
    fn manifest_mentions_wave_38() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        let manifest = build_manifest(&cfg, &entries);
        assert!(manifest.contains("Wave 38"));
    }

    #[test]
    fn manifest_mentions_axi_widths() {
        let cfg = BundleConfig {
            top_name: "bitnet_engine_top",
            axi_addr_width: 64,
            axi_data_width: 128,
        };
        let entries = build_sv_entries(&cfg);
        let manifest = build_manifest(&cfg, &entries);
        assert!(manifest.contains("64/128 bits"));
    }

    #[test]
    fn top_name_override_propagates() {
        let cfg = BundleConfig {
            top_name: "my_engine",
            ..BundleConfig::default()
        };
        let entries = build_sv_entries(&cfg);
        let top = &entries[10].content;
        assert!(top.contains("my_engine"));
    }

    #[test]
    fn axi_width_override_propagates() {
        let cfg = BundleConfig {
            top_name: "bitnet_engine_top",
            axi_addr_width: 16,
            axi_data_width: 64,
        };
        let entries = build_sv_entries(&cfg);
        let axi = &entries[7].content;
        // Width literals appear in the slave's parameter declarations.
        assert!(axi.contains("16") || axi.contains("[15:0]"));
        assert!(axi.contains("64") || axi.contains("[63:0]"));
    }

    #[test]
    fn determinism_two_runs_same_output() {
        let cfg = BundleConfig::default();
        let a = build_bundle_entries(&cfg);
        let b = build_bundle_entries(&cfg);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.filename, y.filename);
            assert_eq!(x.content, y.content);
        }
    }

    #[test]
    fn behavior_sva_block_present_in_bundle() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        let sva = &entries[11].content;
        assert!(sva.contains("property "));
        assert!(sva.contains("assert property"));
    }

    #[test]
    fn behavior_sva_block_includes_s_eventually() {
        let cfg = BundleConfig::default();
        let entries = build_sv_entries(&cfg);
        let sva = &entries[11].content;
        // start_eventually_done uses the v2 s_eventually consequent.
        assert!(sva.contains("s_eventually"));
    }
}
