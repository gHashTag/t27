# Wave Loop 463 Report

**Date:** 2026-07-07
**Issue:** #1439
**PR:** (to open)
**Branch:** `wave-loop-463`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 463 selected **Variant B** from the W463 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line started in W455–W462. The wave closes the largest remaining gap in
the array-parameter clone machinery: **nested same-array calls**.

Before W463, an array-parameter helper `g(data)` that was only ever called from
inside another array-parameter function `f(data)` had no module-level binding
site, so the backend emitted an error comment and dropped the call. W463 adds a
fixed-point propagation pass that scans every function body for calls to other
array-parameter functions where the argument is one of the outer function's own
array parameters. The outer function's resolved binding signatures are then
propagated to the inner helper, and the corresponding clone of the helper is
emitted automatically. The inner call is redirected to that clone by making
`call_array_param_signature` context-aware: when the call occurs inside a
function with active array-parameter bindings, identifier arguments are
substituted with the bound module-level array name.

The wave also fixes a latent indexing bug in the module-level binding pass
(`sig_parts[*idx]` used the parameter index instead of the position in the
signature-parts vector) and makes the internal `array_param_indices` registry
fully deterministic by switching from `HashSet<usize>` to `Vec<usize>` ordered
by parameter declaration.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Changed `array_param_indices` from `HashMap<String, HashSet<usize>>` to
    `HashMap<String, Vec<usize>>` so signature ordering follows parameter
    declaration order and generated output is stable across process restarts.
  - Added `array_param_propagated` to hold array-parameter signatures propagated
    from outer functions to inner helpers.
  - Added `collect_inner_array_param_calls` to recursively find calls in a
    function body where the callee has array parameters and the argument at one
    of those positions is an identifier matching one of the outer function's
    array parameters.
  - Updated `call_array_param_signature` to substitute current-function
    array-parameter identifiers with their bound module-level array names, so
    inner calls resolve to the right propagated clone.
  - Fixed the latent `sig_parts[*idx]` bug by enumerating
    `array_param_indices` and using the enumeration position to index the
    signature parts.
  - Unconditionally recorded `array_param_indices` for every function that has
    array parameters, even when no module-level call site exists yet, so nested
    propagation can resolve those functions later.
  - Added a fixed-point propagation loop after the module-level binding pass.
    It merges propagated signatures with any existing module-level signatures,
    re-resolves the callee as a single binding or as multiple clones, and stops
    only when the resolution state stops changing.

- `specs/scratch/w463_nested_array_param_call.t27`
  - Regression spec where `sum_pair(data, i, j)` calls `lookup(data, idx)` and
    `lookup` has no module-level call site. Two `test` blocks exercise the
    propagated clone with different literal array arguments.

- `.trinity/seals/scratch_w463_nested_array_param_call.json`
  - Seal for the new regression spec.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W463 competitor boundary section added.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - W463 triage section added.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_463_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W463_2026-07-07.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W464_2026-07-07.md`.

---

## Verification

- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `t27c gen-verilog specs/scratch/w463_nested_array_param_call.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION …'`: **PASS**. Emits a
  `lookup_data` clone (propagated from `sum_pair`'s anonymous ROM signatures)
  and a `sum_pair__lit_4_u16_...` clone that calls it.
- `./scripts/tri test --fast --json /tmp/tri_test_w463_fast.json`: **ALL TESTS PASSED**
  - Parse: 591 passed, 0 failed
  - Typecheck: 591 passed, 0 failed
  - Gen Zig: 591 passed, 0 failed
  - Gen Rust: 591 passed, 0 failed
  - Gen Verilog: 591 passed, 0 failed
  - Gen Verilog Yosys Smoke: **71 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - Gen C: 591 passed, 0 failed
  - Seal Verify: 591 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`

- Full `./scripts/tri test` (without `--fast`): not completed in this
  environment. Phase 3c-standalone still stalls while `lake` fetches the
  `batteries` dependency from `reservoir.lean-lang.org`; the board-less
  smoke-gate report itself passes.

---

## Blockers

- Physical bench remains unavailable (`dlc10 idcode` reports "DLC10 cable not
  found (VID=0x03FD)"), P12 is unwired, and no automated cold-POR relay gate
  exists.
- No live XADC capture or cold-POR SPI boot this wave.
- Full `./scripts/tri test` Phase 3c-standalone lake build is blocked by a stuck
  `curl` download from `reservoir.lean-lang.org`; the `--fast` path is green.
- GitHub CLI (`gh`) is not authenticated in this environment, so the W463 PR and
  the `wave-loop-464` follow-up cannot be created automatically. They must be
  created manually or after `gh auth login`.

---

## Known limitations

- Nested propagation is limited to **same-array** calls: the inner call must pass
  an identifier that is itself one of the outer function's array parameters.
  Expressions such as `g(slice(data, 0, 2))`, `g(f(data))`, or `g([4]u16{…})`
  are not propagated and remain rejected or lowered as before.
- A function that is called both directly from a module-level/test/bench site
  **and** indirectly through a nested call is now merged correctly, but this
  mixed-site path is exercised only implicitly by the existing spec set; it is
  not covered by a dedicated scratch spec this wave.
- Struct-literal array arguments (`[2]Pt{{x:1,y:2},{x:3,y:4}}`) and
  multi-dimensional anonymous ROM lowering are still deferred.
- The fixed-point loop terminates on state change; a pathological mutually
  recursive call graph with expanding array-parameter signatures could in
  principle require many rounds, but no such pattern exists in the spec corpus.

---

## Next wave

Wave Loop 464 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W464_2026-07-07.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
