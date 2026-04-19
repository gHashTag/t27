//! Trinity Weight Initialization

pub struct TrinityInitConfig {
    pub gauge_std: f32,
    pub higgs_std: f32,
    pub lepton_std: f32,
}

impl Default for TrinityInitConfig {
    fn default() -> Self {
        Self {
            gauge_std: 0.118034,
            higgs_std: 0.072949,
            lepton_std: 0.045085,
        }
    }
}
