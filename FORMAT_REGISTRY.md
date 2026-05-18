# FORMAT_REGISTRY.md -- Numeric Format Surface (t27)

> **Role:** t27 is the **numeric format registry** for the TRI-NET line.
> All chips in the line draw their numeric kernel from the JSON
> SSOT `conformance/FORMAT-SPEC-001.json`. This document is the
> human-readable mirror of that file plus interop notes.

> **L6 CEILING:** `conformance/FORMAT-SPEC-001.json` and
> `specs/numeric/gf16.t27` are the **numeric SSOT**. If this Markdown
> disagrees with either, **the JSON / spec wins** -- file an issue, do not
> hand-edit the consumers.

---

## 1. Primary path -- GoldenFloat GF16

GF16 is the **primary numeric format** of the line. It is the default for the
gamma mesh and the only GoldenFloat width with hand-built Verilog top-levels
in this repo (`fpga/vivado/gf16_top.v`, `fpga/vivado/gf16_matmul4x4_top.v`).

### 1.1 Bit layout

```
GF16 = [ S(1) | E(6) | M(9) ]   bits 15..0
        15      14..9     8..0
```

| Field   | Bits  | Mask    | Notes                                    |
|---------|-------|---------|------------------------------------------|
| Sign    | 1     | 0x8000  | bit 15                                    |
| Exponent| 6     | 0x7E00  | bits 14..9, bias = 31                     |
| Mantissa| 9     | 0x01FF  | bits 8..0                                 |

**Decoded value:** `(-1)^S * 2^(E - 31) * (1 + M / 2^9)`
(special cases: signed zero on E=0,M=0; subnormals on E=0,M!=0; +/-inf on
E=63,M=0; NaN on E=63,M!=0).

**Source:** `specs/numeric/gf16.t27`, `specs/numeric/formats.t27`,
`conformance/FORMAT-SPEC-001.json` (field `formats.GF16`).

### 1.2 phi anchor (L5 IDENTITY)

`FORMAT-SPEC-001.json` records the following IEEE f64 evidence for the
phi identity:

| Quantity            | f64 hex                  | Decimal             |
|---------------------|--------------------------|---------------------|
| `phi`               | `0x1.9E3779B97F4A8p+0`   | 1.6180339887498948...|
| `phi^2`             | `0x1.4F1BBCDCBFA54p+1`   | 2.6180339887498948...|
| `phi + 1`           | `0x1.4F1BBCDCBFA54p+1`   | 2.6180339887498948...|
| `phi^2 - (phi + 1)` | `0.0` (exact in f64)     | 0                   |

Tolerance for any other tolerance-using check: `1e-15`.

Identity: **`phi^2 + 1/phi^2 = 3`**, exact in the ternary world, witnessed in
f64 to within the tolerance above.

---

## 2. GoldenFloat family (all widths)

All widths are sign-magnitude with a single-stage normalised representation.
Bias is `2^(exp-1) - 1`. Source: `FORMAT-SPEC-001.json` field `formats.*`.

| Format | Bits | S | E  | M  | Bias  | phi distance | Status (this repo)        |
|--------|------|---|----|----|-------|--------------|---------------------------|
| GF4    | 4    | 1 | 1  | 2  | 0     | 0.118        | SPEC                      |
| GF8    | 8    | 1 | 3  | 4  | 3     | 0.132        | SPEC                      |
| GF12   | 12   | 1 | 4  | 7  | 7     | 0.047        | SPEC                      |
| **GF16** | 16 | 1 | 6  | 9  | 31    | 0.0486       | **PRIMARY, SIM-level**    |
| GF20   | 20   | 1 | 7  | 12 | 63    | 0.035        | SPEC                      |
| GF24   | 24   | 1 | 9  | 14 | 255   | 0.025        | SPEC                      |
| GF32   | 32   | 1 | 12 | 19 | 2047  | 0.014        | SPEC                      |

"phi distance" is the L5-IDENTITY-related metric documented in
`FORMAT-SPEC-001.json` (`phi_dist`) -- smaller is closer to the
phi-optimal exp/mant split.

Status column follows [`STATUS.md`](STATUS.md) levels (SPEC / RTL / SIM /
SYNTH / GDS-TAPEOUT / SILICON).

---

## 3. Ternary path -- TF3

`specs/numeric/tf3.t27` -- balanced ternary representation
`{-1, 0, +1}`, also expressed in the 27 Coptic register ISA
(`specs/isa/` and `specs/fpga/ternary_isa.t27`). Generated to
`gen/verilog/numeric/tf3.v`.

Used for the 32-PE mesh on `tt-trinity-gamma`. Not a floating-point format.

---

## 4. Compatibility path -- FP8

**Purpose:** interop with mainstream low-bit AI accelerators
(e.g. NVIDIA Hopper / Blackwell FP8, AMD MI300 OCP-FP8). t27 does **not**
ship its own FP8 implementation today; the listed widths above are the
GoldenFloat family.

**Intent (not implemented in this repo):**

- Provide a `fp8 <-> GF8` and `fp8 <-> GF16` bridge in `specs/numeric/`.
- Reuse the same `FORMAT-SPEC-001.json` framing: a future entry
  `formats.FP8_E4M3` and `formats.FP8_E5M2` would sit alongside the
  GoldenFloat rows.

When implemented, the bridge SHOULD live in `specs/numeric/fp8_bridge.t27`
with conformance vectors under `conformance/fp8_*_vectors.json`. Until then,
the FP8 entry is marked **PLANNED**, not SPEC.

---

## 5. Quantisation bridge -- NF4 / INT4 / INT8

**Purpose:** consume quantised checkpoints from the open weights ecosystem
(NF4 weights from `bitsandbytes`-style flows, INT4/INT8 from GPTQ / AWQ /
GGML / TFLite) and feed them into the gamma mesh's ternary MAC.

**Current state in this repo:**

- `specs/numeric/formats.t27` -- has the conversion utility skeleton but
  is FP-centric (GF16 <-> f32), not yet INT-centric.
- `specs/benchmarks/ternary_vs_binary.t27` -- comparison harness for
  ternary against low-bit binary, useful as a target for the bridge.
- No `int4_bridge.t27` / `nf4_bridge.t27` / `int8_bridge.t27` exists yet.

**Intent:** a single `specs/numeric/quant_bridge.t27` that performs
`{NF4, INT4, INT8} -> {TF3, GF16}` with conformance vectors. Treated as
**PLANNED**.

The reference for the related research line (1-bit / 1.58-bit ternary LLM
weights) is BitNet b1.58: https://arxiv.org/abs/2402.17764 .

---

## 6. GF formats as research differentiator

The GoldenFloat family is presented as a **research-grade numeric** rather
than a production replacement for FP16/BF16/FP8. The honest claim is
narrow:

- GF widths use a **phi-driven exp/mant split**, recorded as `phi_dist` in
  `FORMAT-SPEC-001.json`.
- The split is **derived from a single identity** (`phi^2 = phi + 1`) rather
  than picked per width by industry preference.
- The same identity is used as a CI gate (L5 IDENTITY).

This is interesting because it is reproducible and inspectable. It is **not**
a claim of superior throughput, lower error, or better convergence at any
specified workload. Such claims, if they appear, must cite a benchmark file
in this repo (see [`BENCHMARKS.md`](BENCHMARKS.md)).

---

## 7. Cross-references

- JSON SSOT: `conformance/FORMAT-SPEC-001.json`
- Schema: `schemas/numeric-format-v1.json`
- Primary spec: `specs/numeric/gf16.t27`
- Generated Verilog: `gen/verilog/numeric/gf16.v`
- Hand-built top: `fpga/vivado/gf16_top.v`, `fpga/vivado/gf16_matmul4x4_top.v`
- Family overview: `specs/numeric/goldenfloat_family.t27`
- Identity proofs: `coq/Kernel/` and `proofs/sacred/`

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
