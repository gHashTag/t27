# Wave Loop 870 — Plan

| Field | Value |
|-------|-------|
| Wave | 870 |
| Issue | #1688 (expected) |
| Branch | `wave-loop-870` |
| Base | `wave-loop-869` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[559][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,776 elements × 32 bits = 1,144,832 bits (~1.092 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[559][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W869.
2. **Generator** — copy `scripts/gen_w869.py` → `scripts/gen_w870.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w870_bench_module_559x2p6_aos_var_call_write.t27`
   - module header f-string → `w870_bench_module_...`
   - `OUTER = 559`, `MID_IDX = 279`
3. **Spec** — run `python3 scripts/gen_w870.py` to produce
   `specs/scratch/w870_bench_module_559x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w870_bench_module_559x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W869 (330/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W870_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1688`, push branch, open PR.

## Constants for generator

```python
OUTER = 559
TOTAL = OUTER * 2 ** 6          # 35,776
LAST_IDX = OUTER - 1            # 558
MID_IDX = OUTER // 2            # 279
```

## Cooperation variants for W871

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[561][2]^6 Pt` | 561 | `[2]^6` | 35,904 | 1,148,928 | ~1.096 | Continue mechanical `outer += 2` ladder. |
| **B** | `[559][3]^6 Pt` | 559 | `[3]^6` | 53,472 | 1,711,104 | ~1.633 | Grow second inner dimension, stress stride scaling. |
| **C** | `[559][2]^6 Pt` (neg-index writes) | 559 | `[2]^6` | 35,776 | 1,144,832 | ~1.092 | Negative-index / wrap-around writes. |

*φ² + φ⁻² = 3 | TRINITY*
