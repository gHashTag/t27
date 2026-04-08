---
name: agent-rules
description: Universal Agent Rules Registry — Shared rules that ALL agents must follow. This file is the single source of truth for cross-agent coordination and behavior.
version: 1.0.0
---

# AGENT RULES — Universal Registry

These rules apply to ALL agents in Trinity S³AI project.

## Rule 1: KNOWLEDGE-FIRST

**Before ANY action:**
- Query existing knowledge via `agent-coord`
- Never assume blank slate
- Session amnesia causes duplicate work and wasted tokens

**Penalty:** Agents working without knowledge check may be blocked.

## Rule 2: WRAP-UP-LAST

**After ANY action:**
- Upload wrap-up via `agent-coord`
- Include: summary, decisions, files, next
- Persistent knowledge enables future agents

**Penalty:** Agents not wrapping up cause knowledge gaps.

## Rule 3: ISSUE-BINDING

**For code mutations:**
- MUST have GitHub issue reference
- Use `tri skill begin --issue N`
- Commit message: `Closes #N` or `[ref: ISSUE_N]`

**Penalty:** PRs without issue reference are blocked by Issue Gate.

## Rule 4: SPEC-FIRST

**Code changes:**
- Edit `.t27` specs first
- Generate backends: `tri gen <spec>`
- Never hand-edit generated `.zig`/`.c` files

**Penalty:** Hand-edited backends break De-Zig-fication.

## Rule 5: TEST-DRIVEN

**Spec development:**
- Include `test`, `invariant`, `bench` blocks
- Run: `tri test <spec>` before commit
- Verify: `tri verdict --toxic` for regressions

**Penalty:** Untested code causes toxic mutations.

## Rule 6: TRINITY-CANON

**Canonical hierarchy:**
- `specs/base/*.t27` — Trit types, operations
- `specs/math/*.t27` — Constants, sacred physics
- `specs/numeric/*.t27` — GoldenFloat, TF, IPS
- `specs/api/*.t27` — API contracts
- `specs/benchmarks/*.t27` — Benchmark specs
- `specs/conformance/*.t27` — E2E scenarios

**Penalty:** Wrong layer breaks architecture.

## Rule 7: CI-RESPECT

**GitHub Actions:**
- All PRs must pass issue-gate, now-sync-gate
- All PRs must pass phi-loop-ci
- Failing CI blocks merge

**Penalty:** CI failure is a hard stop condition.

## Rule 8: NO-SECRETS

**Security:**
- Never commit secrets in code
- Use `.env` files locally
- Check `.env.example` patterns in docs

**Penalty:** Secret exposure is critical security violation.

## Rule 9: COLLABORATIVE-EDIT

**Concurrent access:**
- Claim target before editing: `tri claim acquire <spec>`
- Respect active claims
- Use `.trinity/state/` for coordination

**Penalty:** Edit conflicts cause merge conflicts.

## Rule 10: QUEEN-HEALTH

**System integrity:**
- SacredPhysics > Numeric > Graph > Compiler > Runtime > QueenLotus
- Stop feature work if queen_health < 0.5
- Recovery-only mode for critical domains

**Penalty:** Degrading Queen Trinity is unconstitutional.

## Rule 11: TRACEABILITY

**Audit trail:**
- Every action references issue or PR number
- Every wrap-up includes files changed
- Every decision is recorded

**Penalty:** Untraceable work cannot be reviewed or reverted.

## Rule 12: AUTO-RECOVERY

**Self-healing:**
- Doctor agent monitors `.trinity/state/*`
- Health anomalies trigger recovery
- Replay from last clean seal

**Penalty:** Ignoring health degrades system integrity.

## Rule Violation Consequences

| Severity | Penalty |
|----------|----------|
| Critical (secrets, Queen health) | Immediate stop, recovery mode |
| Major (no issue, CI failure) | Block PR, require fix |
| Minor (no wrap-up, no query) | Warning, degrade reputation |
| Advisory (minor style) | Note in decision log |

## Agent Registration

New agents must register here:

```json
{
  "agent_name": "<name>",
  "agent_type": "<code|research|ci|doctor>",
  "skills_used": ["skill1", "skill2"],
  "registered": "YYYY-MM-DD"
}
```

## Skill Reference Matrix

| Agent Type | Required Skills | Optional Skills |
|-----------|----------------|----------------|
| Code | tri, agent-coord | simplify, wrap-up |
| Research | agent-coord | wrap-up |
| CI | agent-coord | wrap-up |
| Doctor | tri, agent-coord | loop |

## Emergency Protocols

If agent coordination fails:
1. Check `.trinity/experience/episodes.jsonl`
2. Check `.trinity/events/akashic-log.jsonl`
3. Check git log for context
4. Use fallback knowledge sources

**Emergency override:** Agent may bypass coordination only if:
- System is in recovery mode
- NotebookLM is unavailable
- User explicitly authorizes bypass

## Compliance Check

Before any action, agent must:

```
✓ Checked agent-rules for applicable laws
✓ Queried agent-coord for existing knowledge
✓ Confirmed issue binding (if code mutation)
✓ Verified target spec path is canonical
✓ Planned wrap-up upload
```

---

**Constitutional Source:** SOUL.md + T27-CONSTITUTION.md + AGENTS.md
