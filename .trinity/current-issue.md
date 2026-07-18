# Wave Loop 587 — Current Issue

**Issue #1558** — Module-scope 8-D array-of-struct variable initialized from a
function call with indexed signed field writes.
**Branch:** `wave-loop-587`.
**Previous:** Wave Loop 586 closed (#1557, branch `wave-loop-586`).

## Chosen variant

**Variant C — module-scope 8-D array-of-struct variable initialized from a call
with indexed field writes.**

Witness: `specs/scratch/w587_bench_module_8d_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub const expected : [2]^8 Pt` with explicit packed literal (values 21..532).
- `pub fn make_oct(offset : u16) -> [2]^8 Pt` returning the same literal.
- `pub var dst : [2]^8 Pt = make_oct(20)` — module-scope mutable packed register.
- `test` block: whole-array equality plus corner indexed reads.
- `bench` block: multi-site reads, signed field writes (`999`, `-999`, `-1234`,
  `1234`), read-back, and frame-condition checks on unchanged elements.

## Status

- Witness generated and parser/generation verified.
- Icarus simulation and cocotb reference-model cross-check passed.
- Seal and Icarus baseline created.
- Integration test `accepts_w587_bench_module_8d_aos_var_call_write` added.
- Full tri pipeline running.

## Next Wave Loop 588 cooperation variants

1. **Variant A — 19-D array-of-struct return call deduplication.**
   Extend rank scaling to `[2]^19 Pt` (16,777,216-bit packed vector, 524,288
   elements). Follow the W573–W584 local-`expected` / `e ≤ 16383` discipline.
   Risk: witness ~90 MB / ~4.8 M lines; direct simulation likely 80+ minutes,
   probably needs background CI rather than interactive loop.

2. **Variant B — 18-D array-of-struct return with non-power-of-two outer
   dimension.**
   Witness `[3][2]^18 Pt` (12,582,912-bit packed vector, 786,432 elements),
   continuing the non-p2 outer-dimension thread from W569/W571.

3. **Variant C — module-scope 9-D array-of-struct variable initialized from a
   call with indexed field writes.**
   Compose call-return CSE with module-scope mutation at `[2]^9 Pt`
   (2,097,152-bit packed vector, 65,536 elements). Exercises the same signed
   packed-slice write/read path one rank higher while staying well under the
   4-MiBit simulation cliff.
