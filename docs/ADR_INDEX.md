# ADR Index — Architecture Decision Records

**Status:** Active (Ring 054)
**Date:** 2026-05-09
**Purpose:** Complete index of all Architecture Decision Records (ADRs) in t27

---

## 1. Overview

This document provides a complete index of all Architecture Decision Records (ADRs) in the T27 project, organized by status and category.

### 1.1 ADR Status

| Status | Description | Count |
|--------|-------------|-------|
| Accepted | Decision implemented and approved | 5 |
| Proposed | Decision under consideration | 0 |
| Deprecated | Decision superseded by newer ADR | 0 |
| Superseded | Decision replaced by another | 0 |

### 1.2 ADR Categories

| Category | Description | ADRs |
|----------|-------------|------|
| Language | Language design decisions | ADR-004, ADR-005 |
| Architecture | System architecture decisions | ADR-001 |
| Development | Development workflow decisions | ADR-003 |
| Documentation | Documentation standards | ADR-002 |

---

## 2. Accepted ADRs

### ADR-001: De-Zigfication

**Status:** Accepted  
**Date:** 2026-04-01  
**Context:** Project was initially Zig-only, needed spec-first approach

**Decision:**
- `.t27` specs are the single source of truth
- Zig, C, Verilog, Rust, TypeScript are generated backends
- No hand-editing of generated code

**Consequences:**
- ✅ Single source of truth
- ✅ Multi-backend support
- ⚠️ Requires regeneration for changes
- ✅ Enforced by CI (L2 GENERATION law)

**Related:**
- `SOUL.md` — L2 GENERATION law
- `docs/BACKEND_CONTRACT.md` — Backend obligations
- `docs/SPECS_BOUNDARY.md` — Core vs research specs

---

### ADR-002: TDD Inside Spec

**Status:** Accepted  
**Date:** 2026-04-02  
**Context:** Need to ensure testability without separate test files

**Decision:**
- Tests defined inside `.t27` specs using `test {}` blocks
- Invariants defined using `invariant {}` blocks
- Benchmarks defined using `bench {}` blocks

**Consequences:**
- ✅ Tests co-located with specs
- ✅ Enforced by L4 TESTABILITY law
- ✅ Conformance vectors auto-generated

**Related:**
- `SOUL.md` — L4 TESTABILITY law
- `docs/TESTING_TAXONOMY.md` — Test classification
- `docs/CONFORMANCE_TRACEABILITY.md` — Conformance mapping

---

### ADR-003: Test-Driven Development Contract

**Status:** Accepted  
**Date:** 2026-04-03  
**Context:** Define TDD workflow for T27

**Decision:**
- Every spec must have `test {}`, `invariant {}`, or `bench {}`
- Tests must pass before spec is sealed
- Conformance vectors required for all specs

**Consequences:**
- ✅ High test coverage (100% achieved)
- ✅ Verified correctness
- ⚠️ Slower initial development

**Related:**
- `docs/nona-03-manifest/TDD-CONTRACT.md` — Full contract
- `docs/TESTING_TAXONOMY.md` — Test types

---

### ADR-004: Language Policy

**Status:** Accepted  
**Date:** 2026-04-04  
**Context:** Need to define language and character set policy

**Decision:**
- All source files ASCII-only
- All identifiers English-only
- Comments English-only
- No Unicode in core specs

**Consequences:**
- ✅ Maximum compatibility
- ✅ Clear error messages
- ⚠️ Limited character set for names

**Related:**
- `SOUL.md` — L3 PURITY law
- `docs/T27-CONSTITUTION.md` — LANG-EN axiom

---

### ADR-005: Language Food Chain

**Status:** Accepted  
**Date:** 2026-05-09  
**Context:** Define language hierarchy and usage

**Decision:**
```
.t27 (write by hand) → .tri (generated IR) → backends (generated)
```

**Consequences:**
- ✅ Clear language boundaries
- ✅ Prevents circular dependencies
- ⚠️ Requires compilation step

**Related:**
- `CLAUDE.md` — Language food chain
- `docs/BACKEND_CONTRACT.md` — Backend mapping

---

### ADR-006: De-Zig Strict (Implied by ADR-001)

**Status:** Accepted (implied)  
**Date:** 2026-04-01  
**Context:** Enforce strict spec-first approach

**Decision:**
- Zero tolerance for hand-written Zig in product paths
- All Zig must be generated from `.t27`
- L2 GENERATION law enforcement

**Consequences:**
- ✅ Guarantees spec-first approach
- ⚠️ More complex setup for contributors

**Related:**
- `architecture/CANON_DE_ZIGFICATION.md` — Full de-zigfication spec

---

## 3. ADR Template

When creating a new ADR, use this template:

```markdown
# ADR-XXX: [Title]

**Status:** [Proposed | Accepted | Deprecated | Superseded]
**Date:** YYYY-MM-DD
**Context:** [Problem statement]

## Decision
[Decision statement]

## Consequences
- **Positive:** [Benefit 1]
- **Positive:** [Benefit 2]
- **Negative:** [Drawback 1]
- **Negative:** [Drawback 2]

## Related
- [ADR-YYY] — Related ADR
- [Document] — Related documentation
```

---

## 4. ADR Process

### 4.1 Creating an ADR

1. Create new ADR in `architecture/ADR-XXX-title.md`
2. Use the template above
3. Set status to `Proposed`
4. Create GitHub Issue for discussion
5. Get approval from maintainer

### 4.2 Accepting an ADR

1. Set status to `Accepted`
2. Update this index
3. Link to related issues
4. Document in `CANON.md` if needed

### 4.3 Deprecating an ADR

1. Set status to `Deprecated` or `Superseded`
2. Add reference to replacement ADR
3. Document deprecation reason
4. Update this index

---

## 5. Module Roles (EPIC-8)

| Module | Primary ADR | Status |
|--------|-------------|--------|
| Language core | ADR-001, ADR-005 | Active |
| Testing | ADR-002, ADR-003 | Active |
| Code generation | ADR-001 | Active |
| Documentation | ADR-004 | Active |

---

## 6. Future ADRs

### Proposed

None currently proposed.

### Under Consideration

- ADR-007: WASM Backend Support
- ADR-008: Coq Verification Integration
- ADR-009: Python FFI Standard

---

## 7. References

- `architecture/ADR-001-de-zigfication.md` — De-Zigfication details
- `architecture/ADR-003-tdd-inside-spec.md` — TDD contract
- `architecture/ADR-004-language-policy.md` — Language policy
- `architecture/CANON_DE_ZIGFICATION.md` — Canonical de-zigfication
- `docs/T27-CONSTITUTION.md` — Constitutional law

---

## 8. Statistics

- **Total ADRs:** 6 (5 accepted, 1 implied)
- **Active ADRs:** 6
- **Deprecated ADRs:** 0
- **Superseded ADRs:** 0

---

**φ² + 1/φ² = 3 | TRINITY**
