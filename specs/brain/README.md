# specs/brain/ — Strand VI (neuroanatomical brain, spec-first)

**Source of truth:** `.t27` specifications only. Zig, C, and Verilog are **generated** via **`tri`** — see `docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md` §4.2.

**Do not add handwritten brain `*.zig` in the t27 repo** for semantics that belong here.

## Charter

[`docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md`](../../docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md) — scope (t27 vs trinity), 27 regions, φ invariants, rings 33–39, `tri brain` roadmap.

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

## Codegen (`tri`)

From repository root (after `cd bootstrap && cargo build --release`):

```bash
./scripts/tri gen-zig       specs/brain/unified_state.t27    # stdout
./scripts/tri gen-zig       specs/brain/                     # → gen/zig/brain/…
./scripts/tri gen-c         specs/brain/unified_state.t27
./scripts/tri gen-verilog   specs/brain/unified_state.t27
./scripts/tri seal          specs/brain/unified_state.t27 --save
./scripts/tri skill seal --hash specs/brain/unified_state.t27
./scripts/tri validate-conformance specs/brain/
./scripts/tri test
```

Project-wide: `./scripts/tri compile-project --backend zig -o build`.

**Note:** `./scripts/tri` is the committed CLI shim. A root `tri` binary may exist locally and is **gitignored**.

## Ownership

[`OWNERS.md`](OWNERS.md)

## Next steps

1. Region files under `cognitive/`, `limbic/`, `brainstem/` (27 total) + `tests/`
2. `api.t27` once Zig codegen supports `use` + `*Module.Type` + slices cleanly
3. `conformance/brain_*.json` + seals for every new spec
