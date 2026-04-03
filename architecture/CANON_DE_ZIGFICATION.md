# CANON — De-Zigfication

**Canonical Law:** This document establishes the principle that de-zigifies Trinity, making `.tri`/.t27` the source of truth for all numeric/formal logic.

---

## Context

Trinity S³AI Framework (DOI: 10.5281/zenodo.19227879) was implemented as a monolithic Zig repository (~50,000 LOC). This created problems:

1. **Agent Contamination** — AI agents see `.zig` files in context and continue writing Zig code instead of `.tri` specs.
2. **Monolithic Coupling** — Math, formats, ISA, neural nets, FPGA, orchestration are interleaved in a flat `src/` directory.
3. **Spec Drift** — No single source of truth; implementations ARE the specs (mutable).
4. **Language Lock-in** — Tri language (B005) is defined, but all real work remains in Zig.

---

## The Law

### Rule 1: Source of Truth

All new mathematical, numerical, and formal logic MUST be authored in `.tri` or `.t27` files.

**Consequences of violation:**
- Any agent seeing `.zig` context MUST write `.tri` instead
- Direct `.zig` edits for math/formal logic are blocked
- Review must fail if implementation violates this law

### Rule 2: Zig as Backend

Zig code is permitted ONLY for:
- Bootstrap/runtime layer
- Generated code from `.tri` specifications
- Compatibility shim for legacy code
- Hardware bridge (FPGA, bindings)

**Consequences of violation:**
- Backend-generated `.zig` files cannot be edited manually (except build infrastructure)
- Spec-first pattern enforced: `.tri` → compiler → backend

### Rule 3: Trinity Libraries Canonical

The canonical Trinity library structure is:

```
trinity/trinity-libs/
├── tri-core/      ← Trit, PackedTrit, TernaryWord
├── tri-math/      ← Constants, identities, sacred math
├── tri-formats/    ← GF16, TF3, encode/decode
├── tri-jit/        ← TRI-27 runtime
├── tri-fpga/       ← FPGA integration
└── ...
```

---

## Agent Protocol

When an AI agent writes code:

1. **Check source**: Is there an existing `.tri` spec for this logic?
2. **Use existing spec**: If yes, edit the `.tri` file (not create new Zig).
3. **Create new spec**: If no spec exists, create `.tri` first, then generate Zig.
4. **Never write Zig directly**: For new math/formal logic, always start with `.tri`.

---

## Exceptions

**Bootstrap code**: `build.zig`, `build.zig.zon`, entrypoints may be edited in Zig directly.

**Legacy shim**: Code in `backend/zig/legacy/` is preserved for compatibility.

**Hardware bridge**: FPGA-specific bindings may use `.zig` directly but depend on `.tri` specs.

---

## Migration Status

- [x] `t27` canonical structure defined
- [x] `trinity/trinity-libs/tri-core` is source of truth
- [ ] Migration of `trinity/trinity-libs/tri-math` to `.tri` specs
- [ ] Migration of `trinity/trinity-libs/tri-formats` to `.tri` specs
- [ ] `tri dev scan` enforces `.tri` context
- [ ] All agents trained to check `.tri` context before writing code

---

**Approved by:** Dmitrii Vasilev, Agent Army General
**Date:** 2026-04-04
