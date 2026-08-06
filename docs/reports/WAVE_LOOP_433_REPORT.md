# Wave Loop 433 Report — FPGA boot-evidence XADC-to-OSCFSEL raw-ns PVT bridge

**Date:** 2026-07-01  
**Issue:** #1393  
**Branch:** `wave-loop-433`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 433 executed **Variant C3** of the FPGA boot-evidence plan: the bench
remains blocked (no P12 CCLK probe, no relay cold-POR gate, no DLC10 cable) and
the `gen-verilog` fix set is still not safely reachable from the wave-loop branch,
so the wave focused on a board-less formal composition.

The key deliverable is a theorem that **composes the W431 live-XADC envelope bound
with the W432 per-process-corner raw-ns OSCFSEL theorem**. For every documented
Artix-7 CCLK selection (OSCFSEL 0..7) and any live XADC operating point inside
the documented envelope with a process corner at least as slow as `ss`, the
nominal raw-ns CCLK capture satisfies the PVT-aware flash predicate under the
measured context, and it produces a flash-spec-compliant SPI read transaction.

The wave also refreshed the competitor snapshot and re-confirmed the 7 residual
yosys smoke failures as a known baseline.

---

## Deliverables

### 1. Live-XADC → OSCFSEL raw-ns PVT bridge theorem

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`
  - Parameterized over `oscfsel : Nat` (with `oscfsel ≤ 7`) and any in-envelope
    `XadcOperatingPoint` whose process corner is at least as slow as `ss`.
  - Proves that the ideal raw-ns capture at the nominal OSCFSEL period satisfies
    the PVT-aware raw-ns flash predicate under the measured PVT context.
  - Composes `xadc_envelope_implies_raw_ns_satisfies_any_in_envelope` (W431) with
    `cclk_variant_raw_ns_per_process_corner_pvt_satisfies_flash_spec` (W432).

- `xadc_envelope_justifies_cclk_variant_transaction_ok`
  - Lifts the predicate theorem to an end-to-end transaction-safety theorem for
    any transaction size.

- `xadc_live_example_oscfsel_6_raw_ns_pvt`
  - Concrete example for a representative live readout and OSCFSEL=6.

### 2. Competitor refresh

`docs/reports/T27_VS_FORMAL_HDL_2026.md`

- Sparkle PR #66 remains open (last push 2026-07-03): USB web server, memcached
  server, full networking stack, compiler performance fixes.
- `firtool-1.152.0` published 2026-07-04; maintenance release.
- Clash 1.11.0 remains a Hackage candidate; no official release yet.
- Aria-HDL continues 2026 updates around retiming and PCIe BAR testing.

### 3. Defect baseline

`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

- W433 triage entry: no compiler work; the 7 residual yosys smoke failures
  remain the documented baseline.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test --bin tri fpga::` | **81 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **49 passed, 7 pre-existing failures** (#1245) |

The 7 pre-existing gen-verilog yosys smoke failures are unchanged:
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

These are covered by the full fix set on `master` (`701d79b3b`), which is on a
divergent lineage relative to `wave-loop-433`.

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel.
- **Relay / remote-power cold-POR gate:** still not wired.
- **DLC10 cable:** still not connected; the Digilent HS2 + `openFPGALoader` path
  remains the working one.
- **Master-merge to clear #1245:** still not safely reachable.

---

## Next wave

Wave Loop 434 should execute the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`.

*φ² + φ⁻² = 3 | TRINITY*
