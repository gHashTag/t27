// SPDX-License-Identifier: Apache-2.0
//! Memory Store Backend for Native Memory System
//!
//! This module provides content-addressable storage for MemoryCell
//! with scope isolation and TTL support for Session scope.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::io::Write;

use sha3::{Digest, Sha3_256};

/// Memory key type: SHA3-27(phi_hash || key_bytes)
pub type MemoryKey = [u8; 27];

/// Memory cell type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryCell {
    pub key: MemoryKey,
    pub value: Vec<u8>,
    pub scope: MemScope,
    pub phi_hash: u64,
    pub timestamp: u64,
    pub ttl: Option<u64>,
}

/// Memory scopes
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemScope {
    Agent { agent_id: String },
    Session { agent_id: String, session_id: String },
    Permanent,
    Ephemeral,
}

/// Memory store interface
pub trait MemoryStore {
    fn write(&mut self, cell: MemoryCell) -> Result<()>;
    fn read(&self, key: &MemoryKey) -> Result<Option<MemoryCell>>;
    fn delete(&mut self, key: &MemoryKey) -> Result<()>;
    fn list(&self, scope: &MemScope) -> Result<Vec<MemoryCell>>;
    fn list_active(&self, scope: &MemScope) -> Result<Vec<MemoryCell>>;
    fn tombstone(&mut self, key: &MemoryKey) -> Result<()>;
    fn cleanup_expired(&mut self) -> Result<()>;
}

/// File-based memory store (`.trinity/memory/`)
#[derive(Debug, Default)]
pub struct FileMemoryStore {
    base_path: PathBuf,
    ephemeral: HashMap<MemoryKey, MemoryCell>,
}

impl FileMemoryStore {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let base = base_path.as_ref();
        fs::create_dir_all(base).ok();
        Self {
            base_path: base.to_path_buf(),
            ephemeral: HashMap::new(),
        }
    }

    fn scope_path(&self, scope: &MemScope) -> PathBuf {
        match scope {
            MemScope::Agent { agent_id } => {
                self.base_path.join("agent").join(agent_id)
            }
            MemScope::Session { agent_id, session_id } => {
                self.base_path.join("session").join(agent_id).join(session_id)
            }
            MemScope::Permanent => {
                self.base_path.join("permanent")
            }
            MemScope::Ephemeral => {
                panic!("Ephemeral scope does not use file-based storage")
            }
        }
    }

    fn is_expired(&self, cell: &MemoryCell) -> bool {
        if let Some(ttl) = cell.ttl {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > ttl
        } else {
            false
        }
    }
}

impl MemoryStore for FileMemoryStore {
    fn write(&mut self, cell: MemoryCell) -> Result<()> {
        let scope_dir = self.scope_path(&cell.scope);
        fs::create_dir_all(&scope_dir)?;
        
        let cell_path = scope_dir.join(format!("{:02x}.json", cell.phi_hash % 256));
        let json = serde_json::to_string_pretty(&cell)?;
        fs::write(&cell_path, json)?;
        
        Ok(())
    }

    fn read(&self, key: &MemoryKey) -> Result<Option<MemoryCell>> {
        if let Some(cell) = self.ephemeral.get(key) {
            if self.is_expired(cell) {
                return Ok(None);
            }
            return Ok(Some(cell.clone()));
        }
        
        // Search in all scope directories
        for scope_dir in ["agent", "session", "permanent"] {
            let search_path = self.base_path.join(scope_dir);
            if !search_path.exists() {
                continue;
            }
            
            for entry in fs::read_dir(&search_path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    continue;
                }
                
                let json = fs::read_to_string(entry.path())?;
                let cell: MemoryCell = serde_json::from_str(&json)?;
                
                if &cell.key == key && !self.is_expired(&cell) {
                    return Ok(Some(cell));
                }
            }
        }
        
        Ok(None)
    }

    fn delete(&mut self, key: &MemoryKey) -> Result<()> {
        self.ephemeral.remove(key);
        
        // Find and delete file
        for scope_dir in ["agent", "session", "permanent"] {
            let search_path = self.base_path.join(scope_dir);
            if !search_path.exists() {
                continue;
            }
            
            for entry in fs::read_dir(&search_path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    continue;
                }
                
                let json = fs::read_to_string(entry.path())?;
                let cell: MemoryCell = serde_json::from_str(&json)?;
                
                if &cell.key == key {
                    fs::remove_file(entry.path())?;
                    return Ok(());
                }
            }
        }
        
        Ok(())
    }

    fn list(&self, scope: &MemScope) -> Result<Vec<MemoryCell>> {
        let mut cells = Vec::new();
        
        if matches!(scope, MemScope::Ephemeral) {
            return Ok(self.ephemeral.values()
                .filter(|cell| !self.is_expired(cell))
                .cloned()
                .collect());
        }
        
        let scope_path = self.scope_path(scope);
        if !scope_path.exists() {
            return Ok(cells);
        }
        
        for entry in fs::read_dir(&scope_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                continue;
            }
            
            let json = fs::read_to_string(entry.path())?;
            let cell: MemoryCell = serde_json::from_str(&json)?;
            
            cells.push(cell);
        }
        
        Ok(cells)
    }

    fn list_active(&self, scope: &MemScope) -> Result<Vec<MemoryCell>> {
        let all_cells = self.list(scope)?;
        Ok(all_cells.into_iter()
            .filter(|cell| !self.is_expired(cell))
            .collect())
    }

    fn tombstone(&mut self, key: &MemoryKey) -> Result<()> {
        self.delete(key)
    }

    fn cleanup_expired(&mut self) -> Result<()> {
        // Clean expired cells from ephemeral store
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.ephemeral.retain(|_, cell| {
            if let Some(ttl) = cell.ttl {
                now <= ttl
            } else {
                true
            }
        });
        
        // Clean expired files from disk
        for scope_dir in ["agent", "session", "permanent"] {
            let search_path = self.base_path.join(scope_dir);
            if !search_path.exists() {
                continue;
            }
            
            for entry in fs::read_dir(&search_path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    continue;
                }
                
                let json = fs::read_to_string(entry.path())?;
                let cell: MemoryCell = serde_json::from_str(&json)?;
                
                if let Some(ttl) = cell.ttl {
                    if now > ttl {
                        fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Compute SHA3-27 hash of phi_hash concatenated with key bytes
pub fn compute_key(phi_hash: u64, key_bytes: &[u8]) -> MemoryKey {
    let mut hasher = Sha3_256::new();
    hasher.update(&phi_hash.to_le_bytes());
    hasher.update(key_bytes);
    let result = hasher.finalize();
    
    let mut key = [0u8; 27];
    key.copy_from_slice(&result[..27]);
    key
}

/// Custom error type for memory operations
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Scope validation error: {0}")]
    ScopeError(String),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
