# CLARA_TRACEABILITY.md -- Mapping to DARPA CLARA Public Goals

> **Scope:** this document maps **public-facing** goals of DARPA's CLARA
> program to **specific artefacts in this repository**, so that an
> external reviewer can trace claim -> file.
>
> It is **not** a claim of CLARA participation, award, or endorsement.
> Where the word "CLARA" appears below, it refers to the **publicly
> described program** at:
>
> - DARPA CLARA: https://www.darpa.mil/research/programs/clara

---

## 1. Why this document exists

The TRI-NET line is positioned in the **high-assurance** corner of AI
silicon (see [`COMPETITORS.md`](COMPETITORS.md)). DARPA CLARA is the most
visible public program articulating goals in this corner. Rather than make
loose "CLARA-aligned" statements, this document gives a single page that
points each public CLARA goal at a file or directory in this repo, with
honest gaps marked.

Sources of CLARA goals: the public program page above. All wording of
"CLARA goal" rows is paraphrased from public material; treat the linked
page as authoritative.

---

## 2. Mapping table

The "goal" column paraphrases public language; the "artefact" column points
into this repo; the "level" column uses [`STATUS.md`](STATUS.md) readiness
levels where it makes sense, or `n/a` where the artefact is a document not
a build target.

| Public CLARA goal (paraphrased)                                 | Artefact in this repo                                       | Level     |
|-----------------------------------------------------------------|-------------------------------------------------------------|-----------|
| Compositional AI assurance -- combining ML and AR components    | `clara-bridge/` (4 hybrid patterns documented in README)    | demo      |
| Bounded reasoning / explainability over inference steps         | `clara-bridge/` proof-trace work; `specs/ar/` AR specs      | demo/SPEC |
| Polynomial-time complexity guarantees on the assurance path     | Stated in `clara-bridge/README.md`; specs under `specs/ar/` | demo      |
| Formal verification of components prior to composition          | `coq/Kernel/`, `coq/Theorems/`, `coq/IGLA/`, `proofs/`      | partial   |
| Reproducible build pipeline auditable end-to-end                | `.t27` -> `t27c` -> `gen/*` -> `.trinity/seals/`            | GREEN     |
| Open and inspectable artefacts                                  | This repo + linked chip repos (see [`LINEUP.md`](LINEUP.md))| n/a       |

Honest gaps:

- **No claim of submission acceptance** to CLARA. `clara-bridge/submission/`
  and `clara-bridge/proposal/` are documents in this repo; their state in
  any external program is **not** asserted here.
- **No claim of CLARA TA-level mapping** (Technical Area X.Y wording).
  When the public program page details such structure, a follow-up PR can
  refine this table.
- **Coq surface is "partial"**: see [`STATUS.md`](STATUS.md) section 2.4.

---

## 3. Reproducing the trace

Anyone with this repo can verify the mapping:

```bash
# Spec-to-gen reproducibility
./scripts/tri parse specs/numeric/gf16.t27
./scripts/tri gen-verilog specs/numeric/gf16.t27
./scripts/tri seal specs/numeric/gf16.t27 --verify

# Assurance bridge examples
ls clara-bridge/
cat clara-bridge/README.md

# Conformance gating
./scripts/tri validate-conformance
./scripts/tri validate-gen-headers
```

The intent is: a reviewer who has never used t27 should be able to land
on this file, click into each row's artefact, and reach a reproducible
build or proof from there.

---

## 4. What this document is not

- Not a CLARA program proposal (those live, if at all, under
  `clara-bridge/submission/` and `clara-bridge/proposal/`).
- Not a CLARA endorsement or affiliation statement.
- Not a substitute for [`STATUS.md`](STATUS.md), which still governs
  what level each component is at.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
