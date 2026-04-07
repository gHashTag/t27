# CLARA Composition Examples

This directory contains example scripts demonstrating ML+AR+VSA composition patterns for the DARPA CLARA submission.

## Examples

### 01_medical_diagnosis.py
**Pattern:** CNN → VSA Encoding → AR Reasoning → XAI Explanation

Medical diagnosis pipeline combining:
- Neural feature extraction from images
- VSA hypervector encoding for semantic memory
- Bounded AR reasoning (≤10 steps)
- Explainable output generation

**Key features:**
- MAX_STEPS = 10 enforcement
- MIN_QUALITY = 0.7 threshold
- VSA similarity search for case retrieval
- Step-by-step explanation generation

Run: `python3 01_medical_diagnosis.py`

### 02_legal_qa.py
**Pattern:** Query Encoder → VSA Similarity Search → Retrieval → AR

Legal document question answering with:
- Query encoding to ternary hypervectors
- VSA cosine similarity search over document memory
- Context extraction and fact generation
- AR reasoning with bounded steps

**Key features:**
- 1024-dim ternary hypervectors
- Pre-encoded document hypervectors
- Similarity threshold for retrieval
- Source attribution in answers

Run: `python3 02_legal_qa.py`

### 03_autonomous_driving.py
**Pattern:** RL Policy Network → VSA Encoding → Rule Engine → Guardrails

Autonomous driving safety system with:
- RL policy for action selection
- VSA encoding for state-action pairs
- Rule engine for safety constraint checking
- Guardrails for final allow/block decisions

**Key features:**
- Safety-critical system design
- Multiple safety constraints
- Emergency override
- Experience memory with VSA encoding

Run: `python3 03_autonomous_driving.py`

### 04_vsa_analogy.py
**Pattern:** Entity Encoding → VSA Bind/Unbind → Similarity Search → AR

VSA analogy reasoning demonstrating:
- Bind/Unbind for associative memory
- Self-inverse property: `bind(A, bind(A, B)) = B`
- Bundle superposition for set-like reasoning
- Permute for position-aware sequence encoding

**Key features:**
- Semantic analogies (king:man :: queen:?)
- Bundle consensus voting
- Sequence position probing
- All core VSA operations demonstrated

Run: `python3 04_vsa_analogy.py`

## VSA Operations Reference

All examples use VSA operations from `specs/vsa/ops.t27`:

| Operation | Description | Property |
|-----------|-------------|----------|
| `bind(a, b)` | XOR-like associative binding | `bind(a, bind(a, b)) = b` |
| `unbind(bound, key)` | Inverse of bind | Same as bind for XOR-like |
| `bundle2(a, b)` | Majority vote of 2 vectors | Commutative |
| `bundle3(a, b, c)` | Consensus of 3 vectors | Commutative |
| `similarity(a, b, metric)` | Similarity computation | COSINE, HAMMING, DOT |
| `permute(v, shift)` | Circular shift | Position encoding |

## Bounded Rationality

All AR operations enforce:
- **MAX_STEPS = 10** - Maximum inference steps
- **MIN_QUALITY = 0.7** - Minimum confidence threshold

## Running Examples

```bash
# Run all examples
for f in *.py; do python3 "$f"; echo; done

# Check syntax
python3 -m py_compile *.py

# Run with verbose output
python3 -v 01_medical_diagnosis.py
```

## Requirements

- Python 3.8+
- No external dependencies (pure Python for portability)

## CLARA Compliance

| Requirement | Example | Demonstrated |
|-------------|---------|--------------|
| Ternary logic | All | TRIT_NEG/TRIT_ZERO/TRIT_POS |
| Bounded proof traces | 1, 2, 3 | MAX_STEPS = 10 |
| Forward-chaining Datalog | 1, 2 | forward_chain() |
| Restraint | 1, 3 | MIN_QUALITY = 0.7 |
| Explainability | 1, 2 | generate_explanation() |
| ASP with NAF | 2 | Rule-based reasoning |
| VSA hypervector ops | 4 | bind, unbind, bundle, permute |
| Similarity | All | cosine_similarity() |
| Bundle | 4 | bundle2, bundle3 |
| ML+AR composition | 1, 2, 3 | Full pipelines |

## License

SPDX-License-Identifier: Apache-2.0

φ² + 1/φ² = 3 | TRINITY
