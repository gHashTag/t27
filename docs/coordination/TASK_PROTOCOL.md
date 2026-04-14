# TASK Protocol — inter-agent coordination (t27)

**Status:** Normative (paired with **`NOW.md`** at repository root and **`docs/T27-CONSTITUTION.md`**).  
**Protocol version:** 1.1  
**Date:** 2026-04-06  

---

## 1. Intent

**NOW** (the file **`NOW.md`** at the **repo root**) is the **shared coordination + rolling snapshot** surface for coding agents, tooling, and maintainers. It subsumes the former **`TASK.md`** coordination file.

- **Shared state** — inspectable in git, reviewable in PRs.
- **Explicit handoffs** — refresh **Revision**, narrative **§3–§9**, and append **experience** logs; use **Epoch** / **locks** in comments on the **Anchor issue** when multiple agents overlap (see §4).
- **Online anchor** — long-lived GitHub **Anchor issue** for comments, PR links, and real-time alignment.

**GitHub Issues** remain the **scheduling and merge SSOT** (`Closes #N`, Issue Gate). **`NOW.md`** must not contradict closed issues, **`CANON.md`**, or **`FROZEN.md`**.

---

## 2. Artifacts

| Artifact | Role |
|----------|------|
| **`NOW.md`** (repo root) | Rolling snapshot + coordination entrypoint; **Last updated** date enforced by **`./scripts/tri check-now`**. |
| **Anchor issue** | Live thread for agents/humans; link and updates referenced from **`NOW.md`**. |
| **`docs/coordination/TASK_PROTOCOL.md`** | This document — rules and **Verification** checklist. |
| **`.trinity/state/github-sync.json`** | Snapshot of ring/META issues; read before claiming work. |

---

## 3. Required freshness of `NOW.md`

- **`Last updated:`** line MUST include calendar **`YYYY-MM-DD`** matching **today** (local timezone) when running **`./scripts/tri check-now`** before commit/CI.
- On **non-trivial** completion, update narrative sections so the next agent reads current truth (see **`NOW.md` §1.1**).

There is **no** separate mandatory `TASK.md` heading scaffold; retired with **`TASK.md`** removal.

---

## 4. Coordination semantics

### 4.1 Lock (soft)

Before editing sensitive paths, the active agent SHOULD post intent on the **Anchor issue** (and optionally note scope in **`NOW.md` Revision**). Others MUST NOT override without a clear handoff comment.

Locks are **social + procedural** (not file locks). Trinity **claims** under `.trinity/` remain governed by **`docs/nona-03-manifest/SOUL.md`** Law **#6**.

### 4.2 Epoch

**Epoch** (when tracked) is a monotonic integer or narrative bump in **`NOW.md`** / **Anchor** when transferring ownership or resolving conflicts. Bump when resetting coordination after a major merge.

### 4.3 Handoff log

Prefer **Anchor issue** comments + **`.trinity/experience/`** append-only lines. If a long-form handoff is needed, use **`docs/coordination/inter-agent-handoff/`** bundles as **supplements** only.

### 4.4 Read / write order

1. `github-sync.json` (queue snapshot)  
2. **`NOW.md`** (current snapshot + coordination pointers)  
3. **Anchor issue** (latest comments)  
4. Target **GitHub issue** for the code change  

---

## 5. Automated checks

**`cargo build`** in **`bootstrap/`** scans **`NOW.md`** (among other first-party Markdown) for **Cyrillic** in identifiers/comments per **LANG-EN** / **ADR-004**.

**`./scripts/tri check-now`** enforces the **`Last updated:`** calendar date against **today**.

---

## 6. Verification (human + CI)

Before opening or updating a PR that touches **`NOW.md`** or multi-agent-critical paths:

1. Run **`./scripts/tri check-now`**.  
2. Post a **short comment** on the **Anchor issue** when multiple agents touched the same slice (link PR).  
3. Code PRs MUST link **`Closes #N`** to a substantive issue (Issue Gate), not only the anchor.

---

## 7. Amendments

- Bump **Protocol version** here when rules change.  
- If governance SSOT moves, amend **`docs/T27-CONSTITUTION.md`** and bump charter version.  

---

## 8. Supplementary handoff bundles (informative)

Optional **portable** markdown under [`docs/coordination/inter-agent-handoff/`](inter-agent-handoff/README.md) are **planning supplements** only — normative coordination remains **`NOW.md`** + **Anchor issue** + this protocol.

---

## References (informative)

- Shared state / handoff discipline in multi-agent coding workflows — aligned with explicit handoff and inspectable state.
