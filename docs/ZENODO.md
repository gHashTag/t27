# Zenodo DOI Registry — Trinity / t27

> **Single source of truth (community-level):** [`https://zenodo.org/communities/trinity-s3ai/`](https://zenodo.org/communities/trinity-s3ai/) — 12 records (8 v5.0 description stubs B001–B008 + 4 D-series D004–D007). Any DOI not in this community is **outside** the canonical Trinity S³AI surface.
>
> **Audit 2026-05-12 (PASS-6)** — registry re-verified under R5-honest deep
> sweep against `/api/communities/trinity-s3ai/records`. All March 2026
> DOIs (`19224xxx`) were marked SUPERSEDED on Zenodo and replaced by the
> May 2026 canonical set (`19227865`–`19227879`). Cite only the canonical
> set in new work.

## Honest framing

These DOIs are **software description stubs** on Zenodo, NOT peer-reviewed
papers. The mathematical anchor `φ² + φ⁻² = 3` is an algebraic identity from
`φ = (1+√5)/2`. Its Coq witness lives in this very repository
([`gHashTag/t27/coq`](https://github.com/gHashTag/t27/tree/main/coq)) —
**10 .v files, 48 statements (6 Theorem + 42 Lemma), 35 Qed, 0 Admitted,
audited 2026-05-12** (per the trinity-queen-hive skill canonical count;
the broader `t27/proofs/` tree adds further work-in-progress lemmas
outside the canonical `coq/` directory and is not counted here).

For "always-latest of B007" use **concept DOI** `10.5281/zenodo.19227876` (it
resolves to the current B007 version regardless of revisions).

## Canonical collection (curated, 2026-05-12, community `trinity-s3ai`)

| DOI | Title | Type | In community |
|-----|-------|------|--------------|
| [10.5281/zenodo.19227879](https://doi.org/10.5281/zenodo.19227879) | Trinity S³AI Framework — Complete Research Collection v5.0 (parent record) | Software description | ✅ trinity-s3ai |
| [10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696) | gHashTag/trinity v2.0.3 — FPGA Autoregressive Ternary LLM | Software release | ❌ outside (legitimate Vasilev record; not attached to community) |

## Individual bundles B001–B007 (canonical, all in community `trinity-s3ai`)

| DOI | Bundle | Description |
|-----|--------|-------------|
| [10.5281/zenodo.19227865](https://doi.org/10.5281/zenodo.19227865) | B001 | HSLM-1.95M ternary LM (training-recipe stub) |
| [10.5281/zenodo.19227867](https://doi.org/10.5281/zenodo.19227867) | B002 | Zero-DSP FPGA — ternary inference architecture sketch |
| [10.5281/zenodo.19227869](https://doi.org/10.5281/zenodo.19227869) | B003 | TRI-27 — ternary ISA with Coptic encoding |
| [10.5281/zenodo.19227871](https://doi.org/10.5281/zenodo.19227871) | B004 | Queen Lotus Cycle — autonomous orchestration |
| [10.5281/zenodo.19227873](https://doi.org/10.5281/zenodo.19227873) | B005 | Tri Language — linear types, algebraic effects |
| [10.5281/zenodo.19227875](https://doi.org/10.5281/zenodo.19227875) | B006 | GF16/TF3 — phi-based arithmetic |
| [10.5281/zenodo.19227877](https://doi.org/10.5281/zenodo.19227877) | B007 | VSA operations — bind/unbind/bundle |

## Retired (superseded on Zenodo 2026-05-12)

The following DOIs now have `[SUPERSEDED — see 10.5281/zenodo.<canonical>]`
title prefixes and `relation: isObsoletedBy` pointing to the canonical set
above. Do not cite them in new work.

| Retired DOI | Was | Canonical replacement |
|-------------|-----|----------------------|
| [10.5281/zenodo.19224105](https://doi.org/10.5281/zenodo.19224105) | Complete Collection (March-2026) | `19227879` |
| [10.5281/zenodo.19224139](https://doi.org/10.5281/zenodo.19224139) | Complete Collection v2 | `19227879` |
| [10.5281/zenodo.19224114](https://doi.org/10.5281/zenodo.19224114) | B001 (March) | `19227865` |
| [10.5281/zenodo.19224115](https://doi.org/10.5281/zenodo.19224115) | B002 (March) | `19227867` |
| [10.5281/zenodo.19224116](https://doi.org/10.5281/zenodo.19224116) | B003 (March) | `19227869` |
| [10.5281/zenodo.19224118](https://doi.org/10.5281/zenodo.19224118) | B004 (March) | `19227871` |
| [10.5281/zenodo.19224119](https://doi.org/10.5281/zenodo.19224119) | B005 (March) | `19227873` |
| [10.5281/zenodo.19224120](https://doi.org/10.5281/zenodo.19224120) | B006 (March) | `19227875` |
| [10.5281/zenodo.19224121](https://doi.org/10.5281/zenodo.19224121) | B007 (March) | `19227877` |
| [10.5281/zenodo.18939351](https://doi.org/10.5281/zenodo.18939351) | Trinity v2.0.3 FPGA LLM (older record) | use `18950696` for v2.0.3 |

## Cross-sibling registry

The full Trinity hive Zenodo registry (with related identifiers, communities,
and supersession history for all 12 canonical records + 28 retired versions)
lives in [`gHashTag/trios/docs/infrastructure/zenodo-registry.md`](https://github.com/gHashTag/trios/blob/main/docs/infrastructure/zenodo-registry.md).

— end of registry —
