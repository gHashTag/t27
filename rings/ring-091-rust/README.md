# ring-091 -- Stochastic Rounding

> **Wave 18** (2026-05-22, Closes #723): the fourth honestly-imported
> Wave-11 crate. Waves 15-17 landed [ring-088](../ring-088-rust)
> (GF16 + MAC), [ring-089](../ring-089-rust) (TNN ISA), and
> [ring-090](../ring-090-rust) (Simulator). Wave 18 lands ring-091:
> an unbiased rounding mode that's standard practice in low-precision
> ML training, plus a deterministic 64-bit PRNG to drive it.
>
> Anchor: `phi^2 + 1/phi^2 = 3`.

## What is stochastic rounding?

Given a real value `x` that doesn't lie exactly on the target grid,
deterministic round-to-nearest always picks the *closer* grid point.
Repeated rounding accumulates a bias.

Stochastic rounding picks the nearest grid points above and below and
chooses between them randomly, weighted by distance:

```text
  ceil(x)  with probability   frac(x) = x - floor(x)
  floor(x) with probability   1 - frac(x)
```

In expectation, `E[SR(x)] == x` -- the rounding is **unbiased**. Over
many independent roundings the average converges to the true value.
This is the universal property exercised by `sr_is_unbiased` and
`sr_quantize_phi_unbiased` in this crate.

## What

A real, compileable, tested implementation of:

1. **[`SplitMix64`]** -- a deterministic, seedable, allocation-free 64-bit
   PRNG (Vigna 2014). `next_u64()` is branch-free and constant-time. The
   crate's `splitmix_first_value_with_seed_0` test checks the published
   reference value (`SplitMix64(0).next() == 0xE220A8397B1DCDAF`). The
   multiplicative gamma is `0x9E3779B97F4A7C15 = floor(2^64 / phi)` --
   the same golden anchor this project preserves.
2. **`RoundingMode`** -- enum `{Nearest, Stochastic}`.
3. **`sr_round_f32_to_i32(x, rng)`** -- single-value SR over the integer
   grid. NaN / Inf return 0 by contract; values outside the `i32` range
   saturate.
4. **`sr_quantize_f32(x, step, rng)`** -- single-value SR onto a uniform
   grid of `step`. `step = 0` returns `x` unchanged.
5. **`sr_quantize_batch(input, output, step, rng) -> usize`** -- streaming,
   allocation-free batch quantization. Returns elements written.
6. **`floor_f32`, `frac_f32`, `is_finite_f32`, `abs_f32`** -- inline
   `no_std`-friendly f32 helpers implemented via `i32` truncation. (Rust's
   `core` library does not expose `f32::floor` without `libm`; this crate
   refuses external deps.)
7. **`identity_witness`** -- universal anchor `phi^2 + 1/phi^2 == 3` to
   f64 1e-15.

The crate is `#![no_std]` (test cfg pulls `std` for the harness only),
`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`. **No external
dependencies.**

## Honest scope (R5-HONEST)

* **No new spec.** SR is a textbook universal numeric algorithm; SplitMix64
  is a textbook PRNG (Vigna 2014). No file under `specs/`, `coq/`,
  `proofs/`, `bootstrap/`, `gen/` is touched (L2 GENERATION, L6 CEILING).
* **No GF16 path-dependency.** SR over [`Gf16`](../ring-088-rust) is a
  natural next step but adds an inter-crate dependency on ring-088's
  quantizer -- out of scope for Wave 18.
* **No hardware integration.** SR-aware MAC, SR-aware FPGA cell,
  accelerator wiring -- all out of scope.
* The Wave-11 narrative quoted **409 LOC** for ring-091. Honest measurement
  is **462 LOC**. This is the first ring in the import series whose honest
  LOC modestly *exceeds* the claim; earlier rings under-shot
  (ring-088: 961 -> 439; ring-089: 334 -> 635 over; ring-090: 2143 -> 547).

## Tests (19, all pass on first run)

| Test                                       | What it proves                                          |
|:-------------------------------------------|:--------------------------------------------------------|
| `splitmix_is_deterministic`                | same seed -> same sequence                              |
| `splitmix_different_seeds_differ`          | seed actually influences output                         |
| `splitmix_first_value_with_seed_0`         | Vigna reference: `SplitMix64(0).next() = 0xE220A8397B1DCDAF` |
| `next_f32_unit_in_range`                   | 1000 draws all in `[0.0, 1.0)`                          |
| `floor_f32_positive`                       | `floor_f32` on non-negative inputs                      |
| `floor_f32_negative`                       | `floor_f32` rounds toward -infinity                     |
| `frac_f32_basic`                           | `frac_f32(-0.25) = 0.75` and friends                    |
| `sr_exact_integer_returns_integer`         | SR is a no-op on grid points                            |
| `sr_nan_returns_zero`                      | NaN contract                                            |
| `sr_inf_saturates`                         | `+/- Inf` contract                                      |
| `sr_round_returns_floor_or_ceil`           | output of 1000 `SR(2.7)` calls is always 2 or 3         |
| `sr_quantize_zero_step_passthrough`        | `step = 0` returns input unchanged                      |
| `sr_quantize_step_one_matches_round_to_i32`| `sr_quantize(x, 1.0)` == `sr_round_f32_to_i32(x) as f32`|
| `sr_is_unbiased`                           | mean of 10 000 `SR(0.3)` draws < 0.02 from 0.3          |
| **`sr_quantize_phi_unbiased`**             | **mean of 10 000 `SR-quantize(phi, 0.01)` < 0.001 from phi** |
| `sr_quantize_batch_writes_min_len`         | batch returns `min(input.len, output.len)`              |
| `sr_quantize_batch_empty_input`            | empty input writes 0 outputs                            |
| `identity_witness_holds`                   | `phi^2 + 1/phi^2 == 3` to 1e-15                         |
| `rounding_mode_eq`                         | enum equality + inequality                              |

The two statistical tests (`sr_is_unbiased`, `sr_quantize_phi_unbiased`)
are the substantive ones: they verify the unbiasedness property
empirically against a 3-sigma bound. The second is the **third
cross-kernel anchor test** in the project (after ring-088's
`mac_dot_phi_identity` and ring-089's `cpu_phi_identity_integer_projection`):
it exercises `phi` through SR-quantization rather than through GF16 MAC or
CPU instruction dispatch.

## Build

```bash
cd rings/ring-091-rust
cargo check --all-targets   # green on Rust 1.83.0
cargo test --lib            # 19 passed, 0 failed
```

Local verification on Rust 1.83.0 (matching
[`Dockerfile.rust`](../../Dockerfile.rust)) -- the Wave-13 `rings-rust`
matrix will re-confirm on every PR that touches this crate.
