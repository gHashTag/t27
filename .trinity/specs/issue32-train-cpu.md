# Issue #32: Φ6 trios-train-cpu — Pure Rust CPU Training Pipeline

**Status:** Planning Phase
**Date:** 2026-04-20
**Authors:** Dmitrii Vasilev
**Issue:** #32

---

## Executive Summary

Implement CPU-only training pipeline for IGLA-GF16 model (d_model=144) using pure Rust.

**Why CPU-first:**
- Immediate execution on local machine (no GPU wait/setup)
- Pure Rust = zero external dependencies
- Memory efficient: ~50MB RAM fits entirely in cache
- Ternary FFN (0.2 bytes/param) minimal CPU load

---

## Requirements

| Requirement | Priority | Status |
|-------------|-----------|--------|
| Pure Rust training loop | P0 | [ ] |
| IGLA-GF16 model (d_model=144) | P0 | [ ] |
| BPB metric calculation | P0 | [ ] |
| FineWeb batch loader | P1 | [ ] |
| φ-LR scheduler (φ-schedule) | P1 | [ ] |
| CLI interface | P2 | [ ] |

---

## Implementation Plan

| Phase | Description | Deliverable | Status |
|--------|-------------|-------------|--------|
| Φ1 | Core training loop | `crates/trios-train-cpu/src/trainer.rs` | [ ] |
| Φ2 | IGLA-GF16 model | `crates/trios-train-cpu/src/model.rs` | [ ] |
| Φ3 | BPB metric | `crates/trios-train-cpu/src/metrics.rs` | [ ] |
| Φ4 | φ-LR scheduler | `crates/trios-train-cpu/src/scheduler.rs` | [ ] |
| Φ5 | FineWeb loader | `crates/trios-train-cpu/src/data.rs` | [ ] |
| Φ6 | CLI interface | `crates/trios-train-cpu/src/main.rs` | [ ] |

---

## Technical Details

### Model Architecture (IGLA-GF16)

```rust
pub struct IglaGf16Config {
    pub d_model: usize,       // 144
    pub n_heads: usize,       // 8
    pub d_head: usize,        // 18
    pub d_ffn: usize,        // 233
    pub n_layers: usize,      // 7
    pub vocab_size: usize,     // 32000
}
```

### Training Configuration

```rust
pub struct TrainingConfig {
    pub batch_size: usize,     // 4
    pub seq_len: usize,        // 128
    pub n_steps: usize,        // 1000
    pub learning_rate: f32,    // α_φ * φ^(-t/τ)
    pub warmup_steps: usize,   // 100
}
```

### BPB Metric

BPB = total_bits_compressed / total_bytes_training

```rust
pub fn compute_bpb(total_bits: u64, total_bytes: u64) -> f64 {
    (total_bits as f64) / (total_bytes as f64)
}
```

### φ-LR Scheduler

```rust
pub fn phi_schedule(step: usize, alpha: f32, phi: f32, tau: usize) -> f32 {
    let decay = phi.powi(-(step as i32) / tau as i32);
    alpha * decay
}
// φ ≈ 0.618, τ ≈ 1000
```

---

## Testing Plan

| Test Type | Coverage | Expected Result |
|-----------|-----------|----------------|
| Unit tests | trainer, model, metrics | [ ] Pass cargo test |
| Integration | Full training run | [ ] BPB improves over baseline |
| Benchmark | 1000 steps on M2 | [ ] ~60 seconds runtime |

---

## Definition of Done (L0: Immutable)

**ALL** completed tasks MUST include these steps before merging:

```bash
# 1. Stage changes
git add crates/trios-train-cpu/

# 2. Commit with Issue reference
git commit -m "feat(trios-train-cpu): <description>

refs #32
- <key changes bullet points>"

# 3. Push to remote
git push origin main
```

**Verification:**
- [ ] All crates modified are staged and committed
- [ ] Commit message contains `refs #32`
- [ ] `git status` shows clean (no uncommitted changes)
- [ ] `cargo clippy -- -D warnings` passes (L3)
- [ ] `cargo test` passes (L4)
- [ ] Training completes 1000 steps in ~60s on M2
- [ ] BPB metric computed and logged

---

## References

- [Issue #32](https://github.com/gHashTag/trios/issues/32)
- [IGLA-GF16 Specification](.trinity/specs/igla-gf16-3model-synthesis.md)

---

## Next Actions

1. **[ ]** Create `crates/trios-train-cpu/Cargo.toml`
2. **[ ]** Implement core training loop (Φ1)
3. **[ ]** Implement IGLA-GF16 model (Φ2)
4. **[ ]** Implement BPB metric (Φ3)
5. **[ ]** Implement φ-LR scheduler (Φ4)
6. **[ ]** Implement FineWeb loader (Φ5)
7. **[ ]** Implement CLI interface (Φ6)

---

**Closes:** #32
