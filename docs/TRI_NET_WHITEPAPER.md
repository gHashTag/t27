# TRI-NET Whitepaper -- Open High-Assurance Ternary AI Silicon Substrate

> **Status:** Position paper. Reflects the current ambition of the TRI-NET
> line and the state of `t27` (the toolchain product of the line). Every
> silicon-readiness statement here is a strict mirror of `STATUS.md` --
> nothing here outruns that file.
>
> **R5-HONEST:** This document does not promise tape-outs, does not name
> dates, and does not claim parity with any commercial product.

---

## 1. Thesis (one paragraph)

The AI-accelerator industry optimises for TOPS, TOPS/W, and SDK breadth.
TRI-NET is not optimising for any of those. TRI-NET optimises for a
different vector: **how much of an inference accelerator can be made
inspectable, formal-friendly, and reproducible from a sealed
specification?** The line bets that, for a growing class of users
(scientific compute, safety-critical inference, on-device assurance,
regulated deployments), the answer "all of it, from `.t27` spec through
Verilog RTL through Tiny Tapeout shuttle" is more valuable than a few
percent of TOPS/W.

---

## 2. What we are

TRI-NET is a **product line of four open artefacts**:

| Product            | Role                                      | Repo (separate)         |
|--------------------|-------------------------------------------|-------------------------|
| `t27`              | Spec-first toolchain, numeric registry    | this repo               |
| `tt-trinity-phi`   | 1x1 phi-anchor / identity chip            | `tt-trinity-phi`        |
| `tt-trinity-euler` | 8x2 safety / control engine               | `tt-trinity-euler`      |
| `tt-trinity-gamma` | 8x4 32-PE ternary mesh                    | `tt-trinity-gamma`      |

The toolchain product is `t27`. The three chip products use `t27`'s
generators to produce their Verilog from `.t27` specifications, rather than
hand-writing HDL. The numeric kernel -- GoldenFloat GF16 (primary path) and
its family (GF4..GF32) -- lives in `t27` as the L6 CEILING SSOT
(`conformance/FORMAT-SPEC-001.json`).

See `LINEUP.md` for the full four-product map; this whitepaper does not
duplicate it.

---

## 3. What we are not

- **Not a TOPS race.** We have not benchmarked against commercial NPUs
  (Coral Edge TPU, Hailo-8, Axelera Metis, Qualcomm Cloud AI 100 Ultra,
  MediaTek Dimensity 9400+). See `COMPETITORS.md` for the restrained
  posture.
- **Not a private fab line.** The silicon target is the **Tiny Tapeout
  shuttle** (1x1 / 2x2 / 4x4 / 8x4 tiles). No private mask set is
  implied.
- **Not a hosted SDK.** The API surface is **file-based**, not network-
  based; see `docs/TRI_NET_API.md`.
- **Not a closed numeric kernel.** GF16 is fully specified in
  `specs/numeric/gf16.t27`; its evidence vectors are in
  `conformance/gf*_vectors.json`.

---

## 4. Why "high-assurance ternary"

### 4.1 Ternary

Ternary inference (weights in `{-1, 0, +1}` or close cousins like INT1.58)
removes the multiplier from the dominant inner loop. R-SI-1, our keystone
constraint, says **the multiplier-free path must remain multiplier-free**.
Coq witnesses for this -- for the LUT-NPU kernel, the AVS-48 voltage
stacking microcode, the AVS-96 dopamine safety gate, the StochRound
operator, the StochSkipSafe gate, the Int2QuantSafe codebook, the RBB /
FBB / CapBoost triple-decker -- already live in `trios-coq/` and
`coq/`. See `NOW.md` for the running ledger.

### 4.2 High-assurance

Every block on the critical path is:

1. authored as a `.t27` spec with `test` / `invariant` / `bench` blocks
   (L4 TESTABILITY);
2. compiled by a sealed bootstrap compiler whose hash is recorded at
   `bootstrap/stage0/FROZEN_HASH` (L2 GENERATION);
3. emitted to `gen/verilog/` (or `fpga/vivado/`) without hand-edit
   (L2);
4. accompanied by conformance vectors under `conformance/` (correctness,
   not throughput -- see `BENCHMARKS.md`);
5. cross-checked by Coq lemmas where the operator is sacred (the W34..W49
   ledger).

This is the assurance posture. It is verifiable today from this repo's
own files; no external trust step is needed.

### 4.3 Open

- License: Apache-2.0 (root LICENSE, NOTICE).
- Specs: `specs/` -- all `.t27`.
- Schemas: `schemas/` -- JSON Schema, draft-07.
- Conformance: `conformance/` -- JSON SSOT.
- Coq: `coq/`, `trios-coq/`, `proofs/`.
- Numeric SSOT: `FORMAT_REGISTRY.md` + `conformance/FORMAT-SPEC-001.json`.

The Tiny Tapeout shuttle target is itself open; the chip repos publish
GDS / pinout / submission per repo when made.

---

## 5. The numeric kernel

GoldenFloat GF16 is the primary 16-bit format of the line. The bit layout,
decode formula, and identity witnesses live in `FORMAT_REGISTRY.md`. The
canonical Trinity identity holds:

```
phi^2 + 1/phi^2 = 3
```

GF16 vs bfloat16 is a recurring question. We standardise that comparison
in `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` (this package). The protocol is
**distribution-explicit** -- there is no single GF16-vs-BF16 number that
isn't tied to a sampling distribution and a seed.

The GoldenFloat family also includes GF4, GF8, GF12, GF20, GF24, GF32
(see `specs/numeric/`). FP8 and the NF4 / INT4 / INT8 bridges are
**planned** but not yet specified.

---

## 6. The four products in detail

### 6.1 `t27` -- the toolchain

What `t27` actually ships today (mirrored from `STATUS.md`):

- Bootstrap Rust compiler (`bootstrap/`) -- SPEC+, sealed.
- `t27c parse` -- 170+ specs parse -- GREEN.
- `t27c gen-verilog` -- 5/5 FPGA modules synthesise -- SIM.
- `t27c gen` (Zig), `t27c gen-c` -- RTL-equivalent software backends.
- `t27c seal` -- GREEN, 729 sealed artefacts at audit time.
- `./scripts/tri` -- canonical CLI wrapper -- GREEN.

### 6.2 `tt-trinity-phi` -- 1x1 phi anchor

Smallest chip. Witnesses the phi identity in silicon and serves as a
proof-of-life CI gate for the line. Status, pinout, GDS, and Tiny Tapeout
submission live in the chip repo -- not here.

### 6.3 `tt-trinity-euler` -- 8x2 e-engine

Mid-tile. Safety / control. Bounded reasoning. Pairs with `clara-bridge/`
in `t27`. 22FDX is a plausible-future PDK target; see
`docs/22FDX_TOPS_W_PROJECTION.md` for the projection methodology.

### 6.4 `tt-trinity-gamma` -- 8x4 32-PE ternary mesh

Largest tile. Inference compute volume of the line. Pairs with the
LUT-NPU operator (`OP_LUT_NPU = 0xE3`), the StochSkipSafe theta gate
(`L2_DG_THETA_SKIP_GATE`), and the AVS-48 / AVS-96 voltage stacking
microcode (see `trios-coq/Physics/`). Triple-Deck RBB / FBB / CapBoost is
specified at the Coq level (W47 / W48 / W49). The chip-side Triple-Deck
implementation lives in this repo's chip-sibling tree.

---

## 7. Who is this for

- **Scientific compute and formal-verification teams** that need a
  multiplier-free, inspectable inference path with Coq-backed identities.
- **Regulated-deployment auditors** that need spec-to-RTL traceability
  (every Verilog comes from a sealed `.t27`).
- **Open-shuttle researchers** that need a Tiny Tapeout-shaped silicon
  target.
- **Educators** that need a small, complete, ternary-AI stack to teach
  from.

It is **not for** users whose primary constraint is TOPS/W or SDK
ergonomics -- see `COMPETITORS.md` for the honest framing.

---

## 8. Roadmap and how it is governed

Roadmap items live in `docs/ROADMAP.md` and the running PHI-LOOP ledger
in `NOW.md`. Every roadmap item:

- has a tracking issue (L1 TRACEABILITY);
- is referenced by `Closes #N` in its PR;
- ships as `.t27` spec + generated `gen/` + conformance + Coq lemma
  (where applicable);
- enters STATUS.md at the appropriate readiness level only when in-repo
  evidence exists.

22FDX TOPS/W projection: `docs/22FDX_TOPS_W_PROJECTION.md` (this package).
Zenodo bundles plan: `docs/ZENODO_BUNDLES.md` (this package).

---

## 9. Cross-links

- `LINEUP.md` -- product map
- `STATUS.md` -- readiness ladder
- `BENCHMARKS.md` -- restrained benchmark posture
- `COMPETITORS.md` -- honest positioning
- `FORMAT_REGISTRY.md` -- numeric SSOT mirror
- `CLARA_TRACEABILITY.md` -- assurance bridge to DARPA CLARA
- `docs/T27-CONSTITUTION.md` -- constitutional stack (L1..L7)
- `docs/TRI_NET_API.md` -- API contract for external integrators
- `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md` -- numeric comparison protocol
- `docs/22FDX_TOPS_W_PROJECTION.md` -- silicon energy projection
- `docs/ZENODO_BUNDLES.md` -- DOI bundle plan
- `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` -- 2026 t27-side roadmap (CL / EN / SN / PUB / OS)

---

## 10. R5-HONEST closing

This whitepaper is a positioning document, not a marketing document. It
sets out **what TRI-NET is trying to be** and **what TRI-NET is not**.
Every reader should be able to verify the readiness levels claimed by
reading the repo files cited. If any cell of `STATUS.md` and any
sentence of this whitepaper disagree, **`STATUS.md` wins**; open an
issue.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
