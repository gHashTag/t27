# CLARA Test Vectors Package

This package contains test vectors for DARPA CLARA TA1 and TA2 submission.

## Structure

```
test_vectors/
├── README.md
├── ta1/
│   ├── ternary_logic.json       (11 test cases)
│   ├── proof_trace.json         (8 test cases)
│   ├── datalog_engine.json      (5 test cases)
│   ├── restraint.json           (4 test cases)
│   ├── explainability.json      (2 test cases)
│   ├── asp_solver.json          (3 test cases)
│   └── composition.json         (4 test cases)
└── ta2/
    ├── vsa_ops.json             (27 test cases, 5 benchmarks)
    └── composition_patterns.json (12 test cases, 3 integration examples)
```

## Test Summary

| Category | File | Test Cases | Invariants | Benchmarks |
|----------|------|------------|------------|------------|
| TA1 | ternary_logic.json | 11 | - | - |
| TA1 | proof_trace.json | 8 | - | - |
| TA1 | datalog_engine.json | 5 | - | - |
| TA1 | restraint.json | 4 | - | - |
| TA1 | explainability.json | 2 | - | - |
| TA1 | asp_solver.json | 3 | - | - |
| TA1 | composition.json | 4 | - | - |
| TA2 | vsa_ops.json | 27 | - | 5 |
| TA2 | composition_patterns.json | 12 | - | - |
| **TOTAL** | **9 files** | **76** | **0** | **5** |

## Format

Each test vector file contains:
- `spec_name`: Name of the specification
- `spec_file`: Path to the .t27 spec file
- `ring`: Ring number in T27 development
- `description`: Human-readable description
- `generated`: Generation date (2026-04-08)
- `phi_identity`: Trinity identity constant
- `test_cases`: Array of test cases with name, description, inputs, expected outputs
- `benchmarks` (optional): Array of performance benchmarks

## TA1 Test Vectors

### ternary_logic.json
Kleene K3 ternary logic test cases:
- Truth tables for AND, OR, NOT, IMPLIES
- No-tautology properties
- Forward chaining with K3
- Boundary conditions

### proof_trace.json
Bounded proof trace test cases:
- MAX_STEPS enforcement (10 steps)
- Modus ponens and modus tollens
- Hypothetical syllogism
- Proof trace validation

### datalog_engine.json
Forward-chaining Datalog test cases:
- Simple forward chaining
- Transitive relation chains
- Fixed point detection
- Recursive rules

### restraint.json
Bounded rationality test cases:
- MAX_STEPS enforcement
- MIN_QUALITY threshold
- Continue/stop conditions
- Quality-based early stopping

### explainability.json
XAI module test cases:
- Explanation step limit (≤10 steps)
- Valid vs invalid traces
- Step count validation

### asp_solver.json
ASP with NAF test cases:
- Simple NAF rule (B :- A, not C)
- NAF conflict resolution
- Stable model consistency
- NAF semantics validation

### composition.json
ML+AR composition pattern test cases:
- CNN_RULES pattern
- MLP_BAYESIAN pattern
- TRANSFORMER_XAI pattern
- RL_GUARDRAILS pattern

## TA2 Test Vectors

### vsa_ops.json
VSA hypervector operations test cases:
- Bind operation (with zeros, commutative, self-inverse, unbind identity)
- Bundle operations (bundle2, bundle3, idempotent, majority)
- Similarity (cosine, Hamming, bounds, symmetry)
- Dot product and Hamming distance
- Vector norm
- Permutation (shift, involutivity)
- 27-dimensional Coptic space operations
- Algebraic properties (distributivity, identity)

**Benchmarks:**
- Bind throughput: <1µs (1024-dim)
- Bundle2 throughput: <1µs (1024-dim)
- Bundle3 throughput: <1.5µs (1024-dim)
- Similarity latency: <2µs (1024-dim)
- Permute latency: <500ns (1024-dim)

### composition_patterns.json
ML+AR composition pattern test cases:
- NEURAL_SYMBOLIC_HYBRID: Neural → VSA → AR → XAI
- VSA_SEMANTIC_MEMORY: Query → Similarity → Retrieval → AR
- VSA_SEQUENCE_ENCODING: Sequence → Position encoding → Temporal reasoning
- NEURAL_VSA_ATTENTION: Attention → Bind/Unbind → Retrieval → AR
- VSA_BUNDLE_SUPERPOSITION: Concepts → Bundle → Set reasoning
- MLP_VSA_HYBRID: MLP → VSA → Rule classification
- VSA_ANALOGY: A:B :: C:? using bind/unbind
- RL_VSA_POLICY: RL → VSA → Safety constraints
- VSA_HIERARCHICAL: Nested bind → Multi-level reasoning
- NEURAL_VSA_XAI: Neural → VSA trace → Explainable AR
- COPTIC_SYMBOLIC: 27-dim Coptic space → Linguistic reasoning
- VSA_NOISE_ROBUST: Noisy input → Bundle3 → Robust reasoning

**Integration Examples:**
- Medical diagnosis composition
- Legal document analysis
- Autonomous driving decision

## Usage

Test vectors can be used to:
1. Validate implementation correctness
2. Measure conformance to CLARA requirements
3. Benchmark performance against targets
4. Verify bounded rationality constraints
5. Test explainability step limits

## Running Tests

From T27 repository root:
```bash
# Parse all test vectors
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta1/*.json
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta2/*.json

# Run full test suite (includes test vector validation)
./scripts/tri test

# Validate specific spec
./bootstrap/target/release/t27c validate-phi-identity
```

## Metadata

- Generated: 2026-04-08
- T27 Version: Ring 088+
- Total Specs: 106+
- Total Test Cases: 90+ (parse/gen) + 76 (test vectors)
- φ² + 1/φ² = 3 | TRINITY
