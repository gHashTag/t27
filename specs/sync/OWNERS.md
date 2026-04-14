# Sync Module Ownership

## Scope

The `sync/` module contains specifications for:
- Change synchronization (sync IDs, states, events)
- Delta generation and application
- Index management and checkpointing
- Conflict resolution and replay

## Maintainers

- Primary: @playra (Playra)
- Contact: `https://github.com/opencode-ai/opencode`

## Dependencies

- `base/` — Core types and utilities
- `bus/` — Pub/Sub event system

## Files

| File | Description | Owner |
|------|-------------|-------|
| `schema.t27` | Sync types (SyncID, states, events) | @playra |
| `index.t27` | Index operations, checkpointing, deltas | @playra |

## Review Requirements

- PRs touching `sync/` must be reviewed by module owner
- Breaking changes require team consensus
- Follow existing patterns and Trinity SSOT guidelines

## Notes

This module implements change synchronization primitives for the OpenCode t27 language
server, enabling distributed editing sessions and conflict resolution across multiple
clients. It provides delta-based updates, index management with checkpoints,
and deterministic replay of operations.
