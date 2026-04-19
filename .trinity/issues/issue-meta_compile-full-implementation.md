# Issue: feat(compiler): Full meta_compile.t27 multi-backend codegen implementation

## Status
- **Status**: OPEN
- **Ring**: 005 (Codegen)
- **Priority**: HIGH
- **Estimate**: 4-8 hours

## Problem

The `specs/compiler/meta_compile.t27` specification was added with PRs #529 and #531, defining a multi-backend compilation system with 5 target backends (Zig, C, Verilog, Rust, TypeScript). However, current implementations are just stub functions that count newlines - they don't actually generate working code for any backend.

### Current State

**In `specs/compiler/meta_compile.t27`:**
- ✅ `CompileResult` struct with all 5 backend fields
- ✅ Stub implementations: `emit_zig()`, `emit_c()`, `emit_verilog()`, `emit_rust()`, `emit_typescript()`
- ✅ Helper functions: `is_full_success()`, `total_lines()`, `any_backend_ok()`
- ✅ 36 tests and invariants

**In `gen/compiler/` (generated output):**
- ✅ `meta_compile.zig` - Zig stub (line-counting only)
- ✅ `meta_compile.c` - C stub (line-counting only)
- ✅ `meta_compile.v` - Verilog stub (line-counting only)
- ❌ `meta_compile.rs` - Missing
- ❌ `meta_compile.ts` - Missing

**In `bootstrap/`:**
- ❌ No integration with `meta_compile.t27` spec
- ❌ Existing `emit_verilog()` in `compiler.rs` is for testbench emission, unrelated to meta_compile spec
- ❌ No TypeScript or Rust code generation integration

## Scope

Implement real code generation for all 5 backends defined in `meta_compile.t27`:

1. **Zig Backend** - Generate syntactically valid Zig code from T27 AST
2. **C Backend** - Generate valid C code with proper type mapping (φ → `uint16_t`, i32 → `int32_t`, etc.)
3. **Verilog Backend** - Generate hardware code for synthesis (modules, wires, regs, always blocks)
4. **Rust Backend** - Generate Rust code with proper type annotations
5. **TypeScript Backend** - Generate TS code for web/browser target (φ → `number`, i32 → `number`, etc.)

## Implementation Plan

### Phase 1: Design Codegen Architecture
1. Review existing parser and AST structures in `bootstrap/src/compiler.rs`
2. Define codegen patterns for each backend target
3. Design integration point between parser output and multi-backend codegen

### Phase 2: Implement Target-Specific Codegen

#### 2.1 Zig Backend
- Implement actual Zig code emission from T27 AST
- Generate syntactically valid Zig code for:
  - Module declarations
  - Function declarations with T27 types (φ, u32, i32, etc.)
  - Expression statements (literals, calls, arithmetic, logic)
  - Control flow (if, while, for)
  - φ arithmetic operations

#### 2.2 C Backend
- Implement C code emission from T27 AST
- Map T27 types to C types (φ → `uint16_t`, i32 → `int32_t`, etc.)
- Generate valid C code for all AST node types
- Include appropriate headers and type definitions

#### 2.3 Verilog Backend
- Implement Verilog code emission for hardware target
- Map T27 constructs to Verilog (modules, wires, regs, always blocks)
- Generate testbench-compatible output
- Handle φ encoding in hardware (2 trits per φ)

#### 2.4 Rust Backend
- Implement Rust code emission from T27 AST
- Map T27 types to Rust types
- Generate valid Rust with proper type annotations
- Handle φ operations using existing GF16 support

#### 2.5 TypeScript Backend
- Implement TypeScript code emission for web/browser target
- Map T27 types to TS types (φ → `number`, i32 → `number`, etc.)
- Generate valid TypeScript declarations and functions
- Handle φ arithmetic as JS Number operations

### Phase 3: Bootstrap Integration
1. Add `meta_compile` module integration to `bootstrap/src/compiler.rs`
2. Wire parser output to `meta_compile` backends
3. Add CLI flags for target backend selection (`--backend zig|c|verilog|rust|ts|all`)
4. Update `main.rs` to handle backend selection

### Phase 4: Testing
1. Run existing `tri test` suite for `meta_compile.t27`
2. Add integration tests for generated code compilation
3. Verify generated Zig/C/Verilog/Rust/TypeScript actually compiles with their respective compilers
4. Test cross-compilation from T27 source to all backends

### Phase 5: Documentation
1. Update `docs/NOW.md` with issue closure
2. Document multi-backend usage in README.md
3. Add examples showing cross-compilation to different targets
4. Document φ type representation in each target language

## Critical Files

**Spec:**
- `specs/compiler/meta_compile.t27` - Update stub implementations with real codegen

**Bootstrap (to modify):**
- `bootstrap/src/compiler.rs` - Add meta_compile integration
- `bootstrap/src/main.rs` - Add CLI flags for backend selection

**Generated (verify after tri gen):**
- `gen/compiler/meta_compile.zig` - Should be generated from spec
- `gen/compiler/meta_compile.c` - Should be generated from spec
- `gen/compiler/meta_compile.v` - Should be generated from spec
- `gen/compiler/meta_compile.rs` - To be created
- `gen/compiler/meta_compile.ts` - To be created

## Verification Checklist

- [ ] All 5 backends generate valid, compilable code
- [ ] Run `./scripts/tri gen compiler/meta_compile.t27` generates all backends
- [ ] Run `./scripts/tri test compiler/meta_compile.t27` passes
- [ ] Manually test: compile a simple T27 module to Zig (`zig build`)
- [ ] Manually test: compile a simple T27 module to C (`gcc`)
- [ ] Manually test: compile a simple T27 module to Verilog (verilator/iverilog)
- [ ] Manually test: compile a simple T27 module to Rust (`cargo build`)
- [ ] Manually test: compile a simple T27 module to TypeScript (tsc)
- [ ] CLI `--backend` flag works for selecting individual backends
- [ ] CLI `--backend all` compiles to all backends
- [ ] Documentation updated in README and NOW.md

## Related Issues

- Closes: TBD (new issue number)
- Related: #519 (GF16 Rust codegen - closed)
- Related: #525 (TypeScript codegen spec - closed via PR #529)
- Related: #530 (All codegen backends in meta_compile spec - closed via PR #531)

## Notes

- This is a multi-hour task (4-8 hours estimated)
- The existing stub implementations in `meta_compile.t27` should be replaced with real codegen logic
- Bootstrap needs new CLI options for backend targeting
- Consider making backends pluggable for future targets (e.g., WASM, Python)
- φ type representation varies by target: uint16_t in C, u16 in Zig, u16 in Rust, number in TS

---

**Created**: 2026-04-19
**Ring**: 005
