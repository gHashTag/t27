# Wave Loop 143 Plan

## Weaknesses Discovered

1. **Invariant coverage gap**: 18 specs remained at 0 invariants after W142 (96.8%). Remaining files were spread across ml/ (7), physics/ (3), sandbox/ (2), tri/agent/ (3), automation/ (1), numeric/ (1).
2. **L3 maintenance**: Prior waves added Cyrillic W140 docs but dropped them from `.legacy-non-english-docs` accidentally. Risk of re-regression.
3. **Competitive landscape**: Strong new 2026 entrants discovered:
   - Gresnigt arXiv:2601.07857 (Cl(10), S3 symmetry, 3 generations)
   - Ardakanian arXiv:2603.15455 (Z3 Froggatt-Nielsen, fermion mass hierarchy)
   - Kulkarni "Geometric Origin of SM" (cuboctahedron K=12, 3 generations)
4. **GitHub issues backlog**: #1040 (P7 Low-bit), #1041 (P8 Integration), #1184, #1183 open.

## Decomposed Tasks

1. **Final invariant sweep (+18 files)**: Target 96.8% -> 100.0% (570/570).
2. **L3 grandfathering hygiene**: Verify `.legacy-non-english-docs` completeness.
3. **Competitor intelligence**: Log new 2026 entrants into competitive memory.
4. **Verification**: 570/570 PASS, clippy clean.
5. **Reporting**: English-only W143 report/cooperation, skill update, memory index.

## Target Metrics

| Metric | Baseline (W142) | Target (W143) |
|--------|-----------------|----------------|
| Invariant coverage | 96.8% (552/570) | 100.0% (570/570) |
| Zero-inv files | 18 | 0 |
| Suite PASS | 570/570 | 570/570 |
| Clippy warnings | 0 | 0 |
