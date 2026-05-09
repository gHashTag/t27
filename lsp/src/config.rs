// Server configuration for t27 Language Server

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    /// Path to t27c binary for fallback parsing
    #[serde(default)]
    pub t27c_path: Option<String>,

    /// Maximum number of workspace files to index
    #[serde(default = "default_max_workspace_files")]
    pub max_workspace_files: usize,

    /// Enable experimental features
    #[serde(default)]
    pub experimental: bool,

    /// Diagnostic configuration
    #[serde(default)]
    pub diagnostics: DiagnosticConfig,

    /// Completion configuration
    #[serde(default)]
    pub completion: CompletionConfig,

    /// Semantic tokens configuration
    #[serde(default)]
    pub semantic_tokens: SemanticTokensConfig,
}

fn default_max_workspace_files() -> usize {
    1000
}

/// Diagnostic configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DiagnosticConfig {
    /// Enable type errors
    #[serde(default = "default_true")]
    pub enable_type_errors: bool,

    /// Enable seal errors
    #[serde(default = "default_true")]
    pub enable_seal_errors: bool,

    /// Enable warnings
    #[serde(default = "default_true")]
    pub enable_warnings: bool,

    /// Enable semantic errors from invariants
    #[serde(default = "default_true")]
    pub enable_semantic_errors: bool,
}

fn default_true() -> bool {
    true
}

/// Completion configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CompletionConfig {
    /// Enable code snippets
    #[serde(default = "default_true")]
    pub enable_snippets: bool,

    /// Show documentation in completion
    #[serde(default = "default_true")]
    pub show_documentation: bool,

    /// Trigger characters for completion
    #[serde(default = "default_trigger_characters")]
    pub trigger_characters: Vec<char>,

    /// Maximum number of completion items
    #[serde(default = "default_max_completion_items")]
    pub max_items: usize,
}

fn default_trigger_characters() -> Vec<char> {
    vec!['.', ':', '(', '{']
}

fn default_max_completion_items() -> usize {
    100
}

/// Semantic tokens configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SemanticTokensConfig {
    /// Enable semantic tokens
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enable semantic tokens for modules
    #[serde(default = "default_true")]
    pub modules: bool,

    /// Enable semantic tokens for functions
    #[serde(default = "default_true")]
    pub functions: bool,

    /// Enable semantic tokens for types
    #[serde(default = "default_true")]
    pub types: bool,

    /// Enable semantic tokens for constants
    #[serde(default = "default_true")]
    pub constants: bool,

    /// Enable semantic tokens for variables
    #[serde(default = "default_true")]
    pub variables: bool,
}

impl ServerConfig {
    /// Load configuration from workspace directory
    pub fn from_workspace(workspace_root: &PathBuf) -> Self {
        let config_path = workspace_root.join(".t27-lsp.json");

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_workspace_files == 0 {
            return Err("max_workspace_files must be greater than 0".to_string());
        }

        if self.completion.max_items == 0 {
            return Err("completion.max_items must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.max_workspace_files, 1000);
        assert!(config.diagnostics.enable_type_errors);
        assert!(config.completion.enable_snippets);
        assert!(config.semantic_tokens.enabled);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_max_files() {
        let mut config = ServerConfig::default();
        config.max_workspace_files = 0;
        assert!(config.validate().is_err());
    }
}
