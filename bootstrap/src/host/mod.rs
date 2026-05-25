// ============================================================================
// Host-side driver module (Wave 39, R-HS-1, Closes #784)
// ============================================================================

pub mod csr_map;
pub mod driver;
pub mod ema;
pub mod irq;
pub mod mmio;

pub use driver::{BitnetDriver, CsrSnapshot, DriverError};
pub use irq::{IrqCallback, IrqCounters, IrqDrivenDriver, IrqHandler, IrqSource, ServiceReport};
pub use mmio::{MmioOp, MmioRecord, MockMmio};
pub use ema::Ema;
