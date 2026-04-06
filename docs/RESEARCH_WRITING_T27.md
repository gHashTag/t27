# Research writing — T27 skill pack (IMRaD + reproducibility)

**Status:** Process guide for humans/agents. English-only.  
**Templates:** EXP block inside [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md); ring freeze from [`NOW.md`](NOW.md).

## 1. IMRaD skeleton (mini-paper or section)

| Section | Answer this |
|---------|-------------|
| **Introduction / Context** | What gap? What does theory or prior work predict? |
| **Methods** | Exact inputs, toolchain versions, commands, acceptance criteria. |
| **Results** | Numbers, CI verdicts, tables — facts only. |
| **Discussion** | Meaning, limits, falsifiability, next ring. |

Use **parallel structure** inside sections (easier for reviewers and agents).

## 2. Reproducibility checklist (bind to CI)

- [ ] **Compiler / runner pinned** — `t27c` built from named commit; Zig version recorded in CI log or doc.  
- [ ] **Commands copy-paste** — `cargo build`, `./scripts/tri test`, etc., as in [`CONTRIBUTING.md`](../CONTRIBUTING.md).  
- [ ] **Artifacts named** — issue `#N`, branch, seal paths under `.trinity/seals/`.  
- [ ] **Codegen idempotency** — same spec + same compiler → stable output (policy in [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md)).  
- [ ] **Claim tier** — tag statements per **[`CLAIM_TIERS.md`](nona-03-manifest/CLAIM_TIERS.md)** (exact / measured / conjecture / …).

## 3. NOW as “structured abstract”

For each material task completion, update **`docs/NOW.md`**:

- **Context** — §1 purpose + current milestone.  
- **Methods / state** — §3 tables (counts, gaps).  
- **Results** — what is green/red in CI (link workflow badges).  
- **Discussion / next** — §5 plan + open gap (E2E pipeline).

On **ring boundary**, freeze a snapshot (commit + tag or export) suitable for a longer report without duplicating the SSOT.

## 4. φ and floating-point language

- **Algebraic identities** (e.g. φ² = φ + 1) — separate from **IEEE float checks**.  
- In prose, distinguish **exact algebraic** vs **empirical approximation** vs **conformance vector verdict**.  
- Formal Rocq path: prefer **[Flocq](https://flocq.gitlabpages.inria.fr/)** for FP semantics when proofs touch floats — see [`T27_KERNEL_FORMAL_COQ.md`](T27_KERNEL_FORMAL_COQ.md).

## 5. One-page verdict line

End every EXP / ring note with:

`Verdict: CLEAN | TOXIC | PARTIAL — <single sentence>.`

---

*IMRaD is widely taught in research-writing guides (university libraries); no single URL is normative for the whole project.*
