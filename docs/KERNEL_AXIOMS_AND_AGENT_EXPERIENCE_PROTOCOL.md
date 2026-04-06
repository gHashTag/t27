# T27 kernel axioms, agent experience protocol & scientific paper discipline

**Status:** Architecture / methodology (draft). English-only for first-party docs policy.  
**See also:** [`TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md`](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md), root **`SOUL.md`**, **`docs/NOW.md`**.

## Executive summary

This note ties together three threads:

1. **Kernel discipline** — a small set of **axioms** for the T27 core, inspired by minimal trusted kernels (e.g. Lean, Coq CIC) and verified microkernels (e.g. seL4): *what is in-kernel must be minimal, checkable, and hard to change casually.*
2. **Agent ↔ Queen experience** — a protocol layering **ExpeL**-style episodes, **Reflexion**-style verbal traces, and **multi-agent co-learning** patterns onto **`.trinity/`** storage and future **`tri` / `t27c`** tooling (not ad-hoc shell orchestration for gates).
3. **Scientific writing** — **IMRaD** (Introduction, Methods, Results, Discussion) as the default shape for **experiment logs** so results stay citable and reviewable.

---

## Part I — Scientific writing (IMRaD + local structure)

### IMRaD (standard article shape)

| Section | Typical share | Reader question |
|---------|----------------|-----------------|
| Introduction | ~10–15% | What gap do we fill? |
| Methods | ~20–30% | How was it done (reproducible)? |
| Results | ~20–25% | What was found (facts only)? |
| Discussion | ~25–30% | What does it mean vs prior work? |

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

| # | Principle | Intent |
|---|-------------|--------|
| 1 | Uniformity | Similar things look and behave similarly |
| 2 | Clarity | Mechanisms are well defined; code is predictable |
| 3 | Referential transparency | Replace expression with equal value → same meaning |
| 4 | Subtyping | Subtype usable wherever supertype expected |
| 5 | Information hiding | Modules see only what they need |
| 6 | Explicit interfaces | Cross-module contracts are explicit |
| 7 | Generality | Fewer special cases; unify constructs |
| 8 | Extensibility | Users can extend without breaking core |
| 9 | Implementability | Compiler/runtime is buildable |
| 10 | Simplicity | One concept beats two if coverage overlaps |

### Functional purity targets for the core

1. Functions as values  
2. Determinism (same inputs → same outputs)  
3. No implicit side effects in core semantics  
4. Immutability by default  
5. Declarative reading of specs  
6. Composition over ad-hoc glue  
7. Referential transparency for pure fragments  

*(Full language may add effectful host edges; **core spec semantics** should stay pure.)*

### AXIOM-K1 — Ternary completeness

> Every primitive value class in the **ternary strand** is drawn from **trits** `{-1, 0, +1}` (no silent fourth “logical” value in that layer).

*Motivation:* strong three-valued logics (e.g. Kleene-style) as conceptual background; concrete ISA details live in specs under **`specs/`**.

### AXIOM-K2 — Phi identity (algebraic)

> Golden ratio satisfies **φ² = φ + 1**, hence **φ² + φ⁻² = 3** as an **algebraic** identity (not a float “==”).

*Practice:* float checks in tooling use **explicit tolerances** (e.g. ≤ 1e-12 where applicable); never claim bitwise equality for IEEE-style floats as “proof” of φ-identities.

### AXIOM-K3 — Referential transparency (core)

> Pure core expressions may be replaced by their values without changing program meaning.

*Intent:* mutable state, I/O, and host effects (if any) are **explicit** at boundaries, not smuggled through the spec core.

### AXIOM-K4 — Minimal trusted kernel (scope)

> The **trusted** slice of the bootstrap pipeline is **minimal**: parsing + type checking + deterministic codegen contracts for supported backends. Libraries and orchestration grow **outside** that slice.

*Mapping:* `bootstrap/src/` evolves under the same discipline: shrink trust surface, test everything else.

### AXIOM-K5 — Issue-gated change (process)

> Work that lands in `master` is expected to trace to **GitHub issues** and PR discipline (`Closes #N` where the project requires it — see **`issue-gate`** workflow).

*Intent:* history reads as a lab notebook, not anonymous diffs.

### AXIOM-K6 — Spec generates code

> Files under **`gen/`** are **outputs** of **`t27c gen*`** from **`.t27`** sources; hand-edits are policy violations unless explicitly exempted.

*Invariant (engineering):* `gen_file ≈ F(spec, compiler_version)` — deterministic for a pinned compiler.

### Theorems / disciplined claims (proof obligations TBD)

The following are **named claims** for documentation and CI direction; **formal in-mechanism proofs** are future work unless stated otherwise.

| ID | Claim | Notes |
|----|--------|------|
| **THEOREM-K1** | Ternary sufficiency for the **ternary NN / HSLM** strand | Engineering thesis: {-1,0,+1} weights support the intended algebra; validate per spec tests. |
| **THEOREM-K2** | **φ-distance** induces an ordering over selected numeric formats | Define metric in numeric specs; compare **documented** distances — not hand-waved. |
| **THEOREM-K3** | **Codegen idempotency** | Same spec + same `t27c` version → bitwise-stable output (modulo explicit non-determinism policy). |
| **THEOREM-K4** | **Issue traceability** | With AXIOM-K5, `git log` + issues reconstruct *why* artifacts exist. |

---

## Part III — Agent experience & Queen protocol

### Literature anchors (no fine-tuning required)

- **ExpeL** — accumulate trials, distill NL insights, retrieve at inference (**AAAI 2024** line).  
- **Reflexion** — actor / evaluator / self-reflection loop; verbal traces improve retries (**NeurIPS 2023** line).  
- **Experiential co-learning** — multi-agent co-tracking / co-memory (**ACL 2024** line).  
- **Hierarchical memory** — working → episodic → semantic → procedural tiers (modern agent stacks).

Details and voting lifecycle: **[`TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md`](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)**.

### Four memory tiers (Trinity mapping)

| Tier | Role | Example locations (evolving) |
|------|------|-------------------------------|
| 1 | Working / in-context | session state, ring context |
| 2 | Episodic | `.trinity/experience/*.jsonl` |
| 3 | Semantic (Queen) | extracted insights / wisdom logs (to be standardized) |
| 4 | Procedural / law | **`SOUL.md`**, **`T27-CONSTITUTION.md`**, this axioms note |

### Episode record (v3 direction — JSON)

Rich episodes include **attempt logs**, **reflection** strings, **learnings** with optional votes, **mistakes** with prevention rules, and **`knowledge_push`** for Queen ingestion. Exact **JSON Schema** is a separate deliverable (Phase 1 in the roadmap below).

### Three channels

1. **PUSH (agent → Queen)** — after a ring/task, append structured learnings + optional broadcast flags.  
2. **PULL (agent ← Queen)** — before work, fetch top-k insights for a **domain** (numerics, compiler, …).  
3. **BROADCAST (Queen → all)** — periodic or ring-boundary digests (health, anti-patterns, open questions).

### Insight evolution (ExpeL-style policy sketch)

| Event | Action |
|-------|--------|
| Agent corroborates insight | `upvotes += 1` |
| Agent refutes with evidence | `downvotes += 1` + `counter_evidence` |
| Score very negative | mark **DEPRECATED** |
| Score very positive | candidate for promotion into **Tier 4** (constitutional review) |
| Conflicting insights | open a **DELTA** / **SIGMA** doc pair to resolve |

---

## Part IV — Phased implementation (roadmap)

Shell scripts for experience **gates** conflict with **`tests/OWNERS.md`** policy; prefer **`t27c` / `tri`** subcommands + JSON Schema + CI steps calling **`./scripts/tri …`** only as the shim.

| Phase | Focus | Example artifacts |
|-------|--------|-------------------|
| 0 | Kernel axiom doc + parseable spec stub | this file; future `specs/kernel/axioms.t27` (tracked issue) |
| 1 | Episodic engine | `episode_v3` JSON Schema; real non-empty episodes |
| 2 | Queen semantic layer | `insights.jsonl` / `wisdom.jsonl`, digest JSON |
| 3 | Paper discipline | `docs/templates/EXP_TEMPLATE.md`, DELTA/SIGMA/OMEGA logs |
| 4 | CI | optional `experience-gate.yml` (warnings first), axiom reference lint |

---

## Part V — Axiom ↔ issue ↔ spec map (working)

| Axiom | Example links |
|-------|----------------|
| **K1** | Balanced ternary / K₃-style truth tables (repo issues **#138**, **#143** — verify on GitHub) |
| **K2** | `specs/math/constants.t27`, sacred physics specs, conformance formats |
| **K3** | `specs/base/ops.t27`, purity language in **`SOUL.md`** |
| **K4** | `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH` |
| **K5** | `.github/workflows/issue-gate.yml` |
| **K6** | Gen header validators, `tri validate-gen-headers`, no hand-edit **`gen/`** |

*(Paths and issue numbers drift — treat this table as **navigation**, not law.)*

---

## Appendix — 48h bootstrap (for humans / agents)

```bash
cd bootstrap && cargo build --release
cd .. && ./scripts/tri test
./scripts/tri validate-conformance
./scripts/tri validate-gen-headers
```

After a green run, record one **`EXP-NNN`** mini-paper under `docs/` or `.trinity/experience/` per team convention, with **`Verdict`** and **Issue** linkage.

---

*This document intentionally omits informal citation URLs; prefer DOI / arXiv / official proceedings links when citing papers in future PRs.*
