# Wave Loop 600 — how much of what compiles is actually right

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_599_REPORT.md`](WAVE_LOOP_599_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W560 answered *"does it compile?"* for the corpus. Forty waves later, this
answers the next question for the specs that do — and the answer is sharper than
expected.

```
1024 tests run · 1018 pass · 6 fail · 99.4%

EVERY FAILING TEST IN THE ENTIRE CORPUS IS IN ONE FILE.
29 of 30 measured specs are at 100%.
```

---

## 1. Three populations, deliberately not merged

The first run of `--all` reported **68 measured** — and 38 of those were
`0/0 — 0.0%`: specs that compile and declare no tests at all. Averaging them in
as zeroes is exactly the collapse W586 spent twenty-five waves' worth of
confusion to undo, when `COMPILE_FAIL` turned out to be *broken* and *never
written* in one bucket. **I made the same mistake in the command's first
version, in a module whose own doc comment warns against it**, and the fix is
the same shape: report the populations separately.

| Population | Count | What it means |
|---|---:|---|
| **MEASURED** | **30** | compiles **and** declares tests — the only ones with a rate |
| **NO TESTS** | **38** | compiles, asserts nothing — an **L4 TESTABILITY** violation |
| **BLOCKED** | **540** | never produced a binary |

Only the first has a pass rate, and the rate below is over those alone.

## 2. The table

| | |
|---|---:|
| Tests run | **1024** |
| Pass | **1018** |
| Fail | **6** |
| Rate | **99.4 %** |
| Specs at 100% | **29 of 30** |

**All six failures are in `specs/igla/race/cordic.t27`**, and all six already
have their arithmetic written down — three gain assertions contradicted by
`K(n)` decreasing to 0.6072529, two arctan-table entries whose index and
expected range disagree by one, and `cordic_sin(π)`, which **T6** proves
unsatisfiable without argument reduction.

There is no second file with a problem. The corpus does not have a long tail of
subtly-wrong specs; it has 540 that do not run and 30 that do, and of those 30,
one has six known assertions to settle.

## 3. Where the passing tests are — and it is the FPGA family

| Family | Specs | Tests | Failures |
|---|---:|---:|---:|
| `specs/fpga/` | 14 | 246 | **0** |
| `specs/igla/` | 6 | 686 | 6 |
| `specs/boards/` | 3 | 54 | **0** |
| `specs/compiler/` | 2 | 17 | 0 |
| `specs/ml/` | 2 | 13 | 0 |
| others | 3 | 8 | 0 |

**`specs/fpga/` and `specs/boards/` together are 17 specs, 300 tests, zero
failures.** `ternary_isa` (29), `gf16_accel` (27), `hir` (26),
`xc7a100t_minimal` (23), `hw_types` (22), `assembler` (19), `stdlib` (17),
`dft` (17), `arty_a7` (17), `e2e_demo` (16) — every one at 100%.

This bears directly on the standing FPGA goal. **The hardware-facing half of the
corpus is the healthiest part of it**, and it is not close: the only failures
anywhere are six assertions in one CORDIC kernel, and T1–T3 (the machine-checked
equivalence, multiplier-freedom, and timing results) already cover the RTL.
Nothing measured here is what blocks the board — that remains a USB cable.

## 4. Verification

| Gate | Result |
|---|---|
| corpus per-test | 1018 / 1024 |
| `cordic.t27` | 330 / 336 |
| `adder_tree.t27` | 335 / 335 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| `cc-gate` | 101 (unchanged) |
| new unit test | `a_blocked_spec_contributes_nothing_to_the_totals` |

---

## 5. Three cooperation variants for W601

### Variant A (recommended) — The 38 specs that assert nothing

L4 says every `.t27` spec must contain `test`/`invariant`/`bench`. **38 specs
compile cleanly and contain none.** They are not broken and not unwritten — they
are the third thing, and this wave is the first time the corpus has been able to
name them, because until `test-report` existed a spec with no tests and a spec
with passing tests were both simply "not failing".

The list is mechanical to produce (`test-report --all --verbose`), is saved at
[`data/W600-specs-with-no-tests.txt`](data/W600-specs-with-no-tests.txt), and
groups cleanly:

| Family | Specs | Note |
|---|---:|---|
| `specs/tri/` | 19 | agent, pipeline, utils, math — the runtime layer |
| `specs/numeric/` | 10 | `gf6`, `gf10`, `gf14`, `gf48`, `gf96`, `gf128`, `gf256`, `gf512`, `gf1024`, `formats_catalog` |
| `specs/sacred/` | 7 | cosmology, gravity, quantum, superconductivity… |
| `specs/ternary/`, `specs/ml/` | 2 | |

The `numeric/gf*` group is the sharpest target: **L6 names `gf16.t27` as numeric
SSOT**, and nine sibling field specs compile while asserting nothing about
themselves. Each entry is small, independent, and carries no judgement call.

### Variant B — Argument reduction for CORDIC (T6)

Unchanged and now sharper: it is **the only remaining code defect in the entire
measured corpus**. T6 gives the algorithm — map θ into [−π/2, π/2] via
`θ' = θ − k·π`, negating both outputs for odd *k*. It changes behaviour, so it
wants an owner's sign-off.

### Variant C — Flash the board

Unchanged: `verdict : BLOCKED -- no programmer on USB`. Worth restating with
this wave's numbers: **the FPGA spec family is at 300/300.** Nothing measurable
stands between the specs and the board except the cable.

---

## Recommendation

**Variant A.** B is one decision that belongs to an owner and C needs hardware;
A is 38 pieces of unblocked work that this wave is the first to be able to
enumerate.

---

*φ² + φ⁻² = 3 | TRINITY*
