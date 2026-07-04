# Wave Loop 406 — CCLK measurement + formal timing-safety (variants A/B/C)

**Issue:** #1313  
**Branch:** `wave-loop-406`  
**Milestone:** W405 closed the flash-boot cold-POR smoke gate. W406 should close
one of the remaining gaps: real CCLK measurement on P12, fully automated
cold-POR, or formal OSCFSEL/CCLK timing-safety in Lean 4.

---

## Goal

1. **Variant A** — Capture the actual CCLK frequency/duty cycle on pin P12 and
   record it in `fpga/HARDWARE_SSOT.md` §3.6.
2. **Variant B** — Automate the cold-POR flash-boot smoke gate with a relay
   power switch and isolated JTAG cable so no operator is required.
3. **Variant C** — Extend the Lean 4 model with `OSCFSEL` constants, nominal
   CCLK ranges, and a `cclk_within_flash_spec` predicate; prove the canonical
   config is timing-safe.

Default recommendation: **Variant A + C bundle** (measurement + formal claim).
If the bench has no logic analyzer wired to P12 when implementation starts,
Variant A becomes "live-capture infrastructure + manual CSV evidence" and the
formal Variant C is still deliverable.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| We do not know the real CCLK frequency of the default bitstream | Competitors can ask "how fast is CCLK?" and we cannot answer with data | Variant A measures P12; Variant C bounds it formally; together they close the question |
| `tri fpga measure-cclk` only parses CSV, it cannot drive a live capture | Operator has to use a separate GUI and remember export format | Add `--live` mode that drives `sigrok-cli` with the connected FTDI/DSLogic analyzer |
| CSV parser only handles analog `Time,Voltage`; sigrok logic exports are `logic` + `Samplerate` | Live capture output cannot be parsed | Extend parser to support logic-analyzer CSV and compute freq/duty from samples |
| Formal model assumes `STARTUPCLK=CCLK` but does not quantify it | Timing-safety claim is incomplete | Variant C adds published Artix-7 `OSCFSEL` tables as axiomatic bounds and links them to the N25Q128 spec |
| No validation that measured CCLK is inside the flash spec | A "works on my bench" value may be marginal across temp/voltage/process | Add `--validate` to `measure-cclk` and a Lean `cclk_within_flash_spec` predicate |
| Flash-boot smoke gate still needs a human operator | Cannot run in CI without physical intervention | Variant B removes the operator with relay + isolated JTAG (fallback if scope limited) |

---

## Competitor scan

| Competitor / project | Relevant capability | t27 differentiator after this wave |
|---|---|---|
| Verilean | Lean 4 hardware proofs | t27 links `cclk_within_flash_spec` to a real `OSCFSEL=0` bitstream and, when wired, to a measured CCLK on real silicon |
| Sparkle HDL | End-to-end formal + simulation | t27 has physical evidence plus a model, not just simulation |
| openFPGALoader ecosystem | Tooling for flash / SRAM load | t27 wraps it with formal traceability, evidence reports, and timing validation |
| Project Trellis / nextpnr | Open-source bitstream tooling | t27 focuses on Artix-7 boot verification and timing-safety, not place-and-route |

The strongest defensive move is to deliver **Variant A + C together**: a
published measurement and a formal claim form a traceability stack that is hard
to reproduce. If bench wiring is not available during this wave, the formal
Variant C still advances the model and prepares the measurement path for W407.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-406.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | `OSCFSEL` constants, nominal CCLK lookup, N25Q128 spec, `cclk_within_flash_spec`, proof for `OSCFSEL=0` |
| 3 | `cli/tri/src/fpga.rs` (Variant A) | `tri fpga measure-cclk --live` using `sigrok-cli` + `ftdi-la` / `dreamsourcelab-dslogic`; logic-CSV parsing; `--validate` against flash spec |
| 4 | `cli/tri/src/fpga.rs` | Unit tests for logic-CSV parsing and validation |
| 5 | `fpga/HARDWARE_SSOT.md` §3.6 | CCLK spec, measurement procedure, expected value range |
| 6 | `docs/reports/*` | W406 report, evidence, W407 cooperation |
| 7 | `.trinity/experience.md` | W406 learnings |
| 8 | git/PR | squash-merge to master, close #1313, open #W407 |

---

## Acceptance criteria

- [x] AC-A1 (Variant A): a CCLK capture is attempted live; the dominant frequency
      is deferred to W407 because P12 is not wired to a logic-analyzer channel.
      A CSV fallback is documented in §3.6.3.
- [x] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6 contains the expected nominal
      value (2.5 MHz for `OSCFSEL=0`) and the validation range (100 kHz–50 MHz).
- [x] AC-A3 (Variant A): `tri fpga measure-cclk --live` drives `sigrok-cli` and
      returns structured frequency/duty output.
- [x] AC-C1 (Variant C): new Lean 4 lemmas link `OSCFSEL`/CCLK bounds to the
      cold-POR predicate and prove `cclk_within_flash_spec 0`.
- [x] AC-D1: `./scripts/tri test` passes.
- [x] AC-D2: `lake build Trinity.TernaryFPGABoot` passes.
- [x] AC-D3: W406 report + evidence + W407 cooperation variants committed.

---

## Default variant

**Variant A + C bundle**. The bench has a Digilent FTDI cable that `sigrok-cli`
sees as `ftdi-la`, so live capture infrastructure is buildable. If P12 is not
wired when the code is ready, capture a CSV manually in DSView and commit it as
evidence. The formal Variant C does not depend on hardware and closes the
timing-safety gap regardless.

---

*phi^2 + phi^-2 = 3 | TRINITY*
