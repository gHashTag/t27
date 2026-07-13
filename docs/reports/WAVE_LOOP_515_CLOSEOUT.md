# Wave Loop 515 Closeout — Function-local packed scalar struct copy initializers

**Issue:** #1484 (placeholder — GH_TOKEN unavailable)  
**Branch:** `wave-loop-515`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute a revised **Variant C** from the W515 cooperation plan: remove the real
unlowered boundary that function-local packed scalar struct variables cannot be
initialized by copying another packed struct value (local-to-local,
module-to-local, or return-to-local).

Reconnaissance showed that the originally recommended Variant A
(multi-dimensional packed arrays-of-structs) was already largely implemented,
so the loop was pivoted to the next concrete boundary.

---

## What changed

### Backend (`bootstrap/src/compiler.rs`)

- Refined `copy_propagate` so that **mutable `var` bindings of struct-like type**
  are no longer copy-propagated away.
  - The root cause of the boundary was that `var b : S = a;` was treated as an
    alias of `a`. `propagate_ident` skips assignment LHS, so reads of `b` were
    rewritten to `a` while the field-mutating assignment `b.tag = 7;` still
    referred to the now-undeclared `b`, producing an unresolved field access.
  - The fix uses a small helper `type_is_struct_like` to detect struct types
    (non-empty, not an array shape, not a primitive scalar). Array and scalar
    `var` bindings keep the existing propagation behavior because the Verilog
    backend currently relies on aliasing for array-copy initializers.
- No new Verilog emission path was required: the existing packed scalar struct
  local path already emits a `reg [W:0]` declaration and correctly handles
  struct-literal initializers. Preserving the `var` declaration is enough for
  identifier-copy and return-copy initializers to lower as value-preserving
  bit-vector assignments.

### Scratch witnesses (`specs/scratch/`)

- `w515_local_packed_struct_copy.t27` — function-local `var b : S = a;`,
  mutate `b.tag`, assert `a` is unchanged.
- `w515_module_to_local_packed_struct_copy.t27` — function-local packed scalar
  struct initialized from a module-level packed struct var.
- `w515_local_packed_struct_return_copy.t27` — function returns a packed struct;
  caller copies the return value into a local packed struct var.

All three witnesses include `test`, `invariant`, and `bench` blocks per L4.

### Lean model / proof (`proofs/lean4/Trinity/IcarusLowerable/`)

- `Lemmas.lean`: added W515 environments and modules mirroring the three scratch
  witnesses (`w515LocalCopyEnv/Fn/Module`, `w515ModuleToLocalCopyEnv/Module`,
  `w515ReturnToLocalCopyEnv/Module`).
- `Soundness.lean`: added lowerability and value-preservation theorems for all
  three W515 shapes:
  - `w515_local_copy_lowerable`
  - `w515_local_copy_value_equiv`
  - `w515_module_to_local_copy_lowerable`
  - `w515_module_to_local_copy_value_equiv`
  - `w515_return_to_local_copy_lowerable`
  - `w515_return_to_local_copy_value_equiv`

### Seals

- Saved seals for the three new W515 scratch specs.
- Resealed 31 existing specs whose generated code layout changed because of the
  `copy_propagate` refinement.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo test -p t27c --bin t27c` | **1525 passed, 0 failed, 2 ignored** |
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | passed, 252 lowerable specs, 0 disagreements |
| `./scripts/tri test --icarus-lowerable --fast` | acceptable — 739/739 parse/typecheck/gen PASS, 0 seal mismatches, Icarus lowerability 0 disagreements |

Smoke summary from the fast suite run:

- Gen Verilog yosys smoke: 2 documented W508 `break` baselines.
- Gen Verilog Icarus smoke: 3 documented failures (W508 `continue` baseline +
  2 function-local pragma Icarus-syntax limitations: `w468_local_ram_style` and
  `w514_function_local_packed_aos_ram_style`).
- Total known failures match baseline; no new failures.

---

## Residual boundaries

- **Whole-array-field reads** from packed scalar structs / AOS are still not
  lowered (`var x : [3]u32 = a.vals;` produces placeholder per-element regs).
- **W508 `break`/`continue`** early-exit yosys/Icarus baselines remain.
- **Scalar and array `var` copy propagation** still aliases the source for
  non-struct types. This is a documented semantic quirk, not a regression, and
  is left for a future optimizer pass that can emit real value copies.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md` for three proposed
Wave Loop 516 variants:

- **Variant A (recommended):** whole-array-field reads from packed scalar structs
  and arrays-of-structs.
- **Variant B:** clear the remaining W508 `break`/`continue` smoke baselines.
- **Variant C:** packed scalar struct equality / comparison operators.

The current issue file is updated to point at W516:
`.trinity/current-issue.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
