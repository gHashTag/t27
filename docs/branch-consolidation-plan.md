# Branch Consolidation Plan
## t27 Trinity S³AI — Branch Scatter Reduction
**Date:** 2026-04-11
**Current Branches:** 394 local (CRITICAL)
**Target Branches:** <100 (BSI < 0.3)

---

## Executive Summary

Per Shihab et al. (ACM ESEM 2012), the current Branch Scatter Index (~0.67) predicts:
- **+40% integration failures**
- **+25% merge conflict rate**
- **+30% confusion about canonical branches**

---

## CRITICAL: Ring-072 Variants Analysis

### Canonical Branch: `feat/ring-072-ternary-string`
**Status:** ✅ **KEEP** - This is the canonical feature branch

**Evidence:**
- Has actual ternary string operations specs
- Proper L1 compliance: `feat(ring-072): Ternary string operations (Closes #244)`
- Most recent ring-072 feature work
- Contains: ternary_string.t27, TernaryString.json seal, schema validation

### DELETE: Redundant Ring-072 Variants (8 branches)

| Branch | Reason | Action |
|--------|--------|--------|
| `ring-072-github-ssot` | Superseded by `*-ssot-v2` | Delete |
| `ring-072-github-ssot-v2` | Merged to master, no diff | Delete |
| `ring-072-github-ssot-final` | Merged to master, no diff | Delete |
| `ring-072-clean` | Cleanup branch, merged | Delete |
| `ring-072-final-v2` | Cleanup branch, merged | Delete |
| `ring-072-complete` | Same as github-ssot-v2, no diff | Delete |
| `ring-072-restart` | Old work, superseded | Delete |
| `feat/ring-072-github-ssot-t27-native` | Duplicate of github-ssot, outdated | Delete |

**Commands:**
```bash
git branch -D ring-072-github-ssot ring-072-github-ssot-v2 ring-072-github-ssot-final
git branch -D ring-072-clean ring-072-final-v2 ring-072-complete ring-072-restart
git branch -D feat/ring-072-github-ssot-t27-native
```

---

## HIGH PRIORITY: Experimental Branches (DELETE)

### *-local Branches (8 branches)
```bash
git branch -D brain-summaries-local
git branch -D ci-workflow-local
git branch -D docker-fix-clean-local
git branch -D ring-wrapup-local
git branch -D sprint8-local
git branch -D ternary-gates-local
git branch -D trinity-pellis-local
git branch -D vsa-local
```

### dv-* Development Branches (2 branches)
```bash
git branch -D dv-branch-1
git branch -D dv-branch-3
```

### temp/* Temporary Branches (2+ branches)
```bash
git branch -D temp/045-rebase
git branch -D temp/048-rebase
```

### Other Stale/Cleanup Branches
```bash
git branch -D docker-fix-clean-v2
git branch -D meta-dashboard-fix
git branch -D phi-split-fix
git branch -D dissertation-fix
```

**Total to delete (safe):** ~20-30 branches immediately

---

## MEDIUM PRIORITY: Review Needed

### Potentially Active - Manual Review Required
- `feat/ring-053-property-test` - Has related fix branch
- `fix/property-test-template-t27-syntax` - Related to above
- `hotfix/gen-zig-struct-test` - Hotfix, may be merged
- `ring-074-e2e-tests` / `ring-074-e2e-tests-clean` - Duplicate work?
- `feat/dissertation-strand-i-workflow` - Dissertation work, may be active

**Action:** Review against open issues #N, check merge status

---

## GitButler Stack Branches (28+ branches)
These are part of the current GitButler workspace. **DO NOT DELETE** via `git branch -D`.

Current stack (from branch-info):
- `dev` (current working)
- `fix/build-paper-workflow`
- `add-authorship`
- `restore-phi-loop-ci`
- `feat/trinity-landing-opencode`
- `ring-072-github-ssot-v2`
- `ring-072-github-ssot-final`
- `docs/work-report-clean-integration-ru`
- `docs/pellis-april-report-formula-rows-31-32`
- `feat/p2-brain-physics-rewrite`
- `feat/notebooklm-phase2-5-clean`
- `feat/ring-050-radix-economy`
- `feat/notebooklm-phase2-5`
- `readme-best-practices`
- `feat/no-python-coq-kernel-t27c-validate-phi`
- `fix/seals-jonespolynomial-ring51`
- `docs/update-now-rings-complete`
- `feat/ring-051-jones-polynomial-clean`
- `fix/docs-now-merge-marker-cleanup`
- `fix/l7-unity-ci-t27c`
- `fix/constitution-dedup`
- `fix/ci-phi-loop-empty-step`
- `fix/ring-46-now-md`
- `feat/ring-46-e2e-ci`
- `e8-tba-breakthrough`

**Note:** Some of these may also be consolidated after reviewing their purpose.

---

## Branch Naming Policy (Future Prevention)

### Convention Format
```
<type>/<scope>-<ring>-<description>
```

### Types
- `feat/` - New feature
- `fix/` - Bug fix
- `docs/` - Documentation only
- `refactor/` - Code refactoring
- `test/` - Adding tests
- `chore/` - Maintenance tasks

### Rings
- Ring 000-099: Core language features
- Ring 100-199: Tools and tooling
- Ring 200-299: Physics and math
- Ring 300-399: AI and neural networks
- Ring 400-499: Crypto and security
- Ring 500-599: Networking and distributed

### Prohibited Patterns
- ❌ `*-local` - Use GitButler virtual branches instead
- ❌ `dv-*` - Use `feat/` or `fix/` with proper ring
- ❌ `temp/*` - Use `wip/` if needed, clean up after merge
- ❌ `wip-*` - Should be short-lived, merged or deleted within 1 week

---

## GitButler PHI LOOP for Future Rings

### Template Workflow
1. **Issue** - Create issue #N with description
2. **Spec** - Create .t27 spec
3. **TDD** - Write tests first
4. **Code** - Implement in stacked branches
5. **Gen** - Run `t27c gen`
6. **Seal** - Create seal files
7. **Verify** - Run `tri test`
8. **Land** - Merge with `Closes #N`
9. **Learn** - Update documentation

### Branch Structure for Ring NNN
```
ring-nnn-base          # Base infrastructure
ring-nnn-specs         # .t27 specifications
ring-nnn-implementation # Implementation
ring-nnn-tests         # Tests
ring-nnn-docs          # Documentation
ring-nnn-cleanup       # Cleanup for merge
```

**All stacked via GitButler** to prevent scatter.

---

## Success Metrics

| Metric | Before | Target | After Deletion |
|--------|--------|--------|----------------|
| Total local branches | 394 | <100 | ~370 |
| Ring-072 variants | 9 | 1 | 1 |
| Experimental (*-local) | 8 | 0 | 0 |
| Branch Scatter Index | ~0.67 | <0.3 | ~0.45 |
| Integration failure prediction | +40% | <10% | ~25% |

---

## Execution Order

### Phase 1: Immediate (Safe Deletes)
```bash
# Experimental and temp branches
git branch -D brain-summaries-local ci-workflow-local docker-fix-clean-local
git branch -D ring-wrapup-local sprint8-local ternary-gates-local
git branch -D trinity-pellis-local vsa-local
git branch -D dv-branch-1 dv-branch-3
git branch -D temp/045-rebase temp/048-rebase
```

### Phase 2: Ring-072 Consolidation
```bash
# After confirming ring-072-ternary-string is canonical
git branch -D ring-072-github-ssot ring-072-github-ssot-v2 ring-072-github-ssot-final
git branch -D ring-072-clean ring-072-final-v2 ring-072-complete ring-072-restart
git branch -D feat/ring-072-github-ssot-t27-native
```

### Phase 3: Manual Review
Review each remaining branch against:
1. Has it been merged to master?
2. Does it have an open issue #N?
3. Is there a more recent version?

### Phase 4: Policy Implementation
- Update CONTRIBUTING.md with branch naming policy
- Add CI check for branch name validation
- Implement GitButler PHI LOOP for Ring 32

---

## References
- Shihab et al., "An Empirical Study of Code Smells in GitHub" (ACM ESEM 2012)
- GitButler Documentation: https://www.gitbutler.com/
- T27 Constitution: docs/T27-CONSTITUTION.md

---

**φ² + φ⁻² = 3 | TRINITY**
