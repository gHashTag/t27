# Evidence: PR #2168 (ADR-008) measured against master

Produced 2026-08-15. Binaries are gitignored; their seals are beside this file.

| | commit | SHA-256 | seal |
|---|---|---|---|
| base | `b92872507f6c7619acce43e5ae262b1dc9c4cbf2` | `836e8bc4d9a8bafa...` | `seal_master-baseline.json` |
| candidate | `a33413de5c0f3bf859c715238b847c9a88fb21b9` | `ef99cc164c0c9d63...` | `seal_pr2168-candidate.json` |

Reproduce either side:

    scripts/ci/rebuild_evidence.sh <commit> <label>
    python3 scripts/ci/artifact_seal.py verify --seal docs/evidence/seal_<label>.json --rebuild

## Full uniform differential over specs/ (one 15 s timeout, both binaries)

```
corpus: 634 specs under specs (scratch excluded)
base:      docs/evidence/bin/t27c.master-baseline
candidate: docs/evidence/bin/t27c.pr2168-candidate

    348  unchanged
      0  field-loss
      1  strict-improvement
      0  malformed-input-tradeoff
      0  unknown
    285  not-evaluated

  not-evaluated by reason code (each is a different fact):
      285  both-error

MEASURED COVERAGE: 349/634 = 55.0% of the corpus
  285 file(s) yielded no verdict and are NOT counted as agreement.
  Any sentence of the form 'no regressions' is admissible only with
  this coverage figure attached, and only when field-loss = 0 and
  unknown = 0. Coverage below 100% bounds what the run can claim.

strict-improvement (1):
  specs/tri/collections/array.t27
      base error, candidate parsed

CLEAN on the measured 349/634 (55.0%): no field-loss and no unknown.
Scope of that statement, stated so it travels with the number:
  * it covers the 349 file(s) on which both binaries gave a
    verdict, and says nothing about the rest;
  * it says nothing about categories the tool does not check --
    generated code, type inference, diagnostics, or timing.
```

Coverage is 55.0 % and is reported next to the regression count, never folded
into it. `specs/scratch` is not-evaluated: at the same uniform timeout the run
exceeded the tick budget and was stopped after 594 files. A prefix of an
alphabetical walk is not a sample, so the partial output is not a result.

## Targeted measurement: the 28 files that use the approved syntax

ADR-008 acceptance is necessary and not sufficient.

    27  error -> error
     1  error -> ok      specs/tri/collections/array.t27

The remaining 27 fail later, on type application in type position
(`fn empty() -> List(void)`, `fn map(io: IO(T))`), filed as #2174. First
candidate error over those 27:

    12  Expected LBrace, got RParen (')')
     8  Expected LBrace, got LParen ('(')
     7  Expected RBrace, got Eof ('')

Files still failing after the change:

    specs/tri/collections/btree.t27
    specs/tri/collections/either.t27
    specs/tri/collections/list.t27
    specs/tri/collections/lru.t27
    specs/tri/collections/map.t27
    specs/tri/collections/maybe.t27
    specs/tri/collections/option.t27
    specs/tri/collections/queue.t27
    specs/tri/collections/result.t27
    specs/tri/collections/ring_buffer.t27
    specs/tri/collections/set.t27
    specs/tri/collections/skip_list.t27
    specs/tri/collections/stack.t27
    specs/tri/collections/state.t27
    specs/tri/collections/tuple.t27
    specs/tri/collections/variant.t27
    specs/tri/graph/graph.t27
    specs/tri/io/io.t27
    specs/tri/io/reader.t27
    specs/tri/io/writer.t27
    specs/tri/io/zip.t27
    specs/tri/net/async.t27
    specs/tri/net/async_stream.t27
    specs/tri/net/channel.t27
    specs/tri/pipeline/builder.t27
    specs/tri/trees/tree.t27
    specs/tri/trees/trie.t27

## Fixture evidential force

Every fixture was also run against the base binary. pos_01..pos_06: error on
base, ok on candidate -- the change is load-bearing. neg_01..neg_05: error on
both -- guard only, establishes no fix.

neg_06_value_rhs and neg_07_no_rhs exit 0 on base. Base does not accept them: it
emits a ConstDecl with empty children plus a stray sibling statement from one
source line -- a hollow declaration and a loose statement. `base=ok` was a
metric of the exit code, not of acceptance. Whether narrowing these two forms is
wanted is an owner decision; the ADR settled the struct RHS and is silent on a
value RHS and on no RHS at all.
