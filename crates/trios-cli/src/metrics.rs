//! BPB/size/time validators

use anyhow::Result;

pub const GATES: &[(&str, f64)] = &[
    ("G-BGH", 1.50),
    ("G-ORTH", 1.20),
    ("G-SWA", 1.15),
    ("G-STACK", 1.12),
    ("G-NEEDLE", 1.00),
];

const MAX_PARAMS: u64 = 1_000_000;
const MAX_TIME_SEC: f64 = 3600.0; // 1 hour

/// Validate BPB against gates
pub fn validate_bpb(bpb: f64) -> Result<&'static str> {
    for (name, threshold) in GATES.iter().rev() {
        if bpb <= *threshold {
            return Ok(name);
        }
    }
    Ok("FAILED")
}

/// Validate parameter count
pub fn validate_param_count(params: u64) -> Result<()> {
    if params > MAX_PARAMS {
        anyhow::bail!("Parameter count {params} exceeds maximum {MAX_PARAMS}");
    }
    Ok(())
}

/// Validate training time
pub fn validate_time(time_sec: f64) -> Result<()> {
    if time_sec > MAX_TIME_SEC {
        anyhow::bail!("Training time {time_sec}s exceeds maximum {MAX_TIME_SEC}s");
    }
    Ok(())
}
