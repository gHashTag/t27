# ring-098-world-model

Bounded internal **World Model** for the Trinity agent: a no_std, no-heap
recorder of brain-state snapshots and (state, action, reward, next_state, done)
transitions, advanced one cognitive phase at a time.

**Anchor:** `phi^2 + 1/phi^2 = 3`

## What it mirrors

| Backing spec                              | Imports                                                                 |
|-------------------------------------------|-------------------------------------------------------------------------|
| `specs/brain/unified_state.t27`           | `BrainState`, `ConsciousnessState`, `Mood`, `ArousalLevel`, `Layer`, `REGION_COUNT = 27`, `LAYER_COUNT = 3`, `REGIONS_PER_LAYER = 9`, `PHI` / `PHI_INV` / `PHI_SQ` / `PHI_INV_SQ` / `TRINITY` constants |
| `specs/ml/rl/dqn.t27`                     | `Transition { state, action, reward, next_state, done }`                |
| `specs/brain/cognitive_loop.t27`          | `COGNITIVE_PHASE_COUNT = 5` (sense, evaluate, decide, act, consolidate) |

All spec-pinned constants are mirrored **byte-for-byte**. State vectors
are stored inline at fixed dimension `STATE_DIM = 8`. Buffer capacities
are `MAX_STATE_HISTORY = 16` and `MAX_TRANSITIONS = 32`.

## Surface

- `BrainState`, `ConsciousnessState`, `Mood`, `ArousalLevel`, `Layer`, `Phase`
- `Transition`
- `WorldModel` -- bounded model with:
  - `snapshot()`, `record_transition(...)`, `step_phase()`, `run_one_cycle()`
  - `verify()` -- checks monotonic `cycle_count` and `phi_coherence in [0, 1]`
  - `reset()`, `state_at(i)`, `transition_at(i)`
- `WorldModelError::{TransitionBufferFull, StateBufferFull}`
- `VerifyStatus::{Valid, Empty, BadPhiCoherence(i), NonMonotonicCycle(i)}`

## Cross-kernel anchor (#10)

`world_model_phi_identity` routes `phi^2 + 1/phi^2 = 3` through:

1. **Integer projection** -- `floor(PHI_SQ) + floor(PHI) = 2 + 1 = 3`
2. **`pow_u64` numeric witness** -- `3^1 == identity_witness()`
3. **Mass conservation** -- `PHI_SQ + PHI_INV_SQ` equals `TRINITY = 3.0` within fp epsilon (chains back to ring-088 GF16 MAC)

## Build

```
cargo check --all-targets
cargo test --lib
```

Tested locally on Rust 1.83.0.

## Constitutional notes

- **L1** -- traceability: this crate exists for issue/PR-tracked work.
- **L2** -- zero edits under `gen/`, `coq/`, `proofs/`, `bootstrap/`, `specs/`, `conformance/`, `architecture/`.
- **L3** -- ASCII source, English doc-comments.
- **L4** -- 27 `#[test]` blocks.
- **L5** -- identity exercised explicitly.
- **L6** -- no numeric kernel/spec changes; constants mirrored byte-for-byte.
- **L7** -- no shell scripts.

`phi^2 + 1/phi^2 = 3`
