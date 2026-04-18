# Ring backlog 047–063 — agent activation (planning)

**Purpose:** Placeholder for **opening GitHub issues** **Ring 047 … Ring 063** so each of the **27 agents** can have **visible** work items beyond the **EPOCH-01-HARDEN** slice (Rings **032–046**).  
**Law:** **`docs/T27-CONSTITUTION.md`** **Article RING-LAW** (one ring = one capability); **`docs/T27-CONSTITUTION.md`** **Article AGENT-DOMAIN**.

**Do not** open all issues at once unless a **milestone** and **Queen** plan exist (**`docs/SOUL.md`** Article **VIII**).

---

## Suggested batch

| Ring | Suggested primary agent | Theme (one capability per issue) |
|------|-------------------------|-----------------------------------|
| 047 | T | Lotus phase automation hook — `TASK.md` sync job |
| 048 | A | ADR index automation + stale ADR lint |
| 049 | Z | Docs i18n debt shrink plan (`docs/.legacy-non-english-docs`) |
| 050 | N | NUMERIC-STANDARD-001 conformance spot-check expansion |
| 051 | P | Sacred physics overlay — claim ID audit only |
| 052 | F | Conformance corpus — property-test template |
| 053 | V | Bench harness — reproducible artifact path |
| 054 | G | `graph_v2.json` — drift detection in CI |
| 055 | W | Seal witness format — cross-backend tag |
| 056 | M | Metrics export — JSON schema for verdicts |
| 057 | C | Compiler error catalog — user-facing codes |
| 058 | R | Runtime stub — documented “not implemented” surface |
| 059 | H | Hardware codegen doc — single source for pins |
| 060 | I | ISA doc — register ↔ agent table completion |
| 061 | J | Job queue spec — t27-side task description |
| 062 | K | Kernel boundary doc — privileged vs user |
| 063 | L | Linker script story — Zig/C agreement |

*Letters **047–063** above are **illustrative**; reassign per **`docs/AGENTS_ALPHABET.md`** and real gaps.*

---

## Paste template (GitHub)

**Title:** `Ring 0NN: <single capability>`  
**Labels:** `ring`, `harden` (or next phase label), `agents`, `phi-loop` as appropriate.  
**Milestone:** create **`EPOCH-02-AGENT-ACTIVATION`** (or similar) before bulk create.

**Body:**

```markdown
## Ring
- **ID:** RING-0NN

## Normative
- `docs/T27-CONSTITUTION.md` — Articles **RING-LAW**, **AGENT-DOMAIN**
- `docs/RINGS.md`
- Primary agent: **X** — `docs/AGENTS_ALPHABET.md`

## Acceptance
- [ ] One capability sealed / documented / tested per **Article RING-LAW**
- [ ] PR `Closes #…`
```

---

*Canonical constitution URL on GitHub (default branch **master**): `https://github.com/gHashTag/t27/blob/master/docs/T27-CONSTITUTION.md`*
