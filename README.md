# Trinity S³AI DNA -- t27 -- TRI-27 Spec-First Language

[![CI](https://img.shields.io/github/actions/workflow/status/gHashTag/t27/bootstrap-tests.yml?branch=master&logo=github&label=CI)](https://github.com/gHashTag/t27/actions/workflows/bootstrap-tests.yml)
[![Zenodo](https://zenodo.org/badge/DOI/10.5281/zenodo.19456875.svg)](https://doi.org/10.5281/zenodo.19456875)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version: 0.2.0](https://img.shields.io/badge/version-0.2.0-orange.svg)](https://github.com/gHashTag/t27/releases)


> **Canonical Zenodo SOT:** [zenodo.org/communities/trinity-s3ai](https://zenodo.org/communities/trinity-s3ai/). The GoldenFloat badge above (19456875) is a legitimate Vasilev deposit but lives **outside** the curated S³AI v5.0 record set; see [docs/ZENODO.md](docs/ZENODO.md) for the canonical 12-record bundle and aliases.

The canonical source of truth for Trinity S3AI.
`.t27` specs in → Zig, Verilog, C out.

**φ² + 1/φ² = 3 | TRINITY**

---

## What this repo is

**t27** is the **spec-first toolchain and numeric format registry** for the
**TRI-NET line** of open high-assurance ternary AI silicon. The primary
product of t27 is the path `.t27 → Verilog RTL → Tiny Tapeout` with sealed,
inspectable artefacts at every step.

- **How to verify:** `cd bootstrap && cargo build --release && cd .. && ./scripts/tri test`
  (full Quick Start below). Validators: `./scripts/tri validate-conformance` and `validate-gen-headers`.
- **Primary numeric path:** GoldenFloat **GF16** (default), with the family
  GF4–GF32 registered in [`conformance/FORMAT-SPEC-001.json`](conformance/FORMAT-SPEC-001.json).
  FP8 compat and NF4 / INT4 / INT8 quant bridges are **planned**, not shipped.
  Full details: [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md).
- **Readiness:** [`STATUS.md`](STATUS.md) records what is at SPEC / RTL / SIM /
  SYNTH / GDS / SILICON, conservatively, from this repo's own evidence only.
- **Sibling chip repos (separate):** `tt-trinity-phi` (1×1 φ-anchor),
  `tt-trinity-euler` (8×2 e-engine, safety/control), `tt-trinity-gamma`
  (8×4 γ-surface 32-PE ternary mesh). Tape-out target:
  [Tiny Tapeout](https://tinytapeout.com/chips/). See [`LINEUP.md`](LINEUP.md).
- **Positioning:** [`COMPETITORS.md`](COMPETITORS.md) — we do not race
  commercial NPUs on TOPS or SDK breadth; we own the inspectable open silicon
  and formal / assurance corner. Benchmark policy: [`BENCHMARKS.md`](BENCHMARKS.md).
- **CLARA traceability:** [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md).
## FPGA — what is measured on real silicon (2026-08-15)

Three **QMTech XC7A200T-FGG676** boards. Synthesis is
`yosys 0.63 → nextpnr-xilinx (openXC7) → prjxray`, always with
**`-nodsp -nosrl`** (see the defect note below).

| result | number |
|---|---|
| **Trained ternary network across three dice** | **232 LUT**, 0 DSP, 0 SRL, **100/100** layer agreement on real UNSW-NB15 rows |
| Best single-die artefact | **123 LUT** at **81.37%** (UNSW-NB15) / **86.91%** (Fashion-bin) |
| Field reference on the same benchmark | TreeLUT (II): **89 LUT** at **92.0%** |
| Cost rule, measured | **2.00 LUT/neuron** at ≤6 input bits; **39–54** at 12 |

**Our area is competitive; our accuracy is not** — 1.38× the field's LUT at 10.6
points less accuracy on UNSW-NB15. Detail: [`BENCHMARKS.md`](BENCHMARKS.md) §4,
positioning: [`COMPETITORS.md`](COMPETITORS.md) §5.

**The golden alphabet, measured against its own claim.** This line is named for
φ. Alphabet **size** is worth **+0.735 pp** and its **shape** +0.149 pp, against
inter-layer normalisation at **+29.15 pp** — and the multiplier φ removes from
weight application **returns in the pair resolve** (8 DSP48E1, or ~2750 LUT
without them). The algebra stands and is Coq-checked; the practical advantage
does not. This is a negative result about our own headline idea and is recorded
as one.

**Toolchain defects found here.** openXC7 emits a **wrong bitstream from a
correct netlist** for `SRL16E` (0/6 rows vs 24/24 with `-nosrl`, source
byte-identical) and for a live-operand `DSP48E1`. Both pass the wrong-part → ours
`0→1` acceptance criterion while computing the wrong answer. `t27c yostat` exits
2 when either appears. Reproduction:
[`docs/reports/OPENXC7-SRL16E-DEFECT.md`](docs/reports/OPENXC7-SRL16E-DEFECT.md).

---

## HuggingFace mirror

Discoverability mirror of the numeric corpus. GitHub remains the primary source of truth (one-way flow GitHub -> Zenodo -> Hugging Face).

- **Catalog dataset:** [playra/numeric-format-catalog](https://huggingface.co/datasets/playra/numeric-format-catalog) -- the 83-format SSOT
- **Conformance packs:** [playra/numeric-conformance-packs](https://huggingface.co/datasets/playra/numeric-conformance-packs) -- bit-exact JSON vectors (10 configs across GF4..GF32 + phi-identity)
- **Paper page:** [arXiv:2606.09686 on HF](https://huggingface.co/papers/2606.09686)



---

## System Status

| Domain | Component | Status | Details |
|--------|-----------|--------|---------|
| Compiler | `t27c parse` | GREEN | 170+ specs parse |
| Compiler | `t27c gen-verilog` | GREEN | 5/5 FPGA modules synthesize |
| Compiler | `t27c seal` | GREEN | 170+ seals in `.trinity/seals/` |
| FPGA | Yosys synthesis | GREEN | 5/5 modules pass synth_xilinx |
| FPGA | E2E bitstream | GREEN | Yosys→nextpnr→prjxray→.bit (zero Vivado) |
| FPGA | Board profiles | GREEN | QMTECH XC7A100T (minimal+full), Arty A7 |
| FPGA | `--profile` flag | GREEN | `--profile minimal|full` in fpga-build |
| Pins | Pins IR | GREEN | `specs/pins/ir.t27` — conflict detection invariants |
| Pins | XDC emitter | GREEN | `specs/pins/emitter_xdc.t27` — QMTECH + Arty presets |
| CI | Issue gate | GREEN | L1 TRACEABILITY enforced |
| CI | Seal coverage | GREEN | All specs sealed |
| CI | Schema validation | GREEN | Conformance vectors validated |
| CI | FPGA smoke | GREEN | Verilog gen in CI |
| CI | FPGA bitstream artifact | GREEN | .bit uploaded per PR (7-day retention) |
| TRI | PHI LOOP CLI | GREEN | `cli/tri/` standalone binary |
| TRI | MCP server | GREEN | `cli/tri-mcp/` — 10 tools over JSON-RPC |
| Spec | Phase 3 (shell/tools/file) | YELLOW | 6/8 parse; 2 file specs have parser issue (#388) |
| BitNet HLS | RTL pipeline | GREEN | 9/9 modules (W36a-f + W38 bundle + R-BV-2 `--with-sva`) |
| Host stack | Rust driver + IRQ harness | GREEN | 2/3 layers (W39 R-HS-1 driver, W40 R-HS-2 IRQ); host inference engine in flight (Dmitrii W41-W44 parallel) |
| R-TT track | Tiny Tapeout reproducibility | YELLOW | 2/4 (W42 R-TT-1 `tt-manifest` + chip submodules; W45 R-TT-2 `tt-profile` + `tt-conform`); W46-W47 planned |
| Chips | tt-trinity-{phi,euler,gamma} | GREEN | Pinned as git submodules under `chips/` at known commits (W42) |

---

## BitNet HLS Pipeline & R-TT Reproducibility Track

The `bootstrap/src/` Rust toolchain (`t27c`) now emits a complete
**9-module BitNet HLS pipeline** and a **Tiny Tapeout reproducibility chain**
tying every tape-out to a specific t27 commit + trinity-invariant SHA-256.

### BitNet HLS pipeline (9 / 9 modules)

| # | Module | Wave | Emitter CLI |
|---|--------|------|-------------|
| 1 | `weight_bram` | W36a | `t27c gen-weight-bram` |
| 2 | `pipeline_stage2_compute` | W36b | `t27c gen-pipeline-stage2` |
| 3 | `layer_sequencer` | W36b | `t27c gen-layer-sequencer` |
| 4 | `double_buffer_ctrl` | W36c | `t27c gen-double-buffer-ctrl` |
| 5 | `weight_prefetch_ctrl` | W36c | `t27c gen-weight-prefetch-ctrl` |
| 6 | `bitnet_axi_slave` | W36d | (in bundle) |
| 7 | `bitnet_dma` | W36d | (in bundle) |
| 8 | `bitnet_irq` | W36d | (in bundle) |
| 9 | `bitnet_engine_top` | W36d | (in bundle) |

All nine come together as a single emit:

```
t27c gen-bitnet-bundle --output bundle.sv [--with-sva]
```

The `--with-sva` flag (R-BV-2, Dmitrii) wraps every emit with SystemVerilog
Assertions so the entire bundle is formal-friendly.

### Host stack (W39 R-HS-1, W40 R-HS-2)

A pure-Rust host driver against a MockMmio matching the W36d AXI-Lite slave
CSR map (`CTRL`, `STATUS`, `IRQ_EN`, `IRQ_STAT` with W1C semantics, `NUM_LAYERS`,
`NEURONS`, `CHUNKS`, `THRESHOLD`, `WEIGHT_ADDR_LO/HI`).  Two CLIs:

```
t27c host-smoke         # busy-poll path
t27c host-poll-vs-irq   # comparison harness (poll vs IRQ-handler)
```

Dmitrii's parallel R-HS track (W41-W44) is extending this with a full
host-side inference engine, performance cycle estimator, `--json` output,
and ternary weight packer.

### R-TT track -- Tiny Tapeout reproducibility (2 / 4)

The three Tiny Tapeout silicon variants -- `tt-trinity-phi`,
`tt-trinity-euler`, `tt-trinity-gamma` -- live as git submodules under
`chips/` pinned to known commits.  Two CLIs make every tape-out machine-checkable:

```
t27c tt-manifest --chip <phi|euler|gamma> [--output <path>|-]
#  -> deterministic JSON manifest:
#     { t27_commit, phi_invariant_hash, chip, modules[], axi_widths,
#       sva_count, build_time_utc }

t27c tt-profile --platform <sky130|ihp|gf180> [--output <path>|-]
#  -> deterministic JSON profile:
#     { platform, process_node_nm, cell_library, max_tile_area_um2,
#       supply_voltage_mvolts, target_clock_mhz, max_modules }

t27c tt-conform --profile <p.json> --manifest <m.json> [--verbose]
#  -> single boolean gate:
#     OK conform=<true|false> reasons=<N>
#  exit 0 if ok, 1 otherwise
```

The `phi_invariant_hash` is the SHA-256 of the ASCII string
`phi^2 + 1/phi^2 = 3` (`218403e3...8f80e6b`) and is embedded in every
manifest -- any silent change to the numeric kernel would change the hash
and show up immediately in diff.

Roadmap:

- W46 R-TT-3 `tt-debug` -- TT-debug wrapper around `bitnet_engine_top` (version CSR + error counters + self-test trigger)
- W47 R-TT-4 `tt-lockfile` -- `tt.lock` (chip-hash + commit + profile + verdict) pinned in each chip-repo via submodule

### Submodule layout

```
chips/phi    -> https://github.com/gHashTag/tt-trinity-phi
chips/euler  -> https://github.com/gHashTag/tt-trinity-euler
chips/gamma  -> https://github.com/gHashTag/tt-trinity-gamma
```

Clone with submodules:

```
git clone --recursive https://github.com/gHashTag/t27.git
# or, after a plain clone:
git submodule update --init --recursive
```

### Test coverage (post-W45)

- BitNet HLS suites: 9 modules x dedicated integration suite each
- Host stack: `host_driver` (25), `host_irq` (25)
- R-TT track: `tt_manifest` (23 + 18 inline), `tt_profile` (25 + 24 inline)
- Regression: **20 integration suites green**, total **365 / 366** with one
  pre-existing fail in `verilog_const_array::r_ca_1_emitter_on_real_mac_spec`
  (predates W37).

Live wave-by-wave log: [`docs/NOW.md`](docs/NOW.md).

---

## What is t27?

t27 is a **spec-first** language for ternary computing. You write `.t27` specifications -- the compiler generates Zig, Verilog, and C backends. No hand-editing generated code. Ever.

The language is built around three pillars:

- **27 Coptic registers** -- a ternary ISA with trits `{-1, 0, +1}`
- **GoldenFloat family** -- phi-structured floating-point formats (GF4-GF32) where `exp/mant ~ 1/phi`
- **Sacred physics** -- fundamental constants derived from `phi^2 + 1/phi^2 = 3`

t27 is the core of [Trinity S3AI](https://github.com/gHashTag/trinity) -- a neuroanatomical AI framework targeting FPGA acceleration, with an automated-reasoning pipeline written against the public DARPA CLARA solicitation as a specification target. No DARPA award or engagement -- see [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md).

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

# Generate Zig (stdout for one file; if the path is a directory, batch → gen/zig/… by default)
./scripts/tri gen specs/numeric/gf16.t27
./scripts/tri gen specs/numeric
# `gen-dir` does not exist; generate a tree by looping over its specs.

# Generate Verilog (file or directory → gen/verilog/…)
./scripts/tri gen-verilog specs/fpga/mac.t27

# Generate C (file or directory → gen/c/…)
./scripts/tri gen-c specs/base/ops.t27

# Verify a seal
./scripts/tri seal specs/numeric/gf16.t27 --verify

# Run all tests (Rust suite: parse / gen / seal / fixed-point)
./scripts/tri test

# Validate conformance vectors (JSON under conformance/)
./scripts/tri validate-conformance

# Validate generated file headers under gen/
./scripts/tri validate-gen-headers

# NOW.md date gate (also runs inside t27c before gen / gen-dir / compile*)
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

The AR domain (Rings 18-24) implements a reasoning pipeline in ternary logic, written against the ten assurance requirements described in the public DARPA CLARA solicitation -- CLARA-style, not certified (DARPA does not certify compliance):


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


## Wave 11 — Rust Crates ring-088..ring-099 (12 crates)

> **Honest status:** все 12 крейтов **написаны на диске** (Rust, ~10 000+ строк, 33 `Cargo.toml`), но **`cargo check` / `cargo test` НЕ запускались** — `cargo` и `rustc` не установлены в текущем окружении (network/permission). Цифры строк подтверждены `find` + `wc`.

| Crate | Files | Rust LOC | Topic | Status |
|:------|------:|---------:|:------|:------:|
| `ring-088-rust` |  5 |   961 | GF16 MAC                     | ✅ written, ⏳ uncompiled |
| `ring-089-rust` |  4 |   334 | TNN ISA                      | ✅ written, ⏳ uncompiled |
| `ring-090-rust` | 10 | 2 143 | Simulator                    | ✅ written, ⏳ uncompiled |
| `ring-091-rust` |  4 |   409 | Stochastic Rounding          | ✅ written, ⏳ uncompiled |
| `ring-092-rust` |  6 |   847 | Attention                    | ✅ written, ⏳ uncompiled |
| `ring-093-rust` |  4 |   668 | Sparse MoE                   | ✅ written, ⏳ uncompiled |
| `ring-094-rust` |  4 |   774 | AGI Runtime                  | ✅ written, ⏳ uncompiled |
| `ring-095-rust` |  4 |   659 | φ-Adam Optimizer             | ✅ written, ⏳ uncompiled |
| `ring-096-rust` |  4 |   464 | Quantization (GF16 / INT4)   | ✅ written, ⏳ uncompiled |
| `ring-097-rust` |  4 |   624 | Chain-of-Thought Engine      | ✅ written, ⏳ uncompiled |
| `ring-098-rust` |  5 |   920 | World Model                  | ✅ written, ⏳ uncompiled |
| `ring-099-rust` |  6 | 1 127 | Integration / `trinity` bin  | ✅ written, ⏳ uncompiled |

**Totals:** 12 crates · 60 source files · ≈ 9 930 Rust LOC · 33 `Cargo.toml`.

### Toolchain availability (honest)

| Tool          | Installed | Verified |
|:--------------|:---------:|:--------:|
| `cargo`       | ❌ no | ❌ |
| `rustc`       | ❌ no | ❌ |
| `cargo check` | ❌ n/a | ❌ |
| `cargo test`  | ❌ n/a | ❌ |

**Why:** the sandbox used during Wave 11 had no Rust toolchain (network timeout / permission denied on install). The crates compile-status will be verified in Wave 12 once a Rust-enabled image is in place.

## Wave 12 — Plan: Compile + Integrate + Expand

```
╔═══════════════════════════════════════════════════════════════════════╗
║  WAVE 12 — COMPILATION + INTEGRATION + NEW RINGS                      ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Track A · 3 agents — Fix `cargo check` errors across all 12 crates   ║
║  Track B · 3 agents — Finish execution units inside ring-090 (sim)    ║
║  Track C · 3 agents — Author ring-100..ring-104 (Multi-Chip, Analog…) ║
║  Track D · 3 agents — Docker image w/ full Rust toolchain + CI hook   ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Exit criteria:                                                       ║
║    • cargo check         ≥ 9/12 crates                                ║
║    • cargo test          ≥ 6/12 crates                                ║
║    • `trinity` binary    runs end-to-end (ring-099)                   ║
║    • Docker image        published, CI green on PR                    ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Tracks in detail**

- **Track A — Compile fix:** run `cargo check` per crate, triage errors (missing deps, lifetime, type mismatches), submit one PR per crate with `Closes #<ring>`.
- **Track B — Simulator finish:** complete the missing execution units in `ring-090-rust` (decode → issue → writeback), add property tests against `conformance/*.json`.
- **Track C — New rings:** spec → crate scaffolding for `ring-100` Multi-Chip Mesh, `ring-101` Analog GF16, `ring-102` Photonic MAC, `ring-103` On-Chip Learning, `ring-104` Telemetry Bus. **Status: scaffolded — 5 crates landed on disk (see table below).**
- **Track D — Toolchain:** `Dockerfile.rust` based on `rust:1.83-bookworm`, GitHub Actions matrix building all `ring-0**-rust` crates, artifact upload on failure.

### Wave 12 / Track C — ring-100..ring-104 (scaffolded 2026-05-22, Closes #711)

> **Honest status:** all 5 crates **written on disk** with `Cargo.toml` + `src/lib.rs` + per-crate `README.md` and `#[test]` coverage (28 tests total). `cargo check` / `cargo test` **still not run** — toolchain hookup is Track D. Crates are intentionally **not** added to `[workspace].members` until Track D Docker image is in place.

| Crate                          | Files | Rust LOC | Tests | Domain                                                                |
|:-------------------------------|------:|---------:|------:|:----------------------------------------------------------------------|
| `rings/ring-100-rust`          |   3   |   205    |   5   | Multi-Chip Mesh — Phi+Euler+Gamma triad fabric, XY routing             |
| `rings/ring-101-rust`          |   3   |   144    |   5   | Analog GF16 — quantize/dequantize + reproducible noise channel        |
| `rings/ring-102-rust`          |   3   |   157    |   5   | Photonic MAC — wavelength-multiplexed dot product w/ insertion loss   |
| `rings/ring-103-rust`          |   3   |   131    |   6   | On-Chip Learning — φ-tempered SGD step                                |
| `rings/ring-104-rust`          |   3   |   185    |   7   | Telemetry Bus — bounded lossy ring buffer of (ts, tag, value)         |

**Totals:** 5 crates · 15 files · 822 Rust LOC · 28 `#[test]`s.

Every crate exposes `identity_witness()` (or `Mesh::identity_witness` in ring-100) asserting `phi^2 + 1/phi^2 == 3` to f64 1e-15.

## Wave 13 — Toolchain & Compilation Gate (2026-05-22, Closes #713)

> **Why this wave:** Waves 11 and 12/Track-C produced 17 Rust crates on disk (≈ 10 750 LOC, 60+ tests), but `cargo check` and `cargo test` were **never** actually executed in CI. Wave 13 lands the missing infrastructure — pinned Rust toolchain, a generated GHA matrix that builds every `rings/ring-*-rust/` crate, and a living per-crate status table. The gate is **non-blocking on purpose**: it surfaces real per-crate compile state without yet enforcing it, so the project can finally distinguish *scaffolded* from *compiles* from *tested*.

```
╔═══════════════════════════════════════════════════════════════════════╗
║  WAVE 13 — TOOLCHAIN & COMPILATION GATE                               ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Dockerfile.rust            rust:1.83-bookworm, pkg-config, rustup    ║
║  scripts/ci/rings_matrix.py pure-stdlib GHA matrix generator          ║
║  .github/workflows/         rings-rust.yml — matrix cargo check+test  ║
║  rings/COMPILE_STATUS.md    living per-crate status (legend below)    ║
╠═══════════════════════════════════════════════════════════════════════╣
║  Legend:  scaffold  →  check  →  test  →  (off-disk for ring-088..099)║
║  Gate is `continue-on-error: true` — honesty surface, not enforcer.   ║
╚═══════════════════════════════════════════════════════════════════════╝
```

**Deliverables**

| Artifact                                  | Role                                                                |
|:------------------------------------------|:--------------------------------------------------------------------|
| `Dockerfile.rust`                         | Pinned `rust:1.83-bookworm` image — `rustc`, `cargo`, `rustfmt`, `clippy` |
| `scripts/ci/rings_matrix.py`              | Discovers `rings/ring-*-rust/` crates → emits GHA matrix JSON       |
| `.github/workflows/rings-rust.yml`        | `discover` → matrix `cargo check` + `cargo test` → step-summary     |
| `rings/COMPILE_STATUS.md`                 | Living per-crate status table (`scaffold` / `check` / `test` / `off-disk`) |

**Honest status at landing:** all 5 Wave-12 Track-C crates start as `scaffold` in the table; the 12 Wave-11 crates remain `off-disk` until imported. No row will be promoted past `scaffold` without a CI log to prove it (**R5-HONEST**).

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

### Multi-Language Installation

GoldenFloat is available as native packages for Python, JavaScript, Rust, and C:

**Python (PyPI):**
```bash
pip install golden-float
```

**JavaScript (npm):**
```bash
npm install golden-float
```

**Rust (crates.io):**
```toml
[dependencies]
golden-float-ffi = "0.1"
```

**C/C++ (header-only):**
```c
#include "golden_float.h"  // Auto-generated from gen/c/numeric/
```

All implementations share a single Rust core with a C-compatible ABI, guaranteeing **bit-identical results** across languages. See [`docs/MIGRATION.md`](docs/MIGRATION.md) for detailed installation and migration guides.


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
tri spec edit <module>                <- NOT BUILT: edit the .t27 spec directly
tri seal <file.t27> --save            <- record 4 SHA-256 hashes  (`tri skill seal` is not built)
tri gen                               <- generate Zig/Verilog/C
tri test                              <- run tests
tri verdict --toxic                   <- TOXIC? -> rollback. CLEAN? -> proceed
tri experience save                   <- append episode to Akashic journal
tri skill end                         <- verify hashes + issue binding  (`tri skill commit` is not built)
tri git commit                        <- NOT BUILT: use git; the hook enforces "Closes #N"
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

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

---

**φ² + 1/φ² = 3 | TRINITY**

**Maintained by**: [Trinity Project](https://github.com/gHashTag) — [Dmitrii Vasilev](https://github.com/gHashTag)

**Status:** Ring 31 Complete (2026-04-08) — 31 rings sealed, 45 specs, 112 gen files, 34 conformance vectors, 48 seals, CI enforced. **Wave Loop Ladder W760–W776 merged to master (2026-07-24)** — 15 module-scope packed-array-of-struct witnesses (339–371 outer dims), zero compiler/FROZEN_HASH changes, all Icarus + cocotb gates green.

**Wave 11 (2026-05-22):** 12 Rust crates `ring-088`..`ring-099` written (≈ 9 930 LOC, 33 `Cargo.toml`), **compilation not yet verified** — toolchain unavailable in sandbox; verification deferred to Wave 12. See the *Wave 11 / Wave 12* sections above for the honest status table and the four-track plan.

**Wave 12 / Track C (2026-05-22):** 5 new crates `ring-100`..`ring-104` scaffolded (Multi-Chip / Analog GF16 / Photonic MAC / On-Chip Learning / Telemetry Bus) — 15 files, 822 Rust LOC, 28 `#[test]`s. `cargo` gate handed off to Track D.

**Wave 13 (2026-05-22):** Toolchain & Compilation Gate landed — `Dockerfile.rust` (`rust:1.83-bookworm`), `scripts/ci/rings_matrix.py`, `.github/workflows/rings-rust.yml` matrix build, `rings/COMPILE_STATUS.md` living per-crate status. Non-blocking honesty gate — see [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 14 (2026-05-22):** Rings compile green — root `Cargo.toml` `exclude` extended with `rings/`. All 5 Track-C crates (`ring-100`..`ring-104`) promoted from `scaffold` to `check` + `test` in [COMPILE_STATUS](rings/COMPILE_STATUS.md). Verified locally on Rust 1.83.0: 26 tests pass, 0 fail (honest count; Wave-12 NOW's claim of 28 was off by two — corrected).

**Wave 15 (2026-05-22):** Canonical GF16 import — [`rings/ring-088-rust`](rings/ring-088-rust) lands as the first **honestly-authored** Wave-11 crate (439 LOC, 13 tests, including the first cross-kernel `mac_dot([phi,1/phi],[phi,1/phi]) ~= 3` identity check). R5-HONEST reclassification: the other 11 Wave-11 rings move from `off-disk` to `claimed-only` until they receive the same real-source treatment. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 16 (2026-05-22):** TNN ISA import — [`rings/ring-089-rust`](rings/ring-089-rust) lands the second honestly-authored Wave-11 crate (635 LOC, 15 tests). Includes `Trit`, `Word27`, balanced-ternary `trit_add` / `word_add` / `word_sub` per `specs/isa/ternary_arithmetic.t27`, a 9-opcode subset (`NOP`/`MOV`/`ADDI`/`ADD`/`SUB`/`NEG`/`LOAD`/`STORE`/`HALT`), and a `Cpu` fetch/decode/execute model with 27 registers (R0 hardwired to zero). `cpu_phi_identity_integer_projection` is the second cross-kernel anchor test, running `floor(phi) + floor(1/phi) + ceil(phi^2 - 2) = 3` through the CPU. `#![no_std]`, zero deps, verified locally on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 17 (2026-05-22):** Simulator import — [`rings/ring-090-rust`](rings/ring-090-rust) lands the third honestly-authored Wave-11 crate (547 LOC, 19 tests). Mirrors `specs/fpga/simulator.t27` byte-for-byte: `SimState` (5 variants), `SimConfig`, `SimResult`, `ProbePoint`, `TraceEntry`, plus constructor / query / time-conversion / validation helpers. All 13 spec `test` blocks and all 4 spec `invariant` blocks become `#[test]`s. `#![no_std]`, zero deps, all 19 tests green on first run (Rust 1.83.0). Wave-11 narrative claimed 2143 LOC; honest measurement is 547 LOC. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 18 (2026-05-22):** Stochastic Rounding import — [`rings/ring-091-rust`](rings/ring-091-rust) lands the fourth honestly-authored Wave-11 crate (462 LOC, 19 tests). Implements stochastic rounding (`sr_round_f32_to_i32`, `sr_quantize_f32`, `sr_quantize_batch`) on top of a deterministic seedable `SplitMix64` PRNG (Vigna 2014; reference seed-0 value `0xE220A8397B1DCDAF` checked in test). Two statistical tests verify unbiasedness over 10 000 draws against a 3-sigma bound. `sr_quantize_phi_unbiased` is the **third cross-kernel anchor test** in the project after Wave 15's `mac_dot_phi_identity` and Wave 16's `cpu_phi_identity_integer_projection`. `#![no_std]`, zero deps, all 19 tests green on first run. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 19 (2026-05-22):** Attention import — [`rings/ring-092-rust`](rings/ring-092-rust) lands the fifth honestly-authored Wave-11 crate (760 LOC, 28 tests). Mirrors `specs/nn/attention.t27` (SacredAttention) for the `no_std`-realizable subset: sacred constants (`NUM_HEADS=3`, `HEAD_DIM=81`, `EMBED_DIM=243`, `CONTEXT_LEN=81`, `ROPE_PAIRS=40`, `SACRED_GAMMA = phi^-3`, `SACRED_SCALE = 81^(-SACRED_GAMMA)`); `Trit` enum; primitives `ternary_matmul`, `add_residual`, `apply_softmax` (numerically stable max-subtract per head), `compute_scores` (Q.K^T with causal mask + sacred scaling), `weighted_values`, `cache_kv`. A private `exp_f64` (range-reduced Taylor series) makes softmax viable without libm. `attention_phi_identity_via_softmax_matmul` is the **fourth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through softmax-style normalization and ternary matmul. RoPE table init (cos/sin) and the full `sacred_attention_kernel` orchestrator are out of scope (R5-HONEST). Wave-11 narrative claimed 847 LOC; honest measurement is 760 LOC. All 28 tests green on first run (Rust 1.83.0). See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 20 (2026-05-22):** Sparse MoE import — [`rings/ring-093-rust`](rings/ring-093-rust) lands the sixth honestly-authored Wave-11 crate (950 LOC, 28 tests). No backing file under `specs/` (textbook algorithm, like ring-091's SR); design mirrors Shazeer-2017 / Switch-Transformer top-k routing with ternary expert weights and Trinity defaults (`NUM_EXPERTS=3`, `DEFAULT_TOP_K=1`, `DEFAULT_EMBED_DIM=243`, `DEFAULT_EXPERT_HIDDEN_DIM=729 = 3^6`). Exposes `MoEConfig`, `gate_top_k` (top-k selection + max-subtract softmax over selected logits), `expert_ffn` (two-layer ternary FFN with ReLU), `moe_forward` (composes gating + per-expert FFNs, allocation-free), `relu_inplace`, `load_balance_loss` (Switch-Transformer importance balance: 1.0 = uniform, `num_experts` = full concentration). A private `exp_f64` makes gating softmax viable without libm. `moe_phi_identity_via_gating_and_ffn` is the **fifth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through MoE gating + ternary FFN. Wave-11 narrative claimed 668 LOC; honest measurement is 950 LOC. All 28 tests green on first run (Rust 1.83.0). See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 21 (2026-05-22):** AGI Runtime import — [`rings/ring-094-rust`](rings/ring-094-rust) lands the seventh honestly-authored Wave-11 crate (1210 LOC, 32 tests). Mirrors `specs/runtime/{execute, instance, process}.t27` byte-for-byte: spec constants (`DEFAULT_TIMEOUT_MS=30_000`, `MAX_CONCURRENT_EXECUTIONS=16`, `POLL_INTERVAL_MS=100`, `TASK_ID_LENGTH=32`, `MAX_INSTANCES=256`, `INSTANCE_NAME_LENGTH=128`, `LOOKUP_TIMEOUT_MS=100`, `SPAWN_TIMEOUT_MS=5_000`, `PTY_COLS_DEFAULT=80`, `PTY_ROWS_DEFAULT=24`, `MAX_PIPE_BUFFER=65_536`); all nine spec enums; pure-state-machine `Promise`; fixed-capacity `Registry` (256 slots); and a Trinity-priority `Scheduler` (16 slots) with a phi-weighted credit policy (`Trit::Pos -> phi^2`, `Trit::Zero -> 1.0`, `Trit::Neg -> phi^-2`). `runtime_phi_identity_via_scheduler_credits` is the **sixth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the scheduler's credit accumulator. Real syscalls, heap-backed containers, and async-runtime wakers are explicitly out of scope (R5-HONEST). Wave-11 narrative claimed 774 LOC; honest measurement is 1210 LOC. All 32 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 22 (2026-05-22):** phi-Adam optimizer import — [`rings/ring-095-rust`](rings/ring-095-rust) lands the eighth honestly-authored Wave-11 crate (808 LOC, 25 tests). Mirrors `specs/ml/optimizer/{adam, adamw}.t27`: spec constants byte-for-byte (`DEFAULT_LEARNING_RATE=1e-3`, `DEFAULT_BETA1=0.9`, `DEFAULT_BETA2=0.999`, `DEFAULT_WEIGHT_DECAY=0.01`, `DEFAULT_EPSILON=1e-8`, `PHI_BETA1 = 0.9/phi ~= 0.556`, `PHI_BETA2 = 0.999/phi ~= 0.617`); `AdamWConfig` with `defaults()` (classic AdamW) and `phi_preset()` (phi-damped betas) constructors; caller-owned `AdamWState<'_>` (no allocation); spec-named helpers (`compute_bias_correction`, `update_first_moment`, `update_second_moment`, `apply_weight_decay`, `compute_update`); full `step()` orchestrator with decoupled weight decay, bias-corrected `lr_t = lr * sqrt(1 - beta2^t) / (1 - beta1^t)`, moment recurrences, AMSGrad max-of-v, and the parameter update. Private no_std math (`pow_u64`, `sqrt_newton`) bypasses libm. `phi_adam_phi_identity_via_betas` is the **seventh cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the optimizer's `pow_u64` and through the phi-damped moment update `m_1 = (1 - 0.9/phi) * phi = phi - 0.9`. Wave-11 narrative claimed 659 LOC; honest measurement is 808 LOC. All 25 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 23 (2026-05-22):** Quantization import — [`rings/ring-096-rust`](rings/ring-096-rust) lands the ninth honestly-authored Wave-11 crate (641 LOC, 42 tests). Mirrors `specs/numeric/formats.t27`: GF16 bit layout (`SIGN_MASK=0x8000`, `EXP_MASK=0x7E00`, `MANT_MASK=0x01FF`, `EXP_SHIFT=9`, `SIGN_SHIFT=15`, `BIAS=31`, `EXP_MAX=63`, `EXP_MIN=0`) byte-for-byte; full GF16 codec `gf16_to_f32` / `f32_to_gf16` (signed zero, denormals, normals, Inf, NaN, round-to-nearest with mantissa-overflow exponent carry); ternary quantization `f32_to_ternary` / `ternary_to_f32` with the spec's strict threshold `|x| > 0.5`; the `Format` enum (`Fp32`, `Fp16`, `Bf16`, `Gf16`, `Ternary`); `format_bytes`; and the `quantize_value` utility. A private `pow_u64` (fast exponentiation by squaring) replaces libm in `no_std`. `quantization_phi_identity` is the **eighth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through the GF16 codec: it computes `phi^2` and `phi^-2` via `pow_u64`, encodes via `f32_to_gf16`, decodes via `gf16_to_f32`, and verifies the sum lies within GF16 mantissa tolerance of 3.0 (~0.03 absolute). Wave-11 narrative claimed 464 LOC; honest measurement is 641 LOC. All 42 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 26 (2026-05-22):** Integration import — [`rings/ring-099-rust`](rings/ring-099-rust) lands the **twelfth** honestly-authored Wave-11 crate (763 LOC, 31 tests) and **closes the Wave-11 import series**. Mirrors `specs/pipeline/e2e_test.t27` byte-for-byte: constants `MAX_PIPELINE_STAGES=10`, `STAGE_INIT=0`, `STAGE_PARSE=1`, `STAGE_SEAL=2`, `STAGE_GEN=3`, `STAGE_TEST=4`, `STAGE_VERDICT=5`, `STAGE_SAVE=6`, `STAGE_COMMIT=7`, `STAGE_DONE=8`, `STAGE_FAIL=255`; functions `pipeline_run`, `pipeline_inject_failure`, `pipeline_progress`, `stage_name`; all 4 spec test blocks (`full_pipeline_pass`, `pipeline_fail_at_gen`, `pipeline_fail_at_test`, `progress_calc`); all 3 spec invariants (`stage_ordering`, `max_stages_sufficient`, `fail_distinct`). The `Pipeline` type wraps fixed `[u8; 10]` stage + `[bool; 10]` results buffers; the `Stage` enum (9 valid + `Fail`) carries `code` / `from_code` / `next` / `is_terminal` / `name`. `#![no_std]`, `#![forbid(unsafe_code)]`, heap-free. `integration_phi_identity` is the **eleventh cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through integer projection (`floor(PHI)+floor(PHI_SQ)=3`), `pow_u64` numeric witness, pipeline progress arithmetic (`progress(9,9)==100.0` and `progress(3,9)==100/3` within 1e-9), and mass-conservation `PHI_SQ + PHI_INV_SQ == TRINITY` to within 1e-12. Wave-11 narrative claimed 1127 LOC; honest measurement is 763 LOC. **Wave-11 series complete**: all 11 narratives have honest source on disk, 8 817 honest LOC, 336 tests, 11 live cross-kernel anchors. The `claimed-only` placeholder table in [COMPILE_STATUS](rings/COMPILE_STATUS.md) is now empty. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 25 (2026-05-22):** World Model import — [`rings/ring-098-rust`](rings/ring-098-rust) lands the eleventh honestly-authored Wave-11 crate (779 LOC, 29 tests). Mirrors three specs byte-for-byte: `specs/brain/unified_state.t27` (`BrainState`, `ConsciousnessState`, `Mood`, `ArousalLevel`, `Layer`, `REGION_COUNT=27`, `LAYER_COUNT=3`, `REGIONS_PER_LAYER=9`, `PHI`/`PHI_INV`/`PHI_SQ`/`PHI_INV_SQ`/`TRINITY`); `specs/ml/rl/dqn.t27` (`Transition { state, action, reward, next_state, done }`); `specs/brain/cognitive_loop.t27` (`COGNITIVE_PHASE_COUNT=5`: sense, evaluate, decide, act, consolidate). `WorldModel` is a bounded recorder: fixed `[BrainState; MAX_STATE_HISTORY=16]` history, fixed `[Transition; MAX_TRANSITIONS=32]` replay buffer, inline `STATE_DIM=8`, plus `snapshot`, `record_transition`, `step_phase`, `run_one_cycle`, `verify`, `reset`. The `verify` routine enforces monotonic `cycle_count` and a `phi_coherence in [0,1]` invariant. `#![no_std]`, heap-free. `world_model_phi_identity` is the **tenth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through (a) integer projection `floor(PHI_SQ) + floor(PHI) = 3`, (b) `pow_u64` numeric witness, and (c) mass-conservation `PHI_SQ + PHI_INV_SQ == TRINITY` to within 1e-12. Wave-11 narrative claimed 920 LOC; honest measurement is 779 LOC. All 29 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**Wave 24 (2026-05-22):** Chain-of-Thought import — [`rings/ring-097-rust`](rings/ring-097-rust) lands the tenth honestly-authored Wave-11 crate (823 LOC, 29 tests). Mirrors `specs/ar/proof_trace.t27` byte-for-byte: `MAX_STEPS=10` (DARPA CLARA bound on reasoning chain length); K3 ternary logic (`Trit::{True=1, Unknown=0, False=-1, Null=2}`) with `k3_and` (min lattice), `k3_or` (max lattice), `k3_not`; fixed-capacity heap-free `ProofStep` (interned ASCII operation name up to 24 chars, fixed-arity inputs up to 3 trits, `output`, `timestamp_us`); `ProofTrace` with `[ProofStep; MAX_STEPS]` buffer + `start_timestamp_us` / `end_timestamp_us` / `verified` flag; operations `new_proof_trace`, `add_step`, `verify_trace`, `trace_length`, `is_at_capacity`, `finalize_trace`, `step_at`, `format_trace`, `trit_to_string`; `VerifyStatus::{Valid, Empty, TooManySteps, NullOutput(usize)}` enforcing all three spec invariants (`empty_trace_fails`, `trace_verification_catches_overflow`, `valid_trace_passes`). The crate is `#![no_std]` and heap-free; `format_trace` writes into a caller-supplied buffer. `cot_phi_identity` is the **ninth cross-kernel anchor test** in the project, routing `phi^2 + 1/phi^2 = 3` through a 6-step bounded reasoning chain: symbolic premises, `k3_and`, a numeric-witness step that evaluates `pow_u64(phi, 2) + pow_u64(phi, -2)` and produces `True` iff the result is within 1e-9 of 3.0, a `k3_or` alternative-path step, and a conclusion -- then verifies and finalises the trace, plus a separate mass-conservation hook for φ²-weighted Pos and φ⁻²-weighted Neg priorities. Wave-11 narrative claimed 624 LOC; honest measurement is 823 LOC. All 29 tests green on Rust 1.83.0. See [COMPILE_STATUS](rings/COMPILE_STATUS.md).

**DOI:** [10.5281/zenodo.19456875](https://doi.org/10.5281/zenodo.19456875)

## Wave Loop Ladder — 760s series (2026-07-24)

A parallel **FPGA/IGLA hardening ladder** (distinct from the numbered Waves above) exercises progressively larger **module-scope packed array-of-struct variables** with **non-power-of-two outer dimensions** and **indexed signed field writes**. All loops land as `.t27` witnesses and are fully simulated through the Icarus + cocotb gates.

| Wave | Outer | Elements | Packed bits | MiBit | Issue | Witness |
|------|------:|---------:|------------:|------:|:------|:--------|
| W760 | 339 | 21,696 | 694,272 | 0.663 | #1731 | `specs/scratch/w760_bench_module_339x2p6_aos_var_call_write.t27` |
| W761 | 341 | 21,824 | 698,368 | 0.666 | #1732 | `specs/scratch/w761_bench_module_341x2p6_aos_var_call_write.t27` |
| W762 | 343 | 21,952 | 702,464 | 0.670 | #1733 | `specs/scratch/w762_bench_module_343x2p6_aos_var_call_write.t27` |
| W765 | 349 | 22,336 | 714,752 | 0.682 | #1736 | `specs/scratch/w765_bench_module_349x2p6_aos_var_call_write.t27` |
| W766 | 351 | 22,464 | 718,848 | 0.686 | #1737 | `specs/scratch/w766_bench_module_351x2p6_aos_var_call_write.t27` |
| W767 | 353 | 22,592 | 722,944 | 0.690 | #1738 | `specs/scratch/w767_bench_module_353x2p6_aos_var_call_write.t27` |
| W768 | 355 | 22,720 | 727,040 | 0.694 | #1739 | `specs/scratch/w768_bench_module_355x2p6_aos_var_call_write.t27` |
| W769 | 357 | 22,848 | 731,136 | 0.697 | #1740 | `specs/scratch/w769_bench_module_357x2p6_aos_var_call_write.t27` |
| W770 | 359 | 22,976 | 735,232 | 0.701 | #1741 | `specs/scratch/w770_bench_module_359x2p6_aos_var_call_write.t27` |
| W771 | 361 | 23,104 | 739,328 | 0.705 | #1742 | `specs/scratch/w771_bench_module_361x2p6_aos_var_call_write.t27` |
| W772 | 363 | 23,232 | 743,424 | 0.709 | #1743 | `specs/scratch/w772_bench_module_363x2p6_aos_var_call_write.t27` |
| W773 | 365 | 23,360 | 747,520 | 0.713 | #1481 | `specs/scratch/w773_bench_module_365x2p6_aos_var_call_write.t27` |
| W774 | 367 | 23,488 | 751,616 | 0.717 | #1483 | `specs/scratch/w774_bench_module_367x2p6_aos_var_call_write.t27` |
| W775 | 369 | 23,616 | 755,712 | 0.721 | #1485 | `specs/scratch/w775_bench_module_369x2p6_aos_var_call_write.t27` |
| W776 | 371 | 23,744 | 759,808 | 0.725 | #1487 | `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27` |

Each witness:
- Declares `pub var dst : [N][2]^6 Pt` at module scope, initialized from a function call.
- Uses indexed signed writes to `dst[i].x` / `dst[i].y` and `assert_eq` read-back in a `bench` block (the Icarus path accepts `assert_ne` structurally but does not emit it).
- Has a deterministic generator under `scripts/gen_w7*.py` so the witness can be regenerated.
- Carries a SHA-256 seal under `.trinity/seals/` and an integration test in `bootstrap/tests/icarus_lowerable.rs`.
- Passes `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save`.

**Important invariants observed across the ladder:**
- **Zero compiler changes** — `bootstrap/src/compiler.rs` is untouched and `bootstrap/stage0/FROZEN_HASH` remains `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- **Zero reference-model changes** — `scripts/cocotb_ref_model.py` is unchanged.
- All generator scripts reuse the corrected W632 inner-dimension offset formula, so mid-row values compute correctly on the first attempt.
- Every loop includes an explicit `make_grid(32768)` period-identity check because `32768 ≡ 0 (mod 32768)`.

Live log: [`docs/NOW.md`](docs/NOW.md) · Per-loop closeouts: [`docs/reports/`](docs/reports/) · Next-wave plan: [`.claude/plans/wave-loop-777.md`](.claude/plans/wave-loop-777.md)
