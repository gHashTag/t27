<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# CLARA Test Vectors

**Comprehensive test suite for DARPA CLARA TA1 and TA2 requirements validation**

---

## 📁 Directory Structure

```
test_vectors/
├── ta1/                     # TA1 (AR capabilities) test vectors
├── ta2/                     # TA2 (ML+AR composition) test vectors
└── README.md                 # This file
```

---

## 📊 Test Summary

| Category | Files | Test Cases | Benchmarks |
|----------|--------|------------|------------|
| **TA1** | 6 | 37 | - |
| **TA2** | 2 | 39 | 5 |
| **TOTAL** | **8** | **76** | **5** |

---

## 🧪 Test Vector Format

Each test vector file contains:
```json
{
  "spec_name": "Name of the specification",
  "spec_file": "Path to .t27 spec file",
  "ring": "Ring number in T27 development",
  "description": "Human-readable description",
  "generated": "Generation date (2026-04-08)",
  "phi_identity": "Trinity identity constant",
  "test_cases": [
    {
      "name": "Test case name",
      "description": "Test description",
      "inputs": { ... },
      "expected_outputs": { ... }
    }
  ],
  "benchmarks": [
    {
      "name": "Benchmark name",
      "metric": "Performance metric",
      "target": "Required performance"
    }
  ]
}
```

---

## 📋 TA1 Test Vectors (AR Capabilities)

### 1. ternary_logic.json

**Kleene K3 ternary logic test cases:**
- Truth tables for AND, OR, NOT, IMPLIES
- No-tautology properties
- Forward chaining with K3
- Boundary conditions

**Test Cases:** 11

### 2. proof_trace.json

**Bounded proof trace test cases:**
- MAX_STEPS enforcement (10 steps)
- Modus ponens and modus tollens
- Hypothetical syllogism
- Proof trace validation

**Test Cases:** 8

### 3. datalog_engine.json

**Forward-chaining Datalog test cases:**
- Simple forward chaining
- Transitive relation chains
- Fixed point detection
- Recursive rules

**Test Cases:** 5

### 4. restraint.json

**Bounded rationality test cases:**
- MAX_STEPS enforcement
- MIN_QUALITY threshold
- Continue/stop conditions
- Quality-based early stopping

**Test Cases:** 4

### 5. explainability.json

**XAI module test cases:**
- Explanation step limit (≤10 steps)
- Valid vs invalid traces
- Step count validation
- Explanation format

**Test Cases:** 2

### 6. asp_solver.json

**ASP with NAF test cases:**
- Simple NAF rule: `B :- A, not C`
- NAF conflict resolution
- Stable model consistency
- NAF semantics validation

**Test Cases:** 3

---

## 🧪 TA2 Test Vectors (ML+AR Composition)

### 1. vsa_ops.json (27 test cases, 5 benchmarks)

**VSA hypervector operations test cases:**

| Operation | Property |
|-----------|----------|
| `bind(a, b)` | XOR-like associative binding, `bind(a, bind(a, b)) = b` |
| `unbind(bound, key)` | Inverse of bind, same as bind for XOR-like |
| `bundle2(a, b)` | Majority vote of 2 vectors, commutative |
| `bundle3(a, b, c)` | Consensus of 3 vectors, commutative |
| `similarity(a, b, metric)` | Cosine, Hamming, Dot product |
| `permute(v, shift)` | Circular shift for position encoding |

**Benchmarks:**
- Bind throughput: <1µs (1024-dim)
- Bundle2 throughput: <1µs (1024-dim)
- Bundle3 throughput: <1.5µs (1024-dim)
- Similarity latency: <2µs (1024-dim)
- Permute latency: <500ns (1024-dim)

### 2. composition_patterns.json (12 test cases, 3 integration examples)

**ML+AR composition pattern test cases:**

| Pattern | Description |
|---------|-------------|
| `NEURAL_SYMBOLIC_HYBRID` | CNN → VSA → AR → XAI |
| `VSA_SEMANTIC_MEMORY` | Query → Similarity → Retrieval → AR |
| `VSA_SEQUENCE_ENCODING` | Sequence → Position → Temporal → AR |
| `NEURAL_VSA_ATTENTION` | Attention → Bind/Unbind → Retrieval → AR |
| `VSA_BUNDLE_SUPERPOSITION` | Concepts → Bundle → Set reasoning |
| `MLP_VSA_HYBRID` | MLP → VSA → Rule classification |
| `VSA_ANALOGY` | A:B :: C:? using bind/unbind |
| `RL_VSA_POLICY` | RL → VSA → Safety constraints |
| `VSA_HIERARCHICAL` | Nested bind → Multi-level reasoning |
| `NEURAL_VSA_XAI` | Neural → VSA trace → Explainable AR |
| `COPTIC_SYMBOLIC` | 27-dim → Linguistic reasoning |
| `VSA_NOISE_ROBUST` | Noisy → Bundle3 → Robust reasoning |

**Integration Examples:**
- Medical diagnosis composition
- Legal document analysis
- Autonomous driving decision

---

## 🚀 Running Tests

### Parse Test Vectors

```bash
# From T27 repository root
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta1/*.json
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta2/*.json
```

### Run Full Test Suite

```bash
# Run all tests (includes test vector validation)
./scripts/tri test

# Validate specific spec
./bootstrap/target/release/t27c validate-phi-identity
```

---

## ✅ CLARA Requirements Coverage

| Requirement | Coverage |
|-------------|------------|
| Ternary logic | ✅ ternary_logic.json |
| Bounded proof traces | ✅ proof_trace.json |
| Forward-chaining Datalog | ✅ datalog_engine.json |
| Restraint | ✅ restraint.json |
| Explainability | ✅ explainability.json |
| ASP with NAF | ✅ asp_solver.json |
| VSA hypervector ops | ✅ vsa_ops.json |
| ML+AR composition | ✅ composition_patterns.json |

---

## 🔗 Related Documentation

- [Technical Proposal](../CLARA-PROPOSAL-TECHNICAL.md)
- [Examples](../examples/README.md)
- [Evidence Package](../evidence/README.md)
- [Submission Package](../submission/README.md)

---

## 📝 Metadata

| Property | Value |
|----------|-------|
| Generated | April 8, 2026 |
| T27 Version | Ring 088+ |
| Total Specs | 106+ |
| φ² + 1/φ² = 3 | TRINITY |

---

**φ² + 1/φ² = 3 | TRINITY**
