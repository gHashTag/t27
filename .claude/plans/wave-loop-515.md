# Wave Loop 515 — Function-local packed scalar struct variables: copy initializers and cross-context copy

**Issue:** #1484 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-515`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Select and implement a W515 cooperation variant that removes a real, currently
unlowered boundary in the t27 Verilog backend: **function-local packed scalar
struct variables cannot be initialized from another packed struct value
(identifier copy), and cannot be copied from module-level packed struct vars.**

---

## Weak points discovered during reconnaissance

### 1. Multi-dimensional packed AOS is already mostly working

Probes showed that `[2][3]S` (where `S` is a lowerable scalar struct) already
lowers end-to-end, including:

- module-level 2-D packed AOS with `ram_style` pragma,
- function-local 2-D packed AOS with pragma,
- read/write of scalar fields and indexed scalar-array fields,
- passing 2-D packed AOS as function parameters.

Therefore the previous W515 **Variant A** ("multi-dimensional packed AOS with
pragma propagation") is largely complete. Remaining gaps in this area are
smaller: whole-array-field reads and nested-struct fields inside packed AOS,
both of which can be deferred to a later loop.

### 2. Function-local packed scalar struct variables lack copy semantics

The W509/W514 backend lowers a function-local `let s : S = S{...}` to a single
`reg [W:0] s` with pragma support. However:

- `var b : S = a;` (copy from another function-local packed struct) emits
  `32'd0 /* UNSUPPORTED_ICARUS: unresolved field access b.tag */ = 7;` and
  never declares `b`.
- `var s : S = m;` (copy from a module-level packed struct var) similarly
  fails: the local `s` is not declared and the assignment is unresolved.

This means the only supported initializer for a function-local packed scalar
struct is a struct literal or a struct-return call. Any value-preserving copy
from another packed struct is rejected. This is a real usability boundary for
FPGA designs that want to snapshot a module-level register into a function-local
register, or copy between locals.

### 3. Whole-array-field reads are not lowered

Reading an entire array-typed field as a value, e.g. `var x : [3]u32 = a.vals`,
produces uninitialized per-element regs and a placeholder comment. This is a
related but separate boundary; it is left out of this loop to keep the scope
bounded.

### 4. W508 early-exit baselines remain

`w508_break_nested`, `w508_break_search`, and `w508_continue_sum` are still
documented smoke failures. They are valid cleanup work but are left as Variant B
for this loop in case the primary Variant A work finishes early.

---

## Scientific / practical anchors

- **SystemVerilog packed structs/arrays (IEEE 1800-2017 §7.4, §7.6):** packed
  dimensions give a contiguous bit layout; copying a packed struct value is a
  simple bit-vector assignment. This is exactly the invariant used by t27's
  `packed_width` helper.
- **AMD Vivado synthesis guidelines (UG901 / UG912):** `(* ram_style = "block" *)`
  on a packed `reg` declaration guides BRAM/LUTRAM inference. The function-local
  packed scalar struct `reg [W:0]` pattern is consistent with Vivado's
  recommendation for small memories and registers.
- **CompCert memory model (Leroy & Blazy; Krebbers, Leroy & Wiedijk):** provides
  the formal foundation for byte-level struct copying and value-preserving
  assignment. t27's packed-vector lowering approximates the same value-copy
  semantics at bit-vector granularity.

Sources already in the repo:
- [IEEE 1800-2017 SystemVerilog packed arrays reference (via UCSD CSE141L)](https://cseweb.ucsd.edu/classes/sp11/cse141L/lab3b/system_verilog.html)
- [AMD UG901 — Packed and Unpacked Arrays](https://docs.amd.com/r/2022.2-English/ug901-vivado-synthesis/Packed-and-Unpacked-Arrays)
- [AMD UG912 — RAM_STYLE property](https://docs.amd.com/r/en-US/ug912-vivado-properties/RAM_STYLE)
- [Xavier Leroy & Sandrine Blazy — Formal verification of a C-like memory model](https://xavierleroy.org/publi/memory-model-journal.pdf)

---

## Selected variant

**Variant A (revised): Function-local packed scalar struct variables —
identifier-copy and cross-context copy with pragma propagation.**

This directly extends W514's pragma work and removes a concrete unlowered
boundary. It is bounded, testable, and leaves the repo releasable.

---

## Decomposed plan

### 1. Reconnaissance / weak-point confirmation (E/O)

- Confirm with probes that `var b : S = a;` and `var s : S = m;` fail.
- Identify the exact code paths in `gen_verilog_stmt` where identifier-copy
  initializers are ignored for packed scalar struct locals.

### 2. Backend implementation (C)

In `bootstrap/src/compiler.rs`:

- In the function-local packed scalar struct branch, detect when the initializer
  is an `ExprIdentifier` that names another packed scalar struct (local or
  module-level). Emit a single bit-vector assignment `safe_b = safe_a;` instead
  of falling through to the unresolved field-access path.
- Ensure the declaration is emitted before the assignment (it already is for
  struct-literal initializers; the copy path currently skips declaration).
- Preserve `node.extra_pragma` emission already added in W514.
- Add a guard so the copy is only used when the RHS type is the same packed
  struct type (or a type-compatible packed scalar struct). For mismatched types
  fall back to the existing behavior.

### 3. Scratch witnesses (C)

- `specs/scratch/w515_local_packed_struct_copy.t27`
  - Function-local packed scalar struct copy `var b : S = a;`, then mutate `b`
    and assert `a` is unchanged.
- `specs/scratch/w515_module_to_local_packed_struct_copy.t27`
  - Function-local packed scalar struct initialized from a module-level packed
    struct var, with `pragma ram_style = "block";` on the local.
- `specs/scratch/w515_local_packed_struct_return_copy.t27`
  - Function returns a packed struct; another function copies the return value
    into a local packed struct var.

Each witness must contain `test`, `invariant`, and `bench` per L4.

### 4. Lean model / proof (C/V)

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Add W515 environments/modules mirroring the scratch witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Add `Module.isLowerable` and value-preservation theorems for function-local
    packed scalar struct copy initializers. The copy is a bit-vector identity,
    so the proof should be straightforward given existing W509/W511 theorems.

### 5. Validation (V)

- `cargo test -p t27c --bin t27c` must report 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable` must be acceptable with only the
  documented W508 early-exit and W514 function-local pragma baselines.
- `./scripts/tri verify --lean-lowerable` must pass with 252 lowerable specs and
  0 disagreements.
- `lake build Trinity.IcarusLowerable.Soundness` must be green with zero `sorry`
  in IcarusLowerable modules.
- Reseal the three new W515 scratch specs and any existing spec whose generated
  Verilog layout changed.

### 6. Reporting and next-wave planning (L)

- Write `docs/reports/WAVE_LOOP_515_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W516_2026-07-07.md` with three
  variants for W516:
  - Variant A: whole-array-field reads from packed scalar structs / AOS.
  - Variant B: clear remaining W508 `break`/`continue` smoke baselines.
  - Variant C: nested struct-field lowering inside packed AOS elements.
- Update `.trinity/current-issue.md` and `docs/NOW.md` for W516.
- Save W515 learnings to memory.

---

*φ² + φ⁻² = 3 | TRINITY*
