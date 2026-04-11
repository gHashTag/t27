//! Sacred Attention — Phase 3 (HSLM)
//!
//! All code is generated from .t27 specifications via t27c gen.

/// Placeholder for HSLM integration
/// Once hslm.t27 is written, this will call generated code
pub const HSLM_READY: bool = false;

/// Check if HSLM spec is available
pub fn is_hslm_available() -> bool {
    // Check if generated code exists
    std::path::Path::new("gen/rust/nn/hslm.rs").exists()
}
