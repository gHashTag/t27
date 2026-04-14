<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# CLARA Evidence Package

Technical evidence supporting the DARPA CLARA PA-25-07-02 submission.

---

## Package Contents

### Core Evidence Documents

| Document | Location | Description |
|----------|----------|-------------|
| Evidence Package | [`../CLARA-EVIDENCE-PACKAGE.md`](../CLARA-EVIDENCE-PACKAGE.md) | Consolidated evidence index |
| SOA Comparison | [`../CLARA-SOA-COMPARISON.md`](../CLARA-SOA-COMPARISON.md) | vs DeepProbLog, REASON, Tensor Logic |
| Literature Review | [`../CLARA-LITERATURE-REVIEW.md`](../CLARA-LITERATURE-REVIEW.md) | 2020-2026 neuro-symbolic survey |
| Scaling Analysis | [`../CLARA-SCALING.md`](../CLARA-SCALING.md) | Performance scaling analysis |
| Red Team Protocol | [`../CLARA-RED-TEAM.md`](../CLARA-RED-TEAM.md) | Adversarial testing protocol |

### Direct Evidence Files (Main Directory)

The main evidence files are located in the parent directory:
- `../CLARA-EVIDENCE-PACKAGE.md` — Master evidence document
- `../CLARA-SOA-COMPARISON.md` — State-of-the-art comparison
- `../CLARA-LITERATURE-REVIEW.md` — Literature survey
- `../CLARA-SCALING.md` — Scaling characteristics
- `../CLARA-RED-TEAM.md` — Red team methodology

---

## Subdirectories

### `benchmarks/`
Performance benchmarks and test results.
- GF16 numeric standard benchmark results
- VSA hypervector operation benchmarks
- Energy efficiency measurements

### `conformance/`
TA1/TA2 conformance test results.
- TA1 conformance validation
- TA2 conformance validation
- Test coverage reports

### `coq/`
Coq formal verification files.
- Theorem definitions
- Proof scripts
- Verification outputs

### `specs/`
.t27 specification files.
- COA planning spec
- VSA operations spec
- Composition pattern specs

---

## Evidence Categories

### Mathematical Foundations
- φ (phi) identities verification
- Sacred constants standard
- GF16 numeric standard

### AR Engine
- Ternary logic (Kleene K3)
- Proof trace constraints (MAX_STEPS=10)
- Forward-chaining Datalog
- ASP with NAF

### ML Components
- Neural network integration
- Bayesian inference
- Reinforcement learning

### Composition
- ML+AR composition patterns
- VSA hypervector operations
- Guardrails and safety

### Verification
- Coq theorem proofs (84 theorems)
- Conformance test results
- Adversarial robustness validation

---

## Running Evidence Tests

```bash
# Run Coq proofs
cd proofs/trinity
make

# Run conformance tests
./scripts/tri test

# Run benchmarks
./bootstrap/target/release/t27c bench specs/clara/
```

---

## References

Key papers and standards referenced in evidence package:
- Kleene K3 ternary logic
- ASP with NAF semantics
- VSA hypervector operations
- DARPA XAI program results
- DoD AI ethics guidelines

---

## License

SPDX-License-Identifier: Apache-2.0

---

**φ² + 1/φ² = 3 | TRINITY**
