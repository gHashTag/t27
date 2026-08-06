# Wave Loop 878 — Plan

| Field | Value |
|-------|-------|
| Wave | 878 |
| Issue | #1706 |
| Branch | `wave-loop-878` |
| Base | `wave-loop-877` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[575][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,800 elements × 32 bits = 1,177,600 bits (~1.124 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[575][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus V13.0 packed-vector
   improvements, Vericert v2.0.0 / PLDI 2024 hyperblock scheduling, Graphiti
   ASPLOS 2026, FPGA Roofline/BRAM bandwidth) and confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w877.py` → `scripts/gen_w878.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27`
   - module header f-string → `w878_bench_module_...`
   - `OUTER = 575`, `MID_IDX = 287`
3. **Spec** — run `python3 scripts/gen_w878.py` to produce
   `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w878_bench_module_575x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W877 (338/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W878_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1706`, push branch, open PR.

## Constants for generator

```python
OUTER = 575
TOTAL = OUTER * 2 ** 6          # 36,800
LAST_IDX = OUTER - 1            # 574
MID_IDX = OUTER // 2            # 287
```

## Cooperation variants for W879

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[577][2]^6 Pt` | 577 | `[2]^6` | 36,928 | 1,181,696 | ~1.128 | Continue mechanical `outer += 2` ladder. |
| **B** | `[575][3]^6 Pt` | 575 | `[3]^6` | 55,200 | 1,766,400 | ~1.685 | Grow second inner dimension, stress stride scaling. |
| **C** | `[575][2]^6 Pt` with negative-index writes | 575 | `[2]^6` | 36,800 | 1,177,600 | ~1.124 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
