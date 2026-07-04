# Wave Loop 108 Plan
## Trinity S³AI — Competitive Hardening + Codebase Hygiene

**Date:** 2026-06-16  
**Branch:** `trinity-rust-rings`  
**Previous:** [Wave Loop 107](../wave_loop_107_report.md)

---

## Weaknesses Discovered (W108 Audit)

| Weakness | Severity | Count |
|----------|----------|-------|
| 61 specs with TODO/FIXME/placeholder/extern fn | MEDIUM | 61 |
| Only 99/564 specs have bench blocks | MEDIUM | 465 naked |
| 9 .tri stub files remain unmigrated | MEDIUM | 9 |
| New competitors: VerilogCL, EvolVE 98.1%, VeriGraphi, LLM4RTL | HIGH | 4 new |
| Physics formal verification gap: SU(5) Lean, SK_EFT_Hawking | HIGH | 2 repos |
| No empirical Pass@K baseline | CRITICAL | — |
| Lean 4 bridge only 5 modules | MEDIUM | — |

---

## Tracks

### Track A — Competitive Intelligence Update (HIGH)
Add 4 new RTL generation competitors discovered in June 2026 arXiv sweep:
- **VerilogCL** (arXiv:2604.18162) — Contrastive learning for robust generation
- **EvolVE** (arXiv:2601.18067) — 98.1% VerilogEval v2, evolutionary+MCTS
- **VeriGraphi** (arXiv:2604.14550v2) — Multi-agent hierarchical RTL
- **LLM4RTL** (arXiv:2606.15500) — Tool-assisted JRCRC pipeline

Add 2 physics formal verification competitors:
- **Physics as Code** (arXiv:2603.28406) — SU(5) GUT in Lean 4
- **SK_EFT_Hawking** (GitHub) — 9944 theorems, 0 sorry, SM fingerprints

### Track B — TODO Elimination Sprint (MEDIUM)
Clean highest-priority TODOs/placeholders from critical-path specs:
- `specs/base/ternary_memory.t27` — memory primitive on hot path
- `specs/test_framework/core.t27` — test infrastructure
- `specs/fpga/spi.t27` — FPGA SPI driver

### Track C — L4 Benchmark Expansion (MEDIUM)
Add bench blocks to 10 additional naked specs on critical paths.

### Track D — .tri Stub Migration (MEDIUM)
Migrate 3 highest-priority .tri stubs to t27 syntax with bench blocks.

### Track E — Lean 4 Bridge Expansion (MEDIUM)
Add physics formal verification awareness to Lean 4 bridge.

---

## Cooperation Variants (W109)

1. **RTL Dataset Engineer** — Partner with OpenRTLSet-scale dataset curator for 100K+ module corpus
2. **Contrastive Learning Researcher** — Collaborate on VerilogCL-style correctness boundary learning
3. **Physics Formalization Partner** — Joint work with SK_EFT_Hawking or GIFT teams on SM proofs

---

*φ² + 1/φ² = 3 | TRINITY*
