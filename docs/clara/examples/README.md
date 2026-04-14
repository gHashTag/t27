<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# CLARA Examples

**Demonstrations and example applications for DARPA CLARA proposal**

---

## 📁 Directory Structure

```
examples/
├── coa-planning.md          # Defense COA planning (t27 spec example)
├── 01_medical_diagnosis.py   # Medical diagnosis (CNN → VSA → AR)
├── 02_legal_qa.py            # Legal QA (Query → VSA → Retrieval → AR)
├── 03_autonomous_driving.py   # Autonomous driving (RL → VSA → Rules → Guardrails)
├── 04_vsa_analogy.py         # VSA analogy reasoning demonstration
└── README.md                  # This file
```

---

## 📄 Example Files

### Defense Domain Example

#### coa-planning.md

**Purpose:** Demonstrates TRINITY's AR capabilities in defense domain (Course-of-Action planning)

**Content:**
- COA planning scenario (military logistics)
- Constraints: fuel, crew, weather, resources, timeline
- TRINITY specification mapping
- Expected decision outputs

**Spec Location:** `../../specs/ar/coa_planning.t27` (522 lines)

**Key Features:**
- MAX_CLAUSES=256 (bounded rationality)
- MAX_STEPS=10 (explainability)
- AR guardrails for safety

---

### Python Composition Examples

#### 01_medical_diagnosis.py

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

---

#### 02_legal_qa.py

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

---

#### 03_autonomous_driving.py

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

---

#### 04_vsa_analogy.py

**Pattern:** Entity Encoding → VSA Bind/Unbind → Similarity Search → AR

VSA analogy reasoning demonstrating:
- Bind/Unbind for associative memory
- Self-inverse property: `bind(A, bind(A, B)) = B`
- Bundle superposition for set-like reasoning
- Permute for position-aware sequence encoding

**Key features:**
- Semantic analogies (king:man :: queen:woman?)
- Bundle consensus voting
- Sequence position probing
- All core VSA operations demonstrated

---

## 🧮 VSA Operations Reference

All examples use VSA operations from `../../specs/vsa/ops.t27`:

| Operation | Description | Property |
|-----------|-------------|----------|
| `bind(a, b)` | XOR-like associative binding | `bind(a, bind(a, b)) = b` |
| `unbind(bound, key)` | Inverse of bind | Same as bind for XOR-like |
| `bundle2(a, b)` | Majority vote of 2 vectors | Commutative |
| `bundle3(a, b, c)` | Consensus of 3 vectors | Commutative |
| `similarity(a, b, metric)` | Similarity computation | COSINE, HAMMING, DOT |
| `permute(v, shift)` | Circular shift | Position encoding |

---

## 🔒 Bounded Rationality

All AR operations enforce:
- **MAX_STEPS = 10** — Maximum inference steps
- **MIN_QUALITY = 0.7** — Minimum confidence threshold

---

## 🚀 Running Examples

```bash
# Run all Python examples
cd docs/clara/examples
for f in *.py; do python3 "$f"; echo; done

# Check syntax
python3 -m py_compile *.py

# Run with verbose output
python3 -v 01_medical_diagnosis.py

# View COA planning example
cat docs/clara/examples/coa-planning.md
```

---

## ✅ CLARA Requirements Compliance

| Requirement | Example | Demonstrated |
|-------------|---------|--------------|
| Ternary logic | All | TRIT_NEG/TRIT_ZERO/TRIT_POS |
| Bounded proof traces | 1, 2, 3, 4 | MAX_STEPS = 10 |
| Forward-chaining Datalog | 1, 2 | forward_chain() |
| Restraint | 1, 2 | MIN_QUALITY = 0.7 |
| Explainability | 1, 2 | generate_explanation() |
| ASP with NAF | 2 | Rule-based reasoning |
| VSA hypervector ops | 4 | bind, unbind, bundle, permute |
| Similarity | 4 | cosine_similarity() |
| Bundle | 4 | bundle2, bundle3 |
| ML+AR composition | 1, 2, 3, 4 | Full pipelines |

---

## 🔗 Related Documentation

- [Technical Proposal](../CLARA-PROPOSAL-TECHNICAL.md)
- [Evidence Package](../evidence/README.md)
- [T27 Specifications](../../specs/)

---

## 📝 Requirements

- Python 3.8+
- No external dependencies (pure Python for portability)

---

**φ² + 1/φ² = 3 | TRINITY**
