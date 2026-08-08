# Wave Loop 556 Report — IGLA CODER audited, and the best specs in the repo

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_555_REPORT.md`](WAVE_LOOP_555_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Seven waves have studied IGLA RACE closely and IGLA CODER barely at all. W556
audited CODER's actual training data — the one substantial piece of this
project not yet examined, and unblocked by the LANG-EN gate.

The result is the most mixed of the chain, and for once part of it is good.

---

## 1. The dataset

`dataset/igla-coder/v0.1/` — described in its README as *"parallel `(spec, gen)`
pairs where `spec` is a `.tri` T27 specification and `gen` is the corresponding
generated code"*.

| | |
|---|---|
| Pairs | 8 |
| **Pairs with a generated-code half** | **0 of 8** |
| `codegen_status` values | `spec_only`, `codegen_pending_train_box`, `codegen_stub_pending_wave2`, `codegen_deferred_wave3` |
| `held_out_eval_defined` | **false** |
| `decontam_bidirectional` | **false** |
| Source specs still present in the repo | **6 of 8** |

**The training corpus for a spec→code model has no code side.** Every `gen_path_in_t27`
is `null`.

**To the project's credit, the manifest says so plainly.** It is not a hidden
claim: `"note": "v0.1 seed: 8 spec-only pairs. Codegen pending TRAIN-BOX"`, and
`DECONTAM.md` states decontamination is *"trivially clean because held-out-eval
is empty at v0.1; v0.2 must populate held-out-eval FIRST."* That is honest
labelling of an incomplete artifact — the opposite of the pattern the previous
six waves kept finding.

Two source specs have since vanished from the repo:
`specs/organism/experience.tri` and `specs/organism/ring_runtime.tri`.

---

## 2. The good news, and it is real

The manifest claims L4 compliance: *">=3 invariants + >=8 tests + >=2 bench per
spec (visual audit)"*. **Measured, it holds exactly** — all 8 pairs have 8
tests, 3–4 invariants, 2 benches.

More importantly, given W555:

| | dataset specs | main IGLA `.t27` corpus |
|---|---|---|
| test form | **8/8 brace-form** | 70.6 % BDD-form (inert) |
| invariant form | **all brace-form** | 86.2 % keyword-form (skipped) |
| test bodies | real multi-assertion | 2,165 are `assert true` |

A representative body:

```t27
test dna_write_roundtrip {
  let payload = bytes_of_ascii("hello")
  let r = dna_record_new(DnaRecordKind.Skill, "ring-105-002", payload)
  assert dna_write(r, payload) == true
  let back = dna_read(r.payload_sha256)
  assert back.magic == r.magic
  assert back.kind  == r.kind
}
```

**These are the best-written specs I have examined in this repository.** They
use the forms that actually execute, and they assert real things. Whoever wrote
the v0.1 seed did the work properly.

---

## 3. The catch — stated correctly

The dataset specs are `.tri`, which is a different language from `.t27`:

```
spec trinity_dna
-- Ring 105-002: Trinity DNA schema          <- `--` comments, not `//`
pub const SKILL_ID_MAX_LEN u32 = 64          <- name-then-type, no colon
```

All 17 `.tri` files under `specs/` fail `t27c parse` at line 1, and there is no
`.tri` handling in `bootstrap/src/` or `cli/tri/src/`.

> **Correction made during this wave.** The first draft of this section
> concluded that `.tri` is "a documented format with no implementation" — the
> same defect class as W555's `given`/`when`/`then`. **That was wrong, and
> checking it took one query.** `gHashTag/trinity` contains **744 `.tri` files**
> and a real parser for them — `src/tri/parser.zig`, plus
> `src/forge/tri_parser.zig` — and its README instructs contributors to *"edit
> or create a specification in `specs/tri/*.tri`"*. `.tri` is **trinity's**
> spec language, implemented in Zig.
>
> t27's `.tri` files say so themselves. `specs/organism/mozg.tri` opens:
> `-- Source repo: gHashTag/trinity (Zig)` / `-- Target path in t27 repo:
> specs/organism/mozg.tri`. **They are imports.**

So the accurate finding is narrower: t27 hosts 17 imported specs in a sibling
project's language, its own Rust toolchain cannot read them, and the IGLA CODER
dataset is built from those imports. The well-written tests in §2 do not
execute *in t27* — whether they execute under trinity's Zig parser is a
question for that repository, not this one.

---

## 4. A third blind spot in my own tooling

`validate-vacuity --specs-dir dataset/igla-coder` reported **"TOTAL over 0
specs"**. The scanner only looked at `.t27`. Every parse census in this chain
used `find specs -name "*.t27"`, so **17 files were never counted** — the
corpus is 1,080 specs, not the 1,063 I have been quoting since W549.

That is the third blind spot found in my own instrumentation:

| Wave | Blind spot | Missed |
|---|---|---|
| W554 | exit status treated as success | hollow synthesis, 0/17 → reported 7/17 |
| W555 | brace-form test blocks only | 7,623 BDD tests |
| **W556** | **`.t27` extension only** | **17 `.tri` specs** |

**Fixed:** the tool now reports `.tri` files as `NOT ANALYSED` with the reason,
so the gap appears in the output instead of being hidden by it. Counting them
with the `.t27` scanner would have produced nonsense; the honest move is to
name what is outside the count.

---

## 5. What IGLA CODER actually is, stated plainly

- A **v0.1 seed dataset of 8 well-written specs**, honestly labelled as
  incomplete.
- With **no generated-code half**, so nothing to train a spec→code model on.
- Written in **`.tri`** — trinity's language, parsed there by
  `src/tri/parser.zig`, but **unreadable by t27's own toolchain**.
- With **no held-out eval**, so no decontamination is possible and no benchmark
  score can be computed even in principle.
- Two of its eight source specs **no longer exist** in the repo.

It is not vapour — the engineering in the specs is real and better than the
surrounding corpus. It is a carefully-built foundation for a pipeline that was
never connected.

---

## 6. Three cooperation variants for W557

### Variant A (recommended) — Migrate the imported `.tri` specs to `.t27`

**Premise, after correction.** `.tri` is trinity's language and is parsed there
(`src/tri/parser.zig`). t27 hosts 17 imported `.tri` files it cannot read, and
builds the IGLA CODER dataset from 8 of them. Nothing is orphaned — but inside
t27 these specs are inert, and they silently sat outside every census in this
chain.

**Deliverables.**
1. Convert the 8 dataset specs (and ideally the other 9) from `.tri` to `.t27`.
   The constructs map closely: `spec X` → `module X;`, `--` → `//`,
   `pub const NAME u32 = 64` → `pub const NAME: u32 = 64`. Their tests are
   already brace-form with real assertions, so they would **execute
   immediately** — the highest-value outcome available without the LANG-EN gate.
2. Then `t27c gen` produces the Zig/Rust/C halves mechanically, giving
   `MANIFEST.json` real `gen_sha256` values instead of `null` (this subsumes
   Variant C).
3. Amend `SOUL.md:25` to say `.tri` is trinity's format, parsed there — so a
   reader does not expect t27 to handle it.

**Unblocked:** entirely. Source migration plus documentation; no `compiler.rs`
change.

**What would falsify it.** If the `.tri` → `.t27` mapping turns out not to be
close — e.g. `.tri` has constructs t27 has no equivalent for — then migration
is a language-design task, not a transcription, and the estimate is wrong.
Convert one spec first and measure.

### Variant B — Clear the LANG-EN gate

Unchanged since W550, now gating **three** compiler tracks: the BDD parser fix
(7,623 tests + 5,163 invariants), the datapath investigation, and the ~84-spec
syntax gaps. Six documents violate L3 and are not allowlisted.

### Variant C — Recover the two missing source specs

`specs/organism/experience.tri` and `specs/organism/ring_runtime.tri` are
referenced by the manifest but absent from the repo. They exist upstream in
`gHashTag/trinity` (744 `.tri` files). Re-import them, or mark those two pairs
as withdrawn in `MANIFEST.json` so the dataset stops claiming 8 pairs when 6 are
backed by present sources.

Small, unblocked, and it removes a stale claim.

---

## Recommendation

**Variant A.** It is unblocked, it resolves a documented-but-unimplemented
format that has quietly corrupted every spec count in this chain including
mine, and its step 3 is the only path that makes the best specs in the
repository actually run.

---

*φ² + φ⁻² = 3 | TRINITY*
