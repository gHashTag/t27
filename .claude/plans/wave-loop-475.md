# Wave Loop 475 — Decomposed Implementation Plan

**Issue:** (to be opened; parent #1447)  
**Branch:** `wave-loop-475`  
**Variant:** B — compiler-backend aggregate hardening (bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the remaining `gen-verilog` aggregate-lowering tail deferred from Wave Loop 474:

1. Array-of-struct equality for arrays whose element struct has array-typed fields.
2. Whole-struct equality for scalar structs that contain array-typed fields.
3. Function-local arrays of structs passed as array parameters.
4. An adversarial yosys-elaboration witness that combines the three features above with module-level AOS return writeback.

End the wave with a green suite, resealed NMSE seals, a close-out report, and three Wave Loop 476 cooperation variants.

---

## Background (why now)

Wave Loop 474 introduced *memory-mode* lowering: a function-local array of structs whose element struct contains an array-typed field is emitted as per-field unpacked memories (`local_shape_pts [0:N-1][0:2]`) rather than flat per-element per-field scalar registers. That wave also added scalar-struct / small array-of-struct equality and module-level AOS return unpacking. The remaining tail lives at the intersection of equality, nested array-typed fields, and array-parameter binding.

The physical bench is still blocked (missing DLC10 cable / unwired P12 relay), so Variant B is selected by default. The scope is intentionally small and reviewable.

---

## Research summary

### Project weak spots (ranked)

- **P0 — Physical boot-evidence gap.** Live cold-POR CCLK sweeps on the Wukong XC7A100T remain blocked by missing hardware. Not solvable in software; Variant A is only viable if the cable/relay materialize.
- **P1 — gen-verilog aggregate-lowering tail.**
  - Array-of-struct equality for structs whose fields are arrays.
  - Whole-struct equality for nested structs with array-typed fields.
  - Function-local AOS passed as array parameters.
- **P2 — Master-merge divergence.** The `master` fix set at `701d79b3b` is still deferred; it should be re-integrated in its own small wave, not slipped into W475.
- **P3 — Formal / Lean gap.** No Lean proof yet that the per-field memory model preserves source read/write semantics. Optional Variant C fallback if Variant B grows beyond one wave.

### Scientific / engineering context

- **AoS/SoA layout transformation** is an active research thread (ICPE 2025, PPAM 2024/2025). Vitis HLS 2025 already disaggregates arrays of structs into per-member arrays by default. t27's per-field memory model matches commercial practice.
- **Lean 4 HDLs are now a real category:** Sparkle/Verilean and CktFormalizer demonstrate dependent types + native theorem proving + Verilog extraction at scale. T27's formal layer must become concrete to maintain differentiation.
- **Ternary accelerators are mainstreaming** (TOM ASIC, TerEffic FPGA, T-SAR CPU/SIMD, KU Leuven ternary-lut-dse). t27's moat is the full-stack spec-to-bitstream + numeric conformance + balanced-trit ISA, not just ternary MACs.
- **Sealed numeric conformance** is no longer unique (June 2026 GoldenFloat paper, 83-format catalog). t27 must enforce these properties through synthesis and bitstream, not just simulation.

---

## Work breakdown

### Task 1 — Array-of-struct equality with nested array-typed fields

**Files:** `bootstrap/src/compiler.rs`

**Spec-first regression:** `specs/scratch/w475_nested_field_equality.t27`

```t27
struct Pt { x: u8, y: u8 }
struct Shape { pts: [3]Pt }

fn eq_shape(a: Shape, b: Shape) -> bool { a == b }
fn eq_shapes(a: [2]Shape, b: [2]Shape) -> bool { a == b }

spec w475_nested_field_equality {
    test scalar_nested_struct_eq {
        given s1 = Shape { pts = [3]Pt { Pt { x = 1, y = 2 }, Pt { x = 3, y = 4 }, Pt { x = 5, y = 6 } } }
        given s2 = Shape { pts = [3]Pt { Pt { x = 1, y = 2 }, Pt { x = 3, y = 4 }, Pt { x = 5, y = 6 } } }
        then s1 == s2
    }
    test aos_nested_field_eq {
        given a = [2]Shape { ... }
        given b = [2]Shape { ... }
        then a == b
    }
}
```

**Implementation:**
- Extend `gen_verilog_pack_array_of_struct_expr` to handle the case where the element struct has an array-typed field.
- When the source is a memory-mode local array (`local_shapes`), emit a nested loop over outer and inner indices, reading `local_shapes_pts[outer][inner]` and concatenating the packed inner element (`Pt`) into the comparison vector.
- When the source is a module-level per-field memory, use the existing `module_struct_array_*` metadata to compute the same concatenation.
- Preserve the existing fallback to the generic path when the struct layout is unsupported.

**Verification substep:**
- `cargo test -p t27c --bin t27c` stays green.
- `./scripts/tri test --fast` passes after resealing.

---

### Task 2 — Whole-struct equality for scalar structs with array-typed fields

**Files:** `bootstrap/src/compiler.rs`

**Spec-first regression:** included in `specs/scratch/w475_nested_field_equality.t27`

**Implementation:**
- Add a new helper `gen_verilog_pack_scalar_struct_expr` that recursively packs a scalar struct expression.
- For a field that is an array of scalar structs (`pts: [3]Pt`), iterate inner indices and pack each element.
- For a field that is an array of primitive scalars, pack each element directly.
- For nested scalar structs, recurse.
- Use the existing `packed_width` to size every slice.
- Hook this into the `==`/`!=` path for `ExprStructLit`, struct variables, and function-call returns whose type is a scalar struct with array fields.

**Verification substep:**
- New scratch spec passes yosys elaboration.
- Existing W467-W474 scratch specs still pass.

---

### Task 3 — Function-local arrays of structs passed as array parameters

**Files:** `bootstrap/src/compiler.rs`

**Spec-first regression:** `specs/scratch/w475_local_aos_param.t27`

```t27
struct Pt { x: u8, y: u8 }
fn sum_pts(pts: [3]Pt) -> u16 { pts[0].x + pts[1].x + pts[2].x }

spec w475_local_aos_param {
    test local_aos_param_call {
        given local_pts = [3]Pt { Pt { x = 1, y = 2 }, Pt { x = 3, y = 4 }, Pt { x = 5, y = 6 } }
        when total = sum_pts(local_pts)
        then total == 9
    }
}
```

**Implementation:**
- Extend the array-parameter binding pass to recognize function-local arrays of structs as valid array arguments.
- For memory-mode local arrays, bind by passing either:
  - a packed vector parameter and slicing inside the callee (simpler for small arrays), or
  - per-field memory references when the callee needs indexed access and the local array is memory-mode.
- The safer default is to pack the local array into a temporary vector and pass that vector, then let the callee's existing scalar-struct parameter slicing (`try_resolve_struct_array_field_path`) read the packed vector.
- Update `call_array_param_signature` to include the local array identifier in the signature.
- Ensure clone creation still groups by binding signature deterministically.

**Verification substep:**
- New scratch spec passes.
- Existing array-parameter specs (w461-w464) still pass.

---

### Task 4 — Adversarial yosys-elaboration witness

**Files:** `specs/scratch/w475_adversarial_nested_equality.t27`

**Implementation:**
- Combine in one spec:
  - a module-level AOS return initializer (`var shapes : [2]Shape = make_shapes();`),
  - a nested-field equality check in a `test` block,
  - a function that accepts a local AOS parameter and compares it with a module array.
- The spec must force Verilog generation that exercises both memory-mode local arrays and module-level per-field memories in the same module.
- Ensure it passes yosys elaboration with `-DSIMULATION`.

**Verification substep:**
- `t27c gen-verilog specs/scratch/w475_adversarial_nested_equality.t27 | yosys -q -p 'read_verilog -sv -DSIMULATION ...'` passes.

---

### Task 5 — Verification, reseal, and stage-0 hash

**Steps:**
1. Run `cargo test -p t27c --bin t27c`.
2. Run `./scripts/tri test --fast` and fix any seal mismatches or failures.
3. Run full `./scripts/tri test` and confirm `TOTAL FAILURES: 0`.
4. Reseal affected specs with `t27c seal --save`.
5. Refreeze `bootstrap/stage0/FROZEN_HASH` after compiler changes.

---

### Task 6 — Close-out report and Wave Loop 476 cooperation variants

**Files:**
- `docs/reports/WAVE_LOOP_475_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W476_2026-07-08.md`
- `.trinity/ring-475.md`
- `.trinity/experience.md`
- `.trinity/current-issue.md`
- `docs/NOW.md`
- `~/.claude/projects/-Users-playra-t27/memory/wave-loop-475.md`
- `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md`

**Close-out report must include:**
- Summary of Variant B selection.
- What landed (Tasks 1–4).
- Weak spots and related work.
- Not done (hardware blockers, master-merge, formal gaps).
- Verification numbers.
- Next wave pointers.

**W476 cooperation variants (typical structure):**
- **Variant A:** live cold-POR CCLK sweep if hardware appears.
- **Variant B (default):** continue compiler-backend hardening — e.g., array-of-struct equality for deeper nesting, module-level scalar struct const/var/compare, or local struct arrays returned through array parameters.
- **Variant C (fallback):** formal synthesizability/correctness lemmas in Lean 4 for the per-field memory model.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Local AOS parameter passing interacts badly with clone signatures | Add one small spec first, run `--fast`, inspect generated Verilog before adding the witness. |
| Nested-field equality produces oversized concatenations | Pack only up to the existing `packed_width`; fall back to generic path for unsupported shapes. |
| Partial reseal leaves unrelated specs red | Reseal all affected specs after each task; run full suite before close-out. |
| Scope grows beyond one wave | Strictly defer Lean formal lemmas to Variant C; do not start them in Variant B. |

---

## Definition of done

- [ ] `specs/scratch/w475_nested_field_equality.t27` added and green.
- [ ] `specs/scratch/w475_local_aos_param.t27` added and green.
- [ ] `specs/scratch/w475_adversarial_nested_equality.t27` added and yosys-clean.
- [ ] All affected seals resealed.
- [ ] `bootstrap/stage0/FROZEN_HASH` refrozen.
- [ ] `./scripts/tri test` reports `TOTAL FAILURES: 0` and `ACCEPTABLE: yes`.
- [ ] `cargo test -p t27c --bin t27c` reports 1524 passed, 0 failed, 2 ignored.
- [ ] Close-out report and W476 cooperation variants written.
- [ ] `wave-loop-476` branch created and pointed at the W475 close-out commit.
- [ ] Memory entry and `MEMORY.md` updated.

---

*φ² + φ⁻² = 3 | TRINITY*
