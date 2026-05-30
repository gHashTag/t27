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

pub mod addrmap;
pub mod bitmask;
pub mod bitstream;
pub mod bufchain;
pub mod capflags;
pub mod checksum;
pub mod cmdq;
pub mod config;
pub mod configreg;
pub mod csr_map;
pub mod descring;
pub mod diag;
pub mod dma;
pub mod driver;
pub mod endpoint;
pub mod errors;
pub mod eventlog;
pub mod firmware;
pub mod health;
pub mod histogram;
pub mod irq;
pub mod irqrouter;
pub mod mempool;
pub mod mmio;
pub mod oplog;
pub mod pipeline;
pub mod protocol;
pub mod ratelimit;
pub mod regcache;
pub mod regmap;
pub mod retry;
pub mod ring_buffer;
pub mod scatter_gather;
pub mod serial;
pub mod session;
pub mod shutdown;
pub mod telemetry;
pub mod transport;
pub mod version;
pub mod watchdog;
pub mod weight_header;
pub mod weight_loader;

pub use addrmap::{AddrMap, AddrError, MemRegion, REGION_BRAM, REGION_CSR, REGION_DDR, REGION_DMA};
pub use bitmask::{clear_bit, extract, field_mask, insert, is_set, mask, popcount, set_bit, toggle_bit, FieldDesc};
pub use bitstream::{BitError, BitstreamReader};
pub use bufchain::{BufChain, BufSegment, ChainError, ChainStats};
pub use capflags::{check_capabilities, CapFlags, CapabilityError, BASE_CAPABILITIES, FULL_CAPABILITIES};
pub use checksum::{checksum, verify, ChecksumError, ChecksumKind, Digest};
pub use cmdq::{CmdKind, CommandQueue, Priority, QueueError, QueueStats};
pub use config::{ConfigError, HostConfig, HostConfigBuilder};
pub use configreg::{ConfigRegistry, ConfigValue, RegistryError};
pub use descring::{Descriptor, DescriptorRing, DescStatus, RingError, RingStats};
pub use diag::DiagCounters;
pub use dma::{DmaChannel, DmaConfig, DmaError, DmaReport, DmaState};
pub use driver::{BitnetDriver, CsrSnapshot, DriverError};
pub use endpoint::{Access, Endpoint, EndpointError, EndpointRegistry};
pub use errors::{by_domain, by_severity, lookup, CatalogEntry, ErrorCode, ErrorDomain, Severity};
pub use eventlog::{Event, EventKind, EventLog, EventLogStats};
pub use firmware::{FirmwareHeader, FirmwareImage, ImageError, SectionHeader};
pub use health::{Health, HealthMonitor, HealthSummary, Subsystem, SubsystemHealth};
pub use histogram::{Histogram, HistogramSummary};
pub use irq::{IrqCallback, IrqCounters, IrqDrivenDriver, IrqHandler, IrqSource, ServiceReport};
pub use irqrouter::{IrqAction, IrqEvent, IrqRouter, IrqSource as RouterIrqSource, RouteError};
pub use mempool::{MemBlock, MemPool, PoolError, PoolStats, BLOCK_SIZE, MAX_BLOCKS};
pub use mmio::{MmioOp, MmioRecord, MockMmio};
pub use oplog::{OpLog, OpLogStats, OpRecord, OpStatus};
pub use pipeline::{InferencePipeline, PipelineConfig, PipelineError, PipelineState};
pub use protocol::{Cmd, CmdPacket, ProtocolError, RespCode, RespPacket, CMD_HEADER_SIZE, RESP_HEADER_SIZE};
pub use ratelimit::{RateLimitConfig, RateLimitError, RateLimiter, RateLimitStats};
pub use regcache::{CacheError, CacheStats, RegisterCache};
pub use regmap::{ConfigSnapshot, CtrlReg, FullSnapshot, IrqEnReg, IrqStatReg, StatusReg, WeightAddr};
pub use retry::{BackoffStrategy, RetryError, RetryPolicy, RetryState};
pub use ring_buffer::{RingBuffer, RingError};
pub use scatter_gather::{SgDescriptor, SgError, SgSegment};
pub use serial::{Deserializer, SerialError, Serializer};
pub use session::{Session, SessionConfig, SessionError, SessionStats};
pub use shutdown::{Phase, ShutdownCoordinator, ShutdownError, ShutdownState};
pub use telemetry::{Metric, MetricKind, MetricValue, TelemetryCollector, TelemetrySnapshot};
pub use transport::{TransportError, TransportFrame};
pub use version::{BuildInfo, SemVer, VersionError, VersionInfo, HOST_VERSION, PROTOCOL_VERSION};
pub use watchdog::{WatchdogConfig, WatchdogError, WatchdogState, WatchdogStats, WatchdogTimer};
pub use weight_header::{HeaderError, WeightHeader, HEADER_SIZE, MAGIC, VERSION};
pub use weight_loader::{load_from_reader, load_words, encode_words, encode_with_crc, LoadConfig, LoadError, LoadReport, WordFormat};
