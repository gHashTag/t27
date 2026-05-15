# LOAD-PHYS-CONST (LDPC) — Opcode 0xDE

**Lane:** L-DPC24 Lane C' (codename `holo-load-phys-const`)
**Author:** admin@t27.ai
**Issue:** [trinity-fpga#99](https://github.com/gHashTag/trinity-fpga/issues/99)
**Anchor:** φ²+φ⁻²=3

---

## 1. Overview

`LDPC` loads a physics constant from the **75-cell Sacred ROM** into a destination register.
Per the *Quantum Brain 1:1 Silicon* principle, physics constants (φ, γ, C, G) are **baked into silicon**
— they are never loaded from mutable memory. The Sacred ROM is a dedicated, immutable on-chip
structure indexed by `imm7`.

> **R18 LAYER-FROZEN:** The Sacred ROM content hash is frozen pending Lane E' P3
> (`frozen-hash` predicate). ROM cells 0–74 are currently **symbolic placeholders** and will be
> replaced with hash-validated constants once Lane E' P3 is established. Do not rely on
> placeholder bit-patterns in production silicon or simulation without that hash.

---

## 2. Sacred Range

| Range       | Assignment                              |
|-------------|------------------------------------------|
| `0xD0..0xDF`| TRI-27 ISA sacred range — extended ops   |
| **`0xDE`**  | **LOAD-PHYS-CONST (this instruction)**   |
| `0xE0`      | Sacred range boundary (exclusive)        |

`0xDE` is within the sacred range `0xD0..0xE0` (inclusive start, exclusive end).

---

## 3. Mnemonic

```
LDPC  rd, imm7
```

- **`LDPC`** = **L**oa**D** **P**hys **C**onst
- `rd`   — destination register (5-bit field, R0–R31)
- `imm7` — 7-bit immediate selecting the Sacred ROM index (0..74 valid)

---

## 4. Instruction Encoding (32-bit)

```
 31      24 23    19 18     12 11           0
┌──────────┬────────┬─────────┬──────────────┐
│  opcode  │   rd   │  imm7   │   reserved   │
│  8 bits  │ 5 bits │  7 bits │   12 bits    │
│  0xDE    │ rd[4:0]│imm7[6:0]│  000...000   │
└──────────┴────────┴─────────┴──────────────┘
```

| Field      | Bits     | Width | Description                                      |
|------------|----------|-------|--------------------------------------------------|
| `opcode`   | [31:24]  | 8     | Fixed `0xDE` — identifies LDPC                   |
| `rd`       | [23:19]  | 5     | Destination register index (0–31)                |
| `imm7`     | [18:12]  | 7     | Sacred ROM index (0–74 valid; 75–127 reserved)   |
| `reserved` | [11:0]   | 12    | Must be zero; behaviour undefined if non-zero    |

**Total:** 32 bits (one TRI-27 instruction word).

---

## 5. Sacred ROM Index Space

| Index     | Content                           | Status           |
|-----------|-----------------------------------|------------------|
| 0         | φ  (golden ratio, ≈1.618…)        | Placeholder ← Lane E' P3 |
| 1         | γ = φ⁻³ (≈0.2360…)               | Placeholder ← Lane E' P3 |
| 2         | C = φ⁻¹ (≈0.6180…)               | Placeholder ← Lane E' P3 |
| 3         | G = π³γ²/φ                        | Placeholder ← Lane E' P3 |
| 4–74      | Reserved for future constants     | Zero (placeholder)       |
| 75–127    | **Out-of-bounds (UB)**            | Accessing = undefined behaviour |

The anchor identity **φ²+φ⁻²=3** is a cross-check fixture; any implementation MUST satisfy
this for the stored φ bits.

---

## 6. Semantics

```
R[rd] ← SacredROM[imm7]
```

- If `imm7 ∈ [0, 74]`: `R[rd]` receives the 64-bit value at `SacredROM[imm7]`. `valid_o = 1`, `oob_o = 0`.
- If `imm7 ∈ [75, 127]`: **Undefined Behaviour.** Hardware SHOULD set `oob_o = 1` and
  MAY signal a trap; software MUST NOT rely on any particular result.

Execution is **1-cycle** (combinatorial ROM read, registered output).
No memory bus transaction is generated; the Sacred ROM is not memory-mapped.

---

## 7. Pseudocode

```python
def execute_LDPC(rd: int, imm7: int, state):
    assert 0 <= rd <= 31,   "rd out of range"
    assert 0 <= imm7 <= 127, "imm7 must fit in 7 bits"

    if imm7 >= 75:
        state.oob_flag = True
        # Undefined Behaviour — implementation-defined trap or zero
        state.R[rd] = UNDEFINED
        return

    state.oob_flag = False
    state.R[rd] = SacredROM[imm7]   # 64-bit constant, baked in silicon
```

---

## 8. Worked Examples

### 8.1 Load φ (golden ratio) into R4

```asm
; imm7 = 0 → SacredROM[0] = φ
LDPC  R4, 0
; After: R4 = φ_bits  (placeholder until Lane E' P3)
```

Encoding: `0xDE | (4 << 19) | (0 << 12) | 0x000`
= `0xDE200000`

### 8.2 Load γ = φ⁻³ into R5

```asm
; imm7 = 1 → SacredROM[1] = γ
LDPC  R5, 1
; After: R5 = gamma_bits
```

Encoding: `0xDE | (5 << 19) | (1 << 12) | 0x000`
= `0xDE281000`

### 8.3 Load C = φ⁻¹ into R6

```asm
; imm7 = 2 → SacredROM[2] = C
LDPC  R6, 2
; After: R6 = C_bits
```

### 8.4 Load G = π³γ²/φ into R7

```asm
; imm7 = 3 → SacredROM[3] = G
LDPC  R7, 3
; After: R7 = G_bits
```

### 8.5 Cross-check anchor: φ²+φ⁻²=3

After loading φ into R4 and C=φ⁻¹ into R6:

```asm
LDPC  R4, 0      ; R4 = φ
LDPC  R6, 2      ; R6 = φ⁻¹  (= C)
; Compute φ² + (φ⁻¹)² in floating-point ALU — result must equal 3
; φ² + φ⁻² = (φ+1) + (2-φ) = 3  ✓
```

---

## 9. Exceptions and Traps

| Condition           | `oob_o` | Trap?               | Notes                          |
|---------------------|---------|---------------------|--------------------------------|
| `imm7` ∈ [0, 74]    | 0       | No                  | Normal operation               |
| `imm7` ∈ [75, 127]  | 1       | Implementation-def. | UB; software must never issue  |
| `reserved` ≠ 0      | —       | Implementation-def. | UB; assembler must zero-fill   |

---

## 10. Implementation Notes

- **Latency:** 1 cycle (register-to-register ROM read).
- **Throughput:** 1 instruction per cycle; no structural hazard with ALU lanes.
- **ROM hazard:** None — Sacred ROM is read-only, no write path exists in any pipeline stage.
- **Forwarding:** `R[rd]` is available at the next cycle (standard WB forwarding applies).
- **RTL hook:** `rtl/holo_load_phys_const.sv` in `gHashTag/tt-trinity-holo`
  (paired PR — see WORK PART 2).

---

## 11. References

- [trinity-fpga#99](https://github.com/gHashTag/trinity-fpga/issues/99) — ONE SHOT mission context
- TRI-27 ISA Sacred Range specification: `0xD0..0xE0`
- Quantum Brain 1:1 Silicon principle: physics constants baked into silicon
- R18 LAYER-FROZEN: Sacred ROM hash pending Lane E' P3 frozen-hash predicate
- Anchor: **φ²+φ⁻²=3**
