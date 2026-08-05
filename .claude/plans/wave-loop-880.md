# Wave Loop 880 — Plan

| Field | Value |
|-------|-------|
| Wave | 880 |
| Issue | #1712 |
| Branch | `wave-loop-880` |
| Base | `wave-loop-879` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[579][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,056 elements × 32 bits = 1,185,792 bits (~1.131 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[579][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus V13.0 packed-vector
   improvements, Vericert v2.0.0 / PLDI 2024 hyperblock scheduling, Graphiti
   ASPLOS 2026, FPGA Roofline/BRAM bandwidth) and confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w879.py` → `scripts/gen_w880.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27`
   - module header f-string → `w880_bench_module_...`
   - `OUTER = 579`, `MID_IDX = 289`
3. **Spec** — run `python3 scripts/gen_w880.py` to produce
   `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w880_bench_module_579x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W879 (340/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W880_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1712`, push branch, open PR.

## Constants for generator

```python
OUTER = 579
TOTAL = OUTER * 2 ** 6          # 37,056
LAST_IDX = OUTER - 1            # 578
MID_IDX = OUTER // 2            # 289
```

## Cooperation variants for W881

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[581][2]^6 Pt` | 581 | `[2]^6` | 37,184 | 1,189,888 | ~1.135 | Continue mechanical `outer += 2` ladder. |
| **B** | `[579][3]^6 Pt` | 579 | `[3]^6` | 55,296 | 1,769,472 | ~1.687 | Grow second inner dimension, stress stride scaling. |
| **C** | `[579][2]^6 Pt` with negative-index writes | 579 | `[2]^6` | 37,056 | 1,185,792 | ~1.131 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
