<<<<<<< Updated upstream
# Current Work — Trinity t27
=======
[PHI Loop CI](https://github.com/gHashTag/t27/actions/workflows/phi-loop-ci.yml)
[NOW sync gate](https://github.com/gHashTag/t27/actions/workflows/now-sync-gate.yml)
[NOW document](https://github.com/gHashTag/t27/blob/master/docs/NOW.md)
[Queen health](https://github.com/gHashTag/t27/blob/master/.trinity/state/queen-health.json)
>>>>>>> Stashed changes

**Last updated:** 2026-04-14
**Active:** CI fixes (PR #409) — all workflow YAML fixed, FPGA build passing + DARPA CLARA PA-25-07-02 Submission Package

## Active Work

**CI Fixes** — All GitHub Actions CI workflows passing (PR #409)
- Workflow YAML syntax errors fixed
- Generated files added for FPGA build
- L1 and L7 compliance met

<<<<<<< Updated upstream
**DARPA CLARA Submission** — Complete submission package for April 17, 2026 deadline
=======
> *"A specification without tests is a lie told in the future tense."*  
> — `SOUL.md`

**Sync gates:** `.githooks/pre-commit` and **phi-loop CI** use `**./scripts/tri check-now`**. The gate compares **calendar date `YYYY-MM-DD`** on the **Last updated** line to **your machine’s local date** when you run `tri` — so write **your wall-clock time** in the header, not UTC, unless you are in UTC.
>>>>>>> Stashed changes

---

## CLARA Submission Package

### Volume 1: Technical & Management Proposal
- **File:** `docs/clara/CLARA-PROPOSAL-TECHNICAL.md`
- **Status:** 1,702 words ≈ 6.8 pages (under 10-page limit)
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

<<<<<<< Updated upstream
### Demo Verification
- **Script:** `scripts/clara/demo.sh`
- **Status:** 20/20 tests PASSED

---

## CLARA Requirements Compliance
=======
### § 1.1  Agent handoff — talk to the next agent / Queen via NOW

**Canonical URL (SSOT for humans + agents):**  
`https://github.com/gHashTag/t27/blob/master/docs/NOW.md`

When you **complete a non-trivial task** (code, specs, CI, seals, architecture docs), **update `NOW.md` before you stop**:

1. Refresh `**Last updated:`** (calendar `**YYYY-MM-DD**` must match **today** for `./scripts/tri check-now`; keep **local wall time** + **RFC3339 with offset** as in the header template).
2. Fix **§ 3** state, **critical gap**, **links**, or **milestone notes** so the **next agent** reads **current truth**, not yesterday’s story.
3. **Commit `docs/NOW.md` in the same PR** as the work (or amend), per Ring 033 / [#141](https://github.com/gHashTag/t27/issues/141).

Skipping this is a **failed handoff** — the fleet coordinates here, not only in issues.

**Recent methodology docs (kernel + experience + formal + science/ops):**  
`[KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md)` · `[KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md](KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md)` · `[SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md](SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md)` · `[RESEARCH_WRITING_T27.md](RESEARCH_WRITING_T27.md)` · `[TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)` · `[T27_KERNEL_FORMAL_COQ.md](T27_KERNEL_FORMAL_COQ.md)` · `[COMPILER_VERIFICATION_STANDARDS.md](COMPILER_VERIFICATION_STANDARDS.md)` (deep map + ring plan; index `[COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md)`; RU impact `[COMPILER_VERIFICATION_IMPACT_RU.md](COMPILER_VERIFICATION_IMPACT_RU.md)`; TOR/TVP `[qualification/](qualification/)`; template `[templates/TOOL_QUALIFICATION_SKETCH_DO330.md](templates/TOOL_QUALIFICATION_SKETCH_DO330.md)`) · repo `[coq/](../coq/)` (Rocq/Coq scaffold; workflow `.github/workflows/coq-kernel.yml`)

---

## § 2  Invariant law (never changes)


| Law                  | Statement                                                                                                                                                  | Enforcement                                                                                                         |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **ISSUE-GATE**       | No code merged without `Closes #N`                                                                                                                         | `.github/workflows/issue-gate.yml`                                                                                  |
| **NO-HAND-EDIT-GEN** | Files under `gen/` are generated; edit the `.t27` spec instead                                                                                             | `./bootstrap/target/release/t27c validate-gen-headers --repo-root .` (or `./scripts/tri` wrapper)                   |
| **SOUL-ASCII**       | All `.t27` / `.zig` / `.v` / `.c` source — ASCII-only identifiers & comments                                                                               | `SOUL.md`, ADR-004                                                                                                  |
| **TDD-MANDATE**      | Every `.t27` spec must contain `test` / `invariant` / `bench`                                                                                              | Ring 037 / [#132](https://github.com/gHashTag/t27/issues/132)                                                       |
| **PHI-IDENTITY**     | **K2 core:** \varphi^2 = \varphi + 1 on \mathbb{R}; **consequence** \varphi^2+\varphi^{-2}=3; **IEEE `f64`** checks use **tolerance** (not exact equality) | `[NUMERIC-CORE-PALETTE-REGISTRY.md](nona-02-organism/NUMERIC-CORE-PALETTE-REGISTRY.md)`, `specs/math/constants.t27` |
| **TRINITY-SACRED**   | `conformance/FORMAT-SPEC-001.json` + `specs/numeric/gf16.t27` are the numeric ceiling                                                                      | SSOT: never forked                                                                                                  |
>>>>>>> Stashed changes

| Requirement | Status | Evidence |
|-------------|--------|----------|
| AR in guts of ML (FAQ 21) | ✅ | K3 logic gates replace ReLU |
| ≤10 step proof traces | ✅ | MAX_STEPS=10 |
| Polynomial guarantees | ✅ | Theorems 1-5 |
| ≥2 AR kinds | ✅ | Logic, ASP, Classical |
| ≥2 ML kinds | ✅ | Neural, Bayesian, RL |
| Apache 2.0 | ✅ | All file headers |

---

## Specification Status

<<<<<<< Updated upstream
| Category | Specs | Parse Status |
|----------|-------|--------------|
| AR (Automated Reasoning) | 7 | 7/7 PASS |
| NN (Neural Networks) | 2 | 2/2 PASS |
| VSA | 1 | 1/1 PASS |
| **Total** | **10** | **10/10 PASS** |
=======
### 3.1  Sealed artifacts


| Artifact             | Count / version                        | Last ring  | Verdict                              |
| -------------------- | -------------------------------------- | ---------- | ------------------------------------ |
| `.t27` specs         | 43 files *(ring narrative)*            | Ring 43    | 43/43 parse PASS                     |
| `gen/zig/`           | 52 files *(ring narrative)*            | Ring 43    | generated, compile-checked           |
| `conformance/` JSON  | 62 files *(ring narrative)*            | Ring 44    | schema v1                            |
| `stage0/FROZEN_HASH` | SHA-256 of `bootstrap/src/compiler.rs` | genesis    | immutable *(if present in checkout)* |
| Experience log       | 45 entries *(ring narrative)*          | Ring 45    | all `verdict: clean`                 |
| Queen health         | 1.0 / GREEN                            | 2026-04-05 | 17/17 domains                        |


***Re-scan before every commit (do not treat stale counts as SSOT):***

```bash
find specs -name "*.t27" | wc -l
find gen/zig -name "*.zig" | wc -l
find conformance -name "*.json" | wc -l
```

The **table counts** above are *ring narrative* snapshots; refresh them when you seal a ring.

### 3.2  Critical open gap

```
bootstrap/src/compiler.rs  ─── parse / gen ──→  AST / emit
                                                    │
                         CI E2E not yet proven:     │
                         seed.t27 → t27c gen → zig test → GREEN
                                                    │
                                              gen/zig/*.zig  (from t27c, not hand-written)
```

**The Rust bootstrap** (`t27c parse`, `t27c gen`, `t27c compile`, `t27c suite`) **exists**.  
**The closed loop** `seed.t27 → t27c gen → output.zig → zig test → GREEN` has **not yet been demonstrated end-to-end in CI** as a **single advertised pipeline**.  
Treat that as the **highest-leverage** gap before Phase 3 (Brain) work is **evidence-grade**.  
**Track in issue:** [#150](https://github.com/gHashTag/t27/issues/150) — every PR that implements this loop must use `**Closes #150`** (or a split child issue) per **ISSUE-GATE**.

**TV reference (`[qualification/TVP.md](qualification/TVP.md)`):** **TV-01** (`tri test` / suite on golden snapshot) — **PENDING** full E2E closure · **TV-02** (regen + blessed hash of `gen/`) — **PENDING** until the same pipeline is wired. See TVP §3 note.

**K2 fast path (binary64):** For the IEEE literal of \varphi, `**fl(φ·φ)`** and `**fl(φ+1.0)**` are **bit-identical** (`0x4004F1BBCDCBFA54`). So `**phi_identity_contract`** in `coq/Kernel/PhiFloat.v` is `**Rabs(0) < phi_tolerance**` (trivial residual). Mantissa / exponent for Flocq: `**7286977268806824**`, exp `**-52**` — cross-check with `**scripts/validate_phi_f64.py**`. Spec: `[PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md](nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md)` · task anchor: `[PHASE_B_FLOCQ_AGENT_TASK.md](nona-03-manifest/PHASE_B_FLOCQ_AGENT_TASK.md)`.

**Optional formal track:** `[coq/](../coq/)` + `[T27_KERNEL_FORMAL_COQ.md](T27_KERNEL_FORMAL_COQ.md)` — Rocq/Coq scaffold for **K1–K4** (not K5/K6); CI `.github/workflows/coq-kernel.yml` when `**coq/**`** changes.  
**K2 / PHI-IDENTITY (summary):** `Kernel/Phi.v` — `Coq.Reals` (`**phi_squared_identity`**, `**phi_tolerance**`). `Kernel/PhiFloat.v` — Flocq `**binary64**`, `**phi_identity_contract**`. Balanced ternary / radix economy context: [#138](https://github.com/gHashTag/t27/issues/138), [#142](https://github.com/gHashTag/t27/issues/142).  
**Certification / evidence vocabulary:** `[COMPILER_VERIFICATION_STANDARDS.md](COMPILER_VERIFICATION_STANDARDS.md)` — **DO-178C / DO-330 / DO-333**, ISO 26262 (TCL), IEC 61508 (T1–T3), EN 50716, ECSS-Q-ST-80C, IEC 62304, IEEE 1012, NIST SSDF, CompCert/CakeML/Alive2/Flocq, TVCP **TV-01–TV-07**, phased plan. Quick index: `[COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md)`. Draft **TOR/TVP:** `[qualification/TOR.md](qualification/TOR.md)`, `[qualification/TVP.md](qualification/TVP.md)`.

### 3.3  Compiler verification — impact digest (trust in `t27c`)

**Question the standards pack answers:** how we **justify trust** in `**t27c`** as a code generator (and in `**coqc**` as proof-checking tooling) using the same vocabulary regulators use (tool qualification, V&V, formal methods).

**Why it matters for T27**

- **DO-330 / ISO 26262 / IEC 61508** all force the same discipline: if a tool **writes** product code or **replaces** verification, its failures must be **controlled** with evidence (TOR/TVP/TVCP/TVR/TAS in aviation-shaped programs).  
- **DO-178C** aligns with repo law: `**TDD-MANDATE`** ≈ requirements-based testing mindset; `**ISSUE-GATE**` ≈ traceability of change to tracked work.  
- **DO-333** is the slot for `**coq/`** (theorem proving); **K2** is proved on `**Reals`** in `Phi.v`; `**PhiFloat.v**` gives the `**f64**` Flocq model + `**phi_identity_contract**` (computational bridge; deeper error lemmas → later ring).  
- **IEEE 1012-style V&V planning** implies generator assurance should be **commensurate** with the integrity of the software the generator affects — `**NO-HAND-EDIT-GEN`** enforces SSOT on `**.t27**`, not hand patches in `**gen/**`.  
- **NIST SSDF** aligns with **pinned toolchains**, `**FROZEN_HASH`**, and append-only **experience** logs.

**Immediate blocker (unchanged):** until `**seed.t27 → t27c gen → zig test → GREEN`** runs as **one advertised CI job**, end-to-end “we can show the compiler pipeline works” remains **weaker than** the standards narrative we are writing. That job is **Phase 1 / NOW §5 step 1.5** — **[#150](https://github.com/gHashTag/t27/issues/150)**.

**Russian full narrative (impact per section):** `[COMPILER_VERIFICATION_IMPACT_RU.md](COMPILER_VERIFICATION_IMPACT_RU.md)` — allowlisted Cyrillic companion; **English SSOT** remains `[COMPILER_VERIFICATION_STANDARDS.md](COMPILER_VERIFICATION_STANDARDS.md)`.
>>>>>>> Stashed changes

---

## Submission Deadline

<<<<<<< Updated upstream
**April 17, 2026, 16:00 ET**
**Submission Bundle:** `/tmp/clara-submission/`

---

**φ² + 1/φ² = 3 | TRINITY**
=======
**[EPOCH-01-HARDEN](https://github.com/gHashTag/t27/milestone/1)** — Rings 032–049


| Issue                                              | Ring | Domain       | Title                                                |
| -------------------------------------------------- | ---- | ------------ | ---------------------------------------------------- |
| [#127](https://github.com/gHashTag/t27/issues/127) | 032  | Tooling      | `TASK.md` + iteration schema                         |
| [#128](https://github.com/gHashTag/t27/issues/128) | 033  | CI           | Issue-gate enforcement — every PR `Closes #N`        |
| [#129](https://github.com/gHashTag/t27/issues/129) | 034  | Numerics     | GoldenFloat benchmark spec (NMSE vs bfloat16)        |
| [#130](https://github.com/gHashTag/t27/issues/130) | 035  | Architecture | `TECHNOLOGY-TREE.md` — ring DAG to 999               |
| [#131](https://github.com/gHashTag/t27/issues/131) | 036  | CI           | Seal coverage — block PRs with missing SHA-256       |
| [#132](https://github.com/gHashTag/t27/issues/132) | 037  | Language     | SOUL.md parser enforcement                           |
| [#133](https://github.com/gHashTag/t27/issues/133) | 038  | Conformance  | Conformance vector schema v2                         |
| [#134](https://github.com/gHashTag/t27/issues/134) | 039  | Science      | CLARA / DARPA TA1–TA2 submission checklist           |
| [#135](https://github.com/gHashTag/t27/issues/135) | 040  | Agents       | `AGENTS_ALPHABET.md` — 27 agent definitions          |
| [#138](https://github.com/gHashTag/t27/issues/138) | 043  | Math         | Balanced ternary addition formal spec                |
| [#139](https://github.com/gHashTag/t27/issues/139) | 044  | Protocol     | PHI LOOP contract v2 + TOXIC rollback                |
| [#140](https://github.com/gHashTag/t27/issues/140) | 045  | ISA          | 27 Coptic register invariants                        |
| [#142](https://github.com/gHashTag/t27/issues/142) | 046  | Math         | Radix economy — base-3 optimality proof              |
| [#143](https://github.com/gHashTag/t27/issues/143) | 047  | Math         | K3 logic truth table — 27-entry isomorphism          |
| [#144](https://github.com/gHashTag/t27/issues/144) | 048  | VSA          | Trit-space bind/unbind formal spec                   |
| [#145](https://github.com/gHashTag/t27/issues/145) | 049  | Physics      | Sacred physics hard-tolerance conformance            |
| [#150](https://github.com/gHashTag/t27/issues/150) | —    | CI           | E2E CI: `seed.t27` → `t27c gen` → `zig test` → GREEN |


*Confirm issue titles with `gh issue view` if links drift.*

**Also:** `[RING_BACKLOG_047_063.md](RING_BACKLOG_047_063.md)` · `[coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md](coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md)` · `[KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md](KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md)` · `[SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md](SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md)` · `[RESEARCH_WRITING_T27.md](RESEARCH_WRITING_T27.md)` · anchor [#141](https://github.com/gHashTag/t27/issues/141)

---

## § 5  Sequential integration plan: Seed → Tests → Queen

**Rule:** Complete each phase before expanding the next.  
**Every PR must contain** `Closes #N` (Ring 033 / [#128](https://github.com/gHashTag/t27/issues/128)).  
**No code without an issue.**

```
SEED (bootstrap/Rust)
  │  Phase 1 — Law & SSOT
  ▼
STEM (conformance vectors)
  │  Phase 2 — Test execution
  ▼
BRANCHES (Ring 050+ science tests)
  │  Phase 3 — Math/physics audit
  ▼
CROWN (Queen brain & automation)
     Phase 4 — Orchestration
```

### Phase 1 — Seed: Law + SSOT + gates *(active now)*


| Step | Issue                                              | Action                                                     | Acceptance criterion                                            |
| ---- | -------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------- |
| 1.1  | [#128](https://github.com/gHashTag/t27/issues/128) | Enable issue-gate CI                                       | All PRs blocked without `Closes #N`; zero bypass                |
| 1.2  | [#132](https://github.com/gHashTag/t27/issues/132) | Parser enforces SOUL.md                                    | Spec without `test`/`invariant`/`bench` → error (when enforced) |
| 1.3  | [#127](https://github.com/gHashTag/t27/issues/127) | Canonicalise `TASK.md` + iteration schema                  | `tri check-now` passes on clean repo                            |
| 1.4  | —                                                  | Verify `FORMAT-SPEC-001.json` + `gf16.t27` as numeric SSOT | Numeric PRs link to these                                       |
| 1.5  | [#150](https://github.com/gHashTag/t27/issues/150) | Document / CI **seed → gen → zig test**                    | Minimal golden spec path green in CI; PRs `**Closes #150*`*     |


### Phase 2 — Stem: Conformance + benchmarks + seals *(next)*


| Step | Issue                                              | Action                       | Acceptance criterion                                                                                     |
| ---- | -------------------------------------------------- | ---------------------------- | -------------------------------------------------------------------------------------------------------- |
| 2.1  | [#133](https://github.com/gHashTag/t27/issues/133) | Conformance vector schema v2 | `phi_distance` + `verdict` in `gf*_vectors.json` where applicable                                        |
| 2.2  | [#129](https://github.com/gHashTag/t27/issues/129) | GoldenFloat NMSE benchmark   | `gf_family_bench.json` semantics documented                                                              |
| 2.3  | [#131](https://github.com/gHashTag/t27/issues/131) | Seal coverage CI             | PRs touching `specs/` need seal discipline                                                               |
| 2.4  | —                                                  | GF16 vectors grow            | e.g. 10 → 33+ in `gf16_vectors.json`                                                                     |
| 2.5  | —                                                  | Numeric debt sprint          | `[NUMERIC-GF16-DEBT-INVENTORY.md](nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md)` — math → nn/vsa → ar |


**Numeric palette:** `[NUMERIC-STANDARD-001.md](nona-02-organism/NUMERIC-STANDARD-001.md)` · `[NUMERIC-GF16-CANONICAL-PICTURE.md](nona-02-organism/NUMERIC-GF16-CANONICAL-PICTURE.md)` · `[NUMERIC-WHY-NOT-GF16-EVERYWHERE.md](nona-02-organism/NUMERIC-WHY-NOT-GF16-EVERYWHERE.md)` · `[NUMERIC-CORE-PALETTE-REGISTRY.md](nona-02-organism/NUMERIC-CORE-PALETTE-REGISTRY.md)`

### Phase 3 — Branches: Ring 050+ science tests *(upcoming)*


| Ring | Issue | Domain          | Key deliverable                     |
| ---- | ----- | --------------- | ----------------------------------- |
| 050  | open  | Math/physics    | `specs/test_framework/` per charter |
| 051  | open  | Physics (P)     | Sacred physics claim audit          |
| 052  | open  | Conformance (F) | Property-test template              |
| 053  | open  | Verilog (V)     | Bench harness                       |
| 054  | open  | Graph (G)       | Graph drift detection               |


**Charter:** `[T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md](nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md)`  
**Claims:** `[RESEARCH_CLAIMS.md](nona-03-manifest/RESEARCH_CLAIMS.md)` · `[CLAIM_TIERS.md](nona-03-manifest/CLAIM_TIERS.md)`

### Phase 4 — Crown: Metrics → brain seals → Queen *(future)*


| Step | Ring | Action                     | Acceptance criterion                                                                                      |
| ---- | ---- | -------------------------- | --------------------------------------------------------------------------------------------------------- |
| 4.1  | 056  | Verdict export JSON schema | Single schema for Queen tooling                                                                           |
| 4.2  | —    | Brain seal refresh         | `.trinity/seals/brain-*.json` from pipeline                                                               |
| 4.3  | 047  | Lotus phase automation     | `.trinity/queen-brain/summaries/` when job exists                                                         |
| 4.4  | —    | META dashboard             | [#126](https://github.com/gHashTag/t27/issues/126) · `[PINNED_ROADMAP_ISSUE.md](PINNED_ROADMAP_ISSUE.md)` |


**Brain artifacts:** `.trinity/seals/brain-*.json` · `.trinity/state/queen-health.json` · `.trinity/experience/clara_track1.jsonl`

---

## § 6  Matryoshka layer map


| Layer  | Name               | Key files                                                                | Integration phase |
| ------ | ------------------ | ------------------------------------------------------------------------ | ----------------- |
| **L0** | **Seed**           | `bootstrap/src/compiler.rs`; `stage0/FROZEN_HASH` *if shipped*           | genesis           |
| **L1** | **Bootstrap**      | `bootstrap/src/main.rs`, `bootstrap/main.zig`                            | Phase 1           |
| **L2** | **Base types**     | `specs/base/types.t27`, `specs/base/ops.t27`                             | Phase 1           |
| **L3** | **Numerics**       | `specs/numeric/gf*.t27`, `specs/numeric/tf3.t27`                         | Phase 2           |
| **L4** | **Math / physics** | `specs/math/constants.t27`, `specs/math/sacred_physics.t27`              | Phase 3           |
| **L5** | **Compiler**       | `specs/compiler/`, `gen/zig/compiler/`                                   | Phase 1–2         |
| **L6** | **Hardware**       | `specs/fpga/`, `specs/isa/registers.t27`                                 | Phase 3           |
| **L7** | **Queen brain**    | `specs/queen/lotus.t27`, `specs/nn/hslm.t27`, `specs/vsa/`, `specs/ar/`* | Phase 4           |


---

## § 7  Sync gates and tooling


| Gate                | Trigger      | Checks                                    | Status *(verify in Actions)*                                        |
| ------------------- | ------------ | ----------------------------------------- | ------------------------------------------------------------------- |
| `pre-commit`        | local commit | `tri check-now`; `NOW.md` date            | active if hooks installed                                           |
| `issue-gate.yml`    | PR           | `Closes #N`                               | see badge / Actions                                                 |
| `phi-loop-ci.yml`   | push         | parse / gen / conformance (see workflow)  | **⚠️ E2E gap** — [#150](https://github.com/gHashTag/t27/issues/150) |
| `now-sync-gate.yml` | push         | `NOW.md` freshness window                 | see badge / Actions                                                 |
| **Conformance**     | CI / local   | `t27c validate-conformance --repo-root .` | run locally or in CI                                                |
| **Gen headers**     | CI / local   | `t27c validate-gen-headers --repo-root .` | run locally or in CI                                                |


**Agent sync:** `.trinity/state/github-sync.json`  
**Hooks:** `bash scripts/setup-git-hooks.sh`  
**Manual:** `./scripts/tri check-now`

---

## § 8  Document map


| Topic                      | Document                                                                                                                                                                          |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Constitution v1.2          | `[T27-CONSTITUTION.md](T27-CONSTITUTION.md)`                                                                                                                                      |
| Ring log                   | `.trinity/experience/clara_track1.jsonl`                                                                                                                                          |
| Queen health               | `.trinity/state/queen-health.json`                                                                                                                                                |
| Rolling integration detail | `[ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md](coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md)`                                                                             |
| Numeric SSOT               | `conformance/FORMAT-SPEC-001.json` + `[NUMERIC-STANDARD-001.md](nona-02-organism/NUMERIC-STANDARD-001.md)`                                                                        |
| Claims registry            | `[RESEARCH_CLAIMS.md](nona-03-manifest/RESEARCH_CLAIMS.md)`                                                                                                                       |
| Math/physics test charter  | `[T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md](nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md)`                                                                             |
| Axiom/theorem format       | `[T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md](nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md)`                                                                       |
| Publications pipeline      | `[PUBLICATION_PIPELINE.md](PUBLICATION_PIPELINE.md)`                                                                                                                              |
| Compiler verification (EN) | `[COMPILER_VERIFICATION_STANDARDS.md](COMPILER_VERIFICATION_STANDARDS.md)` · `[COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md)` |
| Compiler verification (RU) | `[COMPILER_VERIFICATION_IMPACT_RU.md](COMPILER_VERIFICATION_IMPACT_RU.md)` (allowlisted; see ADR-004)                                                                             |
| PHI-IDENTITY Flocq bridge  | `[PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md](nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md)`                                                                                         |
| Phase B Flocq task anchor  | `[PHASE_B_FLOCQ_AGENT_TASK.md](nona-03-manifest/PHASE_B_FLOCQ_AGENT_TASK.md)`                                                                                                     |
| φ / f64 validation script  | `[scripts/validate_phi_f64.py](../scripts/validate_phi_f64.py)`                                                                                                                   |
| Roadmap umbrella           | [#126](https://github.com/gHashTag/t27/issues/126)                                                                                                                                |


---

## § 9  Next actions (48 h)

**Priority:** Close **E2E** `seed.t27 → t27c gen → zig test → GREEN` in **phi-loop CI** — **[#150](https://github.com/gHashTag/t27/issues/150)** (see **§3.2–3.3**, **§5 Phase 1 step 1.5**). Everything else is secondary until that loop is green.

```bash
# 0. NOW gate — run FIRST before any commit (otherwise push / hooks may fail)
./scripts/tri check-now

# 1. E2E CI issue (created — link PRs with Closes #150)
# gh issue view 150

# 2. Milestone hygiene (needs gh auth)
# gh issue edit 127 128 129 130 131 132 133 --milestone "EPOCH-01-HARDEN"

# 3. Bootstrap + suite
cd bootstrap && cargo build --release
./target/release/t27c validate-conformance --repo-root ..
./target/release/t27c validate-gen-headers --repo-root ..
./target/release/t27c suite --repo-root ..

# 4. Optional: compiler hash (if stage0/FROZEN_HASH exists in your tree)
# shasum -a 256 bootstrap/src/compiler.rs

# 5. Experience log — only after a real run
# echo '{"ring":46,"task":"…","verdict":"clean","timestamp":"2026-04-06T12:00:00Z"}' >> .trinity/experience/clara_track1.jsonl

# 6. gh issue comment 126 --body "…"
```

---

*Living documentation corpus · `[T27-CONSTITUTION.md](T27-CONSTITUTION.md)` v1.2, Article DOCS-TREE · **Last updated** must include **calendar date** `YYYY-MM-DD` (for `tri check-now`). Prefer **human-readable local wall time** plus optional **RFC3339 with offset** (e.g. `2026-04-06T18:45:00+07:00`) so tools can echo it — do not require UTC `Z` unless you work in UTC.*
>>>>>>> Stashed changes
