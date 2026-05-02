//! Compiler Memory Store Backend for Native Memory System
//!
//! This module provides content-addressable storage for MemoryCell
//! with scope isolation and TTL support for Session scope.

pub mod store;

pub use store::{
    MemoryCell,
    MemoryKey,
    MemScope,
    MemoryStore,
    FileMemoryStore,
    compute_key,
    Result,
    MemoryError,
};
