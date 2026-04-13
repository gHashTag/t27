# Runtime Module Ownership

## Scope

The `runtime/` module contains specifications for:
- Task execution (sync, async, promises)
- Process spawning and management (PTY, pipes)
- Instance registration and lifecycle management
- Cancellation and timeout handling

## Maintainers

- Primary: @playra (Playra)
- Contact: `https://github.com/opencode-ai/opencode`

## Dependencies

- `base/` — Core types and utilities
- `config/` — For configuration of runtime processes

## Files

| File | Description | Owner |
|------|-------------|-------|
| `execute.t27` | Task execution, promises, cancellation | @playra |
| `process.t27` | Process spawning, PTY, pipes | @playra |
| `instance.t27` | Instance registration, lookup, lifecycle | @playra |

## Review Requirements

- PRs touching `runtime/` must be reviewed by module owner
- Breaking changes require team consensus
- Follow existing patterns and Trinity SSOT guidelines

## Notes

This module provides the runtime execution layer for the OpenCode t27 language server,
enabling synchronous and asynchronous task execution, subprocess management with PTY support,
and instance lifecycle management. It supports cancellation, timeouts, and proper
cleanup of resources.
