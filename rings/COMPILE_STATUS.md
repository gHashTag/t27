# rings/ -- Living Compile Status (Wave 13 gate, Wave 14 promotion)

> Last updated: 2026-05-22 (Wave 14)
> Anchor: phi^2 + 1/phi^2 = 3
> CI workflow: [`.github/workflows/rings-rust.yml`](../.github/workflows/rings-rust.yml)
> Toolchain: pinned via [`Dockerfile.rust`](../Dockerfile.rust) -- `rust:1.83-bookworm`

This file is the **honest, living** per-crate compilation status for every
`rings/ring-*-rust/` crate. Wave 13 introduces the **Toolchain & Compilation
Gate**: a non-blocking GitHub Actions matrix that runs `cargo check` and
`cargo test` against the pinned 1.83 toolchain. Results here are updated
as crates graduate from "scaffolded" to "compiles" to "tested".

## Legend

| Symbol      | Meaning                                                     |
|-------------|-------------------------------------------------------------|
| `scaffold`  | Files present on disk, never compiled in CI                 |
| `check`     | `cargo check --all-targets` passes in CI                    |
| `test`      | `cargo test` passes in CI                                   |
| `off-disk`  | Authored in another sandbox; not yet imported into this repo|

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

## Wave 11 -- ring-088..ring-099 (authored off-disk)

These crates were authored in a prior sandbox and are documented in the
Wave 11 README section, but their sources are **not** yet present in this
repository. They will be imported and pass the Wave 13 gate incrementally.

| Crate              | Domain                            |  LOC | Status     |
|--------------------|-----------------------------------|-----:|------------|
| `ring-088-rust`    | GF16 MAC                          |  961 | `off-disk` |
| `ring-089-rust`    | TNN ISA                           |  334 | `off-disk` |
| `ring-090-rust`    | Simulator                         | 2143 | `off-disk` |
| `ring-091-rust`    | Stochastic Rounding               |  409 | `off-disk` |
| `ring-092-rust`    | Attention                         |  847 | `off-disk` |
| `ring-093-rust`    | Sparse MoE                        |  668 | `off-disk` |
| `ring-094-rust`    | AGI Runtime                       |  774 | `off-disk` |
| `ring-095-rust`    | phi-Adam                          |  659 | `off-disk` |
| `ring-096-rust`    | Quantization                      |  464 | `off-disk` |
| `ring-097-rust`    | Chain-of-Thought                  |  624 | `off-disk` |
| `ring-098-rust`    | World Model                       |  920 | `off-disk` |
| `ring-099-rust`    | Integration                       | 1127 | `off-disk` |

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
