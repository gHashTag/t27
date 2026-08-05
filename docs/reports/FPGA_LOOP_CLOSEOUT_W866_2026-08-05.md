# Wave Loop 866 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1680](https://github.com/gHashTag/t27/issues/1680)  
**Branch:** `wave-loop-866`  
**Parent:** `wave-loop-865` HEAD (earlier waves' PRs remain open)  
**PR:** TBD  
**Author:** Trinity Agent (Claude Code t27)  

---

## 1. What we built

Wave Loop 866 is the next mechanical rung of the module-scope packed array-of-struct
ladder. We selected Variant A from the W866 plan:

```text
module-scope [551][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `551` is odd and non-power-of-two, keeping the boundary-stress
  pattern established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `551 × 64 = 35,264`.
- Packed vector width: `35,264 × 32 = 1,128,448 bits` ≈ **1.076 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w866.py` | Copied from `gen_w865.py`; copy hazard fixed before first run. |
| Spec | `specs/scratch/w866_bench_module_551x2p6_aos_var_call_write.t27` | 104,751 lines, 2.42 MB. |
| Seal | `.trinity/seals/scratch_w866_bench_module_551x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w866_bench_module_551x2p6_aos_var_call_write`. |

---

## 2. Scientific framing

The Wave Loop ladder is a **memory-quanta probe**: each rung increases the packed
vector width `Q` while keeping the compute kernel identical. In the FPGA Roofline
model (Siracusa et al., *IEEE Transactions on Computers*, 2021,
DOI:10.1109/TC.2021.3111761), widening `Q` moves the working set along the memory
bandwidth axis while the arithmetic intensity roof stays flat. As long as the
compiler (t27c) and the target simulator (Icarus Verilog) accept the wider vector
without structural change, we remain on the "soft" side of the bandwidth wall.

At **1.076 MiBit** we are still deep in the 1-MiBit neighborhood and far below the
established 4-MiBit soft cliff (~131,072 elements for 32-bit structs). The relevant
weak-point watch-points remain:

1. **Icarus Verilog packed-vector limits.** The LRM only mandates 65,536-bit
   packed arrays; modern Icarus warns near 1 Gbit. Upstream commit `128c621` fixed
   a bound-normalization bug that could accidentally create billion-bit vectors.
   Historical Icarus 0.8 had a ~256 K-entry allocator assertion, but current
   versions do not hit it at this scale.
2. **t27c lowering path.** The `[N][2]^6 Pt` pattern exercises multi-dimensional
   array flattening, struct packing, signed-index writes, and function-call
   initialization. No compiler change or `FROZEN_HASH` update was required.
3. **Generator copy hazard.** The recurring stale-reference bug (destination path,
   module header f-string, `MID_IDX` comment) was prevented by pre-run grep.

Reference frames:

- **Vericert / CompCert** — verified compilation from C/LLVM to hardware provides
  the formal analog for bit-exact source-to-FPGA mapping.
- **Vitis HLS UG1399** — `compact=bit` option provides the commercial analog for
  packing structs into bit-vectors.

---

## 3. Validation matrix

| Gate | Command / Test | Result |
|------|----------------|--------|
| Parse | `t27c parse specs/scratch/w866_bench_module_551x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable -- accepts_w866_bench_module_551x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **326/0** |
| FROZEN_HASH | `cat bootstrap/stage0/FROZEN_HASH` | unchanged |

No compiler source changes, no reference-model changes, no FROZEN_HASH change.

---

## 4. Weak-point audit

### Confirmed still healthy

- Icarus 1-MiBit neighborhood: no hard cap hit.
- t27c packed-vector lowering for module-scope AoS variables from calls.
- Signed-index writes into `[N][2]^6 Pt`.

### Still open / non-blocking

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing,
  separate from the Wave Loop ladder).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warning cleanup sprint (~626 warnings).
- Vivado-in-Docker CI gap.
- 30-day commit traceability remains low outside wave-loop commits; continue
  `Closes #N` discipline.
- Generator copy hazard persists; eventual fix is to parameterize `WAVE`/`OUTER`
  in a single template.
- Full `./scripts/tri test` suite still stalls on the pre-existing
  `w589_bench_module_17d_aos_var_call_write.t27` parse phase and was not completed
  this wave.

---

## 5. Cooperation variants for Wave Loop 867

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[553][2]^6 Pt` | 553 | `[2]^6` | 35,392 | 1,132,544 | ~1.080 | Continue mechanical `outer += 2` ladder. |
| **B** | `[551][3]^6 Pt` | 551 | `[3]^6` | 52,992 | 1,695,744 | ~1.615 | Grow second inner dimension, stress stride scaling. |
| **C** | `[551][2]^6 Pt` (neg-index writes) | 551 | `[2]^6` | 35,264 | 1,128,448 | ~1.076 | Negative-index / wrap-around writes. |

Variant A is recommended because it preserves the established outer-dimension
ladder and keeps the next rung well under the 4-MiBit soft cliff.

---

## 6. Next steps

1. Land W866 commit (`Closes #1680`) and open PR to `master`.
2. Create W867 issue and branch `wave-loop-867` from `wave-loop-866` HEAD.
3. Implement selected W867 variant per the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
