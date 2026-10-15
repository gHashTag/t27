# Parser and duplicate-body repair (#8756)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Compiler and tri built from
master `12c6d9140`; t27c-bootstrap 0.5.2, Zig 0.16.0, native arm64 t27b.
This complements the executable test repair #8745 and graph repair #8752.

## Changes and their limits

The two seal fixtures lost an unmatched final brace. Their tests check
only their local true value; the old comments claiming project acceptance
were removed. Their old module/parent combinations also collided as
`a_b_c_spec.json`. The second module now has a distinct fixture name so
both have separate, verifiable seals. No repository code imports these
fixture modules.

This avoids a collision in these two fixtures; it does not fix the general
compiler naming algorithm. An isolated replay proves two colliding saves
both return 0, overwrite the same file and leave the first spec's verify
failing (exit 1). That remaining compiler defect is recorded, not claimed
as repaired. Replay: `seal-collision-repro/` under the evidence directory.

The optimizer's two true-valued placeholders did not execute their claimed
dead-store behavior, and their malformed commented given/then blocks
discarded 84 tokens. Their deferred requirements remain explicit prose;
the fake test declarations were removed. This is parser recovery, not an
implementation or passing native test claim for the optimizer.

`ast_shape` reuses `ast_scan::clear` and `ast_scan::mask`, now public, instead
of copying their bodies. `int_cast_plan` and `opaque_plan` reuse the already
public `type_text::same`. Existing executable assertions remain intact.
The duplicate-body ledger is unchanged.

The orchestrator fixture observes only ten command lengths, not text.
Its numeric table has exactly the original lengths and generates Verilog,
where the original local 2D string aggregate did not. All original loop
counts and assertions remain. A runtime regression checks each numeric
length against the original text. These are synthetic loops: no actual
command lookup, sacred-score calculation or performance measurement.

## Executed validation

| Spec | t27c/Zig | Native t27b `--check` |
|---|---|---|
| a/b_c | 1/1 | 1/1 |
| a_b/c | 1/1 | 1/1 |
| ast_scan | 11/11 | 11/11 |
| ast_shape | 13/13 | 13/13 |
| int_cast_plan | 13/13 | 13/13 |
| opaque_plan | 15/15 | 15/15 |
| orchestrator_bench | 4/4 | 4/4 |

All 58 executable tests pass without vacuous tests. All eight changed
source specs parse with zero discarded tokens. Seven generated seals
verify their spec and C/Rust/Verilog/Zig hashes. The optimizer is excluded
from passing-seal claims because its existing native compiler blockers
remain; this repair only restores its complete parse.

The ordinary `t27c seal --save` command also refreshes both optimizer
seal files, explicitly recording tests as BLOCKED by the duplicate enum
member `String`. It needs no force option. Their generated hashes verify,
but that is not a passing test result or a native optimizer implementation.

`cargo test --release -p t27b` passes after regeneration, including the
compiler's integration and differential tests. The duplicate-body ratchet
passes with 669 bodies in 194 known groups; its 11 negative-control shapes
also pass. No baseline blessing was performed. Changing pipeline's length
from 8 to 7 in a temporary source copy fails exactly the new length test,
exit 1, while the three old loop tests still pass. The lookup fixture now
executes 310011 runtime assertions across its four tests.

Evidence: `/Users/ssdm4/trinity-results/spec-parser-8756/`. Generated output
and seals are produced by t27c, not hand-edited. Generated C, Rust and
Verilog seal outputs are not claimed as independently executed; cargo's
t27b generated Rust integration is additionally built and tested.

The final integration also regenerates the five tracked Rust copies that
L2 identified as stale: ast_shape, int_cast_plan, opaque_plan and their
dependent coerce_plan/const_div_plan outputs. ast_scan's tracked copy is
already byte-identical. Each replacement is the unchanged compiler's
stdout for its corresponding `.t27` spec, with no manual Rust edits.

Before the upstream refresh, the integrated 1824-spec corpus ratchet is CLEAN: 71 known primary failures
match the existing ledger, with no unexpected failures, passes, expiries,
discard drift or gate drift. This is not an all-passing corpus claim.
The graph repair removed the conflicting Episode definition; only that
resolved name is removed from `type_conflicts.json`, tightening its set
from 87 to 86 instead of blessing new conflicts.

## Task-chain disposition

1. Request: restore eligible owner-authored specs and green merge checks.
2. Current instructions/base/locks checked; isolated worktree used.
3. Own issue #8756 defines the parser/helper repair and its scope additions.
4. All behavior changes live in `.t27` source.
5. No handwritten foreign implementation, gate weakening or attribution rewrite.
6. Embedded vectors, tests, negative control and seals recorded above.
7. Own PR #8749 carries this repair together with #8745 and #8752.
8. Self-review preserves existing test assertions and dependency signatures.
9. Full reference corpus is clean against its existing failure ledger. The
   latest-head native check requires the separately documented #8767 repair
   and measured pass records; CI must remeasure that final source.
10. Maintainer action is needed only if an ordinary merge is unauthorized.
11. Merge and issue closure are pending until applicable checks pass.
12. Publication is the accepted spec version; no service release is required.
13. No hardware, complete inference, customer activity, reward or performance
    claim follows from these spec tests.
14. This report and the PR retain completed work and remaining limitations.

## Upstream refresh at 102fe3b6d (2026-10-10)

The published head 8db413a0d had 30 successful CI checks and one failed
Verilog widths check. Earlier cancelled duplicates of fix-carries-source and
check-linked-issue each have a later successful check. No merge is claimed.

A normal merge of upstream master 102fe3b6dcc242660df9d8903d0de93f6bd58010
preserves its authors and the new extern-module root test in ast_shape. That
test uses ast_scan's public helpers, as the existing repaired tests do. The
native expectation conflict retains both the viewport pass and upstream vault
passes. The upstream native failure cap is now 9; this repair does not raise it.
The ast_shape seal is refreshed with the ordinary, unforced command, and Rust
is regenerated from the combined specification.

Executed checks on the combined source:
- ast_shape: 13/13 in Zig and native --check, 104 runtime assertions in each,
  no vacuous test; zero discarded tokens and all five seal hashes match.
- All six related tracked Rust outputs remain byte-identical to t27c gen-rust.
- Full cargo test --release -p t27b passes. Repaired viewport, counter, graph,
  int-cast, opaque, orchestrator and structure-validator native tests pass.
- Corpus ratchet CLEAN: 71 known primary failures, zero unexpected failure or
  pass, expiry, discard drift or gate drift. This is not an all-pass corpus.
- Seal coverage: 2393 scanned, 2272 hold and 121 known broken. Currency: 2296
  current, zero stale generated hashes, 94 missing paths and three none hashes.
- Duplicate and type-conflict ratchets pass; classification is 86/86.
- Verilog widths still fails on the unchanged upstream big_frame specimen:
  4799999/9599999-bit ranges. No exception or gate weakening is added.
- rust_repeat_counts remains BLOCKED on the existing repetition-expression
  generator error. It is not repaired or resealed as passing.

Evidence: upstream-refresh-* and upstream-ast-shape-* files under
/Users/ssdm4/trinity-results/spec-tests-7641/. Publication of the refreshed
head and its CI still remain; the earlier CI does not certify this merge.

## Publication refresh at d2570f4a8 (2026-10-10)

The owner approved updating the existing PR. Head 4ea165b3d was published
with both GitHub author and committer resolved to dmitrii-f-t27. Upstream
advanced meanwhile; a second normal merge incorporates
d2570f4a8a391151e4a8ee246217ff02bbe6e767 and preserves upstream authorship.
Its only conflict is the ast_shape seal for changed imported verilog_plan;
the ordinary unforced seal command refreshes it. No source or generated
Rust changes are needed for ast_shape, and all six related generated Rust
outputs still match. Native ast_shape passes 13/13 with 104 assertions.

The rebuilt native compiler suite passes. Corpus ratchet remains CLEAN with
71 known primary failures. Seal coverage is 2394 scanned, 2273 hold and 121
known broken; currency is 2297 current, zero stale hashes, 94 missing paths
and three none hashes. Duplicate bodies and type-conflict classification
86/86 pass. The unchanged big_frame Verilog width failure remains; no cap,
ledger exception or gate weakening is added. Evidence: publication-refresh-*
under /Users/ssdm4/trinity-results/spec-tests-7641/. CI for the new published
head must still be checked; no merge or issue closure is claimed.

## Big-frame width restoration (#8756, local day 2026-10-10)

Final CI at a2028f8a7 had 30 successful checks and one failure: Verilog
widths, run 38097688423. Dependent #8771 at e6f94ca0b had the same result,
run 38097752265. Both were MERGEABLE; no ordinary merge was performed.

The reference Verilog emitter treats any explicit local array initializer,
including `undefined`, as a packed value (compiler.rs, primitive local array
lowering). In big_frame, this produced 4800000-bit low/high and 9600000-bit
block registers. These function-local byte arrays are indexed only, never
passed or returned whole. T27 supports an uninitialized typed declaration;
the Zig emitter supplies `= undefined` and native lowering uses the same
uninitialized aggregate path for either spelling.

The spec now omits the three redundant initializers. Array lengths 600000
and 1200000, every index/write/read, recursion and all five tests are intact.
The generated Zig is byte-identical to the previous version, as is Rust.
Native assembly instructions and addresses are identical; only source-line
comments moved because of the added explanation. Verilog now emits 8-bit
unpacked memories with the original numbers of elements. This restores this
spec's layout; the general initializer decision in the frozen compiler is
not changed. No compiler source, FROZEN_HASH, gate, baseline, cap or tracked
foreign implementation is edited. No hardware execution is claimed.

Executed: normal unforced seal --save gives 5/5, zero vacuous tests; seal
--verify matches all five hashes. Native test --check gives 5/5 and 16 runtime
assertions. The initial probe's reference test-report also executed all five
tests and 16 assertions. Full check_verilog_widths.py --require scans 1854
specs and reports two previously known specimens, zero new; its ledger is
unchanged. Corpus and seal-wide checks are recorded below once complete.
Evidence: big-frame-* under /Users/ssdm4/trinity-results/spec-tests-7641/.
The complete corpus ratchet is CLEAN: 71 known primary failures, zero
unexpected failures/passes, expiries, discard or gate drift. This is not an
all-pass corpus. Seal coverage remains 2394 scanned, 2273 hold, 121 known
broken; currency has 2297 current and zero stale generated hashes (94 missing
spec paths, three none hashes). Fresh published-head CI remains necessary
before merge.


## Conflict refresh at c4fc14c2d (2026-10-11)

GitHub marked #8749 and dependent #8771 CONFLICTING after upstream
repaired overlapping specs. A normal merge incorporates master
`c4fc14c2d83f2470ca468f3d72a3f8457f17051f`, preserving upstream authors.
The seven conflicts are resolved in the source specs and ordinary seals.
The graph keeps the previously measured admission/disclosure policy and
its twelve executable tests. The two fixtures keep distinct module names
and their honest local-value assertions. Optimizer keeps upstream's new
executable test blocks; its two true-valued placeholders remain deferred
prose. Its reference execution still blocks on duplicate enum member
`String`; a matching generated hash is not an optimizer pass.

The helper refactors and big-frame memory declarations already remove
the defects for which upstream added exceptions. This merge removes the
obsolete `bits`, `same`, secondary `clear` and big_frame exceptions. The
first complete corpus run found no new failure and one now-fixed entry,
orchestrator_bench. Only that entry is removed; max_entries falls72 to71.
The native failure cap inherits upstream0. No cap is raised, no check
is disabled, and neither the frozen compiler nor its seal is edited.

Fresh measured checks on the merged source:
- Rebuilt t27c, tri and t27b; native cargo suite161 tests, zero failures.
- Native fixture/graph/big_frame/structure tests34/34,613 runtime assertions.
- Ordinary seals: fixtures1/1 each and graph12/12; five hashes MATCH.
  Optimizer seal records BLOCKED, not failing or passing execution.
- Width scan1858 specs: two known, zero new, without a big_frame exception.
- Duplicate scan671 bodies in195 groups; no new or growing group.
- Seal coverage2392 scanned,2271 hold,121 known broken. Currency2295
  current,zero stale hashes,94 missing sources,3 none. Type ratchet86/86 CLEAN.
- All six related tracked Rust copies are ordinary gen-rust byte-identical.

Evidence: master-c4fc-* under
`/Users/ssdm4/trinity-results/spec-tests-7641/`. The corpus recheck and new
head CI must finish before any merge claim; old31green checks certify only
a34891bd3/cdac218f0. No hardware, model inference or reward is claimed.

The final full corpus recheck is CLEAN: 71 observed primary failures match
71 known entries, zero unexpected failures/passes, expiries or drift.
This remains a ratchet result, not an all-passing corpus.


## Conflict refresh at 49a458676 (2026-10-11)

Upstream compiler changes refreshed the big_frame seal and made both
own PRs conflict. A normal merge preserves upstream compiler, FROZEN_HASH,
optimizer lowering and executable tests without a local foreign-code edit.
The only conflict was big_frame seal metadata and generated hashes; the
source stays unchanged. An ordinary unforced seal on the rebuilt compiler
passes5/5 tests and verifies all five hashes.

Fresh checks: cargo native161 tests pass; targeted native34 tests/613 runtime
assertions pass; full corpus ratchet CLEAN71/71 with zero unexpected failures,
passes, expiries, discard or gate drift. Width1865 specs:2 known/0 new;
duplicates671 bodies/195 groups CLEAN; type86/86 CLEAN. Seal coverage2399:
2278 hold/121 known broken; currency2302 current/0 stale/94 missing/3 none.
All six related generated Rust files remain byte-identical. Optimizer reference
execution still BLOCKED by duplicate enum String; its matching seal is not a pass.

No native cap increase or new missing-spec enrollment is made. Before this
refresh, latest-head CI for3154c5626/b5d996130 was30 successful/1 failed;
only native ratchet failed on four passing upstream files absent from the
ledger. That remains an unresolved CI blocker. New-head CI must be checked
after publication. Evidence: master-49a-* and NATIVE-CI-DIAGNOSIS-2026-10-11.md
in /Users/ssdm4/trinity-results/spec-tests-7641. No hardware, model inference
or reward is claimed.


## Verified native pass enrollment (2026-10-11)

The owner explicitly authorized adding these ten passing specs to the native
expectation ledger and updating existing PRs#8749/#8771 without raising limits.
Latest-head CI artifacts for8a3a77584 (run38129854938) and7c468191c
(run38129931526) report the same ten UNLISTED files and no other ratchet violation.
All ten execute and pass in reference and native, with identical per-test
verdicts,103 tests and4342 runtime assertions. Their source blobs are
unchanged upstream.

Only ten `t27b: pass` rows are inserted; every existing entry, failure cap0
and ledger metadata remains unchanged. This registers demonstrated success;
no failed or unsupported spec is admitted, and no source, test, seal, compiler
or workflow changes. New behavior/spec generation is therefore inapplicable.
The normal ratchet replay of CI run8a3a77584 with the updated ledger is GREEN,
all nine violation counters0 and no findings. Input execution belongs to the
old source SHA; a new-head CI result is still required after this data-only edit.
No passed execution suite is repeated. Evidence: native-ci-8a3a775-artifact,
JTAG native-ci-7c468191-artifact and enrollment-pass10-ratchet.json under the
respective /Users/ssdm4/trinity-results directories.
