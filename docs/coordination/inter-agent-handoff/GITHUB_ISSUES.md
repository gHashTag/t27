# GitHub issues — templates (scientific excellence)

**How to use:** Each `##` section below can become one GitHub Issue.  
**Labels (suggested):** `P0`, `P1`, `P2`, `epic`, `scientific-excellence`, `documentation`, `testing`, `ci`, `hygiene`  
**Law:** Still use **`Closes #N`** on substantive PRs; post parallel work notes on **[#141](https://github.com/gHashTag/t27/issues/141)**.

**Repo snapshot 2026-04-06:** [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md), [`docs/NUMERICS_VALIDATION.md`](../../NUMERICS_VALIDATION.md), [`docs/STATE_OF_THE_PROJECT.md`](../../STATE_OF_THE_PROJECT.md), [`docs/nona-02-organism/LANGUAGE_SPEC.md`](../../nona-02-organism/LANGUAGE_SPEC.md), [`CITATION.cff`](../../../CITATION.cff), [`CONTRIBUTING.md`](../../../CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](../../../CODE_OF_CONDUCT.md), [`docs/SECURITY.md`](../../../SECURITY.md) **already exist** — tasks below say **extend / audit** where applicable.

---

## Issue: [EPIC-01] Claim taxonomy & intellectual honesty

**Labels:** `P0`, `epic`, `scientific-excellence`

### Description

Formalize **complete coverage**: every strong claim in README, `SOUL.md`, sacred/physics specs, and outreach appears in [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md) with evidence and falsification path (constitution **EVIDENCE-LEVELS**).

### Tasks

- [ ] **Audit** README / `SOUL.md` / key `docs/*` against [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md); add missing rows (**C-*** IDs).
- [ ] Optional memos: `WHAT_REMAINS_SPECULATIVE`, `WHY_THIS_IS_NOT_NUMEROLOGY`, `PHYSICS_REVIEW_PROTOCOL` — **only** if Architect approves new files (avoid doc explosion).

### Acceptance criteria

- No outreach above the certainty level in the registry.
- Physics-adjacent rows explicitly marked per **RESEARCH_CLAIMS** vocabulary.

---

## Issue: [EPIC-02] Separate core language from research extensions

**Labels:** `P0`, `epic`, `scientific-excellence`

### Description

Design `specs/core/` vs `specs/research/` (or metadata-only maturity) per constitution **PURPOSE-SCOPE**. **High risk** — needs ring-scoped issues and codegen/conformance updates.

### Tasks

- [ ] ADR + migration plan; pilot on a **small** spec subset.
- [ ] README: core language first; research extensions clearly labeled.

### Acceptance criteria

- Core language intelligible without optional physics narrative.

---

## Issue: [EPIC-03] Reproduction pipeline

**Labels:** `P0`, `epic`, `scientific-excellence`

### Description

One-command reproducibility aligned with [`docs/PUBLICATION_PIPELINE.md`](../../PUBLICATION_PIPELINE.md).

### Tasks

- [ ] `repro/Makefile` (or script) — parse stable specs, conformance spot-check, documented expected hashes.
- [ ] `REPRODUCING.md`; pin Rust (e.g. `rust-toolchain.toml` in `bootstrap/`); document Verilog simulators if used.
- [ ] Zenodo DOI when ready (publication track).

### Acceptance criteria

- Fresh clone can verify a **defined** subset in bounded time.

---

## Issue: [EPIC-04] Formal language specification

**Labels:** `P1`, `epic`, `documentation`

### Description

Bring [`docs/nona-02-organism/LANGUAGE_SPEC.md`](../../nona-02-organism/LANGUAGE_SPEC.md) to parity with compiler; add backend invariants.

### Tasks

- [ ] Complete / correct grammar + semantics sections.
- [ ] Add `docs/BACKEND_CONTRACT.md` (Zig / C / Verilog obligations).
- [ ] Incremental SPEC metadata headers on `.t27` files (**RING-LAW**: small PRs).

### Acceptance criteria

- External reader can understand the language without reading Rust first.

---

## Issue: [EPIC-05] Numeric validation & benchmarking

**Labels:** `P1`, `epic`, `testing`

### Description

Harden GoldenFloat narrative with measurable comparisons — see Ring **#129** and [`docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md`](../../COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md).

### Tasks

- [ ] **Extend** [`docs/NUMERICS_VALIDATION.md`](../../NUMERICS_VALIDATION.md) with rounding, NaN policy, phi-distance protocol.
- [ ] Exhaustive or sampled tests for small formats; comparative GF16 vs IEEE / takum-class baselines.
- [ ] Optional Python/mpmath reference for differential testing.

### Acceptance criteria

- No “superiority” marketing without tables + Zenodo/registry update.

---

## Issue: [EPIC-06] Test infrastructure & fuzzing

**Labels:** `P1`, `epic`, `testing`

### Description

Taxonomy, traceability spec → vector → test, parser fuzzing, Verilog golden tests where feasible.

### Tasks

- [ ] `tests/TEST_TAXONOMY.md`, `tests/COVERAGE_MAP.md` (or under `docs/` if preferred).
- [ ] `cargo-fuzz` (or equivalent) + nightly budget.
- [ ] Verilog simulation golden workflow doc.

### Acceptance criteria

- Every **stable** spec has a traceable test or explicit gap entry.

---

## Issue: [EPIC-07] CI/CD hardening

**Labels:** `P1`, `epic`, `ci`

### Description

Fast PR lane vs nightly heavy jobs; release artifacts; `docs/CI_POLICY.md`.

### Tasks

- [ ] Split workflows without duplicating failing logic.
- [ ] Document required checks for `master`.

### Acceptance criteria

- CONTRIBUTING points to CI expectations.

---

## Issue: [EPIC-08] Documentation for multiple audiences

**Labels:** `P2`, `epic`, `documentation`

### Description

Entry points for researchers / compiler / hardware engineers; external audit pack.

### Tasks

- [ ] `FOR_RESEARCHERS.md`, `FOR_COMPILER_ENGINEERS.md`, `FOR_HARDWARE_ENGINEERS.md` (or merged sections).
- [ ] `EXTERNAL_AUDIT_PACK.md` + `docs/LIMITATIONS.md` if not redundant with [`STATE_OF_THE_PROJECT.md`](../../STATE_OF_THE_PROJECT.md).
- [ ] Diagram pack under `docs/diagrams/`.

### Acceptance criteria

- One-hour honest review path for a skeptical PL reviewer.

---

## Issue: [EPIC-09] Repository hygiene

**Labels:** `P0`, `epic`, `hygiene`

### Description

Professional OSS presentation without breaking constitution paths.

### Tasks

- [ ] Confirm `.env` **not** tracked; keep `.env.example` pattern if needed.
- [ ] Root **`LICENSE`** if README claims MIT (legal maintainer decision).
- [ ] Optional `REPO_MAP.md`; relocate non-essential root files only with ADR / issue.

### Acceptance criteria

- No secrets; governance files discoverable.

---

*End of templates.*
