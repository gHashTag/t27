# Competitive strategy and Ring 999 horizon (t27)

**Document type:** Strategy memo — **English only** (per `[docs/T27-CONSTITUTION.md](T27-CONSTITUTION.md)` Article **LANG-EN**).  
**Date:** 2026-04-06  
**Normative gates:** Article **RING-LAW** (one ring = one capability; horizon vs binding batches), Article **COMPETITION-READY** (when “competitive” language is allowed).

---

## Executive summary (planning; Article COMPETITION-READY)

**t27** combines (1) **spec-first** compilation from **`.t27`** to **Zig**, **C**, and **Verilog**, (2) **K3 / trit**-flavored semantics and **GoldenFloat** (φ-structured numerics — see `[docs/RESEARCH_CLAIMS.md](RESEARCH_CLAIMS.md)`), and (3) seven **AR** specs under [`specs/ar/`](../specs/ar/) whose **themes** overlap public **DARPA CLARA** program materials. That **co-location** is a real architectural story; it does **not**, by itself, prove **ecosystem dominance**, **grant awards**, or **“compliance”** with any solicitation.

**CLARA (public):** Program overview [DARPA CLARA](https://www.darpa.mil/research/programs/clara); solicitation **DARPA-PA-25-07-02** [opportunity page](https://www.darpa.mil/work-with-us/opportunities/darpa-pa-25-07-02) (public framing **Feb 2026**). **Schedule:** [Amendment 1 (PDF)](https://www.darpa.mil/sites/default/files/attachment/2026-03/darpa-clara-amendment-1.pdf) — proposal due **2026-04-17**, target award **2026-06-16**, anticipated program start **2026-06-22**. **Funding caps, period of performance, Technical Areas, and outbound open-source license terms** are binding only in the **full active BAA + amendments** — not in this memo.

**Highest-leverage gaps (in-repo narrative):** publish **GoldenFloat vs takum / posit / IEEE** results under a fixed protocol (§0, Ring **#129**); complete **CLARA preparation** docs/checklists (Ring **#134**); resolve **MIT vs Apache-2.0** (or dual strategy) with **legal** review before any CLARA-class release plan.

**Repository metrics** (badges / snapshots): see `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §1.2 and `[docs/STATE_OF_THE_PROJECT.md](STATE_OF_THE_PROJECT.md)`.

---

## 0. Situational intelligence (primary sources only)

Use these for **scheduling** and **benchmark planning**; do **not** treat blogs or unrelated sites as evidence.


| Finding                                                                            | Primary reference                                                                                                                                                                               | t27 action                                                                                                                                                                 |
| ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **GoldenFloat** lacks **independent** peer bundles vs **takum** on published tasks | [ARITH 2025 proc. 215900a061.pdf](https://www.arith2025.org/proceedings/215900a061.pdf) (takum / bfloat16 sparse-solver style narrative in venue proceedings)                                   | Close gap via **documented** NMSE / solver protocol (**Ring #129**, `[docs/RESEARCH_CLAIMS.md](RESEARCH_CLAIMS.md)` **C-gf-***)                                            |
| **CLARA** schedule shifted (more time before **program start**)                    | [DARPA Amendment 1 PDF](https://www.darpa.mil/sites/default/files/attachment/2026-03/darpa-clara-amendment-1.pdf): proposals **2026-04-17**, awards target **2026-06-16**, start **2026-06-22** | Align **EPOCH-01-HARDEN** and **#134** prep; re-check BAA before submit                                                                                                    |
| **Apache-2.0** often required for CLARA-class outbound code                        | Active **BAA** + amendment (not third-party summaries)                                                                                                                                          | Legal review; README currently **MIT** — see `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §4.4–4.5              |
| **Scallop** = strong **AR** NeSy **without** t27-style **HW codegen** spine        | [ACM PLDI 2023](https://dl.acm.org/doi/10.1145/3591280)                                                                                                                                         | Position t27 on **spec → RTL** + AR **in one corpus**; avoid unmeasured “better than Scallop”                                                                              |
| **MAS adoption %** from vendor blogs                                               | *Not* used here                                                                                                                                                                                 | t27 differentiator is **normative**: **Article AGENT-DOMAIN** + **27-register** roster (`[docs/AGENTS_ALPHABET.md](AGENTS_ALPHABET.md)`), not unverified market statistics |


---

## 1. Where the science already lives

Do **not** fork the long-form math into a second SSOT. Use:


| Topic                                                               | Canonical English memo                                                                                     |
| ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Radix / E(b), radix economy, (3/2)^N, caveats                       | `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §2 |
| Trinity identity, GoldenFloat \delta_\varphi, IEEE/posit/takum, TWN | same, §3                                                                                                   |
| K3, AR specs, CLARA **alignment** (not certification)               | same, §4                                                                                                   |
| Competitor taxonomy                                                 | `[docs/COMPETITIVE_LANDSCAPE_SCIENTIFIC.md](COMPETITIVE_LANDSCAPE_SCIENTIFIC.md)`                          |
| Honest product status                                               | `[docs/STATE_OF_THE_PROJECT.md](STATE_OF_THE_PROJECT.md)`                                                  |
| Claim IDs / evidence                                                | `[docs/RESEARCH_CLAIMS.md](RESEARCH_CLAIMS.md)`                                                            |


**Non-English** competitive drafts (e.g. a Russian “999 rings” report) **must not** be added under `docs/` without Architect exception; keep them **outside** the tree or translate into English before PR.

---

## 2. Corrections to common outdated statements


| Statement                                         | Fact in this repository (2026-04-06)                                                                                                                                                                                                                                            |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| “`docs/T27-CONSTITUTION.md` does not exist / 404” | File **exists**: `[docs/T27-CONSTITUTION.md](T27-CONSTITUTION.md)`. On GitHub (default branch **master**): `https://github.com/gHashTag/t27/blob/master/docs/T27-CONSTITUTION.md`. A 404 is usually **wrong path** (missing `docs/`), **unpushed** commit, or **wrong branch**. |
| “`task.md` is canonical”                          | Root file is `**TASK.md`** with `[docs/TASK_PROTOCOL.md](TASK_PROTOCOL.md)` and Anchor issue linked from `TASK.md`.                                                                                                                                                             |
| “Marketing scorecard ✅ everywhere”                | Capability matrices in `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §6 use **guarded** labels (✓ / ~ / ✗). Article **COMPETITION-READY** lists **six** gates before external **“we win”** claims.                    |
| “No coding until the constitution exists”          | **Superseded:** `[docs/T27-CONSTITUTION.md](T27-CONSTITUTION.md)` is in-repo (v1.7+). Work proceeds under **Issue Gate**, **claims registry**, and **RING-LAW** — not a documentation blockade.                                                                                |


---

## 3. Ring 999 as vocabulary (Article RING-LAW)

- **Ring 999** (and long epoch tables) are **horizon / planning vocabulary** until adopted as a **GitHub Milestone + scoped issues** batch.  
- **Execution SSOT:** Issues (`Closes #N`), `**docs/RINGS.md`**, `**CANON.md**`, milestone **EPOCH-01-HARDEN** (example: [milestone/1](https://github.com/gHashTag/t27/milestone/1) on `gHashTag/t27`).  
- **One ring = one capability** — avoid opening hundreds of speculative issues; use `[docs/RING_BACKLOG_047_063.md](RING_BACKLOG_047_063.md)` and program issues when ready.

---

## 4. “Competition-ready” checklist (Article COMPETITION-READY)

Before grant text, DARPA-style proposals, or “we beat X” outreach, verify **all** items in **Article COMPETITION-READY** in `[docs/T27-CONSTITUTION.md](T27-CONSTITUTION.md)` (invariants, claims registry, repro/CI, Issue Gate, **TASK** protocol, honest competitor gaps).

**CLARA:** thematic **alignment** with public program goals ≠ **certification**. Use the **active BAA + amendments** (e.g. **Amendment 1**, March 2026 — link in `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §4.4) for deadlines, TA scope, and **license** terms.

---

## 5. High-impact competitive actions (low ceremony)

Aligned with open ring issues and `[docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)` §7–8:


| Action                                                          | Competitive target             | Notes                                                                                                                  |
| --------------------------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| Ship **conformance / GoldenFloat** artifacts on tagged releases | TerEffic-class, numerics peers | Tie to **RESEARCH_CLAIMS** + Zenodo per `[docs/PUBLICATION_PIPELINE.md](PUBLICATION_PIPELINE.md)`                      |
| **GF16 vs bfloat16/float16** NMSE (documented protocol)         | Takum, posit, IEEE             | Ring **#129** track; no superiority slogans until tables exist                                                         |
| `**docs/CLARA-*`** + checklist completion                       | CLARA-style programs           | Ring **#134**; license/legal reviewed separately                                                                       |
| **License** compatible with target solicitation                 | Regulators / DARPA             | **MIT** is common in tree; **Apache-2.0** may be required by a specific BAA — **legal** decision + issue, not drive-by |
| Short **phi-distance** note or preprint                         | Academia                       | Must match `**docs/RESEARCH_CLAIMS.md`** statuses                                                                      |


### 5.1 Priority order (EPOCH-01-HARDEN slice, issue-backed)

**Execution SSOT** remains **GitHub issues + milestone**, not this list. For **competitive** urgency, close **dependencies** roughly as:

`#127` (**TASK.md** / protocol) → `#128` (**Issue Gate** CI) → `#131` / `#132` (seal coverage / SOUL enforcement) → `#130` (technology tree) → `#129` (GF16 / NMSE vs baselines) → `#134` (CLARA prep) → `#135`–`#139` / `#140` / `#142` as Queen schedules.

### 5.2 Superseded “first iteration” blockers

The following appeared in older competitive drafts; **do not** treat them as current gates:


| Old action                               | Status (2026-04-06)                                                                                                                                   |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Create `docs/T27-CONSTITUTION.md`        | **Done** — file exists; fix **404** on GitHub via correct path (`docs/…`), branch (**`master`**), and **push**.                                        |
| Rename `task.md` → `TASK.md`             | **Done** — root **`TASK.md`** + `[docs/TASK_PROTOCOL.md](TASK_PROTOCOL.md)`.                                                                           |
| Milestone **EPOCH-01-HARDEN**            | Track on GitHub (e.g. [milestone/1](https://github.com/gHashTag/t27/milestone/1)) — not a doc-only step.                                               |


---

## 6. Multi-agent and constitution

27-agent coordination is **governance**, not automatic advantage over CrewAI/LangGraph unless measured. See `**docs/T27-CONSTITUTION.md`** Articles **AGENT-DOMAIN**, **TASK-MD**, `**docs/AGENT_BRAIN_MAP.md`**, `**TASK.md**`, and **Anchor** coordination issue.

**Do not** cite vendor **“% of enterprises running agents”** statistics in grant or academic text unless the underlying study is primary and methodologically acceptable; t27’s differentiator here is **normative** (register-bound roster + constitution), not survey marketing.

---

## 7. One-line positioning (safe)

**t27** is a **spec-first** toolchain at the intersection of **ternary/K3-flavored semantics**, **φ-structured numerics (GoldenFloat)**, and **generated multi-backends**, with **constitutional** gates (seals, claims, Issue Gate). Uniqueness is **architectural co-location** of these axes; **empirical dominance** over every named competitor is **not** established in-repo.

---

## 8. “999 RINGS” horizon: epochs vs competitive themes (illustrative)

Per **Article RING-LAW**, long ring spans are **planning vocabulary** until backed by **milestones and issues**. The table maps **epochs** to **competitive gaps** they intend to close — ring intervals are **draft** (backlog may renumber).


| Epoch (draft name) | Indicative rings (draft) | Competitive / research theme                         |
| -------------------- | ------------------------ | ---------------------------------------------------- |
| 1 HARDEN             | 32–58                    | CI, docs, sealing, constitution, conformance hygiene |
| 2 BRAIN              | 59–85                    | ISA-linked agent governance vs abstract MAS stacks   |
| 3 NUMERIC            | 86–112                   | GoldenFloat benchmarks vs takum / posit / IEEE       |
| 4 COMPILER           | 113–139                  | IR, tooling — Chisel / MLIR class maturity           |
| 5 FPGA               | 140–166                  | spec → bitstream evidence                            |
| 6 AR / CLARA         | 167–193                  | AR pipeline + solicitation-aligned packaging         |
| 7 SELF-HOST          | 194–220                  | bootstrap / self-host depth                          |
| 8 PUBLISH            | 221–247                  | papers, DOIs, peer review                            |
| 9 SWARM              | 248–274                  | multi-agent autonomy protocols                       |
| 10 OPTIMIZE          | 275–301                  | performance vs TVM / XLA class baselines             |
| 11 INTEROP           | 302–328                  | bindings (Python / Rust / Wasm)                      |
| 12 NEURAL            | 329–355                  | native ternary NN training / inference narratives    |
| 13 FORMAL            | 356–382                  | proof artifacts (Lean / Coq class goals)             |
| 27 TRINITY³          | 734–760                  | cross-stack φ² + φ⁻² = 3 integration (symbolic)      |
| 999 ΩΩΩ              | 999                      | horizon “competition-ready” seal vocabulary          |


**Milestone examples (draft spirit):** first **documented** GoldenFloat vs takum-class table; CLARA **preparation** package ready; first **peer-reviewed** PL/compiler venue submission; **bitstream** on a stated FPGA part; **publication + Zenodo** alignment per `[docs/PUBLICATION_PIPELINE.md](PUBLICATION_PIPELINE.md)`.

---

## 9. Competition-readiness scorecard (illustrative, non-normative)

The formula below is a **heuristic dashboard** for internal prioritization — **not** constitutional law and **not** a substitute for **Article COMPETITION-READY** gates.

\[
\text{COMPETITION\_SCORE} = \bigl(
w_1 \cdot f(\text{publications}) +
w_2 \cdot \mathbb{1}[\text{CLARA package ready}] +
w_3 \cdot \mathbb{1}[\text{GF benchmarks published}] +
w_4 \cdot \mathbb{1}[\text{FPGA artifact verified}] +
w_5 \cdot g(\text{agent autonomy}) +
w_6 \cdot h(\text{external adopters})
\bigr) \times 100
\]

Choose weights \(w_i\) that sum to **1** and define \(f,g,h\) with explicit targets (e.g. papers count cap, adopters cap). A **placeholder** fill (all booleans **false**, autonomy **1/3**) yields order-of-magnitude **~5/100** — useful only as a **template**, not as a shipped metric.

---

*φ² + 1/φ² = 3 — exact as algebra; competitive speech stays **COMPETITION-READY**.*