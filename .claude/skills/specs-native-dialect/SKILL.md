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
4. `tri discard locate --n 5 --lines 30` prints the ranking plus the dropped
   lines of the largest specs in one call. `tri discard classify` gives the
   recovery channel.

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

Kept, safe to use: `&x` args, colon-form struct literals (also inside fn
bodies), when->when->then chains, `::` paths in assert, or-chains in assert.

## 2. Loop per spec

rewrite -> `zig ast-check` is irrelevant here; push the sha -> lab enqueue
(`python3 /app/lab.py enqueue <sha>` over railway ssh, explicit ids) -> read the
suite gate: RATCHET CLEAN and the spec's count down -> bless the lower pin
(`t27c suite --ratchet --bless-expectations`, on the lab) -> commit `Refs #N`.

Never mass-reseal. After every commit, check `.gitattributes` for the stray
`.claude/skills/ci-gates/SKILL.md merge=union` line and revert it.
