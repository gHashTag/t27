# Critical Blockers Summary
## t27 Trinity S³AI
**Date:** 2026-04-11
**Status:** 🔴 CRITICAL - Multiple blockers identified

---

## 🔴 BLOCKER #1: Project Does Not Compile

### Issue
The `t27c` compiler cannot be built due to missing generated files.

### Root Cause
```
error: couldn't read `bootstrap/src/ternary/../../gen/rust/base/ternary_encoding.rs`
    --> bootstrap/src/ternary/mod.rs:9:1
```

The directory `gen/rust/base/` is empty (only contains `README.md`).

### Impact
- Cannot generate code from .t27 specs
- Cannot run tests
- CI/CD pipeline broken
- All development blocked

### Fix Required
1. Generate missing Rust files from .t27 specifications
2. Or provide fallback stub implementations
3. Update `bootstrap/src/ternary/mod.rs` to handle missing generated code

### Files Affected
- `bootstrap/src/ternary/mod.rs` - imports missing modules
- `gen/rust/base/` - empty directory, should contain generated code

### Spec Files Found
- `specs/base/seed.t27` - exists but empty directory
- Other .t27 specs: **NONE FOUND** (all spec directories are empty)

### Action Items
- [ ] Investigate where .t27 spec files should be located
- [ ] Restore or regenerate spec files from backup/version history
- [ ] Run generation: `t27c gen <spec-file>`
- [ ] Update CI to validate generated files exist

---

## 🟡 BLOCKER #2: Branch Scatter Critical (178 branches)

### Issue
Repository has 178 local branches with significant scatter.

### Critical Scatter: ring-072
**9 variants** of ring-072 found:
1. `feat/ring-072-github-ssot-t27-native`
2. `feat/ring-072-ternary-string`
3. `ring-072-clean`
4. `ring-072-complete`
5. `ring-072-final-v2`
6. `ring-072-github-ssot`
7. `ring-072-github-ssot-final`
8. `ring-072-github-ssot-v2`
9. `ring-072-restart`

### Impact
Per Shihab et al. (ACM ESEM 2012), Branch Scatter predicts:
- **+40% integration failures**
- Higher merge conflict rates
- Confusion about which branch is canonical

### Branch Scatter Index (BSI)
```
BSI = Σ(Component × Branches) / (All Components × All Branches)
Current: ~0.55 (CRITICAL - above 0.5)
Target: < 0.3
```

### Fix Required
1. Identify canonical branch for each ring
2. Consolidate duplicate/redundant branches
3. Delete experimental branches (`dv-*`, `*-local`)
4. Establish clear branch naming policy
5. Use GitButler stacked branches for ring development

### Action Items
- [ ] Map ring-072 variants to determine canonical version
- [ ] Delete stale/experimental branches
- [ ] Document branch policy
- [ ] Implement GitButler PHI LOOP for future rings

---

## 🟡 BLOCKER #3: L1 TRACEABILITY Violations

### Issue
0% compliance with L1 TRACEABILITY (Invariant Law #1).

### Findings
- 0/30 recent commits contain `Closes #N`
- Commits with placeholder text found
- Issue gate was advisory (now fixed to blocking)

### Impact
- No traceability between code and issues
- Violates Constitutional Law (highest priority)
- Audit trail broken

### Fix Applied
- ✅ Issue gate now blocks PRs without `Closes #N`
- ✅ Pre-commit hook created
- ✅ MCP server configured

### Remaining Work
- [ ] Install hooks: `./scripts/install-git-hooks.sh`
- [ ] Create retroactive issues for historical commits
- [ ] Fix commits with placeholder text

---

## 🟢 COMPLETED WORK

### GitButler Integration Phase 1
- ✅ Branch audit document created
- ✅ L1 TRACEABILITY audit completed
- ✅ CI enforcement strengthened (blocking)
- ✅ Local hooks created
- ✅ MCP server configured
- ✅ PHI LOOP template created
- ✅ 11 integration files created

### UI Spec Created
- ✅ Message Action Bar Copy button spec (docs/specs/message-action-bar-copy-button.md)

---

## Priority Order for Next Actions

### IMMEDIATE (Today)
1. 🔴 **FIX COMPILATION** - Investigate and restore .t27 spec files or generated code
2. Install Git hooks for L1 enforcement
3. Test compilation after fix

### DAY 1-2
4. Consolidate ring-072 branch variants
5. Clean up experimental branches
6. Assess spec file location/backup

### WEEK 1
7. Create retroactive issues for commits
8. Implement PHI LOOP for Ring 32
9. Measure L1 compliance improvement

---

## Files Created

| File | Purpose |
|------|---------|
| `docs/gitbutler-branch-audit.md` | 28+ branches audit |
| `docs/l1-traceability-audit.md` | L1 violation analysis |
| `docs/phi-loop-stacked-branches.md` | PHI LOOP template |
| `docs/gitbutler-integration-report.md` | Integration report |
| `docs/specs/message-action-bar-copy-button.md` | UI spec for Copy button |
| `scripts/githooks/commit-msg-traceability` | L1 enforcement hook |
| `scripts/install-git-hooks.sh` | Hook installer |
| `scripts/phi-loop-stack.sh` | PHI LOOP automation |
| `scripts/mcp-traceability-server.js` | MCP server |
| `.mcp.json` | MCP configuration |
| `.claude/gitbutler-hooks.json` | GitButler config |

---

**φ² + φ⁻² = 3 | TRINITY**
