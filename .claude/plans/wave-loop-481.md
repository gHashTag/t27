# Wave Loop 481 — Decomposed Plan

**Branch:** `wave-loop-481` (create from `wave-loop-480`)  
**Variant:** B (default) — further reduce the Icarus Verilog baseline  
**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak spots discovered

After W480, the Icarus smoke gate is honest: 126/130 targets pass, 4 `igla/` specs fail in documented categories.

| Spec | Root cause | Representative error |
|------|-----------|----------------------|
| `igla/coder/eval.t27` | Array-of-struct parameter `results: []EvalResult` is passed as a single scalar input, but the function body references `results[idx].test_pass` as if it were a module-level struct array with per-field memories (`results_test_pass`). | `Unable to bind wire/reg/memory 'results_test_pass'` |
| `igla/coder/pipeline.t27` | Function `run_forward` returns a struct (`PipelineResult` with `.logits`), but the return is replaced by a placeholder because the callee is a namespace import. The caller then accesses `out.logits` on the undefined `out`. | `Unable to bind wire/reg/memory 'out_logits'` |
| `igla/race/formal.t27` | Parameter `m: RtlModule` is an imported struct defined in `igla/race/rtl.t27`. The single-file Verilog backend does not know the field layout, so it cannot unpack `m` into per-field local regs (`m_assigns`, etc.). | `Unable to bind wire/reg/memory 'm_assigns'` |
| `igla/race/rtl.t27` | Array-of-struct parameters (`inputs: []Signal`, `assigns: []Assignment`, etc.) are passed as scalar packed-vector inputs but accessed element-wise inside recursive emitters. | `Unable to bind wire/reg/memory 'assigns_lhs'`, `inputs_name`, etc. |

### 1.1 Why these are real weak spots

- The Verilog backend already has specialized paths for **module-level** arrays of structs (`module_struct_array_fields`) and **local/bench-local** arrays of structs (`local_struct_array_fields`), but it has **no path for array-of-struct function parameters**.
- Imported struct types are loaded by the typechecker but their field layout is not propagated into `VerilogCodegen::struct_fields`, so parameters of imported struct types cannot be unpacked.
- Struct-return field access on a call whose callee is not emitted (namespace placeholder) leaves an undefined local binding that Icarus cannot resolve.

### 1.2 What can be closed in W481

A realistic target is to move **3 of the 4** specs from baseline to passing, leaving only `pipeline.t27` (which depends on multiple namespace imports and genuinely host-side runtime helpers) or `rtl.t27` (if the AOS parameter lowering is too invasive).

| Fix | Expected impact |
|-----|-----------------|
| Array-of-struct parameter lowering | Closes `eval.t27` and most of `rtl.t27`. |
| Imported struct field propagation | Closes `formal.t27`. |
| Struct-return placeholder with zero-initialized fields | Closes `pipeline.t27`'s `out_logits` bind error. |

Target outcome: **129/130 Icarus smoke PASS, ≤1 documented baseline**.

---

## 2. Scientific / technical literature

| Finding | Source | How it applies |
|---------|--------|----------------|
| Liveness analysis must mark fields/elements live when accessed, otherwise DCE deletes live variables. | Rust `rustc_passes/dead.rs` ([source](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_passes/dead.rs.html)) | W480 already followed this pattern; W481 extends it to parameter destructuring. |
| HLS compilers lower arrays of structs either as per-member arrays (disaggregation) or as one packed vector (aggregation). | AMD Vitis HLS `AGGREGATE` pragma docs ([Structs](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs), [Examples of Aggregation](https://docs.amd.com/r/2023.2-English/ug1399-vitis-hls/Examples-of-Aggregation)) | We choose packed-vector aggregation for t27 function parameters because it matches the existing packed-width infrastructure and avoids emitting multiple function ports per member. |
| Intel HLS maps composite types to wide RTL signals, first-declared member in the low-order bits. | Intel HLS Compiler Reference ([Mapping HLS Data Types to RTL Signals](https://docs.altera.com/r/docs/683349/24.1/altera-high-level-synthesis-compiler-pro-edition-reference-manual/mapping-hls-data-types-to-rtl-signals?contentId=hXnMHdHcEdQ9%7EkybZQNX0g)) | Confirms our packed-vector slicing order: MSB-first concatenation in Verilog corresponds to first-declared field in the LSB of the packed vector (we preserve existing `struct_return_width` slicing convention). |
| HDL compilation should resolve scope and variable visibility before emitting Verilog. | LiveHD paper, *Design Decisions in LiveHD for HDLs Compilation* ([PDF](https://capra.cs.cornell.edu/latte21/paper/28.pdf)) | Supports resolving imported struct layouts at parse/import time and keeping the Verilog emitter purely structural. |
| Icarus Verilog supports a growing but incomplete SystemVerilog subset; unsupported constructs should be classified, not silently emitted. | Icarus README / GitHub ([steveicarus/iverilog](https://github.com/steveicarus/iverilog)) | Justifies keeping the namespace/helper placeholders and documenting residual baselines honestly. |

---

## 3. Decomposed implementation steps

### 3.1 Propagate imported struct field layouts into `struct_fields`

**File:** `bootstrap/src/compiler.rs`

- When the parser/typechecker resolves a `use` import, also resolve the imported `.t27` file and read its `struct_fields` layout.
- Merge the imported struct definitions into `self.struct_fields` at the start of `gen_verilog_module`.
- Alternative for this wave: if full import resolution is too large, emit imported struct parameters as a packed vector and defer field access to the unsupported placeholder (this is acceptable for `formal.t27` but does not fully close it).

### 3.2 Lower array-of-struct parameters as packed vectors with per-element local memories

**File:** `bootstrap/src/compiler.rs`

- In `gen_verilog_fn_internal`, when a parameter type is an array of structs (e.g. `[]EvalResult`, `[]Signal`, `[]Assignment`):
  - Compute the total packed width.
  - Emit a single scalar `input [TOTAL-1:0] pname`.
  - At the top of the function body, declare per-field local memories/registers for each struct element, e.g. `pname_field_0`, `pname_field_1`, ... up to the array size.
  - Unpack the packed vector into these per-element, per-field registers.
- Update `gen_verilog_expr` field/index access on parameter names to route to these local regs/memories.
- This matches the existing `module_struct_array_fields` approach but applies to function parameters.

### 3.3 Struct-return field access on unsupported calls

**File:** `bootstrap/src/compiler.rs`

- When a call is replaced by a placeholder and its return type is a struct/tuple, emit a zero-initialized packed vector of the correct width and assign it to a local temporary so that subsequent field access (`out.logits`) resolves to packed-vector slices.
- For struct-return calls that are unsupported, the Verilog function/task can still return the zero vector; the caller's field access will slice from that vector.

### 3.4 Update baseline JSON and classification

**File:** `docs/reports/gen_verilog_iverilog_smoke_baseline.json`

- Remove specs that now pass.
- Update classification and note for any remaining baselines.

### 3.5 Witness spec for closed classes

**File:** `specs/scratch/w481_aos_param_and_imported_struct.t27`

- Exercises array-of-struct parameter access (`fn sum_results(xs: []Pt) -> u32`) and imported struct parameter field access.
- Contains tests/invariants.
- Passes yosys and Icarus.

### 3.6 Update ring metadata

**Files:** `.trinity/current-issue.md`, `.trinity/ring-481.md` (create), `docs/reports/WAVE_LOOP_481_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W482_*.md`.

---

## 4. Verification plan

1. `cargo build --release` — must compile.
2. Manual spot check: generate and Icarus-compile `eval`, `pipeline`, `formal`, `rtl`.
3. `./scripts/tri test` full — target ≥129/130 Icarus PASS, ≤1 baseline, all non-smoke green, seals green.
4. `cargo test -p t27c --bin t27c` — must pass.
5. Reseal specs whose generated code changed.

---

## 5. Deliverables

- Code changes in `bootstrap/src/compiler.rs`.
- Updated `docs/reports/gen_verilog_iverilog_smoke_baseline.json`.
- New witness spec `specs/scratch/w481_aos_param_and_imported_struct.t27`.
- Resealed artifacts.
- `docs/reports/WAVE_LOOP_481_CLOSEOUT.md`.
- `docs/reports/FPGA_LOOP_COOPERATION_W482_2026-07-11.md` with Variants A/B/C.
- Memory update.

---

## 6. Risk / trade-off discussion

| Approach | Effort | Regression risk | Honesty of gate |
|----------|--------|-----------------|-----------------|
| **A. Full multi-file import lowering + AOS parameter pass** | High | High | High — closes everything but large change. |
| **B. Packed-vector AOS parameter lowering + imported struct propagation (recommended)** | Medium | Medium | High — closes eval/formal/rtl, keeps pipeline honest if helpers remain unsupported. |
| **C. Document the 4 baselines and formalize subset in Lean** | Medium | Low | Medium — no immediate Icarus count improvement. |

**Recommended:** Approach B. It directly continues W480, addresses the remaining structural root causes, and produces a measurable, honest improvement in the Icarus gate without forcing host-side code into hardware.

---

*φ² + φ⁻² = 3 | TRINITY*
