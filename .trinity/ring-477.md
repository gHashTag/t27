# Ring 477 — Wave Loop 477

**Date:** 2026-07-07  
**Branch:** `wave-loop-477`  
**Variant:** B — compiler-backend hygiene + Icarus simulation gate  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Goal

Close the gen-verilog hygiene gap identified in Wave Loop 476:
- Hoist all local declarations to the top of generated Verilog function bodies so the output is strict Verilog-2001 compliant.
- Add an Icarus Verilog compilation + VVP simulation gate to the conformance suite.
- Keep all existing yosys smoke targets green.
- Add an adversarial scratch spec that interleaves declarations and statements.

## Outcome

W477 added two hoisting passes (block scope and function/task scope), masked
comments/strings during tokenization, dropped unsupported procedural attributes,
and hardened test assertion emission. A new Icarus smoke phase runs after the
yosys smoke phase. The conformance suite is green at 645/645 non-smoke and
125/125 yosys smoke targets; Icarus smoke is 92/125 clean with 33 baseline
failures inherited from W475/W476 packed-vector struct-array lowering.

## Artifacts

- `docs/reports/WAVE_LOOP_477_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W478_2026-07-08.md`
- `.claude/plans/wave-loop-477.md`
- `specs/scratch/w477_hoisting_and_iverilog.t27`

## Verification

- `cargo build --release`: PASS
- `cargo test -p t27c --bin t27c`: 1524 passed, 0 failed, 2 ignored
- `./scripts/tri test --fast`: ALL TESTS PASSED
- `./scripts/tri test`: ALL TESTS PASSED (645/645 non-smoke, 125/125 yosys smoke, 92/125 Icarus smoke baseline)

## Next

- Branch: `wave-loop-478`
- Default Variant B: close the remaining Icarus failures in packed-vector struct-array lowering.
