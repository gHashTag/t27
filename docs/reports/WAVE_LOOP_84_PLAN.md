# Wave Loop 84 Plan

**Scope:** Security fixes + mass closure sprint + arXiv endorsement push.

## Health Gates (daily)

- [ ] `t27c suite --repo-root .` → 549/549
- [ ] `cargo test --workspace` → 534/534
- [ ] `cd proofs/trinity && make` → 0 errors, 0 Admitted
- [ ] `cargo clippy --workspace` → 0 warnings
- [ ] `cargo clippy --workspace --all-features` → 0 warnings

---

## Track A — Security and Engineering

### A1: Implement Auth Middleware (#1193)
**Goal:** Add JWT or API key auth to compiler/server endpoints.
**Acceptance:** Requests without valid credentials rejected; local CI unaffected.

### A2: Implement SSRF Guards (#1194)
**Goal:** Validate `repo_root` before WalkDir; add localhost guard to proxy.
**Acceptance:** `/graph` rejects non-allowlisted paths; proxy restricts to localhost.

### A3: Fix Remaining Audit-Wave Sub-Bugs
**Goal:** Progress at least 2 of #1195, #1196, #1197, #1198.
**Acceptance:** PR opened with fix + test, or honest blocker documented.

---

## Track B — Issue Hygiene Sprint

### B1: Mass Closure Sprint
**Goal:** Reduce open issue count from 66 to ≤60.
**Method:** Identify 6+ issues that are truly resolved but still open (auto-close failures, stale aggregates, duplicates).
**Acceptance:** At least 6 issues closed with honest notes.

---

## Track C — arXiv and Publication

### C1: arXiv Endorser Outreach
**Goal:** Send 2+ endorsement requests for physics.gen-ph.
**Acceptance:** Requests sent with `ENDORSEMENT_REQUEST.md`.

### C2: Zenodo Draft (parallel)
**Goal:** Create Zenodo draft for v1.0.0 artifacts.
**Acceptance:** Draft created with DOI reserved.

---

## Track D — Competitive Intelligence

### D1: July 2026 arXiv Batch
**Goal:** Monitor hep-th and physics.gen-ph for new papers.
**Acceptance:** New papers catalogued or "no new competitors" logged.

---

## Track E — Cooperation Variants

Produce three concrete cooperation proposals (see `WAVE_LOOP_84_COOPERATION.md`).

---

## Definition of Done

- [ ] Suite 549/549, cargo 534/534, Coq 0 Admitted.
- [ ] `cargo clippy --workspace --all-features` passes with 0 warnings.
- [ ] W83 report published.
- [ ] W84 plan published.
- [ ] At least 1 security issue progressed.
- [ ] At least 6 issues closed in mass closure sprint.
- [ ] 3 cooperation variants written.
- [ ] Memory + skills saved.
