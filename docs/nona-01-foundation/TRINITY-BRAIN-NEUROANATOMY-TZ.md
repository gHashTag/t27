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
- **Brain region semantics** that must be **ring-sealed, versioned, and compiler-backed** should be expressed under **`specs/brain/`** and flow through **`tri`** (`tri parse`, `tri gen-zig`, `tri gen-c`, `tri gen-verilog`, `tri seal`, …) like every other domain.

### 1.3 Normative language: `.t27`, not application Zig

Brain semantics are authored in **T27** — the same spec language as `specs/numeric/gf16.t27` (`module`, `pub const`, `pub fn`, `test` blocks, etc.). **Zig, C, and Verilog are generated backends**, not the SSOT for brain behavior inside **this** repository.

**Principle:** there must be **no handwritten `*.zig` brain SSOT in t27**; artifacts live under **`specs/brain/*.t27`** → **`tri`** → **`gen/`** (or product integration paths in **trinity**).

**Entry point:** use **`./scripts/tri`** from this repository (committed shim to the Rust `t27c` binary). A root file named `tri` may exist locally and is **gitignored** if you install a full Trinity CLI build there.

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

### 3.1 Region → spec file (examples)

| # | Region | Spec file (SSOT) | Layer | φ-weight (design doc) |
|---|--------|------------------|-------|------------------------|
| 1 | DLPFC | `specs/brain/cognitive/dlpfc.t27` | Cognitive | φ² |
| 2 | ACC | `specs/brain/cognitive/acc.t27` | Cognitive | φ |
| 3 | PCC | `specs/brain/cognitive/pcc.t27` | Cognitive | 1/φ |
| … | … | `specs/brain/cognitive/*.t27` | Cognitive | … |
| 10 | Amygdala | `specs/brain/limbic/amygdala.t27` | Limbic | φ |
| 11 | Hippocampus | `specs/brain/limbic/hippocampus.t27` | Limbic | φ² |
| … | … | `specs/brain/limbic/*.t27` | Limbic | … |
| 19–27 | Brainstem | `specs/brain/brainstem/*.t27` | Brainstem | … |

---

## 4. t27 spec layout (Strand VI) — target tree (EPIC-6)

All **normative** brain logic for the t27 side lives as **`.t27`** under:

```text
specs/brain/
├── unified_state.t27          # BrainState and shared registers contract
├── cognitive_loop.t27         # Main loop (phases; wiring to regions)
├── phi_timing.t27             # φ-timing controller (phase durations)
├── api.t27                    # Brain public API (periphery-facing) — pending stable cross-module codegen
├── bus.t27                    # BrainBus inter-region messaging contract
│
├── cognitive/                 # Layer 3 — nine region specs
│   ├── dlpfc.t27
│   ├── acc.t27
│   ├── pcc.t27
│   ├── ofc.t27
│   ├── insula.t27
│   ├── prefrontal.t27
│   ├── visual_cortex.t27
│   ├── motor_cortex.t27
│   └── sacred_wave.t27
│
├── limbic/                    # Layer 2 — nine region specs
│   ├── amygdala.t27
│   ├── hippocampus.t27
│   ├── basal_ganglia.t27
│   ├── thalamus.t27
│   ├── hypothalamus.t27
│   ├── intraparietal.t27
│   ├── cingulate.t27
│   ├── vta.t27
│   └── nucleus_accumbens.t27
│
├── brainstem/                 # Layer 1 — nine region specs
│   ├── reticular_formation.t27
│   ├── locus_coeruleus.t27
│   ├── microglia.t27
│   ├── cerebellum.t27
│   ├── corpus_callosum.t27
│   ├── persistence.t27
│   ├── metrics.t27
│   ├── async_relay.t27
│   └── federation.t27
│
├── periphery/                 # Adapters (contracts only in t27)
│   ├── tri27_adapter.t27
│   ├── vsa_adapter.t27
│   ├── fpga_adapter.t27
│   └── hslm_adapter.t27
│
└── tests/
    ├── cognitive_tests.t27
    ├── limbic_tests.t27
    ├── brainstem_tests.t27
    ├── integration_tests.t27
    └── phi_coherence_tests.t27
```

**Each** `.t27` file MUST satisfy **SOUL / TDD mandate**: `test`, `invariant`, and/or `bench` as required by project law.

**Canonical examples** in-repo: `specs/brain/unified_state.t27`, `specs/brain/phi_timing.t27`, `specs/brain/bus.t27`, `specs/brain/cognitive_loop.t27`.

### 4.1 Deliverables: wrong path vs right path

| TZ mistake (do not use in t27) | Correct (SSOT) |
|--------------------------------|----------------|
| `src/brain/unified_state.zig` | `specs/brain/unified_state.t27` |
| `src/brain/cognitive_loop.zig` | `specs/brain/cognitive_loop.t27` |
| `src/brain/phi_timing.zig` | `specs/brain/phi_timing.t27` |
| `src/brain/api.zig` | `specs/brain/api.t27` |
| `src/brain/bus.zig` | `specs/brain/bus.t27` |
| `src/brain/regions/cognitive/dlpfc.zig` | `specs/brain/cognitive/dlpfc.t27` |
| `src/brain/regions/limbic/amygdala.zig` | `specs/brain/limbic/amygdala.t27` |
| `src/quantum/neuro_bridge.zig` | `specs/brain/quantum_bridge.t27` (name TBD) |
| All 27 regions as `*.zig` | All 27 as `*.t27` region specs |

### 4.2 Code generation — **`tri`** commands

From repo root, after `cargo build --release` in `bootstrap/`:

```bash
<<<<<<< Updated upstream
# Whole brain tree (path is a directory → batch under gen/{zig,c,verilog}/…)
./scripts/tri gen-zig       specs/brain
./scripts/tri gen-c         specs/brain
./scripts/tri gen-verilog   specs/brain
=======
# Whole brain tree → gen/{zig,c,verilog}/… (mirrors specs/** under out-root)
./scripts/tri gen-dir --backend zig --out-root gen/zig specs/brain
./scripts/tri gen-dir --backend c --out-root gen/c specs/brain
./scripts/tri gen-dir --backend verilog --out-root gen/verilog specs/brain
>>>>>>> Stashed changes

# Single file (Zig on stdout)
./scripts/tri gen-zig       specs/brain/unified_state.t27

./scripts/tri parse         specs/brain/unified_state.t27
./scripts/tri compile       specs/brain/unified_state.t27 -o /tmp/out.zig
./scripts/tri compile-project --backend zig --output build

# Seal (verify / save)
./scripts/tri seal          specs/brain/unified_state.t27 --verify
./scripts/tri seal          specs/brain/unified_state.t27 --save
<<<<<<< Updated upstream
./scripts/tri skill-seal    specs/brain/unified_state.t27
=======
>>>>>>> Stashed changes

# Conformance JSON check (full repo scan)
./scripts/tri validate-conformance

# Full compiler test suite (parse / gen / seal / fixed point)
./scripts/tri test
```

<<<<<<< Updated upstream
**Implementation note:** `scripts/tri` is an **exec shim** (`t27c --repo-root <repo> …`). **`t27c`** is equivalent when **`--repo-root`** is set.
=======
**Implementation note:** `scripts/tri` is an **exec shim**: it runs `t27c --repo-root <repo> …` (override binary with **`TRI_T27C`**). **`./scripts/tri`** is the canonical entry from repo root; **`t27c`** is equivalent when **`--repo-root`** is set.
>>>>>>> Stashed changes

**Generated layout (target):** directory arguments write under `gen/zig/…`, `gen/c/…`, `gen/verilog/…` mirroring `specs/**` — **never edit generated files by hand**.

**Note:** `api.t27` that `use`s other brain modules is **parser-valid**; the bootstrap Zig backend may still need fixes for qualified types and `[]const u8` before `api.t27` can land as a first-class generated module. Until then, keep API contracts in comments or split stubs that `gen` accepts.

**Conformance** (evidence vectors), analogous to existing JSON suites:

- `conformance/brain_*.json` — φ-timing, bus, loop, and region-critical behaviors.

### 4.3 TZ string replacements (`t27c` → `tri`)

| Was (wrong in TZ) | Use instead |
|-------------------|-------------|
| `t27c gen-zig` | `tri gen-zig` (this repo: `./scripts/tri gen-zig`) |
| `t27c gen-c` | `tri gen-c` |
| `t27c gen-verilog` | `tri gen-verilog` |
| `t27c gen` | `tri gen-zig` (single file) or `tri gen` (same Zig backend) |
<<<<<<< Updated upstream
| `t27c seal --save` | `tri seal <file.t27> --save` or `tri skill-seal <file.t27>` |
=======
| `t27c seal --save` | `tri seal <file.t27> --save` |
>>>>>>> Stashed changes
| `t27c validate-conformance` | `tri validate-conformance` |
| `./bootstrap/target/release/t27c` | `tri` (via `./scripts/tri`) |

### 4.4 `tri brain` (planned product / charter CLI)

In **t27**, `./scripts/tri brain` (→ **`t27c brain`**) exits with a pointer to this doc. Target **product** surface:

```bash
tri brain status
tri brain cycle --once
tri brain cycle --count 10
tri brain map
tri brain map --phi
tri brain regions
tri brain coherence
tri brain connectivity
tri brain benchmark --full
tri brain evolve --scenario baseline --cycles 1000
```

### 4.5 PHI LOOP (example — skills live in product / registry)

```bash
tri skill begin --issue 501
tri spec edit specs/brain/cognitive/dlpfc.t27
tri skill seal --hash
tri gen
tri test
tri verdict --toxic
tri experience save
tri skill commit
tri git commit -m "feat(brain): DLPFC spec — Closes #501"
```

<<<<<<< Updated upstream
Only what **`t27c`** implements applies in this repo (`gen`, `skill-seal`, `test`, …); **`tri skill …`** lines above are **charter / Trinity app** wiring, not the exec shim.
=======
Only the subset forwarded by `scripts/tri` to **`t27c`** works here today (`gen`, `gen-dir`, `seal`, `test`, …); product **`tri skill …`** / **`tri verdict`** lines above are **charter / Trinity app** wiring, not this shim.
>>>>>>> Stashed changes

---

## 5. Epics (summary)

| # | Epic | Priority | t27 focus | Trinity (`trinity`) app focus |
|---|------|----------|-----------|-------------------------------|
| 1 | Unified brain state | P0 | `unified_state.t27`, `bus.t27` | Adapters until codegen covers runtime |
| 2 | φ-cognitive loop | P0 | `cognitive_loop.t27`, `phi_timing.t27` | Loop scheduler, arousal modulation |
| 3 | Neuroanatomical mapping | P1 | Doc tables + spec headers | `NEUROANATOMY.md`, connectivity JSON |
| 4 | Quantum–neuro bridge | P1 | `specs/brain/quantum_bridge.t27` + hooks to `specs/vsa/`, numeric specs | Generated bridge + benchmarks in product |
| 5 | Brain as core router | P0 | `api.t27` + `periphery/*.t27` contracts | TRI-27 / VSA / FPGA consume **generated** brain APIs |
| 6 | File reorganization | P1 | Full tree under `specs/brain/**` (§4) | **trinity** integrates `gen/zig/brain/**` (or shims); **no** new handwritten brain Zig SSOT in **t27** |
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
| 33 | Core brain specs: `unified_state`, `cognitive_loop`, `phi_timing`, `bus`; `api` when codegen ready | IN PROGRESS |
| 34 | L1 brainstem — nine `.t27` region specs | TODO |
| 35 | L2 limbic — nine `.t27` region specs | TODO |
| 36 | L3 cognitive — nine `.t27` region specs | TODO |
| 37 | Full `gen` / `gen-c` / `gen-verilog` for all 27 + CI green | TODO |
| 38 | Brain conformance JSON + seal coverage | TODO |
| 39 | Timing-critical Verilog targets (FPGA) | TODO |

Ring discipline follows `docs/nona-01-foundation/GOLDEN-RINGS-CANON.md` and `bootstrap/stage0/FROZEN_HASH` when touching the bootstrap compiler.

---

## 8. φ invariants (CI-checkable)

The following are **engineering constraints** (to be encoded in tests / conformance, not prose-only):

| ID | Invariant |
|----|-----------|
| INV-1 | Σ(phase_durations) / base_ms = 3.0 ± ε (use **float** phase ms for exact TRINITY; integer truncation per phase may sum to slightly less than `3 × base_ms`) |
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
- `tri parse` / `tri gen-zig` / `tri gen-c` / `tri gen-verilog` / `tri seal` succeed for brain specs in CI (`tri test` → `t27c suite`).
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
