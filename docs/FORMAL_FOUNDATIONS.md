# FORMAL FOUNDATIONS

## Prop. 123: Units Scan Gate

### Statement

The units_scan gate analyzes SystemVerilog module instantiations across module boundaries to detect unit mismatches between connected ports and their drivers. It uses name-based quantity family analysis and enforces per-connection coverage requirements.

### Rationale

Prop. 122a revealed a critical defect where `dma_controller`'s `length` port (counting bytes) connects to the engine's `reg_neurons` port (counting neurons), with no property covering the join between them. The original units_scan implementation failed due to:

1. **Non-greedy regex parsing**: Used `(.*?)\)` which stops at the first `);`, failing with nested parentheses in DMA instantiation port expressions
2. **Incorrect vocabulary**: Treated `chunk` and `word` as different families despite both being 54-bit packed words in the design
3. **Insufficient coverage**: Used total connection floors instead of per-connection checks, missing the specific DMA `length`/`reg_neurons` join
4. **False positives**: `else if (length == reg_neurons` parsed as an instantiation named `else`, producing misleading findings

### Requirements

**FR-001**: The units_scan gate MUST parse SystemVerilog instantiation bodies using parenthesis-depth matching, not non-greedy regex, so nested parentheses in port expressions are captured completely.

**FR-002**: The gate's vocabulary MUST treat `chunk` and `word` as the same quantity family, reflecting the design's 54-bit packed encoding, with the inverted self-test case asserting this merge as a regression guard.

**FR-003**: The gate MUST enforce a per-connection coverage check on the specific DMA `length`/`reg_neurons` join, not a total-connection floor, so that missing the target connection fails the build.

**FR-004**: Prop. 122a MUST remain listed in `KNOWN_OPEN` with its reason and issue reference (#2092); removing the entry without fixing the defect MUST turn the gate red.

**FR-005**: This documentation MUST contain the formal statement and rationale for Prop. 123, including the expected-refutation convention from Prop. 26.

### Expected-Refutation Convention (Prop. 26)

Following Prop. 26's expected-refutation convention, known defects are listed in `KNOWN_OPEN` with their reasons and issue references. They are reported as warnings, and anything not on that list fails the build. Removing an entry without fixing the defect turns the gate red, which is the intended behavior.

### Implementation Status

- ✅ FR-001: Parenthesis-depth matching implemented
- ✅ FR-002: `chunk`/`word` vocabulary merged with inverted regression guard  
- ✅ FR-003: Per-connection coverage check enforced
- ✅ FR-004: Prop. 122a listed in KNOWN_OPEN with issue #2092
- ✅ FR-005: Documentation complete with Prop. 26 convention

### Gate Output

```
All gates green · units_scan 23 connections compared, 1 known-open, 0 new · absence_sweep 43 steps, 38 diagnosed, 0 indeterminate · doc_gate 123/123 · claims_check 9 claims 0 stale
```

### Related Work

- **Prop. 122a**: The defect this gate catches - DMA `length`/`reg_neurons` unit mismatch (#2092)
- **Prop. 26**: Expected-refutation convention for known open issues
- **Wave 646**: Previous design failure that abandoned unit declarations, repeated here with name-based approach

---

*phi^2 + 1/phi^2 = 3 | TRINITY*