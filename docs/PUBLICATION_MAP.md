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

---

## Exploratory preprints

Anything **Tier D** in `docs/PHYSICS_REVIEW_PROTOCOL.md` should go to **preprint** first, not be bundled as core PL truth.

---

## One PhD, many papers

See `docs/PHD-RESEARCH-PROGRAM-AND-DISSERTATION.md` for WP decomposition.

---

*Do not submit the entire monorepo as one paper — slice by falsifiable unit.*
