# Ring-017: tri CLI v0.2.0 — COMPLETE

## Summary

Created unified `tri` CLI binary v0.2.0 that:
- Is a workspace member (built by `cargo build --workspace`)
- Provides ALL commands: status, skill, cell, gen, test, verdict, experience, doctor, health
- Plus experimental commands: pipeline, bench, parse
- Modular command structure with clear stability boundaries

## What Was Done

1. ✅ Added `cli/tri` to workspace members (Cargo.toml)
2. ✅ Created modular command structure:
   - `cli/tri/src/commands/mod.rs` — Module entry
   - `cli/tri/src/commands/pipeline.rs` — Pipeline command
   - `cli/tri/src/commands/bench.rs` — Benchmark command
   - `cli/tri/src/commands/parse.rs` — Parse command
   - `cli/tri/src/commands/experimental.rs` — Banner module
3. ✅ Refactored `cli/tri/src/main.rs` to use modular commands
4. ✅ Added legacy deprecation comment to `bootstrap/src/cli.rs`
5. ✅ Updated `docs/NOW.md` with public entrypoints documentation
6. ✅ Built successfully: `cargo build --workspace --bin tri`
7. ✅ Installed globally: `cargo install --path cli/tri`
8. ✅ Verified on PATH: `which tri`

## Verification Results

```bash
$ tri --help
# Shows all 11 commands with experimental banner on stub commands

$ tri pipeline specs/00-gf-family-foundation.tri
# Shows experimental banner, prints stub message

$ tri status
# Works (existing stable command)

$ tri gen specs/00-gf-family-foundation.tri
# Works (existing stable command, calls t27c)
```

## Architecture

```
cli/tri/src/
├── main.rs              # Entry point with Clap derive
└── commands/
    ├── mod.rs          # Module declarations
    ├── pipeline.rs     # Stub command (experimental)
    ├── bench.rs        # Stub command (experimental)
    ├── parse.rs        # Stub command (experimental)
    └── experimental.rs  # Banner module
```

## Stability Boundaries

**STABLE (Production-ready):**
- `status` — PHI LOOP status
- `skill` — Skill management (begin/end)
- `cell` — Cell management (checkpoint/seal)
- `gen` — Generate code (calls t27c)
- `test` — Run tests (calls t27c)
- `verdict` — Run validation (calls t27c)
- `experience` — Experience save
- `doctor` — Doctor service
- `health` — Health check

**EXPERIMENTAL (Stubs pending Ring-018/019):**
- `pipeline` — Full E2E pipeline
- `bench` — Benchmark runner
- `parse` — .tri syntax validation

## Next Steps

- Ring-018: Integrate t27c parser/codegen into experimental commands
- Ring-019: Remove legacy `bootstrap/src/cli.rs` after full integration

## Legacy Path

`bootstrap/src/cli.rs` is now marked as legacy. Do not use directly — use `tri` instead.
