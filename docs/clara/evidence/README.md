<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# CLARA Evidence Package

**Supporting evidence for DARPA PA-25-07-02 Technical Proposal**

---

## 📁 Directory Structure

```
evidence/
├── specs/                    # t27 specifications
├── coq/                      # Coq verification proofs
├── conformance/              # CLARA requirements mapping (JSON)
├── benchmarks/               # Performance benchmarks and test vectors
└── *.md                      # Supporting documentation
```

---

## 📋 Evidence Files

### Core Evidence Documents

| File | Description |
|------|-------------|
| `CLARA-EVIDENCE-PACKAGE.md` | Consolidated evidence package (main document) |
| `CLARA-SOA-COMPARISON.md` | State-of-the-art benchmark (7 systems) |
| `CLARA-LITERATURE-REVIEW.md` | Post-2020 neuro-symbolic AI survey |
| `CLARA-SCALING.md` | Performance scaling analysis |
| `CLARA-RED-TEAM.md` | Adversarial testing protocol |

---

## 📐 Specifications (`specs/`)

| Spec | Description | Lines |
|------|-------------|-------|
| `ar/coa_planning.t27` | Course-of-Action planning (defense domain) | 522 |

Location: `../../specs/ar/coa_planning.t27`

---

## 🧮 Coq Proofs (`coq/`)

| File | Theorems | Status |
|------|----------|--------|
| `CorePhi.v` | φ identities | ✅ Verified |
| `AlphaPhi.v` | Alpha sequences | ✅ Verified |
| `FormulaEval.v` | Formula evaluation | ✅ Verified |
| `ExactIdentities.v` | Exact phi identities | ✅ Verified |
| `ConsistencyChecks.v` | Consistency validation | ✅ Verified |
| `Unitarity.v` | Unitarity conditions | ✅ Verified |
| `Bounds_*.v` | Physics bounds | ✅ Verified |

Location: `../../proofs/trinity/`

**Total:** 84 Coq theorems (math core verification)

---

## 📊 Conformance (`conformance/`)

| File | Purpose |
|------|---------|
| `CLARA_TA1_CONFORMANCE.json` | TA1 requirements mapping |
| `CLARA_TA2_CONFORMANCE.json` | TA2 requirements mapping |

Location: `../` (parent directory)

---

## 📈 Benchmarks (`benchmarks/`)

| Artifact | Description |
|----------|-------------|
| GF16 benchmark results | GoldenFloat16 validation |
| Test vectors | Numerical test data |

Location: `../../conformance/FORMAT-SPEC-001.json`

---

## 🔗 Quick Links

- [Main Proposal](../CLARA-PROPOSAL-TECHNICAL.md)
- [Submission Package](../submission/README.md)
- [Coq Proofs](../../proofs/trinity/)
- [Specifications](../../specs/)

---

**φ² + 1/φ² = 3 | TRINITY**
