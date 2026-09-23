# Rust Syntax Analysis Report

## Issue Summary
This documents the findings from gHashTag/t27#2398 regarding files with Rust syntax in `.t27` extensions that the T27 compiler cannot parse.

## Boundary File Analysis

### File: specs/ternary/hybrid_bigint.t27

**Status**: Contains extensive Rust syntax that T27 compiler does not support

**Rust Syntax Found**:
- `let mut` declarations (multiple occurrences)
- `for i in 0..N` loops
- `impl` blocks for struct methods
- Array types: `[T; N]` syntax
- Generic types: `Option<[MAX_TRITS]Trit>`
- Method signatures with `&mut self`
- Tuple return types: `-> (Vec32, Vec32)`
- Match expressions in tests

**Compiler Results**:
- `t27c parse`: **FAILED** - "parse error at module level near line 10: unexpected token after expression statement: Ident"
- `t27c typecheck`: **FAILED** - Same parse error

## Context from Issue
- 43 specs use `let` anyway (1352 occurrences)
- 39 specs use `match` 
- 66 files contain at least one Rust marker
- Only 1 of 66 files with Rust markers is AST-check valid
- 21.6% of files (431/431) are pure t27 and valid
- 1.5% of files (1/66) with Rust markers are valid
- Files with multiple Rust markers have 0% validity rate

## Recommendation
This file and others like it cannot be fixed by emitter work alone. The fundamental issue is that the T27 compiler's lexer lacks support for Rust keywords (`KwLet`, `KwMatch`) and syntax constructs. These files appear to be Rust implementations parked in the specs directory, possibly as reference implementations or unfinished work.

The decision of whether to:
1. Convert these files to valid t27 syntax
2. Move them out of the specs directory
3. Keep them as-is with documentation

...should be made with the understanding that they currently represent 6-13% of the corpus and cannot be compiled by the T27 compiler.

Generated: 2025-06-20