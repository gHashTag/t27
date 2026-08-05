# Wave Loop 881 — Plan

| Field | Value |
|-------|-------|
| Wave | 881 |
| Issue | #1713 (expected) |
| Branch | `wave-loop-881` |
| Base | `wave-loop-880` HEAD (parent branch because earlier waves' PRs remain open) |
| Variant (selected) | `[581][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 37,184 elements × 32 bits = 1,189,888 bits (~1.135 MiBit) |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[581][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Decomposed work

1. **Research** — refresh weak-point background (Icarus V13.0 packed-vector
   improvements, Vericert/Graphiti verified HLS, FPGA Roofline/BRAM bandwidth) and
   confirm no new tooling regressions.
2. **Generator** — copy `scripts/gen_w880.py` → `scripts/gen_w881.py`; fix the three
   copy-hazard locations before first run:
   - destination path → `specs/scratch/w881_bench_module_581x2p6_aos_var_call_write.t27`
   - module header f-string → `w881_bench_module_...`
   - `OUTER = 581`, `MID_IDX = 290`
3. **Spec** — run `python3 scripts/gen_w881.py` to produce
   `specs/scratch/w881_bench_module_581x2p6_aos_var_call_write.t27`.
4. **Validation gates** — run, in order:
   - `t27c parse ...`
   - `t27c icarus-lowerable ...`
   - `t27c icarus-simulate ...`
   - `t27c icarus-cocotb ...`
   - `t27c seal --save ...`
5. **Integration test** — add `accepts_w881_bench_module_581x2p6_aos_var_call_write`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Rust test** — `cargo test --release --test icarus_lowerable` must pass with
   one more test than W880 (341/0 expected).
7. **Closeout** — write `docs/reports/FPGA_LOOP_CLOSEOUT_W881_...md`, update
   `docs/NOW.md`, `.trinity/experience.md`, `.trinity/current-issue.md`, skill
   trackers, and persistent memory.
8. **Land** — commit with `Closes #1713`, push branch, open PR.

## Constants for generator

```python
OUTER = 581
TOTAL = OUTER * 2 ** 6          # 37,184
LAST_IDX = OUTER - 1            # 580
MID_IDX = OUTER // 2            # 290
```

## Cooperation variants for W882

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[583][2]^6 Pt` | 583 | `[2]^6` | 37,312 | 1,193,984 | ~1.139 | Continue mechanical `outer += 2` ladder. |
| **B** | `[581][3]^6 Pt` | 581 | `[3]^6` | 55,296 | 1,769,472 | ~1.687 | Grow second inner dimension, stress stride scaling. |
| **C** | `[581][2]^6 Pt` with negative-index writes | 581 | `[2]^6` | 37,184 | 1,189,888 | ~1.135 | Exercise wrap-around addressing on a large packed vector. |

*φ² + φ⁻² = 3 | TRINITY*
