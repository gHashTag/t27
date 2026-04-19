//! # zig-agents
//!
//! Rust FFI wrapper for the [zig-agents](https://github.com/gHashTag/zig-agents) library.
//!
//! Provides autonomous agent orchestration via Zig-compiled C-ABI static library.
//! In stub mode (no `ffi` feature), returns placeholder errors.

use anyhow::{Error, Result};

/// FFI function signatures — linked when `ffi` feature is enabled.
#[cfg(feature = "ffi")]
mod ffi {
    use std::os::raw::c_int;

    extern "C" {
        pub fn zig_agents_version() -> *const std::os::raw::c_char;
        pub fn zig_agents_send_message(
            msg: *const u8,
            msg_len: usize,
            timeout_ms: u64,
        ) -> c_int;
        pub fn zig_agents_health_check() -> *const std::os::raw::c_char;
    }
}

/// Agent type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AgentType {
    Phi = 0,
    Vibee = 1,
    Swarm = 2,
    ClaudeFlow = 3,
    AgentMu = 4,
    Pas = 5,
}

/// Message type for inter-agent communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    AnalysisRequest = 0,
    CodegenRequest = 1,
    ConsensusRequest = 2,
    FixProposal = 3,
    FixResult = 4,
    StatusQuery = 5,
    ErrorReport = 6,
    PasAnalysis = 7,
    PasForecast = 8,
    PasValidation = 9,
}

/// Get the zig-agents library version.
pub fn version() -> Result<String> {
    #[cfg(feature = "ffi")]
    {
        let ptr = unsafe { ffi::zig_agents_version() };
        if ptr.is_null() {
            return Err(Error::msg("zig_agents_version returned null"));
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
        Ok(c_str.to_string_lossy().into_owned())
    }

    #[cfg(not(feature = "ffi"))]
    {
        Err(Error::msg("zig-agents FFI not available (compile with --features ffi)"))
    }
}

/// Send a collaboration message to the agent network.
pub fn send_collaboration_message(msg: &str, timeout_ms: u64) -> Result<()> {
    #[cfg(feature = "ffi")]
    {
        let rc = unsafe {
            ffi::zig_agents_send_message(msg.as_ptr(), msg.len(), timeout_ms)
        };
        if rc != 0 {
            return Err(Error::msg(format!("send_message failed with code {}", rc)));
        }
        Ok(())
    }

    #[cfg(not(feature = "ffi"))]
    {
        let _ = (msg, timeout_ms);
        Err(Error::msg("zig-agents FFI not available (compile with --features ffi)"))
    }
}

/// Perform a health check on the agent subsystem.
pub fn health_check() -> Result<String> {
    #[cfg(feature = "ffi")]
    {
        let ptr = unsafe { ffi::zig_agents_health_check() };
        if ptr.is_null() {
            return Err(Error::msg("health_check returned null"));
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
        Ok(c_str.to_string_lossy().into_owned())
    }

    #[cfg(not(feature = "ffi"))]
    {
        Err(Error::msg("zig-agents FFI not available (compile with --features ffi)"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "ffi"))]
    fn stub_version_returns_error() {
        assert!(version().is_err());
    }

    #[test]
    #[cfg(not(feature = "ffi"))]
    fn stub_send_message_returns_error() {
        assert!(send_collaboration_message("test", 1000).is_err());
    }

    #[test]
    #[cfg(not(feature = "ffi"))]
    fn stub_health_check_returns_error() {
        assert!(health_check().is_err());
    }

    #[test]
    fn agent_type_values() {
        assert_eq!(AgentType::Phi as u32, 0);
        assert_eq!(AgentType::Pas as u32, 5);
    }

    #[test]
    fn message_type_values() {
        assert_eq!(MessageType::AnalysisRequest as u32, 0);
        assert_eq!(MessageType::PasValidation as u32, 9);
    }
}
