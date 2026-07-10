# Wave Loop 483 Plan — Imported struct-return call lowering

**Date:** 2026-07-07  
**Branch:** `wave-loop-483`  
**Variant:** B  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Make imported scalar struct-return calls functional in the gen-verilog
backend. When a spec imports a zero-argument constructor that returns a scalar
struct, calls like `let r = supplier::make_metric()` must lower to a packed
local initialized from a packed struct literal, and field accesses like
`r.value` must resolve through slicing.

## Subtasks

1. **Discover inlinable imported constructors.**
   - Parse each imported spec at module load time.
   - For each zero-argument function whose body is exactly `return StructLit;`,
     record the fully-qualified struct type and ordered field initializer nodes.

2. **Declare packed locals for imported struct-return initializers.**
   - Extend `imported_struct_return_call` to use the inlinable map so `StmtLocal`
     emits a packed `reg [W-1:0]`.

3. **Inline the constructor at the call site.**
   - In the `ExprCall` unsupported-call path, before falling back to a sized-zero
     placeholder, check the inlinable map and emit a synthetic `ExprStructLit`
     packed concatenation via `try_emit_struct_literal_packed`.

4. **Validate with witness specs.**
   - `specs/scratch/w483_imported_struct_return.t27` — single imported
     constructor call + parameter passing + adversarial double call.
   - Update `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27` to assert
     the real value of an imported struct-return field access.

5. **Reseal and run gates.**
   - Global reseal because the packed-local comment changes from `W482` to
     `W482/W483`.
   - `./scripts/tri test --fast`: 656/656 non-smoke, 136/136 yosys, 136/136
     Icarus, 0 seal mismatches.
   - `cargo test -p t27c --bin t27c`: 1525/0/2.

6. **Close-out docs.**
   - `docs/reports/WAVE_LOOP_483_CLOSEOUT.md`
   - `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`
   - Update `.trinity/current-issue.md`, `.trinity/ring-483.md`,
     `.trinity/experience.md`, `docs/NOW.md`, and memory.

## Outcome

All subtasks completed. Imported struct-return calls are functional and the
Icarus smoke gate remains at 0 documented baseline failures.
