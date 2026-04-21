# IGLA-BGH-302: BigramHash(10240) + SmearGate

## Configuration
- Vocab: 3^6 = 729
- Hash size: 10240 (scaled up)
- SmearGate: enabled
- Target ΔBPB: -0.04

## Agent
FOXTROT (status: IN FLIGHT)

## Experiment Design
```rust
// BigramHash with SmearGate
let vocab = 729;
let hash_size = 10240;
let smear_gate = true;
```

## Expected Outcome
Additional BPB improvement from IGLA-BGH-301

## Status
🔴 IN FLIGHT (dispatched 2026-04-21)
