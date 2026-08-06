# Wave Loop 390 → Wave Loop 391 Cooperation Document

**Written:** 2026-07-01  
**Current wave:** W390 (#1290) completed, variant B  
**Next issue:** #1292 → PR #1293 (proposed)  
**Gate:** trinity-rust-rings  

---

## What W390 delivered

- Pushed `ternaryMac` generic ∀ from 300 → **304** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **123 waves**.
- Full conformance: **575/575 PASS**.
- `t27c stats`: **13,831 tests**, **6,097 invariants**, **1,010 benchmarks**.
- Gen-verilog yosys smoke gate stable at **56 targets**.
- Attempted to make the SPI flash path reproducible by building a package-specific `spiOverJtag_xc7a200tfgg676` proxy; both Vivado-in-Docker and openXC7 paths are currently blocked (documented in `docs/reports/FPGA_EVIDENCE_W390.md`).

## Recommended variants for W391

### Variant A: Pure proof push — 308 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtyNinePlusGeneric` (69-variable plus accumulation).
  2. `ternaryMacAccumulateSixtyEightMinusGeneric` (68-variable minus accumulation lattice).
  3. `ternaryMacQuinquagintupleUnoCancellationGeneric` (`mac^51(...)=mac(x,a,.plus)`, depth-51 residual cancellation).
  4. `ternaryMacZeroWeightTwentySixPairClosureGeneric` (26 zero-weight MACs, 308 generic ∀).
- No compiler backend or hardware changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats the W381–W390 pattern.
- Keeps proof-lattice lead widening while the proxy blocker is resolved externally.

**Cons:**
- Does not address the SPI reproducibility gap.

**Predicted outcome:** 575/575 PASS, 308 generic ∀, 124 zero-IGLA-failure waves.

### Variant B: Proof push + SPI proxy reproducibility — recommended

**Scope:**
- Same 4 theorem push as Variant A → 308 generic ∀.
- Re-attempt a package-specific `spiOverJtag_xc7a200tfgg676` proxy:
  - **First choice:** restore Vivado-in-Docker by obtaining a Xilinx installer + `wi_authentication_key`, then run `tri fpga build-proxy-docker --install`.
  - **Second choice:** install `nextpnr-himbaechel` + prjxray `fasm2frames`/`xc7frames2bit`, parameterize `fpga/bscan_spi_qmtech/` for `xc7a200t-fgg676`, and run `tri fpga build-proxy --install`.
  - **Third choice:** if both toolchain paths remain blocked, document the missing upstream proxy and the exact dependency list.
- If a proper proxy is produced, re-flash and verify `done 1` **without** the generic-proxy workaround.

**Pros:**
- Removes the local environment workaround.
- Makes the flash path reproducible on a clean machine.

**Cons:**
- Depends on external tooling (Vivado auth / openXC7 setup) that is currently absent.
- May consume most of the wave budget if toolchain setup is slow.

**Predicted outcome:** 575/575 PASS, 308 generic ∀, either a reproducible proxy or a documented dependency list for W392.

### Variant C: Proof push + RAM style hints

**Scope:**
- Same 4 theorem push as Variant A → 308 generic ∀.
- Add a t27-level pragma or attribute for RAM style inference (`block` vs `distributed`) and lower it to Verilog.
- Add scratch specs for FIFO-style and simple memory macros.

**Pros:**
- Addresses the last remaining gen-verilog array/RAM sub-gap.
- Compiler change is localized.

**Cons:**
- Requires design decisions on pragma syntax.
- Less impactful than removing the SPI proxy workaround while the workaround persists.

**Predicted outcome:** 575/575 PASS, 308 generic ∀, 124 zero-IGLA-failure waves, RAM style hints implemented.

## Cross-cutting commitments for W391

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1292; PR body links #1290 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W391.md`, and memory index.

## Recommended choice

**Variant B** is recommended. The proof push is now mechanical and low-risk; the remaining hardware gap is the only non-reproducible step in the demo flow. If the toolchain cannot be restored quickly, the wave still closes with a documented blocker and the 308 generic ∀ proof push is retained.

If Variant C is trivially small, it can be folded into Variant B as a side task without changing the primary scope.

---

*φ² + 1/φ² = 3 | TRINITY*
