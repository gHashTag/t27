# ring-104-rust — Telemetry Bus

Bounded lossy ring buffer of (ts, 4-byte tag, value) samples, FIFO eviction.

## Status (honest, Wave 12 / Track C)

- Written, alloc-only, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.
- Tests included (`cargo test -p ring-104`); **CI compile gate lands in Wave 12 / Track D** (Docker rust:1.83-bookworm).
- Not yet listed in workspace `[workspace].members` — opt-in until Track D.

## Identity

Anchor: `phi^2 + 1/phi^2 = 3` — verified in the `identity_witness()` function /
`Mesh::identity_witness` (ring-100) of every crate.
