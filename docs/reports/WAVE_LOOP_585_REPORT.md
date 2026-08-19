# Wave Loop 585 — the `default_input` wall was a mask over 571 empty functions

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_584_REPORT.md`](WAVE_LOOP_584_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three W585 variants were taken.

```
A  t27c cc-gate            the C measurement is a command, wired into suite Phase 6
B  default_input           first-error count  109  ->  0
C  flash the board         still hardware-blocked; preflight verified

parse 341 -> 397 (0 regressions) · ALL_PASS 28 · 683 tests · assertions 9,267
lex-conform 29/29 · parse-conform 13/13 · truncation 0 · T1/T2/T3 re-proved
```

And a new document collecting what is actually **proved** versus what is
**measured**: [`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md).

---

## 1. Variant A — the C gate is a command

W583 and W584 measured the C backend with a shell loop assembled at a prompt.
`t27c cc-gate` now does it: generates each header (through the same `use`
resolution `gen-c` performs), runs `cc -fsyntax-only`, and reports the
**first-error class table** with the header count underneath.

It reproduces the hand-built numbers exactly — 101 compiling, 296 failing — and
runs in 11 seconds. Suite Phase 6 prints the top three classes on every run.

The class table is first on purpose: W584 established that a header must clear
*every* class, so fixing one moves specs from failing on A to failing on B
without moving the header count at all.

**It immediately caught a regression I had introduced in W584.** The named-tuple
fix split `gf16::GF16` at the first colon, took `gf16` for a field name, and
emitted `typedef struct { :GF16 f0; … }`. One wave, found by the instrument built
the wave after.

## 2. Variant B — the wall came down, and there was nothing behind it

`default_input()` has been the largest blocker in three measurement systems since
W561. The helper is not derivable from its own call — it takes no arguments and
returns whatever the next line needs. But the next line is `f(input)`, and `f`'s
parameter type is **declared**:

```t27
test insert_basic_case
    given input = default_input()      // type unknown here …
    when result = insert(input)        // … but `insert` declares it
    then result != undefined           // and the test constrains it not at all
```

So the binding's type is recoverable from its **use**, and the value is
unconstrained. Codegen now records the consumer's parameter type and lowers the
binding to a typed placeholder.

```
`default_input` as a first error:   109  ->  0
```

**And the next error is what matters.**

| | |
|---|---:|
| Specs carrying `// TODO: Implement from .tri spec` | **169** |
| Functions with an **empty body** across them | **571** |
| Template tests calling the scaffold | 765 |

**571 empty functions. 571 template tests.** One generated test per
unimplemented function. The scaffold generated a test for every function it also
left unimplemented, and the missing helper stood in front of that fact for
twenty-five waves.

`default_input()` was never a blocker in the sense this project believed. It was
a **mask**. What it hid is that 571 declared functions have no implementation —
a specification-completeness fact, not a compiler defect, and one no amount of
backend work can change.

This is the third time in this chain that removing a mask made a counter worse
and the project better (W569's stray brace, W577's truncations, this).

## 3. Variant C — the board

Verified again, unchanged, and honestly still blocked:

```
board      : QMTech Wukong V1 (XC7A200T-FGG676) [wukong-a200t]
expect id  : 0x03636093        cable : digilent_hs2
bitstream  : ternary_mac_demo_top_v2_200t.bit   9,730,764 bytes  [OK]
cable link : ABSENT
verdict    : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

Everything software-side has been ready since W553. T1, T2 and T3 re-proved this
wave. The only external dependency in the entire project is the physical board
and its Digilent HS2 cable.

## 4. The science

[`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md)
separates three things this chain had been mixing:

- **Theorems** (T1–T3), machine-checked, with scripts — including the W574
  caveat that `miter -equiv` binds ports by *name*, so T1 says what the circuit
  computes and nothing about argument order.
- **Measured propositions** (P1–P6), each with method, number and falsification
  condition — including the withdrawn ones.
- **Context**: balanced ternary (Knuth), low-bit networks (BitNet b1.58), CORDIC
  (Volder — and the spec's own `0.6073` gain assertion matches the classical
  constant, a real cross-check), systolic arrays (Kung & Leiserson). Stated from
  general knowledge, with no fabricated citations, and explicit that **the
  arithmetic and the architecture are not novel** — the spec-first pipeline is.

### The conclusion the chain supports

Eighteen waves of findings share one shape: a component accepted input, produced
a smaller or different program, and **reported success**. The parser (four
times), the lexer (once, changing meaning rather than losing code), the C backend
(409 invalid declarations nobody compiled), and finally the scaffold mask itself.

**Not one was found by a test failing.** Each was found by asking a component to
account for its input — did it consume all of it, does it match a written-down
table, does a real compiler accept its output.

> A stage that cannot fail cannot be trusted.

The FPGA track is the counter-example that proves it: correct since W553 because
`yosys` and `nextpnr` are consumers that refuse nonsense. It is the only part of
this project that was never wrong, and the only part with a real consumer.

---

## 5. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| `default_input` first errors | **109 → 0** |
| `t27c cc-gate` | 101 of 397 headers compile; reproduces the hand-built numbers |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 6. Three cooperation variants for W586

### Variant A (recommended) — The 571 empty functions

Now the only thing standing between this corpus and a real test result, and it is
finally stated as itself rather than as a missing helper.

**Deliverables.** Determine whether the `.tri` sources these specs were generated
from still exist (the header comment says *"Implement from .tri spec"*). If they
do, the bodies are recoverable and this is a regeneration task. If they do not,
each is a spec-authoring decision and the honest move is to mark them
`unimplemented` explicitly so the count is a project metric rather than a
compiler error.

**What would falsify it.** If the `.tri` sources exist and already contain the
bodies, this is not a 571-decision problem at all — it is one regeneration, and
the whole class closes at once. Check for the sources first.

### Variant B — `not yet implemented` as a first-class state

101 specs now fail with `@compileError("not yet implemented")`. That is the
compiler doing the right thing, but it makes those specs indistinguishable from
broken ones in every count. A distinct `UNIMPLEMENTED` verdict in the harness
would separate "this spec is wrong" from "this spec is unwritten" — two facts the
project has been reporting as one number for twenty-five waves.

### Variant C — Flash the board

Unchanged, and now the only item in the project that is neither a decision nor a
measurement: bitstream at 150.63 MHz, preflight clean, three theorems standing.

---

## Recommendation

**Variant A**, with its falsification check first — if the `.tri` sources exist,
571 decisions collapse into one regeneration.

---

*φ² + φ⁻² = 3 | TRINITY*
