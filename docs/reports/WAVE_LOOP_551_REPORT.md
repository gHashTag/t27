# Wave Loop 551 Report — the seal mechanism certifies files the compiler rejects

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_550_REPORT.md`](WAVE_LOOP_550_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W551 set out to diagnose the two largest undiagnosed parse-failure classes
(48 + 43 specs). It found their causes, and on the way found something larger:
**L2's seal mechanism can be satisfied by a file the compiler cannot parse**,
and the documented command that does it is silently destructive.

The wave also had to undo damage it caused itself. That is reported in §3
rather than buried.

---

## 1. The seal finding

`t27c seal --save` does **not** check that a spec parses. On an unparseable
file it writes `gen_hash_* = "none"` for every backend and exits 0.
`t27c seal --verify` then reports:

```
gen_hash_rust: MATCH
all hashes MATCH
```

Because `none` matches `none`. **A green seal that certifies nothing.**

Demonstrated on `specs/api/c_api_contract.t27` — which is a **Markdown
document with a `.t27` extension**, opening `# C API CONTRACT` / `## Specification`.

### Why this is worse than vacuous

An all-`none` seal does not merely fail to add signal; it **removes** signal.
Before the reseal, that spec's seal failed verification with:

```
gen_hash_zig: MISMATCH (saved=sha256:..., current=none)
```

which is precisely the alarm that a spec has stopped generating — the alarm
that fired 1,035 times in the W549 full-suite run. After a reseal, the alarm
stops. The gate goes from red to green with no change to the underlying
problem.

---

## 2. Corpus composition (measured)

| Finding | Count |
|---|---|
| Unparseable specs that carry a seal file | **221 of 326** |
| `.t27` files that are actually **Markdown** documents | **15** — all fail; all sealed; **104 external references** |
| Files in dialects the compiler does not implement | **11** — `spec X { struct Y { f: string } }` (8), `algorithm X { }` (3); all fail |
| Files declaring `module ;` (**empty module name**) | **7** — and **all 7 parse fine** |

The 15 Markdown files are referenced 104 times across the repo and all 15 have
seals, so renaming them to `.md` is a structural decision, not a cleanup — it
is left for a human. They are:

`specs/{benchmarks/gf16_bfloat16_nmse, benchmarks/bench_main, benchmarks/bench_nn,
benchmarks/ternary_vs_binary, api/sdk_contract, api/c_api_contract, api/tri_net_api,
brain/neural_gamma, brain/brain, physics/gamma-conflict, physics/e8_lqg_bridge,
physics/hslm_benchmark, physics/lqg_cs_bridge, physics/quantum,
conformance/e2e_scenarios}.t27`

That the parser accepts `module ;` with no name is a separate laxity worth its
own look.

---

## 3. Damage this wave caused, and repaired

Auditing the seal mechanism required running `seal --save` on an unparseable
file. Doing so **destroyed that file's real seal** — four valid gen hashes from
2026-08-06 replaced with `none`. Restored.

That prompted an audit of this wave-chain's own reseals, which found **30 seals
degraded the same way** in W549 and W550. Resealing repaired specs was correct
for those that now generate — they received real hashes — but for the 30 that
still do not parse, `seal --save` silently overwrote four real gen hashes with
`none`, converting 30 mismatch-flagging seals into vacuous passing ones.

All 30 restored to their pre-wave state (`079ed21ab`); re-audited, 0 remain
degraded (commit `123c7ccc5`). They will keep failing verification until the
specs actually generate again, which is the correct signal.

**Recorded as skill rule 13:** never `seal --save` without gating on
`t27c parse` first. The command does not check, and its failure mode is silent,
destructive, and turns a red gate green.

---

## 4. The two classes W551 set out to diagnose

**Class A — 48 × `unexpected token after expression statement: Ident`.**
Heterogeneous: 15 are the Markdown files above, 13 use the `spec`/`algorithm`
dialects, and 20 are genuine t27 with unrelated defects (`match` statements,
`assert x == 1;` with a trailing semicolon, `import` where `use` is expected).
**Not a single mechanical class** — unlike W550's, this one does not admit a
scripted repair.

**Class B — 43 × `Expected LParen, got Ident`.** Not yet diagnosed; carried to
W552.

### A method note

The first dialect census reported "325 files in neither dialect" and looked
like a major structural finding. It was a regex bug: 316 of those simply had
namespaced module names (`module depin.prove;`, `module github::issues {`) that
the pattern did not allow. Checking before publishing turned a fake headline
into an accurate 26-file one. This is the third consecutive wave where the
first hypothesis was wrong and cheap verification caught it.

---

## 5. Three cooperation variants for W552

### Variant A (recommended) — Make the seal gate mean something

**Hypothesis.** L2 is a constitutional law whose mechanism currently certifies
files the compiler rejects. Fixing it is small, needs no `compiler.rs` change
(so it is not blocked by the LANG-EN gate), and it protects every future wave
from the mistake this one made.

**Deliverables.**
1. `t27c seal --save` refuses to seal a spec that does not parse, with a clear
   message, unless `--force` is passed.
2. `t27c seal --verify` treats an all-`none` seal as **FAIL**, not MATCH.
3. A repo-wide report of seals whose spec no longer parses (221 today), wired
   into the suite as a reporting gate.
4. Backfill: decide per-spec whether the 33 existing all-`none` seals should be
   deleted or flagged.

**Validation.** `seal --save` on `specs/api/c_api_contract.t27` fails.
`seal --verify` on an all-`none` seal fails. No previously-valid seal changes.

**What would falsify it.** If some legitimate spec class has no backends and is
*expected* to seal all-`none`, then all-`none` is not a defect and the rule
needs a carve-out. Check before enforcing.

### Variant B — Clear the LANG-EN gate, then the syntax gaps

Unchanged from W550 and still **blocked on a human decision**: six committed
docs violate L3 and are not allowlisted, so `build.rs` panics on any
`compiler.rs` edit. Once cleared: apply the float-cast patch (~16 specs), add a
block-expression production (~40), add struct-literals-in-expression (~28).

### Variant C — Resolve the `given`/`when`/`then` question

Unchanged from W550. 327 specs use a test form with no parser production that
is nonetheless specified in `SOUL.md`, the language RFC and the TDD contract.
Nobody has established whether those blocks are recognised as tests, skipped,
or mis-parsed — and every "N tests in spec X" claim depends on the answer.

---

## Recommendation

**Variant A.** It is unblocked, small, and closes a hole in a constitutional
law that this wave demonstrated is not theoretical — it fired 30 times against
this very wave-chain. B remains the larger prize and should start the moment
the LANG-EN decision lands.

---

*φ² + φ⁻² = 3 | TRINITY*
