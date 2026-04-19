//! # trios-zig-agents
//!
//! Safe Rust wrapper around [zig-agents](https://github.com/gHashTag/zig-agents),
//! providing MCP server and autonomous agent orchestration.
//!
//! ## Features
//!
//! - **ffi** (default: disabled): Links against zig-agents vendor/ library
//! - **stub** (default: enabled): Provides stub implementations that return errors
//!
//! ## Example
//!
//! ```ignore
//! use trios_zig_agents::{AgentType, CollaborationMessage};
//!
//! let msg = CollaborationMessage {
//!     from: AgentType::agent_mu,
//!     to: AgentType::vibee,
//!     message_type: MessageType::codegen_request,
//!     payload: b"analyze pattern",
//! };
//! ```

#[cfg(feature = "ffi")]
mod ffi;

/// Error returned when FFI is not available.
#[cfg(not(feature = "ffi"))]
#[derive(Debug)]
pub struct FfiNotAvailable;

impl std::fmt::Display for FfiNotAvailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("zig-agents FFI not available. Enable with --features ffi or add zig-agents vendor submodule.")
    }
}

impl std::error::Error for FfiNotAvailable {
    fn description(&self) -> &str {
        "zig-agents functions require zig-agents vendor submodule"
    }
}

#[allow(unused_imports)]
use libc::{c_char, c_int, size_t};

#[cfg(feature = "ffi")]
extern "C" {
    /// Get Trinity version from zig-agents
    pub fn trinity_version() -> *const c_char;

    /// Send a collaboration message between agents
    pub fn trinity_collaboration_send(
        msg_ptr: *const u8,
        msg_len: size_t,
        timeout_ms: size_t,
    ) -> c_int;

    /// Get MCP server health status
    pub fn trinity_health_check() -> c_int;

    /// Deploy MCP server to Fly.io region
    pub fn trinity_deploy_fly(region: *const c_char, org: *const c_char) -> c_int;

    /// Get current instance status
    pub fn trinity_instance_status() -> *const c_char;

    /// Restart an instance
    pub fn trinity_restart_instance() -> c_int;

    /// Stop an instance
    pub fn trinity_stop_instance() -> c_int;

    /// Get TLS certificate status
    pub fn trinity_cert_status() -> *const c_char;

    /// Get agent collaboration status
    pub fn trinity_agent_status(agent_id: *const c_char) -> *const c_char;

    /// Register a new agent
    pub fn trinity_register_agent(
        agent_type: *const c_char,
        config: *const c_char,
    ) -> c_int;

    /// Unregister an agent
    pub fn trinity_unregister_agent(agent_id: *const c_char) -> c_int;

    /// Query agent registry
    pub fn trinity_list_agents() -> *const c_char;

    /// Spawn autonomous task
    pub fn trinity_spawn_task(
        task: *const c_char,
        agent_id: *const c_char,
    ) -> c_int;

    /// Get task status
    pub fn trinity_task_status(task_id: *const c_char) -> *const c_char;

    /// Cancel a task
    pub fn trinity_cancel_task(task_id: *const c_char) -> c_int;

    /// Clean free string returned from zig-agents
    pub fn trinity_free_string(ptr: *mut c_char);
}

/// Agent types supported by zig-agents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Pattern matching specialist
    Phi,
    /// Code generator
    Vibee,
    /// Consensus engine
    Swarm,
    /// MCP bridge
    ClaudeFlow,
    /// Self-reference
    AgentMu,
    /// Predictive Algorithmic Systematics
    Pas,
}

/// Message types for inter-agent communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    AnalysisRequest,
    CodegenRequest,
    ConsensusRequest,
    FixProposal,
    FixResult,
    StatusQuery,
    ErrorReport,
    PasAnalysis,
    PasForecast,
    PasValidation,
}

/// Fly.io deployment regions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlyRegion {
    Amsterdam,
    Paris,
    Frankfurt,
    LosAngeles,
    Chicago,
    Virginia,
    Singapore,
    Tokyo,
    HongKong,
    Sydney,
}

impl FlyRegion {
    pub fn to_region_code(&self) -> &'static str {
        match self {
            Self::Amsterdam => "ams",
            Self::Paris => "cdg",
            Self::Frankfurt => "fra",
            Self::LosAngeles => "lax",
            Self::Chicago => "ord",
            Self::Virginia => "iad",
            Self::Singapore => "sin",
            Self::Tokyo => "nrt",
            Self::HongKong => "hkg",
            Self::Sydney => "syd",
        }
    }

    pub fn to_location_name(&self) -> &'static str {
        match self {
            Self::Amsterdam => "Amsterdam, Netherlands",
            Self::Paris => "Paris, France",
            Self::Frankfurt => "Frankfurt, Germany",
            Self::LosAngeles => "Los Angeles, USA",
            Self::Chicago => "Chicago, USA",
            Self::Virginia => "Virginia, USA",
            Self::Singapore => "Singapore",
            Self::Tokyo => "Tokyo, Japan",
            Self::HongKong => "Hong Kong, Asia",
            Self::Sydney => "Sydney, Australia",
        }
    }
}

/// Instance health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceHealth {
    Healthy,
    Degraded,
    Down,
    Unknown,
}

/// Instance state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Crashed,
    Restarting,
}

/// Instance information
#[derive(Debug, Clone)]
pub struct InstanceStatus {
    pub id: String,
    pub region: FlyRegion,
    pub state: InstanceState,
    pub health: InstanceHealth,
    pub uptime_ns: i64,
    pub last_health_check: i64,
    pub restart_count: u32,
    pub connections: u32,
    pub memory_mb: f64,
    pub cpu_percent: f64,
}

#[cfg(feature = "ffi")]
/// Get Trinity version string
pub fn version() -> String {
    unsafe {
        let ptr = ffi::trinity_version();
        if ptr.is_null() {
            "unknown".into()
        } else {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            c_str.to_string_lossy().unwrap_or_else(|_| "unknown".into())
        }
    }
}

#[cfg(feature = "ffi")]
/// Send a collaboration message to agents
pub fn send_collaboration_message(msg: &str, timeout_ms: u64) -> Result<(), anyhow::Error> {
    let msg_bytes = msg.as_bytes();
    let rc = unsafe {
        ffi::trinity_collaboration_send(
            msg_bytes.as_ptr(),
            msg_bytes.len(),
            timeout_ms as size_t,
        )
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("collaboration send failed with code {}", rc))
    }
}

#[cfg(feature = "ffi")]
/// Get MCP server health status
pub fn health_check() -> Result<String, anyhow::Error> {
    let rc = unsafe { ffi::trinity_health_check() };
    if rc == 0 {
        let ptr = unsafe { ffi::trinity_instance_status() };
        if ptr.is_null() {
            Ok("no instances".into())
        } else {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            Ok(c_str.to_string_lossy().unwrap_or_else(|_| "unknown".into()))
        }
    } else {
        Err(anyhow::anyhow!("health check failed with code {}", rc))
    }
}

#[cfg(feature = "ffi")]
/// Deploy MCP server to Fly.io region
pub fn deploy_to_fly(region: FlyRegion, org: Option<&str>) -> Result<(), anyhow::Error> {
    let region_c = std::ffi::CString::new(region.to_region_code())?;
    let org_c = match org {
        Some(o) => std::ffi::CString::new(o)?,
        None => std::ffi::CString::new("gHashTag")?,
    };

    let rc = unsafe {
        ffi::trinity_deploy_fly(
            region_c.as_ptr(),
            org_c.as_ptr(),
        )
    };

    // Free allocated strings
    unsafe {
        ffi::trinity_free_string(region_c.as_ptr() as *mut _);
        ffi::trinity_free_string(org_c.as_ptr() as *mut _);
    }

    if rc == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("deploy to fly failed with code {}", rc))
    }
}

#[cfg(feature = "ffi")]
/// Get current instance status
pub fn instance_status() -> Result<Option<InstanceStatus>, anyhow::Error> {
    let rc = unsafe { ffi::trinity_instance_status() };
    if rc == 0 {
        let ptr = unsafe { ffi::trinity_instance_status() };
        if ptr.is_null() {
            Ok(None)
        } else {
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let json_str = c_str.to_string_lossy().unwrap_or_else(|_| "unknown".into());
            serde_json::from_str::<serde_json::Value>(&json_str)
                .map_err(|e| anyhow::anyhow!("failed to parse instance status: {}", e))
                .and_then(|v| {
                    // Extract fields from JSON
                    let id = v.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let region = v.get("region").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let state_str = v.get("state").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let health_str = v.get("health").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let uptime_ns = v.get("uptime_ns").and_then(|s| s.as_i64()).unwrap_or(0);
                    let last_check = v.get("last_health_check").and_then(|s| s.as_i64()).unwrap_or(0);
                    let restarts = v.get("restart_count").and_then(|s| s.as_u32()).unwrap_or(0);
                    let connections = v.get("connections").and_then(|s| s.as_u32()).unwrap_or(0);
                    let memory = v.get("memory_mb").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    let cpu = v.get("cpu_percent").and_then(|s| s.as_f64()).unwrap_or(0.0);

                    let region = match region.as_str() {
                        "ams" => FlyRegion::Amsterdam,
                        "cdg" => FlyRegion::Paris,
                        "fra" => FlyRegion::Frankfurt,
                        "lax" => FlyRegion::LosAngeles,
                        "ord" => FlyRegion::Chicago,
                        "iad" => FlyRegion::Virginia,
                        "sin" => FlyRegion::Singapore,
                        "nrt" => FlyRegion::Tokyo,
                        "hkg" => FlyRegion::HongKong,
                        "syd" => FlyRegion::Sydney,
                        _ => FlyRegion::Amsterdam, // default
                    };

                    let state = match state_str.as_str() {
                        "starting" => InstanceState::Starting,
                        "running" => InstanceState::Running,
                        "stopping" => InstanceState::Stopping,
                        "stopped" => InstanceState::Stopped,
                        "crashed" => InstanceState::Crashed,
                        "restarting" => InstanceState::Restarting,
                        _ => InstanceState::Stopped,
                    };

                    let health = match health_str.as_str() {
                        "healthy" => InstanceHealth::Healthy,
                        "degraded" => InstanceHealth::Degraded,
                        "down" => InstanceHealth::Down,
                        _ => InstanceHealth::Unknown,
                    };

                    Ok(InstanceStatus {
                        id,
                        region,
                        state,
                        health,
                        uptime_ns,
                        last_health_check: last_check,
                        restart_count: restarts,
                        connections,
                        memory_mb: memory,
                        cpu_percent: cpu,
                    })
                })
        }
    } else {
        Err(anyhow::anyhow!("instance status failed with code {}", rc))
    }
}

#[cfg(feature = "ffi")]
/// Restart an instance
pub fn restart_instance() -> Result<(), anyhow::Error> {
    let rc = unsafe { ffi::trinity_restart_instance() };
    if rc == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("restart instance failed with code {}", rc))
    }
}

#[cfg(feature = "ffi")]
/// Stop an instance
pub fn stop_instance() -> Result<(), anyhow::Error> {
    let rc = unsafe { ffi::trinity_stop_instance() };
    if rc == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("stop instance failed with code {}", rc))
    }
}

#[cfg(not(feature = "ffi"))]
pub fn version() -> Result<String, FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn send_collaboration_message(_msg: &str, _timeout_ms: u64) -> Result<(), FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn health_check() -> Result<String, FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn deploy_to_fly(_region: FlyRegion, _org: Option<&str>) -> Result<(), FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn instance_status() -> Result<Option<InstanceStatus>, FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn restart_instance() -> Result<(), FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(not(feature = "ffi"))]
pub fn stop_instance() -> Result<(), FfiNotAvailable> {
    Err(FfiNotAvailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_returns_error() {
        let result = version();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "zig-agents FFI not available. Enable with --features ffi or add zig-agents vendor submodule.");
    }
}
