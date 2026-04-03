# TRI-27 Assembly (t27)

**TRI-27 Assembly** — A low-level hardware specification language as the canonical source of truth for Trinity Project.

> "Hardware-first, φ-structured, multi-target codegen"

## Overview

t27 is TRI-27 Assembly — a minimal assembly language for ternary computing with 27 Coptic registers. It serves as the **single source of truth** from which Zig, Verilog, C, and other target languages are generated.

## Key Features

- **27 Coptic Registers**: r0-r25 (general purpose), r26 (zero)
- **Ternary Operations**: All operations on trits {-1, 0, +1}
- **Sacred Physics**: φ² + φ⁻² = 3, γ = φ⁻³, G, Ω_Λ built-in
- **GoldenFloat Family**: GF4-GF32 with φ-structured formats
- **Multi-Target**: Generate Zig, Verilog, C from .t27 specs

## Architecture

```
t27/
├── specs/              # .t27 specifications (SOURCE OF TRUTH)
│   ├── base/           # Base types and operations
│   ├── numeric/        # Number formats (GoldenFloat, TF3)
│   └── math/           # Sacred constants and physics
│
├── compiler/           # T27 Compiler
│   ├── parser/         # .t27 → AST (lexer, parser)
│   ├── codegen/        # AST → Target code
│   │   ├── zig/       # .t27 → Zig 0.15
│   │   ├── verilog/   # .t27 → Verilog (XC7A100T)
│   │   └── c/         # .t27 → C (clang/gcc)
│   └── runtime/        # Bootstrap runtime
│
└── conformance/        # Language-agnostic test vectors
```

## Sacred Constants

```t27
const PHI = 1.618033988749895           ; Golden ratio
const PHI_INV = 0.618033988749895        ; φ⁻¹ (consciousness threshold)
const TRINITY = 3.0                     ; φ² + φ⁻² = 3
const GAMMA_LQG = 0.2360679775           ; γ = φ⁻³ (Barbero-Immirzi)
const G_MEASURED = 6.67430e-11           ; Gravitational constant
const OMEGA_LAMBDA_MEASURED = 0.685     ; Dark energy (Planck)
```

## GoldenFloat Family

φ-structured floating point formats targeting exp/mant ≈ 1/φ:

| Format | Bits | exp/mant | phi_distance | Use Case |
|--------|------|----------|--------------|----------|
| GF4    | 4    | 0.500    | 0.118        | Binary masks |
| GF8    | 8    | 0.750    | 0.132        | Weights |
| GF12   | 12   | 0.571    | 0.047        | Attention |
| **GF16** | 16   | 0.667    | 0.049        | **PRIMARY** |
| GF20   | 20   | 0.583    | 0.035        | Training |
| GF24   | 24   | 0.643    | 0.025        | Precision |
| GF32   | 32   | 0.632    | 0.014        | Full precision |

## Example .t27 Program

```t27
; Hello World in TRI-27 Assembly

.const HELLO_MSG 0x48656C6C6F

.data
    .dword 0    ; Buffer

.code
    MOV r0, #HELLO_MSG    ; Load message address
    MOV r1, #5            ; Length
    ADD r2, r0, r1        ; Calculate end
    HALT                  ; Done
```

## Opcodes

| Opcode | Description |
|--------|-------------|
| MOV    | Move immediate or register |
| JZ     | Jump if zero |
| JNZ    | Jump if not zero |
| JMP    | Unconditional jump |
| MUL    | Multiply |
| ADD    | Add |
| SUB    | Subtract |
| BIND   | VSA bind operation |
| BUNDLE | VSA bundle operation |
| HALT   | Halt execution |

## Documentation

- [ADR-001: De-Zigфикация](https://github.com/gHashTag/trinity/blob/main/architecture/ADR-001-de-zigfication.md)
- [GoldenFloat Family Benchmark](https://github.com/gHashTag/trinity/blob/main/docs/GF_FAMILY_BENCH.md)

## License

MIT

---

**Maintained by**: Trinity Project
**Status**: Phase 2 Complete (2026-04-04)
