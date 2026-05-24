// ============================================================================
// Host-side driver module (Wave 39, R-HS-1, Closes #784)
//
// This module provides a host-side, no-OS Rust driver for the BitNet AXI-Lite
// CSR aperture emitted by Wave 36d.  It is composed of three submodules:
//
//   * `csr_map`  -- byte-offset constants and bit positions for the slave,
//   * `mmio`     -- a minimal `Mmio` trait plus a deterministic `MockMmio`,
//   * `driver`   -- `BitnetDriver<M: Mmio>` orchestrating configure / start /
//                   poll / IRQ flows on top of the trait.
//
// The crate root re-exports the most commonly used types so consumers can
// write `use t27c::host::{BitnetDriver, MockMmio, DriverError};`.
// ============================================================================

pub mod csr_map;
pub mod driver;
pub mod e2e;
pub mod engine;
pub mod irq;
pub mod json_output;
pub mod mmio;
pub mod perf;
pub mod ternary;
pub mod validate;
pub mod weights;

#[allow(unused_imports)]
pub use driver::{BitnetDriver, CsrSnapshot, DriverError};
#[allow(unused_imports)]
pub use e2e::{E2eConfig, E2eResult, E2eJson};
#[allow(unused_imports)]
pub use engine::{InferenceEngine, InferenceReport};
#[allow(unused_imports)]
pub use irq::{IrqDrivenDriver, IrqHandler, IrqSource};
#[allow(unused_imports)]
pub use json_output::{HostSmokeJson, HostPollVsIrqJson, HostInferenceJson, HostPerfJson};
#[allow(unused_imports)]
pub use mmio::{MmioOp, MmioRecord, MockMmio};
#[allow(unused_imports)]
pub use perf::{EngineConfig, PerformanceEstimate};
#[allow(unused_imports)]
pub use ternary::{Trit, pack_word, unpack_word, pack_words, unpack_words, parse_trit_string, format_trits};
#[allow(unused_imports)]
pub use validate::{WeightValidator, ValidationResult, ValidationDiag, ValidationKind, validate_words};
#[allow(unused_imports)]
pub use weights::{WeightPattern, WeightConfig, generate_pattern, generate_weights, pattern_name, parse_pattern};
