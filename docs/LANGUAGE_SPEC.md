# T27 Language Specification

**Version:** 0.1.0  
**Date:** 2026-05-09  
**Status:** Draft (Ring 038)  
**Constitutional basis:** `SOUL.md` **Article VIII**, `docs/T27-CONSTITUTION.md`

---

## 1. Introduction

T27 is a **spec-first** programming language for ternary computing. The language is defined by `.t27` specification files, from which multiple backends (Zig, C, Verilog, Rust, TypeScript) are generated.

### 1.1 Design Goals

1. **Spec-first:** All language semantics are defined in `.t27` specs
2. **Ternary-native:** Optimized for ternary computation with trits `{-1, 0, +1}`
3. **Multi-backend:** Single source generates multiple target languages
4. **Formally verifiable:** Support for invariants and test blocks
5. **φ-optimized:** GoldenFloat numeric family aligned with golden ratio

### 1.2 Reference Implementations

- **Bootstrap compiler:** `bootstrap/src/compiler.rs` (Rust)
- **Runtime:** `bootstrap/target/release/t27c`
- **CLI:** `scripts/tri`

---

## 2. Lexical Grammar

### 2.1 Character Set

T27 source files are **ASCII-only** (see ADR-004). The character set includes:

- Uppercase letters: `A-Z`
- Lowercase letters: `a-z`
- Digits: `0-9`
- Operators: `+`, `-`, `*`, `/`, `%`, `=`, `<`, `>`, `!`, `&`, `|`, `^`, `~`, `@`, `:`, `.`, `,`, `;`, `(`, `)`, `[`, `]`, `{`, `}`
- Whitespace: space, tab, newline, carriage return

### 2.2 Comments

```t27
# Single-line comment
```

Comments extend from `#` to the end of the line.

### 2.3 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```
identifier → (letter | "_") (letter | digit | "_")*
```

Examples: `x`, `my_var`, `Gf16Encode`, `_temp`

### 2.4 Keywords

Reserved words in T27:

```
pub, use, mod, fn, type, struct, enum, const, let, mut,
if, else, match, case, for, while, loop, return, break, continue,
test, invariant, bench, pub, priv, unsafe, extern
```

### 2.5 Literals

**Integer literals:**
```t27
0, 1, 42, -7, 0xFF (hex), 0o77 (octal), 0b1010 (binary)
```

**Float literals (GF16/GF32):**
```t27
0.0, 1.618, 3.14159, -0.618, 1e-10, 6.674e-11
```

**String literals:**
```t27
"hello world", "escaped \"quote\""
```

**Trit literals (ternary):**
```t27
-1, 0, +1  # Representing false, unknown, true
```

---

## 3. Context-Free Grammar

### 3.1 Modules

```t27
# Module declaration
mod module_name {
    # Module contents
}
```

### 3.2 Imports

```t27
use other_module::Type;
use other_module::function;
use other_module::{Type1, Type2};
```

### 3.3 Type Definitions

```t27
# Primitive types
type MyInt = u32;
type MyFloat = GF16;

# Struct types
type Point = struct {
    x: f64,
    y: f64,
};

# Enum types
type Option = enum {
    Some(T),
    None,
};
```

### 3.4 Functions

```t27
# Function declaration
fn add(a: GF16, b: GF16) -> GF16 {
    return a + b;
}

# Public function
pub fn public_func() -> void {
    # ...
}

# Generic function (planned)
fn generic<T>(x: T) -> T {
    return x;
}
```

### 3.5 Constants

```t27
# Module-level constant
pub const PHI: GF16 = 1.618033988749895;

pub const TRINITY: GF16 = 3.0;
```

### 3.6 Control Flow

**If-else:**
```t27
if condition {
    # then branch
} else {
    # else branch
}
```

**Match:**
```t27
match value {
    case 0 => { # zero case },
    case 1 => { # one case },
    case _ => { # default case },
}
```

**Loops:**
```t27
# While loop
while condition {
    # body
}

# For loop
for i in 0..10 {
    # body
}
```

---

## 4. Type System

### 4.1 Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `u8`, `u16`, `u32`, `u64` | Unsigned integers | 8, 16, 32, 64 bits |
| `i8`, `i16`, `i32`, `i64` | Signed integers | 8, 16, 32, 64 bits |
| `GF4`, `GF8`, `GF12`, `GF16`, `GF20`, `GF24`, `GF32` | GoldenFloat formats | 4-32 bits |
| `TF3` | Ternary float | 3 trits |
| `bool` | Boolean | 1 bit |
| `void` | Unit type | 0 bits |
| `Trit` | Trit value | 2 bits (-1, 0, +1) |

### 4.2 Composite Types

**Arrays:**
```t27
# Fixed-size array
type Pixels = [GF16; 100];
```

**Structs:**
```t27
type Vector3 = struct {
    x: GF16,
    y: GF16,
    z: GF16,
};
```

### 4.3 Type Inference

T27 supports limited type inference for local variables:

```t27
let x = 1.0;          # inferred as GF16
let y: GF32 = 1.0;    # explicit GF32
```

### 4.4 Type Checking

T27 uses **static type checking**. Type errors are caught at compile time (spec parse time).

**Type coercion rules:**
- No implicit coercion between integer types
- No implicit coercion between GF formats
- Explicit casts required: `GF16(x)`, `u32(x)`

---

## 5. Operational Semantics

### 5.1 Evaluation Order

T27 uses **strict evaluation** (eager evaluation). Expressions are evaluated left-to-right.

### 5.2 Expression Evaluation

```t27
# Arithmetic
a + b * c  # Multiplies first, then adds

# Comparison
a == b && c > d  # Short-circuit evaluation

# Function call
f(x, y)  # Arguments evaluated left-to-right
```

### 5.3 Side Effects

Side effects occur in the following order:
1. Function argument evaluation
2. Function call
3. Assignment

### 5.4 Backend Mapping

T27 expressions map to backend operations:

| T27 Expression | Zig | C | Verilog | Rust | TypeScript |
|----------------|-----|---|---------|------|------------|
| `a + b` | `a + b` | `a + b` | `assign c = a + b;` | `a + b` | `a + b` |
| `a * b` | `a * b` | `a * b` | `assign c = a * b;` | `a * b` | `a * b` |
| `if c { x } else { y }` | `if (c) x else y` | `if (c) x; else y;` | `if (c) begin x; end else begin y; end` | `if c { x } else { y }` | `if (c) { x } else { y }` |

---

## 6. Testing and Verification

### 6.1 Test Blocks

```t27
test "addition works" {
    let a: GF16 = 1.0;
    let b: GF16 = 2.0;
    let result: GF16 = a + b;
    assert result == 3.0;
}
```

### 6.2 Invariant Blocks

```t27
invariant "phi identity" {
    let phi: GF16 = 1.618033988749895;
    let phi_sq: GF16 = phi * phi;
    let phi_inv: GF16 = 1.0 / phi;
    let result: GF16 = phi_sq + (phi_inv * phi_inv);
    assert abs(result - 3.0) < 0.001;
}
```

### 6.3 Benchmark Blocks

```t27
bench "addition performance" {
    # Benchmark code
    # ...
}
```

### 6.4 Conformance Vectors

Each spec must have corresponding conformance vectors in `conformance/`:

```json
{
  "schema_version": "1.0",
  "spec": "specs/numeric/gf16.t27",
  "tests": [
    {
      "name": "add_positive",
      "input": {"a": 1.0, "b": 2.0},
      "expected": 3.0,
      "tolerance": 0.001
    }
  ]
}
```

---

## 7. Backend Contract

See `docs/BACKEND_CONTRACT.md` for detailed backend obligations.

### 7.1 Required Backend Operations

Each backend MUST implement:

1. **All primitive types** with correct bit widths
2. **All arithmetic operations** with correct semantics
3. **Control flow** with correct semantics
4. **Function calls** with proper calling convention
5. **Type safety** (no unsafe operations without `unsafe` keyword)

### 7.2 Generated Code Headers

All generated files MUST include:

```
// AUTO-GENERATED by t27c from spec: <spec_path>
// DO NOT EDIT — changes will be overwritten
// Generated: <timestamp>
// Commit: <git commit hash>
```

---

## 8. Invariants and Safety

### 8.1 L5 IDENTITY Invariant

```t27
invariant "L5_TRINITY_IDENTITY" {
    # φ² + φ⁻² = 3
    let phi: GF16 = (1.0 + sqrt(5.0)) / 2.0;
    let result: GF16 = phi * phi + (1.0 / phi) * (1.0 / phi);
    assert abs(result - 3.0) < 1e-6;
}
```

### 8.2 L8 FPGA-Safety Invariant

No IEEE f64/f32 arithmetic in core FPGA paths. Enforced by CI:

```yaml
- name: FPGA-Safety lint (L8)
  run: |
    grep -rn "as f64\|: f64" ffi/src/ || echo "L8 PASSED"
```

### 8.3 Type Safety

- No null pointer dereferences (null type not in core)
- No undefined behavior (all operations defined)
- No implicit type coercion

---

## 9. Soundness Results

### 9.1 Proven Theorems

| Theorem | Status | Proof Location |
|---------|--------|----------------|
| THM-001: φ² + φ⁻² = 3 | PROVEN | `specs/math/constants.t27` |
| THM-009: exp/mant = 1/phi optimality | PROVEN | `specs/numeric/phi_ratio.t27` |
| L5 IDENTITY | PROVEN | `conformance/phi_identity_vectors.json` |

### 9.2 Conjectures

| Claim | Status | Evidence |
|-------|--------|----------|
| C-gf-006: Attention in GF16 preserves quality | CONJECTURE | BENCH-005 (3x better perplexity) |
| C-gf-007: VSA ops in GF16 | CONJECTURE | Empirical results |

---

## 10. Extensions and Future Work

### 10.1 Planned Features

- **Generics:** Type parameters and type-level computation
- **Traits:** Interface definitions and impl blocks
- **Macros:** Compile-time metaprogramming
- **Async:** Asynchronous operations and await syntax
- **Modules:** Proper module system with visibility

### 10.2 Research Features

- **Dependent types:** Type-level natural numbers
- **Linear types:** Resource management via type system
- **Effect systems:** Side effect tracking

---

## 11. References

- `SOUL.md` — Constitutional law
- `docs/T27-CONSTITUTION.md` — L1-L7 invariant laws
- `docs/nona-02-organism/NUMERIC-STANDARD-001.md` — GoldenFloat specification
- `docs/nona-03-manifest/RESEARCH_CLAIMS.md` — Claims registry
- `docs/CONFORMANCE_TRACEABILITY.md` — Conformance mapping
- `docs/SPECS_BOUNDARY.md` — Core vs research spec boundary

---

## Appendix A: Complete Grammar (BNF-like)

```
program        → module_declaration*
module_declaration → "mod" identifier "{" module_item* "}"
module_item    → use_decl | type_decl | fn_decl | const_decl | test_block | invariant_block | bench_block

use_decl       → "use" identifier "::" (identifier | "{" identifier ("," identifier)* "}") ";"
type_decl      → "type" identifier "=" type ";"
type           → primitive_type | struct_type | enum_type | array_type | identifier
primitive_type → "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "GF4" | "GF8" | "GF12" | "GF16" | "GF20" | "GF24" | "GF32" | "TF3" | "bool" | "void" | "Trit"
struct_type    → "struct" "{" field_list "}"
field_list     → (identifier ":" type ("," identifier ":" type)*)?
enum_type      → "enum" "{" enum_variant ("," enum_variant)* "}"
enum_variant   → identifier "(" type? ")" | identifier
array_type     → "[" type ";" expression "]"

fn_decl        → "pub"? "fn" identifier "(" param_list? ")" ("->" type)? block
param_list     → (identifier ":" type ("," identifier ":" type)*)?
block          → "{" statement* "}"
statement      → let_decl | assign_stmt | if_stmt | while_stmt | for_stmt | return_stmt | expression ";"
let_decl       → "let" ("mut")? identifier (":" type)? "=" expression ";"
assign_stmt    → identifier "=" expression ";"
if_stmt        → "if" expression block ("else" block)?
while_stmt     → "while" expression block
for_stmt       → "for" identifier "in" expression ".." expression block
return_stmt    → "return" expression? ";"

expression     → logical_or
logical_or     → logical_and ("||" logical_and)*
logical_and    → comparison ("&&" comparison)*
comparison     → term (("==" | "!=" | "<" | "<=" | ">" | ">=") term)*
term           → factor (("+" | "-") factor)*
factor         → power (("*" | "/" | "%") power)*
power          -> unary ("^" unary)?
unary          -> ("-" | "!" | "~")? primary
primary        -> literal | identifier | "(" expression ")" | primary "." identifier | primary "[" expression "]" | primary "(" arg_list? ")"
arg_list       -> expression ("," expression)*

literal        -> integer_literal | float_literal | string_literal | trit_literal
test_block     -> "test" string_literal block
invariant_block -> "invariant" string_literal block
bench_block    -> "bench" string_literal block
```

---

**φ² + 1/φ² = 3 | TRINITY**
