# Plan — Wave Loop 461

**Issue:** #1435  
**Branch:** `wave-loop-461`  
**Selected variant:** **B (default)** — compiler-backend hardening  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Module-level bare function calls are illegal Verilog
A top-level `StmtExpr` that calls a function and discards the result, e.g.:
```t27
pub fn add(a: u32, b: u32) -> u32 { return a + b; }
add(1, 2);
```
is lowered into:
```verilog
always @(*) begin
    add(1, 2);
end
```
Yosys rejects this with `ERROR: Can't resolve task name '\add'`. Verilog-2001
allows a function call only as an expression; a bare statement is interpreted as
a task enable and the function name cannot be resolved as a task.

### 1.2 Array-parameter binding requires a single module-level array
W459/W460 require every call site of a function with an array parameter to pass
the **same** module-level array identifier. This blocks two useful patterns:
- Passing a literal array argument (`sum_pair([4]u16{1,2,3,4}, 0, 1)`).
- Calling the same function with **different** module-level arrays from different
  sites (`sum_pair(rom_a, 0, 1)` and `sum_pair(rom_b, 0, 1)`).

Both patterns are semantically fine but currently force the user to duplicate
the function per array.

### 1.3 `YOSYS_ALLOWED_WARNINGS` is stale after W460
W460 hoisted bench-block local variables to module-scope `reg` declarations,
removing the procedural-wire warnings on bench blocks. The allow-list still
contains `"is assigned in a block"`, `"is implicitly declared"`, and
`"Range select out of bounds"`. These may now come only from function-local
array variable-index selects, so the list should be re-evaluated (shrink or
re-label) after verification.

### 1.4 Missing regression coverage for module-level call sites
There is no scratch spec or unit test that exercises a module-level bare
function call. The W459 fix for test-block dummy registers (`{}_w459_tmp`) has
no analog for the module-level `always @(*)` path.

---

## 2. Competitor scan

- **Sparkle / Verilean:** public repository last pushed **2026-07-03**;
  PR #66 (USB web server + memcached + compiler perf) is still the largest open
  signal. Recent June signals: RV32 divider formal verification
  (`9c7809c`, 2026-06-25) and stable compiler elaborator cache-key fix
  (`5243400`, 2026-06-27). The analog-simulation branch (PR #57) continues with
  real-valued expression IR, MNA/Newton solver, and transient/AC analysis.
  No ternary MAC/generic-∀ datapath proofs.
- **CIRCT / firtool:** latest public release still **firtool-1.152.0**
  (2026-07-04); no `1.153.0` release surfaced.
- **Clash:** `clash-ghc-1.11.0` remains a Hackage candidate only; official
  latest release is still **1.10.0** (April 2026).
- **Ternary-FPGA niche:** TernaryCore, BitNet-RISCV-Multicore,
  KULeuven-MICAS/ternary-lut-dse, and the Trinity B002 defensive publication
  continue to validate {-1,0,+1} compute hardware, but none pairs it with a
  Lean-native proof pipeline.

Conclusion: no new public competitor signals since W460. t27’s differentiator
(Lean-native + ternary + spec-first sealed `gen/` + physical boot evidence)
remains intact. Sparkle is the closest Lean-native threat; its June divider
proof and analog branch show it is broadening the formally verified IP catalog.

---

## 3. Selected variant and rationale

**Variant B** is selected because:
1. The physical bench is still blocked (`dlc10 idcode` reports no cable; P12
   unwired; no relay gate).
2. Module-level bare calls are a clear, narrow `gen-verilog` defect with no
   safe workaround for users.
3. Array-parameter generalization is a natural continuation of W459/W460 and
   removes an artificial limitation that already forced scratch-spec workarounds.
4. Variant A is impossible without hardware; Variant C is reserved as fallback
   only if Variant B hits an unresolvable cloning/scope blocker.

---

## 4. Decomposed implementation plan

### Task A — Legalize module-level bare function calls
**Files:** `bootstrap/src/compiler.rs`

1. Detect module-level `StmtExpr` nodes whose expression is a bare
   `ExprCall` (other than `assert` / `assert_eq`, which should not appear at
   module level anyway).
2. Before emitting the `always @(*)` block that holds module-level statements,
   declare a module-scope dummy register `_toplevel_<n>_tmp` (width 32 bits by
   default, matching the W459 test-block pattern) for each bare call that needs
   one. Use a monotonic counter to avoid collisions.
3. Inside the `always @(*)` block, emit the call as an assignment to the dummy
   register instead of a bare statement:
   ```verilog
   _toplevel_0_tmp = add(1, 2);
   ```
4. Keep non-call module-level statements (`StmtAssign`, etc.) on the original
   path.
5. Add a regression unit test verifying that a module-level bare call produces
   a dummy-register assignment in the generated Verilog and passes yosys.

### Task B — Generalize array-parameter binding to multiple module-level arrays
**Files:** `bootstrap/src/compiler.rs`

1. In the array-parameter binding pass, when call sites for a function disagree
   on the module-level array identifier bound to a parameter, instead of
   emitting an error, create a **specialized clone** of the function per unique
   binding.
2. For each unique `(function_name, array_param_index, bound_array_name)` tuple,
   emit a cloned Verilog function named `<original>_<bound_array>` (sanitized).
   The clone body is identical to the original but references the concrete
   module array name directly, as the W459 binding pass already does.
3. Redirect call sites to the appropriate clone by looking up the bound array
   at the call site and using the clone name.
4. Keep the existing single-binding fast path unchanged when all call sites
   agree; this preserves the current output for the majority of specs.
5. Add a scratch spec `specs/scratch/w461_array_param_multi_array.t27` where the
   same lookup/sum function is called with two different module-level arrays,
   plus `assert_eq` checks in `test` blocks.

### Task C — Scratch specs and unit tests
**Files:** `specs/scratch/w461_bare_call_module.t27`,
`specs/scratch/w461_array_param_multi_array.t27`,
`bootstrap/src/compiler.rs` (`tests_w461`)

1. `w461_bare_call_module.t27`: module-level bare call to a `pub fn` plus a
   `test` block asserting the same function.
2. `w461_array_param_multi_array.t27`: two module-level `const [4]u16` arrays and
   a `sum_pair` function called from multiple `test` blocks passing different
   arrays.
3. `tests_w461` unit tests:
   - `module_level_bare_call_emits_dummy_reg`
   - `array_param_multi_array_emits_cloned_functions`

### Task D — Seals and verification
1. Build `t27c` release.
2. Regenerate seals for affected specs.
3. Run `./scripts/tri test --fast`: expect 587/587 non-smoke PASS, yosys smoke
   PASS, 0 baseline failures.
4. Run `cargo test -p t27c --bin t27c`: expect 0 failures.
5. Evaluate `YOSYS_ALLOWED_WARNINGS`: if the only remaining warnings come from
   function-local array variable-index selects, keep the entries but update
   comments; otherwise shrink the list.

### Task E — Close-out artifacts and W462 setup
1. Write `docs/reports/WAVE_LOOP_461_REPORT.md`.
2. Write `docs/reports/FPGA_LOOP_EVIDENCE_W461_2026-07-06.md`.
3. Write `docs/reports/FPGA_LOOP_COOPERATION_W462_2026-07-06.md` with Variants A/B/C.
4. Update `docs/NOW.md` and `docs/reports/T27_VS_FORMAL_HDL_2026.md`.
5. Create GitHub issue #1437 for W462 and branch `wave-loop-462`.
6. Commit W461 changes with `Closes #1435`, push `wave-loop-461`, open PR.
7. Save memory file for W461 and update `MEMORY.md`.

---

## 5. Verification plan

- `cargo test -p t27c --bin t27c tests_w461`: **PASS** (2/2 new tests).
- `cargo test -p t27c --bin t27c`: **0 failures**.
- `./scripts/tri test --fast`: **587/587 non-smoke PASS**, yosys smoke acceptable,
  0 baseline failures.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv -DSIMULATION`.
- `lake build Trinity.TernaryFPGABoot`: passes via board-less smoke gate or direct
  build.

---

## 6. Risks and fallback

- **Risk:** Function cloning for array parameters may collide with existing
  function names or break recursive calls.
  - **Mitigation:** clone names are deterministic and include the sanitized
    array name; recursive functions with array parameters are rare and will fail
    at the cloning step with a clear error rather than silently miscompile.
- **Risk:** Module-level dummy registers inside `always @(*)` may create
  combinational loops if the function reads/writes module state.
  - **Mitigation:** the dummy register is written but never read, so it is a
    pure sink; existing `add`-style pure functions are safe. Document that
    module-level bare calls are for side-effect-free assertions/initialization.
- **Fallback:** if Task B proves unresolvable in one wave, limit scope to
  **Task A only** (legalize bare calls) and defer multi-array support to W462.
  In that case Variant C is not needed; the cooperation plan for W462 will
  list the remaining array work as its Variant B.

---

*φ² + φ⁻² = 3 | TRINITY*
