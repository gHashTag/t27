# Wave Loop 565 Report — the first defects found and fixed

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_564_REPORT.md`](WAVE_LOOP_564_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
                     W560   W561   W562   W563   W564   W565
ALL_PASS                5      7      9     14     14     16
TEST_FAIL               0      0      0      0      2      0
COMPILE_FAIL          194    192    190    184    183    183
tests passing          45     54     64    167    175    209
```

W564 surfaced the project's first two genuinely failing tests. W565 diagnosed
them, found **three** defects, and fixed all three. Both specs now pass
completely.

---

## 1. `validate_instr_format` required fields to exactly *fill* the word

W564 reported R-type's field widths summing to 21 against `total_bits = 32` and
**deliberately left it as a specification decision**. The authoritative encoder
settles it — `specs/fpga/assembler.t27`:

```t27
encode_r_type: (opcode << 26) | (rd << 21) | (rs1 << 16) | (rs2 << 11)
encode_i_type: (opcode << 26) | (rd << 21) | (rs1 << 16) | (imm & 65535)
```

R-type uses **6+5+5+5 = 21 bits of a 32-bit word, with bits 10..0 reserved**.
I-type uses 6+5+5+16 = 32, matching `i_type_format` exactly.

So `total_bits = 32` is correct (it is the word width) and the field widths are
correct (they match the encoder bit-for-bit). **The validator was wrong**:
fields must *fit* the word, not exactly fill it.

```diff
- if (computed != fmt.total_bits) {
+ if (computed > fmt.total_bits) {
```

This was not a guess. W564 refused to choose without evidence; the encoder is
the evidence.

## 2 & 3. Two `u32` overflows in the simulator time conversions

```t27
sim_time_ns:        cycles * 1000000000 / clock_freq_hz
cycles_for_time_ns: ns * clock_freq_hz / 1000000000
```

At `cycles = 100` (and `ns = 1000` with a 100 MHz clock) the intermediate
product is **100,000,000,000** against a `u32` max of 4,294,967,295. The
arithmetic is right; the intermediate does not fit. Both now widen to `u64` and
narrow the small result back.

The second one was only visible after fixing the first — the test run stopped
at test 7, then at test 10.

**Result:** `ternary_isa.t27` — all 29 tests pass. `simulator.t27` — all 13 pass.
Seals regenerated, and the W552 gate accepted them, which it would have refused
had either spec stopped generating.

---

## 2. Where the corpus stands

| | |
|---|---:|
| Substantive assertion clauses written | 11,282 |
| **Tests passing** | **209** |
| Tests failing | **0** |
| Specs fully passing | 16 of 199 |
| Specs blocked by `default_input()` | 169 |

---

## 3. Three cooperation variants for W566

### Variant A (recommended) — Lower keyword-form invariants

**5,163 invariants** still emit `// invariant: X verified (no statements)` — a
comment claiming verification. This is the last large inert population, and the
W559 lowering pattern applies directly: `parse_invariant_block` has the
identical `skip_to_next_top_level()` discard that `parse_test_block` had.

**Discipline that must be repeated:** W558 attempted the test-side version and
broke 19 specs; W559 landed it only because the failing set had been kept as a
fixture. Gate this on the full census (`FAIL <= 317`, zero regressions) and be
prepared to revert.

**Expected finding:** as with the tests, most invariants will not fail — they
will fail to *compile*, and that taxonomy is the real output.

### Variant B — Decide the fate of the 571 template tests

Unchanged since W562 and still the single biggest lever: **169 specs** cannot
compile because of `default_input()`, proved unfixable mechanically. **This has
needed a human decision for four waves.**

### Variant C — Keep draining the compile-failure queue

Remaining measured classes: `expected X, found Y` (7/80), `expected ; after
statement` (5/80), `duplicate test name` (5 specs), `operator <` on enums
(2/80), plus struct-literal syntax (`TernaryWeight{code:1}` should be
`.{ .code = 1 }`).

---

## Recommendation

**Variant A.** It is the largest remaining inert population, the pattern is
proven, and the discipline for landing it safely is written down. **B** is
bigger but is not mine to decide.

---

*φ² + φ⁻² = 3 | TRINITY*
