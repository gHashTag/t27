//! Top-level library for the t27 project.
//! 
//! This crate re-exports the main components used throughout the project.

// Re-export the DLC10 driver for use by other crates
pub use cli_dlc10::*;