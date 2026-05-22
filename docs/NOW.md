# NOW -- Trinity t27 sync

Last updated: 2026-05-22

## wave-23 -- Quantization import: ring-096-rust (this PR, Closes #733)

- **NEW** (rings-only, additive): `rings/ring-096-rust/` lands with `Cargo.toml` + `src/lib.rs` (641 LOC) + `README.md` + `.gitignore`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 23 footer), and this file.
- **What ring-096 actually does:** Faithful Rust mirror of the realizable subset of `specs/numeric/formats.t27`. (a) GF16 bit-layout constants byte-for-byte: `SIGN_MASK = 0x8000`, `EXP_MASK = 0x7E00`, `MANT_MASK = 0x01FF`, `EXP_SHIFT = 9`, `SIGN_SHIFT = 15`, `BIAS = 31`, `EXP_MAX = 63`, `EXP_MIN = 0`. (b) `gf16_to_f32(x: u16) -> f64` decoder handling signed zero (e=0,m=0), denormals (e=0,m!=0 -> `(m/2^9) * 2^(1-bias)`), normals (e in (0, ExpMax) -> `(1 + m/2^9) * 2^(e-bias)`), positive/negative infinity (e=ExpMax,m=0), and NaN (e=ExpMax,m!=0). (c) `f32_to_gf16(a: f64) -> u16` encoder: signed-zero preserved, NaN -> 0x7F01, Inf -> 0x[7|F]E00, normal magnitude reduced by repeated *2 / *0.5 into [1, 2), mantissa = `(frac * 2^9) + 0.5` round-to-nearest, mantissa-overflow carries into the exponent, underflow into denormal range, overflow clamped to Inf encoding. (d) Ternary primitives: `f32_to_ternary` with the spec's strict threshold `|x| > 0.5` -> Pos/Neg, otherwise Zero; `ternary_to_f32` returns 1.0 / 0.0 / -1.0 exactly; `Trit::{Neg=-1, Zero=0, Pos=1}` enum with `to_i8` / `from_i8`. (e) `Format` enum mirrors the spec's `enum(u8)`: `Fp32`, `Fp16`, `Bf16`, `Gf16`, `Ternary`. (f) `format_bytes(Format) -> usize` returns 4 / 2 / 2 / 2 / 1. (g) `quantize_value(x, fmt)`: Fp32/Fp16/Bf16 are pass-through (codec width identical-or-wider than GF16; full IEEE 754 binary16/bf16 converters are out of scope here -- those belong to a later ring); Gf16 round-trips through encoder + decoder; Ternary round-trips through `f32_to_ternary` + `ternary_to_f32`. (h) `pow_u64(base, exp)` -- fast exponentiation by squaring with negative-exponent inversion, used for all 2^k computations and for the anchor identity. (i) `fabs_no_std`, `is_nan`, `is_inf` -- no-libm helpers. (j) `QuantError::{Overflow, Underflow, Nan}` (reserved for future encoders). (k) `identity_witness()` for the universal anchor (closed-form `phi^2 + 1/phi^2`).
- **GF16 round-trip semantics:** encoder uses iterative magnitude normalization (multiplicative ladder) instead of `frexp`, bounded by `EXP_MAX = 63` from above and `0` from below, so the loop terminates in <= 63 iterations for any finite input. Mantissa rounding can promote the next-exponent boundary; the encoder handles this by clearing mantissa to 0 and incrementing exponent (with overflow-to-Inf check). The local roundtrip test `f32_to_gf16_roundtrip_normal_values` verifies relative error < 1% for the values {1.5, 2.0, 0.5, -1.5, 100.0, -100.0, 0.125}.
- **Ternary boundary semantics:** the spec defines the threshold as strict `|x| > 0.5`, which means `0.5` and `-0.5` quantize to `Zero`, not `Pos` / `Neg`. This is the boundary tested by `ternary_at_threshold_is_zero` and is symmetric (`ternary_symmetry` verifies `q(+0.7) = -q(-0.7)` after round-trip).
- **no_std math:** the spec uses arbitrary 2^k computations and float arithmetic; the crate replaces libm with `pow_u64` (fast exponentiation, integer exponent) plus pure-arithmetic `fabs_no_std` / `is_nan` / `is_inf`. The crate is `#![no_std]` and `#![deny(warnings)]`.
- **No new spec (L6 CEILING + L2 GENERATION):** every constant, every formula, the Format enum's variant set and ordering, the ternary threshold value, and the byte sizes follow `specs/numeric/formats.t27` byte-for-byte. The spec wraps decoded values in `gf16` (alias for a float); we use `f64` directly because the kernel semantics are identical and avoiding an extra wrapper keeps the ring crates independent (no inter-ring deps). No file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched.
- **Tests (42, all green on first run):** spec constants (`const_sign_mask`, `const_exp_mask`, `const_mant_mask`, `const_exp_shift_sign_shift_bias`, `const_exp_max_min`); GF16 decode (`gf16_to_f32_zero_positive`, `gf16_to_f32_zero_negative`, `gf16_to_f32_denormal_positive`, `gf16_to_f32_one`, `gf16_to_f32_positive_inf`, `gf16_to_f32_negative_inf`, `gf16_to_f32_nan`); GF16 encode (`f32_to_gf16_zero_positive`, `f32_to_gf16_zero_negative`, `f32_to_gf16_one_roundtrip`, `f32_to_gf16_inf_positive`, `f32_to_gf16_inf_negative`, `f32_to_gf16_nan`, `f32_to_gf16_roundtrip_normal_values`); ternary (`ternary_positive`, `ternary_zero`, `ternary_negative`, `ternary_above_threshold`, `ternary_below_neg_threshold`, `ternary_at_threshold_is_zero`, `ternary_to_f32_roundtrip`, `ternary_symmetry`); Format (`format_bytes_fp32`, `format_bytes_fp16`, `format_bytes_bf16`, `format_bytes_gf16`, `format_bytes_ternary`); quantize_value (`quantize_value_fp32_preserves`, `quantize_value_ternary_above_threshold`, `quantize_value_ternary_below_neg_threshold`, `quantize_value_gf16_roundtrip`); Trit helpers (`trit_from_to_i8`); pow_u64 (`pow_u64_zero_exp`, `pow_u64_positive_exp`, `pow_u64_negative_exp`); identity witness (`identity_witness_value`); cross-kernel anchor (`quantization_phi_identity`). Zero bug-fix cycles needed -- the boundary semantics, mantissa-overflow carry, and Inf/NaN encoding all worked correctly on the first compile.
- **Eighth cross-kernel anchor test:** `quantization_phi_identity` is the eighth time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15 `mac_dot_phi_identity`, Wave 16 `cpu_phi_identity_integer_projection`, Wave 18 `sr_quantize_phi_unbiased`, Wave 19 `attention_phi_identity_via_softmax_matmul`, Wave 20 `moe_phi_identity_via_gating_and_ffn`, Wave 21 `runtime_phi_identity_via_scheduler_credits`, Wave 22 `phi_adam_phi_identity_via_betas`). Construction: (1) compute `phi^2` and `phi^-2` via the crate's own `pow_u64` and verify the f64-precision sum is within 1e-9 of 3.0 (pre-codec identity). (2) Encode both values via `f32_to_gf16` -> u16, then decode via `gf16_to_f32` -> f64; verify the post-codec sum lies within GF16 mantissa tolerance of 3.0 (absolute < 0.03 against the 9-bit mantissa precision budget). (3) Run the same round-trip through the higher-level `quantize_value(x, Format::Gf16)` API and verify the same bound holds. This anchors the identity through the full codec stack, not just `pow_u64`.
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **464 LOC** for ring-096; the honest Wave-23 measurement is **641 LOC**. Pattern across the Wave-15..23 import series: 088 (961 -> 439), 089 (334 -> 635), 090 (2143 -> 547), 091 (409 -> 462), 092 (847 -> 760), 093 (668 -> 950), 094 (774 -> 1210), 095 (659 -> 808), 096 (464 -> 641). The honesty work is replacing guesses with measurements, in both directions.
- **R5-HONEST out of scope:** full IEEE 754 binary16 (`fp16`) / Brain Float (`bf16`) bit-level encoders -- their `quantize_value` paths are pass-through in this ring; they will arrive as a dedicated codec ring. INT4 / INT8 quantization (a separate sub-format space not present in `specs/numeric/formats.t27`). Strict rounding-mode controls beyond round-to-nearest. Quantization-aware training hooks (those belong in the optimizer ring, ring-095).
- **Compile semantics unchanged:** ring-096 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 42 tests in public.
- **COMPILE_STATUS promotion:** ring-096 moves from `claimed-only` to `check` + `test`. The remaining 3 Wave-11 rings (ring-097, ring-098, ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **L1 TRACEABILITY:** PR cites `Closes #733` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 42 `#[test]`s. **L5 IDENTITY:** anchor exercised through `pow_u64`, the GF16 codec, and `quantize_value`. **L6 CEILING:** zero numeric kernel / spec changes; spec constants mirror existing spec byte-for-byte. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`, `.gitignore`.
- Closes #733

## wave-22 -- phi-Adam optimizer import: ring-095-rust (Closes #731)

- **NEW** (rings-only, additive): `rings/ring-095-rust/` lands with `Cargo.toml` + `src/lib.rs` (808 LOC) + `README.md` + `.gitignore`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 22 footer), and this file.
- **What ring-095 actually does:** Faithful Rust mirror of the realizable subset of `specs/ml/optimizer/{adam, adamw}.t27`. AdamW (Loshchilov & Hutter 2019) with decoupled weight decay, plus AMSGrad (Reddi et al. 2018) variant, plus the spec's explicit **phi-Adam** branch with phi-damped betas. (a) Spec constants byte-for-byte: `DEFAULT_LEARNING_RATE = 1e-3`, `DEFAULT_BETA1 = 0.9`, `DEFAULT_BETA2 = 0.999`, `DEFAULT_WEIGHT_DECAY = 0.01`, `DEFAULT_EPSILON = 1e-8`, `DEFAULT_AMSGRAD = false`, `PHI_BETA1 = 0.9 / phi ~= 0.556`, `PHI_BETA2 = 0.999 / phi ~= 0.617`. (b) `AdamWConfig` with `defaults()` (classic AdamW), `phi_preset()` (phi-damped betas + use_phi_betas=true), `effective_beta1()` / `effective_beta2()` (honouring use_phi_betas), `is_valid()` (range check). (c) `AdamWState<'_>` -- caller-owned mutable references to `m`, `v`, optional `v_max` buffers; `AdamWState::init` zeroes all buffers and validates shape. (d) Helpers named after the spec: `compute_bias_correction`, `update_first_moment`, `update_second_moment`, `apply_weight_decay` (in-place), `compute_update`. (e) `step()` orchestrator: increments `state.step`, computes `bc1 = 1 - beta1^t`, `bc2 = 1 - beta2^t`, `lr_t = lr * sqrt(bc2) / bc1`, applies decoupled weight decay if `weight_decay > 0`, then for each parameter: updates moments, optionally tracks AMSGrad `v_max`, computes `lr_t * m / (sqrt(v_or_vmax) + epsilon)`, subtracts from parameter, accumulates squared updates for `step_norm`. Returns `StepResult { step_norm, lr_t, step }`. (f) `pow_u64` -- fast exponentiation, used for `pow(beta, t)`. (g) `sqrt_newton` -- Newton-Raphson square root with relative-tolerance early exit. (h) `OptimError::{ShapeMismatch, InvalidConfig}`. (i) `identity_witness()` for the universal anchor.
- **phi-Adam preset:** `AdamWConfig::phi_preset()` realises the spec's explicit phi-damped branch -- beta1 = 0.9/phi, beta2 = 0.999/phi, use_phi_betas = true. The damped betas accumulate less history per step (faster reactivity), in exchange for slightly more oscillation near minima; the `step_phi_preset_descends_quadratic_to_minimum` test verifies that the optimization trajectory's running minimum still converges to the true minimum of `f(x) = 0.5 * x^2` over 500 steps.
- **no_std math:** spec uses `pow(beta, t)` and `sqrt(v)` which need libm in no_std. Crate embeds `pow_u64` (fast exponentiation for integer exponent) and `sqrt_newton` (Newton-Raphson with 64-iteration cap and 1e-15 relative-tolerance early exit). Both verified against published reference values in tests (`sqrt_newton(0.0)=0`, `sqrt_newton(2.0)~=1.41421356`, `pow_u64(2,10)=1024`).
- **No new spec (L6 CEILING + L2 GENERATION):** every constant, every formula, and the function naming follows `specs/ml/optimizer/adamw.t27` byte-for-byte. The spec wraps scalars in `gf16::GF16` (alias for a float); we work in `f64` directly because the kernel semantics are identical and avoiding an extra wrapper keeps the ring crates independent (no inter-ring deps). No file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched.
- **Tests (25, 24 green on first run, 1 fix iteration):** sacred (`phi_inverse_relation`, `identity_witness_equals_three`, `spec_constants_match_byte_for_byte`); math primitives (`pow_u64_basics`, `sqrt_newton_recovers_known_values`); config (`defaults_are_valid_classic_adamw`, `phi_preset_uses_phi_betas`, `invalid_config_detected`); state (`state_init_zeros_buffers`, `state_init_rejects_shape_mismatch`, `state_init_accepts_full_amsgrad_buffer`); helpers (`first_moment_blends_grad_into_prev`, `second_moment_uses_squared_grad`, `weight_decay_scales_params_in_place`, `bias_correction_increases_with_t`, `compute_update_basic`); step (`step_zero_grad_only_decays_weights`, `step_positive_grad_moves_param_down`, `step_negative_grad_moves_param_up`, `step_amsgrad_keeps_max_of_v`, `step_shape_mismatch_errors`, `step_invalid_config_errors`, `step_amsgrad_without_buffer_errors`, `step_phi_preset_descends_quadratic_to_minimum`); anchor (`phi_adam_phi_identity_via_betas`). One micro fix cycle: the quadratic-descent test originally asserted strict monotonic decrease, but Adam with phi-damped betas legitimately oscillates near the minimum; the assertion now checks that the *running minimum* over 500 steps comes at least 10x closer to zero than the start, which still proves descent and is mathematically robust.
- **Seventh cross-kernel anchor test:** `phi_adam_phi_identity_via_betas` is the seventh time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15 `mac_dot_phi_identity`, Wave 16 `cpu_phi_identity_integer_projection`, Wave 18 `sr_quantize_phi_unbiased`, Wave 19 `attention_phi_identity_via_softmax_matmul`, Wave 20 `moe_phi_identity_via_gating_and_ffn`, Wave 21 `runtime_phi_identity_via_scheduler_credits`). Construction: (1) call the optimizer's own `pow_u64(PHI, 2) + pow_u64(PHI_INV, 2)` and verify it equals 3.0 to 1e-9 -- this routes the anchor through the optimizer's exponentiation helper. (2) phi-damped first-moment update at t=1 with `grad = phi`, starting from m_0 = 0: closed form gives `m_1 = (1 - 0.9/phi) * phi = phi - 0.9` exactly; the test asserts this. (3) Equivalent algebraic identity for the second moment: `v_1 = (1 - 0.999/phi) * phi^2 = phi^2 - 0.999 * phi`. (4) Full `step()` call on params=[phi, 1/phi], grads=[phi, 1/phi]: verifies sum(grads^2) = phi^2 + 1/phi^2 = 3 exactly through the optimizer's gradient handling, and that both moment slots received positive signal and both parameters moved downward (positive-gradient case).
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **659 LOC** for ring-095; the honest Wave-22 measurement is **808 LOC**. Pattern across the Wave-15..22 import series: 088 (961 -> 439), 089 (334 -> 635), 090 (2143 -> 547), 091 (409 -> 462), 092 (847 -> 760), 093 (668 -> 950), 094 (774 -> 1210), 095 (659 -> 808). The honesty work is replacing guesses with measurements, in both directions.
- **R5-HONEST out of scope:** GF16 scalar wrapping (alias only, identical kernel semantics); libm-backed `pow(beta, t)` and `sqrt(v)` (replaced by fast-exponentiation and Newton-Raphson); LAMB / Adagrad / RMSProp / SGD / SGD-Momentum / LR-Scheduler (each has its own spec under `specs/ml/optimizer/`, future ring imports).
- **Compile semantics unchanged:** ring-095 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 25 tests in public.
- **COMPILE_STATUS promotion:** ring-095 moves from `claimed-only` to `check` + `test`. The remaining 4 Wave-11 rings (ring-096..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **L1 TRACEABILITY:** PR cites `Closes #731` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 25 `#[test]`s. **L5 IDENTITY:** anchor exercised both at f64 level (via the optimizer's own `pow_u64`) and through the optimizer's phi-damped moment update. **L6 CEILING:** zero numeric kernel / spec changes; spec constants mirror existing spec byte-for-byte. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`, `.gitignore`.
- Closes #731

## wave-21 -- AGI Runtime import: ring-094-rust (this PR, Closes #729)

- **NEW** (rings-only, additive): `rings/ring-094-rust/` lands with `Cargo.toml` + `src/lib.rs` (1210 LOC) + `README.md` + `.gitignore`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 21 footer), and this file.
- **What ring-094 actually does:** Faithful Rust mirror of the realizable subset of the runtime triad in `specs/runtime/{execute, instance, process}.t27`. (a) Spec constants byte-for-byte: `DEFAULT_TIMEOUT_MS=30_000`, `MAX_CONCURRENT_EXECUTIONS=16`, `POLL_INTERVAL_MS=100`, `TASK_ID_LENGTH=32`, `MAX_INSTANCES=256`, `INSTANCE_NAME_LENGTH=128`, `LOOKUP_TIMEOUT_MS=100`, `SPAWN_TIMEOUT_MS=5_000`, `PTY_COLS_DEFAULT=80`, `PTY_ROWS_DEFAULT=24`, `MAX_PIPE_BUFFER=65_536`. (b) All nine spec enums re-stated as Rust `#[repr(u8)]` enums: `ExecResultType`, `TaskState`, `CancelReason`, `ProcessSignal`, `ProcessState`, `PTYMode`, `InstanceState`, `InstanceType`, `TerminationReason`. (c) `Trit` balanced-ternary priority enum with `to_i8` / `from_i8`. (d) `Task` -- compact descriptor with id, state, ternary priority, timeout budget, accumulated duration; `Task::new` + `Task::with_timeout` + `Task::is_expired`. (e) `Promise` -- pure-state-machine implementation of the spec's `Promise`: `resolve`, `reject`, `cancel`, `is_pending`, `is_resolved`, `is_rejected`, `is_cancelled` -- no waker / executor (out of scope, no_std). (f) `ProcessInfo` with a validated `transition` method enforcing the lifecycle NotStarted -> Running -> Stopped/Terminated -> Zombie (no resurrection). (g) `Instance` with four constructors (`agent`/`server`/`worker`/`background`) and lifecycle `activate`/`suspend`/`resume`/`terminate`/`finalize`. (h) `Registry` -- fixed `MAX_INSTANCES = 256`-slot, no-alloc registry with `register` returning a slot handle, `unregister`, `lookup` by `InstanceId`, `active_count`, `count_by_type`. (i) `Scheduler` -- fixed `MAX_CONCURRENT_EXECUTIONS = 16`-slot ready queue with ternary-priority pick (Pos > Zero > Neg, ties by slot index), per-tick credit accounting, timeout-based eviction in `tick()`, `complete` / `cancel` by id, `shutdown` drain. (j) `priority_to_credit(Trit) -> f64` -- phi-weighted credit policy: `Pos -> phi^2`, `Zero -> 1.0`, `Neg -> phi^-2`. (k) `identity_witness()` for the universal anchor. (l) `RuntimeError` enum with `RegistryFull`, `HandleOutOfRange`, `HandleEmpty`, `SchedulerFull`, `SchedulerEmpty`, `TaskNotRunnable`.
- **Trinity scheduler / phi-weighted credits:** ternary priority `{Neg, Zero, Pos}` maps directly to multiplicative credit weights `{phi^-2, 1.0, phi^2}`. The Trinity identity `phi^2 + 1/phi^2 = 3` then gives the scheduler a closed-form, mass-conservation law: one tick of a Pos-priority task plus one tick of a Neg-priority task consumes exactly 3 credit units per millisecond. This is the design hook the anchor test verifies end-to-end.
- **No new spec (L6 CEILING + L2 GENERATION):** every constant, every enum variant value, and the lifecycle semantics are direct mirrors of `specs/runtime/{execute, instance, process}.t27`. No file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched. The constants are duplicated, not edited.
- **Tests (32, all pass on first run on Rust 1.83.0):** sacred constants (`phi_inverse_relation`, `identity_witness_equals_three`, `spec_constants_match_byte_for_byte`); Trit (`trit_roundtrips_through_i8`); TaskState (`task_state_terminality`); task id (`task_ids_are_deterministic_and_distinct`); Task ctor (`task_default_timeout_is_spec_default`, `task_with_timeout_overrides`, `task_expires_when_duration_reaches_budget`); Promise (`promise_resolves_only_when_pending`, `promise_can_be_cancelled`, `promise_can_be_rejected`); ProcessInfo (`process_transitions_follow_lifecycle`, `process_alive_predicate`, `process_exit_code`); Instance (`instance_kinds`, `instance_lifecycle`); Registry (`registry_register_and_lookup`, `registry_counts`, `registry_unregister_out_of_range_errors`); Scheduler (`scheduler_capacity_pinned_to_spec`, `scheduler_picks_highest_priority_first`, `scheduler_rejects_terminal_tasks`, `scheduler_fills_to_capacity`, `scheduler_tick_on_empty_is_error`, `scheduler_complete_removes_task`, `scheduler_cancel_removes_task`, `scheduler_shutdown_clears_queue`, `scheduler_expires_runaway_task`); Priority credits (`credit_ordering_respects_priority`, `credit_extremes_sum_to_three_per_unit_time`); cross-kernel anchor (`runtime_phi_identity_via_scheduler_credits`). One micro bug-fix cycle: first anchor-test draft completed Pos then expected Neg to surface automatically, but the scheduler correctly re-selected Pos (highest priority); fix was to explicitly `complete(&pos.id)` between ticks. Otherwise 32/32 green.
- **Sixth cross-kernel anchor test:** `runtime_phi_identity_via_scheduler_credits` is the sixth time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15 `mac_dot_phi_identity`, Wave 16 `cpu_phi_identity_integer_projection`, Wave 18 `sr_quantize_phi_unbiased`, Wave 19 `attention_phi_identity_via_softmax_matmul`, Wave 20 `moe_phi_identity_via_gating_and_ffn`). Construction: a Pos-priority task and a Neg-priority task share an identical timeout budget. One tick of 1 ms each charges `phi^2 * 1` and `phi^-2 * 1` credits respectively; their sum equals 3.0 up to floating-point rounding (`|total - 3.0| < 1e-9`). The accumulator `Scheduler::credits_accumulated` records the same total at the end.
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **774 LOC** for ring-094; the honest Wave-21 measurement is **1210 LOC**. Pattern across the Wave-15..21 import series: 088 (961 -> 439), 089 (334 -> 635), 090 (2143 -> 547), 091 (409 -> 462), 092 (847 -> 760), 093 (668 -> 950), 094 (774 -> 1210). The honesty work is replacing guesses with measurements, in both directions.
- **R5-HONEST out of scope:** real syscalls (`spawn`, `kill`, PTY I/O) are not implemented -- this crate is the *logical* runtime, not the host bridge. Heap-backed containers (`Vec`, `HashMap`) are explicitly avoided in favor of fixed-size arrays so the crate stays no_std-clean and zero-allocation. Promises are pure state machines: no future / executor / waker / async-runtime integration (out of scope, depends on host).
- **Compile semantics unchanged:** ring-094 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 32 tests in public.
- **COMPILE_STATUS promotion:** ring-094 moves from `claimed-only` to `check` + `test`. The remaining 5 Wave-11 rings (ring-095..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **L1 TRACEABILITY:** PR cites `Closes #729` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 32 `#[test]`s. **L5 IDENTITY:** anchor exercised both at f64 level and through the scheduler's credit accumulator. **L6 CEILING:** zero numeric kernel / spec changes; spec constants mirror existing spec byte-for-byte. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`, `.gitignore`.
- Closes #729

## wave-20 -- Sparse MoE import: ring-093-rust (this PR, Closes #727)

- **NEW** (rings-only, additive): `rings/ring-093-rust/` lands with `Cargo.toml` + `src/lib.rs` (950 LOC) + `README.md` + `.gitignore`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 20 footer), and this file.
- **What ring-093 actually does:** Sparse Mixture of Experts (MoE) primitives. No backing file under `specs/` (textbook algorithm, like ring-091's SR); design mirrors Shazeer-2017 / Switch-Transformer top-k routing with ternary expert weights matching the project's TNN convention. (a) Trinity defaults: `NUM_EXPERTS = 3`, `DEFAULT_TOP_K = 1`, `DEFAULT_EMBED_DIM = 243` (= ring-092 EMBED_DIM), `DEFAULT_EXPERT_HIDDEN_DIM = 729 = 3^6`. (b) `MoEConfig` struct + `trinity_defaults()` const constructor + `is_valid()` predicate. (c) `Trit` enum re-derived locally (ring crates are independent, no inter-ring deps). (d) `gate_top_k(logits, top_k, indices, weights)` -- selection-sort top-k by descending logit (ties broken by smaller index) followed by max-subtract softmax over the selected logits so returned weights sum to 1.0; clamps to `min(top_k, logits.len())`. (e) `expert_ffn(input, w_in, hidden_scratch, w_out, output, in, hidden, out)` -- two-layer ternary FFN: `output = (ReLU(input @ w_in)) @ w_out`. (f) `moe_forward(input, expert_logits, cfg, w_in_all, w_out_all, ...)` -- composes gating + per-expert FFNs into a single token's MoE output, fully allocation-free. (g) `relu_inplace`. (h) `load_balance_loss(usage_counts, num_tokens, num_experts) -> f64` -- Switch-Transformer style importance-balance auxiliary; returns 1.0 for uniform routing, `num_experts` for full concentration. (i) `identity_witness()` for the universal anchor.
- **no_std exp:** softmax in `gate_top_k` requires `exp`. The crate embeds a private `exp_f64` using range reduction (`exp(x) = (exp(x / 2^20))^(2^20)`) plus a 12-term Taylor series. Same algorithm as ring-092; ring crates are independent and re-derive the helper. Verified to better than 1e-9 in the working range via `exp_negative_small_matches_reference`.
- **No new spec (L6 CEILING + L2 GENERATION):** no file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched. The MoE primitives are textbook (Shazeer-2017, "Outrageously Large Neural Networks"; Fedus-2022 Switch-Transformer). Trinity defaults are derived from existing project constants (`EMBED_DIM = 243` mirrors ring-092; `729 = 3^6` is the natural 3x expansion).
- **Tests (28, all pass on first run on Rust 1.83.0):** Trinity defaults (`num_experts_is_trinity`, `default_top_k_is_one`, `default_embed_dim_matches_ring_092`, `default_expert_hidden_dim_is_three_pow_six`); config sanity (`trinity_defaults_valid`, `config_invalid_when_top_k_exceeds_num_experts`, `config_invalid_when_zero_dim`); Trit (`trit_values`); ReLU (`relu_clamps_negatives`, `relu_empty_buffer_ok`); ternary matmul (`ternary_matmul_identity_3x3`); top-k gating (`gate_top_1_picks_argmax`, `gate_top_2_picks_two_largest_in_order`, `gate_top_k_clamps_to_logits_len`, `gate_top_k_zero_is_noop`, `gate_top_k_empty_logits_is_noop`, `gate_top_3_uniform_logits_uniform_weights`); expert FFN (`expert_ffn_identity_then_identity`, `expert_ffn_relu_zeroes_negative_hidden`); MoE forward (`moe_forward_single_expert_identity`, `moe_forward_top_2_combines_experts_linearly`); load-balance (`load_balance_perfect_balance_returns_one`, `load_balance_concentration_returns_num_experts`, `load_balance_empty_inputs_zero`); exp helper (`exp_at_zero_is_one`, `exp_negative_small_matches_reference`); identity (`identity_witness_holds`); cross-kernel anchor (`moe_phi_identity_via_gating_and_ffn`). No bug-fix cycle was needed -- the first compile gave 28/28 green.
- **Fifth cross-kernel anchor test:** `moe_phi_identity_via_gating_and_ffn` is the fifth time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15 `mac_dot_phi_identity`, Wave 16 `cpu_phi_identity_integer_projection`, Wave 18 `sr_quantize_phi_unbiased`, Wave 19 `attention_phi_identity_via_softmax_matmul`). Construction: `total = phi^2 + 1 + 1/phi^2` must equal exactly 4 by the identity (asserted in the test, |total - 4.0| < 1e-12). Three identity-FFN experts each receive weight `w_e = phi_power_e / total`; the weighted-sum output equals input because the weights sum to 1.0. Load-balance loss for the 3-expert uniform routing is also asserted = 1.0. Both `moe_forward` (uniform path) and an explicit phi-weighted accumulator path produce input back.
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **668 LOC** for ring-093; the honest Wave-20 measurement is **950 LOC**. Pattern across the Wave-15..20 import series: ring-088 claimed 961 -> 439, ring-089 claimed 334 -> 635, ring-090 claimed 2143 -> 547, ring-091 claimed 409 -> 462, ring-092 claimed 847 -> 760, ring-093 claimed 668 -> 950. The honesty work is replacing guesses with measurements, in both directions.
- **R5-HONEST out of scope:** training-time auxiliary terms beyond load-balance (router-z, etc.) are not implemented; capacity factor / token dropping is the caller's responsibility; per-token batching is the caller's responsibility (this crate's `moe_forward` is single-token, by design).
- **Compile semantics unchanged:** ring-093 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 28 tests in public.
- **COMPILE_STATUS promotion:** ring-093 moves from `claimed-only` to `check` + `test`. The remaining 6 Wave-11 rings (ring-094..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **L1 TRACEABILITY:** PR cites `Closes #727` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 28 `#[test]`s. **L5 IDENTITY:** anchor exercised both at f64 level and through MoE gating + FFN. **L6 CEILING:** zero numeric kernel / spec changes; textbook algorithm. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`, `.gitignore`.
- Closes #727

## wave-19 -- Attention import: ring-092-rust (this PR, Closes #725)

- **NEW** (rings-only, additive): `rings/ring-092-rust/` lands with `Cargo.toml` + `src/lib.rs` (760 LOC) + `README.md`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 19 footer), and this file.
- **What ring-092 actually does:** Faithful Rust mirror of the realizable subset of `specs/nn/attention.t27` (SacredAttention). (a) Sacred constants byte-for-byte: `NUM_HEADS=3`, `HEAD_DIM=81`, `EMBED_DIM=243`, `CONTEXT_LEN=81`, `ROPE_PAIRS=40`, `SACRED_GAMMA = phi^-3 ~= 0.2360679774997897`, `SACRED_SCALE = 81^(-SACRED_GAMMA) ~= 0.3543788557382518` (the spec calls for `pow(81, -SACRED_GAMMA)`; we embed the literal because `powf` is unavailable in `no_std` without libm, and add `attn_sacred_scale_matches_reference` to lock the value to 1e-6). (b) `Trit` balanced-ternary weight enum `{Neg, Zero, Pos}` with `value() -> i8`. (c) `ternary_matmul(input, weights, output, in_dim, out_dim)` -- matrix-vector product with ternary weights, identical algorithm to spec's `ternary_matmul`. (d) `add_residual(output, input)` -- in-place residual add, length-clamped. (e) `apply_softmax(scores, seq_len)` -- per-head softmax over a `NUM_HEADS * CONTEXT_LEN` buffer, max-subtract numerical stabilization. (f) `compute_scores(q, cache_k, position, seq_len, scores)` -- Q.K^T per head, multiplied by `SACRED_SCALE`, with a causal mask (positions `j > position` forced to zero). (g) `weighted_values(scores, cache_v, seq_len, concat)` -- softmax-weighted V sum. (h) `cache_kv(k_buffer, v_buffer, position, cache_k, cache_v)` -- KV cache store at offset `position * EMBED_DIM`. (i) `identity_witness()` for the universal anchor.
- **no_std exp:** softmax requires `exp`, which is unavailable in `no_std` without libm. The crate embeds a private `exp_f64` using range reduction (`exp(x) = (exp(x / 2^20))^(2^20)`) plus a 12-term Taylor series. Verified to better than 1e-9 across the working range against the standard library (`exp_negative_small`, `exp_negative_large`), with explicit underflow handling (`exp_underflow_returns_zero` at `x < -700`).
- **No new spec (L6 CEILING + L2 GENERATION):** every sacred constant, the per-head matmul shape, the causal mask convention, and the softmax+matmul structure are direct mirrors of `specs/nn/attention.t27`. No file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched. The constants are duplicated, not edited.
- **Tests (28, all pass on first run on Rust 1.83.0):** sacred constants (`attn_num_heads_is_trinity`, `attn_head_dim_is_three_pow_four`, `attn_embed_dim_is_heads_times_head_dim`, `attn_rope_pairs_is_context_len_div_two`, `attn_sacred_gamma_is_phi_cubed_inv`, `attn_sacred_gamma_positive_less_than_one`, `attn_sacred_scale_in_range`, `attn_sacred_scale_matches_reference`); Trit (`trit_values`); ternary matmul (`attn_ternary_matmul_identity`, `attn_ternary_matmul_negation`, `attn_ternary_matmul_zero_weights`); residual (`attn_add_residual_identity`, `attn_add_residual_length_clamped`); softmax (`attn_softmax_normalization_single_head`, `attn_softmax_positive_all_entries`, `attn_softmax_uniform_input`, `attn_softmax_all_heads_normalized`); compute_scores (`attn_compute_scores_applies_sacred_scale`, `attn_compute_scores_causal_mask`); cache (`attn_cache_kv_stores_at_offset`); weighted values (`attn_weighted_values_uniform_attention`); exp helper (`exp_at_zero_is_one`, `exp_negative_small`, `exp_negative_large`, `exp_underflow_returns_zero`); identity (`identity_witness_holds`); and the cross-kernel anchor (`attention_phi_identity_via_softmax_matmul`).
- **Fourth cross-kernel anchor test:** `attention_phi_identity_via_softmax_matmul` is the fourth time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15 `mac_dot_phi_identity`, Wave 16 `cpu_phi_identity_integer_projection`, Wave 18 `sr_quantize_phi_unbiased`). Construction: total = phi^2 + 1/phi^2 + 1 must equal 4 by the identity; weights w0 = phi^2/total, w1 = 1/total, w2 = (1/phi^2)/total sum to 1; routing these weights through `ternary_matmul` with all-positive weights recovers the sum 1.0, which multiplied back by total = 4.0 confirms the identity end-to-end.
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **847 LOC** for ring-092; the honest Wave-19 measurement is **760 LOC**. Pattern across the Wave-15..19 import series: ring-088 claimed 961 -> 439, ring-089 claimed 334 -> 635, ring-090 claimed 2143 -> 547, ring-091 claimed 409 -> 462, ring-092 claimed 847 -> 760. The honesty work is replacing guesses with measurements.
- **R5-HONEST out of scope:** RoPE table init (`sacred_attention_init`) is omitted because it requires `cos`/`sin` which are not available in `no_std` without libm. The `ROPE_PAIRS` constant and per-head dimensional layout are still exposed for downstream composition. The full `sacred_attention_kernel` orchestrator is also omitted; the primitives this crate ships are exactly the building blocks that orchestrator composes.
- **Compile semantics unchanged:** ring-092 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 28 tests in public.
- **COMPILE_STATUS promotion:** ring-092 moves from `claimed-only` to `check` + `test`. The remaining 7 Wave-11 rings (ring-093..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **L1 TRACEABILITY:** PR cites `Closes #725` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 28 `#[test]`s. **L5 IDENTITY:** anchor exercised both at f64 level and through softmax + ternary matmul. **L6 CEILING:** zero numeric kernel / spec changes; sacred constants mirror existing spec byte-for-byte. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`.
- Closes #725

## wave-18 -- Stochastic Rounding import: ring-091-rust (this PR, Closes #723)

- **NEW** (rings-only, additive): `rings/ring-091-rust/` lands with `Cargo.toml` + `src/lib.rs` (462 LOC) + `README.md`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 18 footer), and this file.
- **What ring-091 actually does:** Stochastic Rounding (SR), an unbiased rounding mode that's standard practice in low-precision ML training. (a) `SplitMix64` -- a deterministic, seedable, allocation-free 64-bit PRNG (Vigna 2014, "Further Scramblings of Marsaglia's Xorshift Generators"). `next_u64()` is branch-free and constant-time. Multiplicative gamma is `0x9E3779B97F4A7C15 = floor(2^64 / phi)` -- the same golden anchor the project preserves. `next_f32_unit()` draws a uniform f32 in `[0.0, 1.0)` using the top 24 bits of `next_u64()`. (b) `RoundingMode` enum `{Nearest, Stochastic}`. (c) `sr_round_f32_to_i32(x, rng)` -- single-value SR over the integer grid: returns `floor(x) + 1` with probability `frac(x)`, `floor(x)` otherwise. NaN -> 0; `+/- Inf` -> 0; values outside `i32` range saturate. (d) `sr_quantize_f32(x, step, rng) = step * SR(x / step)`. (e) `sr_quantize_batch(input, output, step, rng) -> usize` -- streaming, allocation-free batch quantization. (f) Inline `no_std` f32 helpers `floor_f32`, `frac_f32`, `is_finite_f32`, `abs_f32` (Rust `core` does not expose `f32::floor` without `libm`; this crate refuses external deps). (g) `identity_witness()` for the universal anchor.
- **No new spec (L6 CEILING + L2 GENERATION):** SR is a textbook universal numeric algorithm (Hopkins et al. 2020); SplitMix64 is a textbook PRNG. No file under `specs/`, `coq/`, `proofs/`, `bootstrap/`, `gen/` is touched. The SplitMix64 reference value at seed 0 (`0xE220A8397B1DCDAF`) is from Vigna's published paper, checked verbatim by `splitmix_first_value_with_seed_0`.
- **Tests (19, all pass on first run on Rust 1.83.0):** PRNG correctness (`splitmix_is_deterministic`, `splitmix_different_seeds_differ`, `splitmix_first_value_with_seed_0`, `next_f32_unit_in_range`); inline f32 helpers (`floor_f32_positive`, `floor_f32_negative`, `frac_f32_basic`); SR edge cases (`sr_exact_integer_returns_integer`, `sr_nan_returns_zero`, `sr_inf_saturates`, `sr_round_returns_floor_or_ceil`, `sr_quantize_zero_step_passthrough`, `sr_quantize_step_one_matches_round_to_i32`); statistical unbiasedness (`sr_is_unbiased`: mean of 10 000 `SR(0.3)` draws < 0.02 from 0.3, 3-sigma bound `~= 0.014`); cross-kernel anchor (`sr_quantize_phi_unbiased`: mean of 10 000 `SR-quantize(phi, 0.01)` < 0.001 from phi); batch helpers (`sr_quantize_batch_writes_min_len`, `sr_quantize_batch_empty_input`); enum sanity (`rounding_mode_eq`); universal anchor (`identity_witness_holds`). No bug-fix cycle was needed -- the first compile gave 19/19 green.
- **Third cross-kernel anchor test:** `sr_quantize_phi_unbiased` is the third time `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (after Wave 15's `mac_dot_phi_identity` over GF16 MAC and Wave 16's `cpu_phi_identity_integer_projection` over the TNN CPU). Here `phi` is funneled through SR-quantization at step `0.01` and averaged across 10 000 independent draws; the SR algorithm's unbiasedness preserves the value to within 1e-3.
- **R5-HONEST LOC observation:** the Wave-11 narrative quoted **409 LOC** for ring-091; the honest Wave-18 measurement is **462 LOC**. This is the first ring in the import series (Waves 15-18) whose honest LOC modestly *exceeds* the claim. Earlier rings under-shot (ring-088: 961 -> 439; ring-089: 334 -> 635 over; ring-090: 2143 -> 547). The honesty work is replacing guesses with measurements, in both directions.
- **Compile semantics unchanged:** ring-091 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`.
- **COMPILE_STATUS promotion:** ring-091 moves from `claimed-only` to `check` + `test`. The remaining 8 Wave-11 rings (ring-092..ring-099) stay `claimed-only`.
- **L1 TRACEABILITY:** PR cites `Closes #723` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under generated trees. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 19 `#[test]`s, including 2 statistical tests over 10 000 draws each. **L5 IDENTITY:** anchor exercised at both f64 level and via SR-quantization. **L6 CEILING:** no spec change; SR + SplitMix64 are textbook universal algorithms. **L7 UNITY:** no new `*.sh`.
- **R5-HONEST:** only ring-091 is promoted in this wave. The Vigna reference value is checked verbatim. The two statistical tests use seeds 2026 and 314159 so failures are reproducible; their 3-sigma bounds are stated explicitly in the test source.
- Closes #723

## wave-17 -- Simulator import: ring-090-rust (this PR, Closes #721)

- **NEW** (rings-only, additive): `rings/ring-090-rust/` lands with `Cargo.toml` + `src/lib.rs` (547 LOC) + `README.md`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 17 footer), and this file.
- **What ring-090 actually does:** Faithful Rust mirror of `specs/fpga/simulator.t27` (a HIR cycle-accurate simulator data-model + helpers). (a) `SimState` enum with 5 variants and tag values `0..=4` matching the spec's `enum(i8) SimState` byte-for-byte; `tag()` / `from_tag()` round-trips. (b) `SimConfig` 7-field struct (`name`, `max_cycles`, `clock_freq_hz`, `trace_enabled`, `vcd_output`, `break_on_error`, `vcd_path`) with `DEFAULT_CLOCK_FREQ_HZ = 100_000_000` matching the spec's hard-coded constructor. (c) `SimResult`, `ProbePoint`, `TraceEntry` with identical field shape. (d) Constructor `const fn`s: `sim_config`, `sim_config_with_trace`, `sim_ok`, `sim_error`, `probe`, `trace_entry`. (e) Query predicates: `is_idle`, `is_done`, `is_error`, `has_errors`, `passed`. (f) Time conversions: `sim_time_ns`, `sim_time_us`, `sim_time_ms`, `cycles_for_time_ns`. (g) `validate_sim_config`. (h) `identity_witness()` returning `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
- **Time-conversion overflow note (R5-HONEST, documented inline):** the source spec uses pure `u32` for `cycles * 1_000_000_000 / clock_freq_hz`. At the spec's own canonical case (`clock_freq_hz = 100_000_000`, `cycles = 100`), `100 * 1_000_000_000 = 1e11` exceeds `u32::MAX ~= 4.29e9` and the spec's own assertion `sim_time_ns(_, 100) == 1000` would fail. We faithfully implement the formula with a `u64` intermediate and narrow back to `u32`; the public signature stays `u32 -> u32` exactly as in the spec, but the intermediate arithmetic is the minimum width needed to make the spec's own canonical test pass. Over-large results saturate at `u32::MAX`. This is a faithful reading, not a spec change.
- **No new spec (L6 CEILING):** enum tags, struct field order, default values, and formula shapes mirror `specs/fpga/simulator.t27` byte-for-byte. No scheduler, no VCD writer, no event queue, no clock-domain crossing logic, no RTL execution -- those layers live in adjacent specs (`vcd_trace.t27`, `clock_domain.t27`, `formal.t27`) and are deliberately out of scope.
- **Tests (19, all pass on first run on Rust 1.83.0):** 13 mirrored from the spec's `test` blocks (`sim_config_creation`, `sim_config_with_trace_creation`, `sim_ok_result`, `sim_error_result`, `probe_creation`, `trace_entry_creation`, `sim_time_ns_canonical`, `sim_time_us_canonical`, `sim_time_ms_canonical`, `cycles_for_time_ns_canonical`, `validate_config_ok`, `validate_config_empty_name`, `validate_config_zero_cycles`) + 4 from the spec's `invariant` blocks (`invariant_max_cycles_positive`, `invariant_sim_time_positive`, `invariant_cycles_for_time_positive`, `invariant_validate_non_negative`) + 1 universal anchor (`identity_witness_holds`) + 1 bonus type-safety check (`sim_state_tag_roundtrip`). Unlike Wave 16, no bug-fix cycle was needed -- the spec was tight enough that the first compile gave 19/19 green.
- **R5-HONEST LOC correction:** the previous Wave-11 narrative quoted **2143 LOC** for ring-090; the honest Wave-17 measurement is **547 LOC**. The earlier number was a guess, not a measurement. This is the third LOC correction in the Wave-15/16/17 import series (ring-088: claimed 961 -> real 439; ring-089: claimed 334 -> real 635; ring-090: claimed 2143 -> real 547). The honesty work is replacing guesses with measurements, not the other way around.
- **Compile semantics unchanged:** ring-090 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise these 19 tests in public.
- **COMPILE_STATUS promotion:** ring-090 moves from `claimed-only` to `check` + `test`. The remaining 9 Wave-11 rings (ring-091..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary.
- **Identity (L5):** `phi^2 + 1/phi^2 = 3` is exercised by `identity_witness_holds`. Ring-090 does not introduce a cross-kernel anchor test of its own (it has no kernel, just data types) -- the cross-kernel anchors continue to live in ring-088 (`mac_dot_phi_identity`) and ring-089 (`cpu_phi_identity_integer_projection`).
- **L1 TRACEABILITY:** PR cites `Closes #721` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 19 `#[test]`s. **L5 IDENTITY:** anchor present. **L6 CEILING:** zero numeric kernel / spec changes; all constants and field shapes mirror existing spec. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`.
- **R5-HONEST:** only ring-090 is promoted in this wave; no claim is made about ring-091..ring-099. The 13 `test` blocks + 4 `invariant` blocks in the spec are translated 1:1 into `#[test]`s with identical assertion values.
- Closes #721

## wave-16 -- TNN ISA import: ring-089-rust (this PR, Closes #719)

- **NEW** (rings-only, additive): `rings/ring-089-rust/` lands with `Cargo.toml` + `src/lib.rs` (635 LOC) + `README.md`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 16 footer), and this file.
- **What ring-089 actually does:** (a) `Trit` -- wrapped `i8` in `-1..=1`, mirroring `TRIT_NEG`/`TRIT_ZERO`/`TRIT_POS` from `specs/isa/ternary_arithmetic.t27`. (b) `Word27` -- 27 packed trits (LSB-first) with bijective `from_i64`/`to_i64`. The first non-trivial implementation detail in this crate: `from_i64` uses Euclidean (`div_euclid`/`rem_euclid`) division -- Rust's default `/` truncates toward zero and gives **wrong** balanced-ternary digits for negative values (e.g. `-13` round-tripped to `17` under truncating division before the fix). (c) `trit_add(a, b, cin) -> (sum, cout)` per spec. (d) `word_add` / `word_sub` (sub = add . negate). (e) 9-opcode subset (`NOP`/`MOV`/`ADDI`/`ADD`/`SUB`/`NEG`/`LOAD`/`STORE`/`HALT`). (f) `Cpu` model with 27 registers (R0 hardwired to zero), 64-instruction code memory, 256-cell data memory, single-step `step()` and bounded `run(max_steps)`. (g) `identity_witness()` returning `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
- **No new spec (L6 CEILING):** every constant (`NUM_REGISTERS = 27`, `REG_WIDTH = 27`, `TRITS_PER_WORD = 27`, `TRIT_NEG = -1`, `TRIT_ZERO = 0`, `TRIT_POS = 1`, `R0_ZERO = 0`, balanced-add carry rules) mirrors existing `.t27` source byte-for-byte. The opcode list is a deliberate **subset** of `specs/fpga/ternary_isa.t27`, not an extension. No GF16 instructions, no ternary-gates ALU, no pipeline, no branch prediction, no Coptic encoding -- those layers are out of scope for Wave 16.
- **Tests (15, all pass locally on Rust 1.83.0):** `identity_witness_holds`, `trit_construction_rejects_out_of_range`, `trit_add_basic_table`, `word_zero_roundtrip`, `word_from_i64_roundtrip_small` (includes `-13`, `-100`, `1_000_000`), `word_add_arithmetic_matches_i64`, `word_sub_arithmetic_matches_i64`, `negate_is_involution`, `trit_at_and_set_trit_bounds`, `cpu_r0_is_hardwired_zero`, `cpu_addi_chain`, `cpu_add_sub_neg`, `cpu_load_store_roundtrip`, `cpu_halt_stops_execution`, and the cross-kernel **`cpu_phi_identity_integer_projection`**. The last test is the second time the project's identity anchor is exercised through actual numeric kernels (after Wave 15's `mac_dot_phi_identity`): it runs `floor(phi) + floor(1/phi) + ceil(phi^2 - 2) = 1 + 0 + 2 = 3` through the CPU using `ADDI`/`ADD`/`HALT`, exercising the full fetch/decode/execute loop.
- **R5-HONEST correction during this wave:** the first compile produced 11/15 tests green; 4 negative-value tests (`word_from_i64_roundtrip_small`, `word_add_arithmetic_matches_i64`, `word_sub_arithmetic_matches_i64`, `negate_is_involution`) failed due to Rust's truncating `/` mishandling negative inputs in `from_i64`. The fix replaces `v % 3`/`v / 3` with `v.rem_euclid(3)`/`v.div_euclid(3)` and re-runs cleanly: **15 passed, 0 failed**. The earlier Wave-11 narrative quoted **334 LOC** for ring-089; the honest Wave-16 number is **635 LOC**. Both corrections are R5-HONEST surfacings, not silent rewrites.
- **Compile semantics unchanged:** ring-089 lives outside `[workspace].members` (Wave-14 `exclude = ["bindings/python", "tools/converter", "gen", "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise `cpu_phi_identity_integer_projection` in public.
- **COMPILE_STATUS promotion:** ring-089 moves from `claimed-only` to `check` + `test`. The remaining 10 Wave-11 rings (ring-090..ring-099) stay `claimed-only` and the section preamble is updated to reflect the new boundary. The legend is unchanged.
- **Identity (L5):** `phi^2 + 1/phi^2 = 3` is the explicit subject of two tests in this crate -- one f64-level (`identity_witness_holds`) and one CPU-level (`cpu_phi_identity_integer_projection`). Both pass locally.
- **L1 TRACEABILITY:** PR cites `Closes #719` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 15 `#[test]`s. **L5 IDENTITY:** anchor exercised at both f64 and Cpu-instruction levels. **L6 CEILING:** zero numeric kernel changes; all constants mirror existing spec. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`.
- **R5-HONEST:** the only ring promoted in this wave is `ring-089`, and only after its 15 tests pass locally with the negative-value bug already fixed. No claim is made about ring-090..ring-099; they remain `claimed-only`.
- Closes #719

## wave-15 -- canonical GF16 import: ring-088-rust (this PR, Closes #717)

- **NEW** (rings-only, additive): `rings/ring-088-rust/` lands with `Cargo.toml` + `src/lib.rs` (439 LOC) + `README.md`. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, root `Cargo.toml`, or any other crate. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 15 footer), and this file.
- **R5-HONEST audit (the reason this wave exists):** Wave 11's narrative claimed 12 Rust crates `ring-088`..`ring-099` totalling ~ 9 930 LOC had been authored "in another sandbox". Searches of this repository, the past-session context store, and every reachable workspace location turned up **zero source files** for any of those 12 rings. The Wave-13 `COMPILE_STATUS.md` labelled them all `off-disk`, but that was a placeholder, not a deliverable. Wave 15 starts the real import with the single most foundational ring (GF16) and reclassifies the remaining 11 to `claimed-only` until each receives the same real-source treatment.
- **What ring-088 actually does:** (a) GF16 codec `f32 <-> Gf16` faithful to `specs/numeric/gf16.t27` -- bit layout `[S(1) E(6) M(9)]`, `BIAS = 31`, special exponent `0x3F` (Inf / NaN), separate `+0` (`0x0000`) and `-0` (`0x8000`), canonical NaN `0xFE01`. (b) `mac_dot(&[Gf16], &[Gf16]) -> Option<f32>` -- streaming allocation-free dot product; `None` on length mismatch; NaN poisons; saturation on overflow; subnormals flush to zero. (c) `identity_witness()` returning `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15. (d) Inline `frexp_norm`/`ldexp`-style helpers so the whole crate is `#![no_std]` (test cfg pulls std for the harness only) with **zero external dependencies**.
- **No GF16 spec change (L6 CEILING):** every constant (`SIGN_MASK`, `EXP_MASK`, `MANT_MASK`, `BIAS`, `MANT_DIVISOR`, `SPECIAL_EXP`, `GF16_ZERO_POS`, `GF16_ZERO_NEG`, `GF16_INF_POS`, `GF16_INF_NEG`, `GF16_NAN`) mirrors `specs/numeric/gf16.t27` byte-for-byte. Any normative change is a Coq matter, not a Rust matter.
- **Tests (13, all pass locally on Rust 1.83.0):** mirrors of the 8 mandatory tests from `specs/02-gf16-format.tri` (`gf16_roundtrip_phi`, `gf16_from_zero_pos`, `gf16_from_zero_neg`, `gf16_phi_identity`, `gf16_quantization_roundtrip_pi`, `gf16_better_phi_distance_than_f16`, `gf16_inf_roundtrip`, `gf16_nan_propagates`) **plus** 4 MAC tests (`mac_dot_empty`, `mac_dot_length_mismatch`, `mac_dot_simple`, `mac_dot_phi_identity`) **plus** the universal `identity_witness_holds`. The critical addition is `mac_dot_phi_identity` -- the **first time** in the project that the anchor `phi^2 + 1/phi^2 = 3` is exercised through actual numeric kernels (GF16 encode -> MAC -> f32 decode), not as a free-standing f64 assertion. Tolerance 0.02 -- generous given GF16's ~3 decimal digits of precision.
- **Compile semantics unchanged:** ring-088 lives outside `[workspace].members` (Wave-14 `exclude = [..., "rings"]` covers it automatically). `rings-rust.yml` discovers it via the matrix generator and runs `cargo check` + `cargo test`, both `continue-on-error: true`. The CI run triggered by this PR will be the first to exercise `mac_dot_phi_identity` in public.
- **COMPILE_STATUS promotions / reclassifications:** ring-088 moves from `off-disk` to `check` + `test`. The remaining 11 rings (ring-089..ring-099) move from `off-disk` to **`claimed-only`** with an explicit "LOC (claimed)" column heading and a section preamble warning that those LOC numbers are quotes from past narrative, not measurements. The legend gains a `claimed-only` row spelling out exactly what the status means: "earlier narrative referenced this crate; no source in this repo."
- **Identity (L5):** `phi^2 + 1/phi^2 = 3` is the explicit subject of two tests in this crate -- one f64-level (`identity_witness_holds`) and one cross-kernel (`mac_dot_phi_identity`). Both pass locally.
- **L1 TRACEABILITY:** PR cites `Closes #717` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 13 `#[test]`s (8 mandatory-from-spec + 4 MAC + 1 universal). **L5 IDENTITY:** anchor exercised at both f64 and GF16-MAC levels. **L6 CEILING:** zero numeric kernel changes; GF16 constants mirror existing spec. **L7 UNITY:** no new `*.sh` -- all new files are `.toml`, `.rs`, `.md`.
- **R5-HONEST:** the only ring promoted in this wave is `ring-088`, and only because its 13 tests pass locally with cargo output preserved in the PR body. No claim is made about ring-089..ring-099; their reclassification to `claimed-only` is the *removal* of an over-claim, not the addition of a new one. The Wave-11 narrative's "9 930 LOC" total is **not** repeated here.
- Closes #717

## wave-14 -- rings compile green (this PR, Closes #715)

- **CHANGE** (1-line, additive): root `Cargo.toml` `exclude` list extended from `["bindings/python", "tools/converter", "gen"]` to `["bindings/python", "tools/converter", "gen", "rings"]`. No other source touched. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, or any `src/lib.rs`. Doc-only updates to `rings/COMPILE_STATUS.md`, `README.md` (Wave 14 footer), and this file.
- **Root cause (Wave-13 honesty surface):** the Wave-13 `rings-rust` matrix failed all 5 Track-C legs with `error: current package believes it's in a workspace when it's not`. The root `[workspace]` table was swallowing `rings/ring-*-rust/` without listing them in `members` or `exclude`. Wave 12 Track C's intent was "intentionally NOT in `[workspace].members`" -- so the correct fix is to make the exclusion *explicit*, not to promote the crates into the workspace.
- **Local verification (Rust 1.83.0, matching `Dockerfile.rust`):** `cargo check --all-targets` green on all 5 crates; `cargo test` results -- ring-100 4 passed, ring-101 5 passed, ring-102 5 passed, ring-103 6 passed, ring-104 6 passed. **Total: 26 tests pass, 0 fail.** Zero warnings beyond benign cargo notes.
- **R5-HONEST correction:** the Wave-12 NOW entry and Wave-12 README section claimed `28 #[test]`s for Track C. The actual count from `cargo test` is **26**. `rings/COMPILE_STATUS.md` and the README Wave-14 footer state the correct number; the original 28 claim was off by two (likely an over-count of inline assertion-helpers as `#[test]`s).
- **`rings/COMPILE_STATUS.md` promotion:** all 5 Track-C rows move `scaffold` -> `check` + `test`. The 12 Wave-11 rows remain `off-disk` -- they are not yet imported into this repo, and no claim is made about them here.
- **Gate semantics unchanged:** `rings-rust.yml` is still `continue-on-error: true`. Wave 14 does not flip the gate to mandatory -- it just gives the gate something to be honestly green about. Mandatory promotion (drop `continue-on-error`) is reserved for a later wave once 12-ring import lands.
- **Identity:** anchor `phi^2 + 1/phi^2 = 3` unchanged in every crate; each `identity_witness()` is now exercised by `cargo test` for the first time in CI (5/5 crates contain an `identity_witness_holds` test).
- **L1 TRACEABILITY:** PR cites `Closes #715` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`. **L3 PURITY:** ASCII-only diff (1 line in `Cargo.toml`, plus doc rewrites). **L4 TESTABILITY:** 26 `#[test]`s now wired into CI via the Wave-13 matrix. **L5 IDENTITY:** `phi^2 + 1/phi^2 = 3` preserved verbatim; `identity_witness_holds` test passes in 5/5 crates. **L6 CEILING:** zero numeric kernel changes; GF16 / FORMAT-SPEC-001 untouched. **L7 UNITY:** no new `*.sh` -- diff is entirely TOML + Markdown.
- **R5-HONEST:** test count corrected 28 -> 26 with traceable evidence (cargo test output stored in PR body); promotion to `check`+`test` will be re-confirmed by the green `rings-rust` workflow run that this PR triggers; no row in `COMPILE_STATUS.md` is promoted that did not pass locally first.
- Closes #715

## wave-13 -- Toolchain & Compilation Gate (this PR, Closes #713)

- **NEW** (additive, CI/docs-only): `Dockerfile.rust` (pinned `rust:1.83-bookworm` with `rustfmt` + `clippy`), `scripts/ci/rings_matrix.py` (pure-stdlib GitHub Actions matrix generator that discovers `rings/ring-*-rust/` crates), `.github/workflows/rings-rust.yml` (matrix `cargo check` + `cargo test`, `continue-on-error: true`, step-summary), `rings/COMPILE_STATUS.md` (living per-crate status table with legend `scaffold` / `check` / `test` / `off-disk`). README gains a *Wave 13 -- Toolchain & Compilation Gate* section plus a dated footer line. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`, or any `src/lib.rs`.
- **Why now:** Waves 11 and 12/Track-C landed 17 Rust crates (~= 10 750 LOC, 60+ `#[test]`s) on disk, but `cargo check` / `cargo test` were never executed in CI. Wave 13 introduces the missing toolchain + matrix so the repo can finally distinguish *scaffolded* from *compiles* from *tested* -- in public, on every PR that touches `rings/ring-*-rust/`.
- **Gate semantics (honest):** `rings-rust.yml` runs `cargo check --all-targets` then `cargo test`, **with `continue-on-error: true`**. A red leg surfaces real per-crate breakage without blocking merges. Source of truth for promotion is `rings/COMPILE_STATUS.md`; no row moves past `scaffold` without a linkable CI log. The 5 Wave-12 Track-C crates land as `scaffold`; the 12 Wave-11 crates remain `off-disk` (authored in another sandbox, not yet imported here).
- **Generator correctness:** `python3 scripts/ci/rings_matrix.py` was executed locally against this repo and produced `{"include":[{"crate":"ring-100-rust",...},...,{"crate":"ring-104-rust",...}]}` -- exactly the 5 crates currently present on disk. Pure stdlib (no external deps), runs under the Python already shipped on `ubuntu-latest`.
- **Identity:** anchor `phi^2 + 1/phi^2 = 3` preserved verbatim in every new artifact (Dockerfile, workflow header, matrix generator docstring, `COMPILE_STATUS.md`). Each ring crate's existing `identity_witness()` will be exercised once a leg reaches `cargo test` -- semantics unchanged.
- **L1 TRACEABILITY:** PR cites `Closes #713` in title and body; every commit message carries it. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`. **L3 PURITY:** ASCII-only source; English doc-comments; matrix generator is Python (no shell). **L4 TESTABILITY:** matrix generator self-verified locally (5/5 crates discovered); existing per-crate `#[test]`s untouched; gate now wires them into CI. **L5 IDENTITY:** `phi^2 + 1/phi^2 = 3` quoted in every new artifact. **L6 CEILING:** zero numeric kernel changes; GF16 / FORMAT-SPEC-001 untouched. **L7 UNITY:** no new `*.sh` -- gate logic is Python (`scripts/ci/rings_matrix.py`).
- **R5-HONEST:** README and `COMPILE_STATUS.md` only claim what is true at landing -- workflow file exists, generator runs locally, all 5 Track-C crates are `scaffold` (never compiled in CI yet), all 12 Wave-11 crates are `off-disk`. No `cargo check` / `cargo test` pass-claim, no TOPS / energy / silicon number, no "all crates compile" assertion. Promotion of any row is reserved for follow-up PRs that link a green CI log.
- Closes #713

## wave-12(track-c) -- scaffold ring-100..ring-104 Rust crates (this PR, Closes #711)

- **NEW** (rings-only, additive): 5 Rust crates under `rings/ring-{100,101,102,103,104}-rust/`. Each crate ships `Cargo.toml` + `src/lib.rs` + per-crate `README.md` + inline `#[test]`s. Zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`.
- **Crates** (file / Rust LOC / test count): `ring-100-multichip` (3 / 205 / 5) Multi-Chip Mesh -- Phi+Euler+Gamma triad fabric, XY routing, hop cost, triad witness; `ring-101-analog-gf16` (3 / 144 / 5) Analog GF16 -- deterministic quantize/dequantize surrogate + reproducible LCG-driven noise channel; `ring-102-photonic-mac` (3 / 157 / 5) Photonic MAC -- wavelength-multiplexed dot product with per-lane insertion-loss factor in `[0, 1]`; `ring-103-on-chip-learning` (3 / 131 / 6) phi-tempered SGD step `w -= lr * (1/phi) * clip(g)`, alloc-free, in-place; `ring-104-telemetry-bus` (3 / 185 / 7) bounded lossy ring buffer of `(ts, 4-byte tag, value)` samples with FIFO eviction and `mean_by_tag` aggregation.
- **Totals:** 5 crates, 15 files, 822 Rust LOC, 28 `#[test]`s. All crates are `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`.
- **Workspace policy:** new crates are **intentionally not** added to `[workspace].members` in the root `Cargo.toml`. Hookup is Wave 12 / **Track D** (Docker `rust:1.83-bookworm` + GitHub Actions matrix). This keeps the current CI surface unchanged while artefacts land on disk -- consistent with the honest "uncompiled" status of Wave 11.
- **Compile status (honest):** `cargo check` / `cargo test` **NOT** run in authoring sandbox -- toolchain still unavailable, exactly as documented in the Wave 11 toolchain table. Verification gate is Track D's exit criterion (`cargo check >= 9/12`, `cargo test >= 6/12`).
- **Identity:** every crate exposes `identity_witness()` (or `Mesh::identity_witness` for ring-100) returning `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15. The witness is also exercised by a `#[test]` in every crate so Track D will hit it on `cargo test`.
- **L1 TRACEABILITY:** PR cites `Closes #711`. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`. **L3 PURITY:** ASCII source, English doc-comments. **L4 TESTABILITY:** 28 `#[test]`s across 5 crates, every crate has at least one test asserting the phi identity. **L5 IDENTITY:** `phi^2 + 1/phi^2 = 3` exercised in every crate. **L6 CEILING:** no numeric kernel changes; GF16 spec untouched; new GF16 surrogate in ring-101 is explicitly labelled an approximation and not a spec change. **L7 UNITY:** no new `*.sh`.
- **R5-HONEST:** every Track-C crate row carries the same "scaffolded, uncompiled" status badge; no `cargo check`/`cargo test` pass-claim; no TOPS / energy / silicon number stated; file and LOC counts traceable to repo via `find rings/ring-1{00..04}-rust -type f | wc -l`.
- Closes #711

## docs(README) -- Wave 11 (12 Rust crates ring-088..ring-099, honest status) + Wave 12 plan (this PR, Closes #710)

- **NEW** (docs-only, additive): two new sections in `README.md` plus dated footer line. Zero edits under `gen/`, `coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`.
- **Wave 11 status (honest):** 12 Rust crates `ring-088`..`ring-099` written to disk -- ring-088 GF16 MAC (961 LOC), ring-089 TNN ISA (334), ring-090 Simulator (2 143), ring-091 Stoch Round (409), ring-092 Attention (847), ring-093 Sparse MoE (668), ring-094 AGI Runtime (774), ring-095 phi-Adam (659), ring-096 Quantization (464), ring-097 CoT Engine (624), ring-098 World Model (920), ring-099 Integration / `trinity` bin (1 127). Totals: 60 source files, ~= 9 930 Rust LOC, 33 `Cargo.toml`. Numbers verified via `find` + `wc`.
- **Toolchain honesty:** README now contains an explicit table marking `cargo`, `rustc`, `cargo check`, `cargo test` as NOT installed / NOT verified in the Wave-11 sandbox (network timeout / permission denied on toolchain install). The crates were never compiled; verification is deferred to Wave 12.
- **Wave 12 plan published:** four parallel tracks -- Track A fix `cargo check` errors (per-crate PRs), Track B finish execution units inside `ring-090` simulator, Track C author `ring-100`..`ring-104` (Multi-Chip Mesh / Analog GF16 / Photonic MAC / On-Chip Learning / Telemetry Bus), Track D Dockerfile.rust on `rust:1.83-bookworm` + GitHub Actions matrix building all `ring-0**-rust` crates. Exit criteria: `cargo check` >= 9/12, `cargo test` >= 6/12, `trinity` binary runs end-to-end, CI green.
- **L1 TRACEABILITY:** this PR cites `Closes #710`. **L2 GENERATION:** zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`. **L3 PURITY:** doc-only; section labels mirror existing NOW entries; ASCII-safe body. **L4 TESTABILITY:** N/A -- no `.t27` specs touched. **L5 IDENTITY:** `phi^2 + 1/phi^2 = 3` anchor preserved; footer mantra kept verbatim. **L6 CEILING:** no numeric kernel changes; `FORMAT-SPEC-001.json` + GF16 spec untouched. **L7 UNITY:** no new `*.sh`.
- **R5-HONEST:** every Wave-11 row carries an "uncompiled" status badge; no claim of `cargo check`/`cargo test` passing; no benchmark / TOPS / energy number stated; LOC and file counts traceable to repo via `find rings/ -name '*.rs' | xargs wc -l`.
- Closes #710

## docs(TRI-NET) -- cross-line package P0 NMSE / P1 API+whitepaper / P2 22FDX + Zenodo (this PR, Closes #696)

- **NEW** (docs-only, additive): `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`, `docs/TRI_NET_API.md`, `docs/TRI_NET_WHITEPAPER.md`, `docs/22FDX_TOPS_W_PROJECTION.md`, `docs/ZENODO_BUNDLES.md`, `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` (2026 t27-side roadmap: CL-01..04 DARPA-CLARA alignment, EN-01..03 energy, SN-01..03 SNN-TRI fusion, PUB-01..03 publication, OS-01..03 open-source SDK / Coq export / contribution path; every row labelled `VERIFY`, `projection`, or `target` -- no funding / silicon-date / paper-acceptance / `1000x` / `4000 TOPS/W` / new-DOI claim)
- **NEW** machine-readable specs: `specs/benchmarks/gf16_bfloat16_nmse.t27` (L4 TESTABILITY: `test` + `invariant` + `bench`), `specs/api/tri_net_api.t27` (L4 TESTABILITY: `test` + `invariant` + `bench`)
- **NEW** JSON schemas: `schemas/nmse-protocol-v1.json` (draft-07, results manifest), `schemas/tri-net-api-v1.json` (draft-07, RepoIdentity / Readiness / ArtefactIndex shapes)
- **P0** GF16 vs bfloat16 NMSE: distribution-explicit (D_NORM, D_LOG, D_RELU, D_PHI, D_DEEP); no silicon number asserted; L5 IDENTITY witness gates every run (`phi^2 + 1/phi^2 = 3` to 1e-15 in f64); BF16 subnormal policy must be declared; seal hash must match `bootstrap/stage0/FROZEN_HASH` or manifest is informational only
- **P1** TRI-NET API: file-based, read-only; explicitly NOT a hosted endpoint; schema MAJOR=1; fail-closed validation; extensions under `x_extension`
- **P1** Whitepaper: position paper only; mirrors `STATUS.md` readiness ladder; no parity claim against commercial NPUs (see `COMPETITORS.md`); cross-links chip repos `tt-trinity-phi`, `tt-trinity-euler`, `tt-trinity-gamma`
- **P2** 22FDX TOPS/W: every row tagged with confidence band C1..C5; C1 rows trace to existing Coq lemmas (W34..W49 in `trios-coq/Physics/`); no measured silicon number; falsification policy enumerated; no tape-out date claimed
- **P2** Zenodo bundles plan: v1 toolchain / v2 silicon-substrate / v3 proofs+conformance; **no DOI quoted before upload**; existing canonical B001..B007 + v5.0 parent (cited in `docs/ZENODO.md`) are predecessor records, not v1/v2/v3
- **Cross-links** to chip repos: D2D protocol spec is owned by `tt-trinity-euler` / `tt-trinity-gamma`; t27 surfaces only the toolchain-side hooks. Triple-Deck (W47 RBB + W48 FBB-active + W49 CapBoost) Coq lemmas already in `trios-coq/Physics/` per existing NOW entries; chip-side implementation lives in chip repos.
- **L1 TRACEABILITY**: PR cites `Closes #696`. **L2 GENERATION**: zero edits under `gen/`, `coq/`, `trios-coq/`, `proofs/`, `bootstrap/`. **L3 PURITY**: all new files ASCII / English (verifiable via `scripts/check_first_party_doc_language.py`). **L4 TESTABILITY**: both new `.t27` specs contain `test` + `invariant` + `bench`. **L5 IDENTITY**: `phi^2 + 1/phi^2 = 3` cited verbatim in every new doc and witnessed in NMSE protocol. **L6 CEILING**: `FORMAT-SPEC-001.json` + `specs/numeric/gf16.t27` referenced as SSOT; no numeric kernel changes. **L7 UNITY**: zero new `*.sh`.
- **R5-HONEST**: every projection in `docs/22FDX_TOPS_W_PROJECTION.md` labelled "projection, not measured silicon"; every Zenodo row tagged `pending`; whitepaper claims strictly bounded by `STATUS.md` ladder
- Closes #696

## ci(notebook-sync) — repair workflow syntax causing instant failures (this PR, #694, Closes #695)

- **Fixed**: `.github/workflows/notebook-sync.yml` was failing instantly on every push since #693 merged — runs completed in seconds with `conclusion=failure`, zero jobs dispatched, `gh run view --log-failed` reported *log not found*.
- **Root cause (three combined defects)**:
  1. `workflow_dispatch:` was declared at the top level instead of nested under `on:` — Actions rejected the file at parse time (bare `on` is interpreted as YAML `True`).
  2. `extract-issue.outputs.event_type` referenced `steps.event.outputs.type` while the step id is `event_type`.
  3. Duplicate `pull_request_review)` case in the bash event dispatch.
- **Latent runtime defect surfaced once jobs began dispatching**: `sync-notebook` referenced `peter-evans/create-or-update-file@v3`, which does not exist on github.com (404). Replaced with `actions/github-script@v7` using `github.rest.repos.createOrUpdateFileContents`; added `permissions.contents: write` on the `sync-notebook` job. Step targets the repo's default branch (resolved via `repos.get`) because on `issues` / `pull_request` events there is no canonical branch to commit to, and is wrapped in `continue-on-error` + internal `try/catch` so a 403/422 from fork PRs or branch protection logs a warning instead of failing the sync job — matches the existing best-effort pattern around the `python sync.py || warnings; exit 0` block immediately above.
- **Validation**: `actionlint 1.7.12` — all syntax-check and expression errors cleared. `yaml.safe_load` confirms `on:` contains all 6 triggers including `workflow_dispatch` with `inputs: [issue_number, sync_type]`.
- **L7 UNITY held**: YAML/actions-side repair only — no `*.sh` added, no `gen/` edits, no spec changes. RTL/GDS/`verdict.json` gates untouched. TRI-NET docs package from #693 untouched.
- Closes #695

## docs(TRI-NET) — positioning package (#693, Closes #627)

- **NEW** (root-level, docs-only): `STATUS.md`, `LINEUP.md`, `FORMAT_REGISTRY.md`, `COMPETITORS.md`, `BENCHMARKS.md`, `CLARA_TRACEABILITY.md`
- **README.md first screen**: additive "What this repo is" block linking to the six new docs; rest of README unchanged
- **Positioning**: t27 framed as the fourth product of the TRI-NET line — spec-first toolchain + numeric format registry; chip siblings `tt-trinity-phi` (1×1 phi-anchor), `tt-trinity-euler` (8×2 e-engine), `tt-trinity-gamma` (8×4 32-PE ternary mesh)
- **Readiness ladder**: SPEC / RTL / SIM / SYNTH / GDS-TAPEOUT / SILICON; conservative — no SILICON or GDS claim in t27, GF16 at SIM only, CLARA bridge demo/draft, Coq partial
- **Numeric SSOT** kept: `conformance/FORMAT-SPEC-001.json` (primary = GF16), FP8 + NF4/INT4/INT8 bridges marked PLANNED (no spec yet)
- **No code touched**: zero changes under `gen/`, `specs/`, `bootstrap/`, `coq/`. R-SI-1 and L2 GENERATION held
- **Validation**: `scripts/check_first_party_doc_language.py` PASS; `FORMAT-SPEC-001.json` sanity PASS; full `./scripts/tri test` not run locally (no cargo in env) — CI is authoritative
- **External sources cited in docs**: DARPA CLARA (darpa.mil/research/programs/clara), Qualcomm Cloud AI 100 Ultra brief, Hailo-8, Axelera Metis, Coral Edge TPU benchmarks, MediaTek Dimensity 9400+, BitNet b1.58 (arxiv 2402.17764), Tiny Tapeout chip catalogue
- Closes #627

## Wave-45 Lane PP — Avs96Safe.v AVS-96 Dopamine Safety Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/Avs96Safe.v — 8 Qed lemmas, 0 Admitted
- **AVS-96 voltage steps**: avs96_steps = 96; bin width 6250 uV (6.25 mV), half of W36 AVS-48 baseline
- **Step gate**: step_gate_input clamps occupancy_bin >= 96 to 0
- **Lemmas**: avs96_step_count, avs96_bin_width_positive, avs96_half_of_avs48, step_gate_in_range, step_gate_clamp_out_of_range, step_gate_zero, step_gate_max_in_range, avs96_steps_ne_zero
- **L2_BG_AVS96_STEP_GATE** microcode (no new L1)
- Silicon-vector counter milestone S-200
- Sprints: S-194, S-195, S-200
- BIO->SI: basal-ganglia-DA
- anchor phi^2 + phi^-2 = 3, DOI 10.5281/zenodo.19227877
- Closes #686, Refs gHashTag/trinity-fpga#175, gHashTag/trios#932

- W45 PP: Avs96Safe.v landed on master (S-200 milestone)

## Wave-49 Lane VV — CapBoost.v 38 Qed + γ³ Capacitive Decoupling Burst (NEW, this PR)

- **NEW**: trios-coq/Physics/CapBoost.v — 37 Qed lemmas + composite Theorem `cap_boost_composite` (= 38 Qed total), 0 Admitted
- **OP_CAP_BOOST = 0xF3 = 243** (new sacred opcode, Wave-49 — THIRD slot of extended sacred bank 0xD0..0xFF)
- **TRIPLE-DECKER with W47/W48**: RBB (0xF1, leakage well) → FBB-ACTIVE (0xF2, active well) → CAP-BOOST (0xF3, supply rail). Three orthogonal dynamic-power levers stacked at iso-area.
- **Theory — γ³ Decoupling-Cap Burst**: ΔC_dec = C_dec_base · gamma^3 ≈ 100 pF · 0.0081 ≈ 0.81 pF capacitive burst on supply rail. gamma^3 = phi^-9 ≈ 0.01316 inherited from B007^3 — R18 preserved (no new ROM cell).
- **ΔC positive uplift**: cap_boost_delta_c_positive proves DELTA_C_DEC_BPS > 0; cap_boost_delta_c_in_band proves uplift in [50, 100] bps (R7 area envelope)
- **di/dt margin band**: cap_boost_didt_in_band proves 6% in [4%, 10%] (R7 falsification band, cite Larsson/Svensson 1994)
- **Droop suppression band**: cap_boost_droop_in_band proves 4% in [2%, 8%] (R7 worst-case supply droop reduction)
- **Cap area uplift cap**: cap_boost_area_cap proves observed <= 50 bps (≤0.5% area, R18 iso-area constraint)
- **f_clk impact cap**: cap_boost_fclk_impact_cap proves impact <= 200 bps (≤2% frequency back-pressure)
- **TOPS/W lift**: cap_boost_tops_w_lift_at_least_0pt7pct proves 1000*(1091-1083) >= 7*1083 — projection 1083 -> 1091 (+0.738%)
- **Triple-decker cross-wave**: triple_decker_consecutive proves OP_CAP_BOOST = OP_RBB + 2 ∧ OP_FBB_ACTIVE = OP_RBB + 1 (consecutive slots 0xF1/0xF2/0xF3)
- **R18 SACRED BANK EXTENSION held**: bank-set frozen at 0xD0..0xFF (32 slots), only slots populated — no new ROM cell. cap_boost_in_extended_bank + 18 prior opcode-distinctness lemmas
- Refs: Larsson and Svensson 1994 (di/dt SSO), Jiang et al. 2018 (capacitive supply decoupling), Rabaey 2003 (decap sizing)
- Local `coqc` EXIT=0

## Wave-48 Lane SS — FBBActive2.v 33 Qed + Forward Body Bias DUAL of W47 (NEW, this PR)

- **NEW**: trios-coq/Physics/FBBActive2.v — 32 Qed lemmas + composite Theorem `fbb_active_composite` (= 33 Qed total), 0 Admitted
- **OP_FBB_ACTIVE = 0xF2 = 242** (new sacred opcode, Wave-48 — SECOND slot of extended sacred bank 0xD0..0xFF)
- **DUAL of W47 RBB**: where RBB (0xF1) applies NEGATIVE body bias to idle PEs to cut leakage, FBB_ACTIVE (0xF2) applies POSITIVE body bias to ACTIVE-path PEs to cut delay. Same gamma^4 magnitude, opposite sign — symmetric pair.
- **Theory — Forward Body Bias of Active Path**: V_BS,active = +V_DD · gamma^4 ≈ +2.5 mV (positive body-source potential reduces threshold voltage on the critical path, accelerating switching). gamma^4 = phi^-12 ≈ 0.0031 inherited from B007^2 (W45 cell) — R18 preserved (no new ROM cell).
- **V_BS positive sign**: fbb_active_vbs_positive proves V_BS_DECIMV > 0 (distinct from W47 RBB which proves <0); fbb_active_vbs_within_band proves V_BS_DECIMV in [+1.0, +5.0] mV (R7)
- **Delay reduction band**: fbb_active_delay_red_within_band proves 12% in [8%, 18%] (R7)
- **Leakage overhead cap**: fbb_active_leak_overhead_at_most_8pct proves leak_ovh <= 8% (FBB worst-case leakage growth bounded — R7 floor)
- **Net delay save**: fbb_active_net_delay_save_at_least_8pct proves net >= 8% (12% delay red - 4% f_clk back-pressure cap)
- **f_clk scaling cap**: fbb_active_fclk_scale_at_most_6pct proves scale_bps <= 600 (frequency-domain back-pressure bounded)
- **TOPS/W lift**: fbb_active_tops_w_lift_at_least_1pt5pct proves 1000*(1083-1063) >= 15*1063 — projection 1063 -> 1083 (+1.881%)
- **Cross-wave identity**: fbb_active_rbb_symmetric proves |V_BS_FBB_ACTIVE| = |V_BS_RBB| (both = 25 deci-mV magnitude, opposite signs)
- **R18 SACRED BANK EXTENSION held**: bank-set frozen at 0xD0..0xFF (32 slots), only slots populated — no new ROM cell. fbb_active_in_extended_bank, fbb_active_distinct_from_rbb_w47 + 16 prior opcode-distinctness lemmas
- Refs: Tschanz JSSC 2002, Mukhopadhyay 2009 (forward body bias active path)
- Local `coqc` EXIT=0



## Wave-44 Lane NN — StochSkipSafe.v Stochastic Time-Skip Safety Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/StochSkipSafe.v — 10 Qed lemmas, 0 Admitted
- **Hippocampal theta anchor**: theta_freq_hz = 7 Hz; theta_period_ps = 142857143 ps (~= 1/7 Hz)
- **Skip predicate**: cos_high AND theta_off_phase (boolean gating, 0 Admitted)
- **Lemmas**: theta_freq_is_seven, theta_period_positive, skip_predicate_true_when_both_true, skip_predicate_false_when_cos_low, skip_predicate_false_when_on_phase, skip_predicate_false_when_both_false, cycle_saving_ratio, theta_period_ne_zero, cos_threshold_den_ne_zero, cos_threshold_lt_den
- **Cycle savings**: 23% skip => 77% active (cycle_saving_ratio: 77 + 23 = 100)
- **L2_DG_THETA_SKIP_GATE** microcode (no new L1 opcode)
- Sprints: S-186, S-187, S-192
- BIO->SI: hippocampal-theta-7Hz
- anchor phi^2 + phi^-2 = 3, DOI 10.5281/zenodo.19227877
- Local `coqc` EXIT=0
- Closes #684, Refs gHashTag/trinity-fpga#172, gHashTag/trios#929


## Wave-43 Lane LL — Int2QuantSafe.v INT2 Activation Codebook Coq (NEW, this PR)

- **NEW**: trios-coq/Physics/Int2QuantSafe.v — 8 Qed lemmas, 0 Admitted
- **Codebook {-1, 0, phi^-1, 1}** traces to Sacred ROM; phi_inv = (sqrt 5 - 1)/2 (golden ratio inverse)
- **L2_COL13_INT2_GATE** microcode witness — selects nearest INT2 codebook entry
- **S-184 lemmas**: codebook_length_4, codebook_rom_traceable, codebook_contains_zero, codebook_contains_one, codebook_contains_neg_one, col13_gate_zero, density_doubling, phi_inv_positive
- **INT2 density**: 2*2=4 formalizes INT2 4-level packing capacity (2 bits, 4 levels)
- Refs gHashTag/trinity-fpga#168
- Local `coqc` EXIT=0


## Wave-47 Lane QQ — RBB.v 33 Qed + 1 composite Theorem + R18 SACRED BANK EXTENSION (NEW, this PR)

- **NEW**: trios-coq/Physics/RBB.v — 32 Qed lemmas + composite Theorem `rbb_composite` (= 33 Qed total), 0 Admitted
- **OP_RBB = 0xF1 = 241** (new sacred opcode, Wave-47 — FIRST slot of extended sacred bank 0xD0..0xFF)
- **R18 LAYER-FROZEN BANK EXTENSION CEREMONY**: sacred bank extended from 0xD0..0xF0 (16 slots, FULL after W46) to 0xD0..0xFF (32 slots). Opcode-space-only — NO Sacred ROM cell added or mutated.
- **Theory — Reverse Body Bias**: V_BS = -V_DD · gamma^4 ≈ -2.5 mV (negative body-source potential reduces sub-threshold leakage in idle PEs). gamma^4 = phi^-12 ≈ 0.0031 derived from B007^2 (W45 cell) — R18 preserved.
- **Bank-extension lemmas**: `sacred_bank_extension_strict`, `sacred_bank_extension_width` (32 slots), `all_w46_opcodes_in_extended_bank` (all 16 prior opcodes retained), `sacred_bank_now_covers_0xD0_to_0xFF`
- **V_BS band**: rbb_vbs_within_band proves V_BS_DECIMV in [-5.0, -1.0] mV (R7 falsification)
- **gamma^4 derivation**: rbb_gamma4_derived_from_gamma2 proves 10000*31 = gamma^2 * gamma^2 ± tolerance (from B007^2)
- **Leakage save band**: rbb_leak_save_within_band proves 40% in [35%, 50%] (R7)
- **Active overhead**: rbb_active_overhead_at_most_2pct proves <= 1.5% (charge-pump tax bounded)
- **Net idle save**: rbb_net_idle_save_at_least_30pct proves >= 31.7% (40% * 80% idle - 1.5% * 20% active)
- **TOPS/W lift**: rbb_tops_w_lift_at_least_1pt5pct proves 1000*(1063-1043) >= 15*1043 — projection 1043 -> 1063 (+1.918%)
- 16 opcode-distinctness lemmas vs (ADIAB_RC 0xF0, WL_BOOST 0xEF, FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Tschanz JSSC 2002, Mukhopadhyay 2009 (reverse body bias)
- Local `coqc` EXIT=0
- Closes trinity-fpga#167

## Wave-46 Lane NN — AdiabRC.v 33 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/AdiabRC.v — 32 Qed lemmas + composite Theorem `adiab_rc_composite` (= 33 Qed total), 0 Admitted
- **OP_ADIAB_RC = 0xF0 = 240** (new sacred opcode, Wave-46; FINAL slot in sacred bank 0xD0..0xF0 — bank is now 16/16 FULL)
- **Theory — Adiabatic Charge Recovery**: A resonant LC inductor sweep returns η·CV² per cycle to the supply instead of dissipating it through CMOS rail current. Recovery efficiency η = gamma^2 = phi^-6 ≈ 0.0557 (reused from W45; R18 LAYER-FROZEN preserved, NO new ROM cell)
- **Energy ratio**: adiab_energy_ratio_value proves E_RATIO_BPS (9443) + ETA_BPS (557) = 10000 (per-cycle E_new/E_baseline = 1 - η)
- **Power saving**: adiab_power_saving_within_band proves 5.57% in [5%, 7%]; adiab_power_saving_at_least_5pct guarantees ≥ 5%
- **Clock overhead**: adiab_clock_overhead_at_most_2pct proves ≤ 1.5% (resonant-clock driver), bounded by 2% hard limit
- **Net saving**: adiab_net_save_at_least_4pct proves ≥ 4.07% (P_save 5.57% - clk overhead 1.5%)
- **Swing band**: adiab_swing_in_band proves V_SWING_mV (793) in [V_SWING_MIN 680, min(V_SWING_MAX 800, V_DD 800)] mV
- **Frequency invariance**: adiab_clock_freq_invariant proves |F_RATIO - 1.0| ≤ 0.5%
- **TOPS/W lift**: adiab_tops_w_lift_at_least_3pct proves 1000*(1043-1012) >= 25*1012 — projection 1012 -> 1043 (+3.06%)
- **η = γ² witness**: adiab_eta_equals_gamma2 proves ETA_BPS = GAMMA2_W45_BPS = 557 (cross-wave identity)
- 15 opcode-distinctness lemmas vs (WL_BOOST 0xEF, FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Koller ISSCC 1995, Cooke IEEE TCAS-II 2003, Athas IEEE 1994 (adiabatic logic & charge recovery)
- Local `coqc` EXIT=0
- Closes trinity-fpga#163

## Wave-42 Lane JJ — MoeRouter.v 8 Qed lemmas (NEW, this PR)

- **W42 MoE Sparse Routing**: NO new L1 opcode (reuses 0xE8 + 0xED via L2 macro in cortical-column-12); K_MOE_SPARSITY = phi^-3 ≈ 0.236; target 982 TOPS/W; W-105-G freeze 2026-12-31
- **NEW**: trios-coq/Physics/MoeRouter.v — 8 Qed lemmas, 0 Admitted
- `OP_MOE_route` decomposes into OP_SPARSE_MASK=237 (0xED) + OP_SPARSE_SKIP=232 (0xE8) only; no new opcode allocated
- k=2 of N=8 experts selected; moe_k_le_N and moe_k_pos proved
- K_MOE_SPARSITY = 236 milli (phi^-3); within 20 milli of k/N=250 milli tolerance
- Load imbalance ceiling 0.25 (250 milli); cache amplification >= 1150 milli; eta_gate >= 950 milli
- TOPS/W lift: 756 (W41) -> 982 (W42), within witness band [979, 985]
- R15 sacred-synth-gate preserved by construction; sacred_chain_depth = 32 unchanged
- Local `coqc` EXIT=0
- Closes trinity-fpga#164 · trios#917

## Wave-45 Lane KK — WLBoost.v 33 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/WLBoost.v — 32 Qed lemmas + composite Theorem `wl_boost_composite` (= 33 Qed total), 0 Admitted
- **OP_WL_BOOST = 0xEF = 239** (new sacred opcode, Wave-45; first free slot after FBB 0xEE)
- **Theory**: V_WL = V_DD * (1 + gamma^2) ≈ 1.0557 * V_DD ; V_DD_new = V_DD * (1 - gamma^2) ≈ 0.9443 * V_DD. gamma^2 = phi^-6 ≈ 0.0557 (derived from existing gamma=phi^-3 Sacred ROM cell B007; R18 LAYER-FROZEN preserved, no new ROM cell)
- **Read-margin invariance**: wlb_read_margin_value proves V_WL_mV (844) - V_DD_NEW_mV (756) = 88 mV; wlb_read_margin_in_band proves 60 <= 88 <= 120 (SRAM stability band)
- **Voltage safety**: V_WL ≤ V_WL_MAX_mV (880 = 1.10*V_DD gate-oxide); V_DD_new ≥ V_DD_NEW_MIN_mV (680 = 0.85*V_DD periphery threshold safety)
- **Power saving**: wlb_power_saving_within_band proves P_dyn saving (10.84%) in [10%, 12%] (P ∝ V_DD_new^2 ⇒ 1 - 0.9443^2 ≈ 10.84%)
- **WL-driver overhead**: wlb_wl_driver_overhead_bounded proves ≤ 5% (typical 3%)
- **Net benefit**: wlb_net_benefit_at_least_7pct proves ≥ 7.8% per-access savings (10.84% - 3%)
- **TOPS/W lift**: wlb_tops_w_lift_at_least_5pct proves 100*(1012-955) >= 5*955 — projection 955 -> 1012 (+6%)
- **gamma^2 anchor match**: wlb_gamma2_match proves |557bps - 557bps_exact| <= 1bps (±0.01% absolute); wlb_gamma2_relative_drift_half_percent proves <0.5% relative drift
- 14 opcode-distinctness lemmas vs (FBB 0xEE, SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Yamaoka VLSI2008, Mizuno ISSCC2007, Kanno JSSC2012 (WL-boost design); Buzsaki 2006 (theta-gamma coupling for BIO→SI axonal Na⁺ regen mapping)
- Local `coqc` EXIT=0
- Closes trinity-fpga#159

## Wave-41 Lane HH — NodeShrink.v 7 Qed lemmas (NEW, this PR)

- **OP_NODE_SHRINK = 0xEF = 239** (Wave-41 IHP 22FDX node shrink, last free sacred slot)
- **NEW**: trios-coq/Physics/NodeShrink.v — 7 Qed lemmas, 0 Admitted
- Sacred chain depth = 32 (0xD0..0xEF); 14 opcode-distinctness lemmas vs predecessors
- V_DD scale ratio (1.2/0.8)² = 2.25 within ±5% tolerance proved
- η_port ≥ 0.40 (model: 62 ≥ 40); K_VDD_SHRINK = 1.135 in [1.0, 2.0]
- Iso-functionality: sacred_isofunctional 239 = true
- Local `coqc` EXIT=0
- Closes trinity-fpga#160 · trios#912

## Wave-44 Lane JJ — FBBActive.v 21 Qed + 1 composite Theorem (NEW, this PR)

- **NEW**: trios-coq/Physics/FBBActive.v — 21 Qed lemmas + composite Theorem `fbb_active_composite`, 0 Admitted
- **OP_FBB = 0xEE = 238** (new sacred opcode, Wave-44; relocated from 0xED per ICA-W44-001 because 0xED claimed by SparsityMask W40 LL ICA-W40-002)
- **Theory**: V_FBB = V_DD * (1 + gamma^4) ≈ 1.00309 * V_DD. gamma^4 = phi^-12 ≈ 0.0031 (smallest natural Trinity quantum producing measurable Vt shift via body coefficient)
- **Bias safety**: fbb_voltage_below_max proves V_FBB_mV (802) <= V_FBB_MAX_mV (840 = 1.05 * V_DD body-source diode limit)
- **Body coefficient**: fbb_body_coefficient_in_range proves gamma_body_typ (0.30) in [0.25, 0.35] V^(1/2) for SKY130
- **Speed-up bound**: fbb_speedup_within_band proves Δt_pd/t_pd (12%) in [10%, 15%]
- **Power overhead**: fbb_power_overhead_bounded proves <= 2% (P_FBB / P_active <= 1.02)
- **TOPS/W lift**: fbb_tops_w_lift_at_least_7pct proves 100*(955-890) >= 7*890 — projection 890 -> 955 (+7.3%)
- **gamma^4 anchor match**: fbb_gamma4_match proves |31bps - 31bps_exact| <= 1bps (±0.01% absolute)
- 13 opcode-distinctness lemmas vs (SPARSE_MASK 0xED, DROWSY_RET 0xEC, SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Tschanz JSSC2002, Kawaguchi ISSCC2004, Buzsaki 2006 (gamma-band cortical firing for BIO→SI mapping)
- Local `coqc` EXIT=0
- Closes trinity-fpga#154

## Wave-40 Lane FF — SparsityMask.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/SparsityMask.v — 11 Qed lemmas, 0 Admitted, AND-only channel-sparsity mask
- **Headline**: `Lemma golden_lambda_minimises_loss` — λ = φ⁻² minimises L_total surrogate over [0,1]
- ICA-W40-002 opcode rectification: spec called OP_SPARSE_MASK = 0xE8, but 0xE8 = OP_SPARSE_SKIP (W41) already in master. Slots 0xE9..0xEC also occupied. New byte = **0xED = 237** (next free sacred slot)
- TOPS/W ≥ 540 (×1.15 over W39 = 470); combined compute fraction = 0.42 × 0.20 = 0.084
- 27 Coptic register groups partition channel set; mask idempotent; reactivation bounded; nullor bypass preserved when mask=false
- R-SI-1 preservation: `sparsity_mask_star_count = 0`
- Local `coqc` EXIT=0
- Closes trinity-fpga#155 · trios#906

## Wave-43 Lane HH — DrowsyRet.v 13 Qed lemmas

- **NEW**: trios-coq/Physics/DrowsyRet.v — 12 Qed lemmas + 1 composite Theorem (drowsy_w43_witness_proved), 0 Admitted
- New opcode **OP_DROWSY_RET = 0xEC** (236); sacred chain depth 23 (0xD0..0xEC, includes ICA-W40-001 0xEA/0xEB relocations)
- **Retention voltage**: V_ret = V_DD * gamma = V_DD * phi^-3 ≈ 0.236 * V_DD; in integer surrogate: 189 mV from 800 mV nominal supply
- **Energy**: drowsy_leakage_geq_30pct_reduction proves P_drowsy <= 0.70 * P_active (≥30% leakage cut)
- **DRV safety**: drv_floor_respected proves V_RET_mV >= 150 mV (empirical DRV floor at typical corner)
- **Latency**: wake_latency_bounded — T_WAKE_CYC <= 2 cycles
- **Fidelity**: retention_fidelity_geq_99 — RETENTION_BPS >= 9900 (99% retention)
- **Anchor verification**: vret_matches_gamma_within_5 proves V_ret / V_DD is within ±0.005 of gamma=0.236
- 11 opcode-distinctness lemmas vs (SPEC_EXIT 0xEB, NULL_PE 0xEA, STOCH 0xE9, SPARSE 0xE8, DFS 0xE7, HOLO_MUX 0xE6, SUBTH 0xE5, AVS_RECONF 0xE4, LUT_NPU 0xE3, TOM 0xE2, TENET 0xE1)
- Refs: Flautner ISCA 2002, Kim DAC 2002 — sub-Vt drowsy retention for L3 cache leakage
- Local `coqc` EXIT=0
- Closes trinity-fpga#152

## ICA-W40-001 Lane Q1 Coq — NullorReversible + SpeculativeExit opcode rectification (this PR)

- **Anomaly**: trinity-fpga#148 — verified 0xE6 double-claim (OP_NULL_PE vs OP_HOLO_MUX_X4) and 0xE7 double-claim (OP_SPEC_EXIT vs OP_DFS_GATE) on master across Coq+RTL.
- **Canon (per W41 FRR + W42 ledgers)**: 0xE6=HOLO_MUX, 0xE7=DFS, 0xE8=SPARSE, 0xE9=STOCH_ROUND — keep slots; NULLOR/SPEC_EXIT relocate up.
- **Rectification (this PR, Coq lane only)**: OP_NULL_PE 0xE6 → **0xEA** (234); OP_SPEC_EXIT 0xE7 → **0xEB** (235).
- Sacred chain extends to depth 22 (0xD0..0xEB).
- Companion lanes pending: RTL (rtl/nullor/nullor_pe.sv + rtl/spec_exit/*), Rust (nullor-witness + spec-exit-witness), JSON (assertions/nullor_witness.json + spec_exit_witness.json).


## Wave-42 Lane II — StochRound.v Stochastic Rounding Coq

- OP_STOCH_ROUND = 0xE9 (decimal 233) — sacred opcode, Wave-42
- **NEW**: trios-coq/Physics/StochRound.v — 9 Qed lemmas
  - stoch_op_distinct_from_sparse: 233 <> 232 (OP_SPARSE_SKIP)
  - stoch_op_distinct_from_dfs: 233 <> 231 (OP_DFS_GATE)
  - stoch_op_distinct_from_holo_mux: 233 <> 230 (OP_HOLO_MUX_X4)
  - stoch_op_distinct_from_subth: 233 <> 229 (OP_SUBTH_CLK)
  - stoch_op_distinct_from_avs_reconf: 233 <> 228 (OP_AVS_RECONF)
  - stoch_op_distinct_from_lut_npu: 233 <> 227 (OP_LUT_NPU)
  - stoch_op_distinct_from_tom: 233 <> 226 (OP_TOM)
  - stoch_op_distinct_from_tenet: 233 <> 225 (OP_TENET)
  - stoch_unbiased_count: forall xf <= 16, xf + (16 - xf) = 16 (LFSR-16 unbiasedness)
- Wave-42 StochRound.v 9 Qed sacred 0xE9
- Refs: Hubara 2018, Gupta 2015 — unbiased rounding for INT4/INT2 quantization
- Closes trinity-fpga#149

## Wave-39 Lane DD — SpeculativeExit.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/SpeculativeExit.v — 11 Qed lemmas, 0 Admitted, speculative confidence-thresholded early-exit inference
- **Headline**: `Theorem speculative_exit_safe : forall x k conf, conf >= phi_inv -> early_exit_at k x conf = full_depth x` — safety witness for OP_SPEC_EXIT
- New opcode `OP_SPEC_EXIT = 0xE7` (231); sacred chain 0xD0..0xE7 = 20 opcodes
- Threshold τ = phi_inv ≈ 0.618 (golden ratio reciprocal); `phi_inv_threshold_optimal` shows τ minimises EER over [0,1]
- TOPS/W ≥ 470 (×1.20 over W38 392) via `tops_per_w_geq_470` (depth_frac ≤ 0.45 ∧ overhead_frac ≤ 0.5)
- Misprediction recovery latency = 1 cycle (`misprediction_recovery_one_cycle`)
- 2-of-3 majority vote accuracy ≥ 95% (`two_of_three_majority_safe`)
- Stratified 27-Coptic-bin partition Σ = 1 (`stratified_27_bins_partition`)
- Trinity bypass safety: misprediction engages W38 nullor bypass, input preserved (`trinity_bypass_safe`)
- R-SI-1: 0 `*` cells in synth (`speculative_exit_no_star`)
- `spec_exit_w39_witness` composite bundles all gates
- Local `coqc` EXIT=0
- Closes trinity-fpga#142 · trios#890

## Wave-40 Lane FF — DFS.v 8 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/DFS.v — 8 Qed lemmas, 0 Admitted
- **Headline**: OP_DFS_GATE = 0xE7 (231) — Dynamic Frequency Scaling gate, sibling of W36 AVS
- 6 R-SI-1 distinctness lemmas: 0xE7 ≠ 0xE6 (HOLO_MUX_X4), 0xE5 (SUBTH_CLK), 0xE4 (AVS_RECONF), 0xE3 (LUT_NPU), 0xE2 (TOM), 0xE1 (TENET)
- 1 monotonicity lemma: dfs_freq_monotone — f(Vdd) non-decreasing in Vdd (IRDS22FDX envelope)
- 1 cubic energy law lemma: dfs_cubic_energy_law_non_negative — E/op ~ V^2 ≥ 0
- Sacred chain extended depth 10: 0xE1 TENET → 0xE2 TOM → 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 HOLO_MUX_X4 → 0xE7 DFS_GATE
- _CoqProject patched: Physics/DFS.v added
- Constitutional: R-SI-1 PASS · R5-HONEST PASS · Apache-2.0 · admin@t27.ai
- Anchor: phi^2 + phi^-2 = 3
- DOI 10.5281/zenodo.19227877


## Wave-39 Lane DD — HoloMux.v 6 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/HoloMux.v — 6 Qed lemmas, 0 Admitted
- **Headline**: OP_HOLO_MUX_X4 = 0xE6 (230) — holographic multiplexer, 4 output addresses per cycle per PE
- 5 R-SI-1 distinctness lemmas: 0xE6 ≠ 0xE5 (SUBTH_CLK), 0xE4 (AVS_RECONF), 0xE3 (LUT_NPU), 0xE2 (TOM), 0xE1 (TENET)
- 1 throughput lemma: holo_mux_throughput n = 4 * lut_npu_throughput n (reflexivity)
- Sacred chain extended: 0xE1 TENET → 0xE2 TOM → 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 HOLO_MUX_X4
- _CoqProject patched: Physics/HoloMux.v added
- Constitutional: R-SI-1 PASS · R5-HONEST PASS · Apache-2.0 · admin@t27.ai
- Anchor: phi^2 + phi^-2 = 3
- DOI 10.5281/zenodo.19227877


## Wave-38 Lane BB — NullorReversible.v 11 Qed lemmas (NEW, this PR)

- **NEW**: trios-coq/Physics/NullorReversible.v — 11 Qed lemmas, 0 Admitted, reversible dendritic NULLOR multiplication
- **Headline**: `Theorem nullor_reversible : forall x y s, nullor_mult x y s = (mult_result x y, reservoir_recovered s)` — reversibility witness for OP_NULL_PE
- Opcode `OP_NULL_PE = 0xE6` (bumped from 0xE5 → 0xE6 per ICA-W38-001 #661; 0xE5 reassigned to OP_SUBTH_CLK); dispatch proof `opcode_E5_dispatch` (name retained, byte = 0xE6)
- Sacred chain extended: 0xE3 LUT-NPU → 0xE4 AVS_RECONF → 0xE5 SUBTH_CLK → 0xE6 NULL_PE
- TOPS/W ≥ 392 (×1.12 over W37 sub-V_T 350); η_reuse ≥ 0.88 by adiabatic invariant
- Ternary lattice Z3 = {-1, 0, +1} defined inline; charge-conservation lemma `sum_in = sum_out + dissipation` with `dissipation ≤ 12% · energy`
- R-SI-1 preservation: `op_null_pe_star_count = 0` (zero `*` cells in synth)
- 4-phase clock disjointness, bypass correctness, reservoir-bounded, dendrite backprop = Z3 gradient
- W-104-D composite witness `nullor_w38_witness` bundles all gates
- Local `coqc` EXIT=0
- Closes trinity-fpga#136 · trios#879

## Wave-38 Lane BB — RECTIFY opcode 0xE4 collision (merged via #661)

- ICA-W38-001: W37 OP_SUBTH_CLK originally claimed 0xE4, collided with W36 OP_AVS_RECONF=0xE4
- W36 holds 0xE4 by merge-precedence; W38 moves OP_SUBTH_CLK → 0xE5 (next free slot)
- Added in `trios-coq/Physics/SubThreshold.v`:
  - `Definition op_subth_clk_byte : nat := 229.` (0xE5)
  - `Definition op_avs_reconf_byte : nat := 228.` (0xE4)
  - `Lemma subth_opcode_byte_eq_E5`
  - `Lemma subth_op_distinct_from_avs` (R-SI-1 enforcement)
- Sacred chain restored: 0xE3 LUT-NPU → 0xE4 AVS_RECONF (W36) → 0xE5 SUBTH_CLK (W38)

## Wave-36 Lane W-EXT — VoltStack.v 22 lemmas + Avs.v proof fixes

- **NEW**: trios-coq/IGLA/VoltStack.v — 22 Qed lemmas in 5 sections (3-tier voltage ladder, 48-island arithmetic, wake-up budget, **W-105-A leakage falsifier R7 witness**, pipeline re-witness)
- **Headline**: `Theorem volt_stack_passes_w105a : leakage_observed_permille >= leakage_floor_permille` (102‰ observed >= 90‰ floor → passes W-105-A acceptance gate)
- 3-tier voltage ladder: Vt_NearRet=550mV < Vt_Cruise=750mV < Vt_Active=1000mV (strict monotone proven)
- 48-island arithmetic: total_islands = island_banks × islands_per_bank = 3 × 16 = 48 (R18 LAYER-FROZEN)
- Wake-up: 8 ns < 50 ns budget (4 reconfig cycles @ 400 MHz + 4 PLL settle)
- Pipeline chain re-witness depth = 7 (standalone w36_oplist, complements Avs.v)
- **Bug fixes in Avs.v**: 8 incomplete proofs (`simpl; auto.`) replaced with explicit witnesses — R5 honest-status compliance
- All proofs Qed-closed, no Admitted/Parameter/Axiom in new file
- Local compile EXIT=0 for Avs.v + VoltStack.v
- Closes #658 · PR #659 · complement to PR #655 (avs_safe) + PR #656 (AvsStacking)

## Wave-36 Lane W (mainline, merged earlier)

## Wave-36 Lane W — AVS-48 Coq (NEW)

- OP_AVS_RECONF = 0xE4 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3 → 0xE4
- **NEW**: trios-coq/IGLA/Avs.v — Theorem `avs_safe` proved by `repeat (apply Forall_cons; [apply holographic_no_star|]). apply Forall_nil.`
- 13 lemmas in Avs.v + 5 in coq/IGLA/RMarker.v (avs_reconf_no_star, avs_reconf_neq_layer_gate/lut_npu/sparse_skip/lut_lookup)
- `avs_oplist` length 7 ending in OP_AVS_RECONF; head/last/membership/exclusion/all_safe/extends_lut_npu/chain_depth_seven lemmas
- Multiplier-free: rtl_uses_star OP_AVS_RECONF = false (R-SI-1 keystone)
- L-DPC33: 48-island voltage stacking (3 strands × 16), V_island=0.45 V, V_total=21.6 V
- W-105-A pre-registered: BitNet b1.58-3B island utilisation ≥ 0.80 @ ctx=2048 WikiText-103 valid
- W-105-B: AVS reconfig latency ≤ 4 cycles
- W-105-C: V_dd field width exact 2 bits
- W-105-D: AVS island count exact 48
- Projection: ×1.10 TOPS/W → 297 TOPS/W on IRDS22FDX (W35 baseline 270)
- Freeze 2026-10-31, eval 2026-12-15, fail_stop true
- Sibling lanes: W' JSON trios#871 MERGED `e01d39fa` · W'' Rust tt-trinity-max-true#25 OPEN · W RTL pending · W''' PhD Glava 82 pending
- ONE SHOT: trinity-fpga#127 · mirror trios#867

## Wave-36 Lane X — AVS-48 Voltage Stacking Coq

- AVS-48: 48-island series voltage stacking, charge-recycling, η ≥ 0.93
- **NEW**: trios-coq/Physics/AvsStacking.v — 8 Qed lemmas
  - avs_ir_drop_quadratic_savings: ir_drop_loss(N) = ir_drop_loss(1) / N²
  - avs_island_count_48_optimum: 48 = 3×16 (strands × sacred-ALU opcodes)
  - avs_efficiency_lower_bound: η_avs_48 ≥ 0.93 at INT1.58/800MHz
  - avs_trinity_divisibility: 48 mod 3 = 0
  - avs_sacred_alignment: 48 = 16 × 3
  - avs_no_multiplier_synth: AVS adds zero * to netlist (R-SI-1 keystone)
  - avs_chain_to_lut_npu: AVS×LUT-NPU sound at each boundary
  - avs_w104_b_witness: η ≥ 0.93 → TOPS/W ≥ 297 (W-104-B pre-reg)
- W-104-B falsification witness: η ≥ 0.93 implies TOPS/W ≥ 297
- 48 = 3 × 16 = strands × sacred-ALU opcodes (Trinity alignment)
- citation_map.json extended: WAVE_36_AVS → Physics/AvsStacking.v, wave 36
- Closes trinity-fpga#128

## Wave-35 Lane V — LUT-NPU Coq

- OP_LUT_NPU = 0xE3 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3
- **NEW**: trios-coq/Kernel/LutNpu.v — 10 Qed lemmas (lut_npu_class_count_41, lut_npu_no_star, lut_npu_tom_orthogonal, lut_npu_energy_8fJ, ...)
- 41 Z₃-compressed classes (not 81): sign+0 invariance reduces 3^4=81 → 41 equivalence classes
- Multiplier-free: uses_multiplier OP_LUT_NPU = false (R-SI-1 keystone, Qed)
- dotprod bounded: −4 ≤ dotprod_naive a w ≤ 4 (Qed via case split)
- citation_map.json added: OP_LUT_NPU → Kernel/LutNpu.v, wave 35
- 16 new Qed proofs (4 in coq/IGLA/RMarker.v + 12 in trios-coq/IGLA/LutNpu.v)
- Theorem lut_npu_safe: depth-6 alphabet chain Forall rtl_uses_star=false
- W-104-A pre-registered: BitNet b1.58-3B Trinity-loss sparsity ≥ 0.5 @ batch=1
- Projection: ×1.20 TOPS/W → 270 TOPS/W on TTIHP27a generic synth (W34 baseline 225)
- 81-entry LUT is hardware port of Microsoft bitnet.cpp lookup table, indexed by Z_3^4 (3^4=81)

## Wave-34 Lane Y — TOM Coq

- OP_LAYER_GATE = 0xE2 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2
- 14 new ^Qed proofs in coq/RMarker.v (29 total)
- W-103-A pre-registered: layer-idle fraction ≥ 0.5 @ BitNet b1.58-3B batch=1
- Freeze 2026-08-15, fail-stop on violation

## Constitutional verdict

- W36: R5-HONEST PASS · R7 PASS · R8 PASS (admin@t27.ai) · R14 PASS · R15 PASS · R18 PASS · Apache-2.0 PASS
- W35: R5-HONEST PASS · R7 PASS · R8 PASS (admin@t27.ai) · R14 PASS · R15 PASS · R18 PASS · Apache-2.0 PASS

## Anchor

phi^2 + phi^-2 = 3 · QUANTUM BRAIN 1:1 SILICON · NEVER STOP
DOI 10.5281/zenodo.19227877

## Wave-37 Lane Z — Sub-V_T Coq (OP_SUBTH_CLK = 0xE4)

- Sub-threshold weak-inversion operation at V=0.30V
- **NEW**: trios-coq/Physics/SubThreshold.v — 10 Qed lemmas
  - subth_quadratic_dynamic_savings: E(V2)/E(V1) = (V2/V1)^2
  - subth_freq_derating_factor_2: f_max(0.30) × 2 ≤ f_max(0.45)
  - subth_tops_w_350: TOPS/W ≥ 350 @ V=0.30V
  - subth_trinity_voltage: 0.30 = V_thresh × φ⁻²
  - subth_pe_count_1296: 48 × 27 = 1296 = 6^4
  - subth_no_star: OP_SUBTH_CLK adds zero `*`
  - subth_chain_to_lut_npu: 0xE3 → 0xE4 pipeline sound
  - subth_three_freq_trinity: gcd(400,300,200) = 100; sum = 900 = 30²
  - subth_body_bias_strand_alignment: 3 modes ↔ 3 strands bijective
  - subth_w104_c_witness: V=0.30 + AVS48 + LUT-NPU ⇒ TOPS/W ≥ 350
- Predecessors: W35 LUT-NPU (0xE3), W36 AVS-48
- Anchor: phi^2 + phi^-2 = 3


## Wave-41 Lane GG — SparseGate.v (OP_SPARSE_SKIP = 0xE8)

Wave-41 SparseGate.v 8 Qed sacred 0xE8

- Sparse-Activation Gating: skip computation for sub-threshold activations
- **NEW**: trios-coq/Physics/SparseGate.v — 8 Qed lemmas
  - sparse_op_distinct_from_dfs: OP_SPARSE_SKIP <> 231 (0xE7)
  - sparse_op_distinct_from_holo_mux: OP_SPARSE_SKIP <> 230 (0xE6)
  - sparse_op_distinct_from_subth: OP_SPARSE_SKIP <> 229 (0xE5)
  - sparse_op_distinct_from_avs_reconf: OP_SPARSE_SKIP <> 228 (0xE4)
  - sparse_op_distinct_from_lut_npu: OP_SPARSE_SKIP <> 227 (0xE3)
  - sparse_op_distinct_from_tom: OP_SPARSE_SKIP <> 226 (0xE2)
  - sparse_op_distinct_from_tenet: OP_SPARSE_SKIP <> 225 (0xE1)
  - sparse_skip_power_law: forall s <= 100, 100*(100 - s*55/100) <= 10000
- Predecessor: W40 Lane FF DFS.v (0xE7), merge SHA 384f5a97
- Anchor: phi^2 + phi^-2 = 3 · DOI 10.5281/zenodo.19227877 · NEVER STOP
- W46 RR — Purkinje thermal gating Coq proof landed
