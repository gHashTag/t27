# W480 — Next-Wave Cooperation Variants

**Date:** 2026-07-09  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

The W479 close-out left the repository in an **acceptable** state: yosys smoke is clean, Icarus smoke has 17 documented baseline failures inside `igla/`, all non-smoke tests pass, seals are green, and `cargo test` is green. The next wave can therefore be chosen on engineering merit rather than fire-fighting. Three mutually exclusive variants are proposed.

---

## Variant A — FPGA live-boot evidence (hardware-dependent)

**Goal:** produce a new board-level evidence artifact on the QMTech Wukong XC7A100T.

| Item | Detail |
|------|--------|
| **What** | Cold-POR SPI flash boot or SRAM load with a W480 bitstream, capture `STAT` and `DONE` evidence, and cross-check against the PVT-envelope theorems. |
| **Why** | The FPGA hardware SSOT (`fpga/HARDWARE_SSOT.md`) is authoritative; physical boot evidence is the highest-confidence validation of the RTL→bitstream pipeline. |
| **Risk** | Blocked when the DLC10 cable is unavailable; requires board access. |
| **Success metric** | A new `docs/reports/FPGA_EVIDENCE_W480_*.md` with measured `STAT` and `DONE=1`, plus a matching `--dry-run-live` / measured-to-lean summary. |
| **Recommended if** | The board and cable are on the bench and the prior W470–W479 compiler work has generated a bitstream that is ready to load. |

---

## Variant B — Reduce the Icarus baseline further (default)

**Goal:** shrink the documented Icarus baseline from 17 to as close to 0 as feasible without rewriting the compiler.

| Item | Detail |
|------|--------|
| **What** | Fix the concrete root causes visible in the W479 classification: dead-store/scope visibility errors (`results_test_pass`, `total`, `idx`, `decoded`, `pass_count`, `m_assigns`, etc.), duplicate bench-name declarations (`cordic_top`), wildcard `_` identifiers (`tokenizer`, `opcodes`), and indefinite-width signed literals in concatenations (`cordic`). |
| **Why** | Each fix is small and local, but together they materially improve the honesty and quietness of the Icarus gate. |
| **Risk** | Some failures stem from host-side recursive helpers that are genuinely not synthesizable; those should remain in the baseline rather than being hacked around. |
| **Success metric** | ≥120/127 Icarus smoke targets pass with ≤7 documented baselines, all non-smoke tests green, seals green. |
| **Recommended if** | No hardware is available and the next priority is compiler-backend robustness. |

---

## Variant C — Formal subset predicate / Lean bridge (fallback)

**Goal:** define and mechanically state the Icarus-supported t27 subset in Lean 4.

| Item | Detail |
|------|--------|
| **What** | Add a predicate `is_icarus_supported : t27_expr → Prop` that captures the closed patterns (fixed-size scalar arrays, static `.len`/`.contains`, non-recursive functions, no strings/queues/classes). State and prove that the W479 lowering preserves semantics for every expression in the predicate. |
| **Why** | Literature (Lutsig, Vera) shows that verified compilers must bound their source subset. W479 added the practical boundary; Variant C makes the boundary formal. |
| **Risk** | Requires new Lake package scaffolding and proof work; unlikely to close in one wave unless scoped tightly. |
| **Success metric** | A new `lake/` package with the predicate, at least one preservation lemma for static `.len()`, and a CI step that checks `specs/scratch/w479_icarus_supported_subset.t27` is in the supported fragment. |
| **Recommended if** | FPGA is unavailable and the team wants to invest in long-term compiler correctness rather than incremental Icarus fixes. |

---

## Recommended choice

**Variant B is the default.** It directly continues the W479 momentum, has low risk, and produces a measurable improvement in the Icarus gate. Variant A should be taken only if hardware is confirmed available. Variant C is a strong fallback if the remaining Icarus failures are mostly structural (scope/DCE) and can be closed quickly, leaving time for a small formal lemma.

---

*φ² + φ⁻² = 3 | TRINITY*
