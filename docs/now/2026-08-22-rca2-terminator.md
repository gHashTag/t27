# NOW — the test died at parse and never reached its rule

Last updated: 2026-08-22

## Add the statement terminator the r_ca_2 fixture omits (Closes #2399)

- Branch: `fix/2399-rca2-terminator`
- Issue: #2399 · part of #2386

### Что легло

One character in `bootstrap/tests/verilog_array_literal_expr.rs` — `consume([1, 2, 3, 4]);`
— plus one line pruned from `scripts/ci/test-baseline.txt` (373 → 372).

**The test's name is wrong about the cause.** Four probes: `consume(7)` followed by
`return` fails identically, and the same call with a `;` or as the final statement parses.
The array literal is irrelevant; the condition is an expression statement without a
terminator followed by another statement.

With the terminator the test measures what it names: the emitter writes
`consume(0 /* TODO: array literal [1,2,3,4] not yet lowered to Verilog */)` — a real
argument **plus** a comment, which is what R-CA-2 requires.

### Границы честности (BINDING)

- **Array literals in call position are still not lowered.** The emitter says so and emits
  `0`. R-CA-2 is the narrower rule that the argument must not be comment-only. This change
  does not implement the feature and does not claim to.
- **This is 6 of the 7 in #2386. One remains** (`array_param_index_is_element_part_select`).
- Whether the parser should accept newline-terminated expression statements is a **language
  decision**, raised in #2399 and not answered here. 28 of the 154 non-parsing specs fail
  this way, including `specs/server/session.t27`.

### A measurement I nearly got wrong

Sweeping `t27c gen-verilog` over `specs/` reports **649 of 650 failing** — which sounds
catastrophic and means almost nothing, because most specs are not hardware specs and that
backend declines them legitimately. The right instrument is `t27c parse`: **496 parse, 154
do not**, and 496 matches the repository's own recorded baseline.

`t27c corpus`'s own help text warns about this exact class: *"a parser error count moves
three orders of magnitude from one character and RISES when a real defect is fixed (T119)"*.

### Evidence

Mutant at `bootstrap/src/compiler.rs:15553` — the `0` dropped so the argument becomes
comment-only — verified planted, with a local `FROZEN_HASH` reseal. Both tests FAILED;
restored, `2 passed; 0 failed`. Neither `compiler.rs` nor `FROZEN_HASH` is in this diff.
