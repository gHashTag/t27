#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    InvalidLayers { have: u32, min: u32, max: u32 },
    InvalidNeurons { have: u32, min: u32, max: u32 },
    InvalidChunks { have: u32, min: u32 },
    InvalidThreshold { have: i32, min: i32, max: i32 },
    InvalidAddress { addr: u64 },
    ZeroTimeout,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidLayers { have, min, max } => {
                write!(f, "invalid layers: {have} not in [{min},{max}]")
            }
            ConfigError::InvalidNeurons { have, min, max } => {
                write!(f, "invalid neurons: {have} not in [{min},{max}]")
            }
            ConfigError::InvalidChunks { have, min } => {
                write!(f, "invalid chunks: {have} < {min}")
            }
            ConfigError::InvalidThreshold { have, min, max } => {
                write!(f, "invalid threshold: {have} not in [{min},{max}]")
            }
            ConfigError::InvalidAddress { addr } => {
                write!(f, "invalid weight address: 0x{addr:X}")
            }
            ConfigError::ZeroTimeout => write!(f, "timeout must be > 0"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub const MIN_LAYERS: u32 = 1;
pub const MAX_LAYERS: u32 = 256;
pub const MIN_NEURONS: u32 = 1;
pub const MAX_NEURONS: u32 = 65536;
pub const MIN_CHUNKS: u32 = 1;
pub const MIN_THRESHOLD: i32 = -32768;
pub const MAX_THRESHOLD: i32 = 32767;
pub const DEFAULT_POLL_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_POLL_INTERVAL_US: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    pub num_layers: u32,
    pub neurons_per_layer: u32,
    pub chunks: u32,
    pub threshold: i32,
    pub weight_addr: u64,
    pub poll_timeout_ms: u64,
    pub poll_interval_us: u64,
    pub max_retries: u8,
    pub watchdog_timeout_ms: u64,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            num_layers: 4,
            neurons_per_layer: 128,
            chunks: 16,
            threshold: 0,
            weight_addr: 0x1000_0000,
            poll_timeout_ms: DEFAULT_POLL_TIMEOUT_MS,
            poll_interval_us: DEFAULT_POLL_INTERVAL_US,
            max_retries: 3,
            watchdog_timeout_ms: 10000,
        }
    }
}

impl HostConfig {
    pub fn builder() -> HostConfigBuilder {
        HostConfigBuilder::new()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.num_layers < MIN_LAYERS || self.num_layers > MAX_LAYERS {
            return Err(ConfigError::InvalidLayers {
                have: self.num_layers,
                min: MIN_LAYERS,
                max: MAX_LAYERS,
            });
        }
        if self.neurons_per_layer < MIN_NEURONS || self.neurons_per_layer > MAX_NEURONS {
            return Err(ConfigError::InvalidNeurons {
                have: self.neurons_per_layer,
                min: MIN_NEURONS,
                max: MAX_NEURONS,
            });
        }
        if self.chunks < MIN_CHUNKS {
            return Err(ConfigError::InvalidChunks {
                have: self.chunks,
                min: MIN_CHUNKS,
            });
        }
        if self.threshold < MIN_THRESHOLD || self.threshold > MAX_THRESHOLD {
            return Err(ConfigError::InvalidThreshold {
                have: self.threshold,
                min: MIN_THRESHOLD,
                max: MAX_THRESHOLD,
            });
        }
        if self.weight_addr == 0 {
            return Err(ConfigError::InvalidAddress { addr: 0 });
        }
        if self.poll_timeout_ms == 0 {
            return Err(ConfigError::ZeroTimeout);
        }
        Ok(())
    }

    pub fn total_weights(&self) -> u64 {
        self.num_layers as u64 * self.neurons_per_layer as u64 * self.chunks as u64
    }
}

#[derive(Debug, Clone)]
pub struct HostConfigBuilder {
    config: HostConfig,
}

impl HostConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: HostConfig::default(),
        }
    }

    pub fn layers(mut self, n: u32) -> Self {
        self.config.num_layers = n;
        self
    }

    pub fn neurons(mut self, n: u32) -> Self {
        self.config.neurons_per_layer = n;
        self
    }

    pub fn chunks(mut self, n: u32) -> Self {
        self.config.chunks = n;
        self
    }

    pub fn threshold(mut self, t: i32) -> Self {
        self.config.threshold = t;
        self
    }

    pub fn weight_addr(mut self, addr: u64) -> Self {
        self.config.weight_addr = addr;
        self
    }

    pub fn poll_timeout_ms(mut self, ms: u64) -> Self {
        self.config.poll_timeout_ms = ms;
        self
    }

    pub fn poll_interval_us(mut self, us: u64) -> Self {
        self.config.poll_interval_us = us;
        self
    }

    pub fn max_retries(mut self, n: u8) -> Self {
        self.config.max_retries = n;
        self
    }

    pub fn watchdog_timeout_ms(mut self, ms: u64) -> Self {
        self.config.watchdog_timeout_ms = ms;
        self
    }

    pub fn build(self) -> Result<HostConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for HostConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        HostConfig::default().validate().unwrap();
    }

    #[test]
    fn builder_default_valid() {
        HostConfig::builder().build().unwrap();
    }

    #[test]
    fn builder_custom() {
        let c = HostConfig::builder()
            .layers(8)
            .neurons(256)
            .chunks(32)
            .threshold(10)
            .weight_addr(0x2000_0000)
            .poll_timeout_ms(1000)
            .max_retries(5)
            .build()
            .unwrap();
        assert_eq!(c.num_layers, 8);
        assert_eq!(c.neurons_per_layer, 256);
        assert_eq!(c.chunks, 32);
        assert_eq!(c.threshold, 10);
        assert_eq!(c.max_retries, 5);
    }

    #[test]
    fn validate_zero_layers() {
        let mut c = HostConfig::default();
        c.num_layers = 0;
        let err = c.validate().unwrap_err();
        assert!(matches!(err, ConfigError::InvalidLayers { .. }));
    }

    #[test]
    fn validate_too_many_layers() {
        let mut c = HostConfig::default();
        c.num_layers = MAX_LAYERS + 1;
        assert!(matches!(c.validate(), Err(ConfigError::InvalidLayers { .. })));
    }

    #[test]
    fn validate_zero_neurons() {
        let mut c = HostConfig::default();
        c.neurons_per_layer = 0;
        assert!(matches!(c.validate(), Err(ConfigError::InvalidNeurons { .. })));
    }

    #[test]
    fn validate_zero_chunks() {
        let mut c = HostConfig::default();
        c.chunks = 0;
        assert!(matches!(c.validate(), Err(ConfigError::InvalidChunks { .. })));
    }

    #[test]
    fn validate_threshold_out_of_range() {
        let mut c = HostConfig::default();
        c.threshold = MAX_THRESHOLD + 1;
        assert!(matches!(c.validate(), Err(ConfigError::InvalidThreshold { .. })));
    }

    #[test]
    fn validate_zero_address() {
        let mut c = HostConfig::default();
        c.weight_addr = 0;
        assert!(matches!(c.validate(), Err(ConfigError::InvalidAddress { .. })));
    }

    #[test]
    fn validate_zero_timeout() {
        let mut c = HostConfig::default();
        c.poll_timeout_ms = 0;
        assert!(matches!(c.validate(), Err(ConfigError::ZeroTimeout)));
    }

    #[test]
    fn total_weights() {
        let c = HostConfig::builder().layers(2).neurons(64).chunks(4).build().unwrap();
        assert_eq!(c.total_weights(), 512);
    }

    #[test]
    fn builder_rejects_invalid() {
        let res = HostConfig::builder().layers(0).build();
        assert!(res.is_err());
    }

    #[test]
    fn error_display() {
        let e = ConfigError::InvalidLayers { have: 0, min: 1, max: 256 };
        assert!(e.to_string().contains("0"));
        assert!(ConfigError::ZeroTimeout.to_string().contains("timeout"));
    }
}
