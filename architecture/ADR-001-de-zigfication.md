# ADR-001: De-Zigфикация — Tri as Source of Truth

**Status:** Accepted
**Date:** 2026-04-04
**Decision Makers:** Dmitrii Vasilev, Agent Army General

---

## Context

Trinity S³AI Framework (DOI: 10.5281/zenodo.19227879) is implemented as ~50,000 LOC of Zig monolith. This architectural decision shifts the paradigm:

- **Source of Truth**: `.tri`/.t27` specifications become canonical
- **Zig Role**: Demoted to backend/bootstrap layer
- **New Language**: `t27` established as independent canonical repo

---

## Problem Statement

1. **Agent Contamination** — AI agents see `.zig` files in context and continue writing Zig instead of `.tri` specs.
2. **Monolithic Coupling** — Math, formats, ISA, neural nets, FPGA, orchestration are interleaved in `src/`.
3. **Spec Drift** — Implementations ARE the specs (mutable), not following them.
4. **Language Lock-in** — Tri (B005) defined but never used; work remains in Zig.

---

## Decision

**Create `t27` as a new canonical repository** where:
1. `.tri`/`.t27` files are the SOLE source of truth
2. Zig is a generated compatibility backend (NOT a language to author in)
3. `zig-golden-float` becomes a seed/nursery for experiments
4. Every architectural step records to `experience/.trinity`

### Rationale

1. **Spec-First Development** — All new logic starts with `.tri` specification
2. **Generated Backend** — Zig code produced from `.tri` via `tri gen`
3. **Agent Cleanliness** — Agents work with `.tri` context → write `.tri` → de-zigfication accelerates
4. **Independent Versioning** — Each `.tri` spec library can version independently
5. **Canonical Reference** — `t27` provides language definition separate from implementation concerns

---

## Consequences

### Positive

- ✅ **Agent Decontamination** — Agents see `.tri` → write `.tri`
- ✅ **Modular Libraries** — `trinity/trinity-libs/*` can evolve independently
- ✅ **Spec Immutability** — `.tri` specs are the canonical record
- ✅ **Multi-Target Generation** — Zig, Verilog, C, Rust, Python from one `.tri`
- ✅ **Clear Architecture** — Language vs backend separation is explicit

### Negative

- ⚠️ **50,000 LOC Migration** — Existing Zig code must be preserved in `backend/zig/legacy/`
- ⚠️ **Dual Maintenance** — Both `.tri` specs AND legacy Zig need care during transition
- ⚠️ **Agent Re-training** — Agents must learn to check `.tri` context before writing Zig
- ⚠️ **Build Complexity** — `tri gen` pipeline requires tri-gen → Zig compiler integration

---

## Alternatives Considered

1. **Refactor in-place** — Rejected: Agents would still see `.zig` in context
2. **Rewrite in Rust** — Rejected: Different language, same problem
3. **Gradual Migration** — Rejected: Clean break needed for agent protocol
4. **Keep Zig, Add .tri as Docs** — Rejected: Specs must be executable, not documentation

---

## Implementation Plan

### Phase 1: Foundation (Q2 2026)

1. Create `t27/` repository structure
2. Write `CANON_DE_ZIGFICATION.md` (this ADR)
3. Write `architecture/graph.tri`
4. Establish `base/types.t27` + `base/ops.t27`
5. Set up `build.tri` (canonical, NOT `build.zig`!)

### Phase 2: Core Specs (Q2 2026)

1. `numeric/gf16.t27` — GoldenFloat16 from tri-formats
2. `numeric/tf3.t27` — TF3 format
3. `math/constants.t27` — Sacred constants
4. Conformance JSON vectors

### Phase 3-4: Compiler & Runtime (Q3 2026)

1. `compiler/parser/` — Minimal .t27 parser
2. `compiler/codegen/zig/` — .t27 → Zig generation
3. `compiler/runtime/` — T27 VM/allocator
4. `bindings/zig/` — Trinity interop

---

## References

- Tri Language (B005): https://doi.org/10.5281/zenodo.19227879
- Trinity repo: https://github.com/gHashTag/trinity
- Seed repo: https://github.com/gHashTag/zig-golden-float

---

**Approved by:** Dmitrii Vasilev, Agent Army General
**Date:** 2026-04-04
