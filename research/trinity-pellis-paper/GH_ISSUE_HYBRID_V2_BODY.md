## Context
`tri math compare --hybrid` (#277) implements **hybrid v1**: L1-normalized φ^k dotted with max-normalized Pell weights P_1..P_5, k=0..4. Reproducible CLI value ≈ **0.563780474444** (not L2 cosine).

Exploratory calculations (external / not in-tree) suggest an **L2 cosine + growing N** map may plateau near **cos θ ≈ 0.9617** (θ ≈ 15.9°). That result is **not** Trinity SSOT until implemented, documented, and tested.

## Goal
1. Freeze **v1 vs v2** definitions in `research/trinity-pellis-paper/hybrid-conjecture.md`.
2. Extend `tri math compare` (flags or subcommand) for **hybrid v2** and optional θ = arccos(clip(cosine similarity)) in **Rust only**.
3. **Golden tests** at fixed checkpoints **N = 5, 10, 15, 20, 50, 152** once the map is fixed (no chart-only claims).
4. **Experience JSONL**: log `hybrid_v1`, `hybrid_v2`, `theta_deg`, `N`, `pellis_spec_seal_hash`.

## Non-goals
- Claiming Conjecture H1 "confirmed" from non-reproducible plots.
- Python on the verification critical path.

## Acceptance
- `cargo build --release` + `t27c suite` green.
- One-line repro for collaborators that matches committed math.

Refs: #277, `research/trinity-pellis-paper/hybrid-conjecture.md`, `bootstrap/src/math_compare.rs`
