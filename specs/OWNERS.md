// SPDX-License-Identifier: Apache-2.0
# OWNERS — specs/

## Primary

**T-Queen** (orchestration) with **domain leads** per subtree — `.t27` / `.tri` language SSOT.

## Subtree owners

| Path | Primary agent | Notes |
|------|----------------|-------|
| `base/`, `compiler/` | **C-Compiler** | Core language |
| `numeric/` | **N-Numeric** | GoldenFloat family |
| `math/`, `physics/` | **P-Physics** | Constants and sacred physics overlays |
| `ar/` | **R-Reasoning** | CLARA / proof / ASP |
| `queen/` | **T-Queen** | Lotus orchestration spec |
| `brain/` | **T-Queen** + **P-Physics** + **N-Numeric** | Strand VI — unified brain specs; see `specs/brain/OWNERS.md` |
| `fpga/` | **B-Builder** / hardware | Boards, constraints, testbenches |
| `vsa/`, `nn/` | **N-Numeric** / ML adjacent | Bundles and attention specs |
| `demos/`, `sandbox/` | **A-Architect** / **Q-QA** | Examples; not ring gold by default |
| `isa/` | **C-Compiler** | Register alphabet |

Each subtree with substantial churn should keep a local **`OWNERS.md`** (see below).

## Dependencies

- `conformance/`, `gen/`, `bootstrap/` — downstream of spec changes.
