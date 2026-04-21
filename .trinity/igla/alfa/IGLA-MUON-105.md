# IGLA-MUON-105: Muon WD=0.04

## Configuration
- Optimizer: Muon
- Weight decay: 0.04
- Target ΔBPB: -0.02

## Agent
ALFA (status: IN FLIGHT)

## Experiment Design
```rust
// Muon optimizer with specific WD
let optimizer = Muon::new(
    lr = 3e-4,
    wd = 0.04,
);
```

## Expected Outcome
BPB improvement from baseline

## Status
🔴 IN FLIGHT (dispatched 2026-04-21)
