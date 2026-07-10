# Wave Loop 479 Close-Out Report

**Branch:** `wave-loop-479`  
**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was asked

Close or document the remaining Icarus Verilog simulation failures caused by t27 dynamic aggregate constructs (`.len()`, `.contains()`, recursive string/array helpers) in the `igla/` algorithmic specs. The wave had to:

- keep the Icarus gate honest (no hidden regressions),
- add a scratch witness that exercises the fixed class,
- keep all non-smoke tests green, yosys smoke acceptable, seals green, and `cargo test` green.

The chosen approach was **Variant B from the cooperation document**: statically lower the synthesizable subset of array methods, then document the intentionally non-synthesizable `igla/` specs as a classified Icarus baseline.

---

## 2. Weak spots discovered

### 2.1 Icarus gate noise

Before W479 the Icarus smoke gate failed on any spec that emitted dynamic string/array method calls. Because `igla/` specs are host-language algorithmic specs rather than RTL, the gate produced a large, unclassified failure set that obscured real lowering regressions in `specs/scratch/`.

### 2.2 Method-call receivers were invisible to dead-store elimination

`t27` parses `arr.len()` as an `ExprCall` named `arr.len` and loses the receiver node. `dead_store_elim` therefore treated `arr` as unused and deleted its declaration, so `.len()` lowering later found no array to query.

### 2.3 Module-level scalar array literals were not initialized

`const arr : [4]u32 = [1,2,3,4];` stored its values in `ExprArrayLiteral.extra_size` with no child nodes. The Verilog emitter iterated over empty `children`, producing an empty `initial` block. This made `.contains()` simulation results undefined (`X`).

### 2.4 Hex literals broke Icarus memory initialization

`t27` emits `0x1234` unchanged in some initializer paths. Icarus rejects unsized hex literals when assigning into unpacked memory elements, even though decimal literals work.

---

## 3. Scientific / technical grounding

| Source | Relevance |
|--------|-----------|
| Sutherland-HDL, *Synthesizable SystemVerilog: Busting the Myth that SystemVerilog is only for Verification* (2013) | Justifies defining a clean Icarus-supported subset instead of forcing arbitrary host-side code into RTL. |
| Lööw, *Lutsig: a verified Verilog compiler for verified circuit development*, CPP 2021 | Verified compilers operate on well-defined source subsets; the correct strategy is to bound the subset and verify lowering, not lower everything. |
| Pardalos et al., *Towards mechanized verification of Verilog equivalence checking*, LATTE 2025 | Same message for equivalence checkers: semantics-preserving lowering is possible only when the source boundary is explicit. |
| AMD Vivado UG901, *SystemVerilog Constructs* | Industry practice is tool-specific supported-subset lists; the Icarus gate should maintain its own. |
| Icarus Verilog README / issue tracker | Confirms that dynamic arrays, strings, class/method calls, and recursive functions are outside the Icarus implementation subset. |

---

## 4. Implementation

### 4.1 Static `.len()` lowering

- File: `bootstrap/src/compiler.rs`
- Added helpers: `method_call_split`, `static_array_len`, `try_gen_verilog_static_len`.
- For any `ExprCall` whose name is `<receiver>.len`, the emitter looks up `receiver` in `local_array_dims`, `array_param_types`, and `module_scalar_array_dims`.
- If the receiver is a fixed-size scalar array, it emits the total element count as a decimal literal.
- Otherwise it falls back to `gen_verilog_unsupported_method` with a classified reason.

### 4.2 Static `.contains()` lowering for fixed-size scalar arrays

- Added helper `try_gen_verilog_static_contains`.
- For `receiver.contains(needle)` on a fixed-size scalar array, emits an OR-reduction:
  `(arr[0] == needle) || (arr[1] == needle) || ... || (arr[N-1] == needle)`.
- Supports both function-local and module-level scalar arrays.
- Falls back to unsupported for `u8` buffers and dynamic cases.

### 4.3 Dead-store / reference analysis now sees method-call receivers

- Updated `collect_refs_in`, the global `collect_reads` pass, and the local `collect_reads` pass to parse `name.receiver` from `ExprCall.name` and add the receiver identifier to read/ref sets.
- This stops DCE from deleting arrays whose only use is `.len()` or `.contains()`.

### 4.4 Module-level scalar array literal initialization

- Added `array_literal_elements` helper that reads values from either child `ExprLiteral` nodes or `ExprArrayLiteral.extra_size`.
- Updated module-level `const`/`var` scalar array initializers (both the modern `[N]T` path and the legacy `extra_size` path) to emit every element.
- Values are now passed through `gen_verilog_expr`, so `0x...` hex literals are normalized to decimal and accepted by Icarus.

### 4.5 Unsupported-construct placeholder

- Added `gen_verilog_unsupported_method` that emits `/* UNSUPPORTED_ICARUS: <reason> (<name>) */ 0`.
- This turns a cascade of Icarus syntax errors into a single classified failure per spec, making the baseline JSON stable.

### 4.6 Icarus baseline loader and classification

- File: `bootstrap/src/suite.rs`
- Added `load_gen_verilog_iverilog_smoke_baseline` reading `docs/reports/gen_verilog_iverilog_smoke_baseline.json`.
- Extended `SuiteSummary` with `iverilog_known_failures` and `iverilog_baseline_failures`.
- Icarus smoke phase now uses `run_phase_with_failures`, records failures, and reports them separately.
- Acceptability requires both yosys and Icarus known failures to be within their documented baselines and no other failures.

### 4.7 Baseline file

- File: `docs/reports/gen_verilog_iverilog_smoke_baseline.json`
- Documents 17 `igla/` specs as expected Icarus failures.
- Includes a `classification` map:
  - `syntax_error_unsupported_construct`
  - `dead_store_elimination_receiver_reference`
  - `host_side_function_not_lowered`
  - `wildcard_identifier_unsupported`
  - `indefinite_width_signed_literal_in_concat`
  - `duplicate_bench_name_declaration`

### 4.8 Witness spec

- File: `specs/scratch/w479_icarus_supported_subset.t27`
- Exercises module-level `.len()` and `.contains()` on a fixed-size `u32` array.
- Includes `test` and `invariant` blocks.
- Passes both yosys (`read_verilog -sv -DSIMULATION; synth -run check`) and Icarus (`iverilog -g2005-sv; vvp`).

---

## 5. Verification results

```
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog fails:        0
Gen Verilog smoke fails:  0
Gen Verilog Icarus fails: 17  (all documented baseline)
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:          0
FP divergences:           0

YOSYS BASELINE FAILURES:  0
ICARUS BASELINE FAILURES: 17
ACCEPTABLE:               yes (known failures match baseline, no other failures)
```

- **Yosys smoke:** all targets pass.
- **Icarus smoke:** 110 passed, 17 documented baseline failures.
- **Seals:** 647/647 match after resealing the 31 specs whose generated artifacts changed.
- **`cargo test -p t27c --bin t27c`:** 1525 passed, 0 failed, 2 ignored.
- **FPGA board-less smoke gate:** OK (bit_config, dry_run_sweep, verify_lean, yosys_synthesis all OK).

---

## 6. Files changed

- `bootstrap/src/compiler.rs` — static `.len()` / `.contains()` lowering, receiver reference fix, module array literal init, unsupported method placeholder.
- `bootstrap/src/suite.rs` — Icarus baseline loader, failure collection, summary fields, acceptability logic, tests.
- `docs/reports/gen_verilog_iverilog_smoke_baseline.json` — documented Icarus baseline with classification.
- `specs/scratch/w479_icarus_supported_subset.t27` — witness spec.
- `.trinity/seals/*.json` — resealed specs whose generated code changed.

---

## 7. Residual work (for W480)

- Reduce the Icarus baseline by fixing additional root causes (dead-store/scope visibility, duplicate bench names, wildcard `_` identifiers, indefinite-width signed literals).
- Add a formal subset predicate or Lean bridge for the Icarus-supported t27 fragment.
- Continue FPGA live-boot evidence if hardware becomes available.

---

*φ² + φ⁻² = 3 | TRINITY*
