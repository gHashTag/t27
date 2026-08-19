# Wave Loop 550 Report — repairing the corpus, and what the error messages hid

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_549_RESEARCH.md`](WAVE_LOOP_549_RESEARCH.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W549 ended with a census: **363 of 1063 specs (34.1 %) do not parse**. W550
took the largest single class — the 38 specs failing with
`Expected RBrace, got Eof` — and repaired it. That class was **not** what W549
labelled it, and finding out why is most of this report.

**Result: 700 → 737 of 1063 specs parse (65.9 % → 69.3 %), 0 regressions.**

This wave deliberately took the track that needs **no `compiler.rs` edit**,
because the LANG-EN build blocker (W549 §4.4) still makes any compiler change
unbuildable and its resolution requires Architect approval.

---

## 1. Two wrong hypotheses, in order

### 1.1 "Unterminated blocks, same class as the W339 brace bug"

That is what W549 published. It was wrong: **brace depth in all 38 files is
zero**, verified with a lexer-faithful counter that ignores comments and string
literals. Nothing is unterminated block-wise.

### 1.2 "An unimplemented `given`/`when`/`then` BDD dialect"

The second hypothesis looked much stronger. 35 of the 38 use a braceless
Gherkin-style test form:

```t27
test find_last_basic_case
    given input = default_input()
    when result = find_last(input)
    then result != undefined
```

`given`/`when`/`then` appear in **no** parser production, yet the form is
documented in `SOUL.md`, `docs/rfc/tri-language-core.md`,
`docs/nona-03-manifest/TDD-CONTRACT.md` and two more — and **327 specs use
it**. A canonical, documented, unimplemented dialect would have been a major
finding.

It is not the cause. **158 specs using the same form parse cleanly**, and the
discriminating test fails outright: a `when` clause appears in 76 % of the
failures and 77 % of the passes.

*(The wider question — that 327 specs use a form the parser has no production
for, and that it survives only because the parser tolerates it incidentally —
is real, unresolved, and carried to W551 Variant C.)*

### 1.3 What it actually was

A **corrupted type annotation carrying a stray double quote**. The quote opens
a string literal that swallows the remainder of the file; the parser then
reports where it gave up (`}` expected at EOF) rather than where the file went
wrong. All 38 had exactly one unterminated string.

Two shapes, both an extra leading `[`, a capitalised primitive, and a trailing
`"`:

```
bits     : [[]Usize",           ->  bits     : []usize,
log_file : [?[]Const u8",       ->  log_file : ?[]const u8,
opad     : [[64]U8",            ->  opad     : [64]u8,
children : [[256]?*ACTrieNode", ->  children : [256]?*ACTrieNode,
```

The canonical form was confirmed against specs that parse (`[]const u8`,
`[]ConfigEntry`) rather than assumed.

**Method note.** Three hypotheses, two refuted by measurement before any code
changed. The refutations were cheap — a brace counter, a `when`-clause
frequency comparison — and each would have sent the wave in a useless
direction. The error message was the least reliable evidence available.

---

## 2. Delivered

| | |
|---|---|
| Specs edited | 63 |
| Now parsing | 52 |
| Previously-failing specs repaired | **37** |
| Repo-wide parse rate | **65.9 % → 69.3 %** (700 → 737 of 1063) |
| Regressions | **0** |
| Seals regenerated | 63 |

Two mistakes were caught by verification and are recorded because they nearly
shipped:

1. The first repo-wide pass **regressed** `specs/tri/encoding/{html,xml}.t27`,
   which had been passing. They carry a fourth corruption shape —
   `[std.StringHashMap([]Const u8)"` — that the pattern does not match; fixing
   their other three lines left one stray quote and flipped quote parity from
   even to odd. Both files were reverted and left untouched rather than
   half-repaired.
2. The generalised pattern glued `[]` to the type token, so `Const` never
   lowercased and 15 specs received `[]Const u8` instead of `[]const u8`.
   Fixed with a type-position-anchored pass (30 names, 19 specs).

Every previously-passing file touched was re-parsed afterwards. That check is
what caught defect 1, and it is now a standing rule in the wave-loop skill.

---

## 3. Still blocked, unchanged from W549

- **LANG-EN build gate.** Six committed docs violate L3 and are not
  allowlisted, so `bootstrap/build.rs` panics whenever it re-runs — i.e. on any
  `compiler.rs` edit. `docs/.legacy-non-english-docs` is Architect-approval-only.
  This blocks the float-cast patch and block-expression work.
- **G1 bitstream.** `bbaexport` is OOM-killed: Docker has 3.83 GiB of an 8 GB
  host, against ~3.5 GiB peak for the *smaller* 100T measured natively.
- **G2–G4 hardware.** No board attached.

---

## 4. Three cooperation variants for W551

### Variant A (recommended) — Finish the mechanical corpus repair

**Hypothesis.** W550 repaired one corruption class and moved the parse rate
3.4 points. The remaining 326 failures still contain mechanical classes that
need no compiler change: the fourth corruption shape
(`[std.StringHashMap(...)"`), and whatever drives the 48 `unexpected token
after expression statement: Ident` and 43 `Expected LParen, got Ident` groups —
neither of which has been diagnosed at all.

**Why first.** It is unblocked. Every other track waits on an Architect
decision, more RAM, or hardware.

**Deliverables.**
1. Diagnose the 48 and 43 groups the way W550 diagnosed the 38 — brace/quote
   parity, a discriminating comparison against passing specs, and a confirmed
   canonical form *before* any rewrite.
2. Repair the fourth corruption shape and re-check `html.t27`/`xml.t27`.
3. Re-run the census and report the new rate.
4. Wire `t27c validate-vacuity` and a parse-rate check into CI pinned at the
   achieved number, so the rate can only go up.

**Validation.** Parse rate strictly above 69.3 %; zero regressions among
previously-passing specs; seals regenerated.

**What would falsify it.** If the 48/43 groups turn out to need
block-expressions or struct-literal support, they are not mechanical and the
variant collapses into B — which is a useful finding, not a failure.

### Variant B — Clear the LANG-EN gate, then close the syntax gaps

**Hypothesis.** Three syntax productions (block-expression ~40 specs,
struct-literal-in-expression ~28, float casts ~16) plausibly address ~84 of the
remaining 326. All three need `compiler.rs`, which cannot currently be built.

**Deliverables.**
1. **Human decision first:** translate the six LANG-EN documents, or approve
   allowlisting them. Nothing else in this variant can start.
2. Apply [`docs/patches/W550-f32-cast-whitelist.md`](../patches/W550-f32-cast-whitelist.md); reseal `FROZEN_HASH`.
3. Add a block-expression production; re-measure.
4. Add struct literals in expression position; re-measure.

**Validation.** `cargo build --release -p t27c` green *after* a `compiler.rs`
edit — the condition that currently fails. Parse rate up by roughly the
predicted counts, each measured separately so the predictions are scored.

**What would falsify it.** If block-expressions conflict with the statement-`if`
grammar in a way needing redesign, the "three productions" estimate is wrong
and rewriting the specs becomes the cheaper route.

### Variant C — Resolve the `given`/`when`/`then` question

**Hypothesis.** 327 specs — nearly a third of the corpus — use a test syntax
that has no parser production but is specified in `SOUL.md` and the language
RFC. They parse only because the parser tolerates the shape incidentally. That
is an undefined-behaviour surface sitting under a third of the repo, and
nobody has established what the parser actually does with it.

**Why it matters beyond tidiness.** Every claim of the form "N tests in spec X"
depends on those blocks being understood as tests. If the parser is skipping or
mis-binding them, the test counts are wrong in the same way W549 showed the
vacuity counts were.

**Deliverables.**
1. Determine empirically what the parser does with `given`/`when`/`then` —
   are the blocks recognised as tests, silently skipped, or mis-parsed as
   expression statements?
2. Decide: implement the dialect as documented, or amend `SOUL.md` and the RFC
   to describe the language that exists.
3. Whichever is chosen, make the other side match.

**Validation.** A spec using the BDD form has its tests appear in
`t27c` test enumeration with the expected count — or the documentation no
longer promises they will.

**What would falsify it.** If the parser genuinely implements the form through
a production not found by keyword search, the premise is wrong and the finding
is that the search missed it.

---

## Recommendation

**Variant A.** It is the only track not gated on a human decision, more memory,
or hardware, and W550 demonstrated the method works and the returns are
measurable. B is the larger prize and should start the moment the LANG-EN
decision lands. C is the one most likely to invalidate other people's numbers,
which is an argument for doing it sooner rather than later.

---

*φ² + φ⁻² = 3 | TRINITY*
