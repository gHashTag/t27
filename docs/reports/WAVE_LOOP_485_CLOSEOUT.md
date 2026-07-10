# Wave Loop 485 — Close-out Report

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Variant:** B (default) — harden the Icarus/Verilog backend for the next
soft-failure classes after `UNSUPPORTED_ICARUS` placeholders were eliminated.

---

## 1. Goal

After W484 reduced `UNSUPPORTED_ICARUS` placeholders to **zero**, W485 targeted
the remaining soft-failure classes that prevented IGLA and bench specs from
simulating cleanly:

1. **Host-side recursive helper shadowing** — proof/invariant-only helpers were
   emitted to Verilog, then replaced with sized-zero placeholders because their
   bodies were not synthesizable.
2. **Module-scope wildcard `_` bindings** — `let _ = ...` at module scope could
   emit duplicate `_` identifiers or sized-zero assignments.
3. **Bench-local array hoisting** — bench-local arrays were not always resolved
   when used across function boundaries.

The acceptance gate was the standard wave gate: all non-smoke tests green,
yosys smoke green, Icarus smoke green (no new baseline failures), seals green,
`cargo test -p t27c --bin t27c` green, and zero `UNSUPPORTED_ICARUS`
placeholders.

---

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| Added `host_only_functions` set and `compute_host_only_functions`. | Recursive/dynamic helpers used only in invariants were emitted to Verilog and then replaced with placeholders, causing noisy generated code and simulation failures. | Functions that are dead to emitted Verilog contexts *and* contain unlowerable constructs (recursive `_inner` helpers, dynamic `.len()`/`.contains()`, namespace-qualified calls, unsupported builtins) are now skipped entirely. |
| Updated `compute_host_only_functions` to seed reachability from tests and benches. | The first implementation only seeded reachability from module-level statements, so helpers called from test/bench blocks were incorrectly classified as host-only and replaced with placeholders. | Functions called from emitted test/bench blocks (and their transitive callees) are always emitted; only truly dead helpers are skipped. |
| Added host-only skip in `gen_verilog_fn_internal`. | Skipped functions still attempted to emit a Verilog function body. | Host-only functions now emit only a short comment and no function body. |
| Added host-only call handling in `gen_verilog_expr`. | Calls to skipped helpers produced sized-zero assignments in statement context. | Statement-context calls to host-only functions emit a comment no-op; expression-context calls still get a sized-zero value so the surrounding expression remains syntactically valid. |
| Added wildcard `_` handling in `gen_verilog_stmt`. | `let _ = expr;` created a placeholder named `_`, colliding with later wildcard bindings. | Wildcard locals create an anonymous packed temporary for real expressions, or a comment for host-only/namespace calls, and never emit a named `_` reg. |
| Added module-scope wildcard skip in `gen_verilog_const`. | Module-level `let _ = ...` emitted `localparam _ = ...`, causing duplicate declaration errors. | Module-level `_` declarations are now emitted as comments only. |

### 2.2 Witness specs

- `specs/scratch/w485_host_helper_shadow.t27` — recursive `_inner` helper used
  only in an invariant; the public wrapper is also skipped because it calls the
  helper. A synthesizable `emitted_value()` function is exercised in test and
  bench.
- `specs/scratch/w485_wildcard_binding.t27` — covers module-level and
  function-level wildcard bindings for both host-only and emitted calls, plus a
  wildcard inside a function body.
- `specs/scratch/w485_bench_local_array_hoist.t27` — bench-local fixed-size
  `[4]u32` array hoisted to module scope and used directly inside the bench. The
  cross-function-boundary case remains a known open gap (see §5).

All three witness specs pass yosys synthesis and Icarus simulation.

### 2.3 Global reseal

The compiler changes altered generated Verilog for specs that contain host-only
helpers or wildcard bindings. All `.trinity/seals/*.json` were refreshed after
the final green suite run.

---

## 3. Verification

```bash
./scripts/tri test
```

| Phase | Result |
|-------|--------|
| Parse | 661 / 661 PASS |
| Typecheck | 661 / 661 PASS |
| GF16 conformance | OK |
| Gen Zig | 661 / 661 PASS |
| Gen Rust | 661 / 661 PASS |
| Gen Verilog | 661 / 661 PASS |
| Gen Verilog Yosys Smoke | 141 / 141 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 141 / 141 PASS, **0 documented baseline failures** |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | OK |
| FPGA smoke gate replay | OK |
| Gen C | 661 / 661 PASS |
| Seal verify | 661 / 661 PASS |
| Fixed point | 0 divergences |

```bash
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

Additional metric:

- **Total `UNSUPPORTED_ICARUS` placeholders across all 661 generated Verilog specs: 0.**

The suite reports `acceptable: true` — zero documented Icarus baseline failures
and no other failures.

---

## 4. Literature review

- **Synthesizable Verilog subset semantics (Cambridge VFE V0).** The
  prohibition on unbounded recursion, dynamic strings, and unresolved
  identifiers justifies erasing host-side proof helpers before code generation
  rather than emitting them and later replacing them with placeholders.
- **Proof-assistant extraction / helper erasure (Peregrine, MetaRocq).** Helpers
  used only in specifications or proofs are routinely erased before target
  extraction. W485 applies the same principle: identify helpers whose bodies are
  not synthesizable and skip them in the Verilog target.
- **FIRRTL/Chisel property types.** FIRRTL treats strings as non-synthesizable
  property types and lowers fixed-size vectors to packed Verilog. This supports
  the t27 design choice of keeping string/array helper functions host-only while
  still emitting fixed-size array logic.

---

## 5. What was not closed (and why)

- **Bench-local arrays crossing function boundaries.** The witness spec was
  simplified to direct bench-local array use because the array-parameter binding
  resolution for hoisted bench-local names (`bench_<n>_<name>_<i>`) is a
  separate, deeper lowering problem that intersects with function cloning and
  packed-vector parameter passing. Fixing it cleanly requires more than a
  one-wave change and is left for a future wave.
- **Module-scope wildcard `_` bindings with non-call initializers.** The current
  implementation handles the call/literal cases that appear in the test corpus;
  complex struct/array literal initializers at module scope would reuse the same
  anonymous-temporary logic but are not exercised by existing specs.
- **Dynamic `.len()` / `.contains()` on unknown strings or variable-length
  containers.** W484 already closed the known-string/fixed-size-array cases;
  runtime variable-size containers remain out of scope for the synthesizable
  backend.

These classes are tracked as future Icarus/Verilog backend extensions.

---

## 6. Issue gate

Branch `wave-loop-485` closes **#1455**.

---

*φ² + φ⁻² = 3 | TRINITY*
