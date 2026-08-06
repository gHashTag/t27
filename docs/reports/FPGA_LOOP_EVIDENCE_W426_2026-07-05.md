# FPGA Boot-Evidence Report — Wave Loop 426

**Date:** 2026-07-05  
**Issue:** #1376  
**Branch:** `wave-loop-426`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 426 continued the FPGA boot-evidence line. The physical bench remained
blocked (P12 CCLK probe unwired, no relay gate, no DLC10 cable), so the wave
executed **Variant C**: finite-grid PVT theorems in Lean 4, machine-readable
`tri fpga` JSON output, and a refreshed 2026 formal-HDL competitor snapshot.

Key outcomes:

1. **Lean 4 finite-grid PVT theorems** — `pvt_half_ns_operating_rectangle_grid_bounded`
   and `pvt_low_ns_operating_rectangle_grid_bounded` prove that the worst-case
   corner dominates every grid point inside the documented operating rectangle.
2. **`tri fpga` JSON hardening** — `cclk-sweep`, `boot-log`, and `cold-por` now
   emit a machine-readable `recommendation` object and a `pvt_envelope_margin_ns`
   field (when the CCLK variant is known).
3. **Rust unit tests** — new coverage for `cclk_nominal_hz`,
   `pvt_envelope_margin_ns`, and `recommendation_from_conclusion`.
4. **Competitor refresh** — `docs/reports/T27_VS_FORMAL_HDL_2026.md` updated with
   Sparkle's July 2026 Functional Matsuri talk, Clash 1.8.5 verification fixes,
   and the latest CIRCT/firtool notes.
5. **No regressions** — `./scripts/tri test` reports the same 7 deferred
   `gen-verilog-yosys-smoke` failures that existed before the wave.

---

## What was blocked

| Blocker | Status | Impact |
|---------|--------|--------|
| P12 CCLK probe unwired | unchanged | Variant A (real OSCFSEL 6/7 capture) impossible |
| Relay / remote-power gate | absent | True cold-POR automation impossible |
| DLC10 cable | missing | Xilinx `dlc10` path unavailable; HS2 + openFPGALoader remains the only path |
| XADC readout | not implemented | `xadc.source` stays `"not_read"` |

The board is still reachable via Digilent HS2 (`idcode 0x03636093`), so Variant B
(real XADC readout over JTAG) remains feasible if implemented in a future wave.

---

## Variant C deliverables

### 1. Finite-grid PVT theorems in Lean 4

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added two theorems that enumerate the finite temperature/voltage grid inside the
documented operating rectangle and prove that the worst-case corner dominates
every point:

- `pvt_half_ns_operating_rectangle_grid_bounded`
- `pvt_low_ns_operating_rectangle_grid_bounded`

Each theorem checks the 5 × 5 × 3 = 75 grid points exhaustively using `rcases`
and the existing `pvt_half_ns_worst_case_bound` / `pvt_low_ns_worst_case_bound`
lemmas.

Build status:

```text
lake build Trinity.TernaryFPGABoot
# 2967 jobs, 0 errors
```

This gives the project a reusable grid-boundedness lemma that future captured
PVT points can invoke by exact matching.

### 2. Machine-readable `tri fpga` output

File: `cli/tri/src/fpga.rs`

All `cclk-sweep` log entries now include:

- `pvt_envelope_margin_ns` — nominal CCLK half-period minus the worst-case
  PVT-aware minimum half-period (nanoseconds). Positive means the nominal CCLK
  is safe across the documented operating envelope.
- `recommendation` — a closed-vocabulary JSON object:
  - `action`: `success` | `try_next_oscfsel` | `inspect_mode_straps` |
    `check_cable_and_flash` | `retry_stat_capture` | `retry_or_debug`
  - `oscfsel`: the variant that produced the conclusion
  - `first_working_oscfsel`: the first variant that reached `DONE=HIGH` (if any)
  - `next_steps`: human-readable ordered list

`boot-log` and `cold-por` logs also include `recommendation` and
`pvt_envelope_margin_ns: null` because they do not yet know the bitstream's
OSCFSEL selection.

The margin is computed from a Rust mirror of the Lean `cclk_nominal_hz` table
(2.5 / 4.2 / 6.6 / 10 / 12.5 / 16.7 / 25 / 33.3 MHz for OSCFSEL 0–7) and the
existing PVT worst-case bound of 13 ns.

Example margin values:

| OSCFSEL | Nominal CCLK | Half-period | Worst-case bound | Margin |
|---------|-------------|-------------|------------------|--------|
| 0 | 2.5 MHz | 200 ns | 13 ns | +187 ns |
| 3 | 10 MHz | 50 ns | 13 ns | +37 ns |
| 6 | 25 MHz | 20 ns | 13 ns | +7 ns |
| 7 | 33.3 MHz | 15 ns | 13 ns | +2 ns |

### 3. Rust unit-test coverage

File: `cli/tri/src/fpga.rs` (test module)

Added:

- `test_cclk_nominal_hz_matches_lean`
- `test_pvt_envelope_margin_ns_zero_freq`
- `test_pvt_envelope_margin_ns_2_5mhz`
- `test_pvt_envelope_margin_ns_33mhz`
- `test_recommendation_success`
- `test_recommendation_try_next_without_first_working`
- `test_recommendation_try_next_with_first_working`
- `test_recommendation_mode_mismatch`

Result:

```text
cargo test -p tri
# 101 passed, 0 failed
```

### 4. Competitor snapshot refresh

File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

Updated with:

- Sparkle / Verilean July 11 2026 Functional Matsuri talk (JIT outperforming
  Verilator on LiteX, ~49 GHz equivalent “time-leap” simulation, 2.14× reverse-
  synthesis speedup on a carry-save multiplier).
- Clash 1.8.5 verification-operator fixes (`check` blackbox clock line and string-
  literal typing, PRs #2907 / #2908).
- Existing CIRCT firtool 1.143.0 / 1.152.0 LTL/Verif/BTOR2 notes retained.

---

## Verification

| Check | Command | Result |
|---|---|---|
| Rust unit tests | `cargo test -p tri` | 101/101 pass |
| Full repo sweep | `./scripts/tri test` | 7 deferred yosys smoke failures, no new regressions |
| Lean PVT build | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |

The 7 yosys smoke failures are the same residual `gen-verilog` #1245 cases
identified before W426 and explicitly deferred as unsafe for a single wave.

---

## Weak points still open

1. **Bench still blocked.** P12 wiring and a relay/remote-power gate are
   prerequisites for Variant A.
2. **Gen-verilog #1245 residual failures (7).** The full fix set exists on
   `master` (`701d79b3b`) but is not merged into the wave-loop branch because it
   touches major features (let destructuring, tuple returns, ROM arrays, CORDIC).
3. **PVT model is a conservative upper envelope.** Real Micron N25Q128_3V PVT
   coefficients would improve the margin numbers.
4. **XADC readout remains a placeholder.** `xadc.source` is `"not_read"` in all
   `tri fpga` commands.

---

## Strategic implication

Sparkle/Verilean is now publicly positioning Lean 4 as the core of RTL
development. t27's durable differentiators remain:

- Ternary / balanced-trit compute with a deep Lean proof lattice.
- Spec-first `*.t27 → gen/` sealed pipeline with L2 generation law enforcement.
- Physical boot-evidence instrumentation (`tri fpga measured-to-lean`) that ties
  captured waveforms to generated theorems.

Wave Loop 426 advanced the third differentiator by closing the PVT grid theorem
and making `tri fpga` output machine-readable, even without new bench captures.

---

*φ² + φ⁻² = 3 | TRINITY*
