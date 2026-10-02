# NOW -- Define contributor key contracts and record their gates (2026-10-01)

## Define contributor key contracts and record their gates (Closes #5472)

- Add render and Queen contracts for trusted identity, bounded credentials, stable ownership and probe limits (`specs/automation/hive-contributor-keys.t27`, `queen-contributor-keys.t27`); preserve existing work-based XP without wallet credits.
- Execute 17 spec tests with Zig 0.16.0 (8 + 9, all pass), catch all 19 negative controls (`conformance/automation/*.controls.json`), and kill all 8 render implementation mutants. Both specs are sealed and `t27c seal --verify` reports all hashes MATCH.
- Scope, after review: this PR carries only the two contracts, their controls, their two seals and this entry. The CI repairs it first collected were landed separately (#5501, #5502, #5503, #5504 and the PRs after them); the 80 placeholder tests of `igla/coder/pipeline` and the ratchet moves that follow are #5613; the corpus re-take and test-block pin are left for right before merge, because they move with every port PR.
