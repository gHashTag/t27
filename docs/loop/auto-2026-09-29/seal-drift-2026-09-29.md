# seal-drift-2026-09-29.md — the 681 seal reds decomposed (plan item 5)

Measured on `loop/auto-2026-09-29` (master base 2925def9b + the loop's parser
fixes), suite log `/tmp/loop_suite_i5.log`, seal files under
`.trinity/seals/` (1398 present). Nothing was resealed to produce this.

## The decomposition (681 FAIL lines → 358 mismatch specs + 313 no-file specs)

| class | specs | signature | reading |
|---|---|---|---|
| no-seal-file | 313 | `No saved seal found at .trinity/seals/<name>.json` | seal never saved (or orphaned, below) — NOT drift |
| SPEC+ZIG+RUST+VERILOG+C | 249 | source hash AND all four backends | two honest waves: a repo-wide source normalization (e.g. comment-ruler ASCII, the one #5079 names) PLUS the Sep-20 quotes fix |
| ZIG+RUST+VERILOG+C, spec clean | 31 | all backends, source unchanged | pure codegen change with source untouched — the quotes-fix class exactly (`7bf09b169`, 2026-09-20: "a string literal reached every backend without its quotes") |
| ZIG only | 27 | zig backend only | a later zig-only emission change (candidate: `614e709a9` "[0]str was reaching Zig verbatim") |
| C+VERILOG+ZIG | 23 | three backends, no rust, spec clean | a shared-emitter change |
| SPEC only | 12 | source changed, gen identical | comment-only edits to sources post-seal |
| tails | ~16 | SPEC+one-or-two backends | mixed |

## New defect found while measuring: seal-name COLLISION

The seal namer flattens `specs/` + `/`→`_`. `specs/a/b_c.t27` and
`specs/a_b/c.t27` both flatten to `a_b_c_spec.json` — one of them can never
keep a seal. Exactly one colliding pair exists today (verified by flattening
every failing spec path). Any fix that renames seal files must handle it;
resealing cannot.

## What this means for the "smallest honest reseal batch"

- The **31 + 23 + 27 ≈ 81** pure-codegen drift specs (source clean) are the
  candidates whose reseal would be a pure "the emitter honestly changed"
  event — IF each underlying commit is confirmed intended (quotes fix:
  confirmed by its own commit message; the other two classes need their
  commits named the same way before resealing).
- The **249** SPEC+all-backends specs need the source-normalization commit
  identified first; resealing before that names itself would hide which
  wave moved the hash.
- The **313** missing-file specs are NOT reseal questions at all: either the
  seal was never saved, the naming scheme moved, or the collision orphaned
  it. A `--save` pass (or naming fix) is a separate, deliberate act.
- Standing rule respected: tonight's loop resealed NOTHING except the
  compiler's own FROZEN_HASH (`tri reseal write`, build.rs contract), and
  the memory's "no mass reseal until branch fate decided" stands.

## Self-critique

- The class→commit attribution for the 31/27/23 groups is by SIGNATURE
  (which backends moved), cross-checked against commit titles — not by
  bisecting builds. A reviewer wanting certainty builds t27c at
  `7bf09b169^` and diffs one spec's gen output; this document does not
  claim that was done.
- `spec_hash` semantics CHECKED, not inferred: `bootstrap/src/main.rs:2938` —
  `sha256(source.as_bytes())`, raw source bytes. The 249's source text
  genuinely changed since sealing; the "normalization wave" reading stands.
