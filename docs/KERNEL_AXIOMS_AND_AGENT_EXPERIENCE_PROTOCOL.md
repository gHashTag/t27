# T27 kernel axioms, agent experience protocol & scientific paper discipline

**Status:** Architecture / methodology (draft). English-only for first-party docs policy.  
**See also:** `[TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)`, `[KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md](KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md)`, `[SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md](SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md)`, `[RESEARCH_WRITING_T27.md](RESEARCH_WRITING_T27.md)`, `[T27_KERNEL_FORMAL_COQ.md](T27_KERNEL_FORMAL_COQ.md)` + `**coq/`**, `[COMPILER_VERIFICATION_STANDARDS.md](COMPILER_VERIFICATION_STANDARDS.md)` · `[COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md)` (index), root `**SOUL.md**`, `**NOW.md**`.

## Executive summary

This note ties together three threads:

1. **Kernel discipline** — a small set of **axioms** for the T27 core, inspired by minimal trusted kernels (e.g. Lean, Coq CIC) and verified microkernels (e.g. seL4): *what is in-kernel must be minimal, checkable, and hard to change casually.*
2. **Agent ↔ Queen experience** — a protocol layering **ExpeL**-style episodes, **Reflexion**-style verbal traces, and **multi-agent co-learning** patterns onto `**.trinity/`** storage and future `**tri` / `t27c**` tooling (not ad-hoc shell orchestration for gates).
3. **Scientific writing** — **IMRaD** (Introduction, Methods, Results, Discussion) as the default shape for **experiment logs** so results stay citable and reviewable.

---

## Part I — Scientific writing (IMRaD + local structure)

### IMRaD (standard article shape)


| Section      | Typical share | Reader question                  |
| ------------ | ------------- | -------------------------------- |
| Introduction | ~10–15%       | What gap do we fill?             |
| Methods      | ~20–30%       | How was it done (reproducible)?  |
| Results      | ~20–25%       | What was found (facts only)?     |
| Discussion   | ~25–30%       | What does it mean vs prior work? |


### Context → Content → Conclusion (C–C–C)

Apply at **paper**, **section**, and **paragraph** scales: open with context, deliver new content, close with a takeaway sentence.

### “No zigzag”

One topic → one place. Parallel lists use **parallel syntax** to reduce reviewer load.

### Mini-paper template for T27 experiments (`EXP-NNN`)

```markdown
## EXP-NNN: <title>

**Ring:** NNN  **Issue:** #NNN  **Status:** draft | sealed | published

### Context (Introduction)
<Knowledge gap. What theory predicts.>

### Method
<Inputs. Toolchain. Commands. Acceptance criteria.>

### Results
<CI verdicts. Tables. Numbers. Facts only.>

### Discussion
<Meaning for the next ring. Limits. Open questions.>

### Verdict
CLEAN | TOXIC | PARTIAL — <one line>
```

---

## Part II — Trusted kernel patterns & T27 axioms

### Minimal trusted kernel (pattern)

Trusted code stays **small**. Everything else is elaboration above it (tactics, elaborators, libraries). seL4 illustrates the same idea at OS scale (~10k LOC C with formal proof). **Rule:** if it is “kernel”, it must earn its place.

### Language-design principles (checklist)


| #   | Principle                | Intent                                             |
| --- | ------------------------ | -------------------------------------------------- |
| 1   | Uniformity               | Similar things look and behave similarly           |
| 2   | Clarity                  | Mechanisms are well defined; code is predictable   |
| 3   | Referential transparency | Replace expression with equal value → same meaning |
| 4   | Subtyping                | Subtype usable wherever supertype expected         |
| 5   | Information hiding       | Modules see only what they need                    |
| 6   | Explicit interfaces      | Cross-module contracts are explicit                |
| 7   | Generality               | Fewer special cases; unify constructs              |
| 8   | Extensibility            | Users can extend without breaking core             |
| 9   | Implementability         | Compiler/runtime is buildable                      |
| 10  | Simplicity               | One concept beats two if coverage overlaps         |


### Functional purity targets for the core

1. Functions as values
2. Determinism (same inputs → same outputs)
3. No implicit side effects in core semantics
4. Immutability by default
5. Declarative reading of specs
6. Composition over ad-hoc glue
7. Referential transparency for pure fragments

*(Full language may add effectful host edges; **core spec semantics** should stay pure.)*

### Layering — semantic kernel vs process law

Do **not** treat repository **governance** as φ-style **mathematics**. Keep layers explicit:


| Layer                        | Role                                                       | In this note                                                                      |
| ---------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------- |
| **Semantic kernel**          | Trits, φ-algebra, purity, minimal trusted TC/codegen slice | **AXIOM-K1 … K4**                                                                 |
| **Architectural invariants** | Trust boundary, bootstrap/stage0 discipline                | **K4** + `bootstrap/stage0/`                                                      |
| **Process laws**             | Issue traceability, spec→gen policy                        | **AXIOM-K5 … K6** — mirror `**SOUL.md`**, `**T27-CONSTITUTION.md**`, `**NOW.md**` |


Reviews also recommend an explicit **trust-chain** story for the compiler (what is trusted vs verified) and, for publication honesty, a short **independence / consistency posture** (what is claimed vs out-of-scope, e.g. limits à la incompleteness — without pretending full metatheory is done).

### Semantic kernel axioms (K1–K4)

### AXIOM-K1 — Ternary completeness

> Every primitive value class in the **ternary strand** is drawn from **trits** `{-1, 0, +1}` (no silent fourth “logical” value in that layer).

*Motivation:* strong three-valued logics (e.g. Kleene-style) as conceptual background; concrete ISA details live in specs under `**specs/`**.

### AXIOM-K2 — Phi identity (algebraic)

> Golden ratio satisfies **φ² = φ + 1**, hence **φ² + φ⁻² = 3** as an **algebraic** identity (not a float “==”).

*Practice:* float checks in tooling use **explicit tolerances** (e.g. ≤ 1e-12 where applicable); never claim bitwise equality for IEEE-style floats as “proof” of φ-identities.

### AXIOM-K3 — Referential transparency (core)

> Pure core expressions may be replaced by their values without changing program meaning.

*Intent:* mutable state, I/O, and host effects (if any) are **explicit** at boundaries, not smuggled through the spec core.

### AXIOM-K4 — Minimal trusted kernel (scope)

> The **trusted** slice of the bootstrap pipeline is **minimal**: parsing + type checking + deterministic codegen contracts for supported backends. Libraries and orchestration grow **outside** that slice.

*Mapping:* `bootstrap/src/` evolves under the same discipline: shrink trust surface, test everything else.

### Process laws (K5–K6)

### AXIOM-K5 — Issue-gated change (process)

> Work that lands in `master` is expected to trace to **GitHub issues** and PR discipline (`Closes #N` where the project requires it — see `**issue-gate`** workflow).

*Intent:* history reads as a lab notebook, not anonymous diffs.

### AXIOM-K6 — Spec generates code

> Files under `**gen/*`* are **outputs** of `**t27c gen*`** from `**.t27**` sources; hand-edits are policy violations unless explicitly exempted.

*Invariant (engineering):* `gen_file ≈ F(spec, compiler_version)` — deterministic for a pinned compiler.

### Claim / theorem statuses

Use an explicit **status** on every named claim (avoid bare “QED” without meaning):


| Status          | Meaning                                                               |
| --------------- | --------------------------------------------------------------------- |
| **FORMAL**      | Machine-checked or equivalent in a proof assistant / checker pipeline |
| **RIGOROUS**    | Complete informal proof to a reviewable standard                      |
| **ENGINEERING** | Bounded claim backed by tests + explicit scope                        |
| **CONJECTURE**  | Hypothesis; may be falsified (DELTA-style)                            |


### Theorems / disciplined claims (proof obligations TBD)

The following are **named claims** for documentation and CI direction; assign **status** per row and upgrade over time.


| ID             | Status (target) | Claim                                                    | Notes                                                           |
| -------------- | --------------- | -------------------------------------------------------- | --------------------------------------------------------------- |
| **THEOREM-K1** | ENGINEERING → … | Ternary sufficiency for the **ternary NN / HSLM** strand | Back with spec tests; {-1,0,+1} weight algebra.                 |
| **THEOREM-K2** | ENGINEERING → … | **φ-distance** ordering over selected numeric formats    | Define metric in specs; publish numbers, don’t hand-wave.       |
| **THEOREM-K3** | ENGINEERING     | **Codegen idempotency**                                  | Same spec + same `t27c` → stable output (modulo stated policy). |
| **THEOREM-K4** | ENGINEERING     | **Issue traceability**                                   | With **K5**, history explains *why* artifacts exist.            |


---

## Part III — Agent experience & Queen protocol

### Literature anchors (no fine-tuning required)

- **ExpeL** — accumulate trials, distill NL insights, retrieve at inference (**AAAI 2024** line).  
- **Reflexion** — actor / evaluator / self-reflection loop; verbal traces improve retries (**NeurIPS 2023** line).  
- **Experiential co-learning** — multi-agent co-tracking / co-memory (**ACL 2024** line).  
- **Hierarchical memory** — working → episodic → semantic → procedural tiers (modern agent stacks).

Details and voting lifecycle: `**[TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)*`*.

### Four memory tiers (Trinity mapping)


| Tier | Role                 | Example locations (evolving)                               |
| ---- | -------------------- | ---------------------------------------------------------- |
| 1    | Working / in-context | session state, ring context                                |
| 2    | Episodic             | `.trinity/experience/*.jsonl`                              |
| 3    | Semantic (Queen)     | extracted insights / wisdom logs (to be standardized)      |
| 4    | Procedural / law     | `**SOUL.md**`, `**T27-CONSTITUTION.md**`, this axioms note |


### Episode record (v3 direction — JSON)

Rich episodes include **attempt logs**, **reflection** strings, **learnings** with optional votes, **mistakes** with prevention rules, and `**knowledge_push`** for Queen ingestion. Every on-disk JSON episode **must** carry `**schema_version`** so migrations stay explicit.

Exact **JSON Schema** is a separate deliverable (Phase 1 in the roadmap below). **Shift-left:** add CI validation in the **same** ring band as the schema, before bulk ingestion into Tier 3.

**DELTA / SIGMA / OMEGA** markdown can remain publication-facing; prefer a **single append-only experience SSOT** where practical, with docs as **views** or summaries to avoid contradictory duplicates.

**Queen cold start:** bootstrap Tier 3 from historical logs (early rings, verdicts, DELTA-style docs) while new v3 episodes ramp up.

**KPIs (experience system):** track error repetition, time-to-green CI, reopen rate — not only “N insights created.”

### Three channels

1. **PUSH (agent → Queen)** — after a ring/task, append structured learnings + optional broadcast flags.
2. **PULL (agent ← Queen)** — before work, fetch top-k insights for a **domain** (numerics, compiler, …).
3. **BROADCAST (Queen → all)** — periodic or ring-boundary digests (health, anti-patterns, open questions).

### Insight evolution (ExpeL-style policy sketch)


| Event                       | Action                                                                                                              |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Agent corroborates insight  | `upvotes += 1`                                                                                                      |
| Agent refutes with evidence | `downvotes += 1` + `counter_evidence`                                                                               |
| Score very negative         | mark **DEPRECATED**                                                                                                 |
| Score very positive         | candidate for promotion into **Tier 4** (constitutional review — prefer **RFC / consensus** before treating as law) |
| Conflicting insights        | open a **DELTA** / **SIGMA** doc pair to resolve                                                                    |


---

## Part IV — Phased implementation (roadmap)

Shell scripts for experience **gates** conflict with `**tests/OWNERS.md`** policy; prefer `**t27c` / `tri**` subcommands + JSON Schema + CI steps calling `**./scripts/tri …**` only as the shim.


| Phase  | Focus                                  | Example artifacts                                                                                                            |
| ------ | -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **−1** | **Advertised E2E pipeline**            | Minimal `seed.t27` (or simpler) → `t27c gen` → `**zig test` GREEN** in **GitHub Actions**; merge blocker for dependent epics |
| 0      | Kernel axiom doc + parseable spec stub | this file; future `specs/kernel/axioms.t27` (tracked issue)                                                                  |
| 1      | Episodic engine + **left-shift CI**    | `episode_v3` JSON Schema + `**schema_version`**; validator in CI; real non-empty episodes                                    |
| 2      | Queen semantic layer                   | `insights.jsonl` / `wisdom.jsonl`, digest JSON; **cold-start** import from history                                           |
| 3      | Paper discipline                       | `docs/templates/EXP_TEMPLATE.md`, DELTA/SIGMA/OMEGA logs (as views where possible)                                           |
| 4      | CI depth                               | `experience-gate.yml` tighten (warn → fail), axiom-reference lint, trust-chain checklist                                     |


---

## Part V — Axiom ↔ issue ↔ spec map (working)


| Axiom  | Example links                                                                                |
| ------ | -------------------------------------------------------------------------------------------- |
| **K1** | Balanced ternary / K₃-style truth tables (repo issues **#138**, **#143** — verify on GitHub) |
| **K2** | `specs/math/constants.t27`, sacred physics specs, conformance formats                        |
| **K3** | `specs/base/ops.t27`, purity language in `**SOUL.md`**                                       |
| **K4** | `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`                                  |
| **K5** | `.github/workflows/issue-gate.yml`                                                           |
| **K6** | Gen header validators, `tri validate-gen-headers`, no hand-edit `**gen/`**                   |


*(Paths and issue numbers drift — treat this table as **navigation**, not law.)*

---

## Appendix — 48h bootstrap (for humans / agents)

```bash
cd bootstrap && cargo build --release
cd .. && ./scripts/tri test
./scripts/tri validate-conformance
./scripts/tri validate-gen-headers
```

After a green run, record one `**EXP-NNN**` mini-paper under `docs/` or `.trinity/experience/` per team convention, with `**Verdict**` and **Issue** linkage.

---

*This document intentionally omits informal citation URLs; prefer DOI / arXiv / official proceedings links when citing papers in future PRs.*