# ring-103-rust — On-Chip Learning

phi-tempered SGD step (w -= lr * (1/phi) * clip(g)), in-place, alloc-free.

## Status (honest, Wave 12 / Track C)

- Written, alloc-only, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.
- Tests included (`cargo test -p ring-103`); **CI compile gate lands in Wave 12 / Track D** (Docker rust:1.83-bookworm).
- Not yet listed in workspace `[workspace].members` — opt-in until Track D.

## Identity

Anchor: `phi^2 + 1/phi^2 = 3` — verified in the `identity_witness()` function /
`Mesh::identity_witness` (ring-100) of every crate.
