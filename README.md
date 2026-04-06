# Trinity S³AI DNA -- t27 -- TRI-27 Spec-First Language

The canonical source of truth for Trinity S3AI.  
`.t27` specs in -> Zig, Verilog, C out.

**φ² + 1/φ² = 3 | TRINITY**

---

## What is t27?

t27 is a **spec-first** language for ternary computing. You write `.t27` specifications -- the compiler generates Zig, Verilog, and C backends. No hand-editing generated code. Ever.

The language is built around three pillars:

- **27 Coptic registers** -- a ternary ISA with trits `{-1, 0, +1}`
- **GoldenFloat family** -- phi-structured floating-point formats (GF4-GF32) where `exp/mant ~ 1/phi`
- **Sacred physics** -- fundamental constants derived from `phi^2 + 1/phi^2 = 3`

t27 is the core of [Trinity S3AI](https://github.com/gHashTag/trinity) -- a neuroanatomical AI framework targeting FPGA acceleration and DARPA CLARA compliance.

## Quick Start

```bash
# Clone
git clone https://github.com/gHashTag/t27.git
cd t27

# Build the bootstrap compiler (Rust); use ./scripts/tri as the CLI entry (wraps t27c)
cd bootstrap && cargo build --release
cd ..

# Parse a spec (canonical CLI: tri → wraps bootstrap t27c)
./scripts/tri parse specs/base/types.t27

# Generate Zig backend (stdout, or pass a directory to write gen/zig/…)
./scripts/tri gen-zig specs/numeric/gf16.t27

# Generate Verilog
./scripts/tri gen-verilog specs/fpga/mac.t27

# Generate C
./scripts/tri gen-c specs/base/ops.t27

# Verify a seal
./scripts/tri seal specs/numeric/gf16.t27 --verify

# Run all tests (Rust suite: parse / gen / seal / fixed-point)
./scripts/tri test

# Validate conformance vectors (JSON under conformance/)
./scripts/tri validate-conformance

# Validate generated file headers under gen/
./scripts/tri validate-gen-headers

# NOW.md must reflect today’s calendar date (also enforced before gen/compile via tri)
./scripts/tri check-now
```

## Architecture

The project is organized into 5 strands that evolved ring-by-ring:

```
STRAND I   - Base         : types, ops, constants          (Rings 0-8)
STRAND II  - Numeric+VSA  : GF4-GF32, TF3, phi, VSA ops   (Rings 9-11)
STRAND III - Compiler+FPGA: parser, MAC, ISA registers      (Rings 12-14)
STRAND IV  - Queen+NN     : Lotus orchestration, HSLM, attention (Rings 14-17)
STRAND V   - AR (CLARA)   : ternary logic, proof traces, Datalog, restraint, XAI, ASP, composition (Rings 18-24)
```

Gen backends (Zig, C, Verilog) and conformance vectors were generated across Rings 25-31.

### Agent experience (design)

Multi-agent memory, Queen wisdom, and planned **`tri`** subcommands for experience / insights are outlined in **[`docs/TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md`](docs/TRINITY-EXPERIENCE-EXCHANGE-ARCHITECTURE.md)**. **Today’s supported pipeline** is the Quick Start block above (`tri test`, `tri check-now`, validators, codegen).

### Directory Structure

```
t27/
├── specs/                  # .t27 SPECIFICATIONS -- source of truth
│   ├── base/               #   types, ops (2 specs)
│   ├── numeric/            #   GoldenFloat GF4-GF32, TF3, phi_ratio (10 specs)
│   ├── math/               #   sacred_physics, constants (2 specs)
│   ├── ar/                 #   CLARA AR pipeline -- logic, proof, datalog (7 specs)
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
├── gen/                    # GENERATED backends -- DO NOT EDIT
│   ├── zig/                #   Zig backend (28 modules)
│   ├── c/                  #   C backend (28 .c + 28 .h)
│   └── verilog/            #   Verilog backend (28 modules)
│
├── conformance/            # Language-agnostic test vectors (34 JSON)
│   ├── gf*_vectors.json    #   GoldenFloat arithmetic vectors
│   ├── ar_*.json           #   CLARA AR conformance vectors
│   ├── nn_*.json           #   Neural architecture vectors
│   └── sacred_physics*.json#   phi, gamma, G, Omega_Lambda conformance
│
├── bootstrap/              # Stage-0 compiler (Rust) -- FROZEN
│   └── src/compiler.rs     #   SHA-256 sealed in bootstrap/stage0/FROZEN_HASH
│
├── architecture/           # Dependency graph + ADRs
│   ├── graph.tri           #   Canonical dependency DAG
│   ├── graph_v2.json       #   Machine-readable graph (20 nodes)
│   └── ADR-*.md            #   Architecture Decision Records
│
├── .trinity/               # Agent state (Akashic Chronicle)
│   ├── events/             #   Append-only event journal
│   ├── experience/         #   PHI LOOP episodes (38 episodes)
│   ├── seals/              #   48 SHA-256 integrity seals
│   ├── state/              #   queen-health.json, graph sync
│   ├── claims/             #   Agent ownership claims
│   ├── queue/              #   Task queue
│   └── policy/             #   Coordination law
│
├── contrib/                # Non-core adjacency (API, runners, portable setup) — see OWNERS.md
├── external/               # Vendored upstream (e.g. OpenCode submodule) + kaggle tree — see OWNERS.md
│
├── docs/                   # First-party docs (27-agent / 3-nona layout — see docs/README.md)
│   ├── README.md           #   Index: agents/, coordination/, nona-01..03/, clara/
│   ├── NOW.md              #   Rolling snapshot (sync gates)
│   ├── T27-CONSTITUTION.md #   Charter
│   └── …                   #   nona-01-foundation/, nona-02-organism/, nona-03-manifest/, etc.
│
└── tests/                  # Ring verification + validation scripts
    ├── comprehensive_suite.t27 # Suite contract (see t27c suite)
    └── *.t27             #   Spec tests only — no shell runners
```

**Domain ownership:** each major directory may include an `**OWNERS.md`** (Primary agent, dependencies, outputs). Start at `[OWNERS.md](OWNERS.md)` in the repo root; see also `[docs/agents/AGENTS_ALPHABET.md](docs/agents/AGENTS_ALPHABET.md)`.

## CLARA Automated Reasoning

The AR domain (Rings 18-24) implements a full DARPA CLARA-compliant reasoning pipeline in ternary logic:


| Module             | Spec                          | Description                                                                                                 |
| ------------------ | ----------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Ternary Logic**  | `specs/ar/ternary_logic.t27`  | Kleene K3 logic: `{T, U, F}` isomorphic to trits `{+1, 0, -1}`. 27 truth table entries, verified K3 axioms. |
| **Proof Traces**   | `specs/ar/proof_trace.t27`    | Bounded proof traces with a hard 10-step limit. Each step carries a GF16 confidence score.                  |
| **Datalog Engine** | `specs/ar/datalog_engine.t27` | Forward-chaining Datalog with O(n) complexity. Stratified negation via K3 unknown.                          |
| **Restraint**      | `specs/ar/restraint.t27`      | Bounded rationality: resource limits on inference (max steps, max memory, timeout).                         |
| **Explainability** | `specs/ar/explainability.t27` | CLARA-compliant XAI: explanations <= 10 steps, each with GF16 confidence.                                   |
| **ASP Solver**     | `specs/ar/asp_solver.t27`     | Answer Set Programming with Negation-as-Failure under K3 semantics.                                         |
| **Composition**    | `specs/ar/composition.t27`    | ML+AR composition patterns: CNN+Rules, MLP+Bayesian, Transformer+XAI, RL+Guardrails.                        |


All 7 AR modules have gen backends (Zig, C, Verilog) and conformance vectors.

## Conformance Testing

Every domain has language-agnostic conformance vectors in `conformance/*.json`. These JSON files contain test inputs, expected outputs, and tolerances that any backend must satisfy.

**34 conformance vectors** cover:

- GoldenFloat arithmetic (GF4 through GF32)
- Sacred physics constants (phi, gamma, G, Omega_Lambda)
- Base types and operations
- CLARA AR pipeline (all 7 modules)
- Neural architecture (attention, HSLM)
- Domain modules (VSA ops, ISA registers, FPGA MAC, Queen Lotus)

Validation: `./scripts/tri validate-conformance`

## SEED-RINGS Progress

The compiler grows ring-by-ring. Each ring adds exactly one capability, sealed with SHA-256 hashes.


| Ring | Capability                                         | Layer  | Status      |
| ---- | -------------------------------------------------- | ------ | ----------- |
| 0    | Frozen stage-0 + first green parse                 | SEED   | Sealed      |
| 1    | Lex all 28 specs without errors                    | SEED   | Sealed      |
| 2    | Type declarations -> Zig codegen                   | SEED   | Sealed      |
| 3    | fn signatures -> Zig                               | SEED   | Sealed      |
| 4    | module + use -> Zig imports                        | SEED   | Sealed      |
| 5    | fn body expressions -> Zig                         | ROOT   | Sealed      |
| 6    | test blocks -> Zig test blocks                     | ROOT   | Sealed      |
| 7    | invariant + bench -> Zig                           | ROOT   | Sealed      |
| 8    | Conformance vectors -> test_vector_hash            | ROOT   | Sealed      |
| 9    | Full Zig backend                                   | TRUNK  | Sealed      |
| 10   | Verilog backend                                    | TRUNK  | Sealed      |
| 11   | C backend                                          | TRUNK  | Sealed      |
| 12   | seal --save / --verify                             | TRUNK  | Sealed      |
| 13   | AR pipeline -- all 7 specs                         | BRANCH | Sealed      |
| 14   | Queen + NN specs gen and seal                      | BRANCH | Sealed      |
| 15   | Full test suite -- all 43 specs                    | BRANCH | Sealed      |
| 16   | Self-hosting: stage(N) == stage(N-1)               | CANOPY | Sealed      |
| 17   | Self-hosting verified (fixed point)                | CANOPY | Sealed      |
| 18   | AR ternary logic (K3 isomorphism)                  | AR     | Sealed      |
| 19   | Bounded proof traces                               | AR     | Sealed      |
| 20   | Datalog engine (forward chaining)                  | AR     | Sealed      |
| 21   | Restraint (bounded rationality)                    | AR     | Sealed      |
| 22   | Explainability (CLARA XAI)                         | AR     | Sealed      |
| 23   | ASP solver (NAF + K3)                              | AR     | Sealed      |
| 24   | ML+AR composition (4 patterns)                     | AR     | Sealed      |
| 25   | Gen backends: base/types, base/ops, math/constants | GEN    | Sealed      |
| 26   | Gen backends: numeric core (GF4-GF16, TF3, phi)    | GEN    | Sealed      |
| 27   | Gen backends: extended numerics (GF20-GF32)        | GEN    | Sealed      |
| 28   | Gen backends: VSA, ISA, FPGA, sacred physics       | GEN    | Sealed      |
| 29   | Gen backends: NN attention, HSLM, Queen Lotus      | GEN    | Sealed      |
| 30   | Conformance vectors: AR gap coverage               | GEN    | Sealed      |
| 31   | Compiler/parser gen + graph sync + queen health    | GEN    | Sealed      |
| 32+  | Hardening: docs, validation, CI                    | HARDEN | In Progress |


## GoldenFloat Family

phi-structured floating-point formats where `exp/mant ~ 1/phi`:


| Format   | Bits   | Exp   | Mant  | phi-distance | Use Case       |
| -------- | ------ | ----- | ----- | ------------ | -------------- |
| GF4      | 4      | 1     | 2     | 0.118        | Binary masks   |
| GF8      | 8      | 3     | 4     | 0.132        | Weights        |
| GF12     | 12     | 4     | 7     | 0.047        | Attention      |
| **GF16** | **16** | **6** | **9** | **0.049**    | **Primary**    |
| GF20     | 20     | 7     | 12    | 0.035        | Training       |
| GF24     | 24     | 9     | 14    | 0.025        | Precision      |
| GF32     | 32     | 12    | 19    | 0.014        | Full precision |


## Sacred Constants

```t27
pub const PHI: GF16         = 1.618033988749895;   // Golden ratio
pub const PHI_INV: GF16     = 0.618033988749895;   // phi^-1
pub const TRINITY: GF16     = 3.0;                  // phi^2 + phi^-2 = 3
pub const GAMMA_LQG: GF16   = 0.2360679775;         // phi^-3 (Barbero-Immirzi)
pub const G_MEASURED: GF32   = 6.67430e-11;          // Gravitational constant
pub const OMEGA_LAMBDA: GF32 = 0.685;                // Dark energy density
```

## 27-Agent System

Trinity runs 27 autonomous agents -- one per Coptic register:


| Agent         | Domain                             | Key Files                           |
| ------------- | ---------------------------------- | ----------------------------------- |
| **T** (Queen) | Orchestration, 6-phase Lotus cycle | `specs/queen/lotus.t27`             |
| **A**         | Architecture, SOUL.md, ADRs        | `architecture/`                     |
| **B**         | Build, CI/CD, Railway              | `bootstrap/`                        |
| **C**         | Compiler core, parser, AST         | `compiler/parser/`                  |
| **D**         | De-Zigfication migration           | `specs/` -> generated backends      |
| **F**         | Formal conformance vectors         | `conformance/*.json`                |
| **G**         | Graph topology, ARCH_BENCH         | `architecture/graph.tri`            |
| **H**         | HSLM neural architecture           | `specs/nn/`                         |
| **I**         | ISA, 27 Coptic registers           | `specs/isa/registers.t27`           |
| **K**         | FPGA/MAC kernel                    | `specs/fpga/mac.t27`                |
| **N**         | GoldenFloat numeric                | `specs/numeric/`                    |
| **P**         | Sacred physics constants           | `specs/math/`                       |
| **V**         | Verdict, toxicity scoring          | `conformance/`, `.trinity/verdict/` |
| **27th**      | Security, AAIF compliance          | `.trinity/policy/`                  |


Full list: [docs/agents/AGENTS_ALPHABET.md](docs/agents/AGENTS_ALPHABET.md)

## Constitutional Laws

8 immutable laws govern all mutations. Violations produce **TOXIC** verdicts.


| LAW | Name                 | Rule                                                                         |
| --- | -------------------- | ---------------------------------------------------------------------------- |
| 1   | **De-Zigfication**   | `.t27` specs are the only source of truth. Zig/C/Verilog = generated output. |
| 2   | **PHI LOOP**         | Every mutation follows a 9-step workflow with 4 SHA-256 hashes.              |
| 3   | **SEED-RINGS**       | Language grows ring-by-ring. One ring = one capability.                      |
| 4   | **ISSUE-GATE**       | No byte enters `master` without an Issue, a PR, and `Closes #N`.             |
| 5   | **SOUL.md**          | Every `.t27` spec must contain `test {}`, `invariant {}`, or `bench {}`.     |
| 6   | **NUMERIC-STANDARD** | GoldenFloat defined in specs + conformance JSON. Never in backend code.      |
| 7   | **SACRED-PHYSICS**   | Sacred constants live in `specs/math/` with hard tolerances.                 |
| 8   | **GRAPH TOPOLOGY**   | Evolution follows `architecture/graph.tri`. No circular deps.                |


Details: [SOUL.md](SOUL.md) | [SEED-RINGS](docs/nona-01-foundation/SEED-RINGS.md) | [NUMERIC-STANDARD-001](docs/nona-02-organism/NUMERIC-STANDARD-001.md) | [SACRED-PHYSICS-001](docs/nona-02-organism/SACRED-PHYSICS-001.md)

## PHI LOOP Workflow

Every change follows this exact 9-step cycle:

```
tri skill begin <task> --issue <N>    <- bind to GitHub Issue
tri spec edit <module>                <- edit ONE .t27 spec
tri skill seal --hash                 <- record 4 SHA-256 hashes
tri gen                               <- generate Zig/Verilog/C
tri test                              <- run tests
tri verdict --toxic                   <- TOXIC? -> rollback. CLEAN? -> proceed
tri experience save                   <- append episode to Akashic journal
tri skill commit                      <- verify hashes + issue binding
tri git commit                        <- push with "Closes #N"
```

## Contributing

1. Open a [GitHub Issue](https://github.com/gHashTag/t27/issues) first -- **no issue = no work** (LAW 4)
2. Create a branch: `ring/<N>-<name>`, `ar/<AR-NNN>-<name>`, `fix/<name>`, or `task/<name>`
3. Edit `.t27` specs only -- never hand-edit generated Zig/Verilog/C (LAW 1)
4. Every spec must have `test {}`, `invariant {}`, or `bench {}` blocks (LAW 5)
5. Commit message: `feat(ring-N): description [SEED-N]` with `Closes #N`
6. Open a PR targeting `master`

## PHI LOOP Status

- **31 rings sealed** (SEED-0 through SEED-17, AR 18-24, GEN 25-31)
- **45 .t27 spec files** (28 specs/ + 15 compiler/ + 2 sandbox)
- **112 generated files** across 3 backends (Zig, C, Verilog)
- **34 conformance vectors** covering all domains
- **48 integrity seals** in .trinity/seals/
- **6 CLI commands**: parse, gen, gen-zig, gen-verilog, gen-c, seal
- **5 architecture strands**: Base -> Numeric -> Compiler+FPGA -> Queen+NN -> AR
- **Deterministic fixed point** reached at Ring 17 (CANOPY)
- **CLARA AR module**: 7 specs (ternary logic -> composition)
- **Queen health**: GREEN 1.0 across 15 domains
- CI enforced: Issue Gate + PHI Loop CI on all PRs

## CI Enforcement

All PRs to `master` must:

1. Link to an issue via `Closes #N`
2. Pass PHI Loop CI (build, parse, gen, seal verify)
3. Pass conformance validation
4. Pass gen header validation
5. Pass seal coverage check

See [ISSUE-GATE-001](docs/nona-03-manifest/ISSUE-GATE-001.md) for details.

## Documentation

**Full map (27 agents / three nonas):** [docs/README.md](docs/README.md)

### Governance

- [SOUL.md](SOUL.md) -- Constitutional law
- [SEED-RINGS](docs/nona-01-foundation/SEED-RINGS.md) -- Incremental compiler bootstrap
- [NUMERIC-STANDARD-001](docs/nona-02-organism/NUMERIC-STANDARD-001.md) -- GoldenFloat specification
- [SACRED-PHYSICS-001](docs/nona-02-organism/SACRED-PHYSICS-001.md) -- Sacred physics constants
- [PHI LOOP Contract](docs/nona-03-manifest/PHI_LOOP_CONTRACT.md) -- Workflow contract
- [TDD Contract](docs/nona-03-manifest/TDD-CONTRACT.md) -- Test-driven development policy

### Architecture

- [ADR-001: De-Zigfication](architecture/ADR-001-de-zigfication.md)
- [ADR-003: TDD Inside Spec](architecture/ADR-003-tdd-inside-spec.md)
- [ADR-004: Language Policy](architecture/ADR-004-language-policy.md)
- [ADR-005: De-Zig Strict](architecture/ADR-005-de-zig-strict.md)
- [CANON DE-ZIGFICATION](architecture/CANON_DE_ZIGFICATION.md)
- [TECHNOLOGY-TREE](docs/nona-03-manifest/TECHNOLOGY-TREE.md) -- Evolution roadmap

### Agents & Operations

- [27-Agent Alphabet](docs/agents/AGENTS_ALPHABET.md) -- All 27 agents
- [CLARA Preparation Plan](docs/clara/CLARA-PREPARATION-PLAN.md) -- DARPA compliance
- [Kleene Trit Isomorphism](docs/nona-02-organism/KLEENE-TRIT-ISOMORPHISM.md)
- [TRI Syntax vNext](docs/nona-02-organism/TRI_SYNTAX_VNEXT.md)
- [ISSUE-GATE-001](docs/nona-03-manifest/ISSUE-GATE-001.md) -- Issue gate enforcement law

## License

MIT

---

**phi^2 + 1/phi^2 = 3 | TRINITY**  
Maintained by [Dmitrii [Vasilev]](https://github.com/gHashTag) -- 27 agents, 45 specs, 31 sealed rings

**Maintained by**: Trinity Project
**Status:** Ring 31 Complete (2026-04-04) -- 31 rings sealed, 45 specs, 112 gen files, 34 conformance vectors, 48 seals, CI enforced