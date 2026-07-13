# Wave Loop 516 — Whole-array-field reads from packed scalar structs / AOS

**Issue:** #1485 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-516`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W516 cooperation plan: enable whole-array-field
reads from packed scalar structs and packed arrays-of-structs in the t27
Verilog backend.

---

## Weak points / scientific anchors

- **IEEE Std 1800-2017 §7.2.1 / §7.4.1 / §11.5.1**: a packed struct or packed
  array is a single contiguous bit vector; member access and part-selects
  extract sub-ranges. Whole-array-field reads are therefore bit-vector slices
  with a constant offset and width.
- **CompCert `Ctypes` / concrete memory model** (Leroy & Blazy; Besson, Blazy,
  Wilke): struct layout is computed as byte/bit offsets, and field reads are
  memory loads at a computed offset. t27's packed-vector lowering approximates
  the same offset-based extraction at bit granularity.
- **Lean 4 `bv_decide` / `BitVec.extractLsb'`** (POPL 2025 paper *Interactive
  Bitvector Reasoning using Verified Bit-Blasting*): struct lowering into
  `BitVec` plus slice extraction is the canonical way to reason about packed
  field reads. The W516 proof will mirror this with `native_decide`-style
  direct computation on concrete environments.

Sources:
- [IEEE Std 1800-2017 (MIT course mirror)](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- [IEEE Xplore 1800-2017](https://ieeexplore.ieee.org/document/8299595/)
- [CompCert Ctypes module](https://compcert.org/doc/html/compcert.cfrontend.Ctypes.html)
- [CompCert memory model](https://compcert.org/doc/html/compcert.common.Memory.html)
- [Besson, Blazy, Wilke — A Concrete Memory Model for CompCert](https://people.rennes.inria.fr/Frederic.Besson/concrete_memory_model.pdf)
- [Interactive Bitvector Reasoning using Verified Bit-Blasting (POPL 2025)](https://doi.org/10.1145/3763167)

---

## Selected variant

**Variant A: Whole-array-field reads from packed scalar structs / AOS.**

Reading an array-typed field as a whole value (`var x : [3]u32 = s.vals;`)
will lower as a single bit-vector slice of the parent packed `reg`, preserving
the value semantics needed by the Icarus-lowerable subset.

---

## Decomposed plan

### 1. Reconnaissance (O)

- Confirm with probes that `var x : [3]u32 = s.vals;` fails on current branch.
- Locate the placeholder code path in `bootstrap/src/compiler.rs`.
- Map the existing packed scalar struct local/module/return paths to identify
  the correct insertion point.

### 2. Backend implementation (C)

In `bootstrap/src/compiler.rs`:

- Extend the packed scalar struct field-access lowering to detect when the
  selected field is array-typed and the expression is used as a whole value
  (assigned to a local array variable, passed as argument, or returned).
- Compute the field's bit offset within the packed struct and the total
  packed width of the array field.
- Emit the array field as either:
  - a local packed `reg [W:0] field;` copy, or
  - a direct bit-vector slice expression using the parent packed value and the
    field offset/width.
- Extend the same logic for packed arrays-of-structs: `aos[i].field` where
  `field` is array-typed lowers as a slice of the selected AOS element.
- Ensure the change integrates with the existing local-array and
  local-packed-struct-array paths without duplicating code.

### 3. Scratch witnesses (C)

- `specs/scratch/w516_packed_struct_whole_array_field_read.t27`
  - Read `[3]u32` field from a packed scalar struct local, mutate the copy,
    assert original unchanged.
- `specs/scratch/w516_packed_aos_whole_array_field_read.t27`
  - Read `[2]u16` field from an element of a packed AOS, use the copy.
- `specs/scratch/w516_packed_struct_whole_array_field_return.t27`
  - Return an array-typed field from a packed scalar struct as a whole value.

Each witness must contain `test`, `invariant`, and `bench` per L4.

### 4. Lean model / proof (C/V)

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Add W516 environments/modules mirroring the scratch witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Add lowerability and value-preservation theorems for whole-array-field reads.
  - Use direct `native_decide` or `bv_decide`-style slice-equality proofs on
    concrete environments.

### 5. Validation (V)

- `cargo test -p t27c --bin t27c` must report 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable` must be acceptable with only the
  documented W508 early-exit and function-local pragma baselines.
- `./scripts/tri verify --lean-lowerable` must pass with 252 lowerable specs
  and 0 disagreements.
- `lake build Trinity.IcarusLowerable.Soundness` must be green with zero
  `sorry` in IcarusLowerable modules.
- Reseal the three new W516 scratch specs and any existing spec whose
  generated layout changed.

### 6. Reporting and next-wave planning (L)

- Write `docs/reports/WAVE_LOOP_516_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W517_2026-07-07.md` with three
  W517 variants:
  - Variant A: packed scalar struct equality / comparison operators.
  - Variant B: clear remaining W508 `break`/`continue` smoke baselines.
  - Variant C: nested struct-field lowering inside packed AOS elements.
- Update `.trinity/current-issue.md` and `docs/NOW.md` for W517.
- Save W516 learnings to memory.

---

*φ² + φ⁻² = 3 | TRINITY*
