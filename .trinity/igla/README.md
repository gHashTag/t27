# IGLA Needle Hunt — Agent Task Queue

## Status Overview

| Agent | Task | Status | ΔBPB Target |
|-------|------|--------|-------------|
| **GOLF** | φ-OrthoInit, SWA, ResidMix, Sliding | ✅ DONE | -0.11 |
| **FOXTROT** | BigramHash(729, 10240) | 🔴 IN FLIGHT | -0.06, -0.04 |
| **ALFA** | Muon WD sweep | 🔴 IN FLIGHT | -0.02 |
| **HOTEL** | TTT-LoRA | 🆕 QUEUED | -0.03 |
| **INDIA** | Layer sharing 5L×4iter | 🆕 QUEUED | -0.02 |
| **DELTA** | Spectral Embedding Init | ⏸ AFTER #157 | -0.03 |

## Agent Directories

- `foxtrot/` - Hash-based embedding tricks (BigramHash)
- `alfa/` - Optimizer tuning (Muon, weight decay)
- `hotel/` - Test-time training (TTT-LoRA, ≠ JEPA-TTT)
- `india/` - Architecture tricks (layer sharing, depth recurrence)
- `delta/` - Initialization tricks (spectral, φ-based)

## Running Experiments (L1 compliant — NO .sh)

**L1 LAW: NO .sh files. Use `tri` CLI only.**

```bash
# FOXTROT: BigramHash 729
tri run IGLA-BGH-301 --seeds 3

# FOXTROT: BigramHash 10240 + SmearGate
tri run IGLA-BGH-302 --seeds 3

# ALFA: Muon WD sweep
tri run IGLA-MUON-105 --seeds 3

# Agent dispatch (NATO naming)
tri agent dispatch FOXTROT "IGLA-BGH-301"
tri agent dispatch ALFA "IGLA-MUON-105"
```

## Progress Tracking

```bash
# Experience log (auto-managed by tri)
tri log add IGLA-BGH-301 IN_FLIGHT
```

## Unlock Path

1. ✅ FOXTROT completes → BigramHash winner selected
2. ✅ ALFA completes → Muon WD optimal found
3. ✅ GOLF Tournament (64 runs) → G-STACK ≤ 1.12
4. 🔒 IGLA-STACK-502 → GOLF + FOXTROT + ALFA combined
5. 🔒 IGLA-NEEDLE → Full stack + GF16 + TTT-LoRA (target ≤ 1.10)

## RINGS Progress

| Ring | Category | % |
|------|----------|---|
| R1 CORE | Foundation | 100% ✅ |
| R2 PRETRAIN | Training | 40% |
| R3 SCALING | GOLF stack | 60% ↑ |
| R4 INTEGRATION | IGLA-STACK | 0% 🔒 |
| R5 SUBMIT | Apr 30 | 0% |
| **TOTAL** | | **~48%** |

## Deadline

**30 Apr 2026** · 9 days remaining

Target: **≤ 1.10 BPB** (beating bigbag SOTA 1.0810)
