# NOW -- M5 freeze: module var arrays writable, qualified calls skip namesakes (2026-10-02)

## What changed

- bootstrap/src/compiler.rs: a module-level `var` array is registered with `extra_mutable` instead of as const, so W456's ROM rule no longer rejects element writes (disjoint_set.t27).
- bootstrap/src/use_resolve.rs: a qualified CALL whose local namesake disagrees with every imported declaration of that name keeps its qualifier, so the arity check is skipped by non-match instead of running against a namesake (mha_block.t27).
- bootstrap/stage0/FROZEN_HASH: resealed via `t27c frozen-digest` -> b45a356c2eb651059e73d93e913558131b303d003e80f5d6642d3ddab7dd3526.
- docs/reports/suite_expectations.json: the two entries (issues 5573, 5574) are removed; both specs now pass typecheck.

## Verification

- `cargo test --release -p t27c --bin t27c`: 1750 passed, 0 failed, 2 ignored.
- `t27c typecheck specs/ml/transformer/mha_block.t27`: OK (was 2 errors); `specs/tri/graph/disjoint_set.t27`: OK (was 1 error); `specs/igla/coder/dataset.t27`: OK (no regression).
