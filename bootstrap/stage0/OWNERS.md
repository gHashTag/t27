# OWNERS — bootstrap/stage0/

## Primary

**B-Builder** — stage-0 bootstrap pipeline; **S-Seal** — integrity of `FROZEN_HASH`.

## Contents

- `FROZEN_HASH` — SHA-256 seal of the compiler snapshot for the current ring baseline (see `docs/SEED-RINGS.md`, `docs/GOLDEN-RINGS-CANON.md`).

## Dependencies

- `bootstrap/src/compiler.rs` (hashed artifact).
