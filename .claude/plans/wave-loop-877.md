# Wave Loop 877 — Plan

| Field | Value |
|-------|-------|
| Wave | 877 |
| Issue | #1702 |
| Branch | `wave-loop-877` |
| Base | `wave-loop-876` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[573][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,672 elements × 32 bits = 1,173,504 bits (~1.120 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[573][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline/HBM bandwidth) and
   confirm no new tooling regressions appeared in W876.
2. **Generator** — copy `scripts/gen_w876.py` → `scripts/gen_w877.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w877_bench_module_573x2p6_aos_var_call_write.t27`
   - module header f-string → `w877_bench_module_...`
   - `OUTER = 573`, `MID_IDX = 286`
3. **Spec** — run `python3 scripts/gen_w877.py` to produce
   `specs/scratch/w877_bench_module_573x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w877_bench_module_573x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W876 (337/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W877_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1702`, push branch, open PR.

## Constants for generator

```python
OUTER = 573
TOTAL = OUTER * 2 ** 6          # 36,672
LAST_IDX = OUTER - 1            # 572
MID_IDX = OUTER // 2            # 286
```

## Cooperation variants for W878

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[575][2]^6 Pt` | 575 | `[2]^6` | 36,800 | 1,177,600 | ~1.124 | Continue mechanical `outer += 2` ladder. |
| **B** | `[573][3]^6 Pt` | 573 | `[3]^6` | 55,008 | 1,760,256 | ~1.678 | Grow second inner dimension, stress stride scaling. |
| **C** | `[573][2]^6 Pt` with negative-index writes | 573 | `[2]^6` | 36,672 | 1,173,504 | ~1.120 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
