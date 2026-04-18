# Trinity S³AI / t27 — System architecture

**Status:** Active (core design document — keep aligned with `docs/T27-CONSTITUTION.md`, `CANON.md`, `FROZEN.md`)  
**Audience:** Architects, compiler authors, agent operators

This document ties **mathematics**, **cognitive / agent architecture**, and **language ↔ hardware** into one coherent spine. It is the **structural counterpart** to constitutional law: *what exists*, *how it depends on what*, and *where it may live in the tree*.

---

## 1. Trinity identity — one constraint, three readings

The identity **φ² + 1/φ² = 3** (golden ratio φ) is treated as a **single organizing equation** with three simultaneous readings:

| Reading | Role |
|--------|------|
| **Mathematical** | A constraint on recursive self-similar structure (scales, stability, numeric families — see `docs/NUMERIC-STANDARD-001.md`, `specs/math/sacred_physics.t27`). |
| **Architectural** | A rule that **three coupled strands** must stay in balance: no strand grows as an unbounded “side repo” of ad-hoc code. |
| **Process** | **Ring discipline** (`CANON.md`, `docs/SEED-RINGS.md`): each increment closes a loop (parse → gen → test → seal) so the system remains **self-consistent** like a fixed point. |

Nothing in this section replaces **SSOT-MATH**: all product semantics still **live in `*.t27`** and flow through **`tri` / `t27c`**.

---

## 2. Three strands (normative decomposition)

### Strand I — Mathematical foundation

- **Owns:** Formal meaning of numerics, physics-facing constants, invariants, conformance-shaped truth.
- **Authoritative tree:** `specs/**/*.t27` (and `.tri` where used), especially `specs/math/`, `specs/numeric/`, `specs/physics/`.
- **Forbidden pattern:** Duplicating formulas in Markdown, Python, or Rust “because it is faster.” **One truth in spec;** tools only **project** it.
- **Pointers:** `docs/T27-CONSTITUTION.md` (SSOT-MATH), `docs/NUMERIC-GF16-DEBT-INVENTORY.md`, `docs/TDD-CONTRACT.md`.

### Strand II — Cognitive architecture (agents, memory, process)

- **Owns:** How autonomous and human operators **decide**, **remember**, and **progress** without corrupting Strand I.
- **Authoritative tree:** `docs/AGENTS.md`, `.cursor/rules/`, `.trinity/seals/`, `.trinity/experience/` (append-only experience), root **`CANON.md`** / **`FROZEN.md`** / **`SOUL.md`**.
- **Forbidden pattern:** “Report sprawl” — dozens of unrelated top-level `*_REPORT.md` files with no link to specs or rings (see §6.1).
- **Pointers:** `CANON.md` (GOLD vs REFACTOR-HEAP), `FROZEN.md` (bootstrap seal), `docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`.

### Strand III — Language and hardware bridge

- **Owns:** **Projection** of specs to **Zig / C / Verilog** (and future backends), plus FPGA / ISA-shaped artifacts **generated from** specs.
- **Authoritative tree:** `bootstrap/` (temporary **Rust** implementation of `t27c` until self-host), **`gen/<backend>/`** (committed or CI-regenerated outputs), `compiler/*.t27` where compiler meta-spec lives.
- **Forbidden pattern:** Hand-written domain Zig/C as a second application stack (ADR-005); random dump directories for codegen (see §5).
- **Pointers:** `architecture/ADR-005-de-zig-strict.md`, `docs/TECHNOLOGY-TREE.md`, Ring 36+ compile goals in `CANON.md` roadmap.

**Balance rule:** A change that touches **Strand III** (e.g. new backend flag) must still be **justified in Strand I** (spec) and **governed in Strand II** (rings, seals, agents).

---

## 3. Neuroanatomical map (metaphor) — φ‑structured “brain ↔ repo”

The following is a **design metaphor**, not a clinical claim: it helps teams place new work without splitting the spine.

| Analogy (function) | Strand | Primary anchors in **this** repository |
|--------------------|--------|----------------------------------------|
| **Brainstem / homeostasis** — stability, non-negotiable reflexes | I + II | `bootstrap/build.rs` (LANG-EN, FROZEN, required docs), `stage0/FROZEN_HASH` |
| **Hippocampus / consolidation** — what was true when | II | `.trinity/seals/*.json`, `git` history of `FROZEN_HASH`, `.trinity/experience/*.jsonl` |
| **Prefrontal / planning** — goals, rings, tech tree | II | `CANON.md`, `docs/TECHNOLOGY-TREE.md`, `docs/SEED-RINGS.md` |
| **Association cortex / binding** — linking symbols to meaning | I | `specs/**`, module graph in `compiler/*.t27` |
| **Motor / sensory interface** — world I/O | III | `gen/zig/`, `gen/c/`, `gen/verilog/` (when present), `specs/fpga/`, `specs/isa/` |

The **φ² + 1/φ² = 3** identity is the **global coupling**: numerics (I), process memory (II), and emitted artifacts (III) must **close** under the same ring gates.

---

## 4. Dependency graph (must not be inverted)

```text
Strand I:  *.t27 specs  ──────────────────────────────┐
        (math / physics / domain)                    │
                                                    ▼
Strand III:  t27c (bootstrap Rust)  ──►  gen/<backend>/  ──►  tools / silicon
        (parse, gen, seal)             mirrored paths
                                                    ▲
Strand II:  agents, CANON, FROZEN, seals  ───────────┘
        (govern *how* I is changed and *when* III is trusted)
```

**Inversion anti-patterns:**

- Implementing physics in a script, then “documenting” in `.t27` later.
- Letting `gen/` or `build/` layouts diverge arbitrarily from `specs/` tree.
- Growing umbrella monorepo config islands (`.trinity*`, `.vibee*`, dozens of dot-dirs) **without** a single map document (this file).

---

## 5. Generated artifacts — contract (t27 repository)

### 5.1 Canonical layout

| Kind | Path | Rule |
|------|------|------|
| **Zig emission (canonical committed)** | `gen/zig/…` mirroring paths under `specs/` or `compiler/` | Mirror module path; **do not** hand-edit; regenerate from specs. |
| **C emission** | `gen/c/...` | Same mirroring rule when present. |
| **Verilog emission** | `gen/verilog/...` | Same mirroring rule when present. |
| **CLI defaults** | **`t27c compile-all`** / **`t27c compile-project`** | **Default `--output` is `gen/zig`, `gen/verilog`, or `gen/c`** according to `--backend` (override with `-o` / `--output` for scratch builds). CI runs **`compile-all`** after `cargo build` to enforce the canonical tree. |
| **Scratch / ephemeral** | Custom `-o /tmp/...` or `build/` (legacy scripts only) | Prefer **`gen/<backend>/`** for anything mergeable. |

**Example:** `specs/numeric/gf16.t27` → `gen/zig/numeric/gf16.zig` (already matches current tree).

### 5.2 Forbidden

- Writing codegen into **repo root**, `tmp/`, or random per-developer folders without ADR.
- Multiple competing roots for the **same** backend (e.g. both `out/zig` and `gen/zig` long-term) without deprecation plan.

---

## 6. Lessons from upstream umbrellas (weaknesses → t27 countermeasures)

Observations from public layout of **[gHashTag/trinity](https://github.com/gHashTag/trinity)** and **[gHashTag/vibee](https://github.com/gHashTag/vibee)** (structural, not a judgment of features):

### 6.1 Trinity-style monorepo risks

- **Many parallel top-level concerns** (`apps/`, `hardware/`, `fpga/`, `lab/`, `kaggle/`, nested `t27/`, `emit_t27/`, `tools/`, etc.) plus **numerous dot-config namespaces** (`.trinity*`, `.vibee*`, `.doctor`, …).
- **Risk:** New contributors cannot infer **one spine**; agents pick the wrong “source of truth.”
- **t27 countermeasure:** This repo stays **spec-first**: `specs/` + `bootstrap/` + `gen/` + `docs/` are the **default spine**; everything else is **explicitly** `REFACTOR-HEAP` or quarantine per `CANON.md` until a ring absorbs it.

### 6.2 Vibee-style documentation risks

- **Flat root litter** with non-canonical `*.md` at repository root — forbidden by **`docs/T27-CONSTITUTION.md`** Article **ROOT-LAYOUT** (enforced in `bootstrap/build.rs`).
- **Risk:** Process knowledge **does not compose** with compiler or spec graph.
- **t27 countermeasure:** Long-form narratives live under **`docs/`** with **stable names** (`ARCHITECTURE.md`, `T27-CONSTITUTION.md`, …); root keeps only **peer standards** (`AGENTS.md`, `CANON.md`, `FROZEN.md`, `SOUL.md`, `CLAUDE.md`).

### 6.3 Language entropy

- Mixed **Zig, Python, JS, shell** drivers in umbrella repos.
- **t27 countermeasure:** Critical path = **`.t27` + Rust bootstrap** only; migration spelled out in `docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md` and `docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`.

---

## 7. Authoritative directory map (this repository)

| Path | Strand | Role |
|------|--------|------|
| `specs/` | I | **Normative** t27 specifications. |
| `compiler/` | I / III | Compiler-facing `.t27` meta-specs. |
| `bootstrap/` | III | **Only** hand-written Rust for `t27c` until self-host. |
| `gen/` | III | **Generated** backend code; mirrored paths. |
| `stage0/` | II / III | Bootstrap stage markers (`FROZEN_HASH`). |
| `.trinity/seals/`, `.trinity/experience/` | II | Seals and run experience. |
| `conformance/` | I / II | Vectors (prefer spec-driven generation per `TDD-CONTRACT`). |
| `architecture/` | II | ADRs and structural decisions. |
| `docs/` | II | Architecture + law + tech tree. |
| `external/`, `research/`, `kaggle/` | *Peripheral* | Quarantine / vendor — not ring gold. |

---

## 8. Related documents (read order for new architects)

1. `docs/T27-CONSTITUTION.md` — law (SSOT-MATH, LANG-EN).  
2. `CANON.md` — rings, GOLD vs REFACTOR-HEAP.  
3. `FROZEN.md` — bootstrap seal discipline.  
4. `docs/SEED-RINGS.md` — incremental compiler pattern.  
5. `docs/TECHNOLOGY-TREE.md` — ring roadmap (may lag; prefer CANON for seal state).  
6. `docs/NUMERIC-STANDARD-001.md` — Strand I numerics.  
7. `docs/PHD-RESEARCH-PROGRAM-AND-DISSERTATION.md` — academic program & dissertation roadmap (WPs, chapters, RU/international tracks).  
8. `docs/REPO_MAP.md`, `docs/RESEARCH_CLAIMS.md`, `docs/EXTERNAL_AUDIT_PACKAGE.md` — reviewer-grade traceability and ~1h audit path.  
9. `docs/REPOSITORY_EXCELLENCE_PROGRAM.md` — hardening roadmap (P0/P1/P2).  

---

## 9. Amendments

Changes that alter **strand boundaries**, **canonical `gen/` layout**, or **bootstrap responsibilities** require an **ADR** under `architecture/` and a **ring-tagged** PR (`[GOLD-RING]`).

---

*φ² + 1/φ² = 3 | TRINITY — structure follows truth; truth lives in spec.*
