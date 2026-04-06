# Trinity S³AI — Neuroanatomical brain architecture (technical charter)

**Version:** 1.0  
**Date:** 2026-04-06  
**Status:** DRAFT → REVIEW  
**Priority:** P0 — core architecture (coordination document)

This file is the **cross-repo charter** for unifying Trinity’s neuroanatomical “brain” layers. It applies to **ecosystem planning**. **Normative product math and behavior** for the t27 language remain **`*.t27` specs** in this repository, per **SSOT-MATH** and **De-Zigfication** (see `docs/T27-CONSTITUTION.md`, `architecture/ADR-005-de-zig-strict.md`).

---

## 1. Context

### 1.1 Trinity application repository (`gHashTag/trinity`)

Today, **Trinity** (not this repo) may contain **two parallel implementations**:

| Track | Typical location | Role |
|-------|------------------|------|
| Swarm / coordination | `src/brain/` | Task claims, event bus, telemetry, federation (~23 modules, large test count) |
| Cognitive | `src/tri/` | Amygdala, hippocampus, thalamus, DLPFC, PCC, OFC, ACC, etc. (~10 modules, large test count) |

**Problem:** disjoint types, import paths, and **no shared canonical state** — two “brains,” one product.

### 1.2 This repository (`gHashTag/t27`)

**t27** is the **spec-first** language and compiler corpus:

- **Source of truth:** `.t27` / `.tri` specifications.
- **Zig / C / Verilog:** generated under `gen/`, not hand-written application logic (LAW 1 / De-Zigfication).

Therefore:

- **Hand-maintained `src/brain/*.zig` as SSOT is out of scope for t27** and is **technical debt** if it duplicates semantics that should be specified here.
- **Brain region semantics** that must be **ring-sealed, versioned, and compiler-backed** should be expressed under **`specs/brain/`** and flow through **`t27c parse` / `gen-*` / `seal`** like every other domain.

---

## 2. Goals (unified architecture)

1. **Single coherent brain model** — one shared state and messaging model, not two silos.
2. **Brain as core router** — TRI-27 ISA, VSA, GF16, FPGA, CLARA, federation attach as **periphery** with explicit APIs.
3. **φ-structured topology** — connectivity and phase timing use **golden-ratio constraints** as **testable engineering invariants**, not decoration.
4. **Neuroanatomical grounding** — each region maps to a biological analogue with citable references (see §6).

---

## 3. Layer model — 27 = 3³ regions

Three layers × nine regions each (names are **stable identifiers** for specs and generated modules):

| Layer | Role | Regions (identifiers) |
|-------|------|-------------------------|
| **L3 Cognitive** | Planning, conflict, self-model, reward, interoception, integration, perception, action, coherence | `dlpfc`, `acc`, `pcc`, `ofc`, `insula`, `prefrontal`, `visual_cortex`, `motor_cortex`, `sacred_wave` |
| **L2 Limbic** | Salience, memory formation, action selection, relay, homeostasis, parietal integration, cingulate, reward DA, valuation | `amygdala`, `hippocampus`, `basal_ganglia`, `thalamus`, `hypothalamus`, `intraparietal`, `cingulate`, `vta`, `nucleus_accumbens` |
| **L1 Brainstem** | Arousal, vigilance, immune metaphor, adaptive timing, commissure, persistence, metrics, async IO, federation | `reticular_formation`, `locus_coeruleus`, `microglia`, `cerebellum`, `corpus_callosum`, `persistence`, `metrics`, `async_relay`, `federation` |

**Trinity identity check:** φ² + 1/φ² = 3 — used as a **design constraint** for phase weights and documented invariants (§8).

---

## 4. t27 spec layout (Strand VI) — target tree

All **normative** brain logic for the t27 side lives as **`.t27`** under:

```text
specs/brain/
├── unified_state.t27      # shared BrainState / registers contract
├── cognitive_loop.t27     # sense → evaluate → decide → act → consolidate
├── phi_timing.t27         # phase durations; ties to GoldenFloat / constants
├── api.t27                # periphery-facing brain API contract
├── bus.t27                # inter-region messaging contract
├── cognitive/             # L3 — nine region specs
├── limbic/                # L2 — nine region specs
└── brainstem/             # L1 — nine region specs
```

**Each** `.t27` file MUST satisfy **SOUL / TDD mandate**: `test`, `invariant`, and/or `bench` as required by project law.

**Generated artifacts** (no hand edit):

- `gen/zig/brain/…`, `gen/c/brain/…`, `gen/verilog/brain/…` — via `t27c gen-zig`, `gen-c`, `gen-verilog`.

**Conformance** (evidence vectors), analogous to existing JSON suites:

- `conformance/brain_*.json` — φ-timing, bus, loop, and region-critical behaviors.

---

## 5. Epics (summary)

| # | Epic | Priority | t27 focus | Trinity (`trinity`) app focus |
|---|------|----------|-----------|-------------------------------|
| 1 | Unified brain state | P0 | `unified_state.t27`, `bus.t27` | Adapters until codegen covers runtime |
| 2 | φ-cognitive loop | P0 | `cognitive_loop.t27`, `phi_timing.t27` | Loop scheduler, arousal modulation |
| 3 | Neuroanatomical mapping | P1 | Doc tables + spec headers | `NEUROANATOMY.md`, connectivity JSON |
| 4 | Quantum–neuro bridge | P1 | Spec hooks to `specs/vsa/`, numeric sacred specs | Bridge modules + benchmarks |
| 5 | Brain as core router | P0 | `api.t27` + periphery contracts | TRI-27 / VSA / FPGA adapters |
| 6 | File reorganization | P1 | `specs/brain/**` only | `src/brain/regions/{cognitive,limbic,brainstem}/` **generated or shim** |
| 7 | Testing & validation | P0 | parse/gen/seal + conformance | Integration + legacy test port |
| 8 | CLI & visualization | P2 | spec for CLI surface if needed | `tri brain …` commands, docs site |

---

## 6. Literature anchors (per region — examples)

Formal citations belong in spec comments and `docs/NEUROANATOMY.md` (to be added in **trinity** or **t27/docs** as the charter matures). Examples:

| Region | Biological analogue | Example references |
|--------|---------------------|------------------|
| Amygdala | Fear / salience | LeDoux; Phelps & LeDoux |
| Hippocampus | Episodic / spatial memory | Eichenbaum; Moser et al. |
| Basal ganglia | Action selection | Graybiel; Yin & Knowlton |
| DLPFC | Cognitive control | Miller & Cohen |
| ACC | Conflict monitoring | Botvinick et al. |
| PCC | Default mode | Raichle |
| Thalamus | Relay | Sherman & Guillery |
| Locus coeruleus | Arousal / NE | Aston-Jones & Cohen |

---

## 7. SEED rings — brain tranche (proposal)

| Ring | Capability | Status |
|------|------------|--------|
| 33 | Core brain specs: `unified_state`, `cognitive_loop`, `phi_timing`, `api`, `bus` | TODO |
| 34 | L1 brainstem — nine `.t27` region specs | TODO |
| 35 | L2 limbic — nine `.t27` region specs | TODO |
| 36 | L3 cognitive — nine `.t27` region specs | TODO |
| 37 | Full `gen-*` for all 27 + CI green | TODO |
| 38 | Brain conformance JSON + seal coverage | TODO |
| 39 | Timing-critical Verilog targets (FPGA) | TODO |

Ring discipline follows `docs/GOLDEN-RINGS-CANON.md` and `bootstrap/stage0/FROZEN_HASH` when touching the bootstrap compiler.

---

## 8. φ invariants (CI-checkable)

The following are **engineering constraints** (to be encoded in tests / conformance, not prose-only):

| ID | Invariant |
|----|-----------|
| INV-1 | Σ(phase_durations) / base_ms = 3.0 ± ε |
| INV-2 | \|regions\| = 27 = 3³ |
| INV-3 | \|layers\| = 3 |
| INV-4 | Regions per layer = 9 each |
| INV-5 | Aggregate connectivity statistic bands tied to φ (spec-defined) |
| INV-6 | φ-coherence metric stable after N cycles (spec-defined threshold) |
| INV-7 | Phase duration ratios (decide/sense) ≈ φ² within tolerance |
| INV-8 | Phase duration ratios (evaluate/consolidate) ≈ φ within tolerance |

Exact ε and definitions of “coherence” and “connectivity statistic” are **specified in `.t27`**, not only in this markdown file.

---

## 9. Acceptance criteria (condensed)

**Must (P0)**

- Single **spec-defined** brain state and loop contracts in **`specs/brain/`**.
- φ phase sum invariant **tested**.
- `t27c parse` / `gen-*` / `seal` succeed for brain specs in CI.
- No new hand-written Zig SSOT for those semantics in **t27** (`gen/` only).

**Should (P1)**

- `NEUROANATOMY.md` + connectivity artifact (JSON or `.t27` module).
- Periphery adapters specified in `api.t27` and tested via conformance.

**Could (P2)**

- CLI / visualization — product repo or separate tool, linked from docs.

---

## 10. Risks

| Risk | Mitigation |
|------|------------|
| Type mismatch between legacy `src/brain` and `src/tri` | Adapter layer in **trinity**; **t27** owns types in `.t27` only |
| Performance regression | Benchmarks in `bench` blocks + conformance latency vectors |
| Over-full loop every tick | Spec-defined **conditional activation** of regions |

---

## 11. Timeline

Rough order: **state + bus → φ timing + loop → regions by layer → gen/conformance → FPGA**. **~10 weeks** is a reasonable MVP horizon for the **combined** program; **t27** landings should be **incremental PRs** per ring, each with Issue Gate and green CI.

---

**φ² + 1/φ² = 3 | TRINITY**
