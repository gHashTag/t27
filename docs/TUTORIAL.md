# t27 Tutorial: Complete Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [T27 Language Basics](#t27-language-basics)
4. [GoldenFloat Formats](#goldenfloat-formats)
5. [RTL Generation](#rtl-generation)
6. [Coq Formal Verification](#coq-formal-verification)
7. [Advanced Topics](#advanced-topics)

---

## Introduction

t27 is a **spec-first ternary floating-point specification language** designed for the Trinity hardware ecosystem. It combines:

- **Phi optimization**: Formats designed using φ-ratio (exp/mantissa ≈ 1/φ = 0.618)
- **3-structured design**: Built on the sacred constants φ, e, and γ
- **Formal verification**: Coq proofs for correctness
- **RTL generation**: Automatic Verilog synthesis for FPGAs

### Key Concepts

- **GF (GoldenFloat)**: φ-optimized floating-point formats (GF4-GF256)
- **T27 spec files**: `.t27` extension, declarative specifications
- **3-sacred constants**: φ (1.618...), e (2.718...), γ (0.577...)

---

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/gHashTag/t27.git
cd t27

# Build the compiler
cd bootstrap
cargo build --release

# Verify installation
./scripts/tri --version
```

### Your First Spec

Create `hello.t27`:

```t27
module HelloWorld {
    // Define a simple GF16 constant
    const PI_GF16 : gf16 = gf16.from_f64(3.1415926535);

    // Define a test
    test pi_roundtrip
        given pi = PI_GF16
        and   f64_val = gf16.to_f64(pi)
        then abs(f64_val - 3.14) < 0.01
}
```

Compile and run:

```bash
./scripts/tri gen hello.t27
./scripts/tri test
```

---

## T27 Language Basics

### Module Structure

```t27
module ModuleName {
    // Imports
    use math::constants;
    use numeric::gf16;

    // Constants
    const PI : f64 = 3.14159;

    // Functions
    fn add(a: f64, b: f64) -> f64 {
        return a + b;
    }

    // Structs
    struct Point {
        x : f64,
        y : f64,
    }

    // Tests
    test add_positive
        given a = 5.0
        and   b = 3.0
        and   result = add(a, b)
        then result == 8.0

    // Invariants
    invariant add_commutative
        assert add(2.0, 3.0) == add(3.0, 2.0);
}
```

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `i8, i16, i32, i64` | Signed integers | `42` |
| `u8, u16, u32, u64` | Unsigned integers | `0xFF` |
| `f32, f64` | Floating point | `3.14` |
| `gf4, gf8, gf16` | GoldenFloat formats | `gf16.one` |
| `bool` | Boolean | `true` |
| `Trit` | Ternary (neg, zero, pos) | `.pos` |

### Control Flow

```t27
if (condition) {
    // then branch
} else {
    // else branch
}

while (i < 10) {
    i = i + 1;
}

for (array) |item| {
    // process item
}
```

---

## GoldenFloat Formats

### Format Comparison

| Format | Bits | Exp | Mant | φ-distance | Use Case |
|--------|------|-----|------|------------|----------|
| GF4 | 4 | 1 | 2 | 0.118 | Ultra-compact |
| GF8 | 8 | 3 | 4 | 0.132 | Low-power edge |
| GF16 | 16 | 6 | 9 | 0.049 | **PRIMARY** |
| GF32 | 32 | 12 | 19 | 0.013 | High-precision |
| GF64 | 64 | 24 | 39 | 0.003 | Extended precision |
| GF128 | 128 | 48 | 79 | 0.010 | Scientific computing |
| GF256 | 256 | 97 | 158 | 0.004 | Ultra-high precision |

### Using GF16

```t27
module GF16Example {
    use numeric::gf16;

    // Create GF16 values
    const one : gf16 = gf16.from_f64(1.0);
    const pi  : gf16 = gf16.from_f64(3.14159);

    // Arithmetic
    fn add_gf(a: gf16, b: gf16) -> gf16 {
        return gf16.from_f64(gf16.to_f64(a) + gf16.to_f64(b));
    }

    fn mul_gf(a: gf16, b: gf16) -> gf16 {
        return gf16.from_f64(gf16.to_f64(a) * gf16.to_f64(b));
    }

    test gf16_mul_identity
        given x = one
        and   result = mul_gf(x, one)
        then gf16.to_f64(result) == 1.0
}
```

### Direct bit manipulation

```t27
module GF16Bits {
    // GF16 bit layout: [S(1) | E(6) | M(9)]

    const SIGN_MASK  : u16 = 0x8000;
    const EXP_MASK   : u16 = 0x7E00;
    const MANT_MASK  : u16 = 0x01FF;

    fn extract_sign(x: u16) -> bool {
        return (x & SIGN_MASK) != 0;
    }

    fn extract_exp(x: u16) -> u8 {
        return ((x & EXP_MASK) >> 9) as u8;
    }

    fn extract_mant(x: u16) -> u9 {
        return (x & MANT_MASK) as u9;
    }

    test extract_one_positive
        given x = 0x3C00  // GF16 encoding of 1.0
        then extract_sign(x) == false
         and extract_exp(x) == 31
         and extract_mant(x) == 0
}
```

---

## RTL Generation

### Generating Verilog

```bash
# Generate RTL from spec
./scripts/tri gen specs/numeric/gf16.t27

# The generated Verilog is in rtl_gen/
ls rtl_gen/gf16*.v
```

### Generated Modules

Each format gets:
- `<format>_add.v` - Addition module
- `<format>_mul.v` - Multiplication module

### Example: GF16 Adder

```verilog
`default_nettype none
module gf16_add (
    input  wire [15:0] a,
    input  wire [15:0] b,
    output reg  [15:0] result
);
    // ... implementation
endmodule
```

### Synthesis

```bash
cd rtl_gen
make gf16_add    # Synthesize specific module
make gf16_mul    # Synthesize multiplier
make all          # Synthesize all
```

---

## Coq Formal Verification

### Writing QED Theorems

Coq files are in `trios-coq/Coq/`:

```coq
Module GF16Arithmetic.
  (* QED: Commutativity of GF16 addition *)
  Theorem gf16_add_commutative:
    forall (a b : GF16),
      gf16_add a b = gf16_add b a.
  Proof.
    intros a b.
    unfold gf16_add.
    (* ... proof steps ... *)
    reflexivity.
  Qed.

  (* QED: Zero is additive identity *)
  Theorem gf16_zero_is_additive_identity:
    forall (a : GF16),
      gf16_add a gf16_zero = a.
  Proof.
    (* ... *)
  Qed.
End GF16Arithmetic.
```

### Compiling Coq

```bash
cd trios-coq/Coq
coqc -R . GFFormats.v   # Compile specific file
make -C .                 # Compile all
```

---

## Advanced Topics

### Format Conversions

```t27
module FormatConversions {
    use numeric::formats;

    fn f32_to_gf16(x: f32) -> gf16 {
        return quantize_value(x, .gf16);
    }

    fn gf16_to_int8(x: gf16) -> int8 {
        return quantize_value(gf16.to_f64(x), .int8);
    }

    test f32_to_gf16_one
        given x = 1.0
        and   gf = f32_to_gf16(x)
        and   back = gf16.to_f64(gf)
        then abs(back - 1.0) < 0.01
}
```

### Ternary Computing

```t27
module TernaryOps {
    use base::ternary;

    // VSA (Vector Symbolic Architecture) operations
    fn trit_xor(a: Trit, b: Trit) -> Trit {
        if (a == .zero) { return b; }
        if (b == .zero) { return a; }
        if (a == b) { return .zero; }
        return .pos;
    }

    test trit_xor_properties
        given x = .pos
        and   y = .neg
        and   z = trit_xor(x, y)
        then z == .neg
}
```

### Using the Toolchain

```bash
# Generate all format code
./scripts/tri gen specs/numeric/*.t27

# Run TDD tests
./scripts/tri test

# Generate Zig code
./scripts/tri gen --lang zig specs/numeric/gf16.t27

# Verify conformance
./scripts/tri verify
```

---

## Best Practices

### 1. Always Start with Tests

```t27
// Good: Test first
module GoodExample {
    test add_works
        given result = add(1, 2)
        then result == 3

    fn add(a: i32, b: i32) -> i32 { return a + b; }
}

// Bad: Implementation without tests
module BadExample {
    fn add(a: i32, b: i32) -> i32 { return a + b; }
}
```

### 2. Use Sacred Constants

```t27
module SacredConstants {
    use math::constants;

    const PHI_APPROX : f64 = 1.618;
    const GOLDEN_RATIO = constants::PHI;

    // Use the official constant
    test phi_matches
        then abs(GOLDEN_RATIO - PHI_APPROX) < 0.001
}
```

### 3. Leverage Format Registry

```t27
module FormatUsage {
    use numeric::goldenfloat_family;

    fn get_optimal_format(bits: u8) -> Option<string> {
        const fmt = goldenfloat_family::get_format_by_bits(bits);
        if (fmt != null) {
            return fmt.?.name;
        }
        return null;
    }
}
```

---

## Troubleshooting

### Common Issues

1. **Compilation errors**: Check that all modules used are imported
2. **Test failures**: Use `./scripts/tri test --verbose` for details
3. **RTL synthesis**: Check `synthesis_report.txt` in `rtl_gen/`

### Getting Help

```bash
./scripts/tri --help
./scripts/tri test --help
```

---

## Reference

### Quick Commands

```bash
# Compile a spec
./scripts/tri gen <spec.t27>

# Run tests
./scripts/tri test

# Verify conformance
./scripts/tri verify

# Generate all outputs
./scripts/tri gen --all

# Clean build
./scripts/tri clean
```

### File Structure

```
t27/
├── bootstrap/          # Compiler source
├── docs/              # Documentation
├── scripts/           # Toolchain scripts
├── specs/              # .t27 specifications
│   ├── numeric/        # Format specs
│   ├── math/           # Math specs
│   └── ...
├── rtl_gen/           # Generated RTL
├── trios-coq/         # Coq proofs
└── gen/               # Generated code
    └── rust/          # Rust runtime
```

---

## Appendix

### φ-Ratio Calculation

The φ-ratio is calculated as:

```
φ_ratio = exp_bits / mant_bits
phi_distance = |φ_ratio - 1/φ|
```

Where 1/φ ≈ 0.618 is the golden ratio inverse.

### Format Encoding Formula

For a GoldenFloat with:
- `E` exponent bits
- `M` mantissa bits
- Bias = 2^(E-1) - 1

The value is:

```
value = (-1)^s × (1 + m/2^M) × 2^(e - Bias)
```

Where:
- `s` is sign bit
- `e` is exponent value
- `m` is mantissa value

---

**Version**: v1.0.0
**Last Updated**: 2026-05-17
**License**: Apache-2.0