# Wave Loop 871 — Plan

| Field | Value |
|-------|-------|
| Wave | 871 |
| Issue | #1690 (expected) |
| Branch | `wave-loop-871` |
| Base | `wave-loop-870` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[561][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,904 elements × 32 bits = 1,148,928 bits (~1.096 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[561][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus packed-vector limits,
   Vitis HLS `compact=bit`, Vericert/CompCert, FPGA Roofline) and confirm no new
   tooling regressions appeared in W870.
2. **Generator** — copy `scripts/gen_w870.py` → `scripts/gen_w871.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w871_bench_module_561x2p6_aos_var_call_write.t27`
   - module header f-string → `w871_bench_module_...`
   - `OUTER = 561`, `MID_IDX = 280`
3. **Spec** — run `python3 scripts/gen_w871.py` to produce
   `specs/scratch/w871_bench_module_561x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w871_bench_module_561x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W870 (331/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W871_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1690`, push branch, open PR.

## Constants for generator

```python
OUTER = 561
TOTAL = OUTER * 2 ** 6          # 35,904
LAST_IDX = OUTER - 1            # 560
MID_IDX = OUTER // 2            # 280
```

## Cooperation variants for W872

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[563][2]^6 Pt` | 563 | `[2]^6` | 36,032 | 1,153,024 | ~1.100 | Continue mechanical `outer += 2` ladder. |
| **B** | `[561][3]^6 Pt` | 561 | `[3]^6` | 53,760 | 1,720,320 | ~1.641 | Grow second inner dimension, stress stride scaling. |
| **C** | `[561][2]^6 Pt` (neg-index writes) | 561 | `[2]^6` | 35,904 | 1,148,928 | ~1.096 | Negative-index / wrap-around writes. |

*φ² + φ⁻² = 3 | TRINITY*
