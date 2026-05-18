# Zenodo Bundles Plan -- TRI-NET v1, v2, v3

> **Status:** PLAN. This document is a checklist / manifest for three
> future Zenodo bundles. **No DOI is asserted for v1, v2, or v3 in this
> document.** Zenodo issues DOIs at upload time; quoting one before that
> is not honest. The community page already lists a v0 baseline
> (B001..B007, plus the v5.0 parent record). See `docs/ZENODO.md` for the
> canonical list of *existing* DOIs.

---

## 1. Why three bundles

The TRI-NET line crosses four artefacts (toolchain + three chip repos)
and several conceptual layers (numeric kernel, generator backends,
formal proofs, conformance, assurance bridge). A single mega-bundle is
unreadable. A per-chip bundle leaks toolchain assumptions into each.
Three line-level bundles -- split by *role*, not by chip -- is the
working plan:

| Bundle | Working title                                            | Audience                  |
|--------|----------------------------------------------------------|---------------------------|
| v1     | TRI-NET v1 -- Spec-first toolchain and numeric registry  | tool builders, auditors   |
| v2     | TRI-NET v2 -- Open ternary silicon substrate              | silicon researchers       |
| v3     | TRI-NET v3 -- High-assurance proofs and conformance       | formal-methods reviewers  |

These three bundles correspond to the three persuasion lanes for the
line: "the toolchain is real", "the silicon is real (Tiny Tapeout shape)",
"the assurance story is real (Coq + conformance)".

---

## 2. v1 manifest -- Spec-first toolchain and numeric registry

**Inclusion checklist:**

- [ ] `README.md`, `STATUS.md`, `LINEUP.md`, `FORMAT_REGISTRY.md`,
  `COMPETITORS.md`, `BENCHMARKS.md`, `CLARA_TRACEABILITY.md`
- [ ] `docs/T27-CONSTITUTION.md`, `AGENTS.md`, `CLAUDE.md`, `SOUL.md`
- [ ] `docs/TRI_NET_API.md`, `docs/TRI_NET_WHITEPAPER.md`,
  `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`,
  `docs/22FDX_TOPS_W_PROJECTION.md` (this package)
- [ ] `specs/` (full tree)
- [ ] `bootstrap/` (Stage-0 Rust source; `FROZEN_HASH` seal verbatim)
- [ ] `schemas/` -- `numeric-format-v1.json`, `tri-net-api-v1.json`,
  `nmse-protocol-v1.json` (this package)
- [ ] `conformance/FORMAT-SPEC-001.json` (numeric SSOT)
- [ ] `CITATION.cff`, `codemeta.json`, `.zenodo.json`

**Exclusion list (do not include in v1):**

- `gen/` -- generated, reproducible from specs + sealed compiler.
- `coq/`, `trios-coq/`, `proofs/` -- belong in v3.
- Chip-repo content -- belongs in v2 via cross-link.

**Pre-upload gates:**

1. `scripts/check_first_party_doc_language.py` passes.
2. `FORMAT-SPEC-001.json` JSON sanity passes.
3. `bootstrap` builds with `cargo build --release`.
4. `./scripts/tri test` clean (or its CI surrogate).
5. `STATUS.md` ladder unchanged or only raised by direct evidence.
6. No `.env` or secret-shaped file under tree (`SECURITY.md` policy).

**Upload metadata template:**

```yaml
title: "TRI-NET v1 -- Spec-first Toolchain and Numeric Registry"
upload_type: software
license: Apache-2.0
communities:
  - identifier: trinity-s3ai
related_identifiers:
  - relation: isVersionOf
    identifier: "10.5281/zenodo.19227879"   # v5.0 parent record
  - relation: isPartOf
    identifier: "https://github.com/gHashTag/t27"
keywords:
  - TRI-NET, t27, GoldenFloat, GF16, ternary, spec-first
language: eng
```

---

## 3. v2 manifest -- Open ternary silicon substrate

**Inclusion checklist:**

- [ ] `LINEUP.md` (the four-product map) -- copy for offline reading
- [ ] `docs/TRI_NET_WHITEPAPER.md`
- [ ] `docs/22FDX_TOPS_W_PROJECTION.md`
- [ ] `gen/verilog/` index file (list, not contents -- chip repos are
  authoritative for HDL)
- [ ] `fpga/vivado/` build scripts and testbenches (treated as
  toolchain-side evidence, not silicon)
- [ ] `STATUS.md` (the readiness ladder)
- [ ] Cross-references to:
  - `tt-trinity-phi` GDS / pinout / Tiny Tapeout submission record
  - `tt-trinity-euler` GDS / pinout / Tiny Tapeout submission record
  - `tt-trinity-gamma` GDS / pinout / Tiny Tapeout submission record
- [ ] D2D protocol spec **pointer** (the protocol lives in
  `tt-trinity-euler` / `tt-trinity-gamma`; v2 bundles only the
  pointer plus the toolchain-side hooks)
- [ ] Triple-Deck (W47 RBB + W48 FBB-active + W49 CapBoost) **Coq
  lemma list** -- full sources go to v3

**Exclusion list (do not include in v2):**

- Raw Verilog (chip-repo authoritative).
- GDS files (chip-repo authoritative).
- Anything that would imply a silicon claim t27 can't back from this
  repo's evidence.

**Pre-upload gates:**

1. Each chip-repo cross-link resolves at upload time (live URL).
2. `STATUS.md` and `docs/22FDX_TOPS_W_PROJECTION.md` cross-check: no
  silicon claim exists in the projection doc that is missing from the
  ladder.

---

## 4. v3 manifest -- High-assurance proofs and conformance

**Inclusion checklist:**

- [ ] `coq/` (full tree)
- [ ] `trios-coq/` (full tree, incl. Physics/StochRound.v,
  AvsStacking.v, SubThreshold.v, StochSkipSafe.v, Int2QuantSafe.v,
  Avs96Safe.v, RBB / FBBActive2 / CapBoost)
- [ ] `proofs/` (work-in-progress lemma drawer; clearly labelled)
- [ ] `conformance/` (all vectors; not just numeric)
- [ ] `clara-bridge/` (assurance workflow)
- [ ] `docs/T27_KERNEL_FORMAL_COQ.md`
- [ ] `docs/PHYSICS_REVIEW_PROTOCOL.md`
- [ ] `docs/NUMERICS_VALIDATION.md`
- [ ] `docs/COMPILER_VERIFICATION_*` (the three landscape docs)
- [ ] `CITATION.cff` with `doi:` field populated (PASS-24 gate; currently
  open as issue #653 -- must be closed before v3 upload)

**Pre-upload gates:**

1. `coqc` clean across `coq/` and `trios-coq/` -- `Admitted` count zero
   for everything cited in the manifest.
2. `citation_map.json` consistent with bundled `.v` files.
3. `CITATION.cff` has `doi:` field (gate #653).

---

## 5. Common rules across v1, v2, v3

- License: Apache-2.0 (root `LICENSE` and `NOTICE`).
- Language: English / ASCII first-party (L3 PURITY).
- Author block: per `CITATION.cff` at upload time.
- Community: `trinity-s3ai`.
- Anchor identity stated in description: `phi^2 + 1/phi^2 = 3`.

**DOI policy (this is the heart of the document):**

> No DOI is issued, reserved, or quoted before the bundle is uploaded.
> Zenodo issues a DOI at first publish; a concept DOI (always-latest)
> appears once a second version is published. Until v1 is actually
> uploaded, the v1 row contains the string `pending`; quoting any other
> placeholder is forbidden.

The existing canonical Trinity / B-series DOIs (B001..B007 + v5.0 parent,
plus `10.5281/zenodo.19227877` cited in `NOW.md`) are **predecessor
records**, not the v1/v2/v3 line -- see `docs/ZENODO.md`.

---

## 6. Upload order

1. v1 first (toolchain) -- needed to anchor v2 and v3.
2. v3 second (proofs + conformance) -- read-only relative to specs;
   doesn't depend on v2.
3. v2 last -- depends on chip repos' GDS / submission records being
   public, which is the riskiest external dependency.

Each upload is its own issue with `Closes #N` discipline.

---

## 7. Cross-links

- Existing DOIs: `docs/ZENODO.md`
- Whitepaper: `docs/TRI_NET_WHITEPAPER.md`
- TRI-NET API contract: `docs/TRI_NET_API.md`
- Format registry: `FORMAT_REGISTRY.md`
- Readiness: `STATUS.md`
- Chip repos: `LINEUP.md` (the four-product map)
- `CITATION.cff` DOI gate: issue #653
- Roadmap: `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` -- OS-02 names the
  Coq export consumed by v3; PUB-02 names the sealed NMSE manifest
  whose existence is a v1 / v3 pre-upload signal.

---

## 8. R5-HONEST closing

This is a plan. None of v1, v2, v3 has a DOI. The plan is auditable
because each manifest is enumerable from this repo today; what it produces
is conditional on each pre-upload gate passing. If a future PR adds a v1
DOI without an upload behind it, that PR violates this document and
should be rejected.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
