# NOW -- ring-096 and formats.t27 agree on f32 (2026-10-04)

## ring-096 and formats.t27 agree on f32 (Closes #5746)

- decided by the law, not by taste: Constitution Article SSOT-MATH gives numeric meaning "one normative source of truth: specifications in the t27 language", and the numeric SSOT codec in `specs/numeric/gf16.t27` (L6) crosses the float boundary as `f32` -- `gf16_encode_f32(f32)`, `gf16_decode_to_f32(GF16) f32`
- `specs/numeric/formats.t27`: `gf16_to_f32`, `ternary_to_f32` and `quantize_value` returned `gf16` (lowers to `u16`, an encoding) and now return `f32`; inputs were already `f32`
- its tests unwrapped results with `gf16.to_f64(...)`, which gf16.t27 does not declare; they now compare f32 directly. `gf16_to_f32_normal_one` used `0x3C00` (e=30, i.e. 0.5 -- binary16's 1.0) and now uses GF16's `0x3E00` (FORMAT-SPEC-001.json: bias 31, so 1.0 is E=31); `ternary_to_f32_is_inverse` claimed 0.5 survives the trip and now ranges over trits
- `rings/ring-096-rust`: the five functions take and return `f32` instead of `f64`; a new test re-encodes every one of the 63,488 normal GF16 codes through the f32 boundary and gets the same 16 bits back (43 tests pass, was 42)
- `check_ring_spec_drift.py`: ring-096 DRIFTED (differing 5, rc 1) -> CONVERGED (differing 0, rc 0); both formats seals resealed (tests BLOCKED: the spec's functions have no bodies, before and after)
- not fixed, now visible: as a CONVERGED pair ring-096 enters the differential step, which exits 2 on it -- gen-rust emits the spec's `u5`/`u4` consts verbatim, `Trit`/`Format` cannot be synthesised, the spec side is `unimplemented!()`, and the harness compares f32 with `==`, so code 0xFFFF (NaN on both sides) would read as a disagreement; all four are outside this issue's boundary
- left open, outside the boundary: gf16.t27 is not self-consistent -- its decode formula puts 1.0 at `0x3E00`, its `pow2_table` and `gf16_from_components` tests say `0x3C00`; its NaN is `0xFE01` where formats.t27 and the ring use `0x7F01`
