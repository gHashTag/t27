# Wave Loop 486 — Ring State

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-486`
**Issue:** #1456
**Variant:** B (default)

## Outcome

W486 closed the next soft-failure classes after W485 eliminated all
`UNSUPPORTED_ICARUS` placeholders:

- Bench-local fixed-size arrays now cross function boundaries by resolving to a
  shared `__local__` packed-vector clone and slicing the packed input by element
  width inside the callee.
- Imported namespace-qualified helpers used only in host-side contexts are erased
  cleanly (statement-context comment, expression-context sized-zero placeholder)
  instead of producing `UNSUPPORTED_ICARUS` placeholders.
- Module-scope wildcard `_` bindings with array-literal initializers emit
  anonymous ROMs; struct-literal wildcards remain parser-blocked for a future
  wave.

## Verification snapshot

- 667 / 667 non-smoke PASS.
- 147 / 147 yosys smoke PASS, 0 failures.
- 147 / 147 Icarus smoke PASS, 0 documented baseline failures.
- 667 / 667 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Total `UNSUPPORTED_ICARUS` placeholders across all specs: 0.

## Key files changed

- `bootstrap/src/compiler.rs`
  - bench-local array name pre-collection and `__local__` signature binding
  - packed-vector scalar-array packing at call sites
  - packed-vector element-width slicing for scalar array parameters
  - bench initial-block emission bug fix (split counter/bench emitted sets)
  - `host_only_namespace_calls` set and
    `compute_host_only_namespace_calls`
  - `collect_qualified_calls_skipping_wildcards`
  - module-scope wildcard array-literal anonymous ROM emission
- `specs/scratch/w486_bench_array_param.t27`
- `specs/scratch/w486_helper_module.t27`
- `specs/scratch/w486_namespace_helper_erasure.t27`
- `specs/scratch/w486_wildcard_module_array.t27`
- `specs/scratch/w486_wildcard_module_array_copy.t27`
- `specs/scratch/w486_wildcard_module_literal.t27`
- `.trinity/seals/*.json` (global reseal)
- `docs/reports/WAVE_LOOP_486_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md`
- `docs/NOW.md`

## Next ring

- Branch to create: `wave-loop-487`
- See `docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md` for variants.
