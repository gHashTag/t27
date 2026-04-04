# ADR-001: De-Zigфикация — Tri as Source of Truth

**Status:** Accepted
**Date:** 2026-04-04
**Last Updated:** 2026-04-04
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

### Phase 1: Foundation (Q2 2026) ✅ COMPLETE

1. ✅ Create `t27/` repository structure
2. ✅ Write `CANON_DE_ZIGFICATION.md` (this ADR)
3. ✅ Write `architecture/graph.tri` and `architecture/graph_v2.json`
4. ✅ Establish `specs/base/types.t27` + `specs/base/ops.t27` in canonical format
5. ⏳ Set up `build.tri` (canonical, NOT `build.zig`!)

### Phase 2: Core Specs (Q2 2026) ✅ COMPLETE

1. ✅ `specs/numeric/gf16.t27` — GoldenFloat16 standardized
2. ✅ `specs/numeric/tf3.t27` — TF3 format standardized
3. ✅ `specs/math/constants.t27` — Sacred constants standardized
4. ✅ `specs/math/sacred_physics.t27` — Sacred physics standardized
5. ✅ `specs/numeric/gf4.t27`, `gf8.t27`, `gf12.t27`, `gf20.t27`, `gf24.t27`, `gf32.t27` — Full GoldenFloat family
6. ✅ `specs/numeric/phi_ratio.t27` — Phi ratio split formula
7. ✅ Conformance JSON vectors

### Phase 3: VSA, ISA, FPGA, NN (Q2 2026) ✅ COMPLETE

1. ✅ `specs/vsa/ops.t27` — VSA operations standardized
2. ✅ `specs/isa/registers.t27` — ISA registers standardized
3. ✅ `specs/fpga/mac.t27` — FPGA MAC standardized
4. ✅ `specs/nn/attention.t27` — Sacred attention standardized
5. ✅ `specs/nn/hslm.t27` — HSLM architecture standardized

### Phase 4: Compiler & Runtime (Q2 2026) ✅ COMPLETE

1. ✅ `compiler/parser/lexer.t27` — Lexer standardized
2. ✅ `compiler/parser/parser.t27` — Parser present
3. ✅ `compiler/codegen/zig/codegen.t27` — .t27 → Zig generation
4. ✅ `compiler/codegen/verilog/codegen.t27` — .t27 → Verilog generation
5. ✅ `compiler/codegen/c/codegen.t27` — .t27 → C generation
6. ✅ `compiler/runtime/runtime.t27` — T27 VM/allocator
7. ✅ `compiler/codegen/zig/runtime.t27` — Zig runtime backend

### Phase 5: Queen Orchestrator (Q2 2026) ✅ COMPLETE

1. ✅ `specs/queen/lotus.t27` — 6-phase orchestration standardized

**PHI LOOP Skills 017-031 Completed:**
- 15 skills covering standardization of 13 specs + 2 architecture files
- All .t27 files now in canonical .t27 syntax (module/fn/test/invariant/bench)
- `architecture/graph.tri` and `architecture/graph_v2.json` synchronized

---

## References

- Tri Language (B005): https://doi.org/10.5281/zenodo.19227879
- Trinity repo: https://github.com/gHashTag/trinity
- Seed repo: https://github.com/gHashTag/zig-golden-float

---

**Approved by:** Dmitrii Vasilev, Agent Army General
**Date:** 2026-04-04
