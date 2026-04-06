# CLARA-Bridge: Compositional Assurance Pattern Extraction

Inspired by goals similar to public descriptions of high-assurance compositional AI (e.g., DARPA CLARA), this project extracts a development pattern from the Trinity t27 codebase for building verified, explainable software systems.

## What is this?

CLARA-Bridge demonstrates a formal development pipeline where every component is specified, generated, tested, and verified before composition. The pattern ensures that:

1. **Formal contracts** (.t27 specs) define inputs, outputs, preconditions
2. **Executable verification** (`tri test`) provides conformance checking
3. **Toxicity detection** (`tri verdict`) marks regressions in phi-critical components
4. **Audit trail** (`.trinity/experience/`) records all attempts and resolutions

## The Pattern

```
specification (.t27) → generation (tri gen) → testing (tri test) → verdict (tri verdict) → experience (.trinity/experience/)
```

### Pipeline Phases

| Phase | Command | Purpose |
|-------|----------|---------|
| **spec** | `tri skill seal --hash` | Cryptographically seal specification |
| **gen** | `tri gen <spec>` | Generate executable code from spec |
| **test** | `tri test <conformance>` or `t27c conformance <json>` | Run conformance tests |
| **verdict** | `python conformance/kepler_newton_tests.py --category <CS\|sacred\|E8>` | High-precision verification |
| **experience** | `.trinity/experience/` (auto-recorded) | Audit trail for verified learning |

> **Note:** The `verify_by` fields in scenario JSON describe human-in-the-loop checks for the current MVP. Automated scenario execution is provided by `run_scenario.py`.

## Use Cases

### 1. Vetted Logic Blocks (`vetted-blocks/`)

Browse cataloged components before using them in composition. Each entry includes:
- **Exports**: API surface provided by the block
- **Test invariant**: The mathematical guarantee provided
- **Toxic regression**: Downstream impact if broken
- **Sacred level**: Foundation, phi-critical, or sacred-core

Example subgraph:
```
math/constants (node 4)
    ↓ [phi-critical import]
physics/chern-simons (node 54)
    ↓ [sacred-core import]
math/sacred_physics (node 17)
```

### 2. Scenario Execution (`scenarios/`)

Run verified composition chains end-to-end:

```bash
# Automated execution
python clara-bridge/run_scenario.py clara-bridge/scenarios/chern-simons-phi-verification.json

# Dry-run (print commands only)
python clara-bridge/run_scenario.py --dry-run clara-bridge/scenarios/chern-simons-phi-verification.json

# Run specific step
python clara-bridge/run_scenario.py --step 3 clara-bridge/scenarios/chern-simons-phi-verification.json
```

Each step validates expected outcome before proceeding. Exit codes indicate success/failure.

### 3. Verification Reports (`explainability/`)

Understand what guarantees were provided:

- **Test execution**: Exact commands run
- **Interpretation**: Mathematical derivation and meaning
- **Toxicity impact**: Downstream modules affected if broken
- **References**: Academic papers and spec files

### 4. Audit Trail (`audit-trail/`)

Schema for `.trinity/experience/` learning recording:

- **NOT** gradient training
- **Verified learning** from sealed episodes only
- `mistakes.jsonl` = quarantine list (blocks tri gen)
- `episodes.jsonl` = full episode records

## Installation

```bash
# Verify Trinity pipeline is available
./tri --version

# Run a scenario (automated execution with dependency checking)
python clara-bridge/run_scenario.py clara-bridge/scenarios/chern-simons-phi-verification.json

# Dry-run (print commands without executing)
python clara-bridge/run_scenario.py --dry-run clara-bridge/scenarios/chern-simons-phi-verification.json

# Run specific step
python clara-bridge/run_scenario.py --step 3 clara-bridge/scenarios/chern-simons-phi-verification.json

# Verbose output
python clara-bridge/run_scenario.py --verbose clara-bridge/scenarios/chern-simons-phi-verification.json
```

### Manual Step Execution

If running steps manually (not via `run_scenario.py`):

```bash
# 1. Seal spec
t27c seal specs/math/constants.t27

# 2. Generate code
t27c gen specs/math/constants.t27

# 3. Run tests
t27c conformance conformance/math_constants.json

# 4. High-precision verification
python conformance/kepler_newton_tests.py --category CS      # Chern-Simons
python conformance/kepler_newton_tests.py --category sacred  # Sacred physics
```

## The Sacred Chain

The verified composition path documented in this bridge:

```
math/constants
  └─> exports: PHI, TRINITY, PI, GAMMA_LQG, ...
       └─> invariant: PHI^2 + PHI^-2 = 3

physics/su2_chern_simons (Chern-Simons)
  └─> imports: math::constants, math::sacred_physics
  └─> exports: d_tau, trinity_identity, fibonacci_fusion, ...
  └─> invariant: d_τ = φ at k=3 (Fibonacci anyon)
  └─> verification: kepler_newton_tests.py --category CS

math/sacred_physics
  └─> imports: math::constants
  └─> exports: verify_sacred_physics, sacred_gravity, sacred_dark_energy, ...
  └─> invariant: TRINITY = 3.000000 (within 1e-12)
```

**Verification precision**: Uses `mpmath` library with 50+ decimal places for all CS and sacred physics formulas.

**Experience recording**: Trinity auto-records sealed episodes to `.trinity/experience/` after successful `tri gen` runs. No manual save command required for verified builds.

**Verification precision**: Uses `mpmath` library with 50+ decimal places for all CS and sacred physics formulas.

## Toxicity Policy

When an invariant is violated:
1. **Detection**: `tri verdict` or high-precision test marks as toxic
2. **Quarantine**: Entry added to `.trinity/experience/mistakes.jsonl`
3. **Block**: Downstream phi-critical modules (`nn/attention`, `nn/hslm`) blocked
4. **Resolve**: Explicit fix, removal from mistakes.jsonl, re-verification

## Files Structure

```
clara-bridge/
├── vetted-blocks/          # JSON catalog of logic blocks from graph_v2.json
│   └── math-constants-sacred-chain.json
├── scenarios/                # Step-by-step execution scenarios
│   └── chern-simons-phi-verification.json
├── explainability/            # Template for guarantee explanations
│   └── su2-chern-simons-phi-guarantee.md
└── audit-trail/              # Schema for .trinity/experience/
    └── experience-schema.json
```

## Integration

This bridge integrates with existing Trinity tooling:
- **CLI**: Uses `tri` commands (seal, gen, test, skill)
- **Conformance**: Leverages `conformance/*.json` files
- **High-precision**: Uses `conformance/kepler_newton_tests.py`
- **Experience**: Records to `.trinity/experience/`

See `scripts/clara/README.md` for automated demo pipeline.

## License

Apache License 2.0 — See LICENSE file for details.

## Disclaimer

This is a pattern extraction and documentation project. The "CLARA" framing is inspired by public descriptions of compositional AI assurance goals. This is NOT affiliated with any DARPA program.

The mathematical formulas and verification methods are documented for educational and research purposes.
