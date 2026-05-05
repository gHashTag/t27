# Retroactive Issues Plan
## Creating issues for historical work - 2026-04-11

---

## Summary

**Finding:** Many significant commits lack L1 TRACEABILITY compliance (no "Closes #N").

**Constraint:** Cannot retroactively add issue references to existing commits.

**Solution:** Create retrospective issues for tracking purposes, even though they can't be linked to historical commits.

---

## FPGA Module Development (High Priority)

### Issue: FPGA Conformance & Analysis Infrastructure

**Type:** `feat`
**Title:** Add VCD conformance compare, power analysis, and seal collision detection
**Description:**
- VCD (Value Change Dump) conformance comparison
- Power analysis infrastructure
- Seal collision detection and fixes
**Commits:** `971fbfc1`, `c271bed3`
**Suggested Issue Number:** #500 (outside current range 129-196)

---

### Issue: FPGA Module Specification Complete

**Type:** `feat`
**Title:** Complete 32 FPGA module specs with conformance JSONs
**Description:**
- Conformance JSONs for all 32 FPGA module specifications
- 33 modules, 30 testbenches, 66 specs total
- Test coverage improvements for uart, top_level, bootrom, stdlib
**Commits:** `03fade98`, `860feb07`, `c271bed3`
**Suggested Issue Number:** #501

---

### Issue: FPGA Codegen Infrastructure

**Type:** `feat`
**Title:** Complete HIR-based codegen - XDC generation, SymbiYosys integration
**Description:**
- XDC generation from HIR (Hardware Intermediate Representation)
- CI formal verification upgrade
- Generic HIR support for codegen
- 10 module emitters implemented
- Testbench auto-generation
- SymbiYosys formal properties for MAC, FIFO, UART
**Commits:** `7b5f1d45`, `5b5184a8`, `85f97c47`, `a49e86df`
**Suggested Issue Number:** #502

---

### Issue: FPGA Build Verification

**Type:** `fix`
**Title:** Complete FPGA build verification infrastructure
**Description:**
- Build verification counts (33 modules verified)
- L4 TDD compliance for testbenches
- Generated Rust files integration for t27c compilation
- CdcStrategy fix for clock domain crossing
**Commits:** `983a7eb5`, `c9e6aa5a`, `0a325e17`, `446973d2`, `307097ac`
**Suggested Issue Number:** #503

---

## CI/Workflow Improvements (Medium Priority)

### Issue: L1 TRACEABILITY Merge Commit Handling

**Type:** `fix`
**Title:** Skip merge commits in L1 TRACEABILITY check
**Description:**
- GitButler workspace commits create merge commits
- L1 check should skip these to avoid false violations
**Commits:** `d8efcb38`
**Suggested Issue Number:** #504

---

### Issue: CI Workflow Fixes

**Type:** `fix`
**Title:** Resolve CI failures - workflow YAML and ternary_encoding.rs
**Description:**
- Fixed workflow YAML syntax errors
- Added missing ternary_encoding.rs to build
**Commits:** `fcd9be21`
**Suggested Issue Number:** #505

---

### Issue: FPGA Build Configuration

**Type:** `fix`
**Title:** Remove --profile argument from fpga-build command
**Description:**
- Simplified FPGA build command
- Removed profile flag to simplify usage
**Commits:** `6df3648a`
**Suggested Issue Number:** #506

---

## Documentation Updates (Low Priority)

### Issue: NOW.md Updates and CI Documentation

**Type:** `docs`
**Title:** Update NOW.md with CI fixes and date
**Description:**
- Updated date to 2026-04-14
- Added CI fixes note
- Created NotebookLM artifacts
**Commits:** `a49e86df`, `f2502214`
**Suggested Issue Number:** #507

---

## Issue: L3 PURITY Spec Re-sealing

**Type:** `chore`
**Title:** Re-seal 476 specs after Unicode cleanup
**Description:**
- L3 PURITY enforcement required Unicode character removal
- Re-sealed 476 affected specifications
- Ensures ASCII-only compliance
**Commits:** `983a7eb5`
**Suggested Issue Number:** #508

---

## Git Hook Testing

### Action Required: Test L1 TRACEABILITY Hook

```bash
# Create test commit without L1 compliance
cd bootstrap && git commit --allow-empty -m "test: verify L1 enforcement"
# Expected: REJECTED with error about missing "Closes #N"

# Create test commit with L1 compliance
git commit --allow-empty -m "test: verify L1 enforcement (Closes #999)"
# Expected: ACCEPTED
```

### Action Required: Test L3 PURITY Hook

```bash
# Create commit with non-ASCII identifier
echo "fn test_non_ascii() {}" > test.rs && git add test.rs
git commit -m "test: verify L3 enforcement"
# Expected: REJECTED with error about non-ASCII identifier
```

---

## Implementation Priority

1. **This Week:** Create issues #500-#508 in GitHub
2. **Today:** Test git hooks with commit attempts
3. **Tomorrow:** Document branch naming policy in CONTRIBUTING.md
4. **This Week:** Implement GitButler PHI LOOP template

---

## Issue Template

```markdown
## Title
[Type] [Ring-NNN]: [Brief description]

## Description
[Detailed description of work done]
[Include specific details, metrics, outcomes]

## Related Work
- Commit hashes: [list]
- Branch: [branch name used]
- Date: [when work was done]

## Outcome
[What was achieved, what remains]
```

---

**φ² + φ⁻² = 3 | TRINITY**
