# specs/brain/ — Strand VI (neuroanatomical brain, spec-first)

**Source of truth:** `.t27` specifications only. Zig, C, and Verilog are **generated** by `t27c` — see `docs/TRINITY-BRAIN-NEUROANATOMY-TZ.md` §4.2.

**Do not add handwritten brain `*.zig` in the t27 repo** for semantics that belong here.

## Charter

[`docs/TRINITY-BRAIN-NEUROANATOMY-TZ.md`](../../docs/TRINITY-BRAIN-NEUROANATOMY-TZ.md) — scope (t27 vs trinity), 27 regions, φ invariants, rings 33–39.

## Target layout (EPIC-6)

```text
specs/brain/
├── unified_state.t27
├── cognitive_loop.t27
├── phi_timing.t27
├── api.t27                    # pending: cross-module / ptr codegen hardening
├── bus.t27
├── cognitive/                 # nine region specs
├── limbic/
├── brainstem/
├── periphery/
└── tests/
```

## Landed (P0 stubs)

- `unified_state.t27` — `BrainState`, φ constants, region counts
- `phi_timing.t27` — five phases, float TRINITY sum test
- `bus.t27` — bus version contract
- `cognitive_loop.t27` — five-phase loop identity

## Codegen

```bash
# From repo root, after: (cd bootstrap && cargo build --release)
./bootstrap/target/release/t27c gen           specs/brain/unified_state.t27
./bootstrap/target/release/t27c gen-c         specs/brain/unified_state.t27
./bootstrap/target/release/t27c gen-verilog specs/brain/unified_state.t27
./bootstrap/target/release/t27c seal        specs/brain/unified_state.t27 --save
```

Project-wide: `t27c compile-project --backend zig -o build` (all `specs/` + `compiler/`).

## Ownership

[`OWNERS.md`](OWNERS.md)

## Next steps

1. Region files under `cognitive/`, `limbic/`, `brainstem/` (27 total) + `tests/`
2. `api.t27` once Zig codegen supports `use` + `*Module.Type` + slices cleanly
3. `conformance/brain_*.json` + seals for every new spec
