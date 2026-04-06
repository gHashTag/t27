# OWNERS — compiler/

## Primary

**C-Compiler** — `.t27` compiler specifications (lexer, parser, types, codegen specs) that describe the language implementation.

## Dependencies

- `specs/compiler/`, `specs/base/` — language SSOT.
- **L-Lexer** and **C-Compiler** agent roles both touch parser/lexer paths here; **C-Compiler** is primary for merge conflicts.

## Outputs

Spec-driven definitions consumed by `bootstrap/src/compiler.rs` and future self-host stages.
