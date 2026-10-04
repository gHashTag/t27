# ring-096-quantization

Format quantization primitives — GF16 codec, ternary quantization, Format enum.

Mirrors `specs/numeric/formats.t27` byte-for-byte.

## Primitives

- **GF16 bit layout**: `SIGN_MASK=0x8000`, `EXP_MASK=0x7E00`, `MANT_MASK=0x01FF`, `BIAS=31`
- **`gf16_to_f32(u16) -> f32`** — decode: handles signed zero, denormals, normals, Inf, NaN
- **`f32_to_gf16(f32) -> u16`** — encode (round-to-nearest): handles signed zero, Inf, NaN, overflow, underflow
- **`f32_to_ternary(f32) -> Trit`** — ternary quantization with threshold 0.5
- **`ternary_to_f32(Trit) -> f32`** — convert ternary back to float
- **`Format` enum** — `Fp32`, `Fp16`, `Bf16`, `Gf16`, `Ternary`
- **`format_bytes(Format) -> usize`** — byte size lookup
- **`quantize_value(f32, Format) -> f32`** — generic quantization utility

The float boundary is `f32`, as the spec declares and as the numeric SSOT codec
`specs/numeric/gf16.t27` does. It was `f64` until #5746, which is what
`tools/check_ring_spec_drift.py` reported as five drifted signatures.

## no_std math

Pure `no_std` — no `libm`. Internal helpers:
- `pow_u64` — fast integer exponentiation by squaring
- `fabs_no_std` — branch-based absolute value

## Anchor #8

`quantization_phi_identity` routes the Trinity identity `phi^2 + 1/phi^2 = 3` through:
- `pow_u64(phi, 2)` and `pow_u64(phi, -2)`
- Encode both via `f32_to_gf16`
- Decode via `gf16_to_f32`
- Sum and verify ≈ 3.0 within GF16 mantissa tolerance (~0.03 absolute)

## Build & test

```
cargo check
cargo test --lib
```

## Constitutional compliance

- L1 TRACEABILITY — `Closes #733`
- L2 GENERATION — zero edits under `specs/`, `gen/`, `coq/`
- L3 PURITY — ASCII source, English doc-comments
- L4 TESTABILITY — `#[test]` blocks
- L5 IDENTITY — `phi^2 + 1/phi^2 = 3` exercised
- L6 CEILING — spec constants byte-for-byte
- L7 UNITY — no `*.sh` files

---

`phi^2 + 1/phi^2 = 3`
