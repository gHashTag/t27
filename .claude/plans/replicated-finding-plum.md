# Issue: Full meta_compile.t27 Implementation

## Context

The `specs/compiler/meta_compile.t27` specification was added with PRs #529 and #531, defining a multi-backend compilation system with 5 target backends (Zig, C, Verilog, Rust, TypeScript). However, the current implementations are just stub functions that count newlines - they don't actually generate working code for any backend.

**Problem**: The spec exists but has no functional implementation. Bootstrap codegen doesn't integrate with `meta_compile.t27` at all.

## Current State

**In `specs/compiler/meta_compile.t27`:**
- ✅ `CompileResult` struct with all 5 backend fields
- ✅ Stub implementations: `emit_zig()`, `emit_c()`, `emit_verilog()`, `emit_rust()`, `emit_typescript()`
- ✅ Helper functions: `is_full_success()`, `total_lines()`, `any_backend_ok()`
- ✅ 36 tests and invariants

**In `gen/compiler/` (generated output):**
- ✅ `meta_compile.zig` - Zig stub (line-counting only)
- ✅ `meta_compile.c` - C stub (line-counting only)
- ✅ `meta_compile.v` - Verilog stub (line-counting only)

**In `bootstrap/`:**
- ❌ No integration with `meta_compile.t27` spec
- ❌ Existing `emit_verilog()` in `compiler.rs` is for testbench emission, unrelated to spec
- ❌ No TypeScript or Rust code generation for `meta_compile` spec

## Implementation Plan

### Phase 1: Design Real Codegen Architecture
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

#### 2.3 Verilog Backend
- Implement Verilog code emission for hardware target
- Map T27 constructs to Verilog (modules, wires, regs, always blocks)
- Generate testbench-compatible output

#### 2.4 Rust Backend
- Implement Rust code emission from T27 AST
- Map T27 types to Rust types
- Generate valid Rust with proper type annotations

#### 2.5 TypeScript Backend
- Implement TypeScript code emission for web/browser target
- Map T27 types to TS types (φ → `number`, i32 → `number`, etc.)
- Generate valid TypeScript declarations and functions

### Phase 3: Bootstrap Integration
1. Add `meta_compile` module integration to `bootstrap/src/compiler.rs`
2. Wire parser output to `meta_compile` backends
3. Add CLI flags for target backend selection
4. Add `--all-backends` flag to compile to all targets

### Phase 4: Testing
1. Run existing `tri test` suite for `meta_compile.t27`
2. Add integration tests for generated code compilation
3. Verify generated Zig/C/Verilog/Rust/TypeScript actually compiles

### Phase 5: Documentation
1. Update `docs/NOW.md` with issue closure
2. Document multi-backend usage in README
3. Add examples showing cross-compilation to different targets

## Critical Files

**Spec:**
- `/Users/playra/t27/specs/compiler/meta_compile.t27`

**Bootstrap (to modify):**
- `/Users/playra/t27/bootstrap/src/compiler.rs` - Add meta_compile integration
- `/Users/playra/t27/bootstrap/src/main.rs` - Add CLI flags for backend selection

**Generated (verify after tri gen):**
- `/Users/playra/t27/gen/compiler/meta_compile.zig`
- `/Users/playra/t27/gen/compiler/meta_compile.c`
- `/Users/playra/t27/gen/compiler/meta_compile.v`
- `/Users/playra/t27/gen/compiler/meta_compile.rs` (to be created)
- `/Users/playra/t27/gen/compiler/meta_compile.ts` (to be created)

## Verification

1. Run `./scripts/tri gen compiler/meta_compile.t27` to generate code
2. Run `./scripts/tri test compiler/meta_compile.t27` to verify conformance
3. Manually test: compile a simple T27 module to each backend
4. Verify generated Zig/C/Verilog/Rust/TypeScript files compile with their respective compilers
5. Close issue with `Closes #<issue>` in commit message

## Notes

- This is a multi-hour task involving both spec updates and bootstrap changes
- The existing stub implementations in `meta_compile.t27` should be replaced with real codegen logic
- Bootstrap needs new CLI options for backend targeting
- Consider making backends pluggable for future targets (e.g., WASM, Python)
