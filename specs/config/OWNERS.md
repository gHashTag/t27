# Config Module Ownership

## Scope

The `config/` module contains specifications for:
- Configuration structures (providers, agents, LSP, TUI, logging)
- Configuration file I/O (load, save, merge)
- Path resolution and directory management
- Configuration validation and migration

## Maintainers

- Primary: @playra (Playra)
- Contact: `https://github.com/opencode-ai/opencode`

## Dependencies

- `base/` — Core types and utilities
- `runtime/` — For process execution in config

## Files

| File | Description | Owner |
|------|-------------|-------|
| `schema.t27` | Configuration structures (ProviderConfig, AgentConfig, etc.) | @playra |
| `load.t27` | Config file I/O, merging, validation | @playra |
| `paths.t27` | Path resolution, directory management | @playra |
| `migrate.t27` | Version detection, upgrade, compatibility | @playra |

## Review Requirements

- PRs touching `config/` must be reviewed by module owner
- Breaking changes require team consensus
- Follow existing patterns and Trinity SSOT guidelines

## Notes

This module provides the configuration layer for the OpenCode t27 language server,
supporting multiple LLM providers, agent definitions, LSP server configuration,
and UI preferences. It handles configuration file loading/saving, path resolution
for different environments, and automatic migration between config schema versions.
