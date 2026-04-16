# CLAUDE.md — Instructions for Claude Code and autonomous agents (t27)

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Core Architecture

Trinity S³AI is a **spec-first** language system where:

```
.t27 (human source) → [t27c compiler] → .tri (canonical IR) → [tri runtime] → execution
```

**Critical invariant:** `.t27` compiles **itself via .tri** — there is NO "dogfooding" or self-feeding.

---

## Language Food Chain

| Language | Status | Role |
|----------|--------|------|
| `.t27` | ✅ write by hand | Single source of truth |
| `.tri` | ⚙️ generated | Canonical IR, runtime target |
| Rust | 🔒 frozen | Only `t27c` bootstrap compiler |
| Zig | ❌ consumed | Generate from `.tri` via `tri gen-zig`, never write by hand |
| Python | ❌ consumed | Forbidden by L4 (except legacy) |
| C, Verilog | ❌ consumed | Generate from `.tri` via `tri gen-c` / `tri gen-verilog` |

**Golden rule:** Write ONLY `.t27` files. All other artifacts are generated.

---

## Common Commands

```bash
# Build bootstrap compiler (Rust)
cd bootstrap && cargo build --release

# Parse a .t27 spec
./scripts/tri parse <spec.t27>

# Generate Zig backend
./scripts/tri gen-zig <spec.t27>        # Single file
./scripts/tri gen-zig <directory>           # Batch

# Generate C backend
./scripts/tri gen-c <spec.t27>

# Generate Verilog backend
./scripts/tri gen-verilog <spec.t27>

# Verify a seal
./scripts/tri seal <spec.t27> --verify

# Run all tests
./scripts/tri test

# Validate conformance vectors
./scripts/tri validate-conformance
```

---

## Project Structure

```
t27/
├── specs/              # .t27 SPECIFICATIONS — source of truth
│   ├── base/         # types, ops, constants
│   ├── numeric/       # GoldenFloat GF4-GF32, TF3, phi
│   ├── math/          # sacred_physics, constants
│   ├── compiler/      # parser, codegen, CLI
│   ├── fpga/          # MAC unit, Verilog specs
│   └── ...
│
├── gen/               # GENERATED backends — DO NOT EDIT
│   ├── zig/
│   ├── c/
│   └── verilog/
│
├── bootstrap/          # Stage-0 compiler (Rust) — FROZEN
│   └── src/compiler.rs  # SHA-256 sealed in bootstrap/stage0/FROZEN_HASH
│
├── conformance/       # Language-agnostic test vectors (JSON)
├── architecture/        # Dependency graph + ADRs
└── docs/              # First-party documentation
```

**NEVER edit files under `gen/` — regenerate from specs instead.

---

## Constitutional Laws (L1-L7)

| Law | Name | Summary |
|-----|------|---------|
| L1 | TRACEABILITY | No code merged without `Closes #N` |
| L2 | GENERATION | Files under `gen/` are generated; edit `.t27` spec instead |
| L3 | PURITY | All source files must be ASCII-only with English identifiers |
| L4 | TESTABILITY | Every `.t27` spec must contain `test`/`invariant`/`bench` |
| L5 | IDENTITY | φ² = φ + 1 on ℝ; φ² + φ⁻² = 3; IEEE f64 checks use tolerance |
| L6 | CEILING | `FORMAT-SPEC-001.json` + `gf16.t27` are numeric SSOT |
| L7 | UNITY | No new `*.sh` on critical path; use `tri`/`t27c` only |

**Law priority:** L1 > L2 > L3 > L4 > L5 > L6 > L7

Details: See `docs/T27-CONSTITUTION.md` and root `SOUL.md`.

---

## SEED-RINGS Progress

The compiler grows ring-by-ring. Each ring adds exactly one capability, sealed with SHA-256 hashes.

| Ring | Capability | Status |
|------|-----------|--------|
| 0-8 | Base types, numeric ops, sacred physics | SEED |
| 9-11 | Compiler: parser → codegen → Zig/C/Verilog | SEED |
| 12-14 | FPGA: MAC unit, Verilog gen, bitstream | SEED |
| 15-17 | Queen + NN orchestration, AR modules | SEED |
| 18-24 | CLARA AR pipeline (ternary logic, Datalog, etc.) | AR |
| 25-31 | Gen backends for all domains | GEN |

Current: Ring 31 Complete (all 31 rings sealed)

---

## PHI LOOP Workflow

Every change follows 9-step cycle:

```
Issue → Spec → TDD → Code → Gen → Seal → Verify → Land → Learn
```

```bash
tri skill begin <task> --issue <N>
tri spec edit <module>                # Edit ONE .t27 spec
tri skill seal --hash                 # Record 4 SHA-256 hashes
tri gen                               # Generate Zig/Verilog/C
tri test                              # Run tests
tri verdict --toxic                   # TOXIC? → rollback
tri experience save                   # Append episode to memory
tri skill commit                      # Verify hashes + issue binding
tri git commit                        # Push with "Closes #N"
```

---

## Multi-Agent Coordination

- Anchor issue: https://github.com/gHashTag/t27/issues/141
- See `TASK.md` + `docs/coordination/TASK_PROTOCOL.md` for coordination protocol
- Set locks before editing hot paths; release with handoff log when done

---

## Where to Find Specs

All `.t27` specifications live under `specs/`:

| Directory | Content |
|-----------|----------|
| `specs/base/` | Types, operations, constants |
| `specs/numeric/` | GoldenFloat GF4-GF32, TF3, phi_ratio |
| `specs/math/` | Sacred physics constants |
| `specs/compiler/` | Parser, codegen, CLI, runtime specs |
| `specs/fpga/` | MAC unit, board specs |
| `specs/nn/` | HSLM, attention kernels |
| `specs/ar/` | CLARA AR modules (logic, proof, datalog) |

---

## Read Order

1. `AGENTS.md` — Entry point and constitutional stack
2. `SOUL.md` — Canonical constitution (root wins over docs/nona-03-manifest/SOUL.md)
3. `docs/T27-CONSTITUTION.md` — SSOT-MATH, LANG-EN, DOCS-TREE
4. `TASK.md` + `docs/coordination/TASK_PROTOCOL.md` — Multi-agent coordination
5. Nearest `OWNERS.md` for directories you edit

---

## Important ADRs

| ADR | Topic |
|-----|-------|
| ADR-004 | Language Policy (ASCII + English) |
| ADR-005 | Language Food Chain (t27 → .tri → backends) |
| ADR-001 | De-Zigfication |

---

## φ² + 1/φ² = 3 | TRINITY
