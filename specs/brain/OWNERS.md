// SPDX-License-Identifier: Apache-2.0
# OWNERS — specs/brain/

## Primary

**T-Queen** — orchestration, cognitive loop, and brain–periphery API contracts.  
**P-Physics** / **N-Numeric** — φ-timing, sacred coherence, and numeric ties to `specs/math/` and `specs/numeric/`.

## Dependencies

- `specs/base/types.t27`
- `specs/math/constants.t27`, `specs/math/sacred_physics.t27` (for φ and coherence hooks)
- `specs/numeric/gf16.t27` (GoldenFloat timing / confidence)
- `specs/queen/lotus.t27` (high-level orchestration alignment)

## Outputs

- Generated `gen/{zig,c,verilog}/…` via **`tri`** → **`t27c`** (`gen-dir` for trees, `gen-zig` / `gen-c` / `gen-verilog` for single-file stdout, `compile-project`; shim: `./scripts/tri`).
- Future `conformance/brain_*.json` vectors.

## Note

Strand VI is **chartered** in `docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md`; land specs in **ring-sized PRs** with tests/invariants per SOUL.
