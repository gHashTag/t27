# Server Module Ownership

## Scope

The `server/` module contains specifications for:
- HTTP server functionality (listeners, request/response handling)
- URL routing and pattern matching
- Server-Sent Events (SSE) streaming
- Multicast DNS (mDNS) service discovery

## Maintainers

- Primary: @playra (Playra)
- Contact: `https://github.com/opencode-ai/opencode`

## Dependencies

- `base/` — Core types and utilities
- `config/` — Configuration management

## Files

| File | Description | Owner |
|------|-------------|-------|
| `http.t27` | HTTP server spec, request/response handling | @playra |
| `router.t27` | URL routing, pattern matching | @playra |
| `sse.t27` | Server-Sent Events, streaming | @playra |
| `mdns.t27` | Multicast DNS service discovery | @playra |

## Review Requirements

- PRs touching `server/` must be reviewed by module owner
- Breaking changes require team consensus
- Follow existing patterns and Trinity SSOT guidelines

## Notes

This module implements the server layer for the OpenCode t27 language server,
providing HTTP API, SSE streaming, and mDNS-based service discovery
for local development and testing scenarios.
