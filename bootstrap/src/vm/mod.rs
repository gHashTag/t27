//! VM Module — Trinity VM Interface
//!
//! Public API for Trinity VM execution.
//! Part of Zig migration (Parity Target 1: VM Core Layer).
//!
//! This module provides a façade over both:
//! - Trinity VM (to be implemented in R003)
//! - Zig VM (reference implementation via FFI)
//!
//! Runtime selection: `tri config --runtime {zig|trinity|auto}`

pub mod opcodes;
pub mod memory;
pub mod executor;

// Re-export key types and functions
pub use opcodes::{TribOp, TribHeader, TribSection};
pub use memory::{Stack, Heap, Memory};
pub use executor::{TribImage, ExecutionResult};

/// Runtime mode selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimeMode {
  /// Use Trinity VM (Rust implementation)
  Trinity,

  /// Use Zig VM (reference implementation via FFI)
  Zig,

  /// Auto-select based on availability
  Auto,
}

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
  pub mode: RuntimeMode,
  pub fallback_to_zig: bool,
}

/// Load a .trib file (façade)
///
/// This is the main entry point for .trib bytecode loading.
/// It delegates to either Trinity VM or Zig VM based on runtime configuration.
///
/// ## Usage
/// ```rust
/// use vm::*;
///
/// let config = RuntimeConfig {
///     mode: RuntimeMode::Auto,
///     fallback_to_zig: true,
/// };
///
/// let image = vm::load_trib("model.trib", &config)?;
/// let result = vm::execute_trib(&image, &config)?;
/// println!("Executed {} ops", result.ops_performed);
/// ```
///
/// ## Runtime Selection
///
/// | Mode | Description |
/// |------|-------------|
/// | Trinity | Rust VM implementation (R003) |
/// | Zig | Reference Zig VM (via FFI) |
/// | Auto | Trinity preferred, falls back to Zig |
///
/// ## Parity
///
/// All implementations must conform to `specs/vm_core.t27` interface:
/// - Magic: 0x54524942
/// - Header size: 26 bytes
/// - Opcodes: Must match `specs/vm_ops.t27`
/// - Memory layout: Stack 64KB, Heap 256KB
///
/// See also:
/// - `specs/vm_ops.t27` — VSA operations
/// - `specs/trity_vm_wrapper.t27` — Zig runtime wrapper

pub fn load_trib(file: &str, config: &RuntimeConfig) -> Result<TribImage> {
  // Placeholder: Load .trib file
  // This will delegate to Trinity VM or Zig VM based on config.mode
  // Full implementation in R003

  Err("VM loader not yet implemented (see R003)".into())
}

/// Execute a TribImage (façade)
///
/// Dispatches to Trinity VM or Zig VM based on runtime configuration.
///
/// ## Usage
/// ```rust
/// use vm::*;
///
/// let config = RuntimeConfig {
///     mode: RuntimeMode::Auto,
///     fallback_to_zig: true,
/// };
///
/// let image = vm::load_trib("model.trib", &config)?;
/// let result = vm::execute_trib(&image, &config)?;
/// println!("Executed {} ops", result.ops_performed);
/// ```
pub fn execute_trib(image: &TribImage, config: &RuntimeConfig) -> Result<ExecutionResult> {
  // Placeholder: Execute TribImage
  // This will delegate to Trinity VM or Zig VM based on config.mode
  // Full implementation in R003

  Ok(ExecutionResult {
    ops_performed: 0,
    cycles: 0,
    memory_peak: 0,
    output: "VM executor not yet implemented (see R003)".to_string(),
  })
}

/// Get runtime statistics
pub fn vm_stats() -> String {
  // Placeholder: Return VM statistics
  // Full implementation in R003

  "VM stats not yet implemented (see R003)".to_string()
}
