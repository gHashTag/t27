<p align="center">
  <strong>φ² + 1/φ² = 3 | TRINITY</strong>
</p>

<h1 align="center">t27 — TRI-27 Spec-First Language</h1>

<p align="center">
  The canonical source of truth for Trinity S³AI.<br>
  <code>.t27</code> specs in → Zig, Verilog, C out.
</p>

<p align="center">
  <a href="https://github.com/gHashTag/t27/actions"><img src="https://img.shields.io/github/actions/workflow/status/gHashTag/t27/ci.yml?branch=master&label=CI&style=flat-square" alt="CI"></a>
  <a href="https://github.com/gHashTag/t27/issues"><img src="https://img.shields.io/github/issues-raw/gHashTag/t27?style=flat-square&color=D93F0B&label=open%20issues" alt="Issues"></a>
  <a href="https://github.com/gHashTag/t27/pulls"><img src="https://img.shields.io/github/issues-pr-raw/gHashTag/t27?style=flat-square&label=open%20PRs" alt="PRs"></a>
  <img src="https://img.shields.io/badge/rings-15%20of%2017-blueviolet?style=flat-square" alt="Rings">
  <img src="https://img.shields.io/badge/specs-45%20.t27-blue?style=flat-square" alt="Specs">
  <img src="https://img.shields.io/badge/agents-27-gold?style=flat-square" alt="Agents">
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License">
</p>

---

## What is t27?

t27 is a **spec-first** language for ternary computing. You write `.t27` specifications — the compiler generates Zig, Verilog, and C backends. No hand-editing generated code. Ever.

The language is built around three pillars:
- **27 Coptic registers** — a ternary ISA with trits `{-1, 0, +1}`
- **GoldenFloat family** — φ-structured floating-point formats (GF4–GF32) where `exp/mant ≈ 1/φ`
- **Sacred physics** — fundamental constants derived from `φ² + 1/φ² = 3`

t27 is the core of [Trinity S³AI](https://github.com/gHashTag/trinity) — a neuroanatomical AI framework targeting FPGA acceleration and DARPA CLARA compliance.

## Quick Start

```bash
# Clone
git clone https://github.com/gHashTag/t27.git
cd t27

# Build the bootstrap compiler (Rust)
cd bootstrap && cargo build --release
cd ..

# Parse a spec
./bootstrap/target/release/t27c parse specs/base/types.t27

# Generate Zig backend
./bootstrap/target/release/t27c gen-zig specs/numeric/gf16.t27

# Generate Verilog
./bootstrap/target/release/t27c gen-verilog specs/fpga/mac.t27

# Generate C
./bootstrap/target/release/t27c gen-c specs/base/ops.t27

# Verify a seal
./bootstrap/target/release/t27c seal --verify specs/numeric/gf16.t27
```

## Architecture

```
t27/
├── specs/                  # .t27 SPECIFICATIONS — source of truth
│   ├── base/               #   types, ops (2 specs)
│   ├── numeric/            #   GoldenFloat GF4-GF32, TF3, phi_ratio (10 specs)
│   ├── math/               #   sacred_physics, constants (2 specs)
│   ├── ar/                 #   CLARA AR pipeline — logic, proof, datalog (7 specs)
│   ├── nn/                 #   HSLM, attention kernels (2 specs)
│   ├── isa/                #   27 Coptic registers (1 spec)
│   ├── fpga/               #   MAC unit for XC7A100T (1 spec)
│   ├── vsa/                #   Vector Symbolic Architecture (1 spec)
│   ├── queen/              #   Lotus orchestration (1 spec)
│   └── compiler/           #   Parser self-spec (1 spec)
│
├── compiler/               # Compiler .t27 specs (15 specs)
│   ├── parser/             #   lexer.t27, parser.t27
│   ├── codegen/            #   zig/, verilog/, c/, testgen
│   ├── cli/                #   gen, git, spec commands
│   ├── runtime/            #   commands, validation
│   └── skill/              #   PHI LOOP skill registry
│
├── bootstrap/              # Stage-0 compiler (Rust) — FROZEN
│   └── src/compiler.rs     #   SHA-256 sealed in stage0/FROZEN_HASH
│
├── conformance/            # Language-agnostic test vectors (11 JSON)
│   ├── gf*_vectors.json    #   GoldenFloat arithmetic vectors
│   └── sacred_physics*.json#   φ, γ, G, Ω_Λ conformance
│
├── architecture/           # Dependency graph + ADRs
│   ├── graph.tri           #   Canonical dependency DAG
│   ├── graph_v2.json       #   Machine-readable graph (20 nodes)
│   └── ADR-*.md            #   Architecture Decision Records
│
├── .trinity/               # Agent state (Akashic Chronicle)
│   ├── events/             #   Append-only event journal
│   ├── experience/         #   PHI LOOP episodes
│   ├── claims/             #   Agent ownership claims
│   ├── queue/              #   Task queue
│   └── policy/             #   Coordination law
│
├── docs/                   # 15 governance documents
│   ├── SOUL.md             #   Constitutional law
│   ├── SEED-RINGS.md       #   Compiler bootstrap rings
│   ├── NUMERIC-STANDARD-001.md
│   ├── SACRED-PHYSICS-001.md
│   └── AGENTS_ALPHABET.md  #   27-agent system spec
│
└── tests/                  # Ring verification scripts
```

## SEED-RINGS Progress

The compiler grows ring-by-ring. Each ring adds exactly one capability, sealed with 4 SHA-256 hashes.

| Ring | Capability | Layer | Status |
|------|-----------|-------|--------|
| 0 | Frozen stage-0 + first green parse | SEED | ✅ Sealed |
| 1 | Lex all 28 specs without errors | SEED | ✅ Sealed |
| 2 | Type declarations → Zig codegen | SEED | ✅ Sealed |
| 3 | fn signatures → Zig | SEED | ✅ Sealed |
| 4 | module + use → Zig imports | SEED | ✅ Sealed |
| 5 | fn body expressions → Zig | ROOT | ✅ Sealed |
| 6 | test blocks → Zig test blocks | ROOT | ✅ Sealed |
| 7 | invariant + bench → Zig | ROOT | ✅ Sealed |
| 8 | Conformance vectors → test_vector_hash | ROOT | ✅ Sealed |
| 9 | Full Zig backend | TRUNK | ✅ Sealed |
| 10 | Verilog backend | TRUNK | ✅ Sealed |
| 11 | C backend | TRUNK | ✅ Sealed |
| 12 | seal --save / --verify | TRUNK | ✅ Sealed |
| 13 | AR pipeline — all 7 specs | BRANCH | ✅ Sealed |
| 14 | Queen + NN specs gen and seal | BRANCH | ✅ Sealed |
| 15 | Full test suite — all 43 specs | BRANCH | 🔄 Open |
| 16 | Self-hosting: stage(N) == stage(N-1) | CANOPY | ⬜ Planned |
| 17 | Self-hosting verified | CANOPY | ⬜ Planned |

## GoldenFloat Family

φ-structured floating-point formats where `exp/mant ≈ 1/φ`:

| Format | Bits | Exp | Mant | φ-distance | Use Case |
|--------|------|-----|------|------------|----------|
| GF4 | 4 | 1 | 2 | 0.118 | Binary masks |
| GF8 | 8 | 3 | 4 | 0.132 | Weights |
| GF12 | 12 | 4 | 7 | 0.047 | Attention |
| **GF16** | **16** | **6** | **9** | **0.049** | **Primary** |
| GF20 | 20 | 7 | 12 | 0.035 | Training |
| GF24 | 24 | 9 | 14 | 0.025 | Precision |
| GF32 | 32 | 12 | 19 | 0.014 | Full precision |

## Sacred Constants

```t27
pub const PHI: GF16         = 1.618033988749895;   // Golden ratio
pub const PHI_INV: GF16     = 0.618033988749895;   // φ⁻¹
pub const TRINITY: GF16     = 3.0;                  // φ² + φ⁻² = 3
pub const GAMMA_LQG: GF16   = 0.2360679775;         // φ⁻³ (Barbero-Immirzi)
pub const G_MEASURED: GF32   = 6.67430e-11;          // Gravitational constant
pub const OMEGA_LAMBDA: GF32 = 0.685;                // Dark energy density
```

## 27-Agent System

Trinity runs 27 autonomous agents — one per Coptic register:

| Agent | Domain | Key Files |
|-------|--------|-----------|
| **T** (Queen) | Orchestration, 6-phase Lotus cycle | `specs/queen/lotus.t27` |
| **A** | Architecture, SOUL.md, ADRs | `architecture/` |
| **B** | Build, CI/CD, Railway | `bootstrap/` |
| **C** | Compiler core, parser, AST | `compiler/parser/` |
| **D** | De-Zigfication migration | `specs/` → generated backends |
| **F** | Formal conformance vectors | `conformance/*.json` |
| **G** | Graph topology, ARCH_BENCH | `architecture/graph.tri` |
| **H** | HSLM neural architecture | `specs/nn/` |
| **I** | ISA, 27 Coptic registers | `specs/isa/registers.t27` |
| **K** | FPGA/MAC kernel | `specs/fpga/mac.t27` |
| **N** | GoldenFloat numeric | `specs/numeric/` |
| **P** | Sacred physics constants | `specs/math/` |
| **V** | Verdict, toxicity scoring | `conformance/`, `.trinity/verdict/` |
| **27th** (Ϯ) | Security, AAIF compliance | `.trinity/policy/` |

Full list: [docs/AGENTS_ALPHABET.md](docs/AGENTS_ALPHABET.md)

## Constitutional Laws

8 immutable laws govern all mutations. Violations produce **TOXIC** verdicts.

| LAW | Name | Rule |
|-----|------|------|
| 1 | **De-Zigfication** | `.t27` specs are the only source of truth. Zig/C/Verilog = generated output. |
| 2 | **PHI LOOP** | Every mutation follows a 9-step workflow with 4 SHA-256 hashes. |
| 3 | **SEED-RINGS** | Language grows ring-by-ring. One ring = one capability. |
| 4 | **ISSUE-GATE** | No byte enters `master` without an Issue, a PR, and `Closes #N`. |
| 5 | **SOUL.md** | Every `.t27` spec must contain `test {}`, `invariant {}`, or `bench {}`. |
| 6 | **NUMERIC-STANDARD** | GoldenFloat defined in specs + conformance JSON. Never in backend code. |
| 7 | **SACRED-PHYSICS** | Sacred constants live in `specs/math/` with hard tolerances. |
| 8 | **GRAPH TOPOLOGY** | Evolution follows `architecture/graph.tri`. No circular deps. |

Details: [SOUL.md](SOUL.md) · [SEED-RINGS](docs/SEED-RINGS.md) · [NUMERIC-STANDARD-001](docs/NUMERIC-STANDARD-001.md) · [SACRED-PHYSICS-001](docs/SACRED-PHYSICS-001.md)

## PHI LOOP Workflow

Every change follows this exact 9-step cycle:

```
tri skill begin <task> --issue <N>    ← bind to GitHub Issue
tri spec edit <module>                ← edit ONE .t27 spec
tri skill seal --hash                 ← record 4 SHA-256 hashes
tri gen                               ← generate Zig/Verilog/C
tri test                              ← run tests
tri verdict --toxic                   ← TOXIC? → rollback. CLEAN? → proceed
tri experience save                   ← append episode to Akashic journal
tri skill commit                      ← verify hashes + issue binding
tri git commit                        ← push with "Closes #N"
```

## Contributing

1. Open a [GitHub Issue](https://github.com/gHashTag/t27/issues) first — **no issue = no work** (LAW 4)
2. Create a branch: `ring/<N>-<name>`, `ar/<AR-NNN>-<name>`, `fix/<name>`, or `task/<name>`
3. Edit `.t27` specs only — never hand-edit generated Zig/Verilog/C (LAW 1)
4. Every spec must have `test {}`, `invariant {}`, or `bench {}` blocks (LAW 5)
5. Commit message: `feat(ring-N): description [SEED-N]` with `Closes #N`
6. Open a PR targeting `master`

## Documentation

### Governance
- [SOUL.md](SOUL.md) — Constitutional law
- [SEED-RINGS](docs/SEED-RINGS.md) — Incremental compiler bootstrap
- [NUMERIC-STANDARD-001](docs/NUMERIC-STANDARD-001.md) — GoldenFloat specification
- [SACRED-PHYSICS-001](docs/SACRED-PHYSICS-001.md) — Sacred physics constants
- [PHI LOOP Contract](docs/PHI_LOOP_CONTRACT.md) — Workflow contract
- [TDD Contract](docs/TDD-CONTRACT.md) — Test-driven development policy

### Architecture
- [ADR-001: De-Zigfication](architecture/ADR-001-de-zigfication.md)
- [ADR-003: TDD Inside Spec](architecture/ADR-003-tdd-inside-spec.md)
- [ADR-004: Language Policy](architecture/ADR-004-language-policy.md)
- [ADR-005: De-Zig Strict](architecture/ADR-005-de-zig-strict.md)
- [CANON DE-ZIGFICATION](architecture/CANON_DE_ZIGFICATION.md)

### Agents & Operations
- [27-Agent Alphabet](docs/AGENTS_ALPHABET.md) — All 27 agents
- [CLARA Preparation Plan](docs/CLARA-PREPARATION-PLAN.md) — DARPA compliance
- [Kleene Trit Isomorphism](docs/KLEENE-TRIT-ISOMORPHISM.md)
- [TRI Syntax vNext](docs/TRI_SYNTAX_VNEXT.md)

## License

MIT

---

<p align="center">
  <strong>φ² + 1/φ² = 3 | TRINITY</strong><br>
  <sub>Maintained by <a href="https://github.com/gHashTag">Dmitrii [NeuroCoder]</a> · 27 agents, 45 specs, 15 sealed rings</sub>
</p>
