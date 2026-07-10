# Wave Loop 479 — Decomposed Plan

**Branch:** `wave-loop-479`  
**Variant:** B (default) — close or document the remaining Icarus Verilog failures caused by dynamic string/array constructs  
**Date:** 2026-07-08  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak spots discovered

### 1.1 Current Icarus gate state

Wave Loop 478 closed all structural packed-vector struct-array lowering failures. The remaining **20 Icarus smoke failures** are exclusively inside `specs/igla/`. They are not caused by Verilog emission hygiene; they are caused by trying to emit **host-side string/array algorithmic code** through the hardware-oriented `gen-verilog` backend.

| Class | Count | Root cause | Files affected |
|-------|-------|------------|----------------|
| F1 — dynamic string/array `.len()` | 7 `coder` + some `race` | `text.len()` / `tokens.len()` is flattened to a Verilog function call `text.len()` which Icarus rejects as an unsupported method. | `igla/coder/tokenizer`, `igla/coder/eval`, `igla/coder/dataset`, `igla/coder/bench_proxy`, `igla/coder/pipeline`, `igla/coder/weights`, `igla/race/*` (as termination condition) |
| F2 — dynamic string `.contains()` | 2 | `code.contains(" ")` and `template_name.contains(problem.expected_kw)` are flattened to unsupported method calls. | `igla/coder/tokenizer`, `igla/coder/bench_proxy` |
| F3 — recursive helpers on strings/arrays | 20+ helper functions | Tail-recursive functions such as `tokenize_verilog_inner`, `detokenize_inner`, `count_passed_inner`, `run_benchmark_inner`, etc. are emitted as recursive Verilog functions which Icarus cannot elaborate (and which are not synthesizable RTL). | All 7 `igla/coder` specs and most `igla/race` specs |
| F4 — `[]u32{}` / empty array literals in algorithmic code | scattered | Constructing dynamic arrays inside helpers produces unsupported array expressions. | `igla/coder/tokenizer`, `igla/coder/eval` |

### 1.2 Why this is a weak spot in the process, not only in the backend

- The Icarus gate was added as a **synthesizability/portability** gate, but the `igla/` specs are **host-language benchmark/agent specs**. They exercise the t27 front-end and non-Verilog backends, not RTL generation.
- Treating all 27 `igla_clean_specs` as Icarus targets creates a false expectation that every spec must be hardware-synthesizable. This masks the actual value of the Icarus gate: catching lowering bugs in `specs/scratch/` and non-igla specs.
- The 20 failures are stable and known. Continuing to report them as "failures" without classification wastes attention and makes the gate noisier than the yosys gate.

### 1.3 What can actually be closed in one wave

- **F1 (`.len()` on arrays)** is easy to close for synthesizable contexts: replace `x.len()` with a compile-time constant when `x` is a variable/parameter of known array type.
- **F2 (`.contains()` on strings)** can be closed for the two real call sites if the haystack length is bounded/known, by emitting a bounded `for` loop or a helper task.
- **F3 (recursive helpers)** is generally **not closable** for arbitrary string/array algorithms in one wave without a full host-side evaluator or bounded-unrolling pass. The honest action is to classify these specs as unsupported for Icarus.
- The cleanest engineering outcome: implement F1 (and F2 if feasible), document F3/F4 as an unsupported subset, and update the Icarus gate to accept a documented baseline for specs that are intentionally not hardware-synthesizable.

---

## 2. Scientific / technical literature mapped to the fixes

| Finding | Source | How it applies |
|---------|--------|----------------|
| Icarus Verilog supports only a slowly growing subset of SystemVerilog; dynamic arrays, strings, class/method calls, and queues are limited or unsupported. | [steveicarus/iverilog README](https://github.com/steveicarus/iverilog) | Confirms that `.len`, `.contains`, and recursive string helpers cannot be emitted as Verilog method/function calls and expect Icarus to accept them. |
| Synthesizable SystemVerilog is a well-defined subset. Defining a supported subset is standard practice for HDL tools. | Sutherland-HDL, *Synthesizable SystemVerilog: Busting the Myth that SystemVerilog is only for Verification* ([PDF](https://sutherland-hdl.com/papers/2013-SNUG-SV_Synthesizable-SystemVerilog_paper.pdf)) | Supports the decision to document the `igla/` algorithmic specs as outside the Icarus-supported t27 subset, rather than forcing arbitrary code to synthesize. |
| Verified HDL compilers (Lutsig) and equivalence checkers (Vera) transport source-level correctness down to netlists, but they operate on clearly defined source-language subsets. | Lööw, *Lutsig: a verified Verilog compiler for verified circuit development*, CPP 2021 ([DOI](https://doi.org/10.1145/3437992.3439916)); Pardalos et al., *Towards mechanized verification of Verilog equivalence checking*, LATTE 2025 ([PDF](https://johnwickerson.github.io/papers/vera_LATTE25.pdf)) | Reinforces that the correct long-term strategy is to define an Icarus-supported t27 subset and verify lowering for that subset, not to lower arbitrary host-side code to RTL. |
| Bounded loop lowering preserves semantics for finite data and can be justified by code-motion/value-propagation proofs. | Banerjee et al., *Verification of Code Motion Techniques Using Value Propagation*, IEEE TCAD 2014 ([DOI](https://doi.org/10.1109/tcad.2014.2314392)) | If `.contains` is lowered to a bounded `for` loop, a future formal fallback can prove equivalence to the source semantics for bounded strings. |
| Vivado documents exactly which SystemVerilog constructs it supports. Tool-specific supported-subset lists are industry standard. | AMD Vivado Synthesis User Guide, *SystemVerilog Constructs* ([docs](https://docs.amd.com/r/en-US/ug901-vivado-synthesis/SystemVerilog-Constructs)) | The Icarus gate should maintain its own supported-subset list and baseline, just as Vivado documents unsupported constructs. |

---

## 3. Decomposed implementation steps

### 3.1 Static `.len()` lowering for arrays (F1)

**File:** `bootstrap/src/compiler.rs`

- In `gen_verilog_expr`, `NodeKind::ExprCall` branch, detect method-style calls where `node.name` ends with `.len`.
- Extract the receiver identifier from the flattened name (e.g. `text.len` → `text`).
- Look up the receiver in the available symbol tables:
  - `local_array_dims` / `local_array_elem_info` — function-local arrays
  - `array_param_types` — function parameters of array type
  - `module_array_dims` / `module_array_types` — module-level arrays (discover if map exists)
  - `bench_local_names` + array info — bench-local arrays
- If the type is a fixed-size array `[N]T` (possibly multi-dimensional), emit the total element count as a decimal literal.
- If the receiver is a string literal, emit the byte/character length.
- If the receiver is a `string` parameter/variable, emit the declared capacity if statically known; otherwise fall back to the unsupported path.
- This fixes any future non-`igla` spec that legitimately uses `.len()` in a synthesizable context.

### 3.2 Static `.contains()` lowering for bounded strings (F2)

**File:** `bootstrap/src/compiler.rs`

- Detect method-style calls where `node.name` ends with `.contains`.
- If the receiver is a string variable/parameter with a known compile-time capacity and the needle is a string literal:
  - Emit a helper Verilog `function` or inline a bounded `for` loop (when used in a procedural context).
  - Loop over haystack indices `0 .. haystack_capacity - needle_len`; compare the substring.
- If the call appears in an expression context where a loop cannot be emitted inline, emit a small generated helper function with a unique name.
- If the receiver length is not statically known, fall back to unsupported.
- The two real call sites are in `igla/coder/tokenizer.t27` (inside an invariant — likely comptime) and `igla/coder/bench_proxy.t27` (inside a function). The invariant case may already be evaluated by the type checker; the function case needs a helper.

### 3.3 Unsupported-construct detection and clean diagnostics

**File:** `bootstrap/src/compiler.rs`

- Add a `gen_verilog_unsupported_expr` fallback that detects:
  - method calls on string/arrays that cannot be statically lowered,
  - recursive function calls (heuristic: function calls itself or calls a helper that calls it back),
  - dynamic empty array literals in non-constant contexts.
- Instead of emitting invalid Verilog, emit a clearly commented placeholder such as `/* UNSUPPORTED: dynamic string method x.len() */ 0` and continue, so the Icarus error becomes a single, classified failure rather than a cascade of syntax errors.
- This does not change yosys behavior because yosys already accepts or ignores many of these constructs.

### 3.4 Icarus baseline allow-list and classification

**Files:** `bootstrap/src/suite.rs`, `docs/reports/gen_verilog_iverilog_smoke_baseline.json`

- Create `docs/reports/gen_verilog_iverilog_smoke_baseline.json` with the 20 `igla/` specs and the unsupported category (`dynamic-string-array`, `recursive-helper`, etc.).
- Extend `cmd_gen_verilog_iverilog_smoke` to parse `iverilog` stderr and classify the first error into:
  - `unsupported_method` — `Object ... has no method "..."`
  - `recursive_function` — recursive function or unbounded loop
  - `syntax_error` — malformed statement
  - `other`
- Modify the Icarus phase to:
  - Accept failures that are in the baseline file (with matching category) as documented.
  - Reject any failure that is NOT in the baseline as a regression.
  - Print a summary: `Icarus smoke: X passed, Y baseline failures, Z regressions`.
- Keep the existing 106/126 success count honest; the 20 documented baselines are now reported separately.

### 3.5 Adversarial witness for the supported subset

**File:** `specs/scratch/w479_icarus_supported_subset.t27`

- Uses array `.len()` in a synthesizable context (e.g. a function that bounds a loop or computes a width).
- Uses a fixed-size string/array parameter with a `.contains()` call that the new static lowering can handle.
- Contains tests and a comptime invariant.
- Passes both yosys and Icarus after the backend changes.

### 3.6 Update ring metadata and reports

**Files:** `.trinity/current-issue.md`, `.trinity/ring-479.md`, `.trinity/experience.md`, `docs/NOW.md`

- Record the decision to document `igla/` algorithmic specs as outside the Icarus-supported subset.
- Update verification expectations.

---

## 4. Verification plan

1. `cargo build --release` — must compile.
2. Manual spot check: generate Verilog for `specs/scratch/w479_icarus_supported_subset.t27` and confirm it compiles under both yosys and Icarus.
3. `./scripts/tri test --fast` — must report 0 non-baseline failures and 0 seal mismatches.
4. `./scripts/tri test` full — Icarus smoke should report ≥106/126 clean targets with 20 documented baselines; no new regressions.
5. `cargo test -p t27c --bin t27c` — must pass.
6. Global reseal if generated Verilog changed for any spec.

---

## 5. Deliverables

- Code changes in `bootstrap/src/compiler.rs` and `bootstrap/src/suite.rs`.
- New baseline file `docs/reports/gen_verilog_iverilog_smoke_baseline.json`.
- New witness spec `specs/scratch/w479_icarus_supported_subset.t27`.
- Updated seals (global reseal if needed).
- `docs/reports/WAVE_LOOP_479_CLOSEOUT.md`.
- `docs/reports/FPGA_LOOP_COOPERATION_W480_2026-07-09.md` with three variants.
- Updated `.trinity/current-issue.md`, `.trinity/ring-479.md`, `.trinity/experience.md`, `docs/NOW.md`.
- Memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-479.md` plus `MEMORY.md` index entry.

---

## 6. Risk / trade-off discussion

| Approach | Effort | Regression risk | Honesty of gate |
|----------|--------|-----------------|-----------------|
| A. Lower all string/array methods + recursive helpers to RTL | Very high | High | High if it works, but likely incomplete |
| B. Static `.len`/`.contains` + documented baseline for algorithmic specs (recommended) | Medium | Low | High — gate focuses on specs that should synthesize |
| C. Formal subset predicate only (fallback) | Medium | Low | Medium — no Icarus count improvement |

**Recommended:** Approach B. It closes the real, synthesizable weak spot (F1/F2), makes the Icarus gate honest and quiet, and documents the intentional boundary between host-side and hardware-side specs.

---

*φ² + φ⁻² = 3 | TRINITY*
