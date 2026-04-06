<!-- Licensed under Apache License 2.0 -->

# CLARA Demo Pipeline

End-to-end demonstration of TRINITY S³AI CLARA pipeline for DARPA PA-25-07-02 reviewers.

## Related: CLARA-Bridge Assurance Pack

See [../clara-bridge/](../clara-bridge/) for extracted patterns, scenario runner, and audit trail schemas.

## Prerequisites
- Rust toolchain (for t27c compiler)
- Built t27c: `cd bootstrap && cargo build --release`

## Run
```bash
bash scripts/clara/demo.sh
```

## What it demonstrates

| Phase | Description | Specs |
|-------|-------------|-------|
| 1. **AR Parsing** | All 7 Automated Reasoning specs parse correctly | composition, ternary_logic, proof_trace, explainability, restraint, datalog_engine, asp_solver |
| 2. **NN Parsing** | Neural Network specs parse correctly | hslm, attention |
| 3. **VSA Parsing** | Vector Symbolic Architecture specs parse correctly | ops, core |
| 4. **Zig Codegen** | Specs generate valid Zig source code | composition, ternary_logic, proof_trace |
| 5. **Verilog Codegen** | Specs generate synthesizable Verilog (formal verification) | composition, ternary_logic |
| 6. **Seal Verification** | Cryptographic integrity hashes for spec immutability | composition, ternary_logic, proof_trace |
| 7. **GF16 Numeric** | NUMERIC-STANDARD-001 compliant GF16 format | gf16 |

## CLARA Compliance

- **AR in the guts of ML**: AR rules evaluate ML features via composition patterns (FAQ 21)
- **Concise explanations**: MAX_STEPS=10 enforced in proof traces
- **Polynomial-time guarantees**: O(n*m) + O(10) complexity bounds
- **Multi-backend**: Single `.t27` spec generates both Zig and Verilog
- **Cryptographic seals**: Immutable spec hashes for verification
