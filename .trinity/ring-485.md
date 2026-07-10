# Wave Loop 485 — Ring State

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-485`
**Issue:** #1455 (to be opened)
**Variant:** B (default)

## Outcome

W485 closed the next soft-failure classes after all `UNSUPPORTED_ICARUS`
placeholders were eliminated in W484:

- Host-side recursive/proof-only helpers are now detected and skipped during
  Verilog generation.
- Module-scope and function-scope wildcard `_` bindings no longer emit duplicate
  identifiers or sized-zero assignments.
- A regression witness for bench-local array hoisting was added; the
  cross-function-boundary case remains a known open gap.

## Verification snapshot

- 661 / 661 non-smoke PASS.
- 141 / 141 yosys smoke PASS, 0 failures.
- 141 / 141 Icarus smoke PASS, 0 documented baseline failures.
- 661 / 661 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Total `UNSUPPORTED_ICARUS` placeholders across all specs: 0.

## Key files changed

- `bootstrap/src/compiler.rs`
  - `host_only_functions` set and `compute_host_only_functions`
  - `collect_all_expr_calls`, `fn_body_has_unlowerable_construct`,
    `fn_body_calls_host_only`
  - host-only skip in `gen_verilog_fn_internal`
  - host-only call handling in `gen_verilog_expr`
  - wildcard `_` handling in `gen_verilog_stmt`
  - module-scope wildcard skip in `gen_verilog_const`
- `specs/scratch/w485_host_helper_shadow.t27`
- `specs/scratch/w485_wildcard_binding.t27`
- `specs/scratch/w485_bench_local_array_hoist.t27`
- `.trinity/seals/*.json` (global reseal)
- `docs/reports/WAVE_LOOP_485_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W486_2026-07-07.md`
- `docs/NOW.md`

## Next ring

- Branch to create: `wave-loop-486`
- See `docs/reports/FPGA_LOOP_COOPERATION_W486_2026-07-07.md` for variants.
