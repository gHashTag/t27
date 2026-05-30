use super::mmio::MockMmio;
use super::regmap::{CtrlReg, StatusReg, WeightAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Idle,
    Configured,
    WeightsLoaded,
    Running,
    Complete,
    Error,
}

impl std::fmt::Display for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineState::Idle => write!(f, "idle"),
            PipelineState::Configured => write!(f, "configured"),
            PipelineState::WeightsLoaded => write!(f, "weights_loaded"),
            PipelineState::Running => write!(f, "running"),
            PipelineState::Complete => write!(f, "complete"),
            PipelineState::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineError {
    WrongState { from: PipelineState, expected: PipelineState },
    MmioError,
    InferenceTimeout,
    HardwareError,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::WrongState { from, expected } => {
                write!(f, "wrong state: {from}, expected {expected}")
            }
            PipelineError::MmioError => write!(f, "MMIO error"),
            PipelineError::InferenceTimeout => write!(f, "inference timeout"),
            PipelineError::HardwareError => write!(f, "hardware error"),
        }
    }
}

impl std::error::Error for PipelineError {}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub num_layers: u32,
    pub neurons_per_layer: u32,
    pub chunks: u32,
    pub threshold: u32,
    pub weight_addr: u64,
    pub poll_interval_us: u64,
    pub max_polls: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            num_layers: 4,
            neurons_per_layer: 128,
            chunks: 16,
            threshold: 0,
            weight_addr: 0x1000_0000,
            poll_interval_us: 100,
            max_polls: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub cycles: u32,
    pub state: PipelineState,
}

#[derive(Debug, Clone)]
pub struct InferencePipeline {
    mmio: MockMmio,
    config: PipelineConfig,
    state: PipelineState,
    poll_count: u32,
}

impl InferencePipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            mmio: MockMmio::new(),
            config,
            state: PipelineState::Idle,
            poll_count: 0,
        }
    }

    pub fn state(&self) -> PipelineState {
        self.state
    }

    pub fn poll_count(&self) -> u32 {
        self.poll_count
    }

    pub fn configure(&mut self) -> Result<(), PipelineError> {
        if self.state != PipelineState::Idle && self.state != PipelineState::Error {
            return Err(PipelineError::WrongState {
                from: self.state,
                expected: PipelineState::Idle,
            });
        }
        self.mmio.poke(super::csr_map::NUM_LAYERS, self.config.num_layers);
        self.mmio.poke(super::csr_map::NEURONS, self.config.neurons_per_layer);
        self.mmio.poke(super::csr_map::CHUNKS, self.config.chunks);
        self.mmio.poke(super::csr_map::THRESHOLD, self.config.threshold);
        let addr = WeightAddr::new(self.config.weight_addr);
        self.mmio.poke(super::csr_map::WEIGHT_ADDR_LO, addr.lo);
        self.mmio.poke(super::csr_map::WEIGHT_ADDR_HI, addr.hi);
        self.state = PipelineState::Configured;
        Ok(())
    }

    pub fn load_weights(&mut self) -> Result<(), PipelineError> {
        if self.state != PipelineState::Configured {
            return Err(PipelineError::WrongState {
                from: self.state,
                expected: PipelineState::Configured,
            });
        }
        self.state = PipelineState::WeightsLoaded;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), PipelineError> {
        if self.state != PipelineState::WeightsLoaded {
            return Err(PipelineError::WrongState {
                from: self.state,
                expected: PipelineState::WeightsLoaded,
            });
        }
        let ctrl = CtrlReg::new().with_start();
        self.mmio.poke(super::csr_map::CTRL, ctrl.raw());
        self.mmio.poke(super::csr_map::STATUS, 0);
        self.state = PipelineState::Running;
        self.poll_count = 0;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<PipelineState, PipelineError> {
        if self.state != PipelineState::Running {
            return Err(PipelineError::WrongState {
                from: self.state,
                expected: PipelineState::Running,
            });
        }
        self.poll_count += 1;
        if self.poll_count > self.config.max_polls {
            self.state = PipelineState::Error;
            return Err(PipelineError::InferenceTimeout);
        }
        let raw = self.mmio.peek(super::csr_map::STATUS);
        let status = StatusReg::from_raw(raw);
        if status.error() {
            self.state = PipelineState::Error;
            return Err(PipelineError::HardwareError);
        }
        if status.done() {
            self.state = PipelineState::Complete;
        }
        Ok(self.state)
    }

    pub fn wait_complete(&mut self) -> Result<InferenceResult, PipelineError> {
        loop {
            let state = self.poll()?;
            if state == PipelineState::Complete {
                return Ok(InferenceResult {
                    cycles: self.poll_count,
                    state: PipelineState::Complete,
                });
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = PipelineState::Idle;
        self.poll_count = 0;
        self.mmio = MockMmio::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pipeline() -> InferencePipeline {
        InferencePipeline::new(PipelineConfig::default())
    }

    #[test]
    fn new_pipeline_is_idle() {
        let p = make_pipeline();
        assert_eq!(p.state(), PipelineState::Idle);
    }

    #[test]
    fn state_display() {
        assert_eq!(PipelineState::Idle.to_string(), "idle");
        assert_eq!(PipelineState::Running.to_string(), "running");
        assert_eq!(PipelineState::Error.to_string(), "error");
    }

    #[test]
    fn configure_transitions_to_configured() {
        let mut p = make_pipeline();
        p.configure().unwrap();
        assert_eq!(p.state(), PipelineState::Configured);
    }

    #[test]
    fn configure_writes_csrs() {
        let mut p = make_pipeline();
        p.configure().unwrap();
        let mmio = &p.mmio;
        let val = mmio.peek(super::super::csr_map::NUM_LAYERS);
        assert_eq!(val, 4);
    }

    #[test]
    fn load_weights_requires_configured() {
        let mut p = make_pipeline();
        let err = p.load_weights().unwrap_err();
        assert!(matches!(err, PipelineError::WrongState { .. }));
    }

    #[test]
    fn full_flow_success() {
        let mut p = make_pipeline();
        p.configure().unwrap();
        p.load_weights().unwrap();
        p.start().unwrap();
        assert_eq!(p.state(), PipelineState::Running);
        let status = super::super::csr_map::STATUS_DONE_MASK;
        p.mmio.poke(super::super::csr_map::STATUS, status);
        let result = p.wait_complete().unwrap();
        assert_eq!(result.state, PipelineState::Complete);
        assert_eq!(result.cycles, 1);
    }

    #[test]
    fn poll_timeout() {
        let mut p = InferencePipeline::new(PipelineConfig {
            max_polls: 3,
            ..Default::default()
        });
        p.configure().unwrap();
        p.load_weights().unwrap();
        p.start().unwrap();
        let err = p.wait_complete().unwrap_err();
        assert_eq!(err, PipelineError::InferenceTimeout);
        assert_eq!(p.state(), PipelineState::Error);
    }

    #[test]
    fn hardware_error_detected() {
        let mut p = make_pipeline();
        p.configure().unwrap();
        p.load_weights().unwrap();
        p.start().unwrap();
        p.mmio.poke(super::super::csr_map::STATUS, super::super::csr_map::STATUS_ERROR_MASK);
        let err = p.wait_complete().unwrap_err();
        assert_eq!(err, PipelineError::HardwareError);
    }

    #[test]
    fn start_requires_weights_loaded() {
        let mut p = make_pipeline();
        let err = p.start().unwrap_err();
        assert!(matches!(err, PipelineError::WrongState { .. }));
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut p = make_pipeline();
        p.configure().unwrap();
        p.load_weights().unwrap();
        p.start().unwrap();
        p.reset();
        assert_eq!(p.state(), PipelineState::Idle);
        assert_eq!(p.poll_count(), 0);
    }

    #[test]
    fn configure_from_error_state() {
        let mut p = make_pipeline();
        p.state = PipelineState::Error;
        p.configure().unwrap();
        assert_eq!(p.state(), PipelineState::Configured);
    }

    #[test]
    fn poll_not_running() {
        let mut p = make_pipeline();
        let err = p.poll().unwrap_err();
        assert!(matches!(err, PipelineError::WrongState { .. }));
    }

    #[test]
    fn error_display() {
        let e = PipelineError::WrongState {
            from: PipelineState::Idle,
            expected: PipelineState::Running,
        };
        assert!(e.to_string().contains("idle"));
        assert!(e.to_string().contains("running"));
        assert!(PipelineError::InferenceTimeout.to_string().contains("timeout"));
    }

    #[test]
    fn pipeline_config_default() {
        let c = PipelineConfig::default();
        assert_eq!(c.num_layers, 4);
        assert_eq!(c.neurons_per_layer, 128);
        assert_eq!(c.max_polls, 1000);
    }
}
