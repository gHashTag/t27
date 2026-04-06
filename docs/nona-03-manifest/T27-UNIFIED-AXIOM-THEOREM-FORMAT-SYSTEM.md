# Unified axiom, theorem, and digital-format system — Trinity / t27

**Status:** Normative charter (planning + partial artifacts)  
**Date:** 2026-04-06  
**Language:** English (repository **LANG-EN**)  

**Repositories:** This document primarily governs **[gHashTag/t27](https://github.com/gHashTag/t27)**. Sibling **[gHashTag/trinity](https://github.com/gHashTag/trinity)** holds runtime and publications; **cross-repo deduplication** is an explicit goal of this charter.

**Law:** **`SOUL.md`**, **`docs/nona-03-manifest/TDD-CONTRACT.md`**, **`docs/nona-03-manifest/ISSUE-GATE-001.md`**, **Article SSOT-MATH** in **`docs/T27-CONSTITUTION.md`**.  

**Companions:** **`docs/nona-02-organism/NUMERIC-STANDARD-001.md`**, **`conformance/FORMAT-SPEC-001.json`**, **`schemas/numeric-format-v1.json`**, **`conformance/axiom_system.json`**, **`docs/nona-03-manifest/CLAIM_TIERS.md`**, **`docs/nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md`**, **`docs/nona-03-manifest/RESEARCH_CLAIMS.md`**.

---

## Part I — Current base (t27 inventory)

### I.1 Theorems and exact / algebraic material (representative)

| ID | Statement | Status | Source in / near t27 |
|----|-----------|--------|----------------------|
| **THM-001** | φ² + φ⁻² = 3 (TRINITY identity) | Exact algebraic | `specs/math/constants.t27`; publications `trinity-sacred-mathematics.tex` (sibling / Zenodo) |
| **THM-002** | φⁿ + (−1)ⁿ φ⁻ⁿ = Lₙ (Lucas) | Exact | ibid. |
| **THM-003** | Chebyshev link for TRINITY identity | Exact | ibid. |
| **THM-004** | Ternary radix optimality (r / ln r) | Exact | ibid. |
| **THM-005** | E8 affine Cartan spectrum contains φ-related scales | Exact (computational verification) | `specs/math/e8_lie_algebra.t27` |
| **THM-006** | Zamolodchikov mass ratio m₂/m₁ = φ (E8) | Exact (theorem + experiment) | `specs/math/zamolodchikov_e8.t27` |
| **THM-007–008** | VSA monoid / majority bundle | Exact | publications + `specs/` VSA modules |
| **THM-009** | exp/mant = 1/φ maximizes exp·mant at fixed bit budget | Exact | `specs/numeric/phi_ratio.t27` |
| **THM-010** | GF16 vs BF16 sacred-constant accuracy (~1.8× scenario) | **Empirical benchmark** | **`docs/nona-02-organism/NUMERIC-STANDARD-001.md`** (BENCH-005) |

### I.2 Physics approximations (empirical / falsifiable — not derivations)

| ID | Sketch | Claim tier | Pointer |
|----|--------|------------|---------|
| **PHY-001–004** | Sacred physics ansätze vs CODATA | `empirical_fit` | **`docs/nona-02-organism/SACRED-PHYSICS-001.md`** |
| **PHY-005** | γ = φ⁻³ as **exact** BI parameter | **`falsified_as_exact`** | Documented mismatch; keep visible |
| **PHY-006** | Multi-constant ansatz class | `empirical_fit` (+ overfitting caution) | publications / `trinity-sacred-mathematics.tex` |

**Critical gap (this charter fixes structurally):** machine-readable **catalog** with **`claim_tier`**, single **numeric format descriptor** for all languages, and one **entrypoint** for math/physics verification (aligned with **`T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md`**).

---

## Part II — Architecture

### II.1 Four layers of statements

```text
TIER-0  Mathematical axioms (definitions; unfalsifiable as “physics”)
TIER-1  Proven theorems (machine-checkable where possible)
TIER-2  Empirical hypotheses (fits, benchmarks — honest error)
TIER-3  Open conjectures (registered, falsifiable windows)
```

**Policy doc:** **`docs/nona-03-manifest/CLAIM_TIERS.md`**.

### II.2 Single digital format descriptor (cross-language)

**Problem:** **`NUMERIC-STANDARD-001.md`** is human-first; Rust / Zig / Python bindings can drift.

**Solution (landed as v1.0 seed):**

| Artifact | Role |
|----------|------|
| **`conformance/FORMAT-SPEC-001.json`** | Machine-readable GoldenFloat family; **must match** **`NUMERIC-STANDARD-001.md`** |
| **`schemas/numeric-format-v1.json`** | JSON Schema for format descriptors |

**Validation (local):**

```bash
python3 -c "
import json, jsonschema
from pathlib import Path
root = Path('.')
schema = json.loads((root / 'schemas/numeric-format-v1.json').read_text())
inst = json.loads((root / 'conformance/FORMAT-SPEC-001.json').read_text())
jsonschema.validate(instance=inst, schema=schema)
print('FORMAT-SPEC-001.json: OK')
"
```

**Target (not implemented yet):** `tri gen --from-format-spec` (or codegen from JSON) — track under **Ring 012-class** issues; **do not** hand-edit `gen/*` for format fields.

### II.3 Axiom catalog (machine-readable seed)

**`conformance/axiom_system.json`** — **`status: draft_seed`**. Expand with every closed epic; keep in sync with **`RESEARCH_CLAIMS.md`** and specs.

---

## Part III — Implementation epics (issues to file)

**Rules:** No code without **GitHub issue**; PRs **Closes #N** (**issue-gate**). Commits SHOULD reference issues (project convention). **PHI LOOP:** edit spec → seal → gen → test → verdict → experience → commit.

### EPIC-AX — Unified axiom / theorem catalog

| Issue | Deliverable | Acceptance (target) |
|-------|-------------|---------------------|
| **AX-001** | `specs/base/axioms.t27` (or equivalent module) | Parses; each axiom has `tier`, `statement`, `dependencies` |
| **AX-002** | `specs/base/theorems.t27` | THM-001…009 covered with **test**/**invariant** per **TDD-CONTRACT** |
| **AX-003** | **`docs/nona-03-manifest/CLAIM_TIERS.md`** | **Done** (this file); amend via PR only |

### EPIC-NF — Universal numeric format

| Issue | Deliverable | Acceptance |
|-------|-------------|------------|
| **NF-001** | **`conformance/FORMAT-SPEC-001.json`** + schema | **Done** (seed); `jsonschema` validates |
| **NF-002** | `specs/numeric/format_contract.t27` | Contract + invariants; CI roundtrip when `tri test` exists |

### EPIC-TX — Math / physics test harness

Align with **`T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md`** (Rings 050–054): `specs/stdlib/math_test.t27`, **`tri math-verify`**, expanded **`conformance/axiom_system.json`**.

### EPIC-BR — Brain / CLARA as axiom executor

Proof-chain JSON for “prove TRINITY identity”; honest **`FALSIFIED_AS_EXACT`** for PHY-005 class queries — track under brain charter + CLARA issues.

### EPIC-EX — Experience logs

Template per loop under **`.trinity/experience/`** (or successor); reference closed issues and mutations to **`axiom_system.json`**.

---

## Part IV — Sprint order (suggested)

| Sprint | Issues | Outcome |
|--------|--------|---------|
| **A** | AX-001 + CLAIM_TIERS (done) | Axioms in code |
| **B** | AX-002 + grow **`axiom_system.json`** | Theorems + conformance |
| **C** | NF-001 (done) + NF-002 | Format contract |
| **D** | TX-* (framework spec) | `tri math-verify` |
| **E** | BR-001 | CLARA axiom queries |
| **F** | EX-001 | Experience closure |

---

## Part V — Publication outline (from this system)

1. **Introduction** — IEEE vs φ-structured motivation.  
2. **Mathematical foundation** — THM-001…009, **CLAIM_TIERS**.  
3. **GoldenFloat** — **FORMAT-SPEC-001**, **THM-009**, cross-language checks.  
4. **Empirical validation** — PHY-* tiers, falsifications.  
5. **Sacred physics accuracy** — GF16 vs BF16, BENCH-005.  
6. **CLARA** — axiom queries, conjectures.  
**Appendices:** **`axiom_system.json`**, **`FORMAT-SPEC-001.json`**, `tri math-verify --suite all` (target).

---

## System principles (summary)

1. **Single SSOT for layouts:** **`FORMAT-SPEC-001.json`** matches **`NUMERIC-STANDARD-001.md`**; languages **generate**, not reinterpret ad hoc.  
2. **`claim_tier` is mandatory** for math/physics claims in specs (enforcement phased).  
3. **`falsified_as_exact` is visible success** — not hidden debt.  
4. **Brain / CLARA** is the primary **consumer** of the catalog + tiers.  
5. **Reproducibility:** one command surface (**`tri math-verify`** — target) + sealed artifacts.

---

## References (external)

- Proptest / PBT ecosystem — oracle problem context (see math/physics test framework spec).  
- CODATA — [NIST constants](https://physics.nist.gov/cuu/Constants/).  
- mpmath — reference arithmetic for **differential** oracles (policy-compliant harnesses only).

---

**φ² + 1/φ² = 3 | TRINITY**
