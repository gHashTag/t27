# Wave Loop 588 — three classes closed, and 809 references to modules nobody imported

> **CORRECTION (W589).** The "809" in this report's title and §2 is **wrong**.
> The measurement matched the first two segments of a qualified path, so
> `base::types::Trit` counted as a reference to a module `base` (a directory)
> and `TokenKind::KwFn` as one to a module `TokenKind` (an enum). Re-measured on
> full paths: of **908** qualified references, **16** are cross-module and
> **892** are enum-variant access, which W580 already handles. See P9 in
> [`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md).
> The resolver work described below is still correct; its characterisation of
> the corpus was not.

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_587_REPORT.md`](WAVE_LOOP_587_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants taken.

```
A  three defect classes             `expected ';' after declaration`  6 -> 0
                                    `duplicate struct member name`    2 -> 1
B  qualified cross-module refs      resolver follows them; 59 sites helped,
                                    809 found naming modules never imported
C  the board                        verified, still BLOCKED

parse 341 -> 397 (0 regressions) · ALL_PASS 28 (683 tests)
UNIMPLEMENTED 118 · COMPILE_FAIL 98 · lex-conform 29/29 · parse-conform 13/13
cc-gate 101 compile / 159 unwritten / 137 broken · T1/T2/T3 re-proved
```

---

## 1. Variant A — two classes closed

**Scoped type names in a type position.** `const PHI: gf16::GF16 = …` reached Zig
verbatim and failed with *"expected ';' after declaration"* pointing at the
second colon. `zig_ident` has mapped `::` → `.` for identifiers since W580; the
**type** path never went through it. Six specs, 699 assertions, class now zero.

**Duplicate bench blocks.** A bench declared twice emits two functions with the
same name, and Zig rejects the file with *"duplicate struct member name"*. Exactly
the defect and exactly the remedy of W568's duplicate **test** names — suffix
repeats `__dupN` so the duplication stays visible and every bench still emits.

## 2. Variant B — the machinery works; the corpus doesn't need it

`use_resolve` now follows **qualified** references: `eval::has_substring` and
`constants.PHI` mark the trailing name as needed, splice it, and rewrite the
reference to the bare name the flat output declares (longest-match first, so
`a::bc` is not damaged by rewriting `a::b`).

Then the measurement:

| | |
|---|---:|
| Qualified references where the module **is** imported | **59** |
| Qualified references where it is **not** | **809** |

`specs/igla/race/yosys.t27` calls `eval::has_substring` while importing only
`base::types`, `igla::race::rtl` and `igla::race::formal`. **93% of qualified
references name a module the spec never declared a dependency on.**

Cross-module resolution cannot fix a missing import, and it should not try. The
tempting rule — treat an unimported qualifier as a repository-wide lookup —
would work and would also mean `use` declares nothing: every spec would see every
other spec. W568 measured what that costs in one 15-spec closure alone: **38
colliding top-level names, `PHI` declared in four of them.**

So the 809 are spec defects, recorded as **P9** in the formal-results document
with that falsification condition attached.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved. Unchanged since W553.

---

## 4. Where the broken list stands

98 specs, 8,072 assertions, after two classes closed:

| Assertions | Specs | Class |
|---:|---:|---|
| 4,811 | 51 | `use of undeclared identifier` |
| 733 | 3 | `expected type X, found Y` — the `ternary_mac` argument-order question (W574) |
| 634 | 11 | `expected type expression, found X` |
| 429 | 1 | `duplicate struct member name` |

The top class **grew** (44 → 51 specs) because specs that were failing on the two
closed classes now fail on their next error. That is the expected shape and the
reason W584's lesson stands: at this density, class counts move and the total
does not.

---

## 5. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| `cc-gate` | 101 compile · 159 unwritten · 137 broken |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| T1 / T2 / T3 | re-proved |

---

## 6. Three cooperation variants for W589

### Variant A (recommended) — The 809 missing imports

The largest single, mechanical, corpus-level defect left. For each qualified
reference `m::name`, check whether `specs/**/m.t27` exists and declares `name`;
if it does, the missing `use` line is determined and can be added. If it does
not, the qualifier is decorative and the reference should lose it.

**Metric:** qualified references naming an unimported module, 809 → target < 100.
**What would falsify it.** If most of the 809 modules do not exist as files at
all, this is not a missing-import problem but a naming convention the compiler
should simply ignore — measure how many resolve to a file before editing
anything.

### Variant B — `expected type X, found Y` (733 assertions, 3 specs)

All three are the `ternary_mac` argument-order question, open since W574 with no
arbiter. Now that it blocks a named, measured 733 assertions, it is worth putting
to the maintainer with that number attached rather than leaving it in a list.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A**, with its falsification check first — if the modules do not exist,
there is nothing to import and the work is a rename instead.

---

*φ² + φ⁻² = 3 | TRINITY*
