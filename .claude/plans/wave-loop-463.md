# Wave Loop 463 Plan — Issue #1439

**Date:** 2026-07-07
**Branch:** `wave-loop-463`
**Selected variant:** B (compiler-backend hardening)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points investigated

### 1.1 `gen-verilog` backend gaps

After W462 the `gen-verilog` backend is green (590/590 non-smoke PASS, 70/70
yosys smoke PASS, 0 baseline failures). The dominant remaining user-visible gap
is **nested array-parameter calls**:

- W461/W462 allows a function `f(arr : [N]T)` to be called from module-level
  statements and from `test`/`invariant`/`bench` blocks, using either a
  module-level array identifier or a literal array argument.
- If `f` itself calls another array-parameter function `g(arr)` and `g` has no
  independent module-level call site, `g` is reported as
  "function ... has array parameter(s) but no call site" and skipped.
- This blocks natural composition: helper functions that operate on arrays
  cannot be layered without exposing every layer at module scope.

### 1.2 Safe defect backlog

`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` shows all tracked syntax/semantic
items fixed or verified through W462. The remaining open class is architectural
(RAM style inference, multi-dimensional array-literal initialization, etc.),
not a narrow single-wave sub-defect. Therefore W463 does **not** take a second
backend sub-defect; the nested-call propagation is large enough to fill the
wave safely.

### 1.3 External blockers

- Physical bench: `dlc10 idcode` still reports "DLC10 cable not found"; P12
  unwired; no relay gate. Variant A (live CCLK capture) is not feasible.
- `./scripts/tri test` Phase 3c-standalone still stalls on the external
  `reservoir.lean-lang.org` `batteries` download; the `--fast` path is the
  practical gate.

### 1.4 Competitor boundary

- **Sparkle / Verilean:** last public push 2026-07-03; PR #66 (IP.Net +
  compiler perf) and PR #65 (verified divider) remain open. No new public
  signals after W462 close-out.
- **CIRCT / firtool:** `firtool-1.152.0` (2026-07-04) is still the latest public
  release; no `1.153.0` yet.
- **Clash:** `1.11.0` remains a Hackage candidate; latest official release is
  `1.10.0` (April 2026).
- **Ternary-FPGA niche:** TernaryCore, ternfpga, and KULeuven ternary-lut-dse
  continue to validate `{-1,0,+1}` compute hardware, but none pairs it with a
  Lean-native proof pipeline.

Strategic implication: Sparkle is still the closest structural competitor, but
its public signals are stable. t27's differentiator remains the sealed
`*.t27 → gen/ → seal hash → physical boot-evidence` loop.

---

## 2. Goal

Close the nested array-parameter call gap for the **same-array propagation**
case: a function `f` with array parameter `arr` can call another function `g`
whose array parameter is exactly `f`'s `arr`. The binding signature of `f` is
propagated to `g`, the correct clone of `g` is emitted, and the inner call is
redirected to that clone.

---

## 3. Decomposed tasks

### Task A — Design the propagation analysis

**Owner:** C (Creator agent)  
**Files:** `bootstrap/src/compiler.rs`  
**Details:**
1. Extend the W458/W461 binding pass to also scan function bodies for
   `ExprCall` nodes whose callee is an array-parameter function.
2. Detect the same-array pattern: the argument at `g`'s array-parameter index
   is an `ExprIdentifier` whose name equals one of the current function `f`'s
   array-parameter names.
3. Build a propagation map `array_param_propagated_sigs: HashMap<String,
   HashMap<String, (HashMap<String,String>, HashSet<usize>)>>` keyed by callee
   `g`, then by signature key, holding the binding map and array-param indices
   for `g`.
4. For each signature propagated to `g`, ensure `array_param_indices[g]`,
   `array_param_clones[g]`, and `array_param_clone_bindings[clone_name]` are
   populated. If `g` already has module-level call sites, merge the
   propagated signatures without overwriting existing clones.

### Task B — Redirect inner calls to propagated clones

**Owner:** C  
**Files:** `bootstrap/src/compiler.rs`  
**Details:**
1. Update `call_array_param_signature` to be context-aware: when the current
   function `f` has array-parameter bindings, and a call argument is an
   identifier matching one of `f`'s array-parameter names, substitute it with
   the bound module-level array name before computing the signature.
2. Ensure this substitution works both for the original `f` and for clones
   of `f` (use `current_array_param_bindings()`).
3. Keep the existing module-level/test/invariant/bench call path unchanged.

### Task C — Emit propagated clones

**Owner:** C  
**Files:** `bootstrap/src/compiler.rs`  
**Details:**
1. In `gen_verilog_module`, when emitting functions, include clones created by
   propagation in addition to clones created by module-level call sites.
2. Preserve deterministic output: sort clone names before emission.

### Task D — Regression specs and seals

**Owner:** C + V  
**Files:** `specs/scratch/w463_nested_array_param_call.t27`,
`.trinity/seals/scratch_w463_nested_array_param_call.json`  
**Details:**
1. Spec A: single layer — `f(arr)` calls `g(arr, idx)` and `g` is not called
   at module scope.
2. Spec B: clone propagation — `f(arr)` is called with two different arrays
   from module level, and the inner call to `g` follows the right clone.
3. Spec C (stretch): two-level chain — `f` calls `g`, `g` calls `h`, all using
   the same array parameter.
4. Seal all new specs.

### Task E — Verification and close-out

**Owner:** V (Verifier agent)  
**Details:**
1. Run `./scripts/tri test --fast` and ensure 0 failures.
2. Run `cargo test -p t27c --bin t27c`.
3. Run `t27c check-now`.
4. Reseal any specs whose generated output changed.
5. Produce W463 close-out report and W464 cooperation variants.

---

## 4. Acceptance criteria

- `./scripts/tri test --fast`: 0 failures, `ACCEPTABLE: yes`.
- New scratch specs pass `t27c gen-verilog` + `yosys read_verilog -sv
  -DSIMULATION` and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c`: 0 failures.
- `t27c check-now`: PASS.
- All affected seals regenerated and stable.

---

## 5. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Propagation analysis creates clone name collisions with module-level clones | Use the same `sanitize_identifier` + `verilog_safe_identifier` naming scheme; merge by signature key. |
| Substitution in `call_array_param_signature` breaks existing module-level/test calls | Apply substitution only when inside a function with array-param bindings and the argument identifier matches a bound param; otherwise keep existing path. |
| Deterministic output / seal stability lost | Sort propagated clone names; sort inner iteration order. |
| Scope creep into struct-literal array args or different-array propagation | Explicitly defer to W464; document in limitations. |

---

## 6. Out of scope (deferred to W464)

- Struct-literal array arguments.
- Mixed direct/indirect call sites for the same array-parameter function.
- Different-array propagation (e.g. `f(arr1)` calls `g(arr2)` where `arr2` is
  another module-level array).
- Physical bench work (blocked by hardware).

---

*φ² + φ⁻² = 3 | TRINITY*
