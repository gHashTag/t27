# FPGA Loop Evidence — Wave Loop 432

**Date:** 2026-07-01  
**Issue:** #1391  
**Branch:** `wave-loop-432`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was executed

Wave Loop 432 executed **Variant C2** of the FPGA boot-evidence plan: a board-less
formal fallback because the physical bench is still blocked (P12 CCLK probe
unwired, no relay cold-POR gate, no DLC10 cable connected).

The wave opened by probing the `origin/master` merge path for the `gen-verilog`
fix set (`701d79b3b`) that clears the 7 residual yosys smoke failures. The merge
brought in the gf128/gf96 conformance promotion, but the gen-verilog fix commits
(`701d79b3b`, `507408f47`) are on a divergent `master` lineage not reachable
from `origin/master` relative to `wave-loop-432`. A direct cherry-pick of
`507408f47` also conflicts heavily with the wave-loop compiler state. The wave
therefore redirected to a safe, board-less formal deliverable: **per-process-corner
raw-ns OSCFSEL theorems** in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.

No new physical capture was performed; all validation is board-less.

---

## Evidence items

### 1. Per-process-corner raw-ns OSCFSEL theorems

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added two quantified theorems for every documented Artix-7 OSCFSEL selection
(0..7) and every process corner (`ff`, `tt`, `ss`), at the worst-case temperature
(+85 °C) and minimum VCCINT (900 mV):

- `cclk_variant_raw_ns_per_process_corner_pvt_satisfies_flash_spec`
  - For each `oscfsel ≤ 7` and each `ProcessCorner`, the ideal raw-ns capture
    whose period equals `cclk_period_ns oscfsel` and whose low/high times split
    the period exactly satisfies the PVT-aware raw-ns flash predicate.
- `cclk_variant_raw_ns_per_process_corner_pvt_implies_transaction_ok`
  - The same ideal capture produces a flash-spec-compliant SPI read transaction.

These theorems close the corner-envelope gap: a future real measurement tagged
with any process corner can be justified by the same formal predicate without
re-proving the per-OSCFSEL arithmetic.

### 2. `tri fpga sweep-report --json` remains available

`cli/tri/src/fpga.rs`

The machine-readable `sweep-report --json` path added in earlier waves remains
operational. The W432 validation did not require new code here; the existing
JSON report (variants, first working OSCFSEL, next steps) is part of the
board-less tooling baseline.

### 3. Master-merge feasibility probe

- Attempted: `git merge origin/master` into `wave-loop-432`.
- Result: only the gf128/gf96 conformance promotion merged; the `gen-verilog`
  fix set stayed unreachable.
- Attempted: cherry-pick `507408f47`.
- Result: heavy conflicts in `bootstrap/src/compiler.rs`,
  `.trinity/seals/fpga_ZeroDSP_BPSK.json`, and `docs/NOW.md`.
- Decision: abort the merge and ship a formal lemma instead, documenting the
  infeasibility for a future dedicated merge/rebase wave.

### 4. Documentation refresh

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`: refreshed for W432; noted the
  quantified per-process-corner theorem, the blocked bench, the unchanged 7
  residual yosys failures, and July 2026 competitor signals (firtool 1.152.0,
  Aria-HDL retiming/PCIe BAR updates, Clash 1.11 candidate).
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`: W432 triage — the 7 residual
  yosys smoke failures remain deferred after the master-merge probe showed the
  fix set is not safely reachable from `wave-loop-432`.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test --bin tri fpga::` | **81 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **49 passed, 7 pre-existing failures** (#1245) |

The 7 pre-existing yosys smoke failures are unchanged:
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

These are covered by the full fix set on `master` (commit `701d79b3b`), which is
on a divergent lineage relative to `wave-loop-432`.

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel, so real
  CCLK frequency/duty capture for any OSCFSEL variant is not possible.
- **Relay / remote-power cold-POR gate:** still not wired, so automated
  cold-POR SPI flash boot sweeps require manual power cycling.
- **DLC10 cable:** the on-board Xilinx Platform Cable USB II is still not
  connected; the working path remains the Digilent HS2 cable plus
  `openFPGALoader`.

---

## Artifacts

- `proofs/lean4/Trinity/TernaryFPGABoot.lean` — per-process-corner raw-ns OSCFSEL
  theorems.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed competitor snapshot.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — updated #1245 triage with
  master-merge feasibility result.
- This evidence note.

*φ² + φ⁻² = 3 | TRINITY*
