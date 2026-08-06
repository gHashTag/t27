# Wave Loop 426 — Weak Points and Competitor Scan

**Date:** 2026-07-05  
**Issue:** #1376  
**Branch:** `wave-loop-426`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points

### 1. Bench state is unchanged
- **P12 CCLK probe:** still unwired (no logic-analyzer channel on the board's
  CCLK pin). Without it, Variant A (real CCLK capture for OSCFSEL 6/7) cannot run.
- **Relay/remote-power gate:** still absent. Without it, true cold-POR boot
  experiments require manual power cycling, which is slow and not reproducible
  enough for a wave close-out.
- **DLC10 cable:** still missing. The working path is openFPGALoader + Digilent
  HS2 (`0x0403:0x6014`). Any XADC readout implementation must work over this
  FTDI-based path, not assume a Xilinx Platform Cable.
- **Board is reachable:** `openFPGALoader --detect -c digilent_hs2` finds the
  XC7A200T (`idcode 0x03636093`). This opens Variant B only if real XADC readout
  over HS2 is feasible.

### 2. Gen-verilog #1245 residual failures (7)
The `gen-verilog-yosys-smoke` phase still fails on 7 specs:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` documents that the full fix set exists
on `master` at commit `701d79b3b`, but the `trinity-rust-rings`/`wave-loop-*`
strategy is to apply only narrow, regression-free sub-fixes wave-by-wave. All 7
remaining failures are tied to major features (let destructuring, tuple returns,
ROM arrays, CORDIC). None is a safe narrow fix for a single wave.

### 3. PVT model is still placeholder
The linear derating coefficients in `n25q128_pvt_temp_derating_ns` and
`n25q128_pvt_voltage_derating_ns` are conservative upper envelopes, not real
Micron N25Q128_3V PVT data. The shape properties (monotonicity, antitonicity,
upper envelope) are correct, but the absolute numbers could be wrong. The model
is falsifiable and documented as such.

### 4. XADC readout is a placeholder
`tri fpga boot-log` / `cold-por` / `cclk-sweep` embed an `xadc` object with
`source: "not_read"`. Real readout requires JTAG XADC register access. The HS2
path may or may not expose this easily; openFPGALoader does not currently provide
a generic XADC read command.

### 5. Competitor gap is narrowing
Sparkle/Verilean added a full RV32 divider proof in June 2026 and is presenting
at Functional Festival 2026 on JIT, verification, and reverse synthesis. CIRCT
firtool 1.143.0 shipped with Verif/LTL/BTOR2 formal improvements. Clash 1.8.5
fixed verification-operator translations. t27 must keep the physical boot-
evidence line and ternary proof lattice as clear differentiators.

---

## Competitor scan (2026 mid-year update)

### Sparkle / Verilean
- **Repository:** https://github.com/Verilean/sparkle
- **Latest push:** 2026-07-03.
- **2026 milestones:**
  - RV32 divider correctness proof (`9c7809c`, June 2026) covering signed/
    unsigned division and divide-by-zero.
  - Accepted talk at Functional Festival 2026 (#fp_matsuri, 2026-07-11) on
    "Lean 4をRTL開発の中核にする — Sparkle におけるJIT、検証、Reverse Synthesis".
- **Threat:** same proof assistant, growing IP catalog, polished Signal DSL.
- **t27 differentiation:** ternary/balanced-trit proof lattice, spec-first sealed
  `gen/` pipeline, physical boot-evidence instrumentation (`tri fpga measured-to-lean`).

### Clash
- **Repository:** https://github.com/clash-lang/clash-compiler
- **2026 milestones:**
  - Clash 1.8.5 (Mar 2026) fixed `Clash.Explicit.Verification.check` blackbox
    clock-line handling.
  - Issue #3153 (Feb 2026): open verification-operator translation bugs for
    Yosys/SymbiYosys (`lit True` → `true`, `implies` → `->`).
  - Clash Formal project (QBayLogic) targets verified crypto, RISC-V/CHERI,
    FIDO2/CTAP2 passkey stacks.
- **Threat:** mature ecosystem, active formal program.
- **t27 differentiation:** Lean-native dependent types, ternary compute, physical
  measurement import.

### CIRCT / firtool / Chisel
- **2026 milestones:**
  - firtool 1.143.0 (Mar 2026): Verif dialect improvements, `verif.formal`
    BTOR2 support, named `verif.symbolic_value`, `ltl.past` lowering,
    `--assume-first-clock` / `--assume-init-reset` flags.
  - FIRRTL intrinsics for LTL (`firrtl.int.ltl.*`) and Verif
    (`firrtl.int.verif.*`) now documented.
- **Threat:** mainstream industry adoption, first-class LTL/SVA, contracts/BMC.
- **t27 differentiation:** source-language dependent-type proof (not RTL/SVA),
  ternary focus, sealed spec→bitstream pipeline.

---

## Strategic implication

Wave Loop 426 should execute **Variant C** (formal/tooling fallback) because the
hardware preconditions for A/B are still missing. The wave can still advance the
unique value proposition: strengthen the PVT formal model with a finite-grid
upper-envelope theorem, make the `tri fpga` JSON output more informative for
future physical experiments, and refresh the competitor snapshot to reflect the
June/July 2026 developments.

---

*φ² + φ⁻² = 3 | TRINITY*
