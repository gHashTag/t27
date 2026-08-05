# Wave Loop 874 — Plan

| Field | Value |
|-------|-------|
| Wave | 874 |
| Issue | #1696 |
| Branch | `wave-loop-874` |
| Base | `wave-loop-873` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[567][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,288 elements × 32 bits = 1,161,216 bits (~1.108 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[567][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W873.
2. **Generator** — copy `scripts/gen_w873.py` → `scripts/gen_w874.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27`
   - module header f-string → `w874_bench_module_...`
   - `OUTER = 567`, `MID_IDX = 283`
3. **Spec** — run `python3 scripts/gen_w874.py` to produce
   `specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w874_bench_module_567x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W873 (334/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W874_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1696`, push branch, open PR.

## Constants for generator

```python
OUTER = 567
TOTAL = OUTER * 2 ** 6          # 36,288
LAST_IDX = OUTER - 1            # 566
MID_IDX = OUTER // 2            # 283
```

## Cooperation variants for W875

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[569][2]^6 Pt` | 569 | `[2]^6` | 36,416 | 1,165,312 | ~1.112 | Continue mechanical `outer += 2` ladder. |
| **B** | `[567][3]^6 Pt` | 567 | `[3]^6` | 54,432 | 1,741,824 | ~1.662 | Grow second inner dimension, stress stride scaling. |
| **C** | `[567][2]^6 Pt` (neg-index writes) | 567 | `[2]^6` | 36,288 | 1,161,216 | ~1.108 | Negative-index / wrap-around writes. |

*φ² + φ⁻² = 3 | TRINITY*
