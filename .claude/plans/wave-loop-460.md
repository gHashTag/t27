# Plan — Wave Loop 460

**Issue:** #1433  
**Branch:** `wave-loop-460`  
**Selected variant:** **B (default)** — compiler-backend hardening  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Pre-existing cargo-test failures
Three unit tests in `bootstrap/src/compiler.rs` fail on `HEAD~1` and are unrelated to W459:

- `compiler::tests_compiler_rejects::let_binding_is_lowered_1401`
- `compiler::tests_phase40_coverage::test_let_binding_emitted_c_1401`
- `compiler::tests_phase40_coverage::test_let_binding_emitted_rust_1401`

Root cause: `let y = x; return y;` is copy-propagated into `return x;`, so the local declaration disappears from all generated backends. The parser already accepts `let` (mapped to `KwConst`) and creates a `StmtLocal`, but the optimizer does not preserve it.

### 1.2 Bench-block local-variable lowering
Yosys smoke warnings currently allowed in `bootstrap/src/suite.rs`:
- "is assigned in a block"
- "is implicitly declared"
- "Range select out of bounds"

These originate from bench blocks that declare/use local variables inside Verilog `initial begin … end`. Verilog-2005 does not allow variable declarations inside procedural blocks (R-VD-1 in the source), so the variables are emitted as assignments to implicit wires. The current fix hoists only the cycle counter; other bench-local vars should be hoisted to module scope inside the same `` `ifndef SIMULATION `` guard.

### 1.3 Array-parameter generalization
W459 already extended binding analysis to `test`/`invariant`/`bench` blocks and requires all call sites to agree on the same module-level array identifier. The remaining gap is mostly demonstrative: there is no scratch spec that exercises **multiple module-level call sites** passing the same array. Adding one closes the cooperation-plan acceptance criterion.

---

## 2. Competitor scan

- **Sparkle / Verilean:** last public push remains **2026-07-03**; PR #66 (IP.Net + compiler perf) is still the largest open signal; no new public commits or PRs surfaced after the W459 boundary. No ternary MAC/generic-∀ datapath proofs.
- **CIRCT / firtool:** latest public release still **firtool-1.152.0** (2026-07-04); no `1.153.0` release yet.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate; official latest is still `1.10.0` (April 2026).
- **Ternary-FPGA niche:** TernaryCore, BitNet-RISCV-Multicore, KULeuven ternary-lut-dse continue to validate {-1,0,+1} compute hardware, but none pairs it with a Lean-native proof pipeline.

Conclusion: no new public competitor signals since W459. t27’s differentiator (Lean-native + ternary + spec-first sealed `gen/` + physical boot evidence) remains intact.

---

## 3. Selected variant and rationale

**Variant B** is selected because:
1. The physical bench is still blocked (`dlc10 idcode` reports no cable; P12 unwired; no relay gate).
2. The three pre-existing cargo-test failures are the most visible quality gap in the compiler and block a fully green unit-test suite.
3. Bench-block local-variable lowering is a natural continuation of W458/W459 `gen-verilog` hygiene work.
4. Variant A is impossible without hardware; Variant C is reserved as fallback only if Variant B hits an unresolvable AST/scope blocker.

---

## 4. Decomposed implementation plan

### Task A — Preserve `let` bindings through optimization
**Files:** `bootstrap/src/compiler.rs`

1. Add `TokenKind::KwLet` to the lexer enum and map `"let"` to it in `check_keyword`.
2. In `parse_local_decl`, set `decl.extra_kind = self.current.lexeme.clone()` before consuming the keyword. This stores the original keyword ("let", "const", or "var") on every `StmtLocal`.
3. In `copy_propagate`, skip any `StmtLocal` where `extra_kind == "let"`.
4. In `const_propagate`, skip any `StmtLocal` where `extra_kind == "let"`, so `let x = 3; return x;` is not constant-folded away.
5. Verify the three failing unit tests now pass.

### Task B — Hoist bench-block local variables to module scope
**Files:** `bootstrap/src/compiler.rs`

1. In `gen_verilog_module`, when processing bench blocks, collect all `StmtLocal` nodes inside each bench body (recursively into `if`/`while`/`for` bodies if needed).
2. Emit module-scope declarations for each unique bench-local scalar/array variable inside the existing `` `ifndef SIMULATION `` guard that wraps bench counters, using sanitized names prefixed by the bench name to avoid collisions.
3. Inside the bench `initial` block, emit only assignments to the hoisted variables (no declarations).
4. Add a scratch spec `specs/scratch/w460_bench_local_var.t27` with a bench block that declares and uses a local variable, plus a `test` that exercises the same module-level function.

### Task C — Multi-site array-parameter scratch spec
**Files:** `specs/scratch/w460_array_param_multi_site.t27`

1. Create a spec where a function with an array parameter is called from two module-level `StmtExpr` sites (and optionally a test block), both passing the same module-level array.
2. Add `assert_eq` checks in a `test` block to exercise the function.

### Task D — Seals and verification
1. Build `t27c` release.
2. Regenerate seals for affected specs.
3. Run `./scripts/tri test --fast`: expect 583/583 non-smoke PASS, yosys smoke PASS, 0 baseline failures.
4. Run `cargo test -p t27c --bin t27c`: expect 0 failures.
5. If bench lowering removes the procedural-wire warnings, shrink `YOSYS_ALLOWED_WARNINGS` accordingly; otherwise keep the current allow-list.

### Task E — Close-out artifacts and W461 setup
1. Write `docs/reports/WAVE_LOOP_460_REPORT.md`.
2. Write `docs/reports/FPGA_LOOP_EVIDENCE_W460_2026-07-01.md`.
3. Write `docs/reports/FPGA_LOOP_COOPERATION_W461_2026-07-01.md` with Variants A/B/C.
4. Update `docs/NOW.md` and `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
5. Create GitHub issue #1435 for W461 and branch `wave-loop-461`.
6. Commit W460 changes with `Closes #1433`, push `wave-loop-460`, open PR.
7. Save memory file for W460 and update `MEMORY.md`.

---

## 5. Verification plan

- `cargo test -p t27c --bin t27c`: **0 failures**.
- `./scripts/tri test --fast`: **583/583 non-smoke PASS**, yosys smoke acceptable, 0 baseline failures.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv -DSIMULATION`.
- `lake build Trinity.TernaryFPGABoot`: passes (via board-less smoke gate or direct build).

---

## 6. Risks and fallback

- **Risk:** Modifying the lexer/parser to distinguish `let` from `const` could affect parsing of existing specs that use `let` destructuring or other `let`-adjacent syntax.
  - **Mitigation:** `let` is already parsed through the `KwConst` path; the change only adds a distinct token kind and preserves the existing parse behavior. All existing specs will be exercised by the full parse phase.
- **Risk:** Bench-local-variable hoisting may collide with module-level names or break nested scoping.
  - **Mitigation:** prefix hoisted names with the sanitized bench name; run the full yosys smoke gate to catch collisions.
- **Fallback:** if Task A or Task B proves unresolvable in one wave, switch to **Variant C**: add synthesizability theorems for W458/W459 regression specs, adversarial ±2 ns jitter envelope lemmas, and compiler-correctness bridge statements, leaving the cargo-test failures and bench lowering for a future wave.

---

*φ² + φ⁻² = 3 | TRINITY*
