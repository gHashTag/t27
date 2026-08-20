# ADR-008: parameterised const type declaration `const Name(T) = struct { ... }`

Status: accepted
Date: 2026-08-15
Context: #2162

This ADR fixes the surface syntax and the AST shape BEFORE the parser is changed, so
that the change cannot widen the grammar by accident. Anything not listed as accepted
here is rejected, and there is a negative fixture for each rejection.

## Decision

`pub const Name(T) = struct { ... };` is an accepted **parameterised type
declaration**. The parser accepts it as

    ConstDecl(name, generic_parameters, StructExpr)

and does not require a type identifier after `const`.

This is a parser defect, not a corpus error. The corpus is the evidence of intent:
**33 declarations across 28 files**, every one of them with `struct` on the right-hand
side, and not one instance of any other right-hand side.

## Evidence from the corpus, at `b1884f95`

Parameter lists that actually occur, exhaustively:

| parameter list | occurrences |
|---|---|
| `T` | 22 |
| `K, V` | 4 |
| `W, T` | 1 |
| `T, E` | 1 |
| `S, T` | 1 |
| `R, T` | 1 |
| `L, R` | 1 |
| `A, B, C` | 1 |
| `A, B` | 1 |

Right-hand sides that occur: `struct` × 33. Nothing else.

So the accepted form is narrow by evidence, not by taste: one to three bare
identifiers, comma-separated, and `struct` on the right.

## What is accepted

Grammar, and nothing outside it:

```
ParamConstDecl := [ "pub" ] "const" Ident "(" GenericParams ")" "=" "struct" "{" StructBody "}" [ ";" ]
GenericParams  := Ident { "," Ident }
```

- one or more parameters
- each parameter is a bare `Ident`
- separator is exactly `,`
- the right-hand side is `struct` and only `struct`

## What is rejected, and why each rejection is deliberate

| rejected form | reason |
|---|---|
| `const Name() = struct {}` | an empty parameter list is not a parameterised type; if it is meant as a plain type it should be written without parentheses. Ambiguous, so refused |
| `const Name(T,) = struct {}` | trailing comma is not attested in the corpus. Accepting it is a grammar widening with no evidence behind it |
| `const Name([]T) = struct {}` | a type expression in a parameter position. Parameters are names being bound, not types being used |
| `const Name(T: Trait) = struct {}` | constrained parameters are a language feature, not a parser detail. Out of scope, and accepting the syntax now would commit the language to it |
| `const Name(T) = enum(i8) {}` | not attested. `enum` already has its own accepted form without parameters, and combining the two is a separate decision |
| `const Name(T) = 42;` | a parameterised value is not a type declaration |
| `const Name(T);` | a declaration with no right-hand side |

## `Name(T)` versus function-like syntax

The two are distinguished **by position and by the token after the closing paren**, not
by lookahead over the parameter list:

- `const Name(...) = struct` — reached from `parse_const_decl` after `const` and a name.
  `(` here opens a generic parameter list
- `fn name(...)` — reached from `parse_fn_decl`, a different entry point entirely, where
  `(` opens a value parameter list with `name: Type` pairs
- `Name(args)` as an expression — reached from expression parsing, never from
  `parse_const_decl`'s name position

There is therefore no ambiguity to resolve: the const path never sees a call expression
in that position, and a value parameter list (`x: u8`) is rejected here by the
`Ident { "," Ident }` rule, which admits no colon.

## Accepted terminators

After the closing `}` of the struct body, both `;` and no `;` are accepted, matching the
existing non-parameterised `const Name = struct { ... }` path exactly. Nothing new is
introduced: whatever that path accepts, this path accepts.

## AST shape

- `Node.kind` becomes `NodeKind::StructDecl`, exactly as the non-parameterised form does.
  The declaration is a struct declaration; being parameterised does not change its kind
- `Node.name` is the declared name, WITHOUT the parameter list. `Stack(T)` has
  `name = "Stack"`
- `Node.params` carries the parameters as `(name, "")` pairs, reusing the existing
  `Vec<(String, String)>` field. The second element is the empty string because a
  parameter has no type: it IS a type. No new field is added to `Node`
- struct body children are parsed by the existing `parse_struct_body`, so field nodes are
  indistinguishable from those of a non-parameterised struct

`params` being non-empty is what marks the declaration as parameterised. Nothing else in
the AST changes.

## Codegen is deliberately NOT changed

This ADR changes parsing only. A parameterised type has no single instantiation and the
Verilog backend has no notion of one, so no lowering is defined here.

This has a consequence that must be measured rather than assumed: these 28 files
previously failed to parse **as whole files**, so every declaration in them was invisible.
After this change they parse, and the backend will see declarations it has never seen.
Whether the result is correct emission, harmless emission, or wrong emission is an open
question and is the subject of the two-mode differential in the same tick. It is not
claimed here to be safe.

## What is not claimed

- Not that parameterised types are implemented. They parse; they do not instantiate
- Not that the 28 files now compile. Parsing is the only claim
- Not that this is the whole of #2162's population. The earlier figure of ten files was
  the count of files whose FIRST failing construct was this one, measured on the repaired
  corpus. The real population is 28 files and 33 declarations, and the difference is
  exactly why a first-failure count must never be reported as a population
