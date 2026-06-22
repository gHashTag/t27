# Wave Loop 142 Plan

## Weaknesses Discovered

1. **Invariant coverage gap**: 36 specs still had zero invariants after W141 (93.7%). Remaining files were concentrated in sacred/ (11), ml/ (7), tri/ (3), physics/ (3), sandbox/ (2), brain/ (1), automation/ (1), numeric/ (1).
2. **L3 regression risk**: docs/reports/WAVE_LOOP_140_COOPERATION.md and WAVE_LOOP_140_REPORT.md contain Cyrillic but were dropped from `.legacy-non-english-docs` during W141 rewrite, breaking `cargo clippy`.
3. **Competitive stagnation**: No new July 2026 arXiv papers detected; arXiv 2607 window expected late July.
4. **GitHub issues backlog**: #1040, #1041 (IGLA roadmap) and #1184, #1183 (conformance) remain open.

## Decomposed Tasks

1. **Infrastructure fix**: Re-add W140 Cyrillic docs to `.legacy-non-english-docs`.
2. **Invariant coverage push (+18 files)**: Target 93.7% → 96.8% (552/570).
3. **Competitor intelligence**: Monitor arXiv 2606/2607 windows; discover 1–2 new entrants.
4. **Verification**: `t27c suite --repo-root .` (570/570 PASS), `cargo clippy --all-features --release` (clean).
5. **Reporting**: English-only `WAVE_LOOP_142_REPORT.md`, `WAVE_LOOP_142_COOPERATION.md` to avoid future L3 grandfathering.

## Target Metrics

| Metric | Baseline (W141) | Target (W142) |
|--------|------------------|-----------------|
| Invariant coverage | 93.7% (534/570) | 96.8% (552/570) |
| Zero-inv files | 36 | 18 |
| Suite PASS | 570/570 | 570/570 |
| Clippy warnings | 0 | 0 |
