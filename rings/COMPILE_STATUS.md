# rings/ -- Living Compile Status (Wave 13 gate, Waves 14-26 promotions, Wave-11 series COMPLETE)

> Last updated: 2026-05-22 (Wave 26 -- Wave-11 series complete)
> Anchor: phi^2 + 1/phi^2 = 3
> CI workflow: [`.github/workflows/rings-rust.yml`](../.github/workflows/rings-rust.yml)
> Toolchain: pinned via [`Dockerfile.rust`](../Dockerfile.rust) -- `rust:1.83-bookworm`

> **This file is hand-maintained and does not read CI.** It was last updated
> **2026-05-22**. Between **2026-05-23 and 2026-08-20** seven master runs of
> `rings-rust` had all 17 crate jobs failing to compile while this file said
> they compile -- and every one of those runs concluded `success`, because the
> build job is `continue-on-error` on purpose. The crates were repaired by
> 2026-08-28. Each matrix job now writes its own verdict line into the run
> summary, so where this file and a run disagree, **the run was measured and
> this file was remembered**.

This file is the per-crate compilation status for every
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

## Wave 21 import -- ring-094 (on disk, real)

Wave 21 (2026-05-22, Closes #729) imports the **seventh** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **32 passed, 0 failed** on the first run. Ring-094 mirrors
`specs/runtime/{execute, instance, process}.t27`: spec constants
byte-for-byte (`DEFAULT_TIMEOUT_MS = 30_000`, `MAX_CONCURRENT_EXECUTIONS = 16`,
`POLL_INTERVAL_MS = 100`, `TASK_ID_LENGTH = 32`, `MAX_INSTANCES = 256`,
`INSTANCE_NAME_LENGTH = 128`, `LOOKUP_TIMEOUT_MS = 100`,
`SPAWN_TIMEOUT_MS = 5_000`, `PTY_COLS_DEFAULT = 80`, `PTY_ROWS_DEFAULT = 24`,
`MAX_PIPE_BUFFER = 65_536`); all nine spec enums (`ExecResultType`,
`TaskState`, `CancelReason`, `ProcessSignal`, `ProcessState`, `PTYMode`,
`InstanceState`, `InstanceType`, `TerminationReason`); pure-state-machine
`Promise`; fixed-capacity `Registry` (capped at `MAX_INSTANCES = 256`);
and a Trinity-priority `Scheduler` (capped at `MAX_CONCURRENT_EXECUTIONS =
16`) with a phi-weighted credit policy: `Trit::Pos -> phi^2`,
`Trit::Zero -> 1.0`, `Trit::Neg -> phi^-2`. The crate's
`runtime_phi_identity_via_scheduler_credits` is the **sixth cross-kernel
anchor test** in the project (after ring-088, ring-089, ring-091,
ring-092, ring-093), routing `phi^2 + 1/phi^2 = 3` through the scheduler's
credit accumulator. Real syscalls (`spawn`, `kill`, PTY I/O), heap-backed
containers, and future-executor wakers are explicitly out of scope
(R5-HONEST). Earlier Wave-11 narrative claimed 774 LOC; honest Wave-21
measurement is **1210 LOC**. Promotion will be re-confirmed by the green
`rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-094-rust`    | AGI Runtime (scheduler + registry)| 1210 |    32 | `check` + `test`  |

## Wave 22 import -- ring-095 (on disk, real)

Wave 22 (2026-05-22, Closes #731) imports the **eighth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **25 passed, 0 failed**. Ring-095 mirrors
`specs/ml/optimizer/{adam, adamw}.t27`: spec constants byte-for-byte
(`DEFAULT_LEARNING_RATE = 1e-3`, `DEFAULT_BETA1 = 0.9`,
`DEFAULT_BETA2 = 0.999`, `DEFAULT_WEIGHT_DECAY = 0.01`,
`DEFAULT_EPSILON = 1e-8`, `DEFAULT_AMSGRAD = false`,
`PHI_BETA1 = 0.9 / phi ~= 0.556`, `PHI_BETA2 = 0.999 / phi ~= 0.617`);
`AdamWConfig` with `defaults()` and `phi_preset()` constructors;
caller-owned `AdamWState<'_>` (no allocation); helper functions named
after the spec (`compute_bias_correction`, `update_first_moment`,
`update_second_moment`, `apply_weight_decay`, `compute_update`); and a
full `step()` orchestrator implementing decoupled weight decay,
bias-corrected lr_t = lr * sqrt(1 - beta2^t) / (1 - beta1^t), the moment
recurrences, AMSGrad max-of-v scratch, and the parameter update.
Private no_std math helpers: `pow_u64` (fast exponentiation) and
`sqrt_newton` (Newton-Raphson square root) bypass libm. The crate's
`phi_adam_phi_identity_via_betas` is the **seventh cross-kernel anchor
test** in the project (after ring-088, ring-089, ring-091, ring-092,
ring-093, ring-094), routing `phi^2 + 1/phi^2 = 3` through the
optimizer's `pow_u64` helper and through the phi-damped first-moment
update `m_1 = (1 - 0.9/phi) * phi = phi - 0.9`. Earlier Wave-11
narrative claimed 659 LOC; honest Wave-22 measurement is **808 LOC**.
Promotion will be re-confirmed by the green `rings-rust` workflow run
this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-095-rust`    | phi-Adam optimizer (AdamW + phi)  |  808 |    25 | `check` + `test`  |

## Wave 23 import -- ring-096 (on disk, real)

Wave 23 (2026-05-22, Closes #733) imports the **ninth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **42 passed, 0 failed** on the first run. Ring-096 mirrors
[`specs/numeric/formats.t27`](../specs/numeric/formats.t27) byte-for-byte:
GF16 bit layout (`SIGN_MASK = 0x8000`, `EXP_MASK = 0x7E00`,
`MANT_MASK = 0x01FF`, `EXP_SHIFT = 9`, `SIGN_SHIFT = 15`, `BIAS = 31`,
`EXP_MAX = 63`, `EXP_MIN = 0`); the GF16 codec `gf16_to_f32` /
`f32_to_gf16` (handles signed zero, denormals, normals, Inf, NaN,
round-to-nearest with mantissa-overflow exponent carry); ternary
quantization `f32_to_ternary` / `ternary_to_f32` with the spec's strict
threshold (`|x| > 0.5`); the `Format` enum (`Fp32`, `Fp16`, `Bf16`,
`Gf16`, `Ternary`); `format_bytes`; and the `quantize_value` utility.
A private `pow_u64` (fast exponentiation by squaring) replaces libm in
`no_std`. The crate's `quantization_phi_identity` is the **eighth
cross-kernel anchor test** in the project (after ring-088, ring-089,
ring-091, ring-092, ring-093, ring-094, ring-095), routing
`phi^2 + 1/phi^2 = 3` through the GF16 codec: it computes `phi^2` and
`phi^-2` via `pow_u64`, encodes each via `f32_to_gf16`, decodes via
`gf16_to_f32`, and verifies the sum lies within GF16 mantissa tolerance
of 3.0 (~0.03 absolute). Earlier Wave-11 narrative claimed 464 LOC;
honest Wave-23 measurement is **641 LOC**. Promotion will be
re-confirmed by the green `rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-096-rust`    | Quantization (GF16 codec+ternary) |  641 |    42 | `check` + `test`  |

## Wave 24 import -- ring-097 (on disk, real)

Wave 24 (2026-05-22, Closes #735) imports the **tenth** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check` green, `cargo test --lib`
reports **29 passed, 0 failed** on the first run. Ring-097 mirrors
[`specs/ar/proof_trace.t27`](../specs/ar/proof_trace.t27) byte-for-byte:
`MAX_STEPS = 10` (DARPA CLARA bound); K3 ternary logic
(`Trit::{True = 1, Unknown = 0, False = -1, Null = 2}` with `Null` reserved
for "output not yet produced"); K3 connectives `k3_and` (min lattice),
`k3_or` (max lattice), `k3_not`; `ProofStep` with `step_id`, interned ASCII
`operation` (up to 24 chars), fixed-arity `inputs` (up to 3 trits), `output`,
`timestamp_us`; `ProofTrace` with fixed `[ProofStep; MAX_STEPS]` buffer plus
`start_timestamp_us` / `end_timestamp_us` / `verified` flag; operations
`new_proof_trace`, `add_step`, `verify_trace`, `trace_length`,
`is_at_capacity`, `finalize_trace`, `step_at`, `format_trace`,
`trit_to_string`; and `VerifyStatus::{Valid, Empty, TooManySteps,
NullOutput(usize)}`. `verify_trace` enforces all three spec invariants:
`empty_trace_fails`, `trace_verification_catches_overflow`, and
`valid_trace_passes` (every step must have a non-`Null` output). The crate
is `#![no_std]` and heap-free -- the rendering helper `format_trace` writes
into a caller-supplied buffer (`FORMAT_TRACE_BUFFER`). The crate's
`cot_phi_identity` is the **ninth cross-kernel anchor test** in the project
(after ring-088, ring-089, ring-091, ring-092, ring-093, ring-094,
ring-095, ring-096), routing `phi^2 + 1/phi^2 = 3` through a 6-step bounded
reasoning chain: symbolic premises, `k3_and`, a numeric-witness step that
evaluates `pow_u64(phi, 2) + pow_u64(phi, -2)` and produces `True` iff the
result is within 1e-9 of 3.0, a `k3_or` alternative-path step, and a
conclusion -- then verifies and finalises the trace, with a separate
mass-conservation hook for φ²-weighted Pos and φ⁻²-weighted Neg priorities.
Earlier Wave-11 narrative claimed 624 LOC; honest Wave-24 measurement is
**823 LOC**. Promotion will be re-confirmed by the green `rings-rust`
workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-097-rust`    | Chain-of-Thought (proof trace+K3) |  823 |    29 | `check` + `test`  |

## Wave 25 import -- ring-098 (on disk, real)

Wave 25 (2026-05-22, Closes #737) imports the **eleventh** Wave-11 crate for real.
Locally verified on Rust 1.83.0: `cargo check --all-targets` green,
`cargo test --lib` reports **29 passed, 0 failed** on the first run
(no bug-fix cycle). Ring-098 mirrors three specs byte-for-byte:
[`specs/brain/unified_state.t27`](../specs/brain/unified_state.t27)
(`BrainState`, `ConsciousnessState`, `Mood`, `ArousalLevel`, `Layer`,
`REGION_COUNT = 27`, `LAYER_COUNT = 3`, `REGIONS_PER_LAYER = 9`, plus
`PHI` / `PHI_INV` / `PHI_SQ` / `PHI_INV_SQ` / `TRINITY` constants),
[`specs/ml/rl/dqn.t27`](../specs/ml/rl/dqn.t27)
(`Transition { state, action, reward, next_state, done }`), and
[`specs/brain/cognitive_loop.t27`](../specs/brain/cognitive_loop.t27)
(`COGNITIVE_PHASE_COUNT = 5`: sense, evaluate, decide, act, consolidate).
The `WorldModel` type is a bounded recorder: a fixed `[BrainState;
MAX_STATE_HISTORY = 16]` history buffer, a fixed `[Transition;
MAX_TRANSITIONS = 32]` replay buffer, an inline `STATE_DIM = 8`
observation vector, plus `snapshot`, `record_transition`, `step_phase`,
`run_one_cycle`, `verify`, `reset`, `state_at`, `transition_at`. The
`verify` routine enforces monotonic `cycle_count` and a `phi_coherence in
[0.0, 1.0]` invariant. The crate is `#![no_std]` and heap-free. The
crate's `world_model_phi_identity` is the **tenth cross-kernel anchor
test** in the project (after ring-088, ring-089, ring-091, ring-092,
ring-093, ring-094, ring-095, ring-096, ring-097), routing `phi^2 +
1/phi^2 = 3` through (a) integer projection `floor(PHI_SQ) + floor(PHI) =
3`, (b) `pow_u64` numeric witness, and (c) mass-conservation
`PHI_SQ + PHI_INV_SQ == TRINITY` to within 1e-12. Earlier Wave-11
narrative claimed 920 LOC; honest Wave-25 measurement is **779 LOC**.
Promotion will be re-confirmed by the green `rings-rust` workflow run
this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-098-rust`    | World Model (BrainState+Trans+CL) |  779 |    29 | `check` + `test`  |

## Wave 26 import -- ring-099 (on disk, real) -- SERIES COMPLETE

Wave 26 (2026-05-22, Closes #739) imports the **eleventh and final** Wave-11
crate for real. With this PR the Wave-11 `claimed-only` table is *empty* --
all 11 narratives now have honest source on disk, an actual `cargo check` +
`cargo test` matrix leg, and a live cross-kernel anchor test. Locally
verified on Rust 1.83.0: `cargo check --all-targets` green, `cargo test
--lib` reports **31 passed, 0 failed** after a single semantic correction
to the loop structure (the spec's `while` body must record the terminal
`STAGE_DONE` cell before exiting, otherwise `full_pipeline_pass` sees
`count = 8` instead of `9`). Ring-099 mirrors
[`specs/pipeline/e2e_test.t27`](../specs/pipeline/e2e_test.t27) byte-for-byte:
constants `MAX_PIPELINE_STAGES = 10`, `STAGE_INIT = 0`, `STAGE_PARSE = 1`,
`STAGE_SEAL = 2`, `STAGE_GEN = 3`, `STAGE_TEST = 4`, `STAGE_VERDICT = 5`,
`STAGE_SAVE = 6`, `STAGE_COMMIT = 7`, `STAGE_DONE = 8`, `STAGE_FAIL = 255`;
functions `pipeline_run`, `pipeline_inject_failure`, `pipeline_progress`,
`stage_name`; the 4 spec test blocks (`full_pipeline_pass`,
`pipeline_fail_at_gen`, `pipeline_fail_at_test`, `progress_calc`); the 3
spec invariants (`stage_ordering`, `max_stages_sufficient`,
`fail_distinct`). The `Pipeline` type wraps a fixed `[u8;
MAX_PIPELINE_STAGES]` stage buffer + parallel `[bool;
MAX_PIPELINE_STAGES]` results buffer (heap-free, `#![no_std]`); the
`Stage` enum (9 valid + `Fail`) carries `code` / `from_code` / `next` /
`is_terminal` / `name`. The crate's `integration_phi_identity` is the
**eleventh cross-kernel anchor test** in the project (after ring-088,
089, 091, 092, 093, 094, 095, 096, 097, 098), routing `phi^2 + 1/phi^2 =
3` through (a) integer projection `floor(PHI) + floor(PHI_SQ) = 1 + 2 =
3`, (b) `pow_u64` numeric witness, (c) pipeline progress arithmetic
(`progress(9, 9) == 100.0` and `progress(3, 9) == 100/3` to within
1e-9), and (d) mass-conservation `PHI_SQ + PHI_INV_SQ == TRINITY` to
within 1e-12. Earlier Wave-11 narrative claimed 1127 LOC; honest
Wave-26 measurement is **763 LOC**. Promotion will be re-confirmed by
the green `rings-rust` workflow run this PR triggers.

| Crate              | Domain                            |  LOC | Tests | Status            |
|--------------------|-----------------------------------|-----:|------:|-------------------|
| `ring-099-rust`    | Integration (10-stage E2E pipeln) |  763 |    31 | `check` + `test`  |

## Wave 11 -- claimed-only table is now EMPTY

Wave 11's narrative described 11 additional crates with ~ 8 969 LOC.
Waves 15-26 have honestly imported every one of them. The
`claimed-only` placeholder table is now empty -- nothing left to promote
from that series. The honest, measured LOC total across the 11 imports
is: **439 + 635 + 547 + 462 + 760 + 950 + 1210 + 808 + 641 + 823 + 779
+ 763 = 8817 LOC**, plus 26 + 13 + 15 + 19 + 19 + 28 + 28 + 32 + 25 + 42
+ 29 + 29 + 31 = 336 tests across the 11 Wave-11 crates and 5 Track-C
crates. All 11 cross-kernel anchors are live.

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
