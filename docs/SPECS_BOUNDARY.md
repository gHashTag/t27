# Specs Boundary — Core vs Research

**Status:** Active (Ring 036)
**Date:** 2026-05-09
**Purpose:** Define the boundary between core specs and research specs in t27

---

## 1. Definition

### Core Specs (`specs/core/`)

Core specs are **production-critical** specifications that:

1. Define the **t27 language** (syntax, semantics, runtime)
2. Specify the **GoldenFloat family** (GF4-GF32)
3. Implement the **compiler pipeline** (lexer, parser, codegen)
4. Define **sacred physics constants** with strict tolerances
5. Specify **FPGA synthesis** targets for production hardware

**Properties:**
- MUST have conformance vectors in `conformance/`
- MUST be sealed (SHA-256 hash in `.trinity/seals/`)
- MUST be included in CI `tri test` suite
- Changes require GitHub Issue and PR to `master`

### Research Specs (`specs/research/`)

Research specs are **exploratory** specifications that:

1. Explore **new algorithms** or architectures
2. Implement **experimental features** not yet production-ready
3. Define **hypothetical extensions** to the language
4. Document **theoretical work** without implementation
5. Contain **speculative claims** without full proof

**Properties:**
- MAY have conformance vectors (optional)
- MAY be unsealed
- NOT included in core CI suite
- Changes do not require strict issue gating

---

## 2. Current Classification

### Core Specs Directory

```
specs/core/
├── 00-gf-family-foundation.tri        # GoldenFloat family definition
├── 00-trib-format.tri                 # TRI-27 binary format
├── 01-tri-lang-core.tri              # Language core syntax
├── 01-vm-core.tri                    # Trinity VM core
├── 02-gf16-format.tri                # GF16 format specification
├── 03-bootstrap-lexer.tri            # Bootstrap lexer
├── 03-simple-parser.tri              # Bootstrap parser
├── 03-tri-bootstrap-compiler.tri     # Bootstrap compiler
├── 04-tri-codegen.tri                # Codegen specification
├── 04-tri-runtime.tri                # Runtime specification
├── 05-tri-test-runner.tri            # Test runner
├── 06-tri-bench-runner.tri           # Benchmark runner
├── 07-trib-vm-executor.tri           # VM executor
└── 08-gf32-scientific-demo.tri       # GF32 demo

base/
├── types.t27                         # Core type system
└── ops.t27                           # Core operations

numeric/
├── gf4.t27                           # GF4 format
├── gf8.t27                           # GF8 format
├── gf12.t27                          # GF12 format
├── gf16.t27                          # GF16 format (PRIMARY)
├── gf20.t27                          # GF20 format
├── gf24.t27                          # GF24 format
├── gf32.t27                          # GF32 format
├── goldenfloat_family.t27            # Family registry
├── phi_ratio.t27                     # φ-derivation
├── tf3.t27                           # Ternary float
└── trinity_numeric_surface.t27       # Public surface

math/
├── constants.t27                     # Sacred constants
└── sacred_physics.t27                # Sacred physics

compiler/
├── parser/
│   ├── lexer.t27                     # Lexer spec
│   └── parser.t27                    # Parser spec
├── codegen/
│   ├── zig.t27                       # Zig codegen
│   ├── c.t27                         # C codegen
│   ├── verilog.t27                   # Verilog codegen
│   ├── rust.t27                      # Rust codegen
│   └── typescript.t27                # TypeScript codegen
├── cli/
│   ├── gen.t27                       # Gen command
│   ├── git.t27                       # Git command
│   └── spec.t27                      # Spec command
├── runtime/
│   ├── commands.t27                  # Runtime commands
│   └── validation.t27                # Validation
└── typechecker.t27                   # Typechecker

fpga/
├── mac.t27                           # MAC unit
├── bridge.t27                        # Bridge specs
├── spi.t27                           # SPI interface
├── uart.t27                          # UART interface
├── axi4.t27                          # AXI4 bus
└── apb_bridge.t27                    # APB bridge

ar/                                   # CLARA AR pipeline
├── ternary_logic.t27                 # Kleene K3 logic
├── proof_trace.t27                   # Proof traces
├── datalog_engine.t27                # Datalog engine
├── restraint.t27                     # Bounded rationality
├── explainability.t27                # XAI
├── asp_solver.t27                    # ASP solver
└── composition.t27                   # ML+AR composition
```

### Research Specs Directory

```
specs/research/
├── physics/
│   ├── chern_simons_k3.t27           # Chern-Simons K3 (speculative)
│   └── su2_chern_simons.t27          # SU2 Chern-Simons (experimental)
│
├── ml/
│   ├── activation/                   # Activation functions ( exploratory)
│   ├── layers/                       # Layer implementations (experimental)
│   ├── optimizer/                    # Optimizers (exploratory)
│   ├── recurrent/                    # Recurrent architectures (experimental)
│   ├── transformer/                  # Transformer components (research)
│   └── rl/                           # Reinforcement learning (exploratory)
│
├── nn/
│   ├── attention.t27                 # Attention (transitioning to core)
│   └── hslm.t27                      # HSLM architecture (research)
│
├── vsa/
│   ├── core.t27                      # VSA core (experimental)
│   └── ops.t27                       # VSA operations (research)
│
├── isa/
│   └── ternary_encoding.t27          # Ternary encoding (research)
│
├── queen/
│   └── lotus.t27                     # Lotus orchestration (research)
│
└── demos/
    ├── simple_test.t27               # Demo specs
    └── jones_*.t27                   # Jones polynomial demos
```

---

## 3. Migration Path

### From Research to Core

A research spec migrates to core when:

1. **Stability:** No breaking changes for 3+ rings
2. **Conformance:** Has comprehensive test vectors
3. **Seal:** Has SHA-256 seal in `.trinity/seals/`
4. **Documentation:** Has complete docstrings and examples
5. **Production use:** Used by at least one production system

**Process:**
1. Create GitHub Issue: `[MIGRATION] Move <spec> from research to core`
2. Add conformance vectors if missing
3. Add to CI `tri test` suite
4. Seal spec with `tri seal <spec.t27> --save`
5. Move file from `specs/research/` to `specs/core/`
6. Update `docs/SPECS_BOUNDARY.md`
7. Close issue with PR

### From Core to Research (Debt)

A core spec demotes to research when:

1. **Deprecation:** No longer used in production
2. **Instability:** Frequent breaking changes
3. **Incompleteness:** Missing conformance vectors
4. **Superseded:** Replaced by better implementation

**Process:**
1. Create GitHub Issue: `[DEPRECATION] Move <spec> from core to research`
2. Update documentation
3. Remove from CI `tri test` suite
4. Move file from `specs/core/` to `specs/research/`
5. Archive old seals
6. Update `docs/SPECS_BOUNDARY.md`
7. Close issue with PR

---

## 4. Ownership

| Directory | Primary Agent | Backup Agent |
|-----------|---------------|--------------|
| `specs/core/` | C (Compiler) | N (Numeric) |
| `specs/research/physics/` | P (Physics) | A (Architecture) |
| `specs/research/ml/` | H (HSLM) | C (Compiler) |
| `specs/research/nn/` | H (HSLM) | Q (Queen) |
| `specs/research/ar/` | V (Verdict) | Q (Queen) |
| `specs/research/vsa/` | V (Verdict) | N (Numeric) |
| `specs/demos/` | Z (Docs) | C (Compiler) |

---

## 5. CI Integration

```yaml
# .github/workflows/phi-loop-ci.yml
- name: Validate core specs
  run: |
    ./scripts/tri test --core-only

- name: Validate research specs (optional)
  if: github.event_name == 'schedule'
  run: |
    ./scripts/tri test --research-only
```

---

## 6. Statistics

| Category | Count | Status |
|----------|-------|--------|
| Core specs | ~200 | Stable, sealed |
| Research specs | ~300 | Experimental |
| Migrations pending | 0 | None |
| Deprecations pending | 0 | None |

---

## 7. Decision Matrix

When creating a new spec, ask:

| Question | Yes → | No → |
|----------|-------|------|
| Is this for production use? | `specs/core/` | `specs/research/` |
| Does it need conformance vectors? | `specs/core/` | `specs/research/` |
| Is it part of the language spec? | `specs/core/` | `specs/research/` |
| Is it experimental/exploratory? | `specs/research/` | `specs/core/` |
| Is it a demo or example? | `specs/demos/` | `specs/research/` |

---

## 8. References

- `docs/T27-CONSTITUTION.md` — Constitutional law
- `docs/EPOCH_01_HARDEN_PLAN.md` — Hardening plan
- `docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md` — GF debt
- `SOUL.md` — Constitutional law

---

**φ² + 1/φ² = 3 | TRINITY**
