# Architecture Mismatch Bug — Phase A vs Phase B

## Issue
Phase B fine grid winner LR=0.0262 gave BPB=6.56, which was incorrectly claimed as "15% improvement" over Phase A baseline BPB=5.91.

## Root Cause
Table comparison bug: Phase A and Phase B used DIFFERENT architectures.

| Phase | Binary | Architecture | n_layers |
|-------|--------|------------|----------|
| Phase A | train_real_fixed_v2 | Full transformer | 1 |
| Phase B fine | smart_phase_b / phase_b_fine | Embedding-only | 0 |

## Correct Interpretation
Phase A (5.91) < Phase B fine (6.56) = Full transformer BEATS embedding-only.
Phase B experiments were testing wrong model.

## Decision
ABANDON Phase B LR sweep. Return to Phase A config with warmup.
