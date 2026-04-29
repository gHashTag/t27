# Current Work — Trinity t27

[PHI Loop CI](https://github.com/gHashTag/t27/actions/workflows/phi-loop-ci.yml)
[NOW sync gate](https://github.com/gHashTag/t27/actions/workflows/now-sync-gate.yml)
[NOW document](https://github.com/gHashTag/t27/blob/master/docs/NOW.md)
[Queen health](https://github.com/gHashTag/t27/blob/master/.trinity/state/queen-health.json)

**Last updated:** 2026-04-30
**Active:** FFI bug fixes (#545-#549), tri igla CLI (#541), API completeness

---

> *"A specification without tests is a lie told in the future tense."*
> — `SOUL.md`

**Sync gates:** `.githooks/pre-commit` and **phi-loop CI** use `**./scripts/tri check-now`**. The gate compares **calendar date `YYYY-MM-DD`** on the **Last updated** line to **your machine's local date** when you run `tri` — so write **your wall-clock time** in the header, not UTC, unless you are in UTC.

---

## CLARA Submission Package

### Volume 1: Technical & Management Proposal
- **File:** `docs/clara/CLARA-PROPOSAL-TECHNICAL.md`
- **Status:** 1,702 words / 6.8 pages (under 10-page limit)
- **Sections:**
  1. AR-Based ML Approach (Trit-K3 isomorphism)
  2. Application Task Domain + SOA Benchmark
  3. Polynomial-Time Tractability Proofs (5 theorems)
  4. Demonstrated AR+ML Composition (84 Coq-verified theorems)
  5. Basis for Confidence (GF16 benchmarks)
  6. Metrics Coverage (CLARA requirements mapped)
  7. Schedule + Milestones (24-month delivery plan)
  8. Budget Summary
  9. Bibliography

### Volume 2: Cost Proposal
- **File:** `docs/clara/CLARA-COST-PROPOSAL.md`
- **Status:** $2,000,000 over 24 months
- **Breakdown:** Personnel ($1.2M), Equipment ($200K), Travel ($100K), Indirect ($500K)

### Supporting Evidence
- **File:** `docs/clara/CLARA-EVIDENCE-PACKAGE.md`
- **Content:** Formal proofs, numerical evidence, spec coverage, explainability evidence

### Demo Verification
- **Script:** `scripts/clara/demo.sh`
- **Status:** 20/20 tests PASSED

---

## CLARA Requirements Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| AR in guts of ML (FAQ 21) | done | K3 logic gates replace ReLU |
| <=10 step proof traces | done | MAX_STEPS=10 |
| Polynomial guarantees | done | Theorems 1-5 |
| >=2 AR kinds | done | Logic, ASP, Classical |
| >=2 ML kinds | done | Neural, Bayesian, RL |
| Apache 2.0 | done | All file headers |

---

## Specification Status

### Sealed artifacts

| Artifact             | Count / version                        | Last ring  | Verdict                              |
| -------------------- | -------------------------------------- | ---------- | ------------------------------------ |
| `.t27` specs         | 43+ files                              | Ring 43    | 43/43 parse PASS                     |
| `gen/zig/`           | 52+ files                              | Ring 43    | generated, compile-checked           |
| `conformance/` JSON  | 62+ files                              | Ring 44    | schema v1                            |
| `stage0/FROZEN_HASH` | SHA-256 of `bootstrap/src/compiler.rs` | genesis    | immutable *(if present in checkout)* |
| Experience log       | 45+ entries                            | Ring 45    | all `verdict: clean`                 |
| Queen health         | 1.0 / GREEN                            | 2026-04-05 | 17/17 domains                        |

### Critical open gap

```
bootstrap/src/compiler.rs  --- parse / gen -->  AST / emit
                                                    |
                         CI E2E not yet proven:     |
                         seed.t27 -> t27c gen -> zig test -> GREEN
                                                    |
                                              gen/zig/*.zig  (from t27c, not hand-written)
```

**The Rust bootstrap** (`t27c parse`, `t27c gen`, `t27c compile`, `t27c suite`) **exists**.
**The closed loop** `seed.t27 -> t27c gen -> output.zig -> zig test -> GREEN` has **not yet been demonstrated end-to-end in CI** as a **single advertised pipeline**.
Treat that as the **highest-leverage** gap before Phase 3 (Brain) work is **evidence-grade**.
**Track in issue:** [#150](https://github.com/gHashTag/t27/issues/150)

---

## Invariant law (never changes)

| Law                  | Statement                                                                                                                                                  | Enforcement                                                                                                         |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **ISSUE-GATE**       | No code merged without `Closes #N`                                                                                                                         | `.github/workflows/issue-gate.yml`                                                                                  |
| **NO-HAND-EDIT-GEN** | Files under `gen/` are generated; edit the `.t27` spec instead                                                                                             | `./bootstrap/target/release/t27c validate-gen-headers --repo-root .` (or `./scripts/tri` wrapper)                   |
| **SOUL-ASCII**       | All `.t27` / `.zig` / `.v` / `.c` source -- ASCII-only identifiers & comments                                                                              | `SOUL.md`, ADR-004                                                                                                  |
| **TDD-MANDATE**      | Every `.t27` spec must contain `test` / `invariant` / `bench`                                                                                              | Ring 037 / [#132](https://github.com/gHashTag/t27/issues/132)                                                       |
| **PHI-IDENTITY**     | **K2 core:** phi^2 = phi + 1 on R; **consequence** phi^2+phi^-2=3; **IEEE `f64`** checks use **tolerance** (not exact equality)                            | `specs/math/constants.t27`                                                                                          |
| **TRINITY-SACRED**   | `conformance/FORMAT-SPEC-001.json` + `specs/numeric/gf16.t27` are the numeric ceiling                                                                      | SSOT: never forked                                                                                                  |

---

## Sync gates and tooling

| Gate                | Trigger      | Checks                                    | Status *(verify in Actions)*                                        |
| ------------------- | ------------ | ----------------------------------------- | ------------------------------------------------------------------- |
| `pre-commit`        | local commit | `tri check-now`; `NOW.md` date            | active if hooks installed                                           |
| `issue-gate.yml`    | PR           | `Closes #N`                               | see badge / Actions                                                 |
| `phi-loop-ci.yml`   | push         | parse / gen / conformance (see workflow)  | **E2E gap** -- [#150](https://github.com/gHashTag/t27/issues/150)   |
| `now-sync-gate.yml` | push         | `NOW.md` freshness window                 | see badge / Actions                                                 |
| **Conformance**     | CI / local   | `t27c validate-conformance --repo-root .` | run locally or in CI                                                |
| **Gen headers**     | CI / local   | `t27c validate-gen-headers --repo-root .` | run locally or in CI                                                |

---

**Canonical URL (SSOT for humans + agents):**
`https://github.com/gHashTag/t27/blob/master/docs/NOW.md`

---

**Recent fixes (2026-04-30):**
- FFI: GF16 encode round-to-nearest-even + overflow to +Inf (Closes #545, #546, #547)
- FFI: GF4/GF8/GF12/GF20/GF24 encode/decode with round-to-nearest-even (Closes #549)
- FFI: GF32 [1:13:18] Lucas L6 layout verified (Closes #548)
- CLI: tri igla search/list/gate/check/triplet subcommands (Closes #541)

---

*Living documentation corpus | Last updated must include calendar date YYYY-MM-DD (for tri check-now).*

**phi^2 + phi^-2 = 3 | TRINITY**
