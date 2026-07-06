# FPGA Loop Evidence — Wave Loop 455 (2026-07-01)

**Issue:** #1425  
**Branch:** `wave-loop-455`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was done

Wave Loop 455 executed **Variant B** from `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md`.
The physical bench (DLC10 cable, P12 wiring, relay gate) remains unavailable, so the
wave did not pursue live CCLK capture. Instead it attacked the 7 residual
`gen-verilog` yosys smoke failures that had been accepted as a documented baseline
since W422/W427.

Rather than merge `master` commit `701d79b3b` (which W454 found insufficient and
regression-risky), W455 incrementally ported the missing parser and Verilog backend
features from the historical `wave-loop-383` compiler line into the current
FPGA-focused branch:

- Tuple return type parsing (`-> (T1, T2, ...)`).
- Tuple literals (`(a, b, c)`).
- `let (a, b, c) = expr` destructuring assignment.
- Tuple-return function generation with packed result register.
- Slot-aware nested tuple-return call lowering.
- Module-level `const [N]T{...}` ROM lowering.
- Function-local `var [N]T` array lowering (numeric/variable indices, signed
  elements, `for` loops, 2D arrays, array-literal initializers).
- Keyword-safe full-token identifier escaping for flattened local-array element
  names (e.g. `\buf_0 ` instead of `\buf _0`).

### Files changed

- `bootstrap/src/compiler.rs` — parser and `gen-verilog` backend changes.
- `cli/flash-spi/src/main.rs` — restored workspace build by supplying new
  `FlashOpts` fields (`no_jprogram: false`, `bitswap: true`).
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — updated defect matrix and W455
  triage decision.
- `docs/reports/gen_verilog_smoke_baseline.json` — expected-failure set now empty.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W455 boundary refresh.
- `.trinity/seals/*.json` — 67 seal files resealed to the new compiler output.

---

## Verification results

| Check | Result |
|---|---|
| `cargo build --release` | **PASS** |
| `t27c gen-verilog` + `yosys read_verilog -sv` on the 7 previously failing specs | **PASS** (0 failures) |
| `./scripts/tri test --json /tmp/tri_test_w455.json` | **ALL TESTS PASSED** |
| Parse | 576 passed, 0 failed |
| Typecheck | 576 passed, 0 failed |
| Gen Zig | 576 passed, 0 failed |
| Gen Rust | 576 passed, 0 failed |
| Gen Verilog | 576 passed, 0 failed |
| Gen Verilog Yosys Smoke | **56 passed, 0 failed** |
| FPGA Board-Less Smoke Gate | **OK** (`bit_config=ok`, `dry_run_sweep=ok`, `verify_lean=ok`, `yosys_synthesis=ok`) |
| FPGA Standalone Lake-Package Build | **OK** (`elapsed_ms ~410815`) |
| Gen C | 576 passed, 0 failed |
| Seal Verify | 576 passed, 0 failed |
| Fixed Point | 0 divergences |
| **TOTAL FAILURES** | **0** |
| `ACCEPTABLE` | **yes** |

The 7 previously documented baseline gen-verilog yosys smoke failures are now
**cleared**:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

---

## Not done

- **Physical bench execution:** still blocked. `dlc10 idcode` reports
  "DLC10 cable not found (VID=0x03FD)", P12 is unwired, and no automated cold-POR
  relay gate exists.
- **Live-capture CCLK sweeps:** cannot be performed until the bench unblocks.
- **RAM style pragmas:** block-vs-distributed RAM hints remain future work.

---

*φ² + φ⁻² = 3 | TRINITY*
