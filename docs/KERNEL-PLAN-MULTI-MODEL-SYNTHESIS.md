# Multi-model synthesis — kernel plan, pipeline, and experience CI

**Status:** Meta-note — consolidates convergent recommendations from independent model reviews (no single vendor truth). English-only.  
**Normative docs:** [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md), [`TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md`](TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md), [`SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md`](SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md) (IMRaD + TCB + Flocq + NOW/paper), [`RESEARCH_WRITING_T27.md`](RESEARCH_WRITING_T27.md), [`NOW.md`](NOW.md).

---

## 1. Where reviews agreed

| Finding | Consensus | Evidence / note |
|---------|-----------|------------------|
| Close the **E2E gap** `seed.t27 → t27c gen → zig test → GREEN` in CI first (Phase −1 / step 0.0) | ✓ ✓ ✓ | Treat as the main missing piece called out in **NOW**; blocks credible claims about axioms and experience until the loop is reproducible. |
| **Separate layers** — do not fold **process laws** into **mathematical / semantic axioms** (keep **K5/K6** distinct from **K1–K4**) | ✓ ✓ ✓ | Mixing weakens formality; process belongs in constitution / invariant tables. |
| **Shift-left CI** for new formats (schema, episodes) — do not defer gates to “Phase 4 only” | ✓ ✓ ✓ | Late validation lets invalid episodes accumulate and poisons Tier 3 (Queen semantic). Analogous to IMRaD: instrument *before* bulk data collection. |
| **Theorem / claim discipline** — replace bare “□” sketches with explicit **status** and proof obligations | ✓ ✓ ✓ | Distinguish what is proved, what is rigorously argued, and what is conjecture. |
| **Version protocols** — at minimum **`schema_version`** on episodic / insight JSON | ✓ ✓ ✓ | Avoids silent breakage when formats evolve (v4 vs v3 migrations). |

---

## 2. Where reviews diverged

| Topic | Typical emphasis | Why it differs |
|-------|------------------|----------------|
| **Next “core” axioms** | (A) Error model + termination / decidability · (B) Compositionality + effect tracking + capability isolation · (C) Closing the **Markdown vs `.t27` / `t27c`** executable gap + CI metrics | Different lenses: computability vs semantic modularity & safety vs operational wiring. |
| **DELTA / SIGMA / OMEGA logs** | (A) Standalone artifacts · (B) Prefer **views** over a single experience SSOT · (C) Less emphasis | Trade-off: publication-friendly docs vs duplicate source of truth. |
| **Promoting insight → Tier 4** | (A–B) Higher bar, RFC / consensus / escalation · (C) Stress **cold start** (bootstrap Tier 3 from history before debating promotion) | Governance detail vs filling an empty semantic layer first. |

*These branches are not mutually exclusive; they argue for a **staged** formal core plus explicit **machine-checkable surface**.*

---

## 3. Unique ideas worth keeping

| Source (role) | Idea | Why it matters |
|----------------|------|----------------|
| Review A | **Axiom independence / consistency posture** — state what is *not* claimed (e.g. limits of completeness arguments) | Avoids false rigor; makes external review honest. |
| Review A | **Trust chain / “trusting trust”** as its own plan layer | Compilers and codegen need an explicit trust story, not only on-paper theorems. |
| Review B | **Queen cold start** — bootstrap Tier 3 from historical ring / repo logs (e.g. early rings) | Improves ROI before dozens of new v3 episodes exist. |
| Review B | **KPIs** for the experience system (not only “N insights”) | Measures error repetition, time-to-green, review load — not vanity counts. |

---

## 4. Synthesis

**High confidence.** The plan should be **reordered** so the first externally visible milestone is a **demonstrated, CI-enforced** path: minimal golden spec → `t27c gen` → **`zig test` green**. Until that exists, axioms and theorems read as **declarations without executable consequence**.

**Layering.** **K1–K4** = semantic / mathematical / trusted-kernel story. **K5–K6** = **process laws** (issue gate, spec→gen) — align with **NOW** / **SOUL** / workflows; keep vocabulary distinct.

**Shift-left.** When a new JSON schema for episodes appears, ship a **validator + CI warning/fail** in the **same** ring band as the schema, so Tier 3 is not trained on garbage.

**Formal roadmap (non-exclusive).** (a) Minimal **machine-checked or test-checked** surface; (b) extensions for effects, errors, termination as the language matures; (c) governance for promoting insights (quorum / RFC).

---

## 5. Recommended three-move sequence

1. **Phase −1 — E2E pipeline proof** — advertise and enforce `seed.t27` (or simpler) → gen → `zig test` in **GitHub Actions**; treat as a **merge blocker** for dependent epics.  
2. **Document split** — **Math axioms** · **Architectural invariants** · **Process laws** (migrate **K5/K6** wording into the laws column).  
3. **Phase 1 + CI** — episode / insight **`schema_version`**, early **`experience-gate`** (warn then fail), then Queen semantic layer with **cold-start import** + **theorem/claim statuses** (**FORMAL** / **RIGOROUS** / **ENGINEERING** / **CONJECTURE**).

---

*No secondary “citation” links are included here; cite peer-reviewed sources (DOI / arXiv / proceedings) in PRs when asserting paper claims.*
