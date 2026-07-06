# FPGA Loop Cooperation Plan — Wave Loop 463 (2026-07-07)

**Issue:** #1439 (to create from W462 land commit)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W462

Wave Loop 462 closed #1437 by selecting Variant B: compiler-backend hardening.
The wave extended the W461 array-parameter clone machinery so that literal array
arguments are lowered to anonymous module-level ROMs, made void-return
module-level bare calls emit as task enables without dummy registers, and added
a bench-local + array-parameter integration regression spec. The `--fast` suite
path is green: 590/590 non-smoke PASS, 70/70 yosys smoke PASS, FPGA smoke gate OK,
0 baseline failures, 0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W463 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
theorem fixture set under the post-W462 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W462 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w463/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W463_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 590/590 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: nested array-parameter calls + struct-literal array arguments + another safe gen-verilog defect (default)

Execute when the bench is still blocked. This is the most likely W463 path.

### Goal
Extend the array-parameter work so functions can internally call other
array-parameter functions, allow struct literals to be passed to array-parameter
slots, and clear another safe `gen-verilog` sub-defect.

### Scope
1. **Nested array-parameter calls.** Extend the binding analysis so that when
   a function `f(arr)` internally calls another array-parameter function
   `g(arr)`, the binding signature of `f` is propagated to `g`. Emit the
   corresponding clone of `g` and redirect the inner call to it. Start with the
   same-array case; different arrays or mixed direct/indirect call sites may be
   deferred if they require a larger call-graph refactor.
2. **Struct-literal array arguments.** Allow array parameters whose element type
   is a struct to be passed with a literal array of struct literals, lowering to
   an anonymous packed ROM or memory initialized field-by-field.
3. **Clear one more safe `gen-verilog` sub-defect.** Pick the next small item
   from `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` that does not require
   master-merge integration (e.g., remaining keyword-escape edge cases, signed
   local-array element select, or improved tuple-return destructuring in
   expression contexts).
4. Reseal all affected specs and keep `YOSYS_ALLOWED_WARNINGS` aligned with the
   cleaner output.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New or updated scratch specs pass `t27c gen-verilog` + yosys
  `read_verilog -sv -DSIMULATION` and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c` passes with 0 failures.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by an AST/scope refactor that cannot be
completed safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice with synthesizability and
compiler-correctness bridge statements that treat the now-cleaner `gen-verilog`
backend as a trusted source of synthesizable artifacts.

### Scope
1. **Synthesizability theorem block.** Add propositions in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the W460–W462
   regression specs (`w460_bench_local_var`, `w460_array_param_multi_site`,
   `w461_bare_call_module`, `w461_array_param_multi_array`,
   `w462_array_param_literal`, `w462_void_bare_call`,
   `w462_array_param_bench_local`) produce yosys-clean Verilog, expressed over
   seal hashes and the yosys smoke report.
2. **Adversarial clock-jitter envelope.** Quantify worst-case raw-ns predicate
   preservation under ±2 ns bounded jitter across all OSCFSEL selections and
   all four PVT corners.
3. **Compiler-correctness bridge lemma.** Relate the W460 `let` preservation,
   W460 bench-local lowering, W461 array-parameter cloning, and W462 literal
   array lowering to the abstract ternary MAC semantics in
   `TernaryInference.lean`.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 590/590 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W463 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455–W462.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
