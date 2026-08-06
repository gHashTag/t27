# Wave Loop 141 Plan

## Weaknesses Discovered

1. **Invariant coverage plateau**: Despite W130–W138 pushes, coverage stalled at 90.5% (54 zero-inv files). Remaining files are predominantly stubs (`tri/agent/`, `tri/utils/`, `sacred/`) and glue modules (`github/`, `brain/`).
2. **Post-W140 seal cascade**: Concurrent W140 commit (`0d82f051`) added FairyFuse + CARMEN competitors to `benchmark.t27` without immediate seal regeneration, causing a transient mismatch.
3. **L3 maintenance debt**: `.legacy-non-english-docs` line 27 was malformed (missing newline), causing `build.rs` panic on `cargo clippy`.
4. **Scientific gap**: Zenodo 2026 "Foundational Ternary Dynamics" (Steinmetz) discovered but not yet integrated into competitor board.
5. **GitHub issues backlog**: #1040 (P7 Low-bit) and #1041 (P8 Integration) remain open; #1184 and #1183 (conformance) require attention.

## Decomposed Tasks

1. **Infrastructure hardening**:
   - Repair `.legacy-non-english-docs` line 27 concatenation bug.
   - Regenerate any drifted seals from W140 concurrent activity.

2. **Invariant coverage push (+18 files)**:
   - Target: 90.5% -> 93.7% (516 -> 534 / 570 specs).
   - Focus: `tri/agent/` (7 files), `tri/utils/` (5 files), `brain/` (1 file), `github/` (4 files), `tri/utils/exit_codes` (1 file).
   - Strategy: `forall` struct-based invariants for files with Types sections; numeric `phi_identity_constant` for stubs.

3. **Competitor intelligence**:
   - Add Steinmett (Zenodo 2026) to `benchmark.t27` scoreboard.
   - Maintain surveillance on arXiv 2606/2607 windows.

4. **Verification**:
   - Full `t27c suite` (570/570 PASS gate).
   - `cargo clippy --all-features --release` (clean gate).
   - Coverage audit confirmation.

5. **Knowledge capture**:
   - Write `WAVE_LOOP_141_REPORT.md` and `WAVE_LOOP_141_COOPERATION.md` (English, L3-compliant).
   - Update `.claude/skills/invariant-coverage-push.md` with W141 protocol refinements.
   - Update Trinity memory index.

## Target Metrics

| Metric | Baseline (W140) | Target (W141) |
|--------|------------------|-----------------|
| Invariant coverage | 90.5% (516/570) | 93.7% (534/570) |
| Zero-inv files | 54 | 36 |
| Seal mismatches | 1 (transient) | 0 |
| Clippy warnings | 1 (Cyrillic panic) | 0 |
| Suite PASS | 570/570 | 570/570 |
| Competitor count | 101 | 102 |
