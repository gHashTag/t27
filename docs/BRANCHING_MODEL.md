# Trinity t27 Branching Model

This document records the branch policy agreed in **Wave Loop 392**.

---

## Branches

| Branch | Role | Acceptable force-push | Merge target |
|---|---|---|---|
| `master` | Release / stable dependabot updates | **Never** | N/A |
| `trinity-rust-rings` | IGLA CODER+RACE integration branch | Only emergency recovery | N/A (integration sink) |
| `wave-loop-NNN` | Temporary wave-loop work branches | Allowed while open | `master` (Strategy P) |

## Strategy P (Wave Loop 417+)

- Wave-loop PRs are opened against **`master`**, not `trinity-rust-rings`.
- `trinity-rust-rings` remains the IGLA CODER+RACE integration sink, but the
  `specs/igla/` tree is no longer on `master`, so FPGA/tooling waves land
  directly on `master`.
- Branches are deleted after merge; force-push is still allowed only while the
  PR branch is open.

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

- One branch per wave-loop. Branch from `master` (or from the previous
  wave-loop branch only when it has already landed on `master`).
- Force-push to the work branch is allowed while the branch is open.
- PR target is always `master` per Strategy P.
- Deleted after merge.

## Master-alignment epic

- Replaying `trinity-rust-rings` onto `master` is **not part of the wave-loop workflow**.
- Tracked as a separate epic issue: **#1284** (`epic`, `master-alignment`, `long-running`).
- Requires explicit user approval and a dedicated time window. See #1284 for the replay plan and `docs/reports/WAVE_LOOP_391_MASTER_ALIGNMENT.md`.

---

*phi^2 + phi^-2 = 3 | TRINITY*
