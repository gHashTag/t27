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
pub mod checksum;
pub mod cmdq;
pub mod config;
pub mod csr_map;
pub mod diag;
pub mod dma;
pub mod driver;
pub mod eventlog;
pub mod firmware;
pub mod irq;
pub mod mempool;
pub mod mmio;
pub mod pipeline;
pub mod protocol;
pub mod regmap;
pub mod retry;
pub mod scatter_gather;
pub mod session;
pub mod transport;
pub mod watchdog;
pub mod weight_header;
pub mod weight_loader;

pub use bitstream::{BitError, BitstreamReader};
pub use checksum::{checksum, verify, ChecksumError, ChecksumKind, Digest};
pub use cmdq::{CmdKind, CommandQueue, Priority, QueueError, QueueStats};
pub use config::{ConfigError, HostConfig, HostConfigBuilder};
pub use diag::DiagCounters;
pub use dma::{DmaChannel, DmaConfig, DmaError, DmaReport, DmaState};
pub use driver::{BitnetDriver, CsrSnapshot, DriverError};
pub use eventlog::{Event, EventKind, EventLog, EventLogStats};
pub use firmware::{FirmwareHeader, FirmwareImage, ImageError, SectionHeader};
pub use irq::{IrqCallback, IrqCounters, IrqDrivenDriver, IrqHandler, IrqSource, ServiceReport};
pub use mempool::{MemBlock, MemPool, PoolError, PoolStats, BLOCK_SIZE, MAX_BLOCKS};
pub use mmio::{MmioOp, MmioRecord, MockMmio};
pub use pipeline::{InferencePipeline, PipelineConfig, PipelineError, PipelineState};
pub use protocol::{Cmd, CmdPacket, ProtocolError, RespCode, RespPacket, CMD_HEADER_SIZE, RESP_HEADER_SIZE};
pub use regmap::{ConfigSnapshot, CtrlReg, FullSnapshot, IrqEnReg, IrqStatReg, StatusReg, WeightAddr};
pub use retry::{BackoffStrategy, RetryError, RetryPolicy, RetryState};
pub use scatter_gather::{SgDescriptor, SgError, SgSegment};
pub use session::{Session, SessionConfig, SessionError, SessionStats};
pub use transport::{TransportError, TransportFrame};
pub use watchdog::{WatchdogConfig, WatchdogError, WatchdogState, WatchdogStats, WatchdogTimer};
pub use weight_header::{HeaderError, WeightHeader, HEADER_SIZE, MAGIC, VERSION};
pub use weight_loader::{load_from_reader, load_words, encode_words, encode_with_crc, LoadConfig, LoadError, LoadReport, WordFormat};
