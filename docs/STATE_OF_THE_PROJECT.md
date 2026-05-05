# State of the project — honest subsystem status

**Date anchor:** 2026-04-06 (update when rings or CI change materially)  
**Companion:** `docs/TECHNOLOGY-TREE.md` (roadmap), `CANON.md` (GOLD vs REFACTOR-HEAP)

This document is the **institutionalized reassessment**: what is **strong**, **in progress**, and **explicitly incomplete**.

---

## Summary

| Subsystem | Status | Notes |
|-----------|--------|--------|
| `.t27` spec corpus | **Strong** | ~45 specs; parse/gen sweep in CI; SSOT-MATH enforced. |
| Bootstrap `t27c` (Rust) | **Strong / evolving** | Rings 0–31 history; `FROZEN_HASH` + `build.rs` gates. |
| `gen/` tree (Zig primary) | **Strong** | Canonical `gen/zig`; `compile-all` default wired; headers validated. |
| Conformance vectors | **Strong** | 34 vectors; `validate_conformance.sh`. |
| Seals | **Strong** | 48 seals; verify in tests/CI. |
| SEED-RINGS / self-host narrative | **Good / partial** | Fixed-point smoke in `tests/run_all.sh`; **formal fixed-point proof** not in repo. |
| Rings **32–35** (hardening) | **In progress** | README / tech tree mark documentation, validation, CI enhancement — **not closed**. |
| Cross-backend equivalence | **Early** | Zig/C/Verilog gen exist; **bit-exact cross-backend** = Ring 39+ target. |
| GoldenFloat numerics | **Mixed** | Standards + specs; **differential oracle vs high-precision reference** = P1 (see `docs/NUMERICS_VALIDATION.md`). |
| Sacred / phi physics overlays | **Requires labeling** | Treat as **empirical / conjectural** unless proven; see `WHAT_REMAINS_SPECULATIVE.md`. |
| AR / CLARA chain | **Spec-rich** | Formal boundedness / soundness theorems **not** fully written. |
| FPGA / simulation | **Good start** | Lint/sim scripts exist; **waveform golden regressions** = P2 excellence. |
| Parser fuzzing | **Weak** | Not yet a documented corpus; excellence program target. |
| Monorepo periphery | **Noisy** | `external/`, bridges, backends — **not** part of core proof story (see `REPO_MAP.md`). |

---

## Parser and codegen

- **Parser:** exercised on full spec tree; **fuzzing** not yet first-class.  
- **Codegen:** Zig path most mature; C/Verilog paths follow; **round-trip CI diff** for all stable specs = planned.

---

## CI

- **Today:** Rust build gates, `compile-all` → `gen/zig`, `run_all.sh`, conformance, gen headers, seal counts.  
- **Target:** fast lane vs nightly full reproducibility vs release certification (see `REPOSITORY_EXCELLENCE_PROGRAM.md`).

---

## What we do **not** claim yet

- Full **formal semantics** document for entire t27 (skeleton: `docs/LANGUAGE_SPEC.md`).  
- **SLSA L3** provenance on releases (roadmap).  
- **Zenodo DOI** on every release (roadmap).

---

*Updating this file after major rings is **expected**, not optional.*
