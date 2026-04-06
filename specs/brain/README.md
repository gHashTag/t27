# specs/brain/ — Strand VI (neuroanatomical brain, spec-first)

This directory will hold **`.t27` specifications** for the Trinity **unified brain** model (27 regions, φ-structured loop, periphery API).

**Do not add hand-written Zig/C/Verilog here.** Generated output lives under `gen/` after `t27c gen-*`.

## Charter

See **[`docs/TRINITY-BRAIN-NEUROANATOMY-TZ.md`](../../docs/TRINITY-BRAIN-NEUROANATOMY-TZ.md)** for:

- Scope split between **`gHashTag/t27`** (specs) and **`gHashTag/trinity`** (runtime integration)
- Target file tree (`unified_state`, `cognitive_loop`, `phi_timing`, `api`, `bus`, `cognitive/`, `limbic/`, `brainstem/`)
- Proposed SEED rings 33–39 and φ invariants

## Ownership

When the first `.t27` file lands, add **`OWNERS.md`** here; until then, see root **[`OWNERS.md`](../../OWNERS.md)** and **[`specs/OWNERS.md`](../OWNERS.md)** (Queen / physics / numeric agents by subdomain).

## Next steps (implementation order)

1. `unified_state.t27`, `bus.t27`, `api.t27` (P0)
2. `phi_timing.t27`, `cognitive_loop.t27` (P0)
3. Layer 1 → 2 → 3 region specs (P1/P2)
4. `conformance/brain_*.json` + seals
