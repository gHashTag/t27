# JTAG verdict spec literal repair (#8762)

Author: Dmitrii Fedorov (@dmitrii-f-t27). Parent corpus issue: #7641. This branch includes the dependency PR #8749.
Native tooling rebuilt after synchronizing with upstream cd569e3e0.

## Change and scope

The existing MVP classifier JTAG no-port specification used Rust-style `u32`
suffixes in number expressions. T27 already declares each required u32 context;
removing those unsupported suffixes preserves the values and 32-bit operations.
The original `StartupConfig.prog_usr: &str` type is retained. A literals-only
probe confirms it is supported; no shared configuration module is needed.

All original test cases and six invariants remain. The specification models
verdict-word packing, capture/shift gating, old-state TDO, initial register,
LED-derived flags and BSCAN/STARTUP configuration values. It does not instantiate
BSCANE2/STARTUPE2, synthesize a classifier, flash a bitstream or execute physical
JTAG. No complete model inference is claimed.

## Executed checks

- Complete parse: zero discarded tokens.
- Reference t27c/Zig: 18/18 tests and six compile-time invariants. Sixteen tests
  execute 28 assertions; the two constant-construction tests execute zero
  runtime assertions and are reported as vacuous by the reference counter.
- Native arm64 t27b `--check`: 18/18, six invariants held, 38 runtime assertions
  including constant checks and invariants; JIT/interpreter agree.
- A temporary copy expects A5A5A5A6 instead of A5A5A5A7 for beat=true, ok=true.
  Both runners fail exactly that test, exit 1, with 17 others passing.
- Normal, unforced `seal --save` records 18/18 and two reference vacuous tests.
  `seal --verify` matches spec and generated C/Rust/Verilog/Zig hashes. Only
  the Zig and native T27 paths are execution claims; other hashes prove generation.
- The native expectation ledger adds this measured pass without increasing
  its failure cap (10). Duplicate-body and type-conflict ratchets pass.

Evidence: `/Users/ssdm4/trinity-results/jtag-verdict-spec/`, including final
source, literals-only probe, both damaged-result reports and seal verification.

## Task-chain disposition

1. Request: repair eligible specs and close own work under @dmitrii-f-t27.
2. Existing source, original hardware boundary and corpus blocker checked.
3. Own issue #8762 specifies this exact literal-only scope.
4. Source change is entirely in the existing `.t27` specification.
5. No handwritten foreign code or author remapping.
6. Existing vectors, six invariants, negative control and normal seal exercised.
7. Own PR must link this report, close #8762 and reference #7641.
8. Self-review checks unchanged values/types and old-state TDO behavior.
9. Latest-head CI remains pending after publication.
10. Ordinary merge needs usable upstream write permission; no independent
    GitHub review is required under the owner's standing instruction.
11. Merge and closure remain pending until checks/access permit them.
12. Publication is the accepted spec; physical FPGA/service deployment is
    outside this literal restoration.
13. No physical inference, customer activity or reward claim follows.
14. This report retains completed tests and the remaining publication step.

## Dependency refresh at upstream 102fe3b6d (2026-10-10)

The normal merge of local dependency 4ea165b3d retains the JTAG pass row and
upstream's new native failure cap 9 without increasing it. It contains the
upstream extern-module root test and the combined ast_shape seal and generated
Rust. JTAG source and values are unchanged by this refresh.

The refreshed source again passes 18/18 reference tests with six compile-time
invariants and the same two vacuous constant tests. Native --check passes
18/18 with six held invariants and 38 runtime assertions. Complete parse
reports zero discarded tokens; spec and all four generated hashes match the
existing seal. No hardware execution is claimed. The dependency's full native
suite, corpus ratchet, seals and duplicate/type gates pass locally. The known
upstream Verilog widths failure still blocks an ordinary merge.

Evidence: upstream-refresh-zig.log and upstream-refresh-native.log in
/Users/ssdm4/trinity-results/jtag-verdict-spec/. Earlier CI at e10301252 is
not a CI receipt for this refresh; publication and latest-head CI remain.

## Publication refresh at upstream d2570f4a8 (2026-10-10)

The owner approved updating this PR, and head 223115f2b was published with
GitHub author and committer resolved to dmitrii-f-t27. Upstream advanced
meanwhile. A normal merge of refreshed dependency a2028f8a7 incorporates
d2570f4a8a391151e4a8ee246217ff02bbe6e767 without conflict. JTAG source,
generated hashes and sealed values remain unchanged; its measured native
pass row is retained and the upstream failure cap remains 9.

The actual reference test-report passes 18/18 with six invariants and the
same two vacuous constant tests; native test --check passes 18/18, six held
invariants and 38 runtime assertions. All five seal hashes match. Evidence:
publication-refresh-zig.log, publication-refresh-native.log and
publication-refresh-seal.log under /Users/ssdm4/trinity-results/jtag-verdict-spec/.
Latest-head CI is still required; the dependency's known Verilog widths
failure remains. No merge, closure or hardware execution is claimed.

## Large-frame width dependency repair (local day 2026-10-10)

Final CI at e6f94ca0b had 30 successes and only the inherited Verilog widths
failure. A normal merge of dependency a34891bd3 brings its spec-only repair:
three redundant undefined initializers on index-only large byte arrays are
omitted, with the same lengths, tests and native/Zig behavior. The dependency
report records identical Zig and native instructions, 5/5 tests, 16 assertions,
normal seal, clean corpus ratchet and width scan with zero new violations.
No compiler, gate, cap or exception change is included.

JTAG source and seal are unchanged, and seal --verify still matches all five
hashes (big-frame-dependency-jtag-verify.log in this report's evidence directory).
Earlier JTAG execution results remain applicable to that unchanged source;
they are not described as new executions. Fresh published-head CI remains
required, and upstream merge access is still absent.


## Conflict refresh at c4fc14c2d (2026-10-11)

A normal dependency merge of own #8749 head
`3154c56264ef9c6f421693520e93d1ff52c40923` brings upstream
`c4fc14c2d83f2470ca468f3d72a3f8457f17051f` without conflicts or history
rewrites. Its full corpus ratchet is CLEAN at 71 known failures, width
scan has two known and zero new violations, and native cargo161 tests pass.
It removes obsolete upstream helper/big_frame exceptions and one now-fixed
corpus entry, lowering72 to71; the native cap inherits upstream0.

The JTAG source and its seal remain unchanged. On the rebuilt native
backend, its18 tests and6 invariants pass with38 runtime assertions and
JIT/interpreter agreement; all five seal hashes MATCH. This is a new native
check for the changed backend, not a repeated reference run or hardware
execution. Evidence: master-c4fc-native.log and master-c4fc-seal.log under
`/Users/ssdm4/trinity-results/jtag-verdict-spec/`. New published-head CI is
required; old31green checks do not certify this dependency merge.


## Dependency refresh at 49a458676 (2026-10-11)

A normal merge of own dependency8a3a775844d717d23b48c0d86fc73a54e59cc12c
resolves new upstream conflicts without rewriting history. Dependency source
checks pass:161 native cargo tests, corpus ratchet71/71 CLEAN, width1865
specs2known/0new, duplicates671/195groups CLEAN, seal currency2302current/0stale
and all six generated files byte-identical. No nativecap0 increase or new
missing-four enrollment is made.

Own mvp_ternary_classifier_jtag_noport source and seal are unchanged. On the
changed backend18 tests/6 invariants/38 runtime assertions pass and all five
seal hashes MATCH (master-49a-own-native.log and master-49a-own-seal.log).
These are native execution and generation checks; no physical JTAG, model
inference or reward is confirmed. Earlier CI was30successful/1failed on
missing native ledger entries, not a JTAG execution error; fresh-head CI
and actual upstream access remain necessary before merge.


## Verified pass enrollment dependency (2026-10-11)

The owner explicitly authorized adding ten actually passing specs and
refreshing both existing PRs without raising limits. A normal merge of
dependency9e6cb1b9879f3cf14a741047350f05efcdcb3a00 adds only ten
measured native pass rows and its evidence report. Nativecap0 and every
existing ledger entry/metadata stay unchanged. Own JTAG row/source/seal
are preserved; no source or workflow modification.

This PR's own CI run38129931526 at7c468191c confirms the identical ten
reference/native passes,103 tests/4342 assertions, no per-test disagreement.
Normal ratchet replay of that downloaded run with the updated ledger is
GREEN, all nine counters0 and no findings. Evidence native-ci-7c468191-artifact
and enrollment-pass10-ratchet.json in /Users/ssdm4/trinity-results/jtag-verdict-spec.
No already-passing execution suite is repeated; fresh published-head CI
and actual upstream access still precede merge. No hardware/inference/reward claim.
