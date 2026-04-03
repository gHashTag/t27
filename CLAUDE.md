# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`t27` is the canonical specification repository for the Trinity S³AI Framework. This is a **spec-first** project where `.tri` and `.t27` files are the sole source of truth for all mathematical, numerical, and formal logic. Zig code is a generated backend, not a language to author in.

**Core Principle: De-Zigfication** — AI agents must see `.tri` context and write `.tri` files, never Zig directly (see `architecture/CANON_DE_ZIGFICATION.md` and `architecture/ADR-001-de-zigfication.md`).

## Agent Protocol

When working in this repository, **always**:

1. **Check source**: Is there an existing `.tri` spec for this logic?
2. **Use existing spec**: If yes, edit the `.tri` file (not create new code)
3. **Create new spec**: If no spec exists, create `.tri` first, then consider generation
4. **Never write Zig directly**: For new math/formal logic, always start with `.tri`

**Zig is ONLY permitted for**:
- Bootstrap/runtime layer
- Generated code from `.tri` specifications
- Legacy compatibility shim in `backend/zig/legacy/`
- Hardware bridge (FPGA, bindings)

## T27 Language Basics

`.t27` files use a specification syntax similar to Zig but focused on formal definitions:

```t27
// Types
pub const Trit: type = enum(u2) {
    neg = 0b10,  // -1
    zero = 0b00, // 0
    pos = 0b01,  // +1
};

// Functions
pub fn tritAdd(a: Trit, b: Trit, carry: *Trit) Trit {
    const sum = @as(u2, @intFromEnum(Trit, a)) +% @as(u2, @intFromEnum(Trit, b));
    carry.* = @as(Trit, @intFromEnum(Trit, sum > 1));
    return @as(Trit, @intFromEnum(Trit, sum));
}

// Imports
using base: @import("types.zig");
```

## Architecture Structure

The codebase is organized by **tiers** (0-4) with clear dependency boundaries. `architecture/graph.tri` is the single source of truth for module relationships.

### Tiers

- **Tier 0**: Base types (`tritype-base`) — `Trit`, `PackedTrit`, `TernaryWord`
- **Tier 1**: Core arithmetic (`tritype-numeric`), numeric formats (`triformat-gf16`, `triformat-tf3`), sacred math (`trisacred-constants`, `trisacred-gamma`)
- **Tier 2**: VSA primitives (`trivsa-ops`), ISA (`triisa-registers`), attention (`triatt-sacred`), HSLM (`trinhslm`), FPGA MAC (`trifpga-mac`)
- **Tier 3**: Orchestrator (`triquenn`)
- **Tier 4**: Language tooling (`trilang-cli`)

### Directory Layout

```
t27/
├── specs/           # All .tri/.t27 specifications (source of truth)
│   └── base/       # Tier 0 base types and ops
├── architecture/     # Design documents and module graph
│   ├── graph.tri   # Module dependency graph (canonical)
│   └── ADR-*.md    # Architecture decision records
└── backend/        # Generated code (Zig, Verilog, C) — DO NOT EDIT
```

### Module Dependencies

When creating or modifying specs, check `architecture/graph.tri` for the `deps` array:
- Lower tier specs must not depend on higher tiers
- Modules only declare dependencies they actually use
- Circular dependencies are prohibited

## Core Types

- **Trit**: Ternary digit with values `-1`, `0`, `+1`
- **PackedTrit**: 2 trits per byte (compact representation)
- **TernaryWord**: 24 trits (3 bytes, fits in `u32`)

## Migration Status

- [x] `t27` canonical structure defined
- [x] `trinity/trinity-libs/tri-core` is source of truth
- [ ] Migration of `trinity/trinity-libs/tri-math` to `.tri` specs
- [ ] Migration of `trinity/trinity-libs/tri-formats` to `.tri` specs
- [ ] `tri dev scan` enforces `.tri` context
- [ ] All agents trained to check `.tri` context before writing code

## References

- Tri Language (B005): https://doi.org/10.5281/zenodo.19227879
- Trinity repo: https://github.com/gHashTag/trinity
- Seed repo: https://github.com/gHashTag/zig-golden-float
