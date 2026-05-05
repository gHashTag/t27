# Trinity Framework Publications — index (t27 hub)

**Purpose:** Single **publisher-facing** index for DOIs, publication **series**, and links between the **t27** repo and the broader **Trinity** monorepo. This is not a substitute for [`CITATION.cff`](../CITATION.cff) or [`docs/RESEARCH_CLAIMS.md`](../docs/RESEARCH_CLAIMS.md) — it is the **catalog and pipeline entrypoint**.

**Maintainer:** Dmitrii Vasilev — [ORCID 0009-0008-4294-6159](https://orcid.org/0009-0008-4294-6159).

---

## Concept DOI (umbrella)

| Role | DOI | Note |
|------|-----|------|
| Trinity Framework Publications — **all versions** | [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017) | Use as stable umbrella when citing the ecosystem. |
| Latest Trinity Framework snapshot (as registered) | [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696) | Version-specific; prefer concept DOI for “the programme”. |

---

## Publication series (Zenodo routing)

Use these **series tags** in Zenodo metadata keywords and in release notes so deposits are searchable and policy-compliant.

| Series | Scope (typical artifacts) | Primary repo |
|--------|---------------------------|--------------|
| **Core language** | Canonical spec, parser/ISA notes, conformance corpus, backend contracts, `LANGUAGE_SPEC` snapshots | **t27** |
| **Numerics** | GoldenFloat validation reports, differential-test bundles, numeric benchmark CSV | **t27** / trinity |
| **Hardware** | Verilog backends, FPGA flow notes, waveform/simulation packs | **t27** / trinity |
| **AI / agents** | TRI CLI snapshots, agent-loop reports, Ouroboros logs (when methods are explicit) | trinity |
| **Physics / research** | Phi-structure audits, CODATA delta reports, claim-status tables as standalone reports | **t27** / Zenodo-only |
| **Audit / repro** | Reproducibility bundles, release certification, independent verification packs | **t27** |

---

## Registered DOIs (ecosystem — mirror of `CITATION.cff`)

| DOI | Title / role | Series (suggested) | Source repo |
|-----|----------------|-------------------|-------------|
| [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017) | Concept — all versions | Audit / umbrella | Trinity programme |
| [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696) | Latest framework version | Core / umbrella | trinity |
| [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352) | FPGA Autoregressive Ternary LLM | Hardware / AI | trinity |
| [10.5281/zenodo.19020211](https://doi.org/10.5281/zenodo.19020211) | Self-Evolving Ouroboros | AI / agents | trinity |
| [10.5281/zenodo.19020213](https://doi.org/10.5281/zenodo.19020213) | VSA Balanced Ternary + SIMD | Numerics / AI | trinity |
| [10.5281/zenodo.19020215](https://doi.org/10.5281/zenodo.19020215) | phi-RoPE Attention | AI | trinity |
| [10.5281/zenodo.19020217](https://doi.org/10.5281/zenodo.19020217) | Sparse Ternary MatMul | Hardware / numerics | trinity |
| [10.5281/zenodo.19227877](https://doi.org/10.5281/zenodo.19227877) | VSA Operations for Ternary Computing | Numerics / AI | trinity |

**Preferred citation for phi-structures paper:** see `preferred-citation` in [`CITATION.cff`](../CITATION.cff) (Vasilev & Pellis, 2026).

---

## Read papers and documentation

- **Trinity documentation site:** [gHashTag.github.io/trinity](https://gHashTag.github.io/trinity) — research and DePIN docs.  
- **Zenodo community / records:** search “Trinity” and the DOIs above.  
- **This repository (language kernel):** [github.com/gHashTag/t27](https://github.com/gHashTag/t27).  
- **Umbrella monorepo:** [github.com/gHashTag/trinity](https://github.com/gHashTag/trinity).

---

## Pipeline and audit (normative)

| Document | Role |
|----------|------|
| [`docs/PUBLICATION_PIPELINE.md`](../docs/PUBLICATION_PIPELINE.md) | Release → Zenodo → metadata — **Trinity Publication Policy** |
| [`docs/PUBLICATION_AUDIT.md`](../docs/PUBLICATION_AUDIT.md) | Readiness matrix per artifact |
| [`docs/PUBLICATION_MAP.md`](../docs/PUBLICATION_MAP.md) | Venue / audience routing for papers |
| [`docs/PUBLICATION_QUEUE.md`](../docs/PUBLICATION_QUEUE.md) | Next deposits — each line should have a **GitHub issue** |
| [`docs/ROADMAP.md`](../docs/ROADMAP.md) / [`docs/NOW.md`](../docs/NOW.md) | Public execution index |

---

## t27 — next Zenodo candidates (not yet registered)

| Candidate | Suggested type | Blockers |
|-----------|----------------|----------|
| t27 canonical language spec snapshot | `software` + doc | Finalize `docs/LANGUAGE_SPEC.md`; tag release |
| TRI-27 conformance vector corpus | `dataset` | Schema doc, version string, checksum manifest |
| GoldenFloat validation report | `report` | Fill `docs/NUMERICS_VALIDATION.md` tables + CSV outputs |
| Sacred formula catalog + claim statuses | `report` | Export from `docs/RESEARCH_CLAIMS.md` + specs |
| Reproducibility bundle | `other` / `software` | Pin toolchain; `repro/` one-command parity |

---

*φ² + 1/φ² = 3 | TRINITY — publish on a schedule, not only when convenient.*
