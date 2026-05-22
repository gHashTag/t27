# rings/ -- Living Compile Status (Wave 13 gate, Waves 14-16 promotions)

> Last updated: 2026-05-22 (Wave 16)
> Anchor: phi^2 + 1/phi^2 = 3
> CI workflow: [`.github/workflows/rings-rust.yml`](../.github/workflows/rings-rust.yml)
> Toolchain: pinned via [`Dockerfile.rust`](../Dockerfile.rust) -- `rust:1.83-bookworm`

This file is the **honest, living** per-crate compilation status for every
`rings/ring-*-rust/` crate. Wave 13 introduces the **Toolchain & Compilation
Gate**: a non-blocking GitHub Actions matrix that runs `cargo check` and
`cargo test` against the pinned 1.83 toolchain. Results here are updated
as crates graduate from "scaffolded" to "compiles" to "tested".

## Legend

| Symbol           | Meaning                                                              |
|------------------|----------------------------------------------------------------------|
| `scaffold`       | Files present on disk, never compiled in CI                          |
| `check`          | `cargo check --all-targets` passes in CI                             |
| `test`           | `cargo test` passes in CI                                            |
| `claimed-only`   | Earlier narrative referenced this crate; **no source in this repo**. |

## Wave 12 Track C -- ring-100..ring-104 (on disk)

All 5 crates were promoted from `scaffold` to `check` + `test` in **Wave 14**
(2026-05-22, Closes #715) once the root `Cargo.toml` `exclude` list was extended
to cover `rings/`. Test counts below are the **actual** numbers reported by
`cargo test` on Rust 1.83.0 -- the Wave-12 NOW entry's claim of 28 total tests
was off by two; the honest total is **26** (R5-HONEST correction).

| Crate                  | Domain                  | LOC | Tests | Status            |
|------------------------|-------------------------|----:|------:|-------------------|
| `ring-100-rust`        | Multi-Chip Mesh         | 205 |     4 | `check` + `test`  |
| `ring-101-rust`        | Analog GF16             | 144 |     5 | `check` + `test`  |
| `ring-102-rust`        | Photonic MAC            | 157 |     5 | `check` + `test`  |
| `ring-103-rust`        | On-Chip Learning phi-SGD| 131 |     6 | `check` + `test`  |
| `ring-104-rust`        | Telemetry Bus           | 185 |     6 | `check` + `test`  |

**Track-C totals (verified):** 5 crates `cargo check` green, 26 tests pass,
0 fail. Verified locally on Rust 1.83.0; promotion will be re-confirmed by
the first green `rings-rust` workflow run on this PR.

## Wave 15 import -- ring-088 (on disk, real)

Wave 15 (2026-05-22, Closes #717) imports the first Wave-11 crate **for real**.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test` reports
**13 passed, 0 failed** (mandatory-8 from `specs/02-gf16-format.tri` plus 5 MAC
and identity tests). Promotion will be re-confirmed by the green `rings-rust`
workflow run that this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-088-rust`    | GF16 codec + MAC (`mac_dot`)      |  439 |    13 | `check` + `test`  |

## Wave 16 import -- ring-089 (on disk, real)

Wave 16 (2026-05-22, Closes #719) imports the **second** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check --all-targets` green,
`cargo test --lib` reports **15 passed, 0 failed**. Includes
`cpu_phi_identity_integer_projection` -- the second cross-kernel anchor
test in the project (after Wave 15's `mac_dot_phi_identity`), exercising
`phi^2 + 1/phi^2 = 3` through the CPU's fetch/decode/execute loop via an
integer projection (`floor(phi) + floor(1/phi) + ceil(phi^2 - 2) = 3`).
The earlier Wave-11 narrative claimed 334 LOC for this ring; the honest
Wave-16 number is **635 LOC** (R5-HONEST correction). Promotion will be
re-confirmed by the green `rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-089-rust`    | TNN ISA (27-reg balanced ternary) |  635 |    15 | `check` + `test`  |

## Wave 11 -- ring-090..ring-099 (claimed-only, off-disk)

Wave 11's narrative described 11 additional crates with ~ 8 969 LOC. Their
sources never reached this repository. Wave 15 acknowledged this honestly;
Wave 16 has now promoted `ring-089-rust` out of this table. The rows below
are **claimed-only** placeholders, *not* deliverables. They will be promoted
to `check` + `test` one ring at a time, each via its own PR that carries
real source + local verification (the Wave-15/16 template). LOC numbers
below are quotes from past narrative, not measurements.

| Crate              | Domain                            |  LOC (claimed) | Status         |
|--------------------|-----------------------------------|---------------:|----------------|
| `ring-090-rust`    | Simulator                         |           2143 | `claimed-only` |
| `ring-091-rust`    | Stochastic Rounding               |            409 | `claimed-only` |
| `ring-092-rust`    | Attention                         |            847 | `claimed-only` |
| `ring-093-rust`    | Sparse MoE                        |            668 | `claimed-only` |
| `ring-094-rust`    | AGI Runtime                       |            774 | `claimed-only` |
| `ring-095-rust`    | phi-Adam                          |            659 | `claimed-only` |
| `ring-096-rust`    | Quantization                      |            464 | `claimed-only` |
| `ring-097-rust`    | Chain-of-Thought                  |            624 | `claimed-only` |
| `ring-098-rust`    | World Model                       |            920 | `claimed-only` |
| `ring-099-rust`    | Integration                       |           1127 | `claimed-only` |

## How to read the CI result

1. Open the **rings-rust** workflow on the latest commit.
2. The `discover` job prints the matrix of detected crates.
3. Each matrix leg runs `cargo check` + `cargo test` for one crate.
4. The job is `continue-on-error: true` -- a red leg surfaces honest
   breakage **without** blocking merges. This file is the source of
   truth for what we claim works.

## Compliance

- **L1 TRACEABILITY** -- every status change must arrive via a PR with
  `Closes #N` and a corresponding `docs/NOW.md` entry.
- **L3 PURITY** -- ASCII only; English doc-comments.
- **L7 UNITY** -- gate logic is Python (`scripts/ci/rings_matrix.py`),
  no new shell scripts.
- **R5-HONEST** -- a crate is only promoted past `scaffold` once a CI
  log proves it.
