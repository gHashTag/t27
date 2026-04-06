# Trinity experience exchange — architecture (draft)

**Status:** Design note — aligns agent/Queen memory with multi-agent learning literature.  
**CLI spine:** Prefer **`./scripts/tri …`** for build gates and suite; `t27c` is the bootstrap binary behind `tri`.

## Executive summary

The repo already has seeds of an experience system under **`.trinity/`** (experience episodes, verdict history, event logs, amygdala/queen logs). Many episode files are sparse, queen logs are often heartbeat-only, and there is no structured inter-agent exchange or insight voting yet.

Target properties (from published multi-agent learning work):

1. **Experiential abstraction** — accumulate trials, distill natural-language insights, retrieve similar successes at inference (ExpeL-style).
2. **Verbal reinforcement** — map scalar or binary outcomes to short reflections and store them for the next attempt (Reflexion-style).
3. **Prioritized replay** — when sampling past joint behavior for training or briefing, weight by regret and multi-agent context (MAC-PO-style).

## ExpeL (AAAI 2024) — how it works

**ExpeL** trains LLM **agents from their own experience without changing model weights** (no fine-tuning of parameters). It is a peer-reviewed line of work presented at **AAAI 2024** (a top-tier AI venue), including contributions from groups such as **Tsinghua University**.

**Three stages:**

1. **Experience collection** — the agent solves tasks by trial and error, storing each attempt (success or failure) as a natural-language **episode**.
2. **Insight extraction** — from accumulated episodes the system distills reusable **rules** in text (e.g. “if the formula’s error is > 0.1%, reject the hypothesis”).
3. **Recall at inference** — on a new task the agent retrieves **relevant insights** and **similar past successes** and injects them as context.

**Why it matters for Trinity:** reported results show **performance rising as experience accumulates** — more tasks solved → better behavior on new ones. Trinity already has ExpeL-shaped pieces (**`episodes/*.json`**, **`DELTA-001.md`**) but many episodes are still **empty** (`learnings:[]`, `mistakes:[]`), so the loop does not yet deliver that gain.

**Analogy:** ExpeL is like keeping an engineering **mistake journal** and rereading it before the next task. Without it, the same engineer starts from zero every time.

**Comparison (high level):**

| Approach | Mechanism | Fine-tuning? | Trinity today |
|----------|-----------|--------------|----------------|
| **ExpeL** | Collect episodes → extract rules → recall | No | Partly (episodes exist; **insight voting** still missing) |
| **Reflexion** | Failure → verbal reflection → retry with memory | No | No rich **`reflection`** field in episodes yet |
| **Fine-tuning** | Gradient updates on data | Yes (expensive) | Not the default path for API-hosted models |

Project competitive-intelligence notes singled out **voting (upvote / downvote)** as the key next step: the **ExpeL-style episode container** is there; it needs **real episode content** plus **voting** so strong rules rise and weak ones die.

## What already works (keep)

- **`DELTA-001.md`** — falsified hypothesis documented with measured error and surviving claims.
- **Structured mistake records** (e.g. `wrong-repo-agent-manual.json`) — concrete steps, toxicity, fixes.
- **`verdict_history.json`** — scored verdicts with build/test/spec dimensions.

## Critical gaps

| Gap | Symptom | Direction |
|-----|---------|-----------|
| Episodic depth | `learnings:[]`, `mistakes:[]` | Richer **episode schema** with reflection + links to issues |
| Queen memory | Heartbeat-only lines | **Wisdom store** (append-only rules with evidence counts) |
| Insight quality | No peer signal | **Voting lifecycle** on insights (draft → active / rejected) |
| Cross-agent transfer | Siloed logs | **PUSH / PULL / BROADCAST** protocol (agent → Queen → agents) |

## Target episode record (v2 direction)

Episodes should be JSON (or JSONL) with at least:

- **Identity:** `episode_id`, `issue_number`, `agent_id`, `ring`, timestamps, duration.
- **Outcome:** `outcome`, verdict score/level.
- **Reflection:** `what_happened`, `root_cause`, `what_worked`, `what_failed`, `prevention_rule`, `applies_to[]`.
- **Insights:** list with `id`, `text`, `confidence`, votes, `tags`.
- **Knowledge push:** optional structured rule for Queen aggregation (`pattern_key`, `severity`, …).
- **Fitness:** build/test/spec/time/PR flags.
- **Cross-refs:** related episodes, DELTA/SIGMA/OMEGA doc IDs.

Exact **JSON Schema** and CI validation are implementation tasks (see epics below).

## Queen knowledge store (direction)

Append-only **`wisdom.jsonl`** (or equivalent) with:

- Rule `key`, natural-language `rule`, `category`, `confidence`, evidence count, votes, domains, `status` (`active` / `superseded` / `under_review`).

**Briefing** artifacts (per agent, per task): top rules, similar past episodes, risk warnings — generated before PLAN phase.

## Inter-agent protocol (three channels)

1. **PUSH** — after non-routine episodes, send `knowledge_push` to Queen.
2. **PULL** — before task start, agent receives briefing JSON.
3. **BROADCAST** — TOXIC or mandatory policy changes fan out (with acknowledgement where required).

## Insight voting (ExpeL-style lifecycle)

States: **DRAFT → PROPOSED → VOTED → ACTIVE / REJECTED → SUPERSEDED**.  
Append votes to **`insights_voting.jsonl`** (`insight_id`, `agent`, `action`, `timestamp`, optional `reason`).  
Thresholds (e.g. ≥3 upvotes to promote) are policy — encode in Queen logic, not hard-coded in scattered scripts.

**CLI (future):** e.g. `tri insight vote --up INS-001` — today `tri` forwards only to implemented `t27c` subcommands; new verbs belong on the same spine once specified.

## Failure taxonomy (for search and aggregation)

Examples: **FT-TYPE**, **FT-REPO**, **FT-API**, **FT-SPEC**, **FT-LOGIC**, **FT-SCOPE**, **FT-CIRC**, **FT-PHY**.  
Use consistent codes in reflections and amygdala records.

## Amygdala (structured fear/reward)

Replace tag-only lines with objects: `trigger`, `context`, `response`, `rule_generated`, `intensity`, `ttl`, `tags`.

## Document series next to DELTA

- **DELTA/** — falsified hypotheses (existing pattern).
- **SIGMA/** — confirmed claims with replication count and reproduction commands.
- **OMEGA/** — open questions and planned experiments.
- **PHI/** — φ-ratio-specific confirmations (Trinity-specific).

## Implementation epics (tracking)

| Epic | Scope |
|------|--------|
| 1 | JSON Schema for episode v2 + CI validation |
| 2 | Queen `wisdom.jsonl` + aggregator + briefing generator |
| 3 | IPC spec (PUSH/PULL/BROADCAST) + conformance tests |
| 4 | `insights_voting.jsonl` + lifecycle + `tri insight …` commands |
| 5 | Reflexion pipeline + amygdala v2 + auto rule promotion |
| 6 | SIGMA/OMEGA/PHI templates + doc lint |
| 7 | Issue-close → episode automation + `tri episode create --issue N` |

## Evaluation metrics (when implemented)

- Issue reopen rate (same failure class).
- Verdict score trend (SOLID share).
- Time-to-success by task category.

## Scientific claims discipline

Tag claims as **EXACT**, **MEASURED**, **APPROXIMATE**, **FALSIFIED**, **CONJECTURAL**, or **OPEN**; every SIGMA/DELTA doc should list a **reproduction command** using the **`tri`** surface where applicable (e.g. `tri test`, future `tri bench …`).

## References (literature)

- ExpeL — experiential learning with insight extraction without weight updates (**AAAI 2024**; e.g. Tsinghua-led work in that line).
- Reflexion — verbal reinforcement and episodic memory for retry (2023).
- MAC-PO — prioritized replay in multi-agent settings (AAMAS 2023).

---

*This file is English-first for CI (`scripts/check_first_party_doc_language.py`). For coordination anchors and daily status, see **`docs/NOW.md`** and issue **#141**.*
