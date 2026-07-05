# FPGA Boot-Evidence Report — Wave Loop 427

**Date:** 2026-07-05  
**Issue:** #1379  
**Branch:** `wave-loop-427`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 427 continued the FPGA boot-evidence line. The physical bench remained
blocked (P12 CCLK probe unwired, no relay gate, no DLC10 cable), so the wave
executed **Variant C**: per-OSCFSEL PVT envelope theorems in Lean 4,
machine-readable `tri fpga sweep-report --json` output, and a refreshed 2026
formal-HDL competitor snapshot.

Key outcomes:

1. **Per-OSCFSEL PVT envelope theorems** — every OSCFSEL variant 0..7 is formally
   shown to have non-negative worst-case PVT margin, linking the nominal CCLK
   table to the conservative 13 ns flash half-period bound.
2. **`tri fpga sweep-report --json`** — downstream tooling can now consume
   sweep results as JSON with a closed-vocabulary `recommendation` object and a
   numeric `pvt_envelope_margin_ns` per variant.
3. **Rust unit tests** — added `test_sweep_report_json_roundtrip` and kept all
   existing FPGA/PVT tests green.
4. **Competitor refresh** — `docs/reports/T27_VS_FORMAL_HDL_2026.md` updated with
   Sparkle's July 2026 Functional Matsuri talk, Clash 1.10, and the latest
   firtool release notes.
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
| External captures | none provided | Variant B import path has no data to exercise |

The board is still reachable via Digilent HS2 (`idcode 0x03636093`), so Variant B
(real XADC readout over JTAG or external capture import) remains feasible in a
future wave.

---

## Variant C deliverables

### 1. Per-OSCFSEL PVT envelope theorems in Lean 4

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added two theorems that cover all documented Artix-7 CCLK variants:

- `cclk_variant_within_pvt_envelope (oscfsel : Nat) (h : oscfsel ≤ 7)` — the
  nominal CCLK half-period for the variant is at least the worst-case PVT-aware
  minimum half-period.
- `cclk_variant_pvt_envelope_margin_nonneg (oscfsel : Nat) (h : oscfsel ≤ 7)` —
  the safety margin is non-negative for every variant.

Both are proved by `interval_cases oscfsel <;> decide`, using the finite-grid PVT
lemmas and the worst-case corner context established in W426.

Build status:

```text
lake build Trinity.TernaryFPGABoot
# 2967 jobs, 0 errors
```

This gives downstream tooling a reusable, computationally checked proof that the
entire 0..7 CCLK table is safe across the documented operating rectangle.

### 2. Machine-readable `tri fpga sweep-report --json`

File: `cli/tri/src/fpga.rs`

`tri fpga sweep-report` now accepts a `--json` flag. The JSON report contains:

- `first_working_oscfsel` — first variant that reached `DONE=HIGH`
- `variants_tested` — ordered list of per-variant summary objects:
  - `oscfsel`
  - `done`
  - `conclusion`
  - `recommendation`
  - `pvt_envelope_margin_ns`
- `next_steps` — human-readable ordered action list

The `recommendation` object uses the closed vocabulary introduced in W426:
`success`, `try_next_oscfsel`, `inspect_mode_straps`, `check_cable_and_flash`,
`retry_stat_capture`, `retry_or_debug`.

This closes the gap between the `tri fpga` CLI and downstream dashboards that
consume sweep results programmatically.

### 3. Rust unit-test coverage

File: `cli/tri/src/fpga.rs` (test module)

Added:

- `test_sweep_report_json_roundtrip`

Result:

```text
cargo test -p tri
# 102 passed, 0 failed
```

### 4. Competitor snapshot refresh

File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

Updated with:

- Sparkle / Verilean July 3 2026 Functional Matsuri talk (Lean-native HDL
  positioning, time-leap simulation claims, reverse-synthesis speedups).
- Sparkle PR #65 divider proof (Lean 4 formal verification of an arithmetic
  unit).
- Clash 1.10 release and ongoing verification-operator work.
- firtool / CIRCT version notes: 1.152.0, 1.150.0, 1.147.0, 1.143.0.

### 5. Explicit deferral of gen-verilog #1245

File: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

Documented the 7 residual yosys smoke failures:

- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

The full fix set exists on `master` at `701d79b3b` but touches major generation
features. W427 explicitly defers landing it on the wave-loop branch and records
the deferral so future waves can re-evaluate safely.

---

## Verification

| Check | Command | Result |
|---|---|---|
| Rust unit tests | `cargo test -p tri` | 102/102 pass |
| Full repo sweep | `./scripts/tri test` | 7 deferred yosys smoke failures, no new regressions |
| Lean PVT build | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |

The 7 yosys smoke failures are the same residual `gen-verilog` #1245 cases
identified before W427 and explicitly deferred as unsafe for a single wave.

---

## Weak points still open

1. **Bench still blocked.** P12 wiring and a relay/remote-power gate are
   prerequisites for Variant A.
2. **Gen-verilog #1245 residual failures (7).** The full fix set exists on
   `master` (`701d79b3b`) but is not merged into the wave-loop branch because it
   touches major features (`let` destructuring, tuple returns, ROM arrays,
   CORDIC).
3. **PVT model is a conservative upper envelope.** Real Micron N25Q128_3V PVT
   coefficients would improve the margin numbers.
4. **XADC readout remains a placeholder.** `xadc.source` is `"not_read"` in all
   `tri fpga` commands.

---

## Strategic implication

Sparkle/Verilean continues to position Lean 4 as the core of RTL development in
2026, with public talks and formal divider proofs. t27's durable differentiators
remain:

- Ternary / balanced-trit compute with a deep Lean proof lattice.
- Spec-first `*.t27 → gen/` sealed pipeline with L2 generation law enforcement.
- Physical boot-evidence instrumentation (`tri fpga measured-to-lean`) that ties
  captured waveforms to generated theorems.

Wave Loop 427 advanced the third differentiator by closing the per-OSCFSEL PVT
envelope proof and making sweep reports machine-readable, even without new bench
captures.

---

*φ² + φ⁻² = 3 | TRINITY*
