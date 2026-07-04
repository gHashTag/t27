# Wave Loop 99 Decomposition Plan

**Date:** 2026-06-16  
**Branch:** trinity-rust-rings  
**Target open issues:** ≤10 (current: 10)  
**Focus:** Concrete engineering fixes (L4 coverage, .v migration, extract_names), competitive stability confirmation

---

## Executive Summary

W99 is an **engineering consolidation loop**. No new features — close #1206, #1205, #1204, #933 and split #943. Keep open issue count at ≤10.

---

## Tracks

### Track A — L4 Test Coverage (#1206) [HIGH]
**Goal:** Add `test`/`invariant`/`bench` blocks to specs missing L4 coverage.

**Priority order (most impactful first):**
1. `specs/ternary/packed_trit.t27` — core ternary type, needs bounds invariant
2. `specs/ternary/hybrid_arithmetic.t27` — arithmetic correctness tests
3. `specs/isa/opcode_0xDE_load_phys_const.v` — sacred opcode test stub
4. `specs/igla/race/cordic_fixed.v` — CORDIC needs convergence invariant
5. `specs/tools/registry.t27` — registry API tests

**Skip (tooling/infrastructure):** `specs/tri/*`, `specs/pins/*`, `specs/pipeline/*` — these are meta-tooling stubs, not runtime specs.

**Definition of done:** ≥5 specs get L4 blocks; #1206 closed.

---

### Track B — Migrate Generated .v Files (#1205) [HIGH]
**Goal:** Move generated Verilog files from `specs/` to `gen/`.

**Files to migrate:**
- `specs/isa/opcode_0xDE_load_phys_const.v` → `gen/verilog/isa/`
- `specs/igla/race/cordic_fixed.v` → `gen/verilog/igla/race/` (already exists? check)
- `specs/igla/race/cordic_top.v` → `gen/verilog/igla/race/`
- `specs/igla/race/gemm.v` → `gen/verilog/igla/race/`
- `specs/fpga/verification/build_verify.v` → `gen/verilog/fpga/verification/`
- `specs/fpga/testbench/*_tb.v` → `gen/verilog/fpga/testbench/`

**Process:**
1. Move files to `gen/verilog/` preserving directory structure
2. Update any spec references that load these via `include`
3. Regenerate seals for affected specs
4. Verify suite still passes

**Definition of done:** All generated .v files migrated; #1205 closed.

---

### Track C — Fix extract_names Over-Collection (#1204) [MEDIUM]
**Goal:** `extract_names` in `bootstrap/src/compiler.rs:14013` should not collect numeric literals or keywords.

**Bug:** Current implementation collects any alphanumeric sequence, including:
- Numeric literals: `123`, `0xFF`, `42`
- Keywords: `if`, `else`, `for`, `while`, `case`, `end`, `begin`
- Operators disguised as words: `and`, `or`, `not`

**Fix:** Add filtering:
```rust
const VERILOG_KEYWORDS: &[&str] = &["if", "else", "for", "while", "case", "end", "begin", "module", "endmodule", "always", "initial", "assign", /* ... */];

fn is_valid_name(s: &str) -> bool {
    if s.is_empty() { return false; }
    if s.chars().next().unwrap().is_numeric() { return false; } // starts with digit = numeric literal
    if VERILOG_KEYWORDS.contains(&s) { return false; }
    true
}
```

**Definition of done:** Fix implemented, compiler tests pass, #1204 closed.

---

### Track D — Fix Conformance JSON (#933) [MEDIUM]
**Goal:** Fix invalid conformance JSON and stale spec_path references.

**Subtasks:**
1. Validate all `.json` files under `conformance/` with `jq empty`
2. Fix any trailing commas, missing braces
3. Fix `spec_path` references pointing to non-existent files
4. Add CI gate: `find conformance -name "*.json" -exec jq empty {} +`

**Definition of done:** All conformance JSON valid; #933 closed.

---

### Track E — Split #943 into Atomic Issues [MEDIUM]
**Goal:** Replace monolithic #943 with focused sub-issues.

**Sub-issues to create:**
1. `#943-a` — Bridge watch URL SSRF guard (medium)
2. `#943-b` — GraphQL injection sanitization (medium)
3. `#943-c` — Proxy DoS rate limiting (medium)
4. `#943-d` — Audio processing buffer overflow (medium)
5. `#943-e` — Partial comparison panic in sort (medium)

**Definition of done:** #943 closed as "split into atomic issues"; sub-issues created and linked.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Suite pass | 558/558 |
| Seal mismatches | 0 |
| Clippy warnings | 0 |
| Open issues | ≤10 |
| L4 violations fixed | ≥5 specs |
| .v files migrated | ≥10 files |
| New competitors | Document if any |

---

## Risk Mitigation

- **Risk:** Moving .v files breaks relative includes.
  - **Mitigation:** Search for `include` directives referencing moved files; update paths.
- **Risk:** extract_names fix breaks existing codegen.
  - **Mitigation:** Run full suite before and after; verify no regressions.
- **Risk:** Adding tests to stubs reveals broken syntax.
  - **Mitigation:** Run `t27c typecheck` on each spec individually before committing.

---

phi^2 + 1/phi^2 = 3 | TRINITY
