//! `tri` — Trinity IGLA Needle Hunt CLI
//!
//! Automation CLI for IGLA experiment workflow:
//! - Run experiments and parse BPB from stdout
//! - Parameter sweeps with N×run execution
//! - Auto-sync results to issue #143 table
//! - Agent roster management
//! - Dashboard sync/refresh
//! - Gate checking (BPB, size, time)
//! - Parameter submission
//! - Leaderboard (SQLite)
//! - Agent dispatch
//! - Atomic git commits via libgit2

pub mod cmd;
pub mod config;
pub mod db;
pub mod gh;
pub mod lock;
pub mod metrics;
pub mod table;

pub use config::Config;
pub use db::{Entry, Leaderboard, Stats};
pub use gh::{AgentIssue, GhClient};
pub use lock::LockGuard;
pub use metrics::{validate_bpb, validate_param_count, validate_time};
pub use table::{parse_table, TableRow, update_table};
