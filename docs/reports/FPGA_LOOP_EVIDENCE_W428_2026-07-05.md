# FPGA Boot-Evidence Report — Wave Loop 428

**Date:** 2026-07-05  
**Issue:** #1383  
**Branch:** `wave-loop-428`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 428 continued the FPGA boot-evidence line. The physical bench remained
blocked (P12 CCLK probe unwired, no relay gate, no DLC10 cable, only OSCFSEL
0–5 bitstreams available), so the wave executed **Variant C**: unified OSCFSEL
0..7 PVT theorems in Lean 4, machine-readable `tri fpga pvt-envelope --json`
output, and a refreshed 2026 formal-HDL competitor snapshot.

Key outcomes:

1. **Unified PVT theorems** — four quantified theorems cover all documented
   Artix-7 CCLK variants, giving downstream tooling a single theorem to
   reference instead of eight concrete instances.
2. **`tri fpga pvt-envelope --json`** — the PVT envelope command now emits a
   machine-readable report with context, bound, margin, operating envelope, and
   examples.
3. **Rust unit tests** — added three unit tests for the new JSON report builder.
4. **Competitor refresh** — updated `docs/reports/T27_VS_FORMAL_HDL_2026.md` with
   new 2026 releases and emerging signals.
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
| OSCFSEL 6/7 bitstreams | absent | Only OSCFSEL 0–5 bitstreams exist in `build/fpga/cclk_variants` |
| External captures | none provided | Variant B import path has no data to exercise |

The board is still reachable via Digilent HS2 (`idcode 0x03636093`), so Variant B
(real XADC readout over JTAG or external capture import) remains feasible in a
future wave.

---

## Variant C deliverables

### 1. Unified OSCFSEL 0..7 PVT theorems in Lean 4

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added four theorems that cover all documented Artix-7 CCLK variants:

- `all_oscfsel_cclk_within_pvt_envelope (oscfsel : Nat) (h : oscfsel ≤ 7)` —
  the nominal CCLK half-period for the variant is at least the worst-case
  PVT-aware minimum half-period.
- `cclk_variant_worstcase_pvt_measured_satisfies_flash_spec (oscfsel : Nat)
  (h : oscfsel ≤ 7)` — every variant satisfies the worst-case PVT-aware
  measured-CCLK flash predicate at 50% duty.
- `cclk_variant_implies_transaction_ok (oscfsel : Nat) (h : oscfsel ≤ 7)
  (bits : Nat)` — every variant produces a flash-spec-compliant SPI read
  transaction at its nominal rate.
- `cclk_variant_worstcase_pvt_implies_transaction_ok (oscfsel : Nat)
  (h : oscfsel ≤ 7) (bits : Nat)` — the same, under the worst-case PVT corner.

All four are proved by `interval_cases oscfsel <;> decide` or by applying the
existing PVT implication theorems with the worst-case corner.

Build status:

```text
lake build Trinity.TernaryFPGABoot
# 2967 jobs, 0 errors
```

This gives downstream tooling a reusable, computationally checked proof family
that links any documented OSCFSEL selection directly to a flash-spec-compliant
SPI transaction.

### 2. Machine-readable `tri fpga pvt-envelope --json`

File: `cli/tri/src/fpga.rs`

`tri fpga pvt-envelope` now accepts a `--json` flag. The JSON report contains:

- `pvt_context` — the supplied context (or `null` if none).
- `nominal_min_sck_half_ns` — the nominal 6 ns bound.
- `min_sck_half_ns` — the PVT-derated minimum low/high bound.
- `margin_ns` — derated bound minus the nominal bound.
- `operating_envelope` — documented temp/vccint ranges.
- `examples` — best/typical/worst example contexts and bounds (only when no
  context is supplied).
- `warnings` — placeholder for out-of-envelope warnings.

Example output for the worst-case context:

```json
{
  "margin_ns": 7,
  "min_sck_half_ns": 13,
  "nominal_min_sck_half_ns": 6,
  "operating_envelope": {
    "temp_c_max": 85,
    "temp_c_min": -40,
    "vccint_mv_max": 1100,
    "vccint_mv_min": 900
  },
  "pvt_context": {
    "process_corner": "ss",
    "temp_c": 85,
    "vccaux_mv": 2700,
    "vccint_mv": 900
  },
  "warnings": []
}
```

### 3. Rust unit-test coverage

File: `cli/tri/src/fpga.rs` (test module)

Added:

- `test_pvt_envelope_json_report_with_context`
- `test_pvt_envelope_json_report_no_context`
- `test_pvt_envelope_json_report_has_operating_envelope`

Result:

```text
cargo test -p tri
# 105 passed, 0 failed
```

### 4. Competitor snapshot refresh

File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

Updated with:

- Sparkle / Verilean: no public commits after July 5 2026; next public milestone
  is the July 11 2026 Functional Matsuri talk; added sister project Hesper.
- Clash: 1.11.0 is a Hackage candidate as of July 4 2026, not yet official;
  1.10.0 remains the latest release.
- Chisel 7.13.0 (June 2026) with FIRRTL 7.0.0 bump and ChiselTest compatibility
  layer (including `chiseltest/formal`).
- firtool 1.152.0 (July 4 2026) is the latest available; 1.153 does not yet exist.
- Bluespec Compiler 2026.01 (May 2026) and SpinalHDL v1.14.0 (February 2026).
- New “Emerging signals” subsection: CktFormalizer, Aria-HDL, TernaryCore,
  BitNet-RISCV-Multicore, MINRES RISC-V Tournament.

### 5. Explicit deferral of gen-verilog #1245

File: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

Documented that the W428 start-of-wave probe confirmed the same 7 residual
yosys smoke failures. The wave-loop strategy of narrow, regression-free
sub-fixes is not applicable because the failures remain tied to major features
(tuple-return generation, `let` destructuring, ROM arrays, CORDIC). Resolution
continues to depend on a future master merge/rebase wave.

---

## Verification

| Check | Command | Result |
|---|---|---|
| Rust unit tests | `cargo test -p tri` | 105/105 pass |
| Full repo sweep | `./scripts/tri test` | 7 deferred yosys smoke failures, no new regressions |
| Lean PVT build | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |
| pvt-envelope JSON | `tri fpga pvt-envelope --pvt-context ctx.json --json` | produces expected JSON |

The 7 yosys smoke failures are the same residual `gen-verilog` #1245 cases
identified before W428 and explicitly deferred as unsafe for a single wave.

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
2026, with a public talk and a growing IP catalog. New signals like
CktFormalizer and Aria-HDL show that other groups are also using Lean 4 as a
hardware proof backend. t27's durable differentiators remain:

- Ternary / balanced-trit compute with a deep Lean proof lattice.
- Spec-first `*.t27 → gen/` sealed pipeline with L2 generation law enforcement.
- Physical boot-evidence instrumentation (`tri fpga measured-to-lean`) that ties
  captured waveforms to generated theorems.

Wave Loop 428 advanced the formal/tooling line by closing unified OSCFSEL
覆盖 and making the PVT envelope command machine-readable, even without new
bench captures.

---

*φ² + φ⁻² = 3 | TRINITY*
