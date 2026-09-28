# plan.md — the decomposed plan

Each item is bounded (roughly one 15-minute firing), names its evidence, and
carries a verification step. Status markers: `[open]` `[in-flight]` `[done]`
`[blocked-user]`.

## 1. L1 — baseline inventory `[in-flight]`
Run the suite on the untouched master base; write `baseline.md` with the exact
failing-spec list per phase. Everything later is judged against this file.
**Verify:** the count matches a second run (suite is deterministic).

## 2. L2 — the silent-discard family (top priority after baseline) `[open]`
Baseline re-framed this item: the 9-spec typecheck cluster is one face of a
family with **114 specs silently discarding top-level tokens**
(`parse-no-discard`) and **65 specs with invariants whose bodies were
discarded** (`no-vacuous-invariant` — they check nothing). Worst:
`ternary_inference` 1813 tokens, `ternary_gemm` 1566, `systolic_ternary` 1409;
`gf8/12/20/24/32.t27` each lose 31–37.
**Work (wave-697 lineage, #4756: one construct at a time, probe-backed):**
run `tri unparsed locate` per spec; name the construct behind the biggest
discard rows; either teach the parser the construct (if legitimate language)
or make recovery fail loudly; close the census's "not decided" blind spot for
the 6 Markdown-headed specs with a probe + counter.
**Verify:** suite reds shrink vs `baseline.md`; `tri unparsed probe` rows
self-invalidate as constructs gain support.

## 3. L2 — restore truncated `specs/test_framework/verilog_bench_harness.t27` `[open]` — #5079
Ends mid-struct at line 459/460 since #1399 (Jul 31). Recover the tail from the
intact 459 lines' intent, parse+typecheck, seal, close #5079.
**Verify:** `t27c typecheck` green on the spec; seal verify green.

## 4. L2 — tri CLI additions `[open]`
Born-from-incident additions (the repo's rule: a command earns its place by a
measured story, told in its doc comment):
- `tri loop state` — read/validate this loop's `state.md` (the file every
  firing reads; parsing it wrong = duplicated work, the exact incident
  `tri loop claim` was built for, now in local form).
- checkpoint hygiene: verify branch is ahead of the last pushed checkpoint and
  suite reds did not grow vs `baseline.md`.
Incidents to cite: tonight's stale `~/.local/bin/tri` intercepting hooks;
tonight's `--out/` literal-directory accident from `t27c gen` unknown flags.
**Verify:** `cargo test -p tri` green; new subcommand self-checks.

## 5. L3 — seal-drift cause measurement `[open]`
574 stale seals on the master base (carry-tree run). Measure on pure master:
which codegen commit changed the bytes, is the change intended (like the string
quotes fix), and what the smallest honest reseal batch is. No mass reseal.
**Verify:** every resealed spec's new gen output inspected deliberately.

## 6. L3 — `.tri` codegen silent-wrongness `[open]`
Measured in the header of `specs/mcp/server_registry.t27`: `pub type X = enum`
loses its name, `pub const N u32 = 64` loses its value, enum-body comments
become phantom variants — Zig-rejecting garbage that LOOKS like output.
Minimum honest fix: detect and refuse loudly (error, not garbage) in t27c gen
for the .tri path.
**Verify:** planted-defect self-check on `specs/git/orchestrator.tri`.

## 7. L3 — `t27c` unknown-flag hygiene `[open]`
`t27c gen --out X` created a literal `--out/` directory (measured tonight; the
branch had one committed by accident). Reject unknown flags loudly.
**Verify:** negative control — `t27c gen --bogus` exits non-zero with usage.

## 8. L3 — competitor digest `[open]`
Web scan (Vericert, Calyx, Filament, Vitis HLS, Verilator/Icarus/yosys,
ternary-silicon/BitNet 2026 news) mapped against our weak points. Goes in
`competitors-2026-09-29.md` here. **Verify:** every claim carries a link.

## 9. L4 — close-out `[open]`
Issue + PR (`Closes #N`), one `docs/now/` entry, `docs/README.md` map line for
`docs/loop/`, iteration reports complete, three cooperation variants for the
next loop, skills/experience/memory updates, release the claim.

## Deliberately NOT in this loop

- Merging PRs #5078 / #5081 (owner's call).
- The lean-proofs.yml bidirectional gate (#5082) — needs `workflows` scope.
- Any push of `.github/workflows/`.
- Mass reseal of anything.
