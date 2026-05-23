// ============================================================================
// tt_profile.rs -- Tiny Tapeout platform profile + manifest conformance
// (Wave 45, R-TT-2, Closes #800)
//
// Second R-TT artifact.  Describes a single PDK target (Sky130 / IHP-SG13G2 /
// GF180MCU) with all parameters needed to decide whether a `TtManifest`
// (W42, R-TT-1) is buildable against that platform.  Loadable from JSON,
// emits a `ConformanceVerdict { ok, reasons[] }` so CI can gate tape-out
// merges with a single boolean check.
//
// Determinism: identical inputs -> byte-identical JSON.  Field declaration
// order is the JSON layout; serde preserves it.
//
// Zero edits outside bootstrap/.  No L2 expansion in this wave -- the
// W42 boundary (.gitmodules + chips/) stays untouched.
// ============================================================================

use serde::{Deserialize, Serialize};

use crate::tt_manifest::TtManifest;

/// Supported PDK targets.  Each variant corresponds to one open-source PDK
/// supported by the Tiny Tapeout shuttles as of 2026Q2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtPlatform {
    #[serde(rename = "sky130")]
    Sky130,
    #[serde(rename = "ihp_sg13g2")]
    IhpSg13g2,
    #[serde(rename = "gf180mcu")]
    Gf180mcu,
}

impl TtPlatform {
    /// Parse a CLI argument: `sky130` | `ihp` | `gf180`.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "sky130" | "sky" => Ok(TtPlatform::Sky130),
            "ihp" | "ihp_sg13g2" | "ihp-sg13g2" | "sg13g2" => Ok(TtPlatform::IhpSg13g2),
            "gf180" | "gf180mcu" | "gf" => Ok(TtPlatform::Gf180mcu),
            other => Err(format!("unknown tt platform: {}", other)),
        }
    }

    /// Stable string slug used in JSON output.
    pub fn slug(&self) -> &'static str {
        match self {
            TtPlatform::Sky130 => "sky130",
            TtPlatform::IhpSg13g2 => "ihp_sg13g2",
            TtPlatform::Gf180mcu => "gf180mcu",
        }
    }
}

/// Profile of a PDK target.  All numeric fields use unsigned integer
/// representations to keep JSON byte-identical across platforms.
///
/// `supply_voltage_v_mvolts` is the supply voltage expressed in MILLIVOLTS
/// (so 1.8V -> 1800) to avoid floating-point determinism issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtPlatformProfile {
    pub platform: TtPlatform,
    pub process_node_nm: u32,
    pub cell_library: String,
    pub max_tile_area_um2: u32,
    pub supply_voltage_mvolts: u32,
    pub target_clock_mhz: u32,
    pub max_modules: u32,
}

impl TtPlatformProfile {
    /// Canonical Sky130 (SkyWater 130 nm) profile.  Numbers derived from
    /// efabless/openlane defaults for a TT-class tile.
    pub fn canonical_sky130() -> Self {
        TtPlatformProfile {
            platform: TtPlatform::Sky130,
            process_node_nm: 130,
            cell_library: "sky130_fd_sc_hd".to_string(),
            max_tile_area_um2: 168_000,
            supply_voltage_mvolts: 1800,
            target_clock_mhz: 50,
            max_modules: 12,
        }
    }

    /// Canonical IHP SG13G2 (130 nm BiCMOS) profile.
    pub fn canonical_ihp() -> Self {
        TtPlatformProfile {
            platform: TtPlatform::IhpSg13g2,
            process_node_nm: 130,
            cell_library: "sg13g2_stdcell".to_string(),
            max_tile_area_um2: 160_000,
            supply_voltage_mvolts: 1200,
            target_clock_mhz: 40,
            max_modules: 10,
        }
    }

    /// Canonical GF180MCU (GlobalFoundries 180 nm MCU) profile.
    pub fn canonical_gf180() -> Self {
        TtPlatformProfile {
            platform: TtPlatform::Gf180mcu,
            process_node_nm: 180,
            cell_library: "gf180mcu_fd_sc_mcu7t5v0".to_string(),
            max_tile_area_um2: 200_000,
            supply_voltage_mvolts: 5000,
            target_clock_mhz: 25,
            max_modules: 9,
        }
    }

    /// Build the canonical profile for a given platform.
    pub fn canonical_for(platform: TtPlatform) -> Self {
        match platform {
            TtPlatform::Sky130 => Self::canonical_sky130(),
            TtPlatform::IhpSg13g2 => Self::canonical_ihp(),
            TtPlatform::Gf180mcu => Self::canonical_gf180(),
        }
    }

    /// Deterministic pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Inverse of `to_json` for round-trip tests.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Conformance check: can the given manifest be built on this profile?
    ///
    /// Current rules (will grow with R-TT-3 / R-TT-4):
    ///   * `manifest.modules.len() <= profile.max_modules`
    ///   * `manifest.axi_widths.data == 32` (the W36d slave is fixed-width)
    ///   * `manifest.axi_widths.addr == 32`
    ///   * `manifest.axi_widths.csr_aperture_bytes == 64`
    ///
    /// On failure each broken rule contributes one human-readable reason
    /// string.  `ok` mirrors `reasons.is_empty()`.
    pub fn check_manifest(&self, m: &TtManifest) -> ConformanceVerdict {
        let mut reasons: Vec<String> = Vec::new();
        let n_mods = m.modules.len() as u32;
        if n_mods > self.max_modules {
            reasons.push(format!(
                "module count {} exceeds platform max {} ({})",
                n_mods,
                self.max_modules,
                self.platform.slug()
            ));
        }
        if m.axi_widths.data != 32 {
            reasons.push(format!("axi data width {} != 32", m.axi_widths.data));
        }
        if m.axi_widths.addr != 32 {
            reasons.push(format!("axi addr width {} != 32", m.axi_widths.addr));
        }
        if m.axi_widths.csr_aperture_bytes != 64 {
            reasons.push(format!(
                "csr aperture {} bytes != 64",
                m.axi_widths.csr_aperture_bytes
            ));
        }
        ConformanceVerdict {
            ok: reasons.is_empty(),
            reasons,
        }
    }
}

/// Verdict of `TtPlatformProfile::check_manifest`.  `ok == reasons.is_empty()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConformanceVerdict {
    pub ok: bool,
    pub reasons: Vec<String>,
}

impl ConformanceVerdict {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tt_manifest::{AxiWidths, TtChip, TtManifest};

    const FAKE_COMMIT: &str = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    const FAKE_TIME: &str = "2026-05-23T20:00:00Z";

    fn canonical_manifest(chip: TtChip) -> TtManifest {
        TtManifest::new(
            FAKE_COMMIT,
            chip,
            TtManifest::canonical_modules(),
            AxiWidths::canonical(),
            17,
            FAKE_TIME,
        )
    }

    #[test]
    fn platform_from_str_sky130_variants() {
        assert_eq!(TtPlatform::from_str("sky130").unwrap(), TtPlatform::Sky130);
        assert_eq!(TtPlatform::from_str("SKY").unwrap(), TtPlatform::Sky130);
        assert_eq!(TtPlatform::from_str("sky").unwrap(), TtPlatform::Sky130);
    }

    #[test]
    fn platform_from_str_ihp_variants() {
        assert_eq!(TtPlatform::from_str("ihp").unwrap(), TtPlatform::IhpSg13g2);
        assert_eq!(TtPlatform::from_str("IHP_SG13G2").unwrap(), TtPlatform::IhpSg13g2);
        assert_eq!(TtPlatform::from_str("sg13g2").unwrap(), TtPlatform::IhpSg13g2);
    }

    #[test]
    fn platform_from_str_gf180_variants() {
        assert_eq!(TtPlatform::from_str("gf180").unwrap(), TtPlatform::Gf180mcu);
        assert_eq!(TtPlatform::from_str("GF180MCU").unwrap(), TtPlatform::Gf180mcu);
        assert_eq!(TtPlatform::from_str("gf").unwrap(), TtPlatform::Gf180mcu);
    }

    #[test]
    fn platform_from_str_unknown_fails() {
        assert!(TtPlatform::from_str("tsmc7").is_err());
        assert!(TtPlatform::from_str("").is_err());
    }

    #[test]
    fn platform_slug_stable() {
        assert_eq!(TtPlatform::Sky130.slug(), "sky130");
        assert_eq!(TtPlatform::IhpSg13g2.slug(), "ihp_sg13g2");
        assert_eq!(TtPlatform::Gf180mcu.slug(), "gf180mcu");
    }

    #[test]
    fn canonical_sky130_numbers() {
        let p = TtPlatformProfile::canonical_sky130();
        assert_eq!(p.process_node_nm, 130);
        assert_eq!(p.cell_library, "sky130_fd_sc_hd");
        assert_eq!(p.supply_voltage_mvolts, 1800);
        assert_eq!(p.target_clock_mhz, 50);
        assert_eq!(p.max_modules, 12);
    }

    #[test]
    fn canonical_ihp_numbers() {
        let p = TtPlatformProfile::canonical_ihp();
        assert_eq!(p.process_node_nm, 130);
        assert_eq!(p.cell_library, "sg13g2_stdcell");
        assert_eq!(p.supply_voltage_mvolts, 1200);
        assert_eq!(p.target_clock_mhz, 40);
        assert_eq!(p.max_modules, 10);
    }

    #[test]
    fn canonical_gf180_numbers() {
        let p = TtPlatformProfile::canonical_gf180();
        assert_eq!(p.process_node_nm, 180);
        assert_eq!(p.cell_library, "gf180mcu_fd_sc_mcu7t5v0");
        assert_eq!(p.supply_voltage_mvolts, 5000);
        assert_eq!(p.target_clock_mhz, 25);
        assert_eq!(p.max_modules, 9);
    }

    #[test]
    fn canonical_for_dispatches_correctly() {
        assert_eq!(
            TtPlatformProfile::canonical_for(TtPlatform::Sky130),
            TtPlatformProfile::canonical_sky130()
        );
        assert_eq!(
            TtPlatformProfile::canonical_for(TtPlatform::IhpSg13g2),
            TtPlatformProfile::canonical_ihp()
        );
        assert_eq!(
            TtPlatformProfile::canonical_for(TtPlatform::Gf180mcu),
            TtPlatformProfile::canonical_gf180()
        );
    }

    #[test]
    fn profile_json_round_trip_sky() {
        let p = TtPlatformProfile::canonical_sky130();
        let j = p.to_json().unwrap();
        let back = TtPlatformProfile::from_json(&j).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn profile_json_deterministic() {
        let a = TtPlatformProfile::canonical_sky130().to_json().unwrap();
        let b = TtPlatformProfile::canonical_sky130().to_json().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn profile_json_contains_platform_slug() {
        let j = TtPlatformProfile::canonical_ihp().to_json().unwrap();
        assert!(j.contains("\"platform\": \"ihp_sg13g2\""), "json: {}", j);
    }

    #[test]
    fn three_profiles_distinct_json() {
        let s = TtPlatformProfile::canonical_sky130().to_json().unwrap();
        let i = TtPlatformProfile::canonical_ihp().to_json().unwrap();
        let g = TtPlatformProfile::canonical_gf180().to_json().unwrap();
        assert_ne!(s, i);
        assert_ne!(i, g);
        assert_ne!(s, g);
    }

    #[test]
    fn conformance_ok_for_canonical_manifest_on_sky() {
        let v = TtPlatformProfile::canonical_sky130()
            .check_manifest(&canonical_manifest(TtChip::Phi));
        assert!(v.ok, "verdict: {:?}", v);
        assert!(v.reasons.is_empty());
    }

    #[test]
    fn conformance_ok_for_canonical_manifest_on_ihp() {
        let v = TtPlatformProfile::canonical_ihp()
            .check_manifest(&canonical_manifest(TtChip::Euler));
        assert!(v.ok);
    }

    #[test]
    fn conformance_ok_for_canonical_manifest_on_gf180() {
        let v = TtPlatformProfile::canonical_gf180()
            .check_manifest(&canonical_manifest(TtChip::Gamma));
        assert!(v.ok);
    }

    #[test]
    fn conformance_rejects_too_many_modules() {
        // GF180 has max_modules=9, canonical manifest has 9 -- artificially add one.
        let mut m = canonical_manifest(TtChip::Gamma);
        m.modules.push("extra_one".to_string());
        m.modules.push("extra_two".to_string());
        let v = TtPlatformProfile::canonical_gf180().check_manifest(&m);
        assert!(!v.ok);
        assert!(v.reasons.iter().any(|r| r.contains("module count")));
    }

    #[test]
    fn conformance_rejects_wrong_data_width() {
        let mut m = canonical_manifest(TtChip::Phi);
        m.axi_widths.data = 64;
        let v = TtPlatformProfile::canonical_sky130().check_manifest(&m);
        assert!(!v.ok);
        assert!(v.reasons.iter().any(|r| r.contains("axi data width")));
    }

    #[test]
    fn conformance_rejects_wrong_addr_width() {
        let mut m = canonical_manifest(TtChip::Phi);
        m.axi_widths.addr = 24;
        let v = TtPlatformProfile::canonical_sky130().check_manifest(&m);
        assert!(!v.ok);
        assert!(v.reasons.iter().any(|r| r.contains("axi addr width")));
    }

    #[test]
    fn conformance_rejects_wrong_csr_aperture() {
        let mut m = canonical_manifest(TtChip::Phi);
        m.axi_widths.csr_aperture_bytes = 128;
        let v = TtPlatformProfile::canonical_sky130().check_manifest(&m);
        assert!(!v.ok);
        assert!(v.reasons.iter().any(|r| r.contains("csr aperture")));
    }

    #[test]
    fn conformance_accumulates_multiple_reasons() {
        let mut m = canonical_manifest(TtChip::Phi);
        m.axi_widths.data = 64;
        m.axi_widths.addr = 16;
        m.axi_widths.csr_aperture_bytes = 256;
        let v = TtPlatformProfile::canonical_sky130().check_manifest(&m);
        assert!(!v.ok);
        assert_eq!(v.reasons.len(), 3);
    }

    #[test]
    fn verdict_ok_mirrors_reasons_empty() {
        let v_ok = ConformanceVerdict { ok: true, reasons: vec![] };
        assert!(v_ok.ok && v_ok.reasons.is_empty());
        let v_bad = ConformanceVerdict { ok: false, reasons: vec!["x".to_string()] };
        assert!(!v_bad.ok && !v_bad.reasons.is_empty());
    }

    #[test]
    fn verdict_json_contains_ok_field() {
        let v = ConformanceVerdict { ok: true, reasons: vec![] };
        let j = v.to_json().unwrap();
        assert!(j.contains("\"ok\": true"), "json: {}", j);
    }

    #[test]
    fn profile_supply_voltage_is_millivolts() {
        // Sanity: 1800mV = 1.8V Sky-typical
        assert_eq!(TtPlatformProfile::canonical_sky130().supply_voltage_mvolts, 1800);
        // 5000mV = 5.0V GF180-MCU typical
        assert_eq!(TtPlatformProfile::canonical_gf180().supply_voltage_mvolts, 5000);
    }
}
