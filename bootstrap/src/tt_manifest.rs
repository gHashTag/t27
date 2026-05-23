// ============================================================================
// tt_manifest.rs -- Tiny Tapeout manifest builder (Wave 42, R-TT-1, Closes #792)
//
// Emits a deterministic JSON manifest pinning a Tiny Tapeout silicon variant
// (phi / euler / gamma) to a specific t27 commit, trinity-invariant hash,
// AXI parameter set, and SVA count.  The manifest is the first reproducibility
// artifact of the R-TT track and is consumed by downstream chip repos via
// the `chips/{phi,euler,gamma}` git submodules registered in this same wave.
//
// Determinism contract: identical (t27_commit, chip, modules, axi_widths,
// sva_count, build_time_utc) MUST produce byte-identical JSON output.  This
// is achieved by:
//   - using `serde_json::to_string_pretty` with sorted struct fields,
//   - rendering `modules` in declared-input order (caller responsibility),
//   - hashing the trinity invariant from a fixed string literal,
//   - emitting `build_time_utc` as an RFC3339 string passed in by caller
//     (so unit tests can pin it).
//
// Zero edits under gen/, coq/, proofs/, specs/, conformance/, architecture/,
// rings/, root Cargo.toml.  This wave EXPLICITLY EXPANDS L2 to allow
// `.gitmodules` and `chips/` at repo root -- see PR #N body for rationale.
// ============================================================================

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One of the three Tiny Tapeout silicon variants emitted from t27.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TtChip {
    Phi,
    Euler,
    Gamma,
}

impl TtChip {
    /// Parse from a CLI argument like `phi` / `euler` / `gamma`.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "phi" => Ok(TtChip::Phi),
            "euler" => Ok(TtChip::Euler),
            "gamma" => Ok(TtChip::Gamma),
            other => Err(format!("unknown tt chip: {}", other)),
        }
    }

    /// Stable string slug used in JSON and submodule paths.
    pub fn slug(&self) -> &'static str {
        match self {
            TtChip::Phi => "phi",
            TtChip::Euler => "euler",
            TtChip::Gamma => "gamma",
        }
    }

    /// Submodule path under `chips/<slug>` in the t27 repo root.
    pub fn submodule_path(&self) -> String {
        format!("chips/{}", self.slug())
    }
}

/// AXI-Lite parameter set frozen by the W36d slave.  Width values are bit-counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiWidths {
    pub addr: u32,
    pub data: u32,
    pub csr_aperture_bytes: u32,
}

impl AxiWidths {
    /// Canonical widths used by the W36d slave and host driver.
    pub fn canonical() -> Self {
        AxiWidths {
            addr: 32,
            data: 32,
            csr_aperture_bytes: 0x40,
        }
    }
}

/// Tiny Tapeout manifest tying a t27 commit to a chip variant.
///
/// Field order MUST be stable -- it determines JSON layout.  All fields are
/// `pub` so external builders (e.g. a CI lockfile generator in W45) can
/// instantiate directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtManifest {
    pub t27_commit: String,
    pub phi_invariant_hash: String,
    pub chip: TtChip,
    pub modules: Vec<String>,
    pub axi_widths: AxiWidths,
    pub sva_count: u32,
    pub build_time_utc: String,
}

impl TtManifest {
    /// Build a new manifest with all fields explicit.  Modules are stored in
    /// the order provided -- callers are expected to pass a stable order.
    pub fn new(
        t27_commit: &str,
        chip: TtChip,
        modules: Vec<String>,
        axi_widths: AxiWidths,
        sva_count: u32,
        build_time_utc: &str,
    ) -> Self {
        TtManifest {
            t27_commit: t27_commit.to_string(),
            phi_invariant_hash: phi_invariant_hash(),
            chip,
            modules,
            axi_widths,
            sva_count,
            build_time_utc: build_time_utc.to_string(),
        }
    }

    /// The canonical 9-module BitNet HLS pipeline as of W42.  Module order is
    /// fixed: weight_bram, pipeline_stage2, layer_sequencer, double_buffer,
    /// weight_prefetch, axi_slave, dma, irq, top.
    pub fn canonical_modules() -> Vec<String> {
        vec![
            "weight_bram".to_string(),
            "pipeline_stage2_compute".to_string(),
            "layer_sequencer".to_string(),
            "double_buffer_ctrl".to_string(),
            "weight_prefetch_ctrl".to_string(),
            "bitnet_axi_slave".to_string(),
            "bitnet_dma".to_string(),
            "bitnet_irq".to_string(),
            "bitnet_engine_top".to_string(),
        ]
    }

    /// Render to deterministic pretty-printed JSON.  Identical inputs MUST
    /// yield identical bytes.  This relies on serde keeping struct-field
    /// declaration order, which it does by default.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON -- inverse of `to_json` for round-trip tests.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// SHA-256 of the canonical trinity-invariant string
/// `phi^2 + 1/phi^2 = 3` (ASCII bytes, no newline).  Computed at runtime;
/// stable across hosts because SHA-256 is deterministic.  Returned as
/// lower-case hex.
pub fn phi_invariant_hash() -> String {
    let mut h = Sha256::new();
    h.update(b"phi^2 + 1/phi^2 = 3");
    let digest = h.finalize();
    let mut out = String::with_capacity(64);
    for b in digest.iter() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const FAKE_TIME: &str = "2026-05-23T19:42:00Z";

    fn manifest_phi() -> TtManifest {
        TtManifest::new(
            FAKE_COMMIT,
            TtChip::Phi,
            TtManifest::canonical_modules(),
            AxiWidths::canonical(),
            17,
            FAKE_TIME,
        )
    }

    #[test]
    fn chip_from_str_lowercase() {
        assert_eq!(TtChip::from_str("phi").unwrap(), TtChip::Phi);
        assert_eq!(TtChip::from_str("euler").unwrap(), TtChip::Euler);
        assert_eq!(TtChip::from_str("gamma").unwrap(), TtChip::Gamma);
    }

    #[test]
    fn chip_from_str_uppercase() {
        assert_eq!(TtChip::from_str("PHI").unwrap(), TtChip::Phi);
        assert_eq!(TtChip::from_str("Euler").unwrap(), TtChip::Euler);
    }

    #[test]
    fn chip_from_str_unknown() {
        assert!(TtChip::from_str("delta").is_err());
        assert!(TtChip::from_str("").is_err());
    }

    #[test]
    fn chip_slug_stable() {
        assert_eq!(TtChip::Phi.slug(), "phi");
        assert_eq!(TtChip::Euler.slug(), "euler");
        assert_eq!(TtChip::Gamma.slug(), "gamma");
    }

    #[test]
    fn chip_submodule_path() {
        assert_eq!(TtChip::Phi.submodule_path(), "chips/phi");
        assert_eq!(TtChip::Euler.submodule_path(), "chips/euler");
        assert_eq!(TtChip::Gamma.submodule_path(), "chips/gamma");
    }

    #[test]
    fn axi_widths_canonical() {
        let w = AxiWidths::canonical();
        assert_eq!(w.addr, 32);
        assert_eq!(w.data, 32);
        assert_eq!(w.csr_aperture_bytes, 0x40);
    }

    #[test]
    fn phi_invariant_hash_is_64_hex_chars() {
        let h = phi_invariant_hash();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn phi_invariant_hash_deterministic() {
        let a = phi_invariant_hash();
        let b = phi_invariant_hash();
        assert_eq!(a, b);
    }

    #[test]
    fn canonical_modules_length_nine() {
        assert_eq!(TtManifest::canonical_modules().len(), 9);
    }

    #[test]
    fn canonical_modules_order_stable() {
        let m = TtManifest::canonical_modules();
        assert_eq!(m[0], "weight_bram");
        assert_eq!(m[8], "bitnet_engine_top");
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let m = manifest_phi();
        let j = m.to_json().unwrap();
        let parsed = TtManifest::from_json(&j).unwrap();
        assert_eq!(m, parsed);
    }

    #[test]
    fn manifest_json_is_deterministic() {
        let a = manifest_phi().to_json().unwrap();
        let b = manifest_phi().to_json().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn manifest_json_contains_chip_slug() {
        let j = manifest_phi().to_json().unwrap();
        assert!(j.contains("\"chip\": \"phi\""), "json was: {}", j);
    }

    #[test]
    fn manifest_json_contains_commit() {
        let j = manifest_phi().to_json().unwrap();
        assert!(j.contains(FAKE_COMMIT));
    }

    #[test]
    fn manifest_json_contains_phi_invariant_hash() {
        let j = manifest_phi().to_json().unwrap();
        assert!(j.contains(&phi_invariant_hash()));
    }

    #[test]
    fn manifest_json_contains_build_time() {
        let j = manifest_phi().to_json().unwrap();
        assert!(j.contains(FAKE_TIME));
    }

    #[test]
    fn manifest_three_chips_yield_different_json() {
        let phi = TtManifest::new(FAKE_COMMIT, TtChip::Phi, vec![], AxiWidths::canonical(), 0, FAKE_TIME).to_json().unwrap();
        let euler = TtManifest::new(FAKE_COMMIT, TtChip::Euler, vec![], AxiWidths::canonical(), 0, FAKE_TIME).to_json().unwrap();
        let gamma = TtManifest::new(FAKE_COMMIT, TtChip::Gamma, vec![], AxiWidths::canonical(), 0, FAKE_TIME).to_json().unwrap();
        assert_ne!(phi, euler);
        assert_ne!(euler, gamma);
        assert_ne!(phi, gamma);
    }

    #[test]
    fn manifest_modules_preserved_in_order() {
        let mods = vec!["b".to_string(), "a".to_string(), "c".to_string()];
        let m = TtManifest::new(FAKE_COMMIT, TtChip::Phi, mods.clone(), AxiWidths::canonical(), 0, FAKE_TIME);
        assert_eq!(m.modules, mods);
    }
}
