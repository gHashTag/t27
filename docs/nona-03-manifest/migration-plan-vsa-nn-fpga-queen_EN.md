# Plan: Migration of .t27 Specifications from trinity-w1

## Context

Target repository `/Users/playra/t27` already contains significant structure:
- ✅ **Done**: specs/math/* (constants, sacred_physics), specs/numeric/* (GF4-32, TF3, phi_ratio), compiler/parser, compiler/codegen/zig/verilog, conformance/*, docs/*, architecture/*, .trinity/*
- ✅ **Migration complete**: specs/vsa/ops.t27, specs/isa/registers.t27, specs/nn/attention.t27, specs/nn/hslm.t27, specs/fpga/mac.t27, specs/queen/lotus.t27, compiler/codegen/c/codegen.t27, compiler/runtime/runtime.t27, compiler/parser/lexer.t27
- 📝 **All .t27 files** now in canonical format (module/fn/test/invariant/bench)

Source repository `/Users/playra/trinity-w1` contains Zig code that needs to be extracted into .t27 specifications:
- src/tri/math/ - sacred formula, constants
- src/vsa/ - vector operations
- src/hslm/ - HSLM attention
- src/isa/, src/fpga/ - registers, MAC operations
- src/tri/ - orchestration, cells

## Goal

✅ **Complete**: All 9 .t27 specifications created and standardized in canonical format. Architectural consistency and dependencies in graph_v2.json preserved.

## Execution Plan via tri-cell

### Step 1: Save plan as document

**Action**: Save this plan in `t27/docs/nona-03-manifest/migration-plan-vsa-nn-fpga-queen.md`
This will become canonical reference for all future agents.

### Step 2: Create tri-cell for migration

**Example protocol:**
```bash
tri cell begin --issue <N> --episode migrate-trinity-w1-specs
tri cell checkpoint --step "VSA ops spec skeleton created"
tri cell checkpoint --step "ISA registers spec skeleton created"
tri cell checkpoint --step "NN attention/HSLM specs created"
tri cell checkpoint --step "FPGA MAC + Queen Lotus specs created"
tri cell checkpoint --step "compiler C codegen/runtime/lexer specs created"
tri cell checkpoint --step "graph_v2.json updated with new nodes/edges/order"
tri cell seal
tri verdict --toxic
tri experience save --episode migrate-trinity-w1-specs
git add .
git commit -m "feat: migrate VSA/NN/FPGA/Queen specs from trinity-w1"
git push
```

### Step 3: Execute by steps (in order from plan)

**Detailed execution:**

1. **VSA ops** → `t27/specs/vsa/ops.t27` from `src/vsa/agent/core.zig` and `src/vsa/common.zig`
   - Functions: bind, unbind, bundle, similarity, trit_cosine
   - use base::types, base::ops

2. **ISA registers** → `t27/specs/isa/registers.t27` from `src/tri27/` or `src/isa/`
   - Functions: Register, RegisterFile, R0-R26, Coptic encoding
   - use base::types

3. **NN attention** → `t27/specs/nn/attention.t27` from `src/hslm/attention.zig`
   - Functions: sacred_attention, d_k^(-φ³) kernel
   - use math::constants, base::types, numeric::gf16

4. **HSLM** → `t27/specs/nn/hslm.t27` from `src/hslm/` and `src/tri/brain/`
   - Functions: HSLM, forward, backward, phase
   - use nn::attention, math::sacred_physics, numeric::gf16

5. **FPGA MAC** → `t27/specs/fpga/mac.t27` from `src/fpga/`
   - Functions: ZeroDSP_MAC, LUT, MAC cycle
   - use base::types, base::ops, isa::registers

6. **Queen Lotus** → `t27/specs/queen/lotus.t27` from `src/tri/queen/` or `src/tri/cell.zig`
   - Functions: 6-phase orchestrate, phase management, cell infrastructure
   - use nn::hslm, compiler::runtime

7. **C codegen** → `t27/compiler/codegen/c/codegen.t27` from existing Zig codegen
   - Functions: CCodeGen, emit_c, c_ast, c_headers
   - use compiler::parser, compiler::runtime

8. **Runtime** → `t27/compiler/runtime/runtime.t27` from existing runtime
   - Functions: T27Runtime, init, execute, shutdown
   - use base::types

9. **Lexer** → `t27/compiler/parser/lexer.t27` based on parser.t27
   - Functions: Lexer, tokenize, Token, TokenType
   - Dependencies: parser uses lexer

10. **Update graph_v2.json**
   - Add 8 new nodes
   - Add dependencies
   - Update topological_order

## Files to create/modify

| File | Action | Key elements | Status |
|------|-----------|------------------|--------|
| t27/specs/vsa/ops.t27 | ✅ COMPLETE | bind, unbind, bundle, similarity | Skill 017 (cafc405) |
| t27/specs/isa/registers.t27 | ✅ COMPLETE | Register, RegisterFile, R0-R26 | Skill 020 (8296d67) |
| t27/specs/nn/attention.t27 | ✅ COMPLETE | sacred_attention, d_k^(-φ³) kernel | Skill 018 (f0cf12c) |
| t27/specs/nn/hslm.t27 | ✅ COMPLETE | HSLM, forward, backward, phase | Skill 019 (56c67a9) |
| t27/specs/fpga/mac.t27 | ✅ COMPLETE | ZeroDSP_MAC, LUT, MAC cycle | Skill 021 (e68e1f9) |
| t27/specs/queen/lotus.t27 | ✅ COMPLETE | 6-phase, orchestrate, cell | Skill 022 (3b1cd8c) |
| t27/compiler/codegen/c/codegen.t27 | ✅ COMPLETE | CCodeGen, emit_c, c_ast | Skill 027 (de6c5db) |
| t27/compiler/runtime/runtime.t27 | ✅ COMPLETE | T27Runtime, init, execute | Skill 028 (d8d298d) |
| t27/compiler/parser/lexer.t27 | ✅ COMPLETE | Lexer, tokenize, TokenType | Skill 029 (010a598) |
| t27/compiler/codegen/zig/runtime.t27 | ✅ COMPLETE | Zig runtime generation | Skill 033 (0e989f9) |
| t27/architecture/graph_v2.json | ✅ COMPLETE | new nodes, edges, topological_order | Skill 030 (3ddcffd) |

## Additional standardized files (besides migration)

| File | Action | Key elements | Status |
|-------|-----------|------------------|--------|
| t27/specs/base/types.t27 | ✅ STANDARDIZED | Trit, PackedTrit, TernaryWord | Skill 026 (3173e1a) |
| t27/specs/base/ops.t27 | ✅ STANDARDIZED | trit_multiply, trit_add, trit_carry | Skill 023 (6919cd5) |
| t27/specs/numeric/tf3.t27 | ✅ STANDARDIZED | TF3 encode/decode, TF3 type | Skill 024 (d913ba8) |
| t27/specs/numeric/gf16.t27 | ✅ STANDARDIZED | GF16 encode/decode, phi_round | Skill 025 (c24fd5d) |

## Migration completion criteria

- [x] All nine `.t27` files created and standardized
- [x] All files in canonical format (module/fn/test/invariant/bench)
- [x] graph_v2.json updated (all nodes have "done" status)
- [x] Plan saved as document in `t27/docs/nona-03-manifest/migration-plan-vsa-nn-fpga-queen.md`

## ✅ MIGRATION COMPLETE

All tasks from migration plan completed. PHI LOOP session completed with 17 skills (Skills 017-033).

**Standardization complete:**
- All 14 .t27 specifications in canonical format (module/fn/test/invariant/bench)
- All architecture files synchronized
- Assembly-like (.use/.data/.code) syntax fully replaced

## PHI LOOP Skills Summary

| Skill | Module | Commit | Status |
|-------|--------|--------|--------|
| 017 | specs/vsa/ops.t27 | cafc405 | ✅ COMPLETE |
| 018 | specs/nn/attention.t27 | f0cf12c | ✅ COMPLETE |
| 019 | specs/nn/hslm.t27 | 56c67a9 | ✅ COMPLETE |
| 020 | specs/isa/registers.t27 | 8296d67 | ✅ COMPLETE |
| 021 | specs/fpga/mac.t27 | e68e1f9 | ✅ COMPLETE |
| 022 | specs/queen/lotus.t27 | 3b1cd8c | ✅ COMPLETE |
| 023 | specs/base/ops.t27 | 6919cd5 | ✅ COMPLETE |
| 024 | specs/numeric/tf3.t27 | d913ba8 | ✅ COMPLETE |
| 025 | specs/numeric/gf16.t27 | c24fd5d | ✅ COMPLETE |
| 026 | specs/base/types.t27 | 3173e1a | ✅ COMPLETE |
| 027 | compiler/codegen/c/codegen.t27 | de6c5db | ✅ COMPLETE |
| 028 | compiler/runtime/runtime.t27 | d8d298d | ✅ COMPLETE |
| 029 | compiler/parser/lexer.t27 | 010a598 | ✅ COMPLETE |
| 030 | architecture/graph_v2.json | 3ddcffd | ✅ COMPLETE |
| 031 | architecture/graph.tri | 823a1e9 | ✅ COMPLETE |
| 032 | CANON_DE_ZIGFICATION.md + ADR-001 | ade5ada | ✅ COMPLETE |
| 033 | compiler/codegen/zig/runtime.t27 | 0e989f9 | ✅ COMPLETE |
| 034 | compiler/skill/registry.t27 | f7bf85e | ✅ COMPLETE |
| 035 | compiler/runtime/validation.t27 | 373261d | ✅ COMPLETE |
| 036 | compiler/runtime/commands.t27 | 746e9c3 | ✅ COMPLETE |
| 037 | compiler/cli/spec.t27 | ff0af85 | ✅ COMPLETE |
| 038 | compiler/cli/gen.t27 | b04bb6e | ✅ COMPLETE |
| 039 | compiler/runtime/runtime.t27 | 2fd620a | ✅ COMPLETE |
| 040 | compiler/ast.t27 | d448bc8 | ✅ COMPLETE |
| 041 | compiler/cli/git.t27 | 8018be7 | ✅ COMPLETE |
| 042 | compiler/codegen/testgen.t27 | eccc93e | ✅ COMPLETE |
| 043 | compiler/codegen/verilog/codegen.t27 | 730eaf1 | ✅ COMPLETE |
| 044 | compiler/codegen/zig/codegen.t27 | 7435e2b | ✅ COMPLETE |
| 045 | compiler/parser/parser.t27 | e972f1d | ✅ COMPLETE |
| 046 | parser MemOperand tracking | aa10f07 | ✅ COMPLETE |
| 047 | codegen VSA BIND/BUNDLE | 6aff4a0 | ✅ COMPLETE |
| 048 | testgen verilog TODO | 31f0bc4 | ✅ COMPLETE |
| 049 | verilog codegen TODOs | a3caf16 | ✅ COMPLETE |
| 050 | zig codegen TODOs | da8642f | ✅ COMPLETE |
| 051 | testgen TODOs expansion | a2ddcb0 | ✅ COMPLETE |
| 052 | CANON_DE_ZIGFICATION update | c2ea417 | ✅ COMPLETE |
| 053 | ADR-001 update | cde33b9 | ✅ COMPLETE |
| 054 | migration plan update | 6b67422 | ✅ COMPLETE |
| 055 | CLAUDE.md update | b94ee6d | ✅ COMPLETE |
| 056 | README.md update | 25e040d | ✅ COMPLETE |
| 057 | verilog SVA patterns | e7a8925 | ✅ COMPLETE |

## Next steps

1. ✅ **Update graph_v2.json**: All nodes updated, status "done"
2. ✅ **Architecture files**: CANON_DE_ZIGFICATION.md and ADR-001 updated
3. ✅ **Documentation**: migration-plan, CLAUDE.md, README.md updated
4. ✅ **Verilog SVA patterns**: SystemVerilog assertion patterns documented
5. ⏳ **Verification**: Wait for bootstrap - `tri gen`, `tri test`, `tri verdict --toxic`
6. ⏳ **Optimization**: Generate Zig/C/Verilog from canonical .t27 specifications

**Bootstrap blocker**: tri CLI requires generation, but for generation needs tri CLI. Bootstrap episode needed.
