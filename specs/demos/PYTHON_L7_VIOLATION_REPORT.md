// SPDX-License-Identifier: Apache-2.0
# L7 UNITY Violations: Python Files in t27 Repository

## Executive Summary

**Found 18 Python files** in t27 repository, violating Ring 47 (L7 UNITY — Python ≤ 5 lines).

---

## Violations by Priority

### Critical (Must Fix Before P1)

| File | Size | Purpose | Violation |
|------|------|---------|------------|
| `clara-bridge/run_scenario.py` | 295 lines | CLARA Bridge runner | Depends on `tri/t27c` (external), 295 lines > 5 |
| `contrib/backend/agent-runner/agent-runner.py` | Unknown | Autonomous agent runner using external API | Python (any size) |

### High Priority

| File | Size | Purpose | Violation |
|------|------|---------|------------|
| `conformance/kepler_newton_tests.py` | Unknown | Conformance test framework | Python (unknown size) |

### Medium Priority (Assess and Plan)

| File | Size | Purpose | Violation |
|------|------|---------|------------|
| `bootstrap/parse_t27.py` | Small | Bootstrap script (Python!) | Python |
| `bootstrap/t27c.py` | Unknown | Bootstrap compiler (Python!) | Python |
| `confluence/*.py` | Small | Confluence integration | Python |

### Low Priority (Legacy/Tests)

| File | Size | Purpose | Violation |
|------|------|---------|------------|
| `external/kaggle/scripts/*.py` | Multiple | Research scripts | Python |
| `research/*.py` | Multiple | TBA research scripts | Python |
| `scripts/*.py` | Small | Utility scripts | Python |

---

## Analysis by File

### 1. `clara-bridge/run_scenario.py` (CRITICAL)

**Purpose**: CLARA Bridge scenario runner
**Dependency**: Calls `tri/t27c` (external tool)
**Violation**: 295 lines of Python code
**Impact**: This is a core integration component that MUST be rewritten in t27c

### 2. `contrib/backend/agent-runner/agent-runner.py` (CRITICAL)

**Purpose**: Autonomous agent using Z.AI/Anthropic API
**Dependency**: External API (not t27)
**Violation**: Python (any size not allowed)
**Impact**: Security/compliance issue — external dependencies must be audited

### 3. `conformance/kepler_newton_tests.py` (HIGH)

**Purpose**: Conformance test runner
**Dependency**: Unknown (may use external tools)
**Violation**: Python code in conformance directory
**Impact**: Test infrastructure needs clarification

### 4. `bootstrap/parse_t27.py` (MEDIUM)

**Purpose**: Bootstrap script
**Violation**: Python code in bootstrap directory
**Note**: Bootstrap tools often exempt, but this should be t27c

### 5. `bootstrap/t27c.py` (MEDIUM)

**Purpose**: Bootstrap compiler
**Violation**: Python code in bootstrap directory
**Note**: Compiler itself is Rust, but driver is Python

---

## Proposed Actions

### Phase 1: Immediate (Before P1)

1. **Delete all critical Python files**:
   ```bash
   rm clara-bridge/run_scenario.py
   rm -rf clara-bridge/tests/  # Or assess first
   rm contrib/backend/agent-runner/agent-runner.py
   ```

2. **Create GitHub issue** documenting L7 violations:
   ```bash
   gh issue create \
     --title "L7 UNITY Violations: 18 Python files in t27 repo" \
     --label "priority:P0 security" \
     --body "See PYTHON_L7_VIOLATION_REPORT.md for full inventory. Critical: clara-bridge/run_scenario.py (295 lines) depends on external tri/t27c. Requires t27c rewrite or approved L7 exception."
   ```

### Phase 2: Assessment (Part of P1)

1. **Audit each remaining `.py` file**:
   - Is it critical infrastructure? (CLARA, conformance, agent-runner)
   - Is it legacy research? (kaggle, reasearch)
   - Is it utility? (scripts)
   - Can it be migrated to t27c? Or does it need L7 exemption?

2. **Document migration plan** for each file:
   - Target: t27c implementation
   - Effort estimate: XS/S/M/L/XL
   - Dependencies: None / External / Complex

---

## Questions for User

1. **Should I delete all 18 files immediately?**
   - Risk: May break CLARA Bridge or conformance tests temporarily
   - Alternative: Delete only confirmed violations first

2. **What priority: P1 cleanup or full migration?**
   - P1 Jones cleanup is ~30 minutes
   - Full Python migration could be hours

3. **L7 exception needed?**
   - Some files may be legitimately needed (e.g., legacy tests)
   - Requires Ring 47 review for exemption

---

**Total estimated migration effort**: 4-8 hours (depending on L7 exception process)
