# GitButler Branch Audit for t27 Trinity S³AI
## Generated: 2026-04-11

## Summary
- **Total branches:** 28
- **Current working branch:** dev
- **Unassigned changes:** 11 files
- **GitButler version:** CLI available at gitbutler-tauri

---

## Branch Metadata Map

### 1. Core Development

| Branch | Type | Status | Notes |
|--------|------|--------|-------|
| `dev` | main-dev | active | Current working branch with 50+ commits |

### 2. Ring-Specific Work

| Branch | Ring | Type | Notes |
|--------|------|------|-------|
| `ring-072-github-ssot-v2` | 72 | refactor | GitHub Single Source of Truth v2 |
| `ring-072-github-ssot-final` | 72 | refactor | Final version, likely depends on v2 |
| `ring-050-radix-economy` | 50 | feature | Radix economy optimization |
| `ring-051-jones-polynomial-clean` | 51 | refactor | Jones polynomial cleanup |
| `ring-46-e2e-ci` | 46 | feat | End-to-end CI testing |

### 3. Feature Branches

| Branch | Type | Notes |
|--------|------|-------|
| `feat/trinity-landing-opencode` | feature | Trinity opencode landing |
| `feat/p2-brain-physics-rewrite` | feature | Physics engine rewrite |
| `feat/notebooklm-phase2-5` | feature | NotebookLM Phase 2.5 |
| `feat/notebooklm-phase2-5-clean` | feature | Clean version of Phase 2.5 (possible duplicate) |
| `feat/no-python-coq-kernel-t27c-validate-phi` | feature | Remove Python CoQ kernel, validate phi |

### 4. Fix Branches

| Branch | Type | Notes |
|--------|------|-------|
| `fix/build-paper-workflow` | fix | Paper build workflow |
| `fix/seals-jonespolynomial-ring51` | fix | Seals for Jones polynomial (Ring 51) |
| `fix/docs-now-merge-marker-cleanup` | fix | Documentation merge markers cleanup |
| `fix/l7-unity-ci-t27c` | fix | L7 Unity CI for t27c |
| `fix/constitution-dedup` | fix | Constitution deduplication |
| `fix/ci-phi-loop-empty-step` | fix | PHI LOOP CI empty step |
| `fix/ring-46-now-md` | fix | Ring 46 NOW documentation |

### 5. Documentation Branches

| Branch | Type | Notes | LANG-EN Compliance |
|--------|------|-------|-------------------|
| `docs/work-report-clean-integration-ru` | docs | Work report cleanup | **VIOLATION** (Russian suffix) |
| `docs/pellis-april-report-formula-rows-31-32` | docs | Pellis April report formulas | Compliant |
| `docs/update-now-rings-complete` | docs | NOW documentation update | Compliant |
| `readme-best-practices` | docs | README best practices | Compliant |

### 6. Experimental/Other

| Branch | Type | Notes | Action Needed |
|--------|------|-------|---------------|
| `e8-tba-breakthrough` | experimental | E8 breakthrough research | Keep in research workspace |
| `dv-branch-1` | experimental | Development branch 1 | Merge or remove |
| `dv-branch-2` | experimental | Development branch 2 | Merge or remove |
| `add-authorship` | metadata | Authorship attribution | Review and integrate |
| `restore-phi-loop-ci` | maintenance | Restore PHI LOOP CI | Review status |

---

## Unassigned Changes (11 files)

| File | Type | Action |
|------|------|--------|
| `bootstrap/src/main.rs~` | backup | **REMOVE** - backup file |
| `contrib/backend/music-generator/bark_trap_test.wav` | audio | Removed |
| `contrib/backend/music-generator/generate_musicgen.py` | python | Added |
| `contrib/backend/music-generator/music_all.py` | python | Added |
| `contrib/backend/music-generator/music_gen/__init__.py` | python | Modified |
| `contrib/backend/music-generator/music_gen/acestep.py` | python | Added |
| `contrib/backend/music-generator/music_gen/heartmusa.py` | python | Added |
| `contrib/backend/music-generator/musicgen_test.wav` | audio | Removed |
| `contrib/backend/music-generator/tsar_bell_church_test.wav` | audio | Removed |
| `research/trinity-pellis-paper/ALPHA_S_GOLDEN_RATIO_PREPRINT.md` | doc | Modified |
| `specs/isa/ternary_encoding.t27` | spec | Added - new spec file |

---

## Inferred Dependencies

### Stacked Branches (Candidate)
```
ring-072-github-ssot-final
  └── depends_on: ring-072-github-ssot-v2

feat/notebooklm-phase2-5-clean
  └── depends_on: feat/notebooklm-phase2-5 (possible duplicate)
```

### Ring Dependencies
```
ring-72 (github-ssot-v2, github-ssot-final)
ring-51 (jones-polynomial-clean, seals-jonespolynomial-ring51)
ring-50 (radix-economy)
ring-46 (e2e-ci, ring-46-now-md)
```

---

## Risk Analysis

### High Priority Issues

| Issue | Severity | Impact | Mitigation |
|-------|----------|--------|------------|
| `docs/work-report-clean-integration-ru` | Medium | LANG-EN violation | Translate to English or move to `.legacy-non-english-docs` |
| Branch scatter in docs/ | High | Integration conflicts | Consolidate or stack doc branches |
| `feat/notebooklm-phase2-5` + `phase2-5-clean` | Medium | Possible duplication | Review and merge |
| `dv-branch-1`, `dv-branch-2` | Low | Workspace clutter | Merge to dev or remove |
| `bootstrap/src/main.rs~` | Low | Backup file clutter | Remove immediately |

### Branch Scatter Analysis

Based on Shihab et al. (ACM ESEM 2012), components scattered across many branch families experience more integration failures.

**Scattered Components:**
- **Documentation:** 4 separate branches modifying docs/
- **CI/CD:** 3 separate branches (`ring-46-e2e-ci`, `fix/l7-unity-ci-t27c`, `fix/ci-phi-loop-empty-step`)
- **Ring 46 work:** 2 branches (`ring-46-e2e-ci`, `fix/ring-46-now-md`)
- **Ring 51 work:** 2 branches (`ring-051-jones-polynomial-clean`, `fix/seals-jonespolynomial-ring51`)

**Branch Scatter Index (BSI):** ~0.35 (moderate - needs improvement)

---

## Action Plan

### Immediate (Today)

- [x] Create branch audit document
- [ ] Remove `bootstrap/src/main.rs~` backup file
- [ ] Stage `specs/isa/ternary_encoding.t27` to appropriate branch

### Day 1-2

- [ ] Review and consolidate `feat/notebooklm-phase2-5-*` branches
- [ ] Assess `docs/work-report-clean-integration-ru` for translation
- [ ] Clean up `dv-branch-1` and `dv-branch-2`
- [ ] Verify all commits have `Closes #N` (L1 TRACEABILITY)

### Week 1

- [ ] Create proper stacked structure for ring-072-* branches
- [ ] Consolidate CI/CD changes into single branch
- [ ] Set up AI commit message generation
- [ ] Create PHI LOOP stacked branch template

---

## GitButler Commands Reference

### Current Workflow
```bash
# View status
but status

# Stage file to branch
but stage <file> --branch <branch-name>

# Commit changes
but commit -m "message"

# View diff
but diff

# Undo last operation
but undo
```

### Branch Management
```bash
# View all branches
but branch list

# Apply/unapply branch
but apply <branch-name>
but unapply <branch-name>

# Move changes between branches
but rub <source> <target>

# Stack branches (dependency)
but branch set-upstream <branch> <parent>
```

---

## Next Steps

1. **Phase 1 Complete:** Branch audit documented
2. **Phase 2:** Configure AI commit messages for t27 conventions
3. **Phase 3:** Set up PHI LOOP stacked branches template
4. **Phase 4:** MCP server integration for 27-agent system

---

**φ² + φ⁻² = 3 | TRINITY**
