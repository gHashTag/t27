# IGLA Needle Hunt — Agent Task Queue

## Status Overview

| Agent | Task | Status | ΔBPB Target |
|-------|------|--------|-------------|
| **GOLF** | φ-OrthoInit, SWA, ResidMix, Sliding | ✅ DONE | -0.11 |
| **FOXTROT** | BigramHash(729) + SmearGate | ✅ DONE (#183) | -0.06 |
| **ALFA** | Muon optimizer + WD sweep | ✅ IMPLEMENTED | -0.02 |
| **HOTEL** | TTT-LoRA | 🆕 QUEUED | -0.03 |
| **INDIA** | Layer sharing 5L×4iter | 🆕 QUEUED | -0.02 |
| **DELTA** | Spectral Embedding Init | ✅ IMPLEMENTED (#188) | -0.03 |

## Agent Directories

- `foxtrot/` - Hash-based embedding tricks (BigramHash) ✅
- `alfa/` - Optimizer tuning (Muon, weight decay) ✅
- `hotel/` - Test-time training (TTT-LoRA, ≠ JEPA-TTT) ⏸
- `india/` - Architecture tricks (layer sharing, depth recurrence) ⏸
- `delta/` - Initialization tricks (spectral, φ-based) ✅

## Running Experiments (L1 compliant — NO .sh)

**L1 LAW: NO .sh files. Use `tri` CLI only.**

```bash
# FOXTROT: BigramHash 729 (DONE)
tri run IGLA-BGH-301 --seeds 3

# ALFA: Muon WD sweep (IMPLEMENTED)
tri run IGLA-MUON-105 --seeds 3

# DELTA: Spectral embedding init (IMPLEMENTED)
tri run IGLA-SPEC-P06 --seeds 3

# Agent dispatch (NATO naming)
tri agent dispatch FOXTROT "IGLA-BGH-301"
tri agent dispatch ALFA "IGLA-MUON-105"
tri agent dispatch DELTA "IGLA-SPEC-P06"
```

## Implemented Components

### ✅ FOXTROT (Closes #183)
```rust
// crates/trios-training/src/bigram_hash.rs
pub struct BigramHashTable {
    pub vocab_size: usize,  // 729 = 3^6
    pub dim: usize,         // 64
    pub embeddings: Vec<Vec<f32>>,
}

pub struct SmearGate {
    pub dim: usize,
    pub weights: Vec<f32>,  // φ-initialized
    pub bias: f32,
}
```

### ✅ ALFA (Muon optimizer)
```rust
// crates/trios-training/src/muon.rs
pub struct Muon {
    pub matrix_lr: f32,
    pub momentum: f32,
    pub weight_decay: f32,
    pub backend_steps: usize,
    // ...
}

// WD sweep values for IGLA-MUON-105
pub const MUON_WD_SWEEP: &[f32] = &[0.02, 0.03, 0.04, 0.05, 0.06];
```

### ✅ DELTA (Closes #188)
```rust
// crates/trios-training/src/spectral_init.rs
pub struct SpectralInit {
    pub dim: usize,
    pub vocab_size: usize,
    // SVD-based initialization
}
```

## Progress Tracking

```bash
# Experience log (auto-managed by tri)
tri log add IGLA-BGH-301 DONE
tri log add IGLA-MUON-105 IMPLEMENTED
tri log add IGLA-SPEC-P06 IMPLEMENTED
```

## Unlock Path

1. ✅ FOXTROT completes → BigramHash winner selected
2. ✅ ALFA completes → Muon WD optimal found
3. 🔒 HOTEL + INDIA implementation
4. 🔒 GOLF Tournament (64 runs) → G-STACK ≤ 1.12
5. 🔒 IGLA-STACK-502 → GOLF + FOXTROT + ALFA + DELTA combined
6. 🔒 IGLA-NEEDLE → Full stack + GF16 + TTT-LoRA (target ≤ 1.10)

## RINGS Progress

| Ring | Category | % |
|------|----------|---|
| R1 CORE | Foundation | 100% ✅ |
| R2 PRETRAIN | Training | 60% ↑ |
| R3 SCALING | GOLF stack | 60% |
| R4 INTEGRATION | IGLA-STACK | 20% ↑ |
| R5 SUBMIT | Apr 30 | 0% |
| **TOTAL** | | **~52%** |

## Deadline

**30 Apr 2026** · 9 days remaining

Target: **≤ 1.10 BPB** (beating bigbag SOTA 1.0810)

## Next Steps

1. ✅ FOXTROT — DONE (#183 closed)
2. ✅ ALFA — IMPLEMENTED (muon.rs)
3. ✅ DELTA — DONE (#188 closed)
4. ⏸ HOTEL — TTT-LoRA implementation needed
5. ⏸ INDIA — Layer sharing implementation needed
6. 🔒 IGLA-STACK-502 — Integration after HOTEL/INDIA
