# IGLA-GF16 BPB Status — April 22, 2026 (LIVE)

## Training Results (Pure PyTorch, CPU, FineWeb 50MB)

| Tier | Params | Layers | Steps | Val BPB | Time | Size FP16 | Status |
|------|--------|--------|-------|---------|------|-----------|--------|
| T1 tiny | 935K | 2L/192d | 5000 | **2.0195** | 33min | 1.79 MB | DONE |
| T2 small | 2.7M | 6L/192d | 5000 | **1.9165** | 137min | 5.18 MB | DONE |
| T3 medium | 4.5M | 10L/192d | 5000 | — | ~4h est | ~9 MB | QUEUED |
| T4 submit | 6.3M | 14L/192d | 10000 | — | ~12h est | ~12.6 MB | QUEUED |

## BPB Trajectory (T2 small — best so far)

```
Step  500: BPB 3.12
Step 1000: BPB 2.54  (Δ -0.58)
Step 2000: BPB 2.11  (Δ -0.43)
Step 3000: BPB 1.99  (Δ -0.12)
Step 4000: BPB 1.94  (Δ -0.05)
Step 5000: BPB 1.92  (Δ -0.02) ← converging, needs more capacity
```

## Gap to Target

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Best BPB (T2) | 1.9165 | ≤ 1.10 | 0.82 |
| Expected T3 | ~1.7 | ≤ 1.10 | ~0.6 |
| Expected T4+Muon | ~1.3 | ≤ 1.10 | ~0.2 |

## Next Steps
1. T3 medium (10L/4.5M) — launching now
2. T4 submit (14L/6.3M) — after T3
3. Muon optimizer sweep — after T4 baseline
4. 3-seed sweep for statistics
5. Final submission before Apr 30
