# Compiler Bugs Reference

This document documents significant compiler bugs, their fixes, and verification methods to prevent regressions.

## copy_propagate Silent Infinite Loop Bug

### Defect Shape

**Issue**: `copy_propagate` folded a mutated loop variable with its initializer, producing a silent infinite loop.

**Root Cause**: The copy propagation optimization incorrectly treated a loop variable that was both initialized and mutated within the loop as a static value. It replaced all uses of the variable with its initial value, effectively removing the mutation and causing the loop to run indefinitely.

**Impact**: Silent infinite loops in generated code, leading to hangs during execution rather than compilation errors or test failures.

### Fix

The fix prevents copy propagation of variables that are both initialized and mutated within the same scope. The compiler now tracks mutation status and disables propagation for variables that are modified after their initial assignment.

**Implementation**: Added mutation tracking to the copy propagation pass to identify variables that are reassigned after their initial declaration, and exclude such variables from propagation optimizations.

### Six-Pass Sweep Verification Method

To ensure this bug does not regress, implement a six-pass sweep verification method:

1. **Pass 1**: Identify all loops with local variables that are both initialized and mutated
2. **Pass 2**: Check copy propagation behavior on these variables
3. **Pass 3**: Verify that mutated variables are not propagated
4. **Pass 4**: Test compilation of affected patterns
5. **Pass 5**: Run generated code to detect infinite loops
6. **Pass 6**: Compare behavior before and after the fix

**Verification Command**:
```bash
# Run comprehensive test suite focusing on loop variable propagation
t27c test --focus "loop.*propagation|copy.*propagation" --timeout 30s
```

### Generalization Note

For future maintenance, search for other text-encoded mutations using:

```bash
grep -r "extra_op" src/ compiler/ --include="*.rs" --include="*.t27"
```

The `extra_op` pattern may indicate other places where text-based transformations could introduce similar mutation-related bugs. Look for patterns where:
- Text representation of operations is used instead of AST manipulation
- Mutation status is not properly tracked
- Side effects are not considered in optimizations

### Related Issues

- #3038: Original bug report and fix
- #3041: Documentation-only issue to create this reference

---

*This document is maintained as part of the L1 TRACEABILITY requirement to ensure compiler bugs are durably documented and searchable.*