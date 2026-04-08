# CLARA Integration Guide — VSA + AR + ML

**Version:** 1.0
**Date:** 2026-04-07
**Status:** Draft for CLARA Submission

---

## Overview

This guide explains how to integrate Vector Symbolic Architecture (VSA), Automated Reasoning (AR), and Machine Learning (ML) components in the T27 Trinity Ternary framework. The composition patterns enable explainable, bounded reasoning with neural feature extraction.

---

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│     ML      │     │    VSA      │     │     AR      │     │    XAI      │
│  (Feature   │────▶│  (Semantic  │────▶│ (Bounded    │────▶│ (Explain    │
│  Extractor) │     │   Encoding) │     │  Reasoning) │     │  Generator) │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                     │                  │
                     ▼                  ▼
                Hypervector     MAX_STEPS=10
                (1024-dim)     MIN_QUALITY=0.7
```

---

## Component Interfaces

### 1. VSA Operations (specs/vsa/ops.t27)

```zig
// Core operations for 1024-dim ternary hypervectors
const VSA_DIM : usize = 1024;

fn bind(a: []Trit, b: []Trit, len: usize) -> []Trit;
fn unbind(bound: []Trit, key: []Trit, len: usize) -> []Trit;
fn bundle2(a: []Trit, b: []Trit, len: usize) -> []Trit;
fn bundle3(a: []Trit, b: []Trit, c: []Trit, len: usize) -> []Trit;
fn similarity(a: []Trit, b: []Trit, len: usize, metric: u8) -> f64;
fn permute(v: []Trit, len: usize, shift: usize) -> []Trit;
```

**Key Properties:**
- `bind` is XOR-like: `bind(bind(a, b), a) == b`
- `bundle2/3` uses majority voting for superposition
- `similarity` metrics: COSINE [-1,1], HAMMING [0,1], DOT
- `permute` enables position-aware sequence encoding

### 2. AR Operations (specs/ar/*)

```zig
// Kleene K3 ternary logic
const TRIT_NEG : i8 = -1;
const TRIT_ZERO : i8 = 0;
const TRIT_POS : i8 = 1;
const MAX_STEPS : usize = 10;

fn k3_and(a: i8, b: i8) -> i8;
fn k3_or(a: i8, b: i8) -> i8;
fn k3_implies(a: i8, b: i8) -> i8;
fn forward_chain(facts: []Fact, rules: []Rule, max_steps: usize) -> Conclusion[];
```

**Key Properties:**
- No classical tautologies in K3 (satisfying bounded reasoning requirement)
- Forward chaining with step limit prevents infinite chains
- NAF (Negation as Failure) for ASP-style reasoning

### 3. Restraint Module (specs/ar/restraint.t27)

```zig
const MAX_STEPS : usize = 10;
const MIN_QUALITY : f64 = 0.7;

fn should_continue(steps: usize, quality: f64) -> bool;
```

---

## Composition Patterns

### Pattern 1: Neural → VSA → AR → XAI

**Use Case:** Image classification with explainable reasoning

```python
# Pseudocode integration
def classify_with_explanation(image):
    # Step 1: Neural feature extraction
    features = cnn.extract_features(image)  # Shape: (1024,)

    # Step 2: VSA encoding (trit conversion)
    hv_trits = vsa_encoder.encode_to_trits(features)  # [-1, 0, +1] × 1024

    # Step 3: VSA similarity search for concept retrieval
    concept_hvs = load_concept_hypervectors()  # Pre-learned concepts
    similarities = [vsa.similarity(hv_trits, c, 1024, SIM_COSINE)
                    for c in concept_hvs]
    top_concept = argmax(similarities)

    # Step 4: AR forward chaining with retrieved concept
    facts = [Fact("image_has_feature", top_concept)]
    rules = load_class_rules(top_concept)
    conclusion = ar.forward_chain(facts, rules, MAX_STEPS)

    # Step 5: XAI explanation generation
    explanation = xai.generate_explanation(
        trace=conclusion.trace,
        max_steps=MAX_STEPS
    )

    return {
        "class": conclusion.class,
        "explanation": explanation,
        "quality": conclusion.quality
    }
```

### Pattern 2: VSA Semantic Memory Retrieval

**Use Case:** Question answering with context retrieval

```python
def answer_with_retrieval(question):
    # Encode query to hypervector
    query_hv = vsa_encoder.encode_query(question)  # 1024-dim

    # Similarity search over memory
    memory_hvs = load_memory_hypervectors()  # Pre-encoded documents
    results = []
    for doc_hv, doc_text in memory_hvs:
        sim = vsa.similarity(query_hv, doc_hv, 1024, SIM_COSINE)
        if sim >= 0.5:
            results.append((sim, doc_text))

    # Top-k retrieval
    results.sort(reverse=True)
    context = [r[1] for r in results[:5]]

    # AR reasoning over retrieved context
    facts = extract_facts(context)
    answer = ar.deduce(facts, MAX_STEPS)

    return answer
```

### Pattern 3: VSA Sequence Encoding for Temporal Reasoning

**Use Case:** Sequential pattern recognition

```python
def sequence_analysis(sequence):
    # Encode sequence with position-aware binding
    items = [vsa_encoder.encode_item(item) for item in sequence]
    sequence_hv = items[0]
    for i, item in enumerate(items[1:], 1):
        permuted = vsa.permute(item, 1024, i)
        sequence_hv = vsa.bundle2(sequence_hv, permuted, 1024)

    # Probe for specific item at specific position
    def probe_at_position(candidate, position):
        permuted = vsa.permute(candidate, 1024, position)
        return vsa.similarity(sequence_hv, permuted, 1024, SIM_COSINE)

    # Temporal reasoning over sequence
    pattern = ar.temporal_reasoning(sequence_hv, MAX_STEPS)

    return pattern
```

### Pattern 4: Neural-VSA Attention for Associative Memory

**Use Case:** Neural attention with associative retrieval

```python
def attention_with_bind(query, keys, values):
    # Step 1: Self-attention (Transformer)
    attn_weights = transformer.self_attention(query, keys)

    # Step 2: VSA bind for associative storage
    bound_pairs = []
    for key, value in zip(keys, values):
        key_hv = vsa_encoder.encode(key)
        value_hv = vsa_encoder.encode(value)
        bound = vsa.bind(key_hv, value_hv, 1024)
        bound_pairs.append(bound)

    # Step 3: Unbind to retrieve value
    query_hv = vsa_encoder.encode(query)
    retrieved = []
    for bound in bound_pairs:
        value_hv = vsa.unbind(bound, query_hv, 1024)
        # Find closest match in value space
        best_match = find_best_value(value_hv)
        if vsa.similarity(value_hv, best_match, 1024, SIM_COSINE) >= 0.8:
            retrieved.append(best_match)

    # Step 4: AR deduction with retrieved values
    conclusion = ar.deduce_with_retrieval(retrieved, MAX_STEPS)

    return conclusion
```

### Pattern 5: VSA Bundle Superposition for Set Reasoning

**Use Case:** Multi-concept classification

```python
def classify_concepts(concepts):
    # Encode concepts individually
    concept_hvs = [vsa_encoder.encode(c) for c in concepts]

    # Bundle for superposition
    if len(concept_hvs) == 2:
        bundle = vsa.bundle2(concept_hvs[0], concept_hvs[1], 1024)
    elif len(concept_hvs) >= 3:
        bundle = concept_hvs[0]
        for hv in concept_hvs[1:]:
            bundle = vsa.bundle3(bundle, hv, hv, 1024)
    else:
        bundle = concept_hvs[0] if concept_hvs else [0] * 1024

    # Probe for individual concepts
    recovered = {}
    for i, concept in enumerate(concepts):
        sim = vsa.similarity(bundle, concept_hvs[i], 1024, SIM_COSINE)
        if sim >= 0.6:
            recovered[concept] = sim

    # AR set reasoning
    classification = ar.set_reasoning(recovered, MAX_STEPS)

    return classification
```

---

## Data Flow Specifications

### Trit Representation

| Value | Symbol | Code |
|-------|--------|------|
| -1    | NEG    | TRIT_NEG |
| 0     | ZERO   | TRIT_ZERO |
| +1    | POS    | TRIT_POS |

### Hypervector Encoding

```python
# Continuous → Trit conversion
def to_trits(vector, dim=1024):
    """Convert float vector to ternary hypervector"""
    trits = []
    for v in vector:
        if v > 0.33:
            trits.append(TRIT_POS)
        elif v < -0.33:
            trits.append(TRIT_NEG)
        else:
            trits.append(TRIT_ZERO)
    return trits

# Trit → Continuous (for compatibility)
def from_trits(trits):
    """Convert ternary hypervector to float vector"""
    return [float(t) for t in trits]
```

---

## Step Limits and Bounded Rationality

### MAX_STEPS = 10

All AR operations enforce a maximum of 10 inference steps:
- Prevents infinite forward-chaining
- Ensures explanations are ≤10 steps (XAI requirement)
- Provides bounded rationality guarantee

### MIN_QUALITY = 0.7

Reasoning continues only while quality >= 0.7:
- Early stopping on low confidence
- Prevents propagation of uncertain inferences
- Trade-off between completeness and reliability

### Restraint Check

```python
def should_continue(steps: int, quality: float) -> bool:
    return steps < MAX_STEPS and quality >= MIN_QUALITY
```

---

## Example: End-to-End Pipeline

### Medical Diagnosis

```python
class MedicalDiagnosisSystem:
    def __init__(self):
        self.cnn = load_model("medical_cnn.zig")
        self.vsa_encoder = VSAEncoder(dim=1024)
        self.ar_engine = AREngine(max_steps=10)
        self.xai = XAIModule()

    def diagnose(self, medical_image):
        # Step 1: CNN feature extraction
        features = self.cnn.extract_features(medical_image)

        # Step 2: VSA encoding
        hv = self.vsa_encoder.encode_to_trits(features)

        # Step 3: Retrieve similar cases from memory
        cases = self.retrieve_similar_cases(hv, top_k=5)

        # Step 4: AR reasoning
        facts = [Fact("image_features", hv)]
        for case in cases:
            facts.extend(case.facts)

        conclusion = self.ar_engine.deduce(facts)

        # Step 5: Generate explanation
        explanation = self.xai.generate_explanation(
            trace=conclusion.trace,
            max_steps=10
        )

        return {
            "diagnosis": conclusion.class,
            "confidence": conclusion.quality,
            "explanation": explanation,
            "similar_cases": cases
        }

    def retrieve_similar_cases(self, query_hv, top_k=5):
        """VSA similarity search over case database"""
        case_hvs = load_case_hypervectors()
        results = []
        for case_id, case_hv, case_data in case_hvs:
            sim = vsa.similarity(query_hv, case_hv, 1024, SIM_COSINE)
            if sim >= 0.6:
                results.append((sim, case_id, case_data))

        results.sort(reverse=True)
        return [r[2] for r in results[:top_k]]
```

---

## Performance Benchmarks

Target performance for 1024-dim hypervectors:

| Operation | Target | Notes |
|-----------|--------|-------|
| bind | <1µs | XOR-like operation |
| bundle2 | <1µs | Majority vote |
| bundle3 | <1.5µs | Consensus of 3 |
| similarity (cosine) | <2µs | With norm computation |
| permute | <500ns | Circular shift |

---

## Testing and Validation

### Unit Tests

```bash
# Run all VSA tests
./scripts/tri test vsa

# Run AR tests
./scripts/tri test ar

# Run composition tests
./scripts/tri test composition
```

### Integration Tests

```bash
# Run test vectors
python3 -m pytest docs/clara/test_vectors/
```

### Validation

```bash
# Validate phi identity
./scripts/tri validate-phi

# Validate conformance
./bootstrap/target/release/t27c validate-conformance
```

---

## References

- **VSA Spec:** `specs/vsa/ops.t27`
- **AR Spec:** `specs/ar/*.t27`
- **Test Vectors:** `docs/clara/test_vectors/`
- **Conformance:** `docs/clara/CLARA_TA1_CONFORMANCE.json`
- **Technical Narrative:** `docs/clara/CLARA_TECHNICAL_NARRATIVE.md`

---

φ² + 1/φ² = 3 | TRINITY
