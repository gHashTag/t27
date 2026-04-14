# Queen brain — agent log aggregation (Trinity)

**Purpose:** Optional directory for **aggregated** summaries of multi-agent runs (Lotus cycle, swarm tooling, CI “Queen” reports). It is **not** a substitute for **`.trinity/events/`** (Akashic append-only log) or **`.trinity/experience/`** (episodes); those remain authoritative for coordination and learning per **`docs/SOUL.md`** Laws **#6–#7**.

## Layout (convention)

| Path | Use |
|------|-----|
| `summaries/*.md` | Human-readable rollups per milestone or ring slice (optional, may be committed if small). Example: `summaries/github-sync-YYYY-MM-DD.md` after refreshing **`.trinity/state/github-sync.json`** from GitHub. |
| `*.jsonl` | Machine streams — **gitignored** by default (see repository `.gitignore`). |

## Rules

1. **Do not** store secrets or credentials here.  
2. **Large** or high-churn logs belong in **gitignored** files under this tree, not in forced-tracked blobs.  
3. **AGENT T** (Queen) may reference this directory when publishing a **plan seal** (TAW) for an epoch; the seal record itself should still tie to a **GitHub Milestone / issue** per **`docs/SOUL.md`** Law **#9**.

## See also

- **`docs/AGENTS_ALPHABET.md`** — 27 agents, Lotus phases.  
- **`docs/EPOCH_01_HARDEN_PLAN.md`** — EPOCH-01 (Rings 32–58) milestone and issue templates.  
- **`SOUL.md`** (root) — Articles **VIII–X**.
