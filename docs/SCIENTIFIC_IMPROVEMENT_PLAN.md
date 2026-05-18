# TRI-NET 2026 Scientific Improvement Plan -- t27 (toolchain side)

> **Status:** PLAN. This document is the **t27-side projection** of the
> line-level TRI-NET 2026 scientific improvement plan. It enumerates
> what the **toolchain product** of the line (specs, compilers,
> conformance vectors, schemas, Coq export, SDK) is expected to ship
> in 2026 to support the line's DARPA-CLARA-aligned, energy-efficient,
> SNN-fused, peer-reviewed, open-source posture.
>
> **R5-HONEST gating.** Every row in every table below carries one of
> three labels:
>
> - `VERIFY` -- claim sourced from outside this repo; integrator must
>   verify before quoting. No funding, no programme date, no paper
>   acceptance is asserted as fact in this document.
> - `projection` -- architecture-level estimate, not measured silicon.
> - `target` -- programmatic goal, not an achieved outcome.
>
> Anything that is **measured** in this repo today is in
> `STATUS.md` / `BENCHMARKS.md` instead, and is not duplicated here.

---

## 1. Scope

This plan covers what **`t27`** ships in 2026: tools, schemas, specs,
conformance, Coq export, and integration surfaces. Per-chip readiness
(GDS / pinout / silicon return) lives in the chip repos
(`tt-trinity-phi`, `tt-trinity-euler`, `tt-trinity-gamma`); their own
SCIENTIFIC_IMPROVEMENT_PLAN documents are the authoritative source
there. This document is the **toolchain mirror** of the line plan.

Cross-links to companion docs:

- `docs/TRI_NET_WHITEPAPER.md` -- positioning
- `docs/TRI_NET_API.md` -- external integration contract
- `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` -- numeric comparison protocol
- `docs/22FDX_TOPS_W_PROJECTION.md` -- energy projection methodology
- `docs/ZENODO_BUNDLES.md` -- DOI bundle plan (v1 / v2 / v3)
- `STATUS.md` -- readiness ladder (authoritative for measured state)
- `BENCHMARKS.md` -- restrained benchmark posture
- `COMPETITORS.md` -- honest positioning

---

## 2. DARPA CLARA alignment (CL-01..CL-04)

> **No claim is made that CLARA has funded this work; no programme
> date is named.** The rows below are technical alignments visible in
> this repo today, not funding statements. The full assurance
> workflow lives in `clara-bridge/` and `CLARA_TRACEABILITY.md`.

| ID | Track | t27 deliverable | Label |
|----|-------|------------------|-------|
| CL-01 | Drain-on-restraint semantic surface | Toolchain-side hooks in `docs/TRI_NET_API.md` so chip-repo D2D protocols can be consumed by external auditors without scraping. **D2D protocol itself lives in `tt-trinity-euler` / `tt-trinity-gamma`, not here.** | `target` |
| CL-02 | Assurance bridge (CLARA-style reasoning) | `clara-bridge/` exit-criteria documented; conformance vectors `conformance/ar_*.json` validated against the bridge demo. | `target` |
| CL-03 | Spec-to-RTL traceability | Every `gen/verilog/` artefact traceable to a sealed `.t27` spec; seal-hash field in `schemas/tri-net-api-v1.json#/$defs/RepoIdentity`. | `target` |
| CL-04 | Formal cross-walk | Coq export crate that ingests `.v` files from `coq/` and `trios-coq/` and emits a citation-map JSON consumable by external review tooling. | `target` |

**Out of scope here:** any statement on whether the chip repos hit
their own CL-01..CL-04 rows. Those are in their SIP documents.

---

## 3. Energy efficiency (EN-01..EN-03)

> **No `1000x` or `4000 TOPS/W` claim is restated as fact.** Any
> external press figure of that shape is `VERIFY` -- integrator must
> chase the source before quoting. The existing in-repo
> `28-120 TOPS/W` band stays `projection`, with back-links to
> `BENCHMARKS.md` and `docs/22FDX_TOPS_W_PROJECTION.md`.

| ID | Track | t27 deliverable | Label |
|----|-------|------------------|-------|
| EN-01 | Triple-Deck (RBB + FBB + CapBoost) toolchain support | Coq lemmas already landed in `trios-coq/Physics/CapBoost.v`, `FBBActive2.v`, plus W47 RBB (per `docs/NOW.md`). t27 ships the **citation-map JSON** that lets chip repos point a single artefact at the proofs. Chip-side RTL status (which of RBB / FBB / CapBoost is implemented vs witness-only) is reported in each chip's `TRIPLE_DECK_STATUS.md`. | `projection` (Coq) + `target` (citation export) |
| EN-02 | 22FDX TOPS/W projection methodology | `docs/22FDX_TOPS_W_PROJECTION.md` shipped with C1..C5 confidence bands; falsification policy enumerated. No measured silicon row. | `projection` |
| EN-03 | External press TOPS/W figures | Not restated in this document. If a press release names a figure (`1000x`, `4000 TOPS/W`, etc.), the integrator must `VERIFY` the source before quoting and must label the quoted number `projection` or `target` depending on the source's own framing. | `VERIFY` |

**Reminder.** `BENCHMARKS.md` is the authoritative restraint policy:
the repo publishes only numbers reproducible from a sealed spec or
generated file. When in doubt, omit the row.

---

## 4. SNN-TRI fusion (SN-01..SN-03)

> Surface-level only. t27 hosts the **format** and the **NMSE
> comparison protocol**; SNN integration on the silicon side is owned
> by the chip repos. No `Delta_dB` number exists in t27 today; this
> doc forbids quoting one until a `bench/results/nmse-*.json` lands.

| ID | Track | t27 deliverable | Label |
|----|-------|------------------|-------|
| SN-01 | NMSE harness for SNN-relevant distributions | `specs/benchmarks/gf16_bfloat16_nmse.t27` ships `D_NORM`, `D_LOG`, `D_RELU`, `D_PHI`, `D_DEEP`. SNN-specific tags can be added under `x_extension` (see `schemas/nmse-protocol-v1.json`) without breaking the schema MAJOR. | `target` |
| SN-02 | Theta-gate / StochSkip surface for fusion designs | Coq lemma `StochSkipSafe.v` already landed (hippocampal theta anchor, per `docs/NOW.md`); t27 ships the **lemma index** that chip repos and SNN integrators consume. | `projection` (lemma) + `target` (index export) |
| SN-03 | INT2 codebook for SNN spike weights | `Int2QuantSafe.v` Coq lemma already landed. t27 ships the **codebook description** in `FORMAT_REGISTRY.md` as a planned-bridge row; full `.t27` spec for the codebook is a `target` for 2026. | `projection` (lemma) + `target` (spec) |

---

## 5. Publication path (PUB-01..PUB-03)

> Framed as "draft and submit", not "accepted at venue X". No paper
> acceptance is asserted as fact.

| ID | Track | t27 deliverable | Label |
|----|-------|------------------|-------|
| PUB-01 | GoldenFloat family paper | `docs/WHITEPAPER/gf_paper_v3_imrad_draft.md` already exists in the repo. 2026 work: harden citations, run the NMSE protocol from PUB-02 on a fixed seed set, attach the manifest. | `target` |
| PUB-02 | NMSE manifest contributing to a publication | One sealed-toolchain `bench/results/nmse-*.json` produced under `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`. **No `Delta_dB` figure introduced before this manifest lands.** | `target` |
| PUB-03 | Tooling / spec-first methodology paper | A whitepaper-derivative doc that bundles `docs/TRI_NET_WHITEPAPER.md`, `STATUS.md`, `LINEUP.md`, `COMPETITORS.md` for an external review venue (workshop / preprint server). Draft only; submission is downstream. | `target` |

---

## 6. Open-source community (OS-01..OS-03)

| ID | Track | t27 deliverable | Label |
|----|-------|------------------|-------|
| OS-01 | TRI-NET API SDK -- Python (read-only) | A `tri-net-py` package (separate repo, name `target`) that consumes `schemas/tri-net-api-v1.json`, `schemas/nmse-protocol-v1.json`, `schemas/numeric-format-v1.json` and produces typed bindings. **Read-only**, file-based, mirroring `docs/TRI_NET_API.md`. No hosted endpoint added. | `target` |
| OS-02 | Coq export | A small crate / script under `tools/` that walks `coq/`, `trios-coq/`, `proofs/` and emits a single manifest JSON enumerating `Theorem` / `Lemma` / `Qed` / `Admitted` counts per file, plus a citation map. Consumed by Zenodo v3 (`docs/ZENODO_BUNDLES.md`). | `target` |
| OS-03 | Conformance vector contribution path | A `CONTRIBUTING.md` addendum that explains how to add a new conformance vector under `conformance/` and route it through `./scripts/tri validate-conformance`. Mirrors the L4 TESTABILITY mandate. | `target` |

The SDK row (OS-01) is **read-only**. Producer-side mutation of TRI-NET
artefacts is not in scope -- producers are the t27 build itself and the
chip-repo CI jobs, not the Python SDK.

---

## 7. Timeline

> Quarter labels are **targets**, not commitments. "Open" means no
> date is named because the work depends on artefacts outside t27's
> control (silicon return, external review timelines).

| Quarter | Theme | t27 deliverables |
|---------|-------|------------------|
| **Q2 2026** | NMSE harness usable end-to-end | `target` PUB-02 first sealed manifest; `target` OS-03 contribution path documented |
| **Q3 2026** | Energy projection auditable | `target` EN-02 22FDX projection table linked from chip-repo SIPs; `target` OS-02 Coq export draft |
| **Q4 2026** | Read-only Python SDK preview | `target` OS-01 SDK preview release; `target` SN-01 SNN-tagged NMSE manifest under `x_extension` |
| **open** | Silicon return for any chip in the line | t27-side: when a chip repo posts a measured TOPS/W or NMSE manifest, t27 wires it into `STATUS.md` and `bench/results/`. No date here. |
| **open** | Paper acceptance | t27-side: PUB-01 / PUB-03 drafts ready; acceptance is venue-controlled and not promised. |
| **open** | Zenodo v1 / v2 / v3 upload | Per `docs/ZENODO_BUNDLES.md` pre-upload gates. No DOI quoted. |

---

## 8. Success metrics

> The only metrics that count here are **CI-green workflows** and
> **committed artefacts**, both of which are auditable from this repo.
> Throughput / latency / energy metrics live in the chip repos when
> backed by silicon.

| Metric | Source of truth | Today | Target end of plan |
|--------|------------------|-------|--------------------|
| `.t27` specs that parse via `t27c parse` | README "System Status" + `STATUS.md` | 170+ | `target` 200+ |
| `gen/verilog/` modules that pass simulation | `STATUS.md` 2.1 | 5/5 | `target` 5/5 maintained; new modules tracked |
| Sealed artefacts under `.trinity/seals/` | `STATUS.md` 2.1 | 729 (at audit time) | `target` strictly non-decreasing |
| Coq files with zero `Admitted` cited in line plans | `trios-coq/Physics/`, `coq/` | per `docs/NOW.md` ledger | `target` zero `Admitted` in any file cited by Zenodo v3 manifest |
| Conforming NMSE manifests under `bench/results/` | `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` | 0 | `target` at least 1 sealed manifest for PUB-02 |
| Schema files validating draft-07 | `schemas/` | 3 (numeric-format / tri-net-api / nmse-protocol) | `target` 3+ maintained; new schemas added only via constitutional / issue review |
| Documents passing `scripts/check_first_party_doc_language.py` | the gate itself | PASS (today) | `target` PASS maintained |

No silicon-bound metric appears in this table on purpose. When
silicon does land, the metric joins `STATUS.md`, not this plan.

---

## 9. References

> Repo-internal references are authoritative. External references are
> labelled `VERIFY` and must be checked at the linked source before
> being quoted in derivative work.

**Repo-internal (authoritative):**

- `STATUS.md` -- readiness ladder
- `LINEUP.md` -- the four-product map
- `BENCHMARKS.md` -- restrained benchmark posture
- `COMPETITORS.md` -- honest positioning
- `FORMAT_REGISTRY.md` -- numeric SSOT mirror
- `CLARA_TRACEABILITY.md` -- assurance bridge to DARPA CLARA
- `docs/T27-CONSTITUTION.md` -- L1..L7 constitutional stack
- `docs/TRI_NET_WHITEPAPER.md` -- positioning paper
- `docs/TRI_NET_API.md` -- external integration contract
- `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` -- numeric comparison protocol
- `docs/22FDX_TOPS_W_PROJECTION.md` -- energy projection methodology
- `docs/ZENODO_BUNDLES.md` -- DOI bundle plan
- `docs/ZENODO.md` -- existing canonical DOIs (B001..B007 + v5.0 parent)
- `docs/NOW.md` -- rolling ledger of Coq lemmas and waves
- `trios-coq/Physics/` -- W34..W49 Coq lemmas
- `conformance/FORMAT-SPEC-001.json` -- numeric registry SSOT
- `schemas/numeric-format-v1.json`, `schemas/tri-net-api-v1.json`,
  `schemas/nmse-protocol-v1.json` -- public draft-07 schemas
- `CITATION.cff`, `codemeta.json`, `.zenodo.json` -- repo identity

**External (all `VERIFY` -- not quoted as fact in this doc):**

- DARPA CLARA program page (darpa.mil; date / wording must be checked
  at source before any derivative cites it).
- Any 22FDX vendor brief from GlobalFoundries (vendor page; cite at
  use, not pre-cached here).
- Any commercial-NPU TOPS/W figure (Coral / Hailo / Axelera / Qualcomm /
  MediaTek). `COMPETITORS.md` already documents the restraint.
- Line-level TRI-NET 2026 plan from coordinator notes outside this
  repo. **Authoritative copy is outside `t27`.** This document is
  the t27-side projection.

**Coq references** in `trios-coq/Physics/` cite published prior work
(Rabaey, Tschanz, Mukhopadhyay, Larsson and Svensson, Jiang et al.,
Hubara, Gupta) per the existing NOW ledger entries. Those are
secondary to the in-repo Coq lemmas and are not restated here.

---

## 10. What this document is NOT

- **NOT a funding statement.** No claim is made that DARPA / CLARA /
  any other agency has funded this work, contracted it, or
  scheduled it. `CL-01..CL-04` are technical alignment rows, not
  funding rows.
- **NOT a tape-out commitment.** No silicon arrival date is named.
  Chip-repo timelines are owned by chip repos.
- **NOT a paper acceptance claim.** `PUB-01..PUB-03` are
  draft-and-submit rows; acceptance is venue-controlled.
- **NOT a `1000x` or `4000 TOPS/W` claim.** Any such figure that
  surfaces in derivative work must carry `VERIFY` and a source URL.
- **NOT a new DOI.** Only `10.5281/zenodo.19227877` (existing B007
  record, per `docs/ZENODO.md`) is referenced; v1 / v2 / v3 DOIs do
  not exist until upload (see `docs/ZENODO_BUNDLES.md`).
- **NOT a hosted-service announcement.** The TRI-NET API remains
  file-based and read-only (see `docs/TRI_NET_API.md`).
- **NOT a NPU parity claim.** `COMPETITORS.md` continues to govern
  the line's restraint posture.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
