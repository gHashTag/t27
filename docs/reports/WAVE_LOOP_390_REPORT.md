# Wave Loop 390 Close-Out Report

**Date:** 2026-07-01  
**Branch:** `trinity-rust-rings`  
**Tracking issue:** #1290  
**Selected variant:** Variant B (proof push to 304 `ternaryMac` generic ∀ + SPI proxy reproducibility attempt)  
**Commit:** (see `git log` on `trinity-rust-rings`)  

---

## Summary

Wave Loop 390 executed the recommended **Variant B** scope: it pushed the `ternaryMac` Lean 4 generic ∀ proof lattice from 300 to **304**, extended the IGLA CODER+RACE zero-failure streak to **123 waves**, and attempted to make the SPI flash path reproducible by building or obtaining a package-specific `spiOverJtag_xc7a200tfgg676.bit.gz` proxy.

The proof and conformance work completed cleanly with no compiler backend changes. The SPI proxy reproducibility attempt was **blocked**: the Vivado-in-Docker image does not exist locally and the Xilinx installer/authentication token are not present; the openXC7 open-source toolchain path requires `nextpnr-himbaechel` and the prjxray database/frames tools, none of which are installed on this workstation. The W389 generic-proxy workaround therefore remains in place and is documented as the current dependency.

Full conformance reached **575/575 PASS** and `t27c stats` reports **13,831 tests** and **6,097 invariants**.

## Quantified results

| Metric | W389 | W390 | Δ |
|---|---|---|---|
| Lean 4 `ternaryMac` generic ∀ | 300 | **304** | +4 |
| Pool A floor | 132 | **133** | +1 |
| CODER minimum | 122 | **123** | +1 |
| Pool B depth (`systolic_ternary`) | 150 | **151** | +1 |
| Integration depth (`ternary_inference`) | 131 | **132** | +1 |
| Full-repo tests | 13,777 | **13,831** | +54 |
| Full-repo invariants | 6,070 | **6,097** | +27 |
| Conformance specs | 575 | **575** | 0 |
| Conformance pass rate | 575/575 | **575/575** | 100% |
| Gen-verilog yosys smoke targets | 56 | **56** | 0 |
| Zero-IGLA-failure streak | 122 waves | **123 waves** | +1 |
| SPI proxy reproducibility | Workaround | **Blocked** | documented |

## New theorems in `proofs/lean4/Trinity/TernaryInference.lean`

Wave Loop 390 added **4** new `ternaryMac` generic ∀ theorems:

1. `ternaryMacAccumulateSixtyEightPlusGeneric` — 68-variable plus accumulation (**301 generic ∀ milestone**).
2. `ternaryMacAccumulateSixtySevenMinusGeneric` — 67-variable minus accumulation lattice.
3. `ternaryMacQuinquagintupleCancellationGeneric` — `mac^50(x, a, [.plus,.minus,...]) = x` (depth-50 identity cancellation).
4. `ternaryMacZeroWeightTwentyFivePairClosureGeneric` — 25 zero-weight MACs before and after a plus-weight MAC are transparent (**304 generic ∀ milestone**).

`lake build Trinity.TernaryInference` completed successfully.

## FPGA / hardware — SPI proxy reproducibility attempt

### Goal

Remove the W389 local workaround (copying the generic `spiOverJtag_xc7a200t.bit.gz` to the package-specific `spiOverJtag_xc7a200tfgg676.bit.gz` name) by building or obtaining a proper FGG676 package proxy for the XC7A200T.

### Attempted paths

1. **Vivado-in-Docker (`tri fpga build-proxy-docker`)**
   - Requires a local `t27/vivado:webpack` image.
   - No Vivado image exists on the workstation: `docker images` shows none matching `vivado`.
   - Building the image requires the Xilinx Vivado ML Standard 2025.2 installer `.bin`, `docker/install_config.txt`, and a valid `wi_authentication_key`. None of these are present in the repository or on the host.
   - Result: **blocked**.

2. **openXC7 open-source toolchain (`tri fpga build-proxy` / `fpga/bscan_spi_qmtech/Makefile`)**
   - The in-tree `bscan_spi_qmtech` design is hard-coded for the XC7A100T-FGG676 package and the `tri fpga build-proxy` subcommand targets the same part.
   - Re-targeting to the XC7A200T-FGG676 requires a `xc7a200t` chipdb for `nextpnr-himbaechel` plus the prjxray `fasm2frames` / `xc7frames2bit` tools.
   - `nextpnr-himbaechel` is not installed; `tri fpga setup-openxc7-chipdb --family xc7a200t` can build it, but it also depends on the prjxray database (~1 GiB) and does not provide `fasm2frames` / `xc7frames2bit`.
   - Result: **blocked**.

3. **Upstream openFPGALoader contribution**
   - openFPGALoader 1.1.1 ships generic `spiOverJtag_xc7a200t.bit.gz` but not a package-specific `spiOverJtag_xc7a200tfgg676.bit.gz`.
   - A proper proxy requires either the Vivado `spiOverJtag` build flow or an openXC7 port of the same bridge.
   - Result: **not completed this wave**; tracked as the next external dependency.

### Current state

- The W389 workaround file remains in place:
  `/opt/homebrew/Cellar/openfpgaloader/1.1.1/share/openFPGALoader/spiOverJtag_xc7a200tfgg676.bit.gz`
  (a copy of the generic `spiOverJtag_xc7a200t.bit.gz`).
- Flash programming with `openFPGALoader -c digilent_hs2 -f fpga/verilog/ternary_mac_demo_top_200t.bit --fpga-part xc7a200tfgg676` continues to work on this workstation.
- The bitstream in SPI flash on the XC7A200T board is unchanged and valid.

### Interpretation

The flash path is operational but not yet reproducible from a clean environment. The dependency chain is:

1. Obtain a Xilinx Vivado installer + authentication token, **or**
2. Install `nextpnr-himbaechel` + prjxray database + `fasm2frames`/`xc7frames2bit` and adapt `fpga/bscan_spi_qmtech/` to the XC7A200T package, **or**
3. Upstream the missing proxy file to openFPGALoader.

See `docs/reports/FPGA_EVIDENCE_W390.md` for exact commands, environment state, and next-step dependencies.

## CI smoke gate

- The in-runner smoke gate covers all 27 IGLA specs plus 29 scratch specs = **56 targets**.
- No new scratch spec this wave; gate unchanged.

## Seal / conformance

- 27 core IGLA seals regenerated from `/Users/playra/t27` using `t27c seal --save`.
- Full suite result: **575/575 PASS**, zero seal mismatches, zero yosys smoke failures.
- `t27c stats`: 13,831 tests, 6,097 invariants, 1,010 benchmarks.

---

*φ² + 1/φ² = 3 | TRINITY*
