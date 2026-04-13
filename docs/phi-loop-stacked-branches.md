# PHI LOOP Stacked Branches Template
## t27 Trinity S³AI GitButler Integration

**Version:** 1.0
**Date:** 2026-04-11

---

## Overview

The PHI LOOP is a 9-step workflow for developing t27 rings using GitButler's stacked branches. Each phase is a virtual branch that depends on the previous one, enabling parallel development with clear dependencies.

---

## PHI LOOP: 9 Phases

### Phase 1: Issue
**Branch:** `ring-NNN-issue`
**Purpose:** Define the problem and create the GitHub issue

**Commands:**
```bash
but branch create ring-NNN-issue --from dev
but apply ring-NNN-issue
# Work on issue definition
but commit -m "docs(ring-NNN): Define ring NNN issue

Ring NNN: [Brief description]

## Problem
[Describe the problem]

## Solution
[Describe the solution]

## Success Criteria
- [ ] Criterion 1
- [ ] Criterion 2

Closes #N"
```

**Deliverables:**
- GitHub issue created
- Problem clearly defined
- Success criteria documented

---

### Phase 2: Spec
**Branch:** `ring-NNN-spec`
**Depends on:** `ring-NNN-issue`
**Purpose:** Write .t27 specifications

**Commands:**
```bash
but branch create ring-NNN-spec --from ring-NNN-issue
but apply ring-NNN-spec
# Write .t27 specs
but stage specs/path/to/module.t27 --branch ring-NNN-spec
but commit -m "feat(ring-NNN): Add module specifications

Adds .t27 specifications for [module name].

L2 GENERATION: gen/ files are generated from specs.

Closes #N"
```

**Deliverables:**
- .t27 spec files created
- TDD blocks included (L4 TESTABILITY)
- Invariants defined
- Benchmarks added

---

### Phase 3: TDD
**Branch:** `ring-NNN-tdd`
**Depends on:** `ring-NNN-spec`
**Purpose:** Write tests before implementation

**Commands:**
```bash
but branch create ring-NNN-tdd --from ring-NNN-spec
but apply ring-NNN-tdd
# Write test blocks in .t27 specs
but commit -m "test(ring-NNN): Add TDD tests

Adds test blocks for all functions.

L4 TESTABILITY: Every .t27 spec has test/invariant/bench.

Closes #N"
```

**Deliverables:**
- Test blocks complete
- All invariants defined
- Benchmarks added

---

### Phase 4: Code
**Branch:** `ring-NNN-code`
**Depends on:** `ring-NNN-tdd`
**Purpose:** Implement the feature (in specs, not gen/)

**Commands:**
```bash
but branch create ring-NNN-code --from ring-NNN-tdd
but apply ring-NNN-code
# Implement feature in .t27 specs
but commit -m "feat(ring-NNN): Implement [feature name]

Implements [feature description].

L2 GENERATION: Implementation in specs, gen/ will be updated in next phase.

Closes #N"
```

**Deliverables:**
- Feature implemented in .t27 specs
- Code follows L3 PURITY (ASCII-only, English)
- All tests pass

---

### Phase 5: Gen
**Branch:** `ring-NNN-gen`
**Depends on:** `ring-NNN-code`
**Purpose:** Generate code from specs

**Commands:**
```bash
but branch create ring-NNN-gen --from ring-NNN-code
but apply ring-NNN-gen
# Generate code
./scripts/tri gen specs/path/to/module.t27
# Verify generation
./scripts/tri validate-gen-headers
but commit -m "chore(ring-NNN): Regenerate code from specs

Regenerates gen/ files from updated specs.

L2 GENERATION: gen/ files are generated output, do not edit manually.

Closes #N"
```

**Deliverables:**
- gen/ files regenerated
- All generation checks pass
- L2 GENERATION verified

---

### Phase 6: Seal
**Branch:** `ring-NNN-seal`
**Depends on:** `ring-NNN-gen`
**Purpose:** Create verification seals

**Commands:**
```bash
but branch create ring-NNN-seal --from ring-NNN-gen
but apply ring-NNN-seal
# Create seals
./scripts/tri seal specs/path/to/module.t27 --save
but commit -m "chore(ring-NNN): Update verification seals

Updates seals for [module name].

L6 CEILING: FORMAT-SPEC-001.json and gf16.t27 are numeric SSOT.

Closes #N"
```

**Deliverables:**
- Seals created
- L6 CEILING verified
- SSOT files updated

---

### Phase 7: Verify
**Branch:** `ring-NNN-verify`
**Depends on:** `ring-NNN-seal`
**Purpose:** Verify conformance

**Commands:**
```bash
but branch create ring-NNN-verify --from ring-NNN-seal
but apply ring-NNN-verify
# Verify conformance
./scripts/tri validate-conformance
./scripts/tri test
but commit -m "test(ring-NNN): Verify conformance

All conformance checks pass.

L4 TESTABILITY: All tests pass.

Closes #N"
```

**Deliverables:**
- All conformance checks pass
- All tests pass
- No regressions

---

### Phase 8: Land
**Branch:** `ring-NNN-land`
**Depends on:** `ring-NNN-verify`
**Purpose:** Land to main branch

**Commands:**
```bash
but branch create ring-NNN-land --from ring-NNN-verify
but apply ring-NNN-land
# Update docs/NOW.md if applicable
# Update docs/TASK.md if applicable
but commit -m "docs(ring-NNN): Update documentation for ring completion

- docs/NOW.md: Update ring status
- docs/TASK.md: Update task progress

Ring NNN complete.

Closes #N"

# Push to remote
but push
```

**Deliverables:**
- Documentation updated
- PR created and reviewed
- Merged to main branch

---

### Phase 9: Learn
**Branch:** `ring-NNN-learn`
**Depends on:** `ring-NNN-land`
**Purpose:** Document learnings and improvement opportunities

**Commands:**
```bash
but branch create ring-NNN-learn --from ring-NNN-land
but apply ring-NNN-learn
# Document learnings
but commit -m "docs(ring-NNN): Document learnings from ring NNN

## What Went Well
- [ ] Item 1
- [ ] Item 2

## What Could Be Improved
- [ ] Item 1
- [ ] Item 2

## Action Items
- [ ] Item 1
- [ ] Item 2

Closes #N"
```

**Deliverables:**
- Learnings documented
- Action items identified
- Feedback loop closed

---

## Complete Workflow Script

```bash
#!/bin/bash
# phi-loop-stack.sh - Create complete PHI LOOP stacked branches

set -euo pipefail

RING_NUMBER=$1
ISSUE_NUMBER=$2

if [ -z "$RING_NUMBER" ] || [ -z "$ISSUE_NUMBER" ]; then
    echo "Usage: $0 <ring-number> <issue-number>"
    echo "Example: $0 32 42"
    exit 1
fi

PADDED_RING=$(printf "%03d" $RING_NUMBER)

echo "Creating PHI LOOP stacked branches for Ring $RING_NUMBER..."

# Phase 1: Issue
but branch create ring-${PADDED_RING}-issue --from dev
but apply ring-${PADDED_RING}-issue

# Phase 2: Spec (depends on issue)
but branch create ring-${PADDED_RING}-spec --from ring-${PADDED_RING}-issue

# Phase 3: TDD (depends on spec)
but branch create ring-${PADDED_RING}-tdd --from ring-${PADDED_RING}-spec

# Phase 4: Code (depends on TDD)
but branch create ring-${PADDED_RING}-code --from ring-${PADDED_RING}-tdd

# Phase 5: Gen (depends on code)
but branch create ring-${PADDED_RING}-gen --from ring-${PADDED_RING}-code

# Phase 6: Seal (depends on gen)
but branch create ring-${PADDED_RING}-seal --from ring-${PADDED_RING}-gen

# Phase 7: Verify (depends on seal)
but branch create ring-${PADDED_RING}-verify --from ring-${PADDED_RING}-seal

# Phase 8: Land (depends on verify)
but branch create ring-${PADDED_RING}-land --from ring-${PADDED_RING}-verify

# Phase 9: Learn (depends on land)
but branch create ring-${PADDED_RING}-learn --from ring-${PADDED_RING}-land

echo ""
echo "PHI LOOP stacked branches created!"
echo "Starting from Phase 1 (issue)..."
echo ""
echo "Issue reference: #$ISSUE_NUMBER"
echo ""
echo "Branches created:"
echo "  - ring-${PADDED_RING}-issue"
echo "  - ring-${PADDED_RING}-spec"
echo "  - ring-${PADDED_RING}-tdd"
echo "  - ring-${PADDED_RING}-code"
echo "  - ring-${PADDED_RING}-gen"
echo "  - ring-${PADDED_RING}-seal"
echo "  - ring-${PADDED_RING}-verify"
echo "  - ring-${PADDED_RING}-land"
echo "  - ring-${PADDED_RING}-learn"
```

---

## GitButler CLI Commands Reference

### Creating Stacked Branches
```bash
# Create branch with parent
but branch create <name> --from <parent>

# Apply branch to workspace
but apply <name>

# Unapply branch
but unapply <name>

# Move changes between branches
but rub <source> <target>
```

### Committing
```bash
# Stage file to specific branch
but stage <file> --branch <branch-name>

# Commit changes
but commit -m "message"

# Revert commit
but uncommit
```

### Moving Between Phases
```bash
# Move to next phase
but apply ring-NNN-[next-phase]

# Move changes to next phase
but rub ring-NNN-[current-phase] ring-NNN-[next-phase]
```

### Undo/Redo
```bash
# Undo last operation
but undo

# Redo undone operation
but redo

# View operation history
but oplog
```

---

## Integration with 27-Agent System

### Agent T (Queen Trinity)
Manages PHI LOOP orchestration:
- Creates stacked branches
- Sets up dependencies
- Monitors progress

### Agent L (LSP)
Provides code completion for .t27 specs
- Validates spec syntax
- Suggests test cases

### Agent C (Compiler)
Validates generation:
- Checks gen/ files
- Verifies L2 GENERATION

### Agent V (Verification)
Runs conformance checks:
- Validates invariants
- Runs tests

---

## Troubleshooting

### Issue: Branch Scatter
**Symptom:** Changes are in the wrong branch

**Solution:**
```bash
# Move changes to correct branch
but rub <wrong-branch> <correct-branch>
```

### Issue: Missing Issue Reference
**Symptom:** Commit blocked by L1 TRACEABILITY

**Solution:**
```bash
# Amend commit message
but reword <commit-id>
```

### Issue: Gen/ Files Modified Manually
**Symptom:** L2 GENERATION violation

**Solution:**
```bash
# Discard manual changes
but discard gen/path/to/file
# Regenerate from specs
./scripts/tri gen specs/path/to/module.t27
```

---

## Success Metrics

| Phase | Deliverable | Success Criteria |
|-------|-------------|------------------|
| Issue | GitHub issue | Problem defined, success criteria |
| Spec | .t27 files | TDD blocks, invariants, benchmarks |
| TDD | Test blocks | All functions tested |
| Code | Implementation | All tests pass |
| Gen | gen/ files | L2 GENERATION verified |
| Seal | Seals | L6 CEILING verified |
| Verify | Conformance | All checks pass |
| Land | PR merged | Documentation updated |
| Learn | Learnings | Feedback documented |

---

**φ² + φ⁻² = 3 | TRINITY**
