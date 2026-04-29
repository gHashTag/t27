# t27 Technology Tree

**Source of truth:** `.trinity/experience/` + sealed specs in `specs/`
**Last updated:** 2026-04-29

---

## Level 0 — Core: `.t27` spec → `t27c` parser → hash seal

| Component | Status | Spec | Evidence |
|-----------|--------|------|----------|
| `.t27` syntax parser | Done | `specs/isa/*.t27` | 15 ISA specs |
| `t27c` compiler | Done | `bootstrap/src/` | Rust binary |
| Hash seal system | Done | `.trinity/seals/` | 30+ seal files |
| L1-L7 constitutional checks | Done | `scripts/pre-commit` | PR #554 |
| FFI encode/decode (7 formats) | Done | `ffi/src/lib.rs` | PR #553 |
| Golden float formats (GF4-GF32) | Done | `specs/numeric/*.t27` | 7 format specs |

## Level 1 — Runtime: pipeline → gen(Zig/C/Verilog) → validate

| Component | Status | Spec | Evidence |
|-----------|--------|------|----------|
| `tri` CLI multiplexer | Done | `bootstrap/src/main.rs` | t27c binary |
| Spec generation pipeline | Done | `bootstrap/src/gen_*.rs` | Multiple gen modules |
| `tri math compare` | Done | `bootstrap/src/math_compare.rs` | v1 + v2 hybrid |
| Conformance testing | Done | `conformance/` | Conformance vectors |
| Pipeline E2E spec | Done | `specs/pipeline/e2e_test.t27` | 9-stage pipeline |
| Code gen targets | Partial | Zig primary | C/Verilog planned |

## Level 2 — Memory: experience save → mistakes/ → skill evolution

| Component | Status | Spec | Evidence |
|-----------|--------|------|----------|
| Experience JSONL logging | Done | `.trinity/experience/` | math_compare.jsonl |
| Memory primitives spec | Planned | `specs/memory/memory_primitives.t27` | #517 |
| NotebookLM integration | Done | `specs/memory/notebooklm.t27` | #305 |
| Wrap-up skill | Done | `.claude/skills/wrap-up/` | #304 |
| Session context extraction | Done | `contrib/backend/notebooklm/wrapup.py` | Auto-wrapup |
| Semantic search | Done | `specs/memory/semantic_search.t27` | 140 lines |
| Formula embedding | Done | `specs/memory/formula_embed.t27` | 201 lines |
| Mistake tracking | Planned | `.trinity/mistakes/` | #490 |

## Level 3 — Swarm: 32 agents → shared experience → collective IQ

| Component | Status | Spec | Evidence |
|-----------|--------|------|----------|
| Agent alphabet (27 letters) | Done | `docs/agents/AGENTS_ALPHABET.md` | T,N,P,C,B roles |
| Multi-agent coordination | Done | `docs/coordination/TASK_PROTOCOL.md` | Lock system |
| Domain ownership | Done | `OWNERS.md` (recursive) | Per-directory |
| Shared `.trinity/` state | Done | `.trinity/state/` | Active skill, roads |
| Agent task system | Done | `TASK.md` | Lock files |

## Level 4 — Unfair advantage: ASHA+PBT evolution → competitive edge

| Component | Status | Spec | Evidence |
|-----------|--------|------|----------|
| Hybrid v2 golden tests | Done | `bootstrap/src/math_compare.rs` | N=5..152 |
| GF competitive analysis | Done | `specs/numeric/gf_competitive.t27` | #289 |
| Pellis verification | Done | `specs/numeric/pellis_verify.t27` | 100-digit mpmath |
| Cross-language benchmarks | Planned | `benchmarks/language_tests/` | #289 Phase 3 |
| Whitepaper | Planned | `docs/WHITEPAPER/` | #289 Phase 4 |

---

## Key Metrics

| Metric | Value |
|--------|-------|
| Total specs | 30+ `.t27` files |
| Tests (in specs) | 100+ test blocks |
| Invariants | 50+ invariant blocks |
| Benchmarks | 30+ bench blocks |
| Sealed artifacts | 30+ in `.trinity/seals/` |
| Open issues | 2 (from 48 peak) |
| Constitutional laws | L1-L8 (8 invariant laws) |

---

**phi^2 + 1/phi^2 = 3 | TRINITY**
