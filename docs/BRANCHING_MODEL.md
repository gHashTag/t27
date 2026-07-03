# Trinity t27 Branching Model

This document records the branch policy agreed in **Wave Loop 392**.

---

## Branches

| Branch | Role | Acceptable force-push | Merge target |
|---|---|---|---|
| `master` | Release / stable dependabot updates | **Never** | N/A |
| `trinity-rust-rings` | IGLA CODER+RACE integration branch | Only emergency recovery | N/A (integration sink) |
| `wave-loop-NNN` | Temporary wave-loop work branches | Allowed while open | `trinity-rust-rings` only |

## `master`

- Contains release-quality code, dependabot bumps, and small mergeable-only PRs.
- No wave-loop history. No `specs/igla/` tree as of 2026-07-04.
- All merges must pass CI and require review.

## `trinity-rust-rings`

- Long-lived integration branch for the IGLA CODER and IGLA RACE spec families.
- Carries the `specs/igla/` tree, the Lean 4 `Trinity.TernaryInference` lattice, and the per-wave generator scripts.
- As of Wave Loop 391 it is **224 commits ahead, 10 commits behind, 300 files diverged** from `master`.
- Normal update path: open a PR from `wave-loop-NNN` and squash-merge via GitHub UI/CLI.
- Force-push is reserved for emergency recovery only, not for routine workflow.

## `wave-loop-NNN`

- One branch per wave-loop. Branch from the previous wave-loop branch or from `trinity-rust-rings`.
- Force-push to the work branch is allowed while the branch is open.
- PR target is always `trinity-rust-rings`, never `master`.
- Deleted after merge.

## Master-alignment epic

- Replaying `trinity-rust-rings` onto `master` is **not part of the wave-loop workflow**.
- Tracked as a separate epic issue with labels `epic`, `master-alignment`, `long-running`.
- Requires explicit user approval and a dedicated time window. See the epic issue for the replay plan.

---

*phi^2 + phi^-2 = 3 | TRINITY*
