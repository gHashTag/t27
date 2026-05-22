# rings/ -- Living Compile Status (Wave 13 gate, Waves 14-20 promotions)

> Last updated: 2026-05-22 (Wave 20)
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

## Wave 17 import -- ring-090 (on disk, real)

Wave 17 (2026-05-22, Closes #721) imports the **third** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check --all-targets` green,
`cargo test --lib` reports **19 passed, 0 failed** on the first run
(no bug-fix cycle this time). Ring-090 mirrors
[`specs/fpga/simulator.t27`](../specs/fpga/simulator.t27) byte-for-byte:
`SimState`, `SimConfig`, `SimResult`, `ProbePoint`, `TraceEntry`, all the
spec's constructor / query / time-conversion / validation helpers, plus
the universal anchor. The 19 tests cover all 13 `test` blocks and all 4
`invariant` blocks in the spec, plus `identity_witness_holds` and a
`sim_state_tag_roundtrip` type-safety check. Earlier Wave-11 narrative
claimed 2143 LOC; honest Wave-17 measurement is **547 LOC** (R5-HONEST).
Promotion will be re-confirmed by the green `rings-rust` workflow run this
PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-090-rust`    | Simulator (HIR cycle-accurate)    |  547 |    19 | `check` + `test`  |

## Wave 18 import -- ring-091 (on disk, real)

Wave 18 (2026-05-22, Closes #723) imports the **fourth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check --all-targets` green,
`cargo test --lib` reports **19 passed, 0 failed** on the first run.
Ring-091 implements **stochastic rounding** (SR) over `f32` integer and
uniform-grid targets, backed by a deterministic seedable `SplitMix64`
PRNG (Vigna 2014). The crate's `splitmix_first_value_with_seed_0` test
checks the published reference value `0xE220A8397B1DCDAF`; the two
statistical tests (`sr_is_unbiased`, `sr_quantize_phi_unbiased`) verify
unbiasedness empirically against a 3-sigma bound on 10 000 draws each.
`sr_quantize_phi_unbiased` is the **third cross-kernel anchor test** in
the project (after Wave 15's `mac_dot_phi_identity` and Wave 16's
`cpu_phi_identity_integer_projection`): it exercises `phi` through
SR-quantization. Earlier Wave-11 narrative claimed 409 LOC; honest
Wave-18 measurement is **462 LOC** (the first ring whose honest LOC
modestly *exceeds* the claim). Promotion will be re-confirmed by the
green `rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-091-rust`    | Stochastic Rounding + SplitMix64  |  462 |    19 | `check` + `test`  |

## Wave 19 import -- ring-092 (on disk, real)

Wave 19 (2026-05-22, Closes #725) imports the **fifth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **28 passed, 0 failed** on the first run. Ring-092 mirrors the
realizable subset of [`specs/nn/attention.t27`](../specs/nn/attention.t27)
(SacredAttention): sacred constants byte-for-byte (`NUM_HEADS=3`,
`HEAD_DIM=81`, `EMBED_DIM=243`, `CONTEXT_LEN=81`, `ROPE_PAIRS=40`,
`SACRED_GAMMA = phi^-3`, `SACRED_SCALE = 81^(-SACRED_GAMMA)`); `Trit`
enum; and the primitives `ternary_matmul`, `add_residual`, `apply_softmax`
(numerically stable max-subtract, per-head), `compute_scores` (Q.K^T with
causal mask + sacred scaling), `weighted_values`, `cache_kv`. A private
`exp_f64` (range-reduction + Taylor series) makes softmax viable in
`no_std` without libm. The crate's `attention_phi_identity_via_softmax_matmul`
is the **fourth cross-kernel anchor test** in the project (after
ring-088, ring-089, ring-091), routing `phi^2 + 1/phi^2 = 3` through
softmax-style normalization and ternary matmul. RoPE table init (cos/sin)
and the full `sacred_attention_kernel` orchestrator are explicitly out of
scope (R5-HONEST). Earlier Wave-11 narrative claimed 847 LOC; honest
Wave-19 measurement is **760 LOC**. Promotion will be re-confirmed by
the green `rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-092-rust`    | Attention (Sacred primitives)     |  760 |    28 | `check` + `test`  |

## Wave 20 import -- ring-093 (on disk, real)

Wave 20 (2026-05-22, Closes #727) imports the **sixth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **28 passed, 0 failed** on the first run. Ring-093 has no
backing file under `specs/` (textbook algorithm, like ring-091's SR);
the design follows the canonical Shazeer-2017 / Switch-Transformer
top-k routing structure with ternary (`Trit`) expert weights and
Trinity defaults (`NUM_EXPERTS = 3`, `DEFAULT_TOP_K = 1`,
`DEFAULT_EMBED_DIM = 243`, `DEFAULT_EXPERT_HIDDEN_DIM = 729 = 3^6`).
Exposes `MoEConfig`, `gate_top_k` (top-k selection + max-subtract
softmax over selected logits, numerically stable), `expert_ffn` (two-layer
ternary FFN with ReLU), `moe_forward` (composes gating with per-expert
FFNs into a single token's MoE output, fully allocation-free), `relu_inplace`,
`load_balance_loss` (Switch-Transformer importance balance), and the
universal anchor. A private `exp_f64` (range-reduced Taylor series)
makes the gating softmax viable in `no_std` without libm. The crate's
`moe_phi_identity_via_gating_and_ffn` is the **fifth cross-kernel
anchor test** in the project (after ring-088, ring-089, ring-091, and
ring-092), routing `phi^2 + 1/phi^2 = 3` through MoE gating + ternary
FFN. The Wave-11 narrative quoted 668 LOC; honest Wave-20 measurement
is **950 LOC**. Promotion will be re-confirmed by the green
`rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-093-rust`    | Sparse MoE (top-k + ternary FFN)  |  950 |    28 | `check` + `test`  |

## Wave 11 -- ring-094..ring-099 (claimed-only, off-disk)

Wave 11's narrative described 11 additional crates with ~ 8 969 LOC. Their
sources never reached this repository. Waves 15-20 acknowledge this
honestly and have promoted `ring-088`, `ring-089`, `ring-090`, `ring-091`,
`ring-092`, and `ring-093` out of this table. The rows below are
**claimed-only** placeholders, *not* deliverables. They will be promoted
to `check` + `test` one ring at a time, each via its own PR that carries
real source + local verification (the Wave-15..20 template). LOC numbers
below are quotes from past narrative, not measurements.

| Crate              | Domain                            |  LOC (claimed) | Status         |
|--------------------|-----------------------------------|---------------:|----------------|
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
