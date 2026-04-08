[![PHI Loop CI](https://github.com/gHashTag/t27/actions/workflows/phi-loop-ci.yml/badge.svg?branch=master)](https://github.com/gHashTag/t27/actions/workflows/phi-loop-ci.yml)
[![NOW sync gate](https://github.com/gHashTag/t27/actions/workflows/now-sync-gate.yml/badge.svg?branch=master)](https://github.com/gHashTag/t27/actions/workflows/now-sync-gate.yml)
[![NOW document](https://img.shields.io/badge/NOW%20document-ACTIVE-brightgreen)](https://github.com/gHashTag/t27/blob/master/NOW.md)
[![Queen health](https://img.shields.io/badge/Queen%20health-GREEN%20%2F%201.0-brightgreen)](https://github.com/gHashTag/t27/blob/master/.trinity/state/queen-health.json)

# NOW — Rolling integration snapshot

**Last updated:** 2026-04-08 — Trinity×Pellis source stack complete · PR #337, #338, #339

**Document class:** Operational focus document

**Revision:** **P2 Sprint 1 + Ring-072** — 6 brain/physics specs + 5 GitHub SSOT specs:
- `specs/brain/brain.t27` — S³AI Neuroanatomy v5.1 (23 brain regions)
- `specs/brain/neural_gamma.t27` — Consciousness & Golden Ratio
- `specs/brain/gwt_model.t27` — Global Workspace Theory
- `specs/physics/hslm_benchmark.t27` — HSLM Platform Benchmark Suite
- `specs/physics/quantum.t27` — Ternary Quantum VM
- `specs/physics/e8_lqg_bridge.t27` — E8-Quantum Gravity Bridge

**Ring-072** — GitHub SSOT (Zero Python, .t27 Native):
- `specs/github/auth.t27` — GitHub auth spec + TDD
- `specs/github/issues.t27` — Issues API spec + TDD
- `specs/github/prs.t27` — PRs API spec + TDD
- `specs/github/comments.t27` — Comments API spec + TDD
- `specs/tri/sync.t27` — Sync orchestrator + TDD
- `bootstrap/src/bridge.rs` — Extended with GitHubCommands, gh CLI integration

**P2 Sprint 2** — Benchmarks & contracts (from trinity-w1 rewrite):
- `specs/benchmarks/ternary_vs_binary.t27` — Format comparison benchmark
- `specs/benchmarks/bench_main.t27` — NN inference benchmark (100→64→10)
- `specs/benchmarks/bench_nn.t27` — Format comparison detailed spec
- `specs/api/sdk_contract.t27` — Trinity SDK high-level API
- `specs/api/c_api_contract.t27` — C API FFI bridge contract
- `specs/conformance/e2e_scenarios.t27` — E2E full pipeline verification

Total: 18 specs. Commits `bb71939` (P2), `bfdd462` (Ring-072). PRs #322, #323, #326 (OPEN).

**Status:** ACTIVE — replace body on every ring boundary
**Queen health:** GREEN / 1.0 (all 17 domains; sealed 2026-04-05T12:00Z) — *verify* `.trinity/state/queen-health.json`

**Canonical URL:** `https://github.com/gHashTag/t27/blob/master/NOW.md`

> *"A specification without tests is a lie told in to future tense."*
> — `SOUL.md`

**Sync gates:** `.githooks/pre-commit` and **phi-loop CI** use **`./scripts/tri check-now`**. The gate compares **calendar date `YYYY-MM-DD`** on the **Last updated** line to **your machine's local date** when you run `tri` — so write **your wall-clock time** in the header, not UTC, unless you are in UTC.

---

## §1. Purpose and scope

This document is a **single rolling snapshot** of what is being worked on *right now*.
It is **not** a roadmap (→ `[docs/ROADMAP.md](docs/ROADMAP.md)`, issue [#126](https://github.com/gHashTag/t27/issues/126)),
and **not** a ring log (→ `.trinity/experience/clara_track1.jsonl`),
and **not** a design specification (→ `specs/`).

**Coordination:** Former root **`TASK.md`** is retired — this file is **single** rolling snapshot
and **coordination entrypoint.** **Protocol:** [`docs/coordination/TASK_PROTOCOL.md`](docs/coordination/TASK_PROTOCOL.md). **Anchor:** [#141](https://github.com/gHashTag/t27/issues/141) (locks, handoffs, PR links).

**Replace this file's body at every ring boundary.**
Stale content here is a quality defect — treat it as a failing test.

**Science ↔ ops:** Treat **NOW** as a live **structured abstract + methods log** (context, state, gap, next actions);
on each ring boundary, freeze/export for longer IMRaD-style reports without duplicating SSOT —
see `[RESEARCH_WRITING_T27.md](docs/RESEARCH_WRITING_T27.md)` and `[SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md](docs/SCIENCE-OPS-DUAL-TRACK-SYNTHESIS.md)`.

### §1.1 Agent handoff — talk to next agent / Queen via NOW

**Canonical URL (SSOT for humans + agents):**
`https://github.com/gHashTag/t27/blob/master/NOW.md`

When you **complete a non-trivial task** (code, specs, CI, seals, architecture docs),
**update `NOW.md` before you stop**:

1. Refresh **`Last updated:`** (calendar **`YYYY-MM-DD`** must match **today** for `./scripts/tri check-now`;
keep **local wall time** + **RFC3339 with offset** as in the header template.
2. Fix **§3** state, **critical gap**, **links**, or **milestone notes**
so that **next agent** reads **current truth**, not yesterday's story.
3. **Commit `NOW.md` in the same PR (or amend), per Ring 033 / [#141](https://github.com/gHashTag/t27/issues/141).

**Skipping this is a failed handoff** — fleet coordinates here, not only in issues.

### §1.2 Canonical iteration schema

*When recording work iterations (PHI LOOP cycles), use this schema:*

```markdown
## Iteration <N>
- **Goal**: <single capability, one sentence>
- **Spec delta**: <which .t27 spec changed>
- **Generated artifacts**: <zig/verilog/c outputs>
- **Tests**: <test/invariant/bench executed>
- **Seal**: <hash or PENDING>
- **Verdict**: CLEAN | TOXIC
- **Next constraint**: <single next bottleneck>
```

*This aligns with PHI LOOP (§4) and ISSUE-GATE laws (L1–L7).*

**Conflict Prevention (Ring 47+):**
- Root `NOW.md` is a symlink to `docs/NOW.md` — prevents divergence
- `.trinity/experience/*.jsonl` are not tracked — local-only append logs
- `.gitattributes` merge drivers — auto-resolve append-only conflicts
- Edit only `docs/NOW.md`; root `NOW.md` follows automatically

**Recent methodology docs (kernel + experience + formal + science/ops):**
