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

pub mod bitstream;
pub mod csr_map;
pub mod dma;
pub mod driver;
pub mod irq;
pub mod mmio;
pub mod protocol;
pub mod scatter_gather;
pub mod session;
pub mod transport;
pub mod weight_header;
pub mod weight_loader;

pub use bitstream::{BitError, BitstreamReader};
pub use dma::{DmaChannel, DmaConfig, DmaError, DmaReport, DmaState};
pub use driver::{BitnetDriver, CsrSnapshot, DriverError};
pub use irq::{IrqCallback, IrqCounters, IrqDrivenDriver, IrqHandler, IrqSource, ServiceReport};
pub use mmio::{MmioOp, MmioRecord, MockMmio};
pub use protocol::{Cmd, CmdPacket, ProtocolError, RespCode, RespPacket, CMD_HEADER_SIZE, RESP_HEADER_SIZE};
pub use scatter_gather::{SgDescriptor, SgError, SgSegment};
pub use session::{Session, SessionConfig, SessionError, SessionStats};
pub use transport::{TransportError, TransportFrame};
pub use weight_header::{HeaderError, WeightHeader, HEADER_SIZE, MAGIC, VERSION};
pub use weight_loader::{load_from_reader, load_words, encode_words, encode_with_crc, LoadConfig, LoadError, LoadReport, WordFormat};
