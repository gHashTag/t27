# RFC: .tri Language Core Specification

**Status:** Draft
**Ring:** Ring 01 - Foundation
**Author:** Trinity RFC
**Created:** 2026-04-16
**Related:** [Parameter Golf](https://github.com/openai/parameter-golf)

---

## Abstract

`.tri` (Trinity Intermediate Representation) — is the canonical IR language for ternary/φ-optimized calculations within the Trinity S³AI framework. It serves as the bridge between human-readable specifications `.t27` and binary backends (Zig, Rust, Verilog), ensuring a single source of truth for compilation, testing, and verification.

---

## Motivation

### Why is `.tri` needed as a separate language?

1. **Single source of truth violation**: In the current project, logic is duplicated between `.t27`, the Zig backend, and Runtime. This violates the Trinity "single source of truth" principle.

2. **Ternary-native semantics**: Trinity is optimized under φ² = φ + 1 for ternary logic. `.tri` must express this semantics natively, rather than translating from binary.

3. **Experience integration**: Agent-based learning requires built-in hooks in the IR, so that trace is reproducible without additional tooling.

4. **Parameter Golf insights**: Techniques like QAT, GPTQ embeddings, weight equalization (MuonEq-R) work at type/representation levels, not via translation.

---

## Core Language Design

### Lexical Structure

`.tri` uses ASCII-only lexicon with English identifiers (L3 purity law).

#### Comments

```tri
// Single-line comment
// That explains a single line

/* Multi-line comment
   that spans multiple lines
*/
```

#### Identifiers

```tri
// Variables and parameters
let variable_name = value;

// Constants
pub const CONSTANT_NAME: type = value;

// Functions
pub fn function_name(param: type) -> return_type;

// Built-in Types
pub type Bool = Trit; // -1 = false, 0 = unknown, +1 = true

// Fixed-size arrays
pub type Vector3 = [3]f64;
pub type Matrix4x4 = [4][4]f64;

// Dynamic arrays
pub type DynamicBytes = [_]u8{0};

// Generic types
pub type Option(T) = struct {
    some: T,
    none: Trit
};
```

### Type System

#### Built-in Types

```tri
// Core ternary type (native to Trinity)
pub const Trit = enum(i8) {
    neg = -1,
    neu = 0,
    pos = 1
}

// Boolean (derived from Trit)
pub type Bool = Trit; // -1 = false, 0 = unknown, +1 = true

// Fixed-size arrays
pub type Vector3 = [3]f64;
pub type Matrix4x4 = [4][4]f64;

// Dynamic arrays
pub type DynamicBytes = [_]u8{0};

// Generic types
pub type Option(T) = struct {
    some: T,
    none: Trit
};
```

#### Built-in Types

```tri
// Core ternary type (native to Trinity)
pub const Trit = enum(i8) {
    neg = -1,
    neu = 0,
    pos = 1
}

// Boolean (derived from Trit)
pub type Bool = Trit; // -1 = false, 0 = unknown, +1 = true

// Fixed-size arrays
pub type Vector3 = [3]f64;
pub type Matrix4x4 = [4][4]f64;

// Dynamic arrays
pub type DynamicBytes = [_]u8{0};

// Generic types
pub type Option(T) = struct {
    some: T,
    none: Trit
};
```

---

## Type System

### Phi-Structured Types (Trinity Optimization)

All Trinity-optimized types follow the φ-proportional exp/mantissa layout:

| Format | Exp Bits | Mant Bits | φ-distance | Use Case |
|--------|-----------|------------|----------------|----------|
| GF4 | 1 | 2 | 0.118 | Binary masks |
| GF8 | 3 | 4 | 0.132 | Weights |
| GF12 | 4 | 7 | 0.047 | Attention |
| GF16 | 6 | 9 | 0.049 | Primary |
| GF20 | 7 | 12 | 0.035 | Training |
| GF24 | 9 | 14 | 0.025 | Precision |
| GF32 | 12 | 19 | 0.014 | Scientific |
