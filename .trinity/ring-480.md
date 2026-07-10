# Ring 480 — Wave Loop 480

**Date:** 2026-07-09  
**Branch:** `wave-loop-480`  
**Variant:** B — reduce the Icarus Verilog baseline by fixing small, classified root causes  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Close as many of the 17 documented Icarus smoke failures from W479 as possible, while keeping all non-smoke tests, yosys smoke, seals, and Rust unit tests green.

## Outcome

W480 closed 13 of the 17 documented Icarus failure classes. The Icarus smoke gate is now honest at **126 / 130 PASS**, with **4 documented baseline failures** remaining:

- `specs/igla/coder/eval.t27` — array-of-struct parameter destructure.
- `specs/igla/coder/pipeline.t27` — struct-return field access on unsupported call.
- `specs/igla/race/formal.t27` — imported struct parameter field access (single-file Verilog lowering cannot see `RtlModule` from `race/rtl.t27`).
- `specs/igla/race/rtl.t27` — array-of-struct parameter destructure.

Key backend changes in `bootstrap/src/compiler.rs`:
- DCE condition-read fix: `StmtIf` / `StmtWhile` / `StmtFor` / `StmtForRange` now collect reads from their conditions; `StmtAssign` only collects reads from the RHS.
- Bench-block deduplication by sanitized name prevents duplicate module-scope counters and named `initial` blocks.
- Sized unsupported placeholders for array literals, dynamic method calls, namespace-qualified calls, and non-emitted host-side functions.
- Emitted-function tracking with forward-reference precompute before const/var emission, so module-level AOS initializers can call functions defined later in the module.
- Braced block-expression parsing (`if (c) { a } else { b }`) so let-bound if-expressions survive lowering.
- Sized decimal literals inside tuple literals to avoid indefinite-width concatenation errors.
- Statement-context placeholder: bare unsupported calls in `StmtExpr` emit a comment-only no-op instead of an invalid sized literal statement.

Spec / baseline changes:
- `docs/reports/gen_verilog_iverilog_smoke_baseline.json` updated from 17 to 4 documented failures with classifications.
- `specs/scratch/w480_icarus_scope_and_wildcard.t27` witness spec covers braced if-expressions, array-index variables, field access, wildcard discard, and dropped helper calls; passes yosys and Icarus.

## Artifacts

- `docs/reports/WAVE_LOOP_480_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W481_2026-07-10.md`
- `.claude/plans/wave-loop-480.md`
- `specs/scratch/w480_icarus_scope_and_wildcard.t27`

## Verification

- `cargo build --release`: PASS
- `cargo test -p t27c --bin t27c`: 1525 passed, 0 failed, 2 ignored
- `./scripts/tri test --json /tmp/tri_w480.json`: ACCEPTABLE
  - 650 / 650 non-smoke PASS
  - 130 / 130 yosys smoke PASS
  - 126 / 130 Icarus smoke PASS, 4 documented baseline failures
  - 650 / 650 seal matches
  - 0 fixed-point divergences
  - FPGA board-less smoke gate: OK
  - FPGA standalone lake-package build: OK
  - FPGA smoke gate replay: OK

## Next

- Branch: `wave-loop-481`
- Default Variant B: attack the remaining 4 Icarus baseline specs with focused AOS / struct-return / imported-struct lowering.
