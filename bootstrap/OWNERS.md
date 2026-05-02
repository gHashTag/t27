# OWNERS — bootstrap/

## Primary

**B-Builder** — Rust bootstrap compiler (binary name `t27c`; user-facing **`tri`** via `scripts/tri`), `Cargo.toml`, `Dockerfile`, `build.rs` language guard.

## Dependencies

- `specs/**/*.t27`, `compiler/**/*.t27` — parse/gen targets.
- `bootstrap/stage0/FROZEN_HASH` — ring seal.

## Outputs

- `target/release/t27c` (local build; avoid committing binaries).
- Unified production image when built from repo root `bootstrap/Dockerfile`.
