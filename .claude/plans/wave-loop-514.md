# Wave Loop 514 — Propagate ram_style / rom_style pragmas to packed structs and AOS

**Issue:** #1483 (placeholder — to create when GitHub token is available)
**Branch:** `wave-loop-514`
**Variant:** A from `docs/reports/FPGA_LOOP_COOPERATION_W514_2026-07-07.md`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the W509–W513 packed-vector lowering so that `ram_style` and `rom_style`
pragmas are honored for:
1. module-level const/var **packed scalar structs** (`reg [W:0] x;` / `localparam [W:0] x = …;`),
2. module-level **packed arrays-of-structs** (`reg [W:0] base [0:N-1];`),
3. bench-local **packed arrays-of-structs** (hoisted to module scope),
4. function-local **packed arrays-of-structs** (W513 follow-up).

For `rom_style` on read-only module constants, the current `localparam` /
`initial` emission is already read-only; the pragma mainly documents intent to
synthesis. For `ram_style` on variables, the pragma must appear immediately
before the packed `reg` declaration so Vivado/yosys can map the memory to the
requested primitive.

---

## Weak points found in W513 closeout

- The module-level packed scalar struct const/var paths in
  `gen_verilog_const` / `gen_verilog_var` emit the packed `localparam`/`reg`
  but never emit `node.extra_pragma`.
- The function-local packed AOS branch added in W513 emits the packed-vector
  memory but also skips `node.extra_pragma`.
- Bench-local and module-level packed AOS already honor the pragma; the gap is
  therefore narrow and localized.
- There is no witness exercising a packed scalar struct or packed AOS with a
  synthesis pragma, so the yosys smoke gate does not currently cover this path.

---

## Scientific / practical anchors

- **SystemVerilog packed arrays** (IEEE 1800-2017 §7.6): packed dimensions
  guarantee a contiguous bit layout, which is exactly the invariant used by
  t27's `packed_width` / `packed_field_offset` helpers. Unpacked memories of
  packed vectors (`reg [W:0] mem [0:N-1]`) remain addressable by variable index.
- **Vivado synthesis guidelines** (UG901 / UG912): `(* ram_style = "block" *)`
  / `(* rom_style = "block" *)` attributes are placed on the memory
  declaration. Unpacked-array-of-packed-vector style is the recommended pattern
  for BRAM inference in Vivado; fully packed 2-D arrays can fail to infer RAM.
- **Vitis HLS struct handling** (UG1399): interface structs are aggregated into
  a single wide vector in declaration order, while internal arrays-of-structs are
  disaggregated per member. t27's packed-vector approach is closer to the
  interface-aggregation style, but applied to internal storage.
- **CompCert memory model** (Leroy & Blazy; Krebbers, Leroy & Wiedijk): provides
  the formal foundation for arrays of structs, byte-level copying, and
  end-of-array semantics. This anchors the t27 semantic model in a proven C
  memory framework.

Sources:
- [IEEE 1800-2017 SystemVerilog packed arrays reference (via UCSD CSE141L)](https://cseweb.ucsd.edu/classes/sp11/cse141L/lab3b/system_verilog.html)
- [AMD UG901 — Packed and Unpacked Arrays](https://docs.amd.com/r/2022.2-English/ug901-vivado-synthesis/Packed-and-Unpacked-Arrays)
- [AMD UG912 — RAM_STYLE property](https://docs.amd.com/r/en-US/ug912-vivado-properties/RAM_STYLE)
- [AMD UG1399 — Vitis HLS Alignment Rules and Semantics](https://docs.amd.com/r/2023.2-English/ug1399-vitis-hls/Vitis-HLS-Alignment-Rules-and-Semantics)
- [Xavier Leroy & Sandrine Blazy — Formal verification of a C-like memory model](https://xavierleroy.org/publi/memory-model-journal.pdf)

---

## Decomposed plan

### 1. Reconnaissance (E/O)

- Re-read `docs/reports/WAVE_LOOP_513_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W514_2026-07-07.md`.
- Confirm the four pragma-gap sites in `bootstrap/src/compiler.rs`:
  - `gen_verilog_const` packed scalar struct const (line ~12818).
  - `gen_verilog_var` packed scalar struct var (line ~13280).
  - `gen_verilog_local_decl_hoisted` bench-local packed AOS (already done, verify).
  - `gen_verilog_stmt` / `StmtLocal` function-local packed AOS (W513 branch).
- Confirm `extra_pragma` is populated by the parser for `ram_style` and
  `rom_style` (line ~1239).

### 2. Backend implementation (C)

In `bootstrap/src/compiler.rs`:

- In `gen_verilog_const`, when emitting a packed scalar struct const, emit
  `(* {node.extra_pragma} *)` before the `localparam` declaration when the pragma
  is non-empty.
- In `gen_verilog_var`, when emitting a packed scalar struct var, emit
  `(* {node.extra_pragma} *)` before the `reg` declaration when the pragma is
  non-empty.
- In the W513 function-local packed AOS branch, emit the pragma before the packed
  `reg` declaration (same pattern as bench-local packed AOS).
- Keep module-level and bench-local packed AOS paths unchanged (already correct).
- Ensure no double pragma emission when the declaration is hoisted to module
  scope and then revisited in the bench `initial` block.

### 3. Scratch witnesses (C)

- `specs/scratch/w514_packed_struct_ram_style.t27`
  - Module-level packed scalar struct var with `pragma ram_style = "block";`.
  - Read/write field values in a bench and assert correctness.
- `specs/scratch/w514_packed_struct_rom_style.t27`
  - Module-level packed scalar struct const with `pragma rom_style = "block";`.
  - Read-only field access in a bench.
- `specs/scratch/w514_packed_aos_ram_style.t27`
  - Function-local packed AOS with `pragma ram_style = "block";`.
  - Mutate elements and read them back in a bench.

Each witness must contain `test`, `invariant`, and `bench` per L4.

### 4. Lean model / proof (C/V)

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Add W514 environments and modules mirroring the scratch witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Add `Module.isLowerable` and value-preservation theorems.
  - The pragma is not modeled in the shallow Lean Verilog semantics, so the
    theorems should assert that lowerability and value equivalence still hold
    when `extra_pragma` is present (the pragma only affects synthesis, not the
    computational semantics).

### 5. Validation (V)

- `cargo test -p t27c --bin t27c` must report 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable` must be acceptable with only the
  documented W508 early-exit baselines as smoke failures.
- `./scripts/tri verify --lean-lowerable` must pass with 252 lowerable specs and
  0 disagreements.
- `lake build Trinity.IcarusLowerable.Soundness` must be green with zero `sorry`
  in IcarusLowerable modules.
- Reseal the three new W514 scratch specs and any existing spec whose generated
  Verilog layout changed.

### 6. Reporting and next-wave planning (L)

- Write `docs/reports/WAVE_LOOP_514_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W515_2026-07-07.md` with three
  variants for W515, e.g.:
  - Variant A: extend packed AOS lowering to 2-D/3-D arrays and nested struct
    arrays.
  - Variant B: clear remaining W508 break/continue smoke baselines.
  - Variant C: add packed-AOS copy/assignment between module-level and
    function-local storage with full alias analysis.
- Update `.trinity/current-issue.md` and `docs/NOW.md` for W515.
- Save W514 learnings to memory.

---

*φ² + φ⁻² = 3 | TRINITY*
