# RFC: .tri Language Core Specification

**Status**: Draft
**Ring**: Ring 01 - Foundation
**Author**: Trinity RFC
**Created**: 2026-04-16
**Related**: [Parameter Golf](https://github.com/openai/parameter-golf)

---

## Abstract

`.tri` (Trinity Intermediate Representation) — это **канонический IR язык** для тернарных/φ-оптимизированных вычислений в рамках Trinity S³AI. Он служит мостом между человеко-читаемыми спецификациями `.t27` и бинарными backendами (Zig, Rust, Verilog), обеспечивая единый source of truth для компиляции, тестирования и верификации.

---

## Motivation

### Почему `.tri` нужен как отдельный язык

1. **Один источник правды**: В текущем проекте логика дублируется между `.t27`, Zig backend и Runtime. Это нарушает принцип Trinity "один source of truth".

2. **Ternary-native semantics**: Trinity оптимизирована под φ² = φ + 1 и тернарную логику. `.tri` должен выражать эту семантику нативно, а не через трансляцию из бинарного мира.

3. **Experience integration**: Agent-based обучение требует встроенных хуков в IR, чтобы trace был воспроизводимым без дополнительной инструментации.

4. **Parameter Golf insights**: Техники вроде QAT, GPTQ embeddings, weight equalization (MuonEq-R) работают на уровне типов/представлений, а не трансляции.

### Инсайты из Parameter Golf

| Техника | Применение к .tri |
|-----------|------------------|
| **Ternary QAT** | Симуляция шума квантизации во время обучения через встроенный тип `noise_layer` |
| **GPTQ embeddings** | Group-wise квантизация через `quantize_groups()` для φ²-embeddings |
| **Weight Equalization (MuonEq-R)** | Балансировка весов через `scale_factor` в слоях |
| **XSA** | Частичное само-внимание для cross-ring операций |
| **Parallel Residuals** | Отдельные lanes для attention/MLP residuals → вдохновение для параллельных тернарных потоков |

---

## Core Language Design

### 1. Lexical Structure

`.tri` использует ASCII-only лексику с английскими идентификаторами (L3 purity law).

#### Comments

```tri
// Single-line comment
// This explains a single line

/* Multi-line comment
   that spans multiple lines
*/

/// Documentation comment (preferred for pub items)
```

#### Identifiers

```tri
// Variables and parameters
let variable_name = value;

// Constants
pub const CONSTANT_NAME: type = value;

// Functions
pub fn function_name(param: type) -> return_type;
```

### 2. Type System

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

// Tuples
pub type Pair(T, U) = struct {
    first: T,
    second: U
};
```

#### Trinity Built-in Constants

```tri
// Mathematical constants (L5 identity law)
pub const PHI: f64 = 1.6180339887498948482;
pub const TRINITY: f64 = 3.0;
pub const PI: f64 = 3.141592653589793;
pub const E: f64 = 2.718281828459045;

// Machine invariants
pub const MAX_U8: u8 = 255;
pub const MAX_U16: u16 = 65535;
pub const MAX_U32: u32 = 4294967295;

// Numeric format identifiers
pub const FORMAT_GF16: string = "gf16";
pub const FORMAT_TF3: string = "tf3";  // ternary float3
```

### 3. Expressions

```tri
// Literals
42                         // f64 literal
"hello"                   // string literal
true                       // Bool literal

// Variables
x                           // identifier

// Binary operators
x + y                        // addition
x - y                        // subtraction
x * y                        // multiplication
x / y                        // division
x % y                        // modulo
x < y                        // less than
x <= y                       // less than or equal
x > y                        // greater than
x == y                       // equality
x != y                       // inequality

// Logical operators
x && y                       // logical AND
x || y                       // logical OR
!x                          // logical NOT

// Field access
struct.field                  // Access struct field
array[index]                  // Array indexing
```

### 4. Statements

```tri
// Variable declaration
let x: f64 = 42.0;

// Struct declaration
pub struct Point {
    x: f64,
    y: f64,
}

// Enum declaration
pub const Direction = enum(u8) {
    north = 0,
    east = 1,
    south = 2,
    west = 3
};

// Function declaration
pub fn add(a: f64, b: f64) -> f64 {
    return a + b;
}

// Assignment
result = add(x, y);

// Return statement
return result;

// If expression (ternary)
let result = if condition then true_value else false_value;

// For loop
for i in 0..10 {
    do_something(i);
}

// While loop
while condition {
    do_something();
}

// Block statement
{
    statement1;
    statement2;
    statement3;
}
```

### 5. Control Flow

```tri
// Switch on enum
switch value {
    case Direction.north => handle_north(),
    case Direction.south => handle_south(),
    default => handle_default()
}

// Match on struct/union
match value {
    Point { x, y } => calculate(),
    _ => handle_default()
}
```

### 6. Sections

```tri
spec module_name {

    // Types section
    pub type MyType = struct { ... };

    // Constants section
    pub const MY_CONST: f64 = 42.0;

    // Functions section
    pub fn my_function(x: MyType) -> f64 { ... };
}
```

### 7. Test System

Тесты являются **обязательными** по Article II SOUL.md — каждый spec обязан иметь тесты.

```tri
test test_addition_of_trits {
    given a = Trit.pos;
    given b = Trit.pos;

    // Basic assertions
    assert (a + b) == Trit.pos;

    // Computed assertions
    let result = a + b;
    assert result == Trit.pos;
}
```

#### Test Syntax

| Construct | Syntax | Example |
|-----------|--------|--------|
| `given` | `given x: f64 = 42.0;` |
| `when` | `when condition then { ... }` |
| `then` | `then expected_value` |
| `assert` | `assert expected == actual;` |
| `bench` | см. ниже |

### 8. Invariant System

Инварианты — это **математические законы** языка (отличные от тестов). Они описывают свойства, которые должны выполняться всегда.

```tri
invariant phi_squared_identity {
    // φ² + φ⁻² = 3 (L5 identity law)
    given x: f64;
    given y: f64;
    
    let lhs = (x * PHI) + (y / PHI);
    let rhs = x * x + y * y;
    
    assert lhs == rhs;
}
```

#### Invariant Syntax

```tri
invariant name {
    // Precondition
    given <inputs>;

    // Property to verify
    assert <property>;

    // Optional: forall quantifier
    forall <var> in <range> => <property>;
}
```

### 9. Benchmark System

Бенчмарки измеряют производительность операций с указанием targets.

```tri
bench matrix_multiply {
    measure: nanoseconds to matrix_multiply(4, 4);

    target: gf16  // Format target
    target: tf3   // Alternative target
    target: <1us;   // Absolute timing target

    // Warmup runs
    warmup: 3;

    // Number of iterations
    runs: 100;
}
```

---

## Numeric Format Declaration

`.tri` поддерживает декларативное указание числового формата через блок `numeric_format`:

```tri
numeric_format gf16 | tf3 {
    // Primary format for φ-optimized arithmetic
    // Target: zig-golden-float GF16 implementation
}

numeric_format tf3 | gf16 | gf32 {
    // Ternary float3: {-1, 0, +1} mapped to GF16/GF32
    // For efficient ternary operations in non-primary contexts
}
```

**Правило**: Генерация без указания `numeric_format` использует стандарт IEEE 754 (по умолчанию).

---

## IR Features

### 1. Experience Hooks

Интеграция с `.trinity/experience/` для agent-based обучения:

```tri
// Experience hook declaration
experience capture_my_decision {
    // Context: what task/decision
    context: "optimization_choice";

    // Decision: what was chosen
    decision: "prefer_ternary_over_binary";

    // Rationale: why
    rationale: "Parameter Golf shows ternary QAT improves by 0.005 BPB";

    // Outcome: what happened
    outcome: "accepted";

    // Confidence: 0.0-1.0
    confidence: 0.85;
}
```

### 2. Proof Seals

Интеграция с `.trinity/seals/` для cryptographic верификации:

```tri
// Seal declaration (seal hash for reproducibility)
seal compute_result {
    // Hash algorithm
    algorithm: "sha256";

    // Version
    version: "1.0";

    // Input fingerprint
    input_hash: "abc123...";

    // Output fingerprint
    output_hash: "def456...";

    // Computation timestamp
    timestamp_ns: 169749820000;
}
```

### 3. Calibration Layers

Поддержка для QAT-like техник (симуляция шума квантизации):

```tri
// Noise layer for quantization-aware training
noise_layer ternary_quant_noise {
    // Ternary noise pattern: {-1, 0, +1} × range
    noise: [ -1 .. +1 ]f32;

    // Apply during training to reduce quantization error
    // Integrated into ternary operations naturally
}
```

---

## Module System

`.tri` поддерживает модульную систему для организации кода:

```tri
// Import external module
import std.math;

// Export selected items
pub type MyType;
pub fn my_function(x: f64) -> f64;

// Or export entire module
export {
    pub type Point = struct { x: f64, y: f64 };
    pub fn distance(a: Point, b: Point) -> f64;
}
```

---

## Compilation Targeting

Явное указание target backend для генерации:

```tri
// Target declaration at module level
target zig_golden_float {
    // Compiler backend
    compiler: "zig-golden-float";

    // Sub-target options
    format: "gf16";  // or tf3, gf32, etc.
    optimize: "size";   // or speed, accuracy
}
```

Или через флаг компилятора:

```bash
tri gen spec.tri --target zig --format gf16
```

---

## Validation Rules

### L2 Generation Law

Все сгенерированные файлы в `gen/` являются auto-generated. Ручное редактирование запрещено.

### L3 Purity Law

`.tri` файлы — ASCII-only с английскими идентификаторами.

### L5 Identity Law

Все числовые операции должны использовать константы PHI, TRINITY, PI, E корректно.

---

## Examples

### Hello World

```tri
// Complete module with main function
spec hello_world {
    pub fn main() -> void {
        print("Hello from .tri!");
    }
}
```

### Matrix Operations

```tri
spec matrix_ops {
    pub struct Matrix3x3 {
        data: [3][3]f64,
    }

    pub fn multiply(m: Matrix3x3, n: Matrix3x3) -> Matrix3x3 {
        let result: Matrix3x3 = undefined;

        for i in 0..3 {
            for j in 0..3 {
                let sum: f64 = 0.0;
                for k in 0..3 {
                    sum = sum + m.data[i][k] * n.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }

        return result;
    }
}
```

### Ternary Operations

```tri
spec ternary_logic {
    pub const TritOps = struct {
        // Ternary addition: {-1, 0, +1}
        add: fn(a: Trit, b: Trit) -> Trit,

        // Ternary multiplication
        mul: fn(a: Trit, b: Trit) -> Trit,
    };

    // Using ternary encoding for neural activations
    pub fn neural_layer(inputs: [10]f64) -> [10]f64 {
        // {-1, 0, +1} encoding naturally fits ReLU patterns
        let weights: [10][10]f64;
        // ... computation using TritOps.add
    }
}
```

---

## Migration Path

Для миграции существующего кода в `.tri`:

### Ring 1: Core Types
- [x] Перенести встроенные типы из `.t27` в `.tri`
- [x] Формализовать Trit enum как тип ядра языка

### Ring 2: Numeric Formats
- [x] Добавить `numeric_format` декларации в модули
- [x] Интегрировать GF16/TF3 из GoldenFloat как built-in

### Ring 3: Experience Integration
- [x] Определить секции `experience` в `.tri`
- [x] Создать хуки для agent-based обучения

### Ring 4: Lowering to Targets
- [x] Обновить `tri gen` для поддержки `.tri` targets
- [x] Добавить поддержку `--target` флаг

---

## Open Questions

1. **Syntax sugar**: Нужен ли синтаксический сахар для работы с массивами (например, slice notation)?

2. **Pattern matching**: Поддерживать ли полноценный pattern matching на enums/structs?

3. **Memory model**: Должен ли `.tri` описывать выделение памяти явно или через inferred?

4. **Error handling**: Как должны обрабатываться ошибки компиляции/рантайма?

5. **Cross-target code**: Нужно ли поддерживать shareable функции между разными backendами?

---

## References

- [Parameter Golf Techniques](https://github.com/openai/parameter-golf#readme)
- [Trinity S³AI](https://github.com/gHashTag/trinity)
- [SOUL.md](./SOUL.md) — Article II: Test Requirements
- [T27 Constitution](./docs/T27-CONSTITUTION.md)
- [GoldenFloat](https://github.com/gHashTag/zig-golden-float)
