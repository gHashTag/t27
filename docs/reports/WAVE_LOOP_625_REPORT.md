# Wave Loop 625 — the unmeasured 14%, and why the sweep never returns

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_624_REPORT.md`](WAVE_LOOP_624_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W624 recommended Option 1 — close T21's blind spot. Done, and it paid immediately.

```
A  forced analysis of the 180 unreferenced bodies -> 1069 becomes 1104
   and T18's "9 -> 0" becomes "9 of 10; the tenth was never compiled"
B  the tenth site fixed by a taint FIXPOINT       -> forced count 1103, usize 0
C  why `t27c suite` never returns                 -> 578 MB of generator
   scratch vs 6.5 MB of real spec, all of it parsed every run

T22  forcing analysis grows the SUPPORT, not just the count
T23  expression-local taint dies at the first binding; a positional probe
     cannot see that, because the defect sits where the probe says "clean"
T24  a verification command's cost is set by its widest input glob
```

---

## 1. Variant A — the first unconditioned count

T21 established that `zig test --test-no-exec` analyses referenced bodies only,
and that 180 of 1286 generated functions (14.0%) are never referenced. It called
1069 a lower bound. This wave measured the bound: append
`comptime { _ = &f; }` for every top-level function to each generated file —
**no change to any generated logic** — and re-run.

| | reachable | forced | Δ |
|---|---:|---:|---:|
| total diagnostics | 1069 | **1104** | **+35** |
| `expected type '<sized int>', found 'usize'` | **0** | **1** | **+1** |

**T18's headline is false as an unqualified statement.** A tenth `.len` site
exists, in `coder/dataset`, in a function no test, no measurement and no previous
wave has ever compiled:

```zig
fn estimate_10k_size(base_templates: [][]const u8, bitwidths: []u32) u32 {
    const base     = base_templates.len * bitwidths.len;   // usize
    const permuted = base << 2;                            // usize
    const mutated  = permuted << 3;                        // usize
    const composed = mutated + (mutated * (mutated - 1));  // usize
    ...
    return composed;   // error: expected type 'u32', found 'usize'
}
```

**And the +35 is not the same errors at larger scale.** Three classes are zero in
every figure this project has published and non-zero under forcing:

| class | reachable | forced |
|---|---:|---:|
| `not yet implemented` (`@compileError`) | **0** | **15** |
| `invalid pointer-pointer arithmetic operator` | **0** | 1 |
| `incompatible types: '*const [14:0]u8' and 'u32'` | **0** | 1 |
| `invalid operands to binary expression: 'pointer' and 'pointer'` | 35 | 47 |

The fifteen `@compileError("not yet implemented")` are the backend's own marker
for an unwritten spec function — the population `t27c impl-status` exists to
count. They were invisible **by construction**: an unwritten function has no
callers, so nothing references it, so Zig never reaches the `@compileError`.

> **The project's two instruments — "how many specs are stubs" and "how many
> errors does the corpus have" — were measuring populations that cannot overlap,
> and neither said so.** (T22.)

---

## 2. Variant B — the tenth site, and what T20's probe could not see

W624's probe enumerated five *syntactic positions* and closed them. The tenth
site is in **none of the five**: it is `return composed;`, a bare identifier,
which the probe correctly classifies as "nothing to do here."

`len_tainted_int_expr` walked the return expression's own tree. The four `const`
bindings carrying the length are not in that tree, so the taint died at the first
one — and the site needs four hops.

**Fixed by making the taint a fixpoint over the local environment**
([compiler.rs](bootstrap/src/compiler.rs)): a local whose initializer is tainted
and whose declared type did not already absorb it carries the taint onward;
`<<` and `>>` join the operator set because the site shifts twice. A local that
the W624 `let` rule already cast is explicitly *removed* from the taint set, so
the return is not cast twice.

| | before | after |
|---|---:|---:|
| forced total | 1104 | **1103** |
| forced `usize` mismatches | 1 | **0** |
| reachable total | 1069 | **1069** |
| generated lines changed, whole corpus | — | **1** |

**No unmasking this time**: the class diff removes exactly one entry and adds
none, unlike T19's case. One line changed in 34 files.

> **T23 — an analysis computing taint by structural recursion on one expression
> is sound only when no binding introduces a tainted name.** With untyped locals
> it must be a fixpoint over `Γ`, not a fold over one term.

**T20's method has a blind spot of its own, and it is the same shape.** T20
replaced *sample the corpus* with *enumerate the class* — and indexed the
enumeration by **position**, when the class also ranges over **dataflow
distance**. A probe is a population too. Nothing about enumerating rather than
sampling protects against choosing the wrong axis.

---

## 3. Variant C — why the sweep never returns

`CLAUDE.md` §2 names `./target/release/t27c suite --repo-root .` as the local
CI-like sweep. **Two waves in a row could not complete it.** Sampling the process
found it inside `Command::output()`, draining a child that had spent minutes on
one file: `w590_bench_module_17d_aos_var_call_reassign.t27` — **14.3 MB, 786 483
lines, one function, one test**, a 17-dimensional nested array literal from the
AoS-swarm generator.

`run_comprehensive` opens with `collect_t27(&repo.join("specs"))` and runs a
`parse` phase over every result — 1064 files, 588 MB.

| | files | bytes |
|---|---:|---:|
| `specs/scratch/*x2p6*` — one generator sweep, committed iteration by iteration | 288 | **378.9 MB** |
| all of `specs/scratch/` | 455 | 578.0 MB |
| **every other spec in the repository** | **609** | **6.5 MB** |

**89 : 1 by bytes, in favour of the scaffolding.** Measured parse throughput:
**0.081 MB/s** across the `x2p6` family (linear in the outer multiplier, N = 137
… 597) against **2.75 MB/s** for `21x2p7` — a 34× spread by *shape*, so no total
is derivable from bytes and none is claimed. Directly observed: **47 minutes,
still inside the `parse` phase, no output at all** — no pass, no fail, no
progress line.

> **T24 — `cost(V)` is a function of the glob `G`, not of the artefacts `A ⊆ G`
> under test.** When a generator writes into a directory `G` admits, `|G \ A|`
> grows at no review cost, and the command stops terminating — which reports as
> neither pass nor fail.

This is §4's failure mode with the sign flipped: every entry in that table is a
stage that *silently discarded* input and reported success. `suite` silently
*admits* input and reports nothing.

**Not fixed here, deliberately.** Narrowing the glob or deleting 578 MB of
committed artefacts changes what the Icarus regression baselines in
`.trinity/icarus-baselines/` are computed against. That is a decision with a
blast radius, not a repair — it is Option 1 below.

---

## 4. What was NOT done

- **`t27c suite` still has not completed.** Its non-termination is the datum
  behind T24; nothing in this wave was verified through it.
- **No web literature this loop either.** `WebSearch`/`WebFetch` have failed with
  a provider error for the whole session. Nothing was cited that was not
  described from general knowledge under §3's standing rule, and **no citation
  was fabricated.**
- **The other 34 forced-only diagnostics were not repaired**, only classified.
- **No `.t27` spec text changed**; no FPGA step ran.

---

## 5. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --release -p t27c --bins len_` | **8 passed**, 0 failed |
| reachable corpus | 1069 / 0 usize — **unchanged**, no regression |
| forced corpus | 1104 → **1103**, usize 1 → **0** |
| class diff (forced) | one class removed, **none added** |
| generated-code diff W624→W625 | **1 line**, `coder_dataset.zig:798` |
| `FROZEN_HASH` | re-sealed, canonical two-field form |

Two new unit tests: the four-hop taint case, and the negative case that a local
already cast by the W624 rule is *not* cast a second time.

---

## 6. Three ways to continue (pick one for W626)

### Option 1 — **Make the sweep terminate: separate the generator scaffolding from the corpus**

Move `specs/scratch/` benchmark artefacts out of the `parse`-phase glob (a
`specs/bench/` tree, or a `.suiteignore`), keeping the `w5*`/`w3*` witnesses the
Icarus regression genuinely needs. Re-run `suite` end to end and record the first
complete wall time this project has.

- **Cost:** low in code, careful in review — the Icarus baselines in
  `.trinity/icarus-baselines/` are keyed to paths.
- **Pays off in:** the constitution's own verification command becomes usable,
  which unblocks every future wave's verification step.
- **Risk:** a witness quietly drops out of the regression set and a real
  behaviour stops being covered. Mitigate by diffing the baseline key set before
  and after, not the wall time.
- **Falsifies:** T24, by producing the completed run its falsification condition
  names.

### Option 2 — **Repair the forced-only classes, starting with the 34 pointer-arithmetic diagnostics**

`invalid operands to binary expression: 'pointer' and 'pointer'` is 47 of 1103
under forcing (35 reachable, 12 forced-only) and is the largest class this wave
newly characterised. Probe it by *dataflow shape* as well as position — T23's
lesson applied prospectively rather than retroactively.

- **Cost:** medium-high; likely a real lowering gap for slice arithmetic.
- **Pays off in:** the largest single class after `use of undeclared identifier`,
  and the first fix designed with T23's axis-choice warning in hand.
- **Risk:** T19 applies — expect unmasking, and budget for the class table rather
  than the total.
- **Falsifies:** T23's corollary, if a positional enumeration turns out to cover
  this class completely.

### Option 3 — **Land forced analysis as a first-class mode and make it the default measurement**

Add `t27c gen --force-analysis` (emitting the `comptime` reference block) and
re-baseline every error figure in `IGLA-FORMAL-RESULTS.md` against it, marking
each existing number as reachability-conditioned. Today the forced measurement
exists only as a post-processing script in a scratch directory.

- **Cost:** small compiler change, large documentation pass.
- **Pays off in:** T21/T22 stop being a caveat and become the default; no future
  wave can publish a reachability-conditioned figure by accident.
- **Risk:** re-baselining touches ~15 published propositions and is exactly the
  kind of bulk edit that has produced wrong numbers here before (P35: the
  register was 15-of-16 wrong). Do it one proposition at a time, with the
  measurement re-run for each.
- **Falsifies:** nothing — it is bookkeeping. That is also the argument against
  doing it first.

**Recommendation: Option 1.** It is the only one that unblocks *verification
itself*, and the last two waves have both ended with "could not complete the
sweep." Option 2 is the right follow-on once there is a sweep that returns.

---

## Appendix — reproduction

```bash
cargo build --release -p t27c
cargo test --release -p t27c --bins len_
```

Forced measurement: for each generated `.zig`, append
`comptime { _ = &f; }` for every top-level `fn` (excluding `__t27*` helpers),
then `zig test --test-no-exec`. Class table:
`grep -oE 'error: .*' | sed 's/[0-9]\+/N/g' | sort | uniq -c`.
Corpus sizes: `find specs -name '*.t27' [-not] -path 'specs/scratch/*'`.
Parse throughput: wall time of `t27c parse <file>` over the
`w{601,659,693,732,770,810,849,889}` ladder.

**φ² + φ⁻² = 3 | TRINITY**
