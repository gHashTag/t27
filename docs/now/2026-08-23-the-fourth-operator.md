# NOW — the fourth operator, and the count that would have been wrong (2026-08-23)

`--boundary` moves a comparison one place: `>` ↔ `>=`, `<` ↔ `<=`. Ratchets, floors and tolerances live on a boundary, and a control that tests *clearly worse* and *clearly better* never tests **equal**.

Yesterday's entry predicted a fourth question would find something, and that twice already it had found the instrument. Both held.

- **First run said 8 of 13 gates had survivors. That number was wrong.** The scanner tracked quote state per line, so every `>` inside a multi-line docstring became a site — prose about ratchets and usage on lines 10, 43, 136 and 230 of four gates. Real count, wrong meaning, in the instrument, on its first run.
- Carrying triple-quote state across lines: **5 gates, 21 mutants, 9 killed, 12 survived.**
- **The survivor count still overstates the gap.** Classified by hand: 2 real closeable thresholds, 4 real semantic, 2 candidate theorems (an infinity is never zero, so both forms agree), 2 cosmetic "(+N more)" truncations, 1 plumbing. Publishing "12 uncovered boundaries" would have been every word measured and the sentence false.
- **One closed:** the catalog floor. Every case in its control tested 0 against a floor of 109 — clearly under — so `n_ssot < MIN_ROWS` rewritten to `<=` passed the whole control while failing every catalog sitting exactly *on* the floor.

**The rule:** a new operator's first number is a claim about the operator, not about the code. Read the surviving sites, not the count.
