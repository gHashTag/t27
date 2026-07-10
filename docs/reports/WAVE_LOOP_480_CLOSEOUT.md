# Wave Loop 480 — Close-out Report

**Date:** 2026-07-09  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Variant:** B — reduce the Icarus Verilog baseline by fixing small, classified root causes.

---

## 1. Goal

Close as many of the 17 documented Icarus smoke failures from W479 as possible, while keeping all non-smoke tests, yosys smoke, seals, and Rust unit tests green.

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| DCE condition-read fix: `StmtIf` / `StmtWhile` / `StmtFor` / `StmtForRange` now call `collect_reads` on their condition, and `StmtAssign` only collects reads from the RHS. | C1 scope/liveness | Prevents live condition variables from being deleted, which caused Icarus “Unable to bind” errors. |
| Bench-block deduplication by sanitized name. | C4 duplicate bench declarations | Stops duplicate module-scope counters and named `initial` blocks for specs that bench the same target more than once. |
| Sized unsupported aggregate / method / call placeholders. | C5 indefinite-width literals | Unsupported array literals and dynamic method/call placeholders now emit an explicit width (`WIDTH'd0`) so they do not break concatenations with `'sd0 has indefinite width`. |
| Track emitted functions and replace calls to non-emitted functions with a classified placeholder. | C2 namespace-qualified calls, C6 host-side helpers | `module::func(...)` and calls to host-side helpers no longer emit malformed Verilog; they become `/* UNSUPPORTED_ICARUS: ... */ WIDTH'd0`. |
| Braced block-expression parsing in `parse_expr_primary`. | C1 let-bound if-expressions | `let x = if (c) { a } else { b };` is now parsed, so local bindings such as `new_acc` / `mag_a` survive lowering. |
| Sized decimal literals inside tuple literals. | C5 indefinite-width concatenation | Plain `0` inside a tuple concatenation is emitted as `32'd0`. |
| Statement-context placeholder: bare unsupported calls in `StmtExpr` emit a comment-only no-op instead of a sized literal statement. | C2/C6 syntax errors in tasks | Prevents Yosys/Icarus syntax errors when a host-side call is used as a statement. |
| Precompute `emitted_functions` before const/var emission. | Forward references | Module-level array-of-struct initializers that call functions defined later in the module (e.g. `make_pts()`) now resolve correctly. |

### 2.2 `docs/reports/gen_verilog_iverilog_smoke_baseline.json`

Updated the documented Icarus baseline from 17 failures (W479) to 4 failures:

- `specs/igla/coder/eval.t27` — array-of-struct parameter destructure.
- `specs/igla/coder/pipeline.t27` — struct-return field access on unsupported call.
- `specs/igla/race/formal.t27` — imported struct parameter field access (single-file Verilog lowering cannot see `RtlModule` fields from `race/rtl.t27`).
- `specs/igla/race/rtl.t27` — array-of-struct parameter destructure.

### 2.3 Witness spec

- `specs/scratch/w480_icarus_scope_and_wildcard.t27` exercises braced if-expressions, array-index variables in conditions, field access on local structs, wildcard discard, and dropped helper calls.
- It passes yosys and Icarus smoke and has its own seal under `.trinity/seals/`.

## 3. Verification

```
./scripts/tri test --json /tmp/tri_w480.json
```

| Phase | Result |
|-------|--------|
| Parse | 650 / 650 PASS |
| Typecheck | 650 / 650 PASS |
| GF16 conformance | OK |
| Gen Zig | 650 / 650 PASS |
| Gen Rust | 650 / 650 PASS |
| Gen Verilog | 650 / 650 PASS |
| Gen Verilog Yosys Smoke | 130 / 130 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 126 / 130 PASS, 4 documented baseline failures |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | OK (~215 s) |
| FPGA smoke gate replay | OK |
| Gen C | 650 / 650 PASS |
| Seal verify | 650 / 650 PASS |
| Fixed point | 0 divergences |

```
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

The suite reports `acceptable: true` — all failures are documented baselines and there are no new regressions.

## 4. What was not closed (and why)

The remaining 4 Icarus failures are honest limitations of the current single-file Verilog backend:

- **eval / rtl** — recursive destructuring of array-of-struct parameters and struct-return field access. These require a more general AOS/return lowering pass.
- **pipeline** — struct-return field access on an unsupported dynamic helper call.
- **formal** — field access on an imported struct parameter (`RtlModule` is defined in `race/rtl.t27`, not visible to the single-file `gen-verilog` path).

Fixing these safely would require multi-file import lowering or a new AOS lowering phase, which is too large for the remainder of this wave.

## 5. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W481_2026-07-10.md` for three W481 variants:

- **Variant A:** complete the Icarus-supported-subset predicate in Lean 4.
- **Variant B (default):** attack the remaining AOS/struct-return Icarus baseline (eval / rtl / pipeline) with a focused lowering pass.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech Wukong XC7A100T and DLC10 cable are available.

---

*φ² + φ⁻² = 3 | TRINITY*
