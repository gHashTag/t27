# Wave Loop 385 Implementation Plan

**Issue:** #1280  
**Base branch:** `trinity-rust-rings`  
**Branch to create:** `wave-loop-385`  
**Selected variant:** Variant B from `docs/reports/WAVE_LOOP_384_COOPERATION.md`

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **119 waves**, push the Lean 4 `ternaryMac` generic ∀ lattice to **284**, and generalize function-local array lowering to **signed element types** and **array-literal initialization**.

## Theorems (4 new generic ∀)

Add to `proofs/lean4/Trinity/TernaryInference.lean`:

1. `ternaryMacAccumulateSixtyThreePlusGeneric` — 63-variable plus accumulation.
2. `ternaryMacAccumulateSixtyTwoMinusGeneric` — 62-variable minus accumulation lattice.
3. `ternaryMacQuadragintupleQuinqueCancellationGeneric` — `mac^45(x, a, [.plus,.minus,...]) = x` (depth-45 identity cancellation).
4. `ternaryMacZeroWeightTwentyPairClosureGeneric` — 20 zero-weight MACs before and after a plus-weight MAC.

Proof pattern: `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega`.

## Compiler backend changes

Target: `bootstrap/src/compiler.rs`, `gen_verilog_stmt`, `NodeKind::StmtLocal` array branch.

Current behavior for `var buf : [4]u16 = [4]u16{...}`:
- Emits per-element regs `buf_0`..`buf_3`.
- Writes a comment placeholder and calls `gen_verilog_expr` on the initializer, which for `ExprArrayLiteral` emits `0 /* TODO ... */`.

New behavior:
- If the initializer `children[0]` is an `ExprArrayLiteral`, emit a scalar assignment for each element:
  ```verilog
  buf_0 = 16'h1111;
  buf_1 = 16'h2222;
  ...
  ```
- Apply width padding for `0x` and `0b` literals in each element assignment (reuse existing scalar literal padding logic or call `gen_verilog_expr` on the element and let the existing return/var padding handle it — elements here are simple literals in the regression specs; arbitrary expressions can use `gen_verilog_expr`).
- If no initializer, keep current behavior.
- Signed element types already pass `elem_signed` to the reg declaration; no new logic needed beyond verifying with a scratch spec.

## New scratch regression specs

1. `specs/scratch/w385_signed_local_array.t27`
   - `var temps : [4]i16;`
   - Write/read signed values including negative values.
   - Tests for sign preservation.

2. `specs/scratch/w385_local_array_init.t27`
   - `var buf : [4]u16 = [4]u16{0xA1B2, 0xC3D4, 0xE5F6, 0x6789};`
   - Read back initialized values via variable and literal indices.

3. `specs/scratch/w385_signed_local_array_init.t27` (optional, combines both)
   - `var temps : [3]i16 = [3]i16{-100, 0, 100};`
   - Read back values and verify sign.

These specs must contain `test` blocks so L4 TESTABILITY is satisfied and they enter the yosys smoke gate.

## IGLA spec forward-appending

- Copy/adapt `scripts/gen_w384.py` → `scripts/gen_w385.py`.
- Copy/adapt `scripts/gen_w384_lean.py` → `scripts/gen_w385_lean.py`.
- Run `gen_w385.py` to append W385 `test`/`invariant` blocks to all 27 IGLA specs.
- Run `gen_w385_lean.py` to append the 4 theorems.

## Validation steps

1. `lake build Trinity.TernaryInference`
2. `./target/release/t27c suite --repo-root .` → expect 568/568 PASS, 0 seal mismatches, 47 yosys smoke targets (27 IGLA + 20 scratch).
3. Manual `yosys read_verilog -sv` + `synth -top <spec>` on the new scratch specs.

## Seal regeneration

- Regenerate all 27 IGLA seals from `.`.
- Regenerate new scratch seal(s).
- Regenerate any non-IGLA seals whose generated code changed.

## Documentation

- `docs/reports/WAVE_LOOP_385_REPORT.md`
- `docs/reports/WAVE_LOOP_385_COOPERATION.md` with W386 variants
- `docs/reports/FPGA_EVIDENCE_W385.md`
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to mark signed/init arrays fixed.
- Update `.trinity/experience.md`.
- Save memory `wave-loop-385.md` and update `MEMORY.md` index.
- Update `.trinity/current-issue.md` to W385 issue #1280.

## Commit and PR

- Commit to `wave-loop-385` with message closing #1280.
- Push to origin.
- Open PR against `trinity-rust-rings`.

## Risk and mitigation

| Risk | Mitigation |
|---|---|
| Array literal initializer parsing produces unexpected AST | Inspect AST with a minimal spec before writing codegen. |
| Width padding for initialized elements is inconsistent | Reuse `gen_verilog_expr` for each element; if literal, existing padding in return/var paths is not active here, so add explicit `0x`/`0b` padding in the new loop mirroring `StmtLocal` scalar-init logic. |
| Signed default value in mux chain is wrong for negative reads | Default is `0`; signed regs handle sign extension on reads. Acceptable. |
| Yosys smoke gate regression | Run full suite before commit. |

## Acceptance criteria

- `lake build Trinity.TernaryInference` passes.
- `t27c suite --repo-root .` returns 0 failures and 0 seal mismatches.
- New scratch specs pass `yosys read_verilog -sv`.
- 27 IGLA specs remain yosys-clean.
- Commit closes #1280.
