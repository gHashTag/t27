# Wave Loop 480 — Decomposed Plan

**Branch:** `wave-loop-480` (create from `wave-loop-479`)  
**Variant:** B (default) — reduce the Icarus baseline by fixing small, classified root causes  
**Date:** 2026-07-09  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak spots discovered

### 1.1 Remaining Icarus baseline (17 specs)

After W479, the Icarus smoke gate is honest: 110/127 targets pass, 17 `igla/` specs fail in documented categories. The failures cluster into six concrete root causes, all fixable or documentable in one wave:

| Class | Count | Root cause | Representative symbol(s) | Files |
|-------|-------|------------|--------------------------|-------|
| **C1 — DCE/scope visibility** | 9 | `dead_store_elim` / reference analysis does not recognize array-index + field-access receivers (`results[idx].test_pass`), index variables (`idx`), or tuple-sliced temps (`_let_tmp_0`) as uses. The declaration is deleted, then Icarus cannot bind the identifier. | `results_test_pass`, `total`, `idx`, `decoded`, `pass_count`, `m_assigns`, `new_acc`, `mag_a`, `assigns_lhs`, `c`, `s` | `eval`, `bram_weights`, `cordic_fixed`, `eda`, `formal`, `gemm`, `rtl`, `systolic_array`, `ternary_mac` |
| **C2 — namespace-qualified calls** | 2 | `module::function(...)` is emitted literally; Icarus rejects the `::` syntax as a malformed statement. | `eval::generate_verilog`, `tokenizer::tokenize_prompt_hybrid`, `arch::forward_with_bank`, `arch::generate_next_token_unified` | `coder/benchmark`, `coder/pipeline` |
| **C3 — wildcard `_` assignment** | 2 | t27 allows `_ = expr` as a discard statement; Icarus has no variable named `_`. | `_` | `coder/tokenizer`, `race/opcodes` |
| **C4 — duplicate bench declarations** | 1 | A bench block name or its counter register is emitted twice under `ifndef SIMULATION`, causing Icarus redeclaration errors. | `_bench_cordic_top_latency_cycles`, `cordic_top_latency_bench` | `race/cordic_top` |
| **C5 — indefinite-width literal in concatenation** | 1 | Unsupported aggregate lowering emits `0 /* TODO: ... */` inside `{}`. Icarus sees `{0, 0}` with unsized decimal operands and reports "'sd0 has indefinite width". | `{0 /* TODO: array literal [s] ... */, 0 /* TODO: ... */}` | `race/cordic` |
| **C6 — host-side helper not emitted** | 2 | Functions such as `contains_multiply_in_rhs`, `is_const`, `is_power_of_two_const`, `ternary_mul` are referenced but never emitted as Verilog functions (or are module-qualified). | `contains_multiply_in_rhs`, `is_const`, `ternary_mul` | `race/backend`, `race/systolic_ternary` |

### 1.2 Why these are real weak spots

- **C1** is the biggest class. The W479 fix only added receiver recognition for *method calls* (`arr.len`). It missed field access, array indexing, and tuple slicing. Fixing it makes a large class of local-variable uses visible to DCE and to the Verilog symbol table.
- **C2/C3/C4** are emitter hygiene issues. They do not reflect unsupported t27 semantics; they reflect incomplete lowering hygiene.
- **C5** is a placeholder emission bug: the `0` placeholder needs an explicit width when used in a concatenation context.
- **C6** is the genuinely non-synthesizable residue. The `igla/` specs model host-side compiler passes (`backend`) and ternary PEs (`systolic_ternary`) that operate on AST-like data. These should remain in the baseline unless we add a major host-side evaluator pass.

### 1.3 What can be closed in W480

A realistic, honest target is to move **≈10 of the 17** specs from baseline to passing:

- **C1** can close 7–9 specs if the reference-analysis fix is general enough.
- **C3** closes 2 specs (`tokenizer`, `opcodes`).
- **C4** closes 1 spec (`cordic_top`).
- **C5** closes 1 spec (`cordic`).
- **C2** may close `benchmark` and/or `pipeline` if we replace namespace-qualified calls with the unsupported placeholder (which prevents syntax errors) or inline the callee.
- **C6** (`backend`, `systolic_ternary`) should stay in the baseline because they rely on host-side recursive helpers.

Expected outcome: **120/127 Icarus smoke PASS, ≤7 documented baselines**.

---

## 2. Scientific / technical literature

| Finding | Source | How it applies |
|---------|--------|----------------|
| DCE must preserve *all* value-observing contexts, including field access and indexing, to be semantics-preserving. | Blech, Gesellensetter & Glesner, *Formal verification of dead code elimination in Isabelle/HOL*, SEFM 2005 ([DOI](https://doi.org/10.1109/sefm.2005.20)) | Confirms that the W480 fix must extend reference analysis beyond method-call receivers to array-index and field-access receivers; otherwise DCE silently deletes live variables. |
| Bisimulation-based correctness of dead-variable elimination in functional IRs requires tracking variable occurrences in *all* syntactic positions. | Schneider, Smolka & Hack, *An Inductive Proof Method for Simulation-based Compiler Correctness*, arXiv:1611.09606 ([arXiv](https://ar5iv.labs.arxiv.org/html/1611.09606)) | Supports the decision to make `collect_refs_in` and `collect_reads` handle `ExprFieldAccess`, `ExprArrayAccess`, and tuple slicing uniformly. |
| Classical compiler optimizations, including DCE, are proved correct via rewrite rules with temporal-logic side conditions on variable liveness. | Lacey, Jones, Van Wyk & Frederiksen, *Proving correctness of compiler optimizations by temporal logic*, POPL 2002 ([DOI](https://doi.org/10.1145/565816.503299)) | Reinforces that the W480 fix should be framed as a liveness-side-condition fix, not an ad-hoc backend patch. |
| Tool-specific SystemVerilog subsets are standard practice; unsupported constructs should be documented, not silently emitted. | Sutherland & Mills, *Synthesizable SystemVerilog: Busting the Myth that SystemVerilog is only for Verification*, SNUG 2013 ([PDF](https://sutherland-hdl.com/papers/2013-SNUG-SV_Synthesizable-SystemVerilog_paper.pdf)) | Justifies the C2/C6 fallback: namespace-qualified calls and host-side recursive helpers are outside the Icarus-supported t27 subset and should emit a classified placeholder. |
| HDL compiler infrastructures (LiveHD) explicitly separate SSA/scope handling from Verilog emission to avoid variable-visibility bugs. | Mighuel et al., *Design Decisions in LiveHD for HDLs Compilation*, LATTE 2021 ([PDF](https://capra.cs.cornell.edu/latte21/paper/28.pdf)) | Suggests that t27 should keep strengthening reference/scope analysis *before* Verilog emission, which is exactly the C1 fix. |
| Featherweight Verilog shows that high-level HDL descriptions need a two-level type system and explicit handling of variable scope/storage. | Gillenwater, *Synthesizable High Level Hardware Descriptions*, PEPM 2008 ([PDF](https://jgillenw.com/pepm08.pdf)) | Supports adding explicit handling for wildcard identifiers and duplicate bench names as scope/namespace issues. |

---

## 3. Decomposed implementation steps

### 3.1 Generalize reference analysis for array-index and field-access receivers (C1)

**File:** `bootstrap/src/compiler.rs`

- Extend `collect_refs_in`, global `collect_reads`, and local `collect_reads` so that any `ExprFieldAccess` or array-index expression (`ExprCall` / `ExprArrayAccess` with index child) marks its base identifier as read/ref.
- Specifically handle:
  - `arr[i].field` → mark `arr`, `i`.
  - `x.field` → mark `x`.
  - tuple-return slicing `_let_tmp_0[63:32]` → mark `_let_tmp_0`.
- Ensure DCE does not delete locals/parameters whose only use is in such expressions.
- Add a unit test or at least a scratch spec (`w480_scope_visibility.t27`) that reproduces the original `eval` failure pattern and passes Icarus after the fix.

### 3.2 Ensure referenced locals are declared even after DCE/simplification (C1 follow-up)

**File:** `bootstrap/src/compiler.rs`

- In `gen_verilog_var` / local variable declaration, if a local is referenced but its initializer was simplified/eliminated, still emit the declaration (with a default zero value or the static value if known).
- This prevents Icarus "Unable to bind" errors when only a condition remains (e.g. `if ((total == 0.0))` but `total` was deleted).

### 3.3 Fix wildcard `_` assignment lowering (C3)

**File:** `bootstrap/src/compiler.rs`

- Detect assignments or bindings where the LHS identifier is `_`.
- Do not emit a Verilog variable named `_`. Either:
  - emit the RHS as a statement (side-effect only), or
  - emit a unique dummy reg (`__unused_N`) that is never read.
- This fixes `tokenizer` and `opcodes`.

### 3.4 Deduplicate benchmark block names and counter registers (C4)

**File:** `bootstrap/src/compiler.rs`

- Track already-emitted bench names and counter registers in `VerilogCodegen`.
- If the same bench would be emitted twice, skip the duplicate.
- This fixes `cordic_top`.

### 3.5 Sized placeholder for unsupported aggregate literals in concatenations (C5)

**File:** `bootstrap/src/compiler.rs`

- In the unsupported aggregate placeholder path, detect whether the expression is inside a concatenation (`{}`).
- Emit a sized literal using the known target width, e.g. `32'd0` instead of plain `0`.
- This fixes `cordic`.

### 3.6 Replace namespace-qualified calls with unsupported placeholder (C2)

**File:** `bootstrap/src/compiler.rs`

- Detect `ExprCall` names containing `::`.
- Emit the existing `/* UNSUPPORTED_ICARUS: host-side / module-qualified call (name) */ 0` placeholder instead of invalid `module::func(...)` syntax.
- This prevents syntax-error cascades in `benchmark` and `pipeline`; the specs will still fail (because the functions are host-side), but they will be classified as unsupported rather than malformed.

### 3.7 Update baseline JSON and classification

**File:** `docs/reports/gen_verilog_iverilog_smoke_baseline.json`

- Remove specs that now pass.
- Update classification for any spec whose error category changes.
- Add a note explaining which classes were closed in W480.

### 3.8 Witness spec for closed classes

**File:** `specs/scratch/w480_icarus_scope_and_wildcard.t27`

- Exercises field access on array element, array-index variable, wildcard discard, and sized placeholder contexts.
- Contains tests/invariants.
- Passes yosys and Icarus.

### 3.9 Update ring metadata

**Files:** `.trinity/current-issue.md`, `.trinity/ring-480.md` (create), `docs/reports/WAVE_LOOP_480_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W481_*.md`.

---

## 4. Verification plan

1. `cargo build --release` — must compile.
2. Manual spot check for each closed class:
   - Generate and Icarus-compile `eval`, `tokenizer`, `opcodes`, `cordic_top`, `cordic`.
3. `./scripts/tri test` full — target ≥120/127 Icarus PASS, ≤7 baselines, all non-smoke green, seals green.
4. `cargo test -p t27c --bin t27c` — must pass.
5. Reseal specs whose generated code changed.

---

## 5. Deliverables

- Code changes in `bootstrap/src/compiler.rs`.
- Updated `docs/reports/gen_verilog_iverilog_smoke_baseline.json`.
- New witness spec `specs/scratch/w480_icarus_scope_and_wildcard.t27`.
- Resealed artifacts.
- `docs/reports/WAVE_LOOP_480_CLOSEOUT.md`.
- `docs/reports/FPGA_LOOP_COOPERATION_W481_2026-07-10.md` with Variants A/B/C.
- Memory update.

---

## 6. Risk / trade-off discussion

| Approach | Effort | Regression risk | Honesty of gate |
|----------|--------|-----------------|-----------------|
| **A. Aggressively lower every `igla/` helper to RTL** | Very high | High | Low — would force host-side code into hardware |
| **B. Fix C1–C5 emitter/reference-analysis hygiene (recommended)** | Medium | Low | High — closes real bugs, keeps genuinely host-side specs in baseline |
| **C. Skip fixes and only formalize the subset in Lean** | Medium | Low | Medium — no immediate Icarus count improvement |

**Recommended:** Approach B. It directly continues W479, addresses the largest root cause (C1), and produces a measurable, honest improvement in the Icarus gate.

---

*φ² + φ⁻² = 3 | TRINITY*
