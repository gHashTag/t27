# Iteration 8 — the "clause not lowerable" given-family: six rewrites,
# two deliberate leaves

## The family, decomposed first (the state.md instruction)

At plan time the census held four not-lowerable rows: `given` 19 events /
8 specs (the largest), `then` 15/9, `assert` 10/4, `and` 7/4. This
iteration took ONLY the given row and split it by root construct before
touching anything:

| root construct | specs | tokens | verdict |
|---|---|---|---|
| python heredoc `as []T` casts | ternary/packed_trit | 74 | rewritten natively |
| Zig spread struct literal + slice-assign + `then assert` | nn/attention | 417 | rewritten natively |
| Zig spread struct literal (×5) | nn/hslm | 153 | rewritten natively |
| `std.*` constructor / `&[_]T{}` literal in given | tri/encoding/mime | 44 | rewritten natively |
| same + colon field syntax | tri/encoding/html | 37 | rewritten natively |
| same + brace-empty children | tri/encoding/xml | 43 | rewritten natively |
| generics `Reader(R,T)` + string fn-types + inline closures | tri/io/reader | ~120 | **left — sketch module** |
| `new f32[param_count]` runtime alloc + undeclared locals | ml/optimizer/sgd | ~75 | **left — sketch module** |

644 of the family's 986 tokens were one construct: **a struct value in a
clause, spelled in Zig instead of t27**. The language already has the
native spelling — the spec's own functions use it (`LayerWeights{ .w_q =
[Trit.zero; EMBED_DIM * EMBED_DIM], ... }` was already inside hslm's
bodies; iteration 7's scratch proof of field-assign/indexed-assign/full
literal forms stands). The fixes are spec-side rewrites, iteration-3
class; no compiler change, no FROZEN_HASH move.

## The rewrites

- **attention.t27** (417→0): one `AttentionBuffers{...}` full literal
  using the spec's own line-127 idiom (`.field = [0.0; LEN]` arrays);
  slice-assign `buffers.scores[0..4] = scores` → four element assigns in
  the when-clause (`buffers.scores[0] = 1.0 and ...` — the form iteration
  7's scratch verified); multi-line conjunctions flattened; `then assert
  X` → `then X` (the parser expects the expression directly). Emits 48
  real `__t27_assert_fail` checks now.
- **hslm.t27** (153→0): 1× `LayerBuffers{...}` + 4× `LayerWeights{...}`
  full literals — every field natively typed and zero-filled.
- **packed_trit.t27** (74→0): python-heredoc `as`-casts → native typed
  literals (`[] as []i8` → `[]i8{}`, `[0x00] as []u8` → `[]u8{0x00}`).
- **mime.t27** (44→0): TWO fixes of different class — (1) the `to` field's
  type was a needlessly-quoted pseudo-type `"[]const []const u8"`;
  unquoted to `[][]const u8` (iteration-3 class: quotes are the
  pseudo-type escape hatch; the native type is real now and the declared
  API is unchanged). (2) the test literal's `&[_][]const u8{...}`
  address-of-anonymous-array → native `["recipient@example.com"]`. The
  recipient data SURVIVES (verified in emitted Zig).
- **html.t27** (37→0): `HtmlNode{tag: "div", attributes:
  std.StringHashMap([]const u8).init(), ...}` — colon field syntax →
  `.tag = ...`; the map value → `undefined`, which is the CORPUS'S OWN
  idiom for the unconstructible map field (mime.t27 parse body: `.to =
  undefined`). No native map type exists (json/msgpack model maps as
  structs/ unions; msgpack carries the same quoted StringHashMap field
  and stays green — the quoted field is inert, only the VALUE
  construction fell). query_selector is a void stub: the map value was
  never observable, so `undefined` loses nothing.
- **xml.t27** (43→0): same as html (`std.StringHashMap(...){}` →
  `undefined`; `.children={}` → `[]`).

Probe first (/tmp/t27_hyg/enc_scratch.t27): both the `undefined`-field
form and the native `[][]const u8` + `["s"]` literal form parse with zero
discard and typecheck clean — the rewrites were applied only after the
probe proved the target shapes.

## The deliberate leaves (recorded, not "fixed")

- **reader.t27** — the module is a generic-sketch: `Reader(R, T)` type
  parameters, string-literal function types (`"fn(R) -> T"`), inline
  closures capturing environments. Rewriting its tests would mean
  inventing a module the spec never was. Its fallback is the module
  being a sketch, not a dialect slip in an otherwise-native module.
- **sgd.t27** — `new f32[param_count]` heap allocations sized by runtime
  parameters, undeclared locals, `params 0 != 1.0` postfix indexing.
  Same verdict: whole-module pseudo-code.

These two hold the remaining 5 given-row events (2 specs). The row's
other three specs are gone.

## Measured

- given not-lowerable row: **19 events / 8 specs → 5 / 2** (the two
  leaves).
- total fallback events: 108 → 92.
- discard sum: 7,789 → **7,021 tokens, 99 → 93 specs** (cumulative
  **−74.5%** from 27,562).
- emission: attention alone now emits **48 runtime checks**; mime's
  format test emits its assertion with the recipient data intact;
  packed_trit's three cast-tests emit real index assertions.
- all six: `parse-complete --show` → **nothing discarded**; typecheck 0
  errors each (warnings pre-existing).

## Gate

First run: **0 unexpected failures, 6 unexpected passes** (all six specs
cleared `parse-no-discard` and are FULLY green — corpus 142 → 136).
Unlike iteration 7, no phase-masked defect waited behind the cleared
phase — per-spec typecheck had already been verified before the gate,
and the gate confirmed it. DISCARD WORSENED ×6 = the phase-move mechanic
(no reading taken at a phase the spec no longer fails); re-blessed
136/136. Confirm gate: **RATCHET CLEAN**, rc=0.

## Self-critique

- The family was NOT one defect. "38 events" was a plan-time number that
  mixed five root constructs; had I patched the walker for the census
  row's most common shape I would have extended the parser to accept Zig
  struct-literal syntax — the exact opposite of the right move (the
  language already owns the spelling; the specs were wrong).
- `undefined` as a field value is a compromise: it keeps the map field
  unconstructible rather than inventing a native map type. If t27 ever
  grows a map type, html/xml's tests should construct a real one — the
  rewrite comments say so.
- reader/sgd stay red at parse-no-discard with ~195 tokens combined.
  That is honest: their fallback measures the corpus containing sketch
  modules, which is a SPECIFICATION problem (write them natively or
  delete them), not a parser problem. Next loop should decide their
  fate explicitly rather than leave them as permanent residents.
- The mime type-unquote changes a type declaration, not just a test —
  first time this loop touched a declared field type. Justified because
  the quoted form never checked anything (inert string), but it widens
  the blast radius of "spec rewrites"; the gate must show no other spec
  importing TriMime (none does — `use` scan clean).
