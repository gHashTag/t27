# t27 numeric-format conformance vector packs (v0)

Bit-precise conformance vectors for a family of low-precision numeric formats,
in a single shared row schema so one differ runs across all packs.

- SSOT: https://github.com/gHashTag/t27/blob/master/conformance/FORMAT-SPEC-001.json
- Anchor identity (ASCII): `phi^2 + 1/phi^2 = 3`
- Context preprint: https://arxiv.org/abs/2606.05017
- Schema tag: `t27-conformance/v0.1`

Every pack decodes its native bits to f64 exactly (`abs_error = 0` by
construction for representable values). Values that are NOT exactly
representable in a format report a nonzero `abs_error` honestly (for example
0.1 in bf16) -- nothing is hidden.

## Index

| Pack | Format | Bits | Vectors | 3.0 anchor bits | Max finite | Inf | NaN | Has golden | tt-mlir issue |
|---|---|---|---|---|---|---|---|---|---|
| `gf16_conformance_v0.json` | GF16 | 16 | 21 | `0x4100` | n/a | yes | no | no | sibling (SSOT anchor pack) |
| `fp8_e4m3fn_conformance_v0.json` | FP8 E4M3FN | 8 | 14 | `0x44` | 448.0 (`0x7E`) | no | single (`0x7F`/`0xFF`) | no | [#8549](https://github.com/tenstorrent/tt-mlir/issues/8549) |
| `fp8_e5m2_conformance_v0.json` | FP8 E5M2 | 8 | 16 | `0x42` | 57344.0 (`0x7B`) | yes (`0x7C`/`0xFC`) | yes (E=11111,M!=0) | no | [#8549](https://github.com/tenstorrent/tt-mlir/issues/8549) |
| `mxfp4_e2m1_conformance_v0.json` | MXFP4 E2M1 | 4 | 16 | `0x5` | 6.0 (`0x7`) | no | no | no | [#7085](https://github.com/tenstorrent/tt-mlir/issues/7085) |
| `bf16_golden_conformance_v0.json` | bfloat16 | 16 | 8 + golden | `0x4040` | n/a | yes | yes | yes | [#6252](https://github.com/tenstorrent/tt-mlir/issues/6252) |

## SHA-256

```
7aea5b9e86ea71a54ae0c1601cea13e2d90d95fecaf2ae969eac1349cf7a2b42  gf16_conformance_v0.json
7193ccd0d330d3e05154432abcec5da4a4c170e11004d4ffa44ff5cbbff9cba9  fp8_e4m3fn_conformance_v0.json
9c31fbd03923bd6555304848a092504dfbc02f72d2be82d2b80f49243e925a18  fp8_e5m2_conformance_v0.json
b5795fed0c0f2b580174b443d2c54519c4953916525237bb7ea7d6831f14fde7  mxfp4_e2m1_conformance_v0.json
98bbddcbb8a520dc45a6dfed7209c50a0acc0fabc4d3b430359969467eee4e13  bf16_golden_conformance_v0.json
```

## Shared row schema

Each vector row carries:

```
name              short label
input_f64         the f64 value (or "NaN" / "Inf" / "-Inf" for specials)
input_f64_hex     big-endian IEEE-754 double hex of input
<fmt>_bits_hex    the native format bit pattern, hex
<fmt>_bits_int    the native format bit pattern, integer
decoded_f64       value after decode back to f64
decoded_f64_hex   big-endian double hex of decoded
abs_error         |input_f64 - decoded_f64| (0 for representable values)
category          one of: zero, subnormal, normal, inf, nan, phi_anchor
```

The bits field is named per format: `gf16_bits_*`, `fp8_bits_*`,
`mxfp4_bits_*`, `bf16_bits_*`. All other keys are identical across packs.

## Per-format notes (the round-trip decision in each)

### GF16 (16-bit)
The original anchor pack. Carries `phi`, `inv_phi`, `phi^2`, `inv_phi^2`, and
the identity sum `phi^2 + 1/phi^2 = 3` (vector `identity_sum`, bits `0x4100`).
Categories include `phi_anchor`. Sibling to the FP8/MXFP4/bf16 packs via the
shared row schema.

### FP8 E4M3FN (`torch.float8_e4m3fn`)
1s4e3m, bias 7. No inf. Single NaN at `S.1111.111`. Max finite 448.0 at `0x7E`
(mantissa 110, since all-ones mantissa at max exponent is reserved for NaN).
Min subnormal 2^-9, min normal 2^-6.
Encode policy (reference): round-nearest-ties-even; overflow SATURATES to
448.0, not NaN. That saturate-vs-NaN choice is the round-trip decision in
tt-mlir #8549.

### FP8 E5M2 (`torch.float8_e5m2`)
1s5e2m, bias 15. HAS inf at `0x7C`/`0xFC` (E=11111, M=00); NaN at E=11111,
M!=00. Max finite 57344.0 at `0x7B`. Min subnormal 2^-16, min normal 2^-14.
Encode policy (reference): round-nearest-ties-even; overflow goes to +/-Inf
(true IEEE), NOT saturate. The saturate (E4M3FN) vs overflow-to-Inf (E5M2)
split is the practical round-trip gotcha across the two FP8 variants.

### MXFP4 E2M1 (OCP Microscaling element format)
1s2e1m, bias 1. Only 16 codes; no inf, no NaN. Full positive grid
`{0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0}`. Max finite 6.0 at `0x7`.
tt-mlir #7085 clamps to +/-7, which is one step ABOVE the format max:
values in (6, 7] still round to the 6.0 grid point. The clamp bound and the
representable max are different numbers; the pack enumerates all 16 codes so
that relationship is explicit. Anchor 3.0 is an exact grid point at `0x5`.

### bfloat16 (golden accumulation)
1s8e7m, bias 127 (top 16 bits of fp32). HAS inf and NaN. Decode exact into f64.
Element table plus a `golden_accumulation` section targeting tt-mlir #6252:
the all-close gap in depth-31 fused reduction trees comes from accumulation
ORDER, not single-op rounding. The section gives an f64 golden reference and
compares sequential vs pairwise bf16 reductions; pairwise stays near the golden
while sequential drifts. A reproducible all-close test needs a clamped
deterministic input set plus an f64 golden -- both are supplied.

## Provenance

Generators (re-run to reproduce byte-identical packs):

```
gen_fp8_e4m3.py    -> fp8_e4m3fn_conformance_v0.json
gen_fp8_e5m2.py    -> fp8_e5m2_conformance_v0.json
gen_mxfp4_e2m1.py  -> mxfp4_e2m1_conformance_v0.json
gen_bf16_golden.py -> bf16_golden_conformance_v0.json
```

All packs are ASCII-only. Apache-2.0, consistent with the t27 repository.
