# Wave Loop 872 — Plan

| Field | Value |
|-------|-------|
| Wave | 872 |
| Issue | #1691 |
| Branch | `wave-loop-872` |
| Base | `wave-loop-871` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[563][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,032 elements × 32 bits = 1,153,024 bits (~1.100 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[563][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W871.
2. **Generator** — copy `scripts/gen_w871.py` → `scripts/gen_w872.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w872_bench_module_563x2p6_aos_var_call_write.t27`
   - module header f-string → `w872_bench_module_...`
   - `OUTER = 563`, `MID_IDX = 281`
3. **Spec** — run `python3 scripts/gen_w872.py` to produce
   `specs/scratch/w872_bench_module_563x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w872_bench_module_563x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W871 (332/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W872_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1692`, push branch, open PR.

## Constants for generator

```python
OUTER = 563
TOTAL = OUTER * 2 ** 6          # 36,032
LAST_IDX = OUTER - 1            # 562
MID_IDX = OUTER // 2            # 281
```

## Cooperation variants for W873

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[565][2]^6 Pt` | 565 | `[2]^6` | 36,160 | 1,157,120 | ~1.104 | Continue mechanical `outer += 2` ladder. |
| **B** | `[563][3]^6 Pt` | 563 | `[3]^6` | 54,048 | 1,729,536 | ~1.650 | Grow second inner dimension, stress stride scaling. |
| **C** | `[563][2]^6 Pt` (neg-index writes) | 563 | `[2]^6` | 36,032 | 1,153,024 | ~1.100 | Negative-index / wrap-around writes. |

*φ² + φ⁻² = 3 | TRINITY*
