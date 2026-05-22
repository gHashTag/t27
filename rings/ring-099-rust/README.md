# ring-099-integration

**Final import of the Wave-11 series.** With this crate, every Wave-11 ring
has real source on disk, every cross-kernel anchor is live, and the
Wave-11 `claimed-only` table in `rings/COMPILE_STATUS.md` is empty.

Bounded, no_std, no-heap 10-stage End-to-End pipeline state machine,
mirroring `specs/pipeline/e2e_test.t27` byte-for-byte.

**Anchor:** `phi^2 + 1/phi^2 = 3`

## What it mirrors

| Backing spec                        | Imports                                                                       |
|-------------------------------------|-------------------------------------------------------------------------------|
| `specs/pipeline/e2e_test.t27`       | `MAX_PIPELINE_STAGES = 10`, `STAGE_INIT`..`STAGE_DONE` (0..8), `STAGE_FAIL = 255`, `pipeline_run`, `pipeline_inject_failure`, `pipeline_progress`, `stage_name`, the 4 spec test blocks, the 3 spec invariants |

All spec constants are mirrored byte-for-byte. Buffers are fixed-size
`[u8; MAX_PIPELINE_STAGES]` and `[bool; MAX_PIPELINE_STAGES]`; no
allocator is used.

## Surface

- `Stage` enum (9 valid stages + `Fail`) with `code()`, `from_code(u8)`,
  `next()`, `is_terminal()`, `name()`
- `Pipeline` -- bounded state machine with:
  - `run()` -- full sequence to `Done`
  - `inject_failure(fail_at)` -- short-circuit failure
  - `verify()` -- enforces the 3 spec invariants at runtime
  - `reset()`, `current()`, `count()`, `stage_at(i)`, `result_at(i)`
- Free functions matching the spec surface exactly:
  - `pipeline_run(stages, results, count)`
  - `pipeline_inject_failure(fail_at, stages, results, count)`
  - `pipeline_progress(completed, total)`
  - `stage_name(stage)`
- `InvariantStatus::{Ok, OrderingViolated, MaxStagesTooSmall, FailNotDistinct}`

## Cross-kernel anchor (#11)

`integration_phi_identity` routes `phi^2 + 1/phi^2 = 3` through:

1. **Integer projection** -- `floor(PHI) + floor(PHI_SQ) = 1 + 2 = 3`
2. **`pow_u64` numeric witness** -- `3^1 == identity_witness()`
3. **Pipeline progress arithmetic** -- a full 9-stage run yields exactly
   `100.0%`, and `pipeline_progress(3, 9) == 100/3` to within `1e-9`
4. **Mass conservation** -- `PHI_SQ + PHI_INV_SQ == TRINITY` (within `1e-12`)

This is the **11th** and final cross-kernel anchor in the Wave-11 series.

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
- **L4** -- 32 `#[test]` blocks.
- **L5** -- identity exercised explicitly.
- **L6** -- no numeric kernel/spec changes; constants mirrored byte-for-byte.
- **L7** -- no shell scripts.

`phi^2 + 1/phi^2 = 3`
