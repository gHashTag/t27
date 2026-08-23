# ADR-009: Generic type application in type positions

Status: Accepted
Date: 2026-08-22
Decision authority: language owner

## Context

The language owner accepted `Name(T)` as a generic type application in a type
position. The parser had already accepted parameterised type declarations such
as `const Name(T) = struct`, but it did not retain applications when those
declared types appeared in function parameter or return annotations.

This ADR records the contract post hoc following the language owner decision of
2026-08-22. The implementation is intentionally made before this record rather
than waiting for a prior ADR.

## Decision

In a type position, an identifier followed by parenthesised type arguments is a
generic type application. The accepted forms include:

```t27
fn empty() -> List(void) {}
fn add(set: *HashSet(T)) -> void {}
fn is_left(either: Either(L, R)) -> void {}
```

`Name(T)` in a type position differs from an expression call. The type parser
has already entered an annotation or return-type grammar before it reads
`Name(T)`, so it records a type application. In an expression position, the
expression parser continues to interpret the same token sequence as a call.
This decision does not make calls into types outside type grammar.

`*Name(T)` binds as a pointer to the application `Name(T)`. The pointer prefix
is consumed first and its pointee is parsed through the type grammar; it is not
an application of `T` to the pointer `*Name`.

Existing numeric type arguments remain accepted and retain their spelling:
`P(2)`, `Z(1)`, and `N(0)`. They are a regression risk because a type-argument
parser that accepts identifiers only would silently narrow the existing type
surface.

## Consequences

The parser preserves the complete annotation text for generic applications,
including nested type syntax handled by the shared type parser and numeric
arguments. Tests assert AST type text for the accepted type applications rather
than parse success alone.
