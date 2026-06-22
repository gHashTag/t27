# Wave Loop 144 Plan

## Weaknesses Discovered

1. **Property depth gap**: With 100% coverage reached (W143), 245 specs (43.0%) still have only a single invariant. Average invariants per spec: 2.07. L4 TESTABILITY is satisfied in breadth but not yet in depth.
2. **Competitive stagnation**: arXiv 2607 window still closed. No new indexed July 2026 papers. However, viXra:2603.0042 (Triality-Resolved Spectral Update Theory) discovered as a new ternary-competitor framework.
3. **GitHub issues backlog**: #1040, #1041, #1184, #1183 remain open. No progress on conformance debt.
4. **Seal hygiene**: No mismatches detected at HEAD, but concurrent W141b commit (`b9c8312a`) modified `benchmark.t27` + 8 igla/race specs, confirming that seal cascades remain a risk from parallel igla activity.

## Decomposed Tasks

1. **Property depth push (+20 second invariants)**: Target 245 → 225 single-inv files, raising average invariants per spec from 2.07 → 2.28.
2. **Competitor intelligence**: Log Triality-Resolved Spectral Update Theory (viXra:2603.0042). Monitor arXiv 2607 weekly.
3. **L3 hygiene**: Verify `.legacy-non-english-docs` is stable (no new Cyrillic regressions).
4. **Verification**: 570/570 PASS, clippy clean.
5. **Reporting**: English-only W144 report/cooperation, skill update, memory index.

## Target Metrics

| Metric | Baseline (W143) | Target (W144) |
|--------|-----------------|----------------|
| Single-inv files | 245 | 225 |
| Two-inv files | 59 | 79 |
| Avg invariants/spec | 2.07 | 2.28 |
| Suite PASS | 570/570 | 570/570 |
| Clippy warnings | 0 | 0 |
