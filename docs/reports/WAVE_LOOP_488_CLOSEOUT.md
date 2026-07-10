# Wave Loop 488 — Close-out Report

**Date:** 2026-07-07  
**Branch:** `wave-loop-488`  
**Issue:** #1458  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant B — Continue backend hardening** from
`docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md`.

The original Variant B proposal targeted three sub-fixes:

1. Colon-style struct-literal field separators.
2. Non-synthesizable struct fields beyond zero placeholders.
3. Wildcard array-of-struct aliases with array-typed fields.

After investigation and a short implementation attempt, **sub-fix #3 was the
safe, completable slice for W488**. Sub-fixes #1 and #2 were deferred to W489
because enabling colon struct literals exposed latent function-scope duplicate
struct-local declarations and keyword-name collisions in `igla/` specs that
require a dedicated hoisting/renaming pass rather than a single-wave parser
change.

---

## What was implemented

### Wildcard array-of-struct aliases with array-typed fields

`bootstrap/src/compiler.rs`: extended the AOS alias branch in
`gen_verilog_const` (around the W487 anonymous-copy code) so that when the
element struct of a module-scope array-of-structs contains an array-typed field,
the alias emits a multi-dimensional per-field memory and copies every inner
element in an `initial` block.

Before:
```verilog
// let _ = pts emitted as anonymous AOS copy _wildcard_copy_0
reg [31:0] _wildcard_copy_0_x [0:3];
// (AOS alias contains array-typed fields; copying those is not yet implemented)
```

After:
```verilog
reg [31:0] _wildcard_copy_0_x [0:3];
initial begin ... end
reg [7:0] _wildcard_copy_0_coords [0:3][0:2];
initial begin
    _wildcard_copy_0_coords[0][0] = pts_coords[0][0];
    ...
end
```

### Witness spec

`specs/scratch/w488_wildcard_aos_array_field_alias.t27` covers the new path:
a module-scope `let _ = pts;` alias where `pts : [4]Pt` and `Pt` has a
`coords: [3]u8` array-typed field.

---

## What was attempted and rolled back

A guarded colon-style struct-literal parser (`field: value`) and a test-block
local-struct-variable emission fix were prototyped. They parsed the existing
`igla/` and `scratch/` colon struct literals correctly, but the generated
Verilog then hit:

- duplicate `reg` declarations for same-name struct locals inside functions
  (`igla/race/backend.t27` local named `assign`, `igla/race/yosys.t27` local
  named `body`),
- malformed packed-vector indexing for array-typed fields of packed scalar
  struct locals,
- missing keyword-name escaping for struct-local identifiers.

These are real `gen-verilog` lowering gaps, but fixing them properly requires a
function-scope local-deduplication/renaming pass and a clear host-only policy
for `string`/`f32` struct fields. Rolling the colon parser back keeps W488
green and leaves the gaps documented for W489.

---

## Verification

- 673 / 673 non-smoke PASS.
- 153 / 153 yosys smoke PASS, 0 failures.
- 153 / 153 Icarus smoke PASS, 0 documented baseline failures.
- 673 / 673 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 673 specs: 0.**
- NMSE reseal: FROZEN_HASH and `repro/numerics/nmse_manifest*.json` refreshed.

---

## Artifacts

- Implementation: `bootstrap/src/compiler.rs`
- Witness: `specs/scratch/w488_wildcard_aos_array_field_alias.t27`
- Plan: `.claude/plans/wave-loop-488.md`
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`
- NMSE seal: `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest.json`,
  `repro/numerics/nmse_manifest_protocol_v1.json`

---

*φ² + φ⁻² = 3 | TRINITY*
