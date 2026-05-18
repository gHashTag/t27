# STATUS.md -- t27 Readiness Levels

> **Position in TRI-NET line:** t27 is the **spec-to-RTL toolchain and numeric format registry**.
> Sibling chip repositories (separate repos):
> - `tt-trinity-phi`   -- 1x1 phi-anchor proof / identity chip
> - `tt-trinity-euler` -- 8x2 e-engine safety / control engine
> - `tt-trinity-gamma` -- 8x4 gamma-surface 32-PE ternary mesh
>
> See [`LINEUP.md`](LINEUP.md) for the four-product map, and
> [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md) for the numeric SSOT
> shared across the line.

This document records what t27 has actually shipped, at what readiness
level, with conservative wording. Sources are limited to **this repository's
own evidence** (specs, generated files, seals, conformance vectors, CI logs).
Claims about external chip repositories are deliberately omitted here; cross
to those repos for their own STATUS.

---

## 1. Readiness levels (definitions)

t27 uses a six-level scale for any artifact that targets silicon. Each level
is a strict superset of the levels below it.

| Level         | Meaning                                                                                                     |
|---------------|-------------------------------------------------------------------------------------------------------------|
| **SPEC**      | A `.t27` specification exists, parses, has `test` / `invariant` / `bench` per L4 TESTABILITY.               |
| **RTL**       | Verilog generated under `gen/verilog/` (or `fpga/vivado/` hand-built blocks), produced from a `.t27` spec.  |
| **SIM**       | Logic simulation passes (Verilator / Icarus / Yosys check), with conformance vectors when applicable.       |
| **SYNTH**     | Synthesis to a real cell library passes (Yosys `synth_xilinx` or equivalent), no latches, timing reported.  |
| **GDS/TAPEOUT** | Layout closed, DRC/LVS clean, tape-out submission made (e.g. Tiny Tapeout shuttle). Repo-local proof only. |
| **SILICON**   | Physical die received and bring-up reported in writing. **Asserted only on direct device evidence.**        |

A component at level **N** is not claimed at level **N+1** without textual
evidence in this repo. When in doubt, the level is *lowered*, not raised.

---

## 2. Component status (this repo)

### 2.1 Compiler and toolchain

| Component                  | Level   | Evidence                                                                                             |
|----------------------------|---------|------------------------------------------------------------------------------------------------------|
| `bootstrap/` (Stage-0 Rust)| SPEC+   | `bootstrap/stage0/FROZEN_HASH` seal, `cargo build --release` documented in `README.md`.              |
| `t27c parse`               | GREEN   | 170+ specs parse (see README "System Status" table).                                                 |
| `t27c gen-verilog`         | SIM     | 5/5 FPGA modules synthesize per README; Verilog under `gen/verilog/` and `fpga/vivado/`.             |
| `t27c gen-zig`             | RTL-eq  | Backend present under `gen/zig/` for 28 modules; treated as software backend, not silicon.           |
| `t27c gen-c`               | RTL-eq  | Backend present under `gen/c/` for 28 modules; treated as software backend, not silicon.             |
| `t27c seal`                | GREEN   | `.trinity/seals/` contains many sealed artifacts (729 files at audit time).                          |
| `./scripts/tri`            | GREEN   | Canonical CLI wrapper, documented in `README.md` Quick Start.                                        |

### 2.2 Numeric stack (GoldenFloat family)

| Format | Level    | Evidence                                                                                              |
|--------|----------|-------------------------------------------------------------------------------------------------------|
| GF4    | SPEC     | `specs/numeric/gf4.t27`, entry in `conformance/FORMAT-SPEC-001.json`.                                 |
| GF8    | SPEC     | `specs/numeric/gf8.t27`, entry in `FORMAT-SPEC-001.json`.                                             |
| GF12   | SPEC     | `specs/numeric/gf12.t27`, entry in `FORMAT-SPEC-001.json`.                                            |
| **GF16** | **SIM** | **Primary format.** `specs/numeric/gf16.t27`, `gen/verilog/numeric/gf16.v`, `fpga/vivado/gf16_*.v`,   |
|        |          | hand-built top-levels (`gf16_top.v`, `gf16_matmul4x4_top.v`), `conformance/gf*_vectors.json`.         |
| GF20   | SPEC     | `specs/numeric/gf20.t27`, entry in `FORMAT-SPEC-001.json`.                                            |
| GF24   | SPEC     | `specs/numeric/gf24.t27`, entry in `FORMAT-SPEC-001.json`.                                            |
| GF32   | SPEC     | `specs/numeric/gf32.t27`, entry in `FORMAT-SPEC-001.json`.                                            |
| TF3 (ternary) | SPEC | `specs/numeric/tf3.t27`, generated to `gen/verilog/numeric/tf3.v`.                                   |

GF16 is also documented at FPGA top-level under `fpga/vivado/` (Vivado-buildable
top + UART + matmul testbench), which is the most advanced numeric artifact
in this repository. See [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md) for the
full bit layout and interop notes.

### 2.3 FPGA / RTL surface

| Block                          | Level     | Evidence                                                       |
|--------------------------------|-----------|----------------------------------------------------------------|
| MAC unit (`specs/fpga/mac.t27`)| SYNTH     | README: "5/5 FPGA modules synthesize" with `synth_xilinx`.     |
| `gf16_top.v` / `gf16_matmul`   | SYNTH     | `fpga/vivado/build.tcl`, `build_gf16.tcl`, testbenches present.|
| QMTECH XC7A100T profile        | SYNTH     | Listed in README "System Status"; pins IR + XDC emitter green. |
| Arty A7 profile                | SYNTH     | Listed in README "System Status".                              |
| Bitstream artifact in CI       | SYNTH     | README: ".bit uploaded per PR (7-day retention)".              |

> Tiny Tapeout tape-outs (1x1, 8x2, 8x4) live in **sibling repositories** and
> are out of scope for this document. See `LINEUP.md`.

### 2.4 Formal / proof surface

| Surface           | Level    | Evidence                                                                  |
|-------------------|----------|---------------------------------------------------------------------------|
| Coq kernel        | partial  | `coq/Kernel/`, `coq/Theorems/`, `coq/IGLA/`; multiple Wave* commits cite Qed counts (`git log`). |
| Phi identity (L5) | GREEN    | `conformance/FORMAT-SPEC-001.json` records IEEE f64 hex + zero residual.  |
| Sacred physics    | SPEC+    | `proofs/sacred/`, `proofs/trinity/`, `proofs/gravity/`.                   |

### 2.5 CLARA / assurance bridge

| Artifact                | Level    | Evidence                                                |
|-------------------------|----------|---------------------------------------------------------|
| `clara-bridge/`         | demo     | README in `clara-bridge/README.md`, Python examples,    |
|                         |          | scenarios, audit-trail, explainability harness.         |
| Submission package      | draft    | `clara-bridge/submission/` and `clara-bridge/proposal/`.|

See [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md) for how t27 maps to the
public DARPA CLARA program goals.

---

## 3. Conservative status decisions

The following decisions were made when authoring this document, in line with
the rule "when in doubt, lower":

1. **No SILICON claim anywhere in t27.** Silicon belongs to the chip repos, not here.
2. **No GDS/TAPEOUT claim in t27.** Tape-out artifacts live in `tt-trinity-*` repos.
3. **GF16 marked SIM, not SYNTH-on-vendor-cells**, because the Verilog evidence in this repo
   is Yosys-friendly + Vivado scripts; we do not assert a closed vendor flow from t27 alone.
4. **CLARA bridge is "demo / draft"**, not "submitted" -- the repo contains examples and a
   `submission/` directory, but no public award or acceptance evidence is asserted.
5. **Coq surface is "partial"**, even though recent commits add Qed counts; an external
   reviewer should still audit each `*.v` rather than trust the aggregate.

---

## 4. How to verify

Reproducible checks **from this repo only**:

```bash
# Build bootstrap (Rust)
cd bootstrap && cargo build --release && cd ..

# Parse specs
./scripts/tri parse specs/numeric/gf16.t27

# Run the Rust test suite (parse / gen / seal / fixed-point)
./scripts/tri test

# Validate conformance vectors and gen headers
./scripts/tri validate-conformance
./scripts/tri validate-gen-headers

# Inspect format registry
cat conformance/FORMAT-SPEC-001.json
```

For RTL flow, see `fpga/vivado/build.tcl` and `fpga/vivado/build_gf16.tcl`.

---

## 5. Update policy

`STATUS.md` is updated only when **textual evidence in this repo** moves a
component up or down a level. PRs that change levels must cite the file(s),
seal(s), or CI run(s) that justify the change.

**phi^2 + 1/phi^2 = 3  |  TRINITY**
