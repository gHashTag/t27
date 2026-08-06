# EPIC — Trinity Ecosystem → t27 Rewrite (Wave 2026-07-08)

**Root repo:** `gHashTag/t27` (user decision 2026-07-08)
**Wave branch series:** `ring-105-XXX-<repo>-<module>`
**Base commit:** `4832ec6a` on `master`
**Constitutional law:** L1 > L2 > L3 > L4 > L5 > L6 > L7 > L8 (Asimov)
**Loop:** AEL v2.0 (OBSERVE → PLAN → DELEGATE → VERIFY → SYNTHESIZE → LEARN)
**Cycle:** edit spec → seal hash → gen → test → verdict → save experience → skill commit → git commit

---

## Motivation (from user, verbatim, translated)

> "Перепиши 7 репозиториев в спецификационный язык t27, объедини в одну экосистему в
> корне `gHashTag/t27`, сделай базу для обучения IGLA CODER/RACE, собери все связанные
> GitHub issues, создай эпик + weakness audit + научный research + декомпозированный
> план + реализация + отчёт + 3 варианта cooperation + сохрани скиллы."

Rewrite prime rules from user:
- **NEVER write .zig/.rs directly** — always generate from `.tri`.
- **NO logic duplication between spec and code** — one source of truth.
- **.tri creation FIRST, generation LAST.**
- Use many todo items + subagents for parallelism.

## Scope Lock (from Weakness Audit)

This Wave delivers **pilots**, not full-repo rewrites. Six-eight `.tri` seeds, one codegen plan, one dataset v0.1 seed, one report, one PR. Everything else is Wave-2/3/4 (see §7 roadmap).

## Ring numbering rationale

`ring-104-rust` is the latest live ring on master (verified via `ls rings/ | tail`). This Wave opens **`ring-105-*`** namespace. All pilot branches follow `ring-105-XXX-<repo>-<module>` schema:

| ring | branch | source repo | module | target codegen | LOC estimate |
|---|---|---|---|---|---|
| 105-000 | `ring-105-000-preflight-parse` | (meta) | preflight baseline | none (docs) | 50 |
| 105-001 | `ring-105-001-t27-experience-format` | t27 | experience.tri (PHI LOOP artefact schema) | none (spec only) | 120 |
| 105-002 | `ring-105-002-trinity-mozg-dna` | trinity | organism/mozg.tri + organism/dna.tri | zig | 200 (spec) → 500 (gen) |
| 105-003 | `ring-105-003-trios-git-orchestrator` | trios | orchestrator/git_bridge.tri | zig | 150 → 300 |
| 105-004 | `ring-105-004-trios-mcp-tool-registry` | trios-mcp | mcp/tool_registry.tri | rust | 180 → 400 |
| 105-005 | `ring-105-005-multibots-scene-schema` | 999-multibots-telegraf | multibots/scene_schema.tri (ANONYMISED, PUBLIC-SAFE) | typescript (DEFERRED to Wave-3) | 130 → 300 |
| 105-006 | `ring-105-006-multibots-rust-ring-runtime` | 999-multibots-rust | multibots/ring_runtime.tri (ANONYMISED) | rust | 160 → 350 |
| 105-007 | `ring-105-007-igla-coder-dataset-v01` | (meta) | dataset/igla-coder/v0.1/ manifest + decontam | none (dataset) | 100 |

**Total pilots:** 8 branches, ~1090 LOC of `.tri` spec + ~1850 LOC of generated code + dataset manifest.

**Note on ring-105-005 (999-multibots-telegraf):** the source is PRIVATE and includes payment surface + PII. The pilot `scene_schema.tri` describes only the **generic public-safe schema** (state machine types, transition invariants, message types) — NO real bot names, NO real tariffs, NO real user IDs. Kept in the public t27 repo because the schema itself carries no sensitive value.

**Note on ring-105-006 (999-multibots-rust):** same anonymisation. Pilot describes the **Ring runtime state machine** at type level — no wallet keys, no seed phrases, no bot-token surface.

---

## §1. Hard Rules (carried into every pilot PR)

Sourced from `t27/AGENTS.md`, `t27/SOUL.md`, and `t27/CLAUDE.md`.

1. **L1 TRACEABILITY** — every PR body has `Closes #<N>` linking to an existing t27 issue (or a new issue we file first).
2. **L2 GENERATION** — no hand-edit under `gen/**` in any pilot PR. Any change in `gen/` MUST come from re-running codegen on updated spec.
3. **L3 PURITY** — ASCII-only, English identifiers, no Cyrillic. Applies to `.tri`, `.t27`, generated code, and Markdown docs under `docs/`, `specs/`, `architecture/`, `conformance/`.
4. **L4 TESTABILITY** — every `.tri` in a pilot has **≥3 invariants, ≥8 tests, ≥2 benchmarks** (matches existing template `specs/01-tri-lang-core.tri`).
5. **L5 IDENTITY** — where a pilot involves phi or Trinity math, use the tolerance-based IEEE f64 check (`abs(phi*phi + 1/phi*phi - 3) < 1e-15`), never bit-exact equality.
6. **L6 CEILING** — no pilot invents new numeric formats. Any numeric mention (gf16, tf3, bf16, fp8_*) MUST cite `specs/numeric/formats_catalog.t27` line via the `numericformat` header.
7. **L7 UNITY** — no new `*.sh`. Any tooling MUST be `tri` subcommand or `t27c` extension. Python only if `bootstrap/t27c.py` line already exists and no critical-path violation is introduced.
8. **HR-async** — all outreach that this Wave triggers (issues we file on t27) is async-only. NO Zoom / call offers.
9. **HR-catalog** — any `.tri` file referencing a numeric format count MUST include the live grep in its PR body: `grep -c '// CATALOG:' specs/numeric/formats_catalog.t27` → 83 today.
10. **HR-drift** — every pilot PR body includes a `gen-diff-report.txt` snippet showing `git diff --stat gen/` after regeneration. If diff is non-empty and unexpected, PR is BLOCKED.
11. **HR-decontam** — for the dataset pilot (ring-105-007), every added `(spec, gen)` pair passes a Lee 2022 k=50 substring check against a held-out eval set (structure documented in `dataset/igla-coder/v0.1/DECONTAM.md`).
12. **HR-experience** — every pilot PR creates `experience/ring-105-XXX/verdict.json` with `{spec_hash, gen_hash, tests_passed, invariants_verified, bench_delta_pct, timestamp, agent_id}`.
13. **HR-1** — never nudge. If maintainer silence >7 days on a filed issue, no follow-up.
14. **HR-anonymise** — private-repo pilots (105-005, 105-006) carry only type-level structure. No secrets, no PII, no wallet, no bot-token, no user-ID, no tariff amount.
15. **HR-toxic** — the final report leads with the honest gap ("v0.1 seed = 20-40 pairs, NOT training corpus"), never with the achievement.

---

## §2. Per-Pilot Spec (all 8 rings)

### ring-105-000 — Preflight Parse

- **Type:** docs-only meta PR
- **Files added:**
  - `audit/BASELINE_PARSE.log` (already drafted)
  - `docs/wave-2026-07-08/README.md` (index of this Wave)
- **Purpose:** land the parse-baseline decision (TRAIN-BOX PENDING) publicly BEFORE any pilot spec merges, so downstream pilots have a documented starting-point.
- **`Closes`:** we file `t27#new-1` with title "Wave 2026-07-08: ecosystem tri rewrite preflight"; body links this EPIC.md.
- **Verdict:** PASS on merge if user confirms in issue that they'll run local `cargo build --release --bin t27c`.

### ring-105-001 — Experience Format spec

- **Type:** `.tri` spec addition (new module, no codegen)
- **File:** `specs/organism/experience.tri`
- **Content:**
  - `spec experience { ... }` block
  - Types: `Verdict`, `SpecHash`, `AgentId`, `RingId`, `Timestamp`
  - Functions (≥3): `verdict_new()`, `verdict_seal(v: Verdict) -> Bytes32`, `verdict_pass(v: Verdict) -> bool`
  - Invariants (≥3): sha256 seal length = 32 bytes, timestamps monotone within a ring, agent_id ASCII-only
  - Tests (≥8): new/roundtrip/seal-idempotent/pass-conditions/agent_id-ascii/timestamp-monotone/hash-collision-resistance-anecdote/serialize-deserialize
  - Benchmarks (≥2): sha256 seal on 4KB verdict, roundtrip serialize+deserialize
- **`Closes`:** we file `t27#new-2` "define PHI LOOP experience artefact schema".
- **No gen/ change** — this is a spec-only pilot at design time (kodegen for it comes in Wave-2).

### ring-105-002 — Trinity Mozg + DNA

- **Type:** `.tri` spec + zig codegen (one target)
- **Files:**
  - `specs/organism/mozg.tri` — describes the Trinity Mozg (cognitive layer) as a state machine: 27 states (one per agent letter), transitions guarded by law-priority.
  - `specs/organism/dna.tri` — describes the DNA (persistence layer): schema for skills, verdicts, and ring-experience blobs. NOT a database — a spec of the on-disk format the trinity Zig runtime writes.
  - Regenerated `gen/organism/mozg.zig` + `gen/organism/dna.zig` under L2.
- **Types:** `MozgState`, `Transition`, `DnaRecord`, `SkillId`, `RingBlob`
- **Functions:** transition step, dna_write, dna_read, dna_gc
- **Invariants:** transitions respect law priority (L1 first), no dna record without ring_id, ASCII agent_id
- **Tests:** 8 covering neuron flip, transition guard, dna round-trip, gc semantics, skill upsert, ring blob append, monotone timestamps, invalid transition rejected
- **Benchmarks:** state-machine step throughput, dna_read on 10k records
- **`Closes`:** `trinity#new-3` "trinity: freeze Mozg + DNA schema via t27 tri spec" (we file on gHashTag/trinity, referenced back from t27 PR body).
- **Gen target:** Zig (matches existing `compiler/codegen/zig/codegen.t27`).

### ring-105-003 — Trios Git Orchestrator

- **Type:** `.tri` spec + zig codegen
- **File:** `specs/git/orchestrator.tri`
- **Modelled:** the git bridge state machine used by trios: repo_open → fetch → validate → merge → push, with failure branches.
- **Types:** `GitState`, `RepoRef`, `MergeStrategy`, `OrchestrationError`
- **Functions:** state_step, fetch, validate, merge (strict fast-forward only), push
- **Invariants:** no push without validate, no merge without fetch, error states are terminal
- **Tests:** 8 covering the state graph (happy path, each error branch, idempotent open, no-op fetch, dry-run push)
- **Benchmarks:** state-step throughput, worst-case merge with 1k commits
- **`Closes`:** `trios#new-4` "trios: git orchestrator state machine as t27 spec".
- **Gen target:** Zig.

### ring-105-004 — Trios MCP Tool Registry

- **Type:** `.tri` spec + rust codegen
- **File:** `specs/mcp/tool_registry.tri`
- **Purpose:** proposes ONE canonical registry surface across trios-mcp, trios-mcp-rag, trios (dedup surface W-G).
- **Types:** `ToolId`, `ToolDescriptor`, `InvocationRecord`, `RegistryError`
- **Functions:** register, describe, invoke_prepare (no actual invocation — that's runtime), lookup
- **Invariants:** tool_id uniqueness, descriptors carry SHA-256 of their JSON schema, ASCII names only
- **Tests:** 8 covering register/collision/lookup-miss/describe-idempotence/sha256-of-schema-stable/case-sensitivity/rejection-of-cyrillic/max-name-length
- **Benchmarks:** register 1k tools, describe on 10k lookups
- **`Closes`:** `trios-mcp#new-5` "trios-mcp: unify tool registry via t27 spec".
- **Gen target:** Rust (existing `compiler/codegen/` doesn't have a Rust backend yet — this pilot INCLUDES a **stub** `compiler/codegen/rust/codegen.t27` skeleton pointing to Wave-2 for full implementation). **HONEST GAP:** Wave-3 responsibility to complete the Rust codegen.

### ring-105-005 — Multibots Scene Schema (public-safe)

- **Type:** `.tri` spec (no codegen this Wave — TypeScript codegen deferred)
- **File:** `specs/scenes/scene_schema.tri`
- **Purpose:** describes the *generic* multibot scene state machine at type level, anonymised. No real bot names.
- **Types:** `SceneState`, `Transition`, `MessageKind`, `SceneError`
- **Functions:** scene_step, transition_allowed (guard predicate), on_enter, on_exit
- **Invariants:** transitions form a DAG (no infinite loop), on_enter/on_exit idempotent, message ordering respected
- **Tests:** 8 covering typical scene flows without secrets
- **Benchmarks:** scene_step throughput, guard-eval on 1k transitions
- **`Closes`:** we open a NEW public issue `t27#new-6` "multibot scene schema as t27 spec (public-safe, TypeScript codegen deferred)". The private repo 999-multibots-telegraf receives NO issue this Wave.
- **Gen target:** DEFERRED — mention in PR body that `compiler/codegen/typescript/` does not exist yet.

### ring-105-006 — Multibots Rust Ring Runtime (anonymised)

- **Type:** `.tri` spec + rust codegen (STUB from ring-105-004)
- **File:** `specs/organism/ring_runtime.tri`
- **Purpose:** describes the multibots Ring runtime — worker state machine, message dispatch, retry policy — at type level, no secrets.
- **Types:** `RingWorker`, `Dispatch`, `RetryPolicy`, `RuntimeError`
- **Functions:** worker_spawn, dispatch, retry, worker_shutdown
- **Invariants:** retry count bounded, shutdown is idempotent, dispatch queue FIFO
- **Tests:** 8 covering worker lifecycle without secret surface
- **Benchmarks:** dispatch throughput, retry decision latency
- **`Closes`:** `t27#new-7` "ring runtime as t27 spec (public-safe, Rust codegen stub, closes gap with 999-multibots-rust arch)".
- **Gen target:** Rust (STUB — depends on ring-105-004 skeleton).

### ring-105-007 — IGLA CODER Dataset v0.1

- **Type:** dataset artefact PR (no `.tri` spec added; a `.tri` schema for the manifest lives inside)
- **Files:**
  - `dataset/igla-coder/v0.1/README.md` — v0.1 charter, scope, decontam approach.
  - `dataset/igla-coder/v0.1/MANIFEST.json` — array of `{pair_id, spec_path, spec_sha256, gen_path, gen_sha256, target_lang, ring_id, license, decontam_status}`.
  - `dataset/igla-coder/v0.1/DECONTAM.md` — describes Lee 2022 k=50 substring guard, disjoint-set assertion.
  - `dataset/igla-coder/v0.1/pairs/` — populated from rings 105-001..105-006 outputs (spec + gen file per module).
  - `specs/dataset/igla_coder_manifest.tri` — the schema itself (types + invariants + tests, ≥8 tests, ≥3 invariants, ≥2 benchmarks).
- **`Closes`:** `t27#new-8` "IGLA CODER dataset v0.1 seed".
- **Verdict criteria:**
  - MANIFEST.json parses as JSON, matches schema in `igla_coder_manifest.tri`.
  - Every `pair_id` is unique.
  - Every SHA-256 matches actual file bytes.
  - DECONTAM.md documents the check; even at v0.1 with n=20-40 pairs, we ASSERT bidirectional decontam (train vs held-out eval names).

---

## §3. PHI LOOP for each ring (mechanical)

Each pilot ring follows this 8-step cycle. The Wave agent (this session) executes steps 1-6 in workspace; steps 7-8 are user actions on train-box (documented in report).

```
1. edit spec          → write .tri file into t27/specs/**/*.tri
2. seal hash          → sha256sum specs/**/*.tri >> experience/ring-105-XXX/seal.txt
3. gen                → t27c gen <target> (rust/zig/c/verilog) → gen/**/*
                        (this Wave: designed but not machine-run; TRAIN-BOX PENDING)
4. test               → t27c test specs/**/*.tri (PENDING)
5. verdict            → experience/ring-105-XXX/verdict.json = { status: PASS|FAIL|PENDING, ... }
6. save experience    → git add experience/ring-105-XXX/
7. skill commit       → update tt-lang-t27-integration + trinity-ecosystem skills
8. git commit         → atomic commit on ring-105-XXX-* branch, PR body links Closes #N
```

**Where each ring stops in this Wave:**
- ring-105-000: steps 1-2-5-6-7-8 (docs only, no gen).
- ring-105-001, 003, 005: steps 1-2-5-6-7-8 (spec designed; gen/test = TRAIN-BOX PENDING).
- ring-105-002, 004, 006: steps 1-2-5-6-7-8 (spec designed; gen/test = TRAIN-BOX PENDING for codegen too).
- ring-105-007: steps 1-2-5-6-7-8 (schema + manifest of what rings 001-006 produced).

Step 3 (`gen`) and step 4 (`test`) run on train-box because sandbox lacks `cargo`+`rustc`.

---

## §4. Dependency graph & sequencing

```
ring-105-000 (preflight)
    |
    v
ring-105-001 (experience.tri) ──────┐
    |                               |
    v                               v
ring-105-002 (mozg+dna) ── ring-105-003 (git orch) ── ring-105-004 (mcp registry, rust stub)
    |                               |                        |
    +─────────┬─────────────────────┘                        v
              |                                    ring-105-006 (ring runtime, rust)
              v                                              |
      ring-105-005 (scene schema, TS deferred)               |
              |                                              |
              └──────────────┬───────────────────────────────┘
                             v
                    ring-105-007 (IGLA CODER v0.1 dataset)
```

**Merge order (recommended):** 000 → 001 → (002 || 003 || 004 in parallel) → 005 → 006 → 007. But each PR is standalone-mergeable; graph is soft.

---

## §5. CI + verification hooks the Wave *proposes* (not implements)

These are additions to `.github/workflows/**` that a future ring should implement. This Wave records them, does not merge them.

- **check-tri-parse.yml** — on every PR touching `specs/**/*.tri`, run `t27c parse` on the changed files; block if any fails.
- **check-gen-regenerated.yml** — on every PR touching `specs/**/*.tri`, re-run `t27c gen <target>`, compare against `gen/**`; block if diff non-empty.
- **check-experience-artefact.yml** — hint (not block) — warn if a ring-* branch has spec changes but no `experience/ring-XXX/verdict.json`.
- **check-l3-ascii.yml** — already partly exists; extend to `.tri` files explicitly.
- **check-catalog-count.yml** — HR-catalog rule: any doc referencing "N format" must match `grep -c '// CATALOG:' specs/numeric/formats_catalog.t27`.

**Filed as follow-up issue** in the report; not merged this Wave (out of scope).

---

## §6. Dataset — IGLA CODER v0.1 (per-pair contract)

Each pair in `dataset/igla-coder/v0.1/pairs/` looks like:

```
pairs/
  0001-mozg-state-machine/
    spec.tri              — from ring-105-002
    gen.zig               — from same ring (TRAIN-BOX GENERATED)
    metadata.json         — {ring, source_repo, license, target_lang, spec_sha256, gen_sha256, decontam_status}
  0002-dna-schema/
    spec.tri
    gen.zig
    metadata.json
  ...
```

**v0.1 goals:**
- ~20-40 pairs (from 6 pilot rings that produce gen outputs).
- Decontam: bidirectional Lee 2022 k=50 substring test against `dataset/igla-coder/v0.1/held-out-eval/` (which is empty at v0.1 — noted in DECONTAM.md as "assertion: no held-out eval yet defined at v0.1; decontam trivially clean; v0.2 must populate held-out eval FIRST").
- License: every pair inherits the source repo's license (Apache-2.0 for t27, MIT/varies for trinity/trios — recorded per-pair).
- SHA-256: per-file, verified in MANIFEST.json.

**v0.1 known limits (in DECONTAM.md and FINAL_REPORT.md):**
- n=20-40 pairs << phi-1 floor (350M params, ~1B tokens) — W-18 stays HIGH.
- Held-out eval undefined at v0.1 → decontam is a placeholder.
- No tokenizer/detokenizer spec included this Wave.
- No BPB baseline measurement.

**v0.2..v0.5 roadmap (out of scope, documented as future work):**
- v0.2: define held-out eval (12+ programs, name-disjoint), rerun decontam properly.
- v0.3: augment via deterministic alpha-rename (WP-10 pattern from tt-lang-integration-weakness-map).
- v0.4: add tokenizer spec `specs/tokenizer/tri_tokenizer.tri`.
- v0.5: reach ≥200K train tokens (deficit ≤ 262× per WP-10 analog).

---

## §7. Roadmap AFTER this Wave

| Wave | Focus | Trigger |
|---|---|---|
| **Wave-2** | Complete Rust codegen (used by rings 105-004/006), TypeScript codegen (ring 105-005), full `t27c parse` CI, gen-regen CI | user runs this Wave train-box, all 8 rings PASS `t27c parse` |
| **Wave-3** | Ecosystem-wide rewrite of remaining 5 modules per repo (Mozg → 27 states worked out, Trios git actions, MCP full surface, multibots scenes list) | Wave-2 CI green |
| **Wave-4** | IGLA CODER v0.2 with held-out eval, WP-10 augmentation applied to reach ≥200K train tokens | Wave-3 rewrites merged |
| **Wave-5** | First real train run of IGLA CODER on Trinity training box, W-12 compile@1 measured, W-18 verdict | v0.5 dataset ready |

---

## §8. Cooperation variants (deliverable of this Wave)

Full 3-variant menu lives in `report/COLLAB_VARIANTS.md`, but headlines:
- **A — Hunhold (Takum) tri-spec adapter:** propose `specs/numeric/takum.tri` shim, invite reciprocal citation.
- **B — P3109 WG passive cite:** publish `docs/P3109_CROSSWALK_v2.md` referring `specs/numeric/formats_catalog.t27`.
- **C — IST-DASLab / MR-GPTQ compat:** offer IGLA CODER v0.1 seed as a substrate replication target; ask 1-page sanity review.

---

## §9. Non-goals (spelled out to prevent scope creep)

- Merging any pilot to master this Wave (PR only; merge = user action).
- Touching any of the 20+ conflicting `wave-loop-XYZ` PRs (#1362..#1437).
- Rewriting `bootstrap/t27c.py` (SSOT-MATH exception — L7 debate reserved for Wave-3).
- Publishing dataset to Zenodo (v0.1 is repo-local; publication = Wave-4+).
- Any real training run (W-18/W-19 already documented as HIGH; not the goal here).
- Any change to `paper3-methodology` count (84 vs 83) — separate skill and workflow.
- Any change to `bootstrap/target/**` binaries (train-box territory).

---

## §10. Definition of Done for this Wave

1. All 8 rings have `.tri` specs designed (present in the workspace repo clone under `pilots/ring-105-XXX/`).
2. Each spec passes L3 (ASCII-only), L4 (≥3 invariants, ≥8 tests, ≥2 benchmarks), L5 (any phi math uses tolerance).
3. `dataset/igla-coder/v0.1/MANIFEST.json` schema-valid; SHA-256 of each pair file computed.
4. `experience/ring-105-XXX/verdict.json` written for each ring (status = DESIGNED_PENDING_TRAIN_BOX for design-only steps).
5. `report/FINAL_REPORT.md` names: (a) toxic verdict, (b) benchmark vs baseline, (c) 3 cooperation variants, (d) explicit gap list mirrored from Weakness Audit.
6. Skills updated: `tt-lang-t27-integration` (v1.18 delta), `scientific-works-canon` (v2.1 delta), `task-status-board` (v0.13 delta), NEW `trinity-ecosystem` at all 3 scopes (user + space + org).
7. One PR opened against `gHashTag/t27` for the whole Wave (branch `ring-105-ecosystem-tri-rewrite` collecting all 8 sub-rings' contents), body linking Closes for each `t27#new-N` we filed.
8. No banned-hype word in any artefact.
9. No claim of PASS on gen/test unless machine-verified (train-box).

---

**End of EPIC.** Root repo = **t27**.
