# NOW — Rolling integration snapshot

**Last updated:** 2026-04-06 — Monday, 06 April 2026 · 18:45 local time (UTC+07) · RFC3339 2026-04-06T18:45:00+07:00

**Document class:** Operational focus document  
**Revision:** 2026-04-06 — Monday, 06 April 2026 · 18:45 local (UTC+07) · Ring 45 (narrative seal — verify in `.trinity/` + CI)  
**Status:** ACTIVE — replace body on every ring boundary  
**Queen health:** GREEN / 1.0 (all 17 domains; sealed 2026-04-05T12:00Z) — *verify* `.trinity/state/queen-health.json`  
**Canonical URL:** `https://github.com/gHashTag/t27/blob/master/docs/NOW.md`

> *"A specification without tests is a lie told in the future tense."*  
> — `SOUL.md`

**Sync gates:** `.githooks/pre-commit` and **phi-loop CI** use **`./scripts/tri check-now`**. The gate compares **calendar date `YYYY-MM-DD`** on the **Last updated** line to **your machine’s local date** when you run `tri` — so write **your wall-clock time** in the header, not UTC, unless you are in UTC.

---

## § 1  Purpose and scope

This document is the **single rolling snapshot** of what is being worked on *right now*.  
It is **not** a roadmap (→ `[docs/ROADMAP.md](ROADMAP.md)`, issue [#126](https://github.com/gHashTag/t27/issues/126)),  
**not** a ring log (→ `.trinity/experience/clara_track1.jsonl`),  
and **not** a design specification (→ `specs/`).

**Replace this file’s body at every ring boundary.**  
Stale content here is a quality defect — treat it as a failing test.

---

## § 2  Invariant law (never changes)


| Law                  | Statement                                                                                           | Enforcement                                                                                                         |
| -------------------- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **ISSUE-GATE**       | No code merged without `Closes #N`                                                                  | `.github/workflows/issue-gate.yml`                                                                                  |
| **NO-HAND-EDIT-GEN** | Files under `gen/` are generated; edit the `.t27` spec instead                                      | `./bootstrap/target/release/t27c validate-gen-headers --repo-root .` (or `./scripts/tri` wrapper)                   |
| **SOUL-ASCII**       | All `.t27` / `.zig` / `.v` / `.c` source — ASCII-only identifiers & comments                        | `SOUL.md`, ADR-004                                                                                                  |
| **TDD-MANDATE**      | Every `.t27` spec must contain `test` / `invariant` / `bench`                                       | Ring 037 / [#132](https://github.com/gHashTag/t27/issues/132)                                                       |
| **PHI-IDENTITY**     | \varphi^2 + \varphi^{-2} = 3 — **algebraic** truth; **IEEE `f64`** checks use **tolerance** in code | `[NUMERIC-CORE-PALETTE-REGISTRY.md](nona-02-organism/NUMERIC-CORE-PALETTE-REGISTRY.md)`, `specs/math/constants.t27` |
| **TRINITY-SACRED**   | `conformance/FORMAT-SPEC-001.json` + `specs/numeric/gf16.t27` are the numeric ceiling               | SSOT: never forked                                                                                                  |


---

## § 3  System state (as of Ring 45 · 2026-04-06)

### 3.1  Sealed artifacts


| Artifact             | Count / version                        | Last ring  | Verdict                              |
| -------------------- | -------------------------------------- | ---------- | ------------------------------------ |
| `.t27` specs         | 43 files *(ring narrative)*            | Ring 43    | 43/43 parse PASS                     |
| `gen/zig/`           | 52 files *(ring narrative)*            | Ring 43    | generated, compile-checked           |
| `conformance/` JSON  | 62 files *(ring narrative)*            | Ring 44    | schema v1                            |
| `stage0/FROZEN_HASH` | SHA-256 of `bootstrap/src/compiler.rs` | genesis    | immutable *(if present in checkout)* |
| Experience log       | 45 entries *(ring narrative)*          | Ring 45    | all `verdict: clean`                 |
| Queen health         | 1.0 / GREEN                            | 2026-04-05 | 17/17 domains                        |


***Workspace snapshot (re-scan at each seal):*** `specs/**/*.t27` ≈ **50**, `gen/zig/**/*.zig` ≈ **97**, `conformance/**/*.json` ≈ **65** — run `find`/`wc` before quoting in commits.

### 3.2  Critical open gap

```
bootstrap/src/compiler.rs  ─── parse / gen ──→  AST / emit
                                                    │
                         CI E2E not yet proven:     │
                         seed.t27 → t27c gen → zig test → GREEN
                                                    │
                                              gen/zig/*.zig  (from t27c, not hand-written)
```

**The Rust bootstrap** (`t27c parse`, `**t27c gen`**, `t27c compile`, `t27c suite`) **exists**.  
**The closed loop** `seed.t27 → t27c gen → output.zig → zig test → GREEN` has **not yet been demonstrated end-to-end in CI** as a **single advertised pipeline**.  
Treat that as the **highest-leverage** gap before Phase 3 (Brain) work is **evidence-grade**.

---

## § 4  Active GitHub milestone

**[EPOCH-01-HARDEN](https://github.com/gHashTag/t27/milestone/1)** — Rings 032–049


| Issue                                              | Ring | Domain       | Title                                          |
| -------------------------------------------------- | ---- | ------------ | ---------------------------------------------- |
| [#127](https://github.com/gHashTag/t27/issues/127) | 032  | Tooling      | `TASK.md` + iteration schema                   |
| [#128](https://github.com/gHashTag/t27/issues/128) | 033  | CI           | Issue-gate enforcement — every PR `Closes #N`  |
| [#129](https://github.com/gHashTag/t27/issues/129) | 034  | Numerics     | GoldenFloat benchmark spec (NMSE vs bfloat16)  |
| [#130](https://github.com/gHashTag/t27/issues/130) | 035  | Architecture | `TECHNOLOGY-TREE.md` — ring DAG to 999         |
| [#131](https://github.com/gHashTag/t27/issues/131) | 036  | CI           | Seal coverage — block PRs with missing SHA-256 |
| [#132](https://github.com/gHashTag/t27/issues/132) | 037  | Language     | SOUL.md parser enforcement                     |
| [#133](https://github.com/gHashTag/t27/issues/133) | 038  | Conformance  | Conformance vector schema v2                   |
| [#134](https://github.com/gHashTag/t27/issues/134) | 039  | Science      | CLARA / DARPA TA1–TA2 submission checklist     |
| [#135](https://github.com/gHashTag/t27/issues/135) | 040  | Agents       | `AGENTS_ALPHABET.md` — 27 agent definitions    |
| [#138](https://github.com/gHashTag/t27/issues/138) | 043  | Math         | Balanced ternary addition formal spec          |
| [#139](https://github.com/gHashTag/t27/issues/139) | 044  | Protocol     | PHI LOOP contract v2 + TOXIC rollback          |
| [#140](https://github.com/gHashTag/t27/issues/140) | 045  | ISA          | 27 Coptic register invariants                  |
| [#142](https://github.com/gHashTag/t27/issues/142) | 046  | Math         | Radix economy — base-3 optimality proof        |
| [#143](https://github.com/gHashTag/t27/issues/143) | 047  | Math         | K3 logic truth table — 27-entry isomorphism    |
| [#144](https://github.com/gHashTag/t27/issues/144) | 048  | VSA          | Trit-space bind/unbind formal spec             |
| [#145](https://github.com/gHashTag/t27/issues/145) | 049  | Physics      | Sacred physics hard-tolerance conformance      |


*Confirm issue titles with `gh issue view` if links drift.*

**Also:** `[docs/RING_BACKLOG_047_063.md](RING_BACKLOG_047_063.md)` · `[docs/coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md](coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md)` · anchor [#141](https://github.com/gHashTag/t27/issues/141)

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
| 1.5  | —                                                  | Document / CI **seed → gen → zig test**                    | Minimal golden spec path green in CI                            |


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


| Layer  | Name               | Key files                                                                   | Integration phase |
| ------ | ------------------ | --------------------------------------------------------------------------- | ----------------- |
| **L0** | **Seed**           | `bootstrap/src/compiler.rs`; `stage0/FROZEN_HASH` *if shipped*              | genesis           |
| **L1** | **Bootstrap**      | `bootstrap/src/main.rs`, `bootstrap/main.zig`                               | Phase 1           |
| **L2** | **Base types**     | `specs/base/types.t27`, `specs/base/ops.t27`                                | Phase 1           |
| **L3** | **Numerics**       | `specs/numeric/gf*.t27`, `specs/numeric/tf3.t27`                            | Phase 2           |
| **L4** | **Math / physics** | `specs/math/constants.t27`, `specs/math/sacred_physics.t27`                 | Phase 3           |
| **L5** | **Compiler**       | `specs/compiler/`**, `gen/zig/compiler/*`*                                  | Phase 1–2         |
| **L6** | **Hardware**       | `specs/fpga/`**, `specs/isa/registers.t27`                                  | Phase 3           |
| **L7** | **Queen brain**    | `specs/queen/lotus.t27`, `specs/nn/hslm.t27`, `specs/vsa/`**, `specs/ar/`** | Phase 4           |


---

## § 7  Sync gates and tooling


| Gate                | Trigger      | Checks                                    |
| ------------------- | ------------ | ----------------------------------------- |
| `pre-commit`        | local commit | `tri check-now`; `NOW.md` date            |
| `issue-gate.yml`    | PR           | `Closes #N`                               |
| `phi-loop-ci.yml`   | push         | parse / gen / conformance (see workflow)  |
| `now-sync-gate.yml` | push         | `NOW.md` freshness window                 |
| **Conformance**     | CI / local   | `t27c validate-conformance --repo-root .` |
| **Gen headers**     | CI / local   | `t27c validate-gen-headers --repo-root .` |


**Agent sync:** `.trinity/state/github-sync.json`  
**Hooks:** `bash scripts/setup-git-hooks.sh`  
**Manual:** `./scripts/tri check-now`

---

## § 8  Document map


| Topic                      | Document                                                                                                    |
| -------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Constitution v1.2          | `[T27-CONSTITUTION.md](T27-CONSTITUTION.md)`                                                                |
| Ring log                   | `.trinity/experience/clara_track1.jsonl`                                                                    |
| Queen health               | `.trinity/state/queen-health.json`                                                                          |
| Rolling integration detail | `[ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md](coordination/ROLLING-INTEGRATION-PLAN-SEED-TO-QUEEN.md)`       |
| Numeric SSOT               | `conformance/FORMAT-SPEC-001.json` + `[NUMERIC-STANDARD-001.md](nona-02-organism/NUMERIC-STANDARD-001.md)`  |
| Claims registry            | `[RESEARCH_CLAIMS.md](nona-03-manifest/RESEARCH_CLAIMS.md)`                                                 |
| Math/physics test charter  | `[T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md](nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md)`       |
| Axiom/theorem format       | `[T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md](nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md)` |
| Publications pipeline      | `[PUBLICATION_PIPELINE.md](PUBLICATION_PIPELINE.md)`                                                        |
| Roadmap umbrella           | [#126](https://github.com/gHashTag/t27/issues/126)                                                          |


---

## § 9  Next actions (48 h)

```bash
# 1. Milestone (needs gh auth)
# gh issue edit 127 128 129 130 131 132 133 --milestone "EPOCH-01-HARDEN"

# 2. Bootstrap + suite
cd bootstrap && cargo build --release
./target/release/t27c validate-conformance --repo-root ..
./target/release/t27c validate-gen-headers --repo-root ..
./target/release/t27c suite --repo-root ..

# 3. NOW gate (local calendar date must match **Last updated** date prefix)
cd .. && ./scripts/tri check-now

# 4. Optional: compiler hash (if stage0/FROZEN_HASH exists in your tree)
# shasum -a 256 bootstrap/src/compiler.rs

# 5. Experience log — only after a real run
# echo '{"ring":46,"task":"…","verdict":"clean","timestamp":"2026-04-06T12:00:00Z"}' >> .trinity/experience/clara_track1.jsonl

# 6. gh issue comment 126 --body "…"
```

---

*Living documentation corpus · `[T27-CONSTITUTION.md](T27-CONSTITUTION.md)` v1.2, Article DOCS-TREE · **Last updated** must include **calendar date** `YYYY-MM-DD` (for `tri check-now`). Prefer **human-readable local wall time** plus optional **RFC3339 with offset** (e.g. `2026-04-06T18:45:00+07:00`) so tools can echo it — **do not require UTC `Z`** unless you work in UTC.*