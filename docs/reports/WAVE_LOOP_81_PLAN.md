# Wave Loop 81 Plan

**Scope:** Server feature compilation fix + zombie issue split + arXiv endorsement push.

## Health Gates (daily)

- [ ] `t27c suite --repo-root .` → 549/549
- [ ] `cargo test --workspace` → 534/534
- [ ] `cd proofs/trinity && make` → 0 errors, 0 Admitted
- [ ] `cargo clippy --workspace` → 0 warnings
- [ ] `cargo clippy --workspace --all-features` → 0 warnings (NEW GATE)

---

## Track A — Engineering Hygiene

### A1: Fix `--all-features` Clippy
**Goal:** Make `cargo clippy --workspace --all-features` pass cleanly.
**Known fixes needed:**
1. Add `tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }` to `bootstrap/Cargo.toml`.
2. Add `use tower_http::limit::RequestBodyLimitLayer;` to `bootstrap/src/main.rs`.
3. Verify server feature compiles after fixes.
**Acceptance:** `cargo clippy --workspace --all-features` exits 0.

### A2: Zombie Issue Split
**Goal:** Split multi-bug audit-wave issues into atomic sub-issues for honest tracking.
**Targets:**
- #930 → Split Bug 1 (auth middleware), Bug 2 (SSRF /graph), Bug 3 (SSRF proxy) into separate issues. Close #930 after referencing new issues.
- #940 → Split Bug 3 (unknown variable assignment), Bug 4 (O(N²) re-parsing) into separate issues.
- #985 → Close after confirming sub-bugs 2-4 fixed; open new issue for L6 CEILING (WASM→FFI bridge) if desired.
**Acceptance:** At least 3 new atomic issues created, 3 zombie issues closed.

---

## Track B — arXiv and Publication

### B1: Server Feature Deprecation Decision
**Goal:** Decide whether to maintain the HTTP server feature or deprecate it.
**Rationale:** Server feature adds heavy deps (axum, tokio, tracing) and is not part of the critical path (CLI compiler is). If unused, removal simplifies maintenance.
**Acceptance:** Decision documented in `docs/adr/` or issue created.

### B2: arXiv Endorser Outreach
**Goal:** Send 2+ endorsement requests for physics.gen-ph.
**Acceptance:** Requests sent with `ENDORSEMENT_REQUEST.md` attached.

---

## Track C — Competitive Intelligence

### C1: July 2026 arXiv Batch
**Goal:** Monitor hep-th and physics.gen-ph for new geometric-unification papers after 2026-07-01.
**Acceptance:** New papers catalogued or "no new competitors" logged.

### C2: Ω-Theory Commit Watch
**Goal:** Check for spectral-action or heat-kernel commits.
**Acceptance:** Activity logged.

---

## Track D — Cooperation Variants

Produce three concrete cooperation proposals (see `WAVE_LOOP_81_COOPERATION.md`).

---

## Definition of Done

- [ ] Suite 549/549, cargo 534/534, Coq 0 Admitted.
- [ ] `cargo clippy --workspace --all-features` passes OR decision to deprecate server feature documented.
- [ ] W80 report published.
- [ ] W81 plan published.
- [ ] At least 3 zombie issues split or closed.
- [ ] 3 cooperation variants written.
- [ ] Memory + skills saved.
