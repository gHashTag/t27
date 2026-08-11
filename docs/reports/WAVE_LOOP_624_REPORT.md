# Wave Loop 624 — the measurement was the defect three times over

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_620_REPORT.md`](WAVE_LOOP_620_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
A  W623's own numbers re-run    -> both endpoints reproduce, the DELTA does not
B  the fix's class enumerated   -> 2 of 5 positions were implemented; 2 closed here
C  the measurement instrument   -> 14.0% of generated bodies are never type-checked
D  FROZEN_HASH                  -> operational line had lost its path field; restored

T19  a compile-error count is not an order on correctness
T20  a fix's scope is set by the population that exercised it, not by its class
T21  the corpus error count is reachability-conditioned, not a compiler property
```

This wave audited the previous wave's work rather than extending it. All three
findings came from **re-executing** published measurements, not from re-reading
them — and in each case the published prose asserted the property the re-run
refuted.

---

## 1. Variant A — T18 reproduces, and its table does not reconcile

W623 shipped T18 as a demonstration refuting P12 (*"not one remaining blocker is
a compiler defect"*). Independent re-measurement over all 34 specs under
`specs/igla` (`t27c gen` + `zig test --test-no-exec`, zig 0.16.0):

| | W622 (HEAD) | W623 | recorded in T18 |
|---|---:|---:|---|
| `expected type '<sized int>', found 'usize'` | 9 | **0** | 9 → 0 ✔ |
| total error lines | 1076 | **1069** | 1076 → 1069 ✔ |

**Both endpoints reproduce exactly.** The delta does not: nine errors removed,
total down seven. Diffing by error *class* rather than by total finds the missing
two:

| error class | W622 | W623 |
|---|---:|---:|
| `expected type 'u32', found 'usize'` | 9 | **0** |
| `incompatible types: 'struct { u32 }' and '[]u32'` | 2 | **4** |
| everything else | 1065 | 1065 |

The two new ones are at `coder_tokenizer.zig:470` and `:527` — **the same two
lines that previously carried a usize error**. Fixing the argument let the type
checker reach the enclosing expression `.{ kw_id } + <call returning []u32>`,
which was ill-typed all along and had never been analysed.

> **T19 — Diagnostics are not independent, so `|E|` is not monotone under defect
> repair.** A repair that strictly removes defects can raise the count; a fall in
> the count is compatible with new defects. Only a per-class, per-site diff
> settles a delta.

This is the mirror of the cascaded-spurious-error problem that motivated parser
error recovery: the field built machinery so one defect would not *inflate* the
count, and the same coupling *deflates* it.

---

## 2. Variant B — the fix implemented 2 of 5 positions, and the corpus could not say so

T18 names its defect class semantically — `.len` is `usize`, every t27 signature
carrying a length declares a *sized* integer — then implements the two syntactic
positions the nine measured sites happened to occupy. Its last line records the
reasoning as a virtue: *"the measurement, not the exemplar, set the fix's scope."*

A six-position probe (`probe_len.t27`, one function per position) enumerates the
class instead of sampling it:

| # | position | cast emitted by W623 | actually a Zig error? |
|---|---|:--:|---|
| 1 | `return s.len()` under `-> u32` | yes | yes |
| 2 | `f(s, s.len())` where `f` declares `u32` | yes | yes |
| 3 | `return base + s.len()` under `-> u32` | yes | yes |
| 4 | `let n : u32 = s.len();` | **no** | **yes** |
| 5 | `Box { n: s.len() }`, field `n : u32` | **no** | **yes** |
| 6 | `s.len() > cap`, `cap : u32` | no | **no** — Zig peer-resolves |

Two genuine gaps **and one non-gap**. Position 6 matters as much: a fix scoped by
"wherever a length meets a sized int" would have wrapped a comparison that was
already correct, narrowing it.

**Implemented this wave** — `bootstrap/src/compiler.rs`, `StmtLocal` and
`ExprStructLit`:

```zig
// before                          // after
const n: u32 = s.len;              const n: u32 = @as(u32, @intCast(s.len));
return Box{ .n = s.len };          return Box{ .n = @as(u32, @intCast(s.len)) };
```

`zig test --test-no-exec` on the probe: **rc = 0**, was 2 errors.

**And the corpus output is byte-identical.** `diff -rq` over all 34 generated
`.zig` files, W623 vs W624: no difference; error classes unchanged. That is not a
weak result — **it is the proof of T20.** The extension is verified entirely by
constructed witnesses because the corpus contains zero instances of either
position; a fix justified by corpus measurement could not have been written.

> **T20 — A fix derived from measurement over corpus `K` implements `Σ_K`, not
> the class `Σ`. These coincide iff `Σ_K = Σ`, which measurement over `K` cannot
> establish.** Closing the gap needs a constructed enumeration, which is a
> different activity from measuring.

T20 is **T16's sibling with the selector moved**: T16 is a *rule* validated on
the population authored from it; T20 is a *fix* scoped by the population that
exercised it. In both, a syntactic selector stands in for an epistemic one, and
in both the report — "0 mismatches", "9 of 9 sites fixed" — is indistinguishable
from the sound version.

---

## 3. Variant C — the instrument itself

Validating the probe exposed the larger finding. The first probe put all six
positions in one file with one test; it compiled clean. The second contained the
**same function bodies** plus tests calling them, and reported two errors.

| probe | bodies for positions 4–6 | tests referencing them | errors |
|---|---|---:|---:|
| `probe_len.t27` | identical | 0 | **0** |
| `probe_len2.t27` | identical | 3 | **2** |

Zig analyses a body only when referenced. Over the 34 generated files:

| | count |
|---|---:|
| distinct generated functions | 1286 |
| never referenced in their own compilation unit | **180 (14.0%)** |

**Roughly one generated function body in seven has never been type-checked.**

> **T21 — `N(f)` is a function of *(generated code, reference graph)*, not of the
> code alone.** Adding a test, changing no logic, can raise it. Every "total
> compile errors" figure in `IGLA-FORMAL-RESULTS.md` is a joint measurement of
> the backend and the corpus's own coverage. 1069 is a **lower bound**, not a
> count.

Deltas between figures measured against the same reference graph (P25, P30, T18,
T19) remain valid. What is not valid is reading any of them as *"the corpus
contains N errors."*

---

## 4. Variant D — the seal

`bootstrap/stage0/FROZEN_HASH` had been rewritten as a bare digest, dropping the
repo-relative path. `build.rs` tolerates it (`split_whitespace().next()`), so
nothing failed — but the canonical operational line is `<64-hex-sha256>
<repo-relative-path>`, as emitted by `t27c frozen-digest`, written by
`scripts/reseal-apply.sh`, and named in `build.rs`'s own panic text and
`FROZEN.md` §4. A silent format divergence that only a *future* consumer would
notice. Restored, and re-sealed against the W624 compiler.

---

## 5. What was NOT done

- **No web literature was consulted this loop.** `WebSearch` and `WebFetch` both
  failed with a provider error for the whole session. Every work named in §3 of
  `IGLA-FORMAL-RESULTS.md` is described from general knowledge under that
  document's existing standing rule (*"named because it is well known and its
  content is being described accurately"*), and **no citation was fabricated to
  fill the gap.** Verifying those attributions against sources is carried
  forward.
- **The 180 unreferenced bodies were not analysed.** T21 bounds the blind spot;
  it does not measure what is inside it.
- **No `.t27` spec text changed**, and no FPGA hardware step ran (the board is
  still absent — `DLC10 cable not found`, unchanged since W620).

---

## 6. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean (636 pre-existing warnings) |
| `cargo test --release -p t27c --bins len_` | **6 passed**, 0 failed |
| probe positions 4/5 | 2 errors → **rc = 0** |
| corpus regression (`diff -rq` 34 files) | **byte-identical** to W623 |
| corpus totals | 1069 / 0 usize, unchanged |
| `FROZEN_HASH` | re-sealed, canonical two-field form |

Three new unit tests pin the behaviour, including
`len_comparison_is_left_alone_because_zig_peer_resolves_it` — the non-gap, pinned
so a later wave does not "fix" it.

---

## 7. Three ways to continue (pick one for W625)

### Option 1 — **Close T21: measure inside the blind spot**

Force analysis of the 180 unreferenced bodies (generate a `comptime` reference
block, or `std.testing.refAllDecls`) and re-run the corpus. Produces the first
*unconditioned* error count this project has ever had, and either confirms 1069
as near-total or reveals a large hidden population.

- **Cost:** one backend change, one re-measurement. Low risk.
- **Pays off in:** every error-count figure in the document becomes attributable.
- **Risk:** the number may jump sharply, which invalidates the *narrative* of
  steady progress while making the *measurement* honest for the first time.
- **Falsifies:** T21's corollary that 1069 is a lower bound worth tightening.

### Option 2 — **Generalise T20: enumerate the remaining defect classes by probe**

The `usize` class is now closed by construction. Apply the same method — probe
first, corpus second — to the two largest remaining classes: `use of undeclared
identifier` (485 of 1069, 45%) and the `struct { … }` / slice mismatch family.
Build a positional probe for each before writing any fix.

- **Cost:** highest of the three; two probe suites plus fixes.
- **Pays off in:** the largest absolute error reduction available, and it is the
  only option that moves kernels toward compiling.
- **Risk:** 485 undeclared identifiers is likely the `use_resolve` gap (W587),
  i.e. one root cause, not a class needing enumeration — the probe may show the
  method does not apply and that is a useful negative result.
- **Falsifies:** T20's generality, if either class turns out to be positionally
  uniform.

### Option 3 — **Turn the five hygiene theorems into an executable gate**

T16, T17, T19, T20, T21 are currently prose. Make them checks: a `t27c
claim-audit` subcommand that (a) rejects a bare cross-document claim label
(**T15**), (b) flags any measurement whose stated population is selected by a
syntactic predicate (**T16/T20**), (c) requires an error-count claim to carry its
class table (**T19**) and its reachability qualifier (**T21**).

- **Cost:** medium; a new subcommand plus a doc-scanning pass.
- **Pays off in:** the failure modes stop recurring — this document has now
  recorded the *same* selection defect three times under different names.
- **Risk:** a gate over prose is heuristic and will produce false positives; it
  must be advisory, not blocking, or it becomes the next thing that reports
  success while discarding input (§4's own rule).
- **Falsifies:** the implicit claim that stating a methodological rule prevents
  its recurrence — which the T15 near-miss already made doubtful.

**Recommendation: Option 1.** It is the cheapest, it is the only one that makes
every *existing* number in the document mean something more precise, and T21 is
the finding with the widest blast radius — it qualifies figures back to P25.
Option 3 is the right follow-on once there is an unconditioned baseline to gate
against.

---

## Appendix — reproduction

```bash
cargo build --release -p t27c
cargo test --release -p t27c --bins len_
```

Corpus measurement, per spec: `t27c gen <spec> > out.zig` then
`zig test --test-no-exec out.zig`, aggregating stderr over all 34 specs under
`specs/igla` (one spec fails `gen` and is excluded from both endpoints).
Class table: `grep -oE 'error: .*' | sed 's/[0-9]\+/N/g' | sort | uniq -c`.
Reachability: distinct `fn NAME(` declarations per generated file, counting a
name as unreferenced when it occurs exactly once in its own unit.

**φ² + φ⁻² = 3 | TRINITY**
