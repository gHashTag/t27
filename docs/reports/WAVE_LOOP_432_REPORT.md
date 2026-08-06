# Wave Loop 432 Report — FPGA boot-evidence per-process-corner raw-ns theorems

**Date:** 2026-07-01  
**Issue:** #1391  
**Branch:** `wave-loop-432`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 432 executed **Variant C2** of the FPGA boot-evidence plan: the bench
remains blocked (no P12 CCLK probe, no relay cold-POR gate, no DLC10 cable), so
the wave focused on a board-less formal deliverable.

The key deliverable is a pair of **quantified per-process-corner raw-ns OSCFSEL
theorems** in `proofs/lean4/Trinity/TernaryFPGABoot.lean`. For every documented
Artix-7 CCLK selection (OSCFSEL 0..7) and every process corner (`ff`, `tt`,
`ss`), the theorems prove that the ideal raw-ns CCLK capture satisfies the
PVT-aware flash-spec predicate and produces a flash-spec-compliant SPI read
transaction at the worst-case temperature (+85 °C) and minimum VCCINT (900 mV).

The wave also probed the `origin/master` merge path for the `gen-verilog` fix
set (`701d79b3b`) that clears the 7 residual yosys smoke failures. The merge
was not feasible: the fix commits are on a divergent `master` lineage not
reachable from `origin/master` relative to `wave-loop-432`, and a direct
cherry-pick conflicts heavily with the wave-loop compiler state. The failures
are documented as a known baseline and left for a future dedicated merge/rebase
wave.

---

## Deliverables

### 1. Per-process-corner raw-ns OSCFSEL theorems

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- `cclk_variant_raw_ns_per_process_corner_pvt_satisfies_flash_spec`
  - Parameterized over `oscfsel : Nat` (with `oscfsel ≤ 7`) and
    `corner : ProcessCorner`.
  - Proves that the ideal raw-ns capture
    (`period_ns = cclk_period_ns oscfsel`, `low_ns = period_ns / 2`,
    `high_ns = period_ns - low_ns`) satisfies the PVT-aware raw-ns flash predicate
    at the worst-case envelope corner.
  - Covers 24 concrete combinations (8 OSCFSEL values × 3 process corners) in a
    single quantified theorem proved by `interval_cases` and `cases`.

- `cclk_variant_raw_ns_per_process_corner_pvt_implies_transaction_ok`
  - Lifts the predicate theorem to an end-to-end transaction-safety theorem for
    any transaction size.

These lemmas let downstream `measured-to-lean` proofs reference a single
quantified theorem for any documented OSCFSEL/process-corner pair, instead of
re-proving the arithmetic per measurement.

### 2. Master-merge feasibility probe

- Attempted `git merge origin/master` into `wave-loop-432`.
- The merge brought in the gf128/gf96 conformance promotion but **not** the
  `gen-verilog` fix commits (`701d79b3b`, `507408f47`).
- Attempted cherry-pick of `507408f47`; it conflicts with
  `bootstrap/src/compiler.rs`, `.trinity/seals/fpga_ZeroDSP_BPSK.json`, and
  `docs/NOW.md`.
- Decision: abort and document; ship a formal lemma instead.

### 3. Documentation and triage

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`: refreshed for W432; noted the new
  per-process-corner theorem, the blocked bench, the unchanged 7 residual yosys
  failures, and July 2026 competitor signals.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`: W432 triage — master-merge
  attempted and found not feasible; the 7 yosys smoke failures remain the
  documented baseline.
- `docs/reports/FPGA_LOOP_EVIDENCE_W432_2026-07-01.md`: this report's companion
  evidence note.
- `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md`: next-wave cooperation
  variants.

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
divergent lineage relative to `wave-loop-432`.

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel.
- **Relay / remote-power cold-POR gate:** still not wired.
- **DLC10 cable:** still not connected; the Digilent HS2 + `openFPGALoader` path
  remains the working one.

---

## Next wave

Wave Loop 433 should execute the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md`.

*φ² + φ⁻² = 3 | TRINITY*
