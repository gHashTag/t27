# Science vs operations — dual-track synthesis (multi-model review)

**Status:** Meta-note — aggregates convergent recommendations. English-only.  
**Related:** `[KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md](KERNEL-PLAN-MULTI-MODEL-SYNTHESIS.md)`, `[RESEARCH_WRITING_T27.md](RESEARCH_WRITING_T27.md)`, `[NOW.md](NOW.md)`.

---

## 1. High agreement


| Finding                                                                                        | Consensus | Notes                                                                                                                                                                                    |
| ---------------------------------------------------------------------------------------------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Scientific skeleton** — IMRaD (or equivalent) for citable, reviewable claims                 | Strong    | Introduction / Methods / Results / Discussion maps to reproducible engineering reports. See university writing guides on IMRaD (e.g. library research guides).                           |
| **Minimize TCB** — small trusted kernel + explicit gates                                       | Strong    | Same pattern as verified compilers/OS: trust scales with **small** checkable core + deterministic tooling.                                                                               |
| **Numeric φ / floats** — reuse established FP formalisms                                       | Strong    | Prefer **[Flocq](https://flocq.gitlabpages.inria.fr/)** (Rocq/Coq) or equivalent for **specification** of tolerances and formats; avoid ad-hoc “home-grown FP math” in proof statements. |
| **E2E closed loop** `seed.t27 → t27c gen → zig test → GREEN` as **first evidence-grade proof** | Strong    | Matches **NOW.md** critical gap; “tests as truth gate” for the whole stack.                                                                                                              |
| **Reproducibility protocol** — pinned env, deterministic codegen, idempotent gen               | Strong    | IMRaD *Methods* ↔ pinned compiler, locked seeds, CI logs as artifacts.                                                                                                                   |


---

## 2. Divergence (organizational, not technical)


| Topic                                   | Variant A                                                                 | Variant B                                                             | Resolution                                                                                                                                    |
| --------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **What “execute the plan” means first** | Architecture / publication deliverables (RFC, ADR, claim tiers) then code | Close **E2E** gap immediately; methodology in parallel                | **Compatible:** treat E2E CI as the first *Methods + Results* artifact; ADRs document the same boundary.                                      |
| **NOW vs paper track**                  | Strict split: IMRaD docs vs ops                                           | **NOW as live structured abstract**; paper = ring-boundary **freeze** | Aligns with “replace NOW at ring boundary” — export a frozen report without duplicating SSOT.                                                 |
| **φ / constants — epistemology**        | Claims registry + falsifiability tiers                                    | Conformance + tolerance **primary**; paper comparative                | Use **both:** `[CLAIM_TIERS.md](nona-03-manifest/CLAIM_TIERS.md)` + `conformance/` harness; state “empirical approximation” where applicable. |


---

## 3. Unique ideas to preserve


| Idea                                                                                                           | Why it matters                                                                                                           |
| -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| **NOW ≈ structured abstract + ops log**                                                                        | One write path for agents; ring freeze yields paper sections without rewriting history.                                  |
| **Explicit TCB boundary** — `t27c` + `tri` shims + CI gates vs **process laws** (issue gate, no hand-edit gen) | Rocq/Coq proofs are only meaningful relative to a **defined** trusted core; org policy stays outside the proof artifact. |


---

## 4. Recommendations (actionable)

1. **Research writing pack** — keep a short repo guide: `**[RESEARCH_WRITING_T27.md](RESEARCH_WRITING_T27.md)`** (IMRaD mini-template + reproducibility checklist + link to claim tiers).
2. **Ring blocker** — ship **advertised E2E** `seed → gen → zig test` in CI; mirror status in **NOW §3.2**.
3. **Trusted kernel boundary** — document in one place (this synthesis + `[KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md)` + `[T27_KERNEL_FORMAL_COQ.md](T27_KERNEL_FORMAL_COQ.md)`): what is **in-TCB** vs **process law**.
4. **Numeric formalization path** — when extending `coq/`, plan **Flocq**-aligned models for float/tolerance claims instead of raw `Reals` only.

---

*Secondary links from informal uploads are intentionally omitted; prefer HAL/Inria, OPAM package pages, and institutional IMRaD guides.*