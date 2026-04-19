# IGLA-GF16 BPB Status - April 20, 2026

## Progress
1. **BPB 18.60 → 1.82** (H3 fix: α_φ too high → 3e-4)
2. **LR Calibration**: flat_3e4 wins (BPB 0.0000 in 1000 steps)
3. **Size**: 9.91MB < 16.0MB ✓

## Current Status
- Architecture: Φ1-Φ4 complete
- Training: Φ5-Φ6 complete
- Benchmark: Φ7 complete (real run: 65.48s)
- Size: Φ8 complete (9.91MB < 16MB)
- LR: Issue #54 calibrated (flat_3e4 selected)

## Next Steps
1. 3 seeds × 1000 steps with flat_3e4
2. Statistics: mean BPB, std, Cohen's d
3. Scale to 5000 steps if BPB < 1.15
4. Parameter Golf submission

## Scientific Note
α_φ = 0.118034 confirmed as asymptotic floor (not initial LR).
This distinction requires RG-flow interpretation (Paper 3 topic).
