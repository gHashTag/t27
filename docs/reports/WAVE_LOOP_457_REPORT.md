# Wave Loop 457 Report

**Date:** 2026-07-01
**Issue:** #1428
**Branch:** `wave-loop-457`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 457 selected **Variant B** from the W457 cooperation plan: with the
physical bench still blocked, add synthesizer-controllable **RAM style pragma**
support for module-level arrays. The implementation introduces a minimal
`pragma ram_style = "...";` statement that attaches the standard Verilog
`(* ram_style = "..." *)` attribute to the next module-level array declaration.
This gives Vivado and Yosys an explicit hint to infer block RAM vs. distributed
RAM while staying within t27's spec-first, seal-verified workflow.

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `KwPragma` token and `pragma` keyword lexer mapping.
  - Added `extra_pragma: String` to `Node`; initialized in `Default` and `new`.
  - Added `pending_pragma` to `Parser` and `ParserCheckpoint` with save/restore.
  - Added `parse_pragma` for `pragma name = "value";` top-level statements;
    currently accepts `ram_style = "block"` and `ram_style = "distributed"`
    and rejects unknown pragma names so typos fail fast.
  - `parse_module_body` now consumes `pragma` directives before the next
    module-level declaration.
  - `parse_const_decl` and `parse_var_decl` capture the pending pragma into the
    declaration node and clear it so it is not accidentally reused.
  - `gen_verilog_var` emits `(* {pragma} *)` before the synthesizable
    `reg ... [0:N]` memory declaration for true array types (e.g. `[4]u16`).
  - New `tests_w457_ram_style` unit-test module with three tests:
    - `ram_style_block_pragma_emitted`
    - `ram_style_distributed_pragma_emitted`
    - `unknown_pragma_rejected`

- `specs/scratch/w457_ram_style_block.t27`
  - Regression spec with `pragma ram_style = "block";` on a module-level writable
    `[4]u16` array; write/read and loop-sum tests.

- `specs/scratch/w457_ram_style_distributed.t27`
  - Regression spec with `pragma ram_style = "distributed";` on a module-level
    writable `[4]u16` array; write/read tests.

- `.trinity/seals/scratch_w457_ram_style_block.json`
- `.trinity/seals/scratch_w457_ram_style_distributed.json`
  - Seals for the new regression specs.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W457 competitor boundary section.

- `docs/reports/FPGA_LOOP_EVIDENCE_W457_2026-07-01.md`
  - Evidence file.

- `docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`
  - Next-wave handoff with three variants.

## Verification

- `cargo test -p t27c --bin t27c tests_w457_ram_style`: **PASS** (3/3).
- `t27c gen-verilog specs/scratch/w457_ram_style_block.t27` +
  `yosys read_verilog -sv; synth -top w457_ram_style_block`: **PASS**,
  emits `(* ram_style = "block" *)`.
- `t27c gen-verilog specs/scratch/w457_ram_style_distributed.t27` +
  `yosys read_verilog -sv; synth -top w457_ram_style_distributed`: **PASS**,
  emits `(* ram_style = "distributed" *)`.
- `./scripts/tri test --json /tmp/tri_test_w457.json`: **ALL TESTS PASSED**
  - Parse: 579 passed, 0 failed
  - Typecheck: 579 passed, 0 failed
  - Gen Zig: 579 passed, 0 failed
  - Gen Rust: 579 passed, 0 failed
  - Gen Verilog: 579 passed, 0 failed
  - Gen Verilog Yosys Smoke: **59 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - FPGA Standalone Lake-Package Build: **OK**
  - Gen C: 579 passed, 0 failed
  - Seal Verify: 579 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`

## Blockers

- Physical bench remains unavailable (DLC10 cable not detected, P12 unwired).
- No live XADC capture or cold-POR SPI boot this wave.

## Next wave

Wave Loop 458 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
