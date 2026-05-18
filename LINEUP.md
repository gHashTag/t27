# LINEUP.md -- TRI-NET Product Line

> **Positioning, one line:** TRI-NET is an **open high-assurance ternary AI
> silicon substrate** -- not a TOPS race. The line trades raw throughput for
> inspectable specs, reproducible builds, and a formal-friendly toolchain.

This document is the **single page** for "what are the four pieces and how do
they relate?". It links out; it does not duplicate per-repo READMEs.

---

## 1. The four products

| #   | Repo (external, separate) | Role in the line                          | Silicon target            |
|-----|---------------------------|-------------------------------------------|----------------------------|
| 1   | **`t27`** (this repo)     | Spec-first toolchain + numeric format registry | N/A -- toolchain          |
| 2   | `tt-trinity-phi`          | phi-anchor proof / identity chip          | Tiny Tapeout, 1x1 tile     |
| 3   | `tt-trinity-euler`        | e-engine -- safety / control engine       | Tiny Tapeout, 8x2 tiles    |
| 4   | `tt-trinity-gamma`        | gamma-surface 32-PE ternary mesh          | Tiny Tapeout, 8x4 tiles    |

Tiny Tapeout shuttle program: https://tinytapeout.com/chips/

### 1.1 t27 -- the foundation

Spec-first language (`.t27`), bootstrap Rust compiler (`bootstrap/`), generator
backends (`gen/zig/`, `gen/c/`, `gen/verilog/`), conformance vectors
(`conformance/`), seals (`.trinity/seals/`), formal proofs (`coq/`, `proofs/`).

**Primary product of t27:** `.t27 -> Verilog RTL -> Tiny Tapeout` -- a
reproducibility path that lets sibling chip repos generate their RTL from
sealed specifications rather than hand-written HDL.

See [`STATUS.md`](STATUS.md) for what is actually shipped today,
[`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md) for the numeric SSOT, and
[`COMPETITORS.md`](COMPETITORS.md) for honest positioning.

### 1.2 tt-trinity-phi -- 1x1 phi-anchor

Smallest of the three chips. Carries the `phi` identity
(`phi^2 = phi + 1`, `phi^2 + 1/phi^2 = 3`) as an in-silicon witness; serves
as proof-of-life and CI gate for the line's numeric kernel. Status, pinout,
GDS, and Tiny Tapeout submission live in the chip repo.

### 1.3 tt-trinity-euler -- 8x2 e-engine

Mid-tile. Acts as the **safety / control** engine: bounded reasoning, restraint
behaviour, and the gateway through which the gamma mesh's outputs are exposed.
Pairs with the `clara-bridge/` assurance workflow in t27.

### 1.4 tt-trinity-gamma -- 8x4 gamma-surface

Largest tile. A **32-PE ternary mesh** for inference workloads (low-bit MAC,
ternary attention kernels). The compute volume of the line; deliberately not
benchmarked against commercial NPUs (see `BENCHMARKS.md`).

---

## 2. How the four fit together

```
                +-----------------------------+
                |   t27  (this repo)          |
                |   .t27 specs                |
                |   bootstrap compiler        |
                |   FORMAT-SPEC-001 registry  |
                |   conformance vectors       |
                |   coq / clara-bridge        |
                +--------------+--------------+
                               |
                               v   (.t27 -> Verilog RTL)
        +----------------------+----------------------+
        |                      |                      |
        v                      v                      v
+-----------------+  +------------------+  +---------------------+
| tt-trinity-phi  |  | tt-trinity-euler |  | tt-trinity-gamma    |
| 1x1 phi anchor  |  | 8x2 safety/ctrl  |  | 8x4 32-PE ternary   |
+--------+--------+  +---------+--------+  +----------+----------+
         |                     |                      |
         +---------------------+----------------------+
                               |
                               v
                +-----------------------------+
                |   Tiny Tapeout shuttle      |
                |   (open submission)         |
                +-----------------------------+
```

**Edges in this diagram are not asserted as "currently green for all three";**
they describe the **intended** flow. For each chip's actual status, see its
repo. For t27's status, see [`STATUS.md`](STATUS.md).

---

## 3. Why a line and not one big chip

- **Risk isolation.** A 1x1 phi-anchor that fails to bring up does not block
  the 32-PE mesh; an 8x4 that fails physical verification does not invalidate
  the safety engine.
- **Layered evidence.** phi-anchor proves the numeric kernel. euler proves
  control + safety wiring. gamma proves compute volume. Each chip is a
  separate, reviewable artifact.
- **Open-shuttle economics.** Tiny Tapeout's 1x1 / 2x2 / 4x4 / 8x4 tile sizes
  match the line's risk tiers naturally.
- **Format symmetry.** All three chips draw from the same
  `FORMAT-SPEC-001.json` numeric registry maintained in t27 -- one numeric
  surface, three silicon expressions.

---

## 4. Out of scope for this document

- Per-chip status, pinout, schematic, GDS, or tape-out date -- see chip repos.
- Marketing comparisons against commercial NPUs -- see
  [`COMPETITORS.md`](COMPETITORS.md) for the restrained version.
- Benchmark numbers -- see [`BENCHMARKS.md`](BENCHMARKS.md).

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
