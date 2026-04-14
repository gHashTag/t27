<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# CLARA Pack — DARPA PA-25-07-02 Submission

**Proposal Reference:** CLARA-PA25-07-02-TRINITY
**Submission Date:** April 17, 2026
**Repository:** https://github.com/gHashTag/t27

---

## Quick Reference

| Section | Location | Description |
|---------|----------|-------------|
| **Submission Package** | [`submission/`](submission/) | Volume 1/2 proposals + deliverables |
| **Evidence Package** | [`evidence/`](evidence/) | Technical evidence & benchmarks |
| **Examples** | [`examples/`](examples/) | ML+AR+VSA composition demos |
| **Test Vectors** | [`test_vectors/`](test_vectors/) | TA1/TA2 conformance tests |
| **Technical Proposal** | [`CLARA-PROPOSAL-TECHNICAL.md`](CLARA-PROPOSAL-TECHNICAL.md) | Main proposal (~10 pages) |
| **Cost Proposal** | [`CLARA-COST-PROPOSAL.md`](CLARA-COST-PROPOSAL.md) | Budget ($2.0M) |
| **SOA Comparison** | [`CLARA-SOA-COMPARISON.md`](CLARA-SOA-COMPARISON.md) | vs DeepProbLog, REASON, etc. |

---

## Directory Structure

```
docs/clara/
├── README.md                           (this file — index)
├── CLARA-PROPOSAL-TECHNICAL.md         (main proposal)
├── CLARA-COST-PROPOSAL.md              (budget breakdown)
├── CLARA-SOA-COMPARISON.md             (state-of-the-art)
├── CLARA-LITERATURE-REVIEW.md          (2020-2026 survey)
├── CLARA-COMPOSITION-PATTERNS.md       (ML+AR patterns)
├── CLARA-EVIDENCE-PACKAGE.md           (evidence index)
├── CLARA-RED-TEAM.md                   (adversarial testing)
├── CLARA-SCALING.md                    (performance scaling)
│
├── submission/                         (DARPA submission package)
│   ├── README.md
│   ├── SUBMISSION_REPORT.md
│   └── SUBMISSION-FINAL-REPORT.md
│
├── evidence/                           (technical evidence)
│   ├── README.md
│   ├── benchmarks/                    (GF16, VSA benchmarks)
│   ├── conformance/                   (TA1/TA2 conformance)
│   ├── coq/                          (Coq verification files)
│   └── specs/                        (.t27 specs)
│
├── examples/                          (ML+AR+VSA demos)
│   ├── README.md
│   ├── 01_medical_diagnosis.py
│   ├── 02_legal_qa.py
│   ├── 03_autonomous_driving.py
│   ├── 04_vsa_analogy.py
│   └── coa-planning.md
│
└── test_vectors/                       (TA1/TA2 test vectors)
    ├── README.md
    ├── ta1/                          (37 test cases)
    │   ├── ternary_logic.json
    │   ├── proof_trace.json
    │   ├── datalog_engine.json
    │   ├── restraint.json
    │   ├── explainability.json
    │   ├── asp_solver.json
    │   └── composition.json
    └── ta2/                          (39 test cases, 5 benchmarks)
        ├── vsa_ops.json
        └── composition_patterns.json
```

---

## Submission Package ([`submission/`](submission/))

### Volume 1: Technical & Management Proposal
- **Main Proposal:** ~1,700 words, ≤10 pages
  - AR-Based ML Approach (Trit-K3 isomorphism)
  - Application Task Domain (COA Planning)
  - Polynomial-Time Tractability Proofs (5 theorems)
  - Demonstrated AR+ML Composition
  - Metrics Coverage
  - 24-month schedule + milestones
  - Budget Summary ($2.0M)

### Volume 2: Cost Proposal
- Personnel: $1.2M (60%)
- Equipment: $200K (10%)
- Travel: $100K (5%)
- Indirect: $500K (25%)

**See:** [`submission/README.md`](submission/README.md) for complete details

---

## Evidence Package ([`evidence/`](evidence/))

### Technical Evidence
- [`CLARA-EVIDENCE-PACKAGE.md`](CLARA-EVIDENCE-PACKAGE.md) — Consolidated evidence
- [`CLARA-SOA-COMPARISON.md`](CLARA-SOA-COMPARISON.md) — vs SOA systems
- [`CLARA-LITERATURE-REVIEW.md`](CLARA-LITERATURE-REVIEW.md) — Neuro-symbolic survey
- [`CLARA-SCALING.md`](CLARA-SCALING.md) — Performance scaling
- [`CLARA-RED-TEAM.md`](CLARA-RED-TEAM.md) — Adversarial testing

### Benchmarks
- GF16 numeric standard results
- VSA hypervector benchmarks

### Verification
- Coq theorem proofs (84 theorems verified)
- Conformance test results

**See:** [`evidence/README.md`](evidence/README.md) for complete details

---

## Examples ([`examples/`](examples/))

### Composition Patterns

| Example | Pattern | Domain | Run |
|---------|---------|--------|-----|
| `01_medical_diagnosis.py` | CNN→VSA→AR→XAI | Medical | `python3 01_medical_diagnosis.py` |
| `02_legal_qa.py` | Query→VSA→Retrieval→AR | Legal | `python3 02_legal_qa.py` |
| `03_autonomous_driving.py` | RL→VSA→Rules→Guardrails | Autonomous | `python3 03_autonomous_driving.py` |
| `04_vsa_analogy.py` | VSA Bind/Unbind→Similarity | Analogical | `python3 04_vsa_analogy.py` |
| `coa-planning.md` | COA Planning spec | Defense | — |

### Key Features
- **MAX_STEPS = 10** — Bounded proof traces
- **MIN_QUALITY = 0.7** — Quality threshold
- **VSA Operations** — bind, unbind, bundle, similarity
- **XAI Generation** — Step-by-step explanations

**See:** [`examples/README.md`](examples/README.md) for complete details

---

## Test Vectors ([`test_vectors/`](test_vectors/))

### TA1 Test Vectors (37 test cases)

| File | Test Cases | Description |
|------|------------|-------------|
| `ternary_logic.json` | 11 | Kleene K3 truth tables, forward chaining |
| `proof_trace.json` | 8 | MAX_STEPS enforcement, modus ponens/tollens |
| `datalog_engine.json` | 5 | Forward-chaining, fixed point, recursion |
| `restraint.json` | 4 | Bounded rationality, MIN_QUALITY |
| `explainability.json` | 2 | ≤10 step limit, trace validation |
| `asp_solver.json` | 3 | ASP with NAF, stable models |
| `composition.json` | 4 | ML+AR composition patterns |

### TA2 Test Vectors (39 test cases, 5 benchmarks)

| File | Test Cases | Benchmarks | Description |
|------|------------|------------|-------------|
| `vsa_ops.json` | 27 | 5 | Bind, bundle, similarity, permute, algebraic properties |
| `composition_patterns.json` | 12 | 0 | 12 ML+AR composition patterns |

**See:** [`test_vectors/README.md`](test_vectors/README.md) for complete details

---

## Running Tests

```bash
# Parse test vectors
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta1/*.json
./bootstrap/target/release/t27c parse docs/clara/test_vectors/ta2/*.json

# Run examples
cd docs/clara/examples
for f in *.py; do python3 "$f"; done

# Run full test suite
./scripts/tri test
```

---

## CLARA Requirements Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| AR in guts of ML (FAQ 21) | ✅ | K3 logic gates replace ReLU |
| ≤10 step proof traces | ✅ | MAX_STEPS=10 |
| Polynomial guarantees | ✅ | Theorems 1-5 |
| ≥2 AR kinds | ✅ | Logic, ASP, Classical (3 kinds) |
| ≥2 ML kinds | ✅ | Neural, Bayesian, RL (3 kinds) |
| Apache 2.0 | ✅ | All file headers |
| Restraint | ✅ | K_UNKNOWN = bounded rationality |
| Explainability | ✅ | ≤10 step traces, XAI formats |

---

## Key Documents

| Document | Purpose | Location |
|----------|---------|----------|
| Technical Proposal | Main proposal document | [`CLARA-PROPOSAL-TECHNICAL.md`](CLARA-PROPOSAL-TECHNICAL.md) |
| Cost Proposal | Budget breakdown | [`CLARA-COST-PROPOSAL.md`](CLARA-COST-PROPOSAL.md) |
| SOA Comparison | Competitive analysis | [`CLARA-SOA-COMPARISON.md`](CLARA-SOA-COMPARISON.md) |
| Literature Review | Research survey | [`CLARA-LITERATURE-REVIEW.md`](CLARA-LITERATURE-REVIEW.md) |
| Composition Patterns | ML+AR patterns catalog | [`CLARA-COMPOSITION-PATTERNS.md`](CLARA-COMPOSITION-PATTERNS.md) |
| Evidence Package | All technical evidence | [`CLARA-EVIDENCE-PACKAGE.md`](CLARA-EVIDENCE-PACKAGE.md) |
| Red Team Protocol | Adversarial testing | [`CLARA-RED-TEAM.md`](CLARA-RED-TEAM.md) |
| Improvements Summary | v1.5 changes | [`CLARA-IMPROVEMENTS-SUMMARY.md`](CLARA-IMPROVEMENTS-SUMMARY.md) |

---

## Statistics

| Metric | Value |
|--------|-------|
| Total Specs | 106+ |
| Total Test Cases (parse/gen) | 90+ |
| Test Vector Cases | 76 |
| Coq Theorems Verified | 84 |
| Python Examples | 4 |
| Composition Patterns | 12 |

---

## License

SPDX-License-Identifier: Apache-2.0

---

**φ² + 1/φ² = 3 | TRINITY**
