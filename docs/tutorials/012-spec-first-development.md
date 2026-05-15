# Spec-First Development with t27 — From Specification to Code

**Author:** Trinity S³AI Team
**Version:** 1.0.0
**Last Updated:** 2026-05-15

---

## Introduction

t27 introduces a **spec-first development model**. Instead of writing code and testing afterward, you write specifications (`.t27` files) that define both semantics and tests. Code is generated automatically.

This tutorial explains the spec-first workflow, conventions, and best practices.

---

## Part 1: The Philosophy

### Why Spec-First?

Traditional development:
```
Write Code → Write Tests → Debug → Fix
```

t27 development:
```
Write Spec (with embedded tests) → Generate Code → Verify
```

### Benefits

| Aspect | Traditional | Spec-First |
|--------|-------------|------------|
| Bugs found | After coding | During spec writing |
| Multiple backends | Manual porting | Auto-generation |
| Documentation | Separate | Embedded |
| Formal verification | Optional | Built-in |
| Single source of truth | ❌ | ✅ |

### The t27 Promise

> **"If it's in the spec, it works in all backends."**

---

## Part 2: The .t27 File Structure

### Basic Template

```t27
module ModuleName {
    // 1. Imports
    use other_module;

    // 2. Constants
    const NAME: type = value;

    // 3. Types
    struct TypeName {
        field: type,
    }

    // 4. Functions
    fn function_name(params) -> return_type {
        // implementation
        return value;
    }

    // 5. Tests
    test "test_name" {
        given { /* setup */ }
        then { /* action */ }
        expect { /* assertion */ }
    }

    // 6. Invariants
    invariant: expression == expected;

    // 7. Benchmarks
    bench "bench_name" {
        // benchmark code
    }
}
```

### Section Order (Constitution L4)

1. **Imports** — Dependencies
2. **Constants** — Compile-time values
3. **Types** — Structs, enums, aliases
4. **Functions** — Implementation
5. **Tests** — Test cases
6. **Invariants** — Mathematical properties
7. **Benchmarks** — Performance tests

---

## Part 3: Defining Constants

### Basic Constants

```t27
module Constants {
    // Numeric constants
    const ZERO: i32 = 0;
    const MAX_U8: u8 = 255;
    const PI: f32 = 3.141592653589793;

    // String constants
    const VERSION: str = "1.0.0";
    const GREETING: str = "Hello, Trinity!";

    // Boolean constants
    const DEBUG: bool = true;
    const PRODUCTION: bool = false;
}
```

### Phi Constants (Constitution L5)

```t27
module PhiConstants {
    // The golden ratio
    const PHI: phi = 1.618033988749895;

    // Derived constants
    const PHI_SQUARED: f32 = PHI * PHI;  // ≈ 2.618
    const PHI_NEG: f32 = 1.0 / PHI;      // ≈ 0.618

    // Trinity identity (must hold)
    invariant: PHI_SQUARED + (1.0 / PHI_SQUARED) ≈ 3.0;
}
```

### GoldenFloat Constants

```t27
module GFConstants {
    const GF16_BIAS: i32 = 31;
    const GF32_BIAS: i32 = 2047;
    const GF64_BIAS: i32 = 8388607;
}
```

---

## Part 4: Defining Types

### Structs

```t27
module Types {
    struct Point {
        x: f32,
        y: f32,
    }

    struct Color {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    }
}
```

### Enums

```t27
module Enums {
    enum Result<T, E> {
        Ok(value: T),
        Err(error: E),
    }

    enum Trit {
        Negative = -1,
        Zero = 0,
        Positive = 1,
    }
}
```

### Type Aliases

```t27
module Aliases {
    // Standard aliases
    type u8 = uint<8>;
    type i32 = int<32>;

    // Custom aliases
    type Vec2D = Point;
    type Matrix = vec<vec<f32>>;
}
```

---

## Part 5: Defining Functions

### Basic Function

```t27
module Math {
    fn add(a: i32, b: i32) -> i32 {
        return a + b;
    }

    test "add works correctly" {
        given {
            let x = 5;
            let y = 3;
        }
        then {
            let result = add(x, y);
        }
        expect {
            result == 8;
        }
    }
}
```

### Function with Multiple Returns

```t27
module MultiReturn {
    fn divmod(a: i32, b: i32) -> (i32, i32) {
        let quotient = a / b;
        let remainder = a % b;
        return (quotient, remainder);
    }

    test "divmod returns correct values" {
        given {
            let dividend = 17;
            let divisor = 5;
        }
        then {
            let (q, r) = divmod(dividend, divisor);
        }
        expect {
            q == 3 && r == 2;
        }
    }
}
```

### Recursive Function

```t27
module Recursion {
    fn factorial(n: u32) -> u32 {
        if n == 0 {
            return 1;
        } else {
            return n * factorial(n - 1);
        }
    }

    test "factorial base case" {
        given { }
        then { let result = factorial(0); }
        expect { result == 1; }
    }

    test "factorial recursive case" {
        given { }
        then { let result = factorial(5); }
        expect { result == 120; }
    }
}
```

### Generic Function

```t27
module Generics {
    fn identity<T>(value: T) -> T {
        return value;
    }

    test "identity works for i32" {
        given { let x = 42; }
        then { let result = identity(x); }
        expect { result == 42; }
    }

    test "identity works for str" {
        given { let s = "test"; }
        then { let result = identity(s); }
        expect { result == "test"; }
    }
}
```

---

## Part 6: Writing Tests

### Test Structure

```t27
test "descriptive_test_name" {
    given {
        // Arrange: Set up test data
        let input = create_test_data();
    }
    then {
        // Act: Execute the function
        let result = function_under_test(input);
    }
    expect {
        // Assert: Verify the result
        result == expected_value;
    }
}
```

### Test Best Practices

1. **Descriptive names** — "add_returns_sum_of_two_numbers"
2. **One assertion per test** — Clear failure messages
3. **Given/Then/Expect** — Clear separation of concerns
4. **Independent tests** — No shared state
5. **Cover edge cases** — Zero, empty, negative values

### Edge Case Testing

```t27
module EdgeCases {
    fn safe_divide(a: f32, b: f32) -> option<f32> {
        if b == 0.0 {
            return None;
        }
        return Some(a / b);
    }

    test "divide normal values" {
        given { }
        then { let result = safe_divide(10.0, 2.0); }
        expect { result == Some(5.0); }
    }

    test "divide by zero returns None" {
        given { }
        then { let result = safe_divide(10.0, 0.0); }
        expect { result == None; }
    }

    test "divide negative numbers" {
        given { }
        then { let result = safe_divide(-10.0, 2.0); }
        expect { result == Some(-5.0); }
    }
}
```

---

## Part 7: Writing Invariants

### What are Invariants?

Invariants express **mathematical properties** that must always hold:

```t27
module Invariants {
    const PHI: f64 = 1.618033988749895;

    invariant "trinity_identity" {
        return PHI * PHI + 1.0 / (PHI * PHI) ≈ 3.0;
    }

    invariant "golden_ratio_continued_fraction" {
        return PHI ≈ 1.0 + 1.0 / PHI;
    }

    invariant "phi_squared_identity" {
        return PHI * PHI ≈ PHI + 1.0;
    }
}
```

### Invariant Best Practices

1. **Express fundamental properties** — Not implementation details
2. **Use tolerance for floating-point** — `≈` operator
3. **Document with names** — Explain what's being asserted
4. **Keep them simple** — Complex invariants are hard to verify

### Property-Based Invariants

```t27
module Properties {
    fn reverse<T>(arr: vec<T>) -> vec<T> {
        let mut result: vec<T> = [];
        for i in (len(arr) - 1)..0 {
            result.push(arr[i]);
        }
        return result;
    }

    invariant "reverse_double_reverse_returns_original" {
        for arr in all_vectors() {
            let reversed = reverse(arr);
            let double_reversed = reverse(reversed);
            assert(double_reversed == arr);
        }
    }

    invariant "reverse_length_preserved" {
        for arr in all_vectors() {
            assert(len(reverse(arr)) == len(arr));
        }
    }
}
```

---

## Part 8: Writing Benchmarks

### Basic Benchmark

```t27
module Benchmarks {
    fn fibonacci(n: u32) -> u32 {
        if n <= 1 {
            return n;
        }
        return fibonacci(n - 1) + fibonacci(n - 2);
    }

    bench "fibonacci_10" {
        for _ in 0..1000 {
            fibonacci(10);
        }
    }

    bench "fibonacci_20" {
        for _ in 0..100 {
            fibonacci(20);
        }
    }
}
```

### Benchmark Best Practices

1. **Warm-up iterations** — Don't count first runs
2. **Stable input** — Same data across iterations
3. **Appropriate iterations** — Balance precision and time
4. **Meaningful comparisons** — Benchmark alternative implementations

---

## Part 9: The Development Workflow

### Step 1: Write the Spec

Create `mymodule.t27`:

```t27
module MyModule {
    fn calculate_average(values: vec<f32>) -> f32 {
        if len(values) == 0 {
            return 0.0;
        }
        let mut sum: f32 = 0.0;
        for v in values {
            sum = sum + v;
        }
        return sum / (len(values) as f32);
    }

    test "average of positive numbers" {
        given {
            let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        }
        then {
            let result = calculate_average(data);
        }
        expect {
            result == 3.0;
        }
    }

    test "average of empty list" {
        given {
            let data: vec<f32> = [];
        }
        then {
            let result = calculate_average(data);
        }
        expect {
            result == 0.0;
        }
    }
}
```

### Step 2: Parse and Verify

```bash
# Parse the spec
tri parse mymodule.t27

# Output:
# ✓ Parsed mymodule.t27
# ✓ 2 tests found
# ✓ 0 invariants found
```

### Step 3: Generate Code

```bash
# Generate Zig code
tri gen-zig mymodule.t27 > gen/mymodule.zig

# Generate C code
tri gen-c mymodule.t27 > gen/mymodule.c

# Generate Rust code
tri gen-rust mymodule.t27 > gen/mymodule.rs
```

### Step 4: Run Tests

```bash
# Run tests from spec
tri test mymodule.t27

# Output:
# ✓ test "average of positive numbers" passed
# ✓ test "average of empty list" passed
# 2/2 tests passed
```

### Step 5: Seal and Verify

```bash
# Seal the spec with hash
tri seal mymodule.t27

# Verify integrity
tri verify mymodule.t27

# Output:
# ✓ Spec hash verified
# ✓ Generated code matches spec
```

---

## Part 10: Advanced Topics

### Module Composition

```t27
// utils.t27
module Utils {
    fn clamp(value: f32, min: f32, max: f32) -> f32 {
        if value < min {
            return min;
        }
        if value > max {
            return max;
        }
        return value;
    }
}

// main.t27
module Main {
    use Utils;

    fn normalize(value: f32) -> f32 {
        return clamp(value, 0.0, 1.0);
    }
}
```

### Trait-like Behavior

```t27
module Traits {
    trait Numeric {
        fn add(self, other: Self) -> Self;
        fn mul(self, other: Self) -> Self;
    }

    impl Numeric for f32 {
        fn add(self, other: f32) -> f32 {
            return self + other;
        }
        fn mul(self, other: f32) -> f32 {
            return self * other;
        }
    }
}
```

### Conditional Compilation

```t27
module Conditional {
    #cfg(target = "wasm")
    fn platform_specific() -> str {
        return "WebAssembly";
    }

    #cfg(target != "wasm")
    fn platform_specific() -> str {
        return "Native";
    }
}
```

---

## Part 11: Common Patterns

### The Option Pattern

```t27
module OptionPattern {
    enum Option<T> {
        Some(value: T),
        None,
    }

    fn divide(a: f32, b: f32) -> Option<f32> {
        if b == 0.0 {
            return None;
        }
        return Some(a / b);
    }

    fn safe_divide(a: f32, b: f32, default: f32) -> f32 {
        match divide(a, b) {
            Some(value) => value,
            None => default,
        }
    }
}
```

### The Result Pattern

```t27
module ResultPattern {
    enum Result<T, E> {
        Ok(value: T),
        Err(error: E),
    }

    fn parse_number(s: str) -> Result<i32, str> {
        // Simplified parsing
        if s == "NaN" {
            return Err("not a number");
        }
        return Ok(42); // Simplified
    }
}
```

### The Builder Pattern

```t27
module BuilderPattern {
    struct Config {
        port: u16,
        host: str,
        debug: bool,
    }

    struct ConfigBuilder {
        port: option<u16>,
        host: option<str>,
        debug: bool,
    }

    fn new_config_builder() -> ConfigBuilder {
        return ConfigBuilder {
            port: None,
            host: None,
            debug: false,
        };
    }

    fn port(mut self: ConfigBuilder, p: u16) -> ConfigBuilder {
        self.port = Some(p);
        return self;
    }

    fn host(mut self: ConfigBuilder, h: str) -> ConfigBuilder {
        self.host = Some(h);
        return self;
    }

    fn build(self: ConfigBuilder) -> Config {
        return Config {
            port: self.port.unwrap_or(8080),
            host: self.host.unwrap_or("localhost"),
            debug: self.debug,
        };
    }
}
```

---

## Part 12: Debugging Generated Code

### When Generated Code Fails

1. **Check the spec first** — Errors often originate in source
2. **Review the generated code** — Look at `gen/` directory
3. **Run in debug mode** — `tri test --debug`
4. **Check backend-specific issues** — Some backends have different limitations

### Reading Generated Zig

```zig
// Generated from spec
pub fn calculate_average(values: []const f32) f32 {
    if (values.len == 0) {
        return 0.0;
    }
    var sum: f32 = 0.0;
    for (values) |v| {
        sum += v;
    }
    return sum / @as(f32, @floatFromInt(values.len));
}
```

### Reading Generated C

```c
// Generated from spec
float calculate_average(const float* values, size_t len) {
    if (len == 0) {
        return 0.0f;
    }
    float sum = 0.0f;
    for (size_t i = 0; i < len; i++) {
        sum += values[i];
    }
    return sum / (float)len;
}
```

---

## Part 13: FAQ

### Q: Can I edit generated code?

A: **No!** (Constitution L2) Generated files are compiler output. Edit the spec instead.

### Q: How do I handle backend-specific features?

A: Use conditional compilation with `#cfg` directives in your spec.

### Q: What if I need a feature not in the spec language?

A: File an issue or contribute to the language specification. The spec-first approach ensures consistency across backends.

### Q: How do I debug test failures?

A: Use `tri test --verbose` to see detailed output. Check that your given/then/expect blocks are correctly structured.

### Q: Can I have multiple modules in one file?

A: No, each `.t27` file contains exactly one module. Use multiple files for multiple modules.

---

## Conclusion

Spec-first development with t27 provides:

- **Single source of truth** — The spec is the specification
- **Multi-backend generation** — One spec, many languages
- **Embedded tests** — TDD is part of the spec
- **Formal verification** — Invariants ensure correctness
- **Constitution compliance** — L1-L7 laws enforced

The workflow is: **Write Spec → Generate Code → Test → Deploy**

No manual porting, no separate test files, no documentation drift.

**φ² + 1/φ² = 3 | TRINITY**