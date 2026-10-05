---
name: specs-native-dialect
description: Rewrite a .t27 spec that silently discards tokens into the native clause dialect the parser keeps. Use when `t27c parse-complete` / `tri discard top` names a spec, or a suite row says parse-no-discard.
---

# specs-native-dialect

The parser reaches EOF on these specs but throws tokens away; they never reach
codegen and the suite only stays green because the ledger
(`docs/reports/suite_expectations.json`) amnesties them. Each rewrite lowers a
pinned number; the ratchet then holds it.

## 0. Before touching a spec (learned the hard way, AUTO-LOOP 2026-09-29 iter 17)

1. `git fetch origin master` and check whether master already rewrote it:
   `git log origin/master --oneline -- <spec>`. Master's #5572 and #5575 had
   redone 19 of the same specs in parallel, and PR #5084 sat CONFLICTING,
   668 commits behind. On overlap, **take master's side** (it is reviewed and
   merged) and carry only the specs that are still unique.
2. Measure on the Railway lab, not the Mac (memory `t27c-railway-lab-and-steward`).
   The lab's `/runs/<sha>/suite.log` lists every `FAIL parse-no-discard` with
   its token count.
3. Merging master in: `--theirs` is right for specs master already fixed, but
   **never** for `bootstrap/stage0/FROZEN_HASH` when both sides touched
   `bootstrap/src/compiler.rs` -- the merged file is a third file. Write
   `<shasum -a 256 bootstrap/src/compiler.rs>  bootstrap/src/compiler.rs`
   (the lab's frozen-hash gate prints the live hash too). Run 4d09237cf went
   red on exactly this.
   After the merge, the lab's `specs-parse` gate is the check that a
   *conflict-free* textual merge did not lose a line: 4d09237cf auto-merged
   contrastive_loss.t27 and dropped fn forward's closing `}` (bab50665c red).
   Then re-bless the ledger at the merged head on the lab -- master's ledger
   lacks the entries the branch's parser fixes expose.
4. `tri discard locate --n 5 --lines 30` prints the ranking plus the dropped
   lines of the largest specs in one call. `tri discard classify` gives the
   recovery channel.
5. After editing, before committing: `tri lab-parse <spec>...` ships the
   working-tree files to the lab and prints what the parser still drops
   (exit 1 if anything does). One call per batch, not one push per attempt.
   Read the bottom of the table below first; most rounds hit a known poison.

## 1. Poison -> proven form

| Poison (dropped) | Proven native form (kept) |
|---|---|
| `then x == void` | `then` with a real comparison, or drop the clause |
| `test my test {` (unquoted name) | `test "my test" {` / snake_case name |
| bare statement call `f(x)` in a clause | `assert f(x) == expected` |
| `;` after `then ...` | no terminator |
| `a implies b` | `assert !a or b` |
| `forall x in ...` in clause position | per-point instantiation, with the original kept as a `//` comment |
| `x in {a, b}` | `x == a or x == b` |
| `.length()` method calls | `len(x)` or the module fn |
| `not x` | `!x` |
| `.{ .f = v }` single-field dotted literal | colon form `T{ f: v }` |
| Zig `[_]T{...}`, `_ = f();` | `[v] ** N`, `[v; N]`, `let _r = f();` |
| prose `measure:` / prose theorem lines | `//` comment + a computable witness `assert` |
| uncommented doc prose (API listing inside a markdown fence) | `//` the exact lines `parse-complete --show` names; nothing else |
| `var x = ..; x.f = ..;` mutation in a test/invariant | fixture fn returning the value (`fn locked_cell() -> T { var c = ..; c.f = ..; return c; }`), `given x = locked_cell()` |
| `for`/`while` loop in a test | helper fn holds the loop (`fn step_n(s, n) -> S { var i: u32 = 0; while (i < n) {..} return s; }`) |
| `test_name` (underscore-joined keyword) | `test name` |
| `invariant name:` (trailing colon) | `invariant name`, indented under the module |
| `\|x\|` absolute value, `a ~= b within t` | `abs(x) < t`, `abs(a - b) < t` |
| `_ = f();` or `f()` in a **bench** body | `var r = f();` then `_ = r;` (`void` f: fixture fn that calls it and returns a value) |
| theorem the spec cannot compute (e.g. Reidemeister invariance) | `//` comment saying NOT CHECKED and why -- never an invariant the suite pretends to run |

Fixtures and helpers are ASCII `->` even when the file's own signatures use
`→` (L3). Keep the original clause as a `//` comment above its rewrite when
the form changed (implies, forall), so a reviewer can check the logic.

Kept, safe to use: `&x` args, colon-form struct literals (also inside fn
bodies), when->when->then chains, `::` paths in assert, or-chains in assert.

## 2. Loop per spec

rewrite -> `tri lab-parse <spec>` until clean -> push the sha -> lab enqueue
(`python3 /app/lab.py enqueue <sha>` over railway ssh, explicit ids) -> read the
suite gate: RATCHET CLEAN and the spec's count down -> bless the lower pin
(`t27c suite --ratchet --bless-expectations`, on the lab) -> commit `Refs #N`.

Never mass-reseal. After every commit, check `.gitattributes` for the stray
`.claude/skills/ci-gates/SKILL.md merge=union` line and revert it.
