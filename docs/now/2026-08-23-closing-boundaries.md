# NOW — closing boundaries, and a classification that was wrong (2026-08-23)

Three of yesterday's twelve boundary survivors are closed, one is corrected, and the correction is the useful part.

- **The catalog floor**: every case tested 0 against a floor of 109 — clearly under — never 109 itself.
- **The ledger ratchet**: its control proved *refuses to grow* (1→2) and *writes on shrink* (2→1) and never went through **equal**, which is where a ratchet is defined.
- **The re-derive tolerance**: every case sat far from `1e-12`. The new one is exact on purpose — a row decoding to its own input gives `rederived = 0.0`, and an `abs_error` of exactly the tolerance makes the difference the tolerance itself with no rounding anywhere.
- **One closed rather than declared**: a `"... and N more"` display guard, classified cosmetic and closed anyway with a case planting exactly ten departed entries. Twice now a written-down limitation here has turned out invented; a declared exception costs a reader more than a case costs to write.

**The classification that was wrong.** `math.isinf(dec) and (inp > 0) == (dec > 0)` was called a *candidate* theorem resting on an unchecked property of the codec. It rests on nothing of the kind: the branch is guarded by `elif math.isinf(inp):` one line above, so both values are infinite by construction and `x > 0` ≡ `x >= 0` for them. A proven equivalence — and I wrote the caveat after reading the comparison and not the guard above it.

**`# mutant-equivalent: <why>`** now prints its reason beside the surviving row. Printed, never acted on: the row still reads SURVIVED and still counts. Suppressing a row on the strength of a comment is how a declared `UNCOVERED` stood for a week while being false.

**And the marker's first implementation was a broken ruler** — marker + 2 lines, against a fifteen-line proof, naming a line inside its own explanation. Third time this campaign that a measuring device was calibrated against the one example in front of it.

State: 13 gates, 3 with boundary survivors, 8 remaining — two of them proven equivalences that now say so.
