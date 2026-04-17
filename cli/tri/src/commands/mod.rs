//! CLI command modules
//!
//! This module contains all CLI commands organized by maturity:
//! - **Stable**: Commands that call t27c and are production-ready
//! - **Experimental**: Commands that are stubs pending future integration

pub mod experimental;
pub mod pipeline;
pub mod bench;
pub mod parse;

// Make modules available to main.rs
// Note: This file only declares the module structure here
