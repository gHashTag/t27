# IGLA-BGH-301: BigramHash(729) Sweep

## Configuration
- Vocab: 3^6 = 729
- Hash size: 729 → 10240
- Target ΔBPB: -0.06

## Agent
FOXTROT (status: IN FLIGHT)

## Experiment Design
```rust
// BigramHash embedding initialization
let vocab = 729; // 3^6
let hash_size = 729;
```

## Expected Outcome
BPB improvement from baseline by ~0.06 bits

## Status
🔴 IN FLIGHT (dispatched 2026-04-21)
