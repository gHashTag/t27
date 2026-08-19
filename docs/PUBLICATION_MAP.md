# Publication map — which part of t27 → which venue

**Purpose:** Route work packages to **PL, formal methods, hardware, numerics, ML safety**, without overselling immature pieces.

**Publishing conveyor:** [`publications/README.md`](../publications/README.md) (DOI catalog + series), [`docs/PUBLICATION_PIPELINE.md`](PUBLICATION_PIPELINE.md), [`docs/PUBLICATION_AUDIT.md`](PUBLICATION_AUDIT.md).

---

## Suggested routing

| Repo focus | Venue style | Example angle |
|------------|-------------|---------------|
| SEED-RINGS, self-host, incremental compiler | PL / compilers workshop or journal | Ghuloum-style narrative + frozen hash discipline |
| `LANGUAGE_SPEC` + soundness fragments | Formal methods (CPP, ITP workshop, FM) | Core fragment semantics |
| GoldenFloat + validation | Numerics / HPC / arithmetic | Error bounds, differential testing |
| K3 / ternary AR, bounded traces | Logic + XAI / neurosymbolic | Bounded reasoning, explainability depth |
| FPGA / MAC / Verilog | FPL, DATE, FPGA journal | Resource / timing vs spec |
| PHI LOOP, seals, FROZEN, CI | SE / reproducibility / governance | Integrity constraints on research software |
| Physics-flavored specs (labeled empirical) | Physics / interdisciplinary | **Only** with honest tier labels |
| **Low-precision benchmark methodology** (TNF article) | **ML systems / reproducibility** (MLSys, or a numerics venue) | Two trainer defects each *reversing* a reported effect; parity reached by removing them, not by architecture (T456b) |

---

## Exploratory preprints

Anything **Tier D** in `docs/PHYSICS_REVIEW_PROTOCOL.md` should go to **preprint** first, not be bundled as core PL truth.

---

## One PhD, many papers

See `docs/PHD-RESEARCH-PROGRAM-AND-DISSERTATION.md` for WP decomposition.

---

*Do not submit the entire monorepo as one paper — slice by falsifiable unit.*

---

*W845: the TNF article had no row here. Its own reconciliation (§6) concluded it
is a **methods paper**, not a format paper — "low-precision benchmarks manufacture
orderings that do not exist" — which routes it away from FPL/DATE and toward a
venue that takes negative and methodological results. **That routing is a human
decision and is recorded here as a proposal, not a settled choice.***
