# Wave Loop 478 — Decomposed Plan

**Branch:** `wave-loop-478`  
**Variant:** B (default) — close Icarus Verilog failures in generated RTL  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak spots discovered

The Wave Loop 477 Icarus gate exposed **33 failures** across `specs/scratch/*.t27` and the 27 `igla_clean_specs`:

| Class | Count | Root cause | Files affected |
|-------|-------|------------|----------------|
| A — indefinite-width concatenation | 12 scratch | `gen_verilog_pack_array_of_struct_literal` / `emit_struct_literal_leaf` emit unsized literals or `(expr & {width{1'b1}})` masks that Icarus rejects inside concatenations. | `w470_array_of_struct_return`, `w474_local_nested_struct_array`, `w474_module_aos_return_assign`, `w474_struct_equality`, `w475_nested_field_equality`, `w476_*` |
| B — whole-array / slice assignment | 2 scratch | `gen_verilog_struct_return_slicing` assigns a packed slice to an **unpacked memory** for array-typed struct fields; Icarus does not support array-slice assignment. | `w475_nested_field_equality`, `w476_nested_whole_struct_assign` |
| C — duplicate module-level regs | 2 scratch | The "Registers (from struct declarations)" block re-emits per-field regs already declared for module-level scalar struct vars. | `w469_module_scalar_struct*`, possibly others |
| D — duplicate test named blocks | 7 igla | A `test "name"` block is emitted twice when the source spec contains duplicate test names; Icarus rejects duplicate `begin : name` labels. | `igla/coder/tokenizer`, `igla/race/{adder_tree,cordic,cordic_fixed,cordic_top,opcodes,ternary_mac}` |
| E — source-level mismatch | 1 scratch | `w469_2d_struct_array` calls `set_and_sum_2d` with three arguments where the function takes two. | `w469_2d_struct_array` |
| F — unsupported dynamic constructs | 14 igla | t27 string/array methods (`.len`, `.contains`) and recursive helper calls are emitted as Verilog method/function calls that Icarus cannot elaborate. | `igla/coder/{bench_proxy,benchmark,dataset,eval,pipeline,weights}`, `igla/race/{backend,bram_weights,eda,formal,gemm,rtl,systolic_array,systolic_ternary}` |
| G — packed array-param indexing | 1 scratch | `sum_array_param(pts : [2]Pt)` is lowered to a 32-bit packed input and indexed as `pts[31:0][0][0]`, which is illegal. | `w469_struct_field_array_2d` |

Classes A–E and the duplicate test labels are fixable in this wave. Class F is a backend feature gap (dynamic arrays / methods) that is **out of scope** for one wave and will be documented as a baseline. Class G may be fixable if it shares the same packed-width miscalculation as class A; otherwise it becomes a tracked residual.

---

## 2. Scientific / technical literature mapped to the fixes

| Finding | Source | How it applies |
|---------|--------|----------------|
| Icarus strictly rejects unsized expressions as concatenation operands; sized literals and SystemVerilog casts (`<width>'(expr)`) give unambiguous widths. | [Icarus Verilog Quirks](https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html), [SourceForge bug #394](https://sourceforge.net/p/iverilog/bugs/394/) | Replace `(expr & {width{1'b1}})` with `<width>'(expr)` and emit sized literals (`<width>'d<value>`) in all packed struct/array literal packers. |
| SystemVerilog packed arrays allow per-element and per-slice assignment; unpacked arrays of structs must be assigned element-by-element to avoid tool-specific restrictions. | [SystemVerilog Arrays — Verilog Pro](https://www.verilogpro.com/systemverilog-arrays-synthesizable/), [Sutherland HDL synthesizable SV paper](https://sutherland-hdl.com/papers/2013-SNUG-SV_Synthesizable-SystemVerilog_paper.pdf) | For struct-return slicing into array-typed fields, expand one packed-vector slice into per-element memory writes. |
| Formal equivalence of code-motion / scheduling transformations is an established research area; value propagation can verify that reordered RTL preserves semantics. | Banerjee et al., *Verification of Code Motion Techniques Using Value Propagation*, IEEE TCAD 2014 ([DOI](https://doi.org/10.1109/tcad.2014.2314392)) | The W477 declaration-hoisting transform is a form of code motion; in a future wave the same literature can justify a Lean equivalence lemma. For W478, the focus is on making the emitted RTL simulator-clean. |
| Verified Verilog compilers and equivalence checkers (Lutsig, Vera) show that compiler correctness for HDL is an active research frontier. | [Lutsig — verified Verilog compiler](https://doi.org/10.1145/3437992.3439916), [Vera LATTE 2025](https://johnwickerson.github.io/papers/vera_LATTE25.pdf) | Supports Variant C fallback: formalizing t27c lowering passes in Lean 4 if direct fixes become too large. |

---

## 3. Decomposed implementation steps

### 3.1 Width-aware packed literal emission (Class A)
- Modify `emit_struct_literal_leaf` in `bootstrap/src/compiler.rs`:
  - For literal leaves emit `<width>'d<clean>`.
  - For non-literal leaves emit `<width>'(expr)` instead of `(expr & {width{1'b1}})`.
- Replace `gen_verilog_pack_array_of_struct_literal` with a call into the same width-aware leaf path (`emit_struct_literal_leaf` per element) so array-of-struct returns and equality packers also produce sized operands.
- Update `struct_field_widths` to use `packed_width` so array-typed fields get correct total width for slicing math.

### 3.2 Struct-return slicing for array-typed fields (Class B)
- In `gen_verilog_struct_return_slicing`, detect when a field type is an array (or contains arrays).
- For each element index combination, emit a per-element memory assignment from the corresponding packed slice of the return temporary, using `index_combinations` and `packed_width`.

### 3.3 Module-level scalar struct reg deduplication (Class C)
- In the "Registers (from struct declarations)" block, skip emitting struct field regs when the chosen prefix already belongs to a module-level scalar struct `var`/`const` that declared the same regs.
- Alternative/additional safety: track a set of emitted module-level reg names and skip duplicates.

### 3.4 Duplicate test label deduplication (Class D)
- In test assertion emission (`gen_verilog_test_stmt` / test-block generation), maintain a set of already-used test block labels per module and append a numeric suffix for duplicates.

### 3.5 Source-level mismatch (Class E)
- Fix `specs/scratch/w469_2d_struct_array.t27` so all calls to `set_and_sum_2d` pass exactly two arguments.

### 3.6 Icarus warning gate (hygiene)
- Extend `cmd_gen_verilog_iverilog_smoke` in `bootstrap/src/suite.rs` to parse `iverilog` stderr for both `error:` and `warning:` and count warnings in the machine-readable report; keep warnings non-fatal for this wave but surface them.
- Add `specs/scratch/w478_icarus_struct_array.t27` as an adversarial witness that exercises the previously failing patterns (AOS return, AOS equality, nested struct array copy) under both yosys and Icarus.

### 3.7 Packed array-param indexing (Class G, opportunistic)
- Investigate whether the 32-bit input for `sum_array_param(pts : [2]Pt)` is caused by `packed_width` not being used; if yes, fix the parameter width and the field-access lowering path to emit correct packed slices (`pts[high:low]`) instead of `pts[31:0][0][0]`.
- If the fix touches array-parameter clone binding, defer to a follow-up wave and document as residual.

---

## 4. Verification plan

1. `cargo build --release` — must compile.
2. Manual Icarus loop over all 125 Icarus targets; compare against baseline (92/125).
3. `./scripts/tri test --fast` — must report 0 non-baseline failures and 0 seal mismatches.
4. `./scripts/tri test` full — must report acceptable baseline.
5. `cargo test -p t27c --bin t27c` — must pass.
6. Global reseal if generated Verilog changed for any spec.

---

## 5. Deliverables

- Code changes in `bootstrap/src/compiler.rs` and `bootstrap/src/suite.rs`.
- Fixed source spec `specs/scratch/w469_2d_struct_array.t27`.
- New witness spec `specs/scratch/w478_icarus_struct_array.t27`.
- Updated seals (global reseal if needed).
- `docs/reports/WAVE_LOOP_478_CLOSEOUT.md`.
- `docs/reports/FPGA_LOOP_COOPERATION_W479_2026-07-08.md` with three variants.
- Updated `.trinity/current-issue.md`, `.trinity/ring-478.md`, `.trinity/experience.md`, `docs/NOW.md`.
- Memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-478.md` plus `MEMORY.md` index entry.

---

*φ² + φ⁻² = 3 | TRINITY*
