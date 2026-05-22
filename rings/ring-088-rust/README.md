# ring-088 -- GF16 MAC

> **Wave 15** (2026-05-22): the first honestly-imported Wave-11 crate.
>
> Anchor: `phi^2 + 1/phi^2 = 3`.

## What

A real, compileable, tested implementation of:

1. **GF16 codec** -- `f32 <-> GoldenFloat16` round-trip per
   [`specs/numeric/gf16.t27`](../../specs/numeric/gf16.t27).
   Bit layout `[S(1) E(6) M(9)]`, bias 31, special exponent `0x3F` for
   `Inf` / `NaN`, separate `+0` / `-0`.
2. **Multiply-Accumulate** -- [`mac_dot`](src/lib.rs) is an allocation-free
   scalar dot product over equal-length `&[Gf16]` slices, returning the
   accumulator as `f32`. NaN poisons; length mismatch returns `None`.
3. **Identity witness** -- [`identity_witness`](src/lib.rs) returns `true`
   iff `phi^2 + 1/phi^2 == 3` to f64 1e-15. Required of every t27 ring crate.

The crate is `#![no_std]` (test cfg pulls `std` for the test harness),
`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`. No external dependencies.

## Honest scope

* **No SIMD, no FPGA, no benchmarks claimed.** Performance work is a later wave.
* **No new GF16 spec.** All constants (`BIAS`, `MANT_DIVISOR`, `SPECIAL_EXP`,
  `GF16_INF_POS`, ...) mirror `specs/numeric/gf16.t27` byte-for-byte.
* **R5-HONEST:** the previous Wave-11 narrative claimed ring-088 was authored
  "in another sandbox" with 961 LOC, but no source ever reached this repo.
  Wave 15 starts the real import; this is the actual ring-088.

## Tests (13, all pass)

Mirrors `specs/02-gf16-format.tri` "mandatory 8" plus extra MAC tests:

| Test                              | What it proves                                            |
|:----------------------------------|:----------------------------------------------------------|
| `identity_witness_holds`          | `phi^2 + 1/phi^2 == 3` to 1e-15                           |
| `gf16_roundtrip_phi`              | `|round-trip(1.618) - 1.618| < 0.01`                      |
| `gf16_from_zero_pos`              | `+0 -> 0x0000`                                            |
| `gf16_from_zero_neg`              | `-0 -> 0x8000`                                            |
| `gf16_phi_identity`               | `phi` survives encode/decode within 0.01                  |
| `gf16_quantization_roundtrip_pi`  | `pi` survives within 0.05                                 |
| `gf16_better_phi_distance_than_f16` | spec invariant 0.049 < 0.118                            |
| `gf16_inf_roundtrip`              | +-Inf encode and decode                                   |
| `gf16_nan_propagates`             | NaN -> `GF16_NAN`, decodes back to NaN                    |
| `mac_dot_empty`                   | `dot([], []) == 0`                                        |
| `mac_dot_length_mismatch`         | unequal lengths -> `None`                                 |
| `mac_dot_simple`                  | `[1,3] . [2,4] = 14` within 0.05                          |
| `mac_dot_phi_identity`            | **`mac_dot([phi, 1/phi], [phi, 1/phi]) ~= 3`** (anchor)   |

The last test is the first time the project's identity anchor is exercised
*through actual numeric kernels*, not as a free-standing f64 assertion.

## Build

```bash
cd rings/ring-088-rust
cargo check --all-targets   # green on Rust 1.83.0
cargo test                  # 13 passed, 0 failed
```

Local verification on Rust 1.83.0 (matching
[`Dockerfile.rust`](../../Dockerfile.rust)) -- the Wave-13 `rings-rust`
matrix will re-confirm on every PR that touches this crate.
