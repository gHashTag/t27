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
   It also prints `VACUOUS <spec>: N` and the N names -- invariants the
   generator cannot lower. A name listed twice is declared twice (tokenizer:
   two of them); rename the second. An invariant calling a fn the spec does
   not define becomes a NOT CHECKED comment, not an assert (1374c29d2). A discard MASKS that phase (the spec fails upstream), so fixing the
   discard turns N > 0 into a new primary failure: the ratchet goes red with
   `UNEXPECTED FAILURES ... [no-vacuous-invariant]` (a6ed71d4c,
   multi_lang_harness). Get N to 0 in the same commit (forall-over-type ->
   check the producer fn on two inputs), or leave the spec for an owner
   decision: moving a ledger entry to a later phase is the owner's call, not
   the loop's (permission denied 2026-10-05).
   Then it runs `typecheck` and `gen-verilog`, the phases after that, on the
   edit AND on the lab's checkout of the same path. Only `NEW` (passes there,
   fails here) counts as a problem. 0 problems on a spec whose ledger entry is
   `no-vacuous-invariant` means delete that entry in the same commit, or the
   ratchet reports an unexpected pass. Removing an entry tightens the ledger;
   moving one does not (cec0294c0: 4 igla specs, 13 invariants).
   Ready queue: `tri ledger-queue` (lab suite.log of HEAD x the ledger,
   smallest count first; `--log FILE`, `--phase P`, `--all`). It flags
   `fragment` (a hand-written `.zig` beside the spec -- `igla/race/cordic.t27`,
   skip), `masked` (an earlier phase fails too) and `absent` (ledger entry
   with no FAIL: delete it). Editing a spec can stale its seal; diff the
   lab's `/runs/<sha>/seal-currency.log` against the previous run -- a new
   line is yours (059ba17: none; PhiRatio was already stale).
   `parse-complete --show` is stricter than the suite's `parse-no-discard`,
   which counts TOP-LEVEL tokens only. cordic drops 64 tokens inside its
   invariants and is still "clean" in phase 1a2.
6. Brace and keyword traps found 2026-10-05: a lone `}` at EOF with one more
   `}` than `{` in the file is dropped -- count both before deleting it. A
   hyphenated module name with a keyword part (`igla-coder-bench-proxy`)
   loses the rest of the line; the fix is in the parser (compiler.rs
   module-name loop), which moves FROZEN_HASH -- a freeze ceremony, owner
   only. Do not rename the module: its seal file is keyed by the name.

7. Lowered is not TRUE. `tri lab-exec <spec>...` generates the Zig on the lab
   and runs `zig test`: invariants are `comptime` blocks there, so a false
   claim is a compile error. It prints FALSE (fix the spec, keep the original
   as a comment), UNDECLARED (a name or field nothing defines -- NOT CHECKED
   or define it), CODEGEN (generator bug: compiler.rs, owner only, report it)
   and IMPORT (an error in a `use`d module). First run, 2026-10-05: constants
   `phi_golden_conjugate` (sqrt(5) == phi) and three tokenizer "empty input
   is BOS only" claims were false. Run it after lab-parse is clean. Do not
   reseal on the lab to clear a stale seal your edit caused: the lab's PATH
   has no zig, so the seal records `tests: blocked` -- leave it stale and
   say so.
   Literal traps that block lab-exec for the whole file (zig stops at a
   parse error, 2026-10-05 weights.t27): `[0:i8]` is emitted verbatim --
   write `[0 as i8]`; `[0u8; 12]` becomes `{ 0u8;12 }` -- write the zeros
   out. Generator bugs to report, not dodge: `x as i8` on an f32 lowers to
   `@intCast`; array-of-struct literals become tuples `.{...}` that do not
   coerce to `[]T`; string `a + b` is `pointer + pointer`. An invariant
   that hits one is lowered but not executed -- say so in the commit, and
   hand-evaluate it (weights int4: six "roundtrip is identity" claims were
   false, 0.5 -> 3/7). Before pinning a value, read the fn body: a
   generate_prompt("divider") sample was itself false (no such template).
   Before fixing a FALSE in a shared spec, `git log origin/master -- <spec>`:
   ternary_encoding bits_trits_roundtrip is already recorded false (#5286).
   Runtime counts too (2026-10-05): `test` blocks run at runtime, and a
   panicking assert used to stop `zig test` while lab-exec still said
   "compiles" -- 12 false ternary_inference tests and 6 more specs hid that
   way. lab-exec now patches `__t27_assert_fail` on the lab copy so every
   failing test prints a FALSE `test <name> (runtime)` row, and a real panic
   (overflow, out of bounds) is FALSE with "run stopped, later tests not
   executed". "compiles" with no "All N tests passed" line is not a pass.
   Runtime traps: Zig globals are shared by every test in the file, so a
   test that mutates module state (timing_tb `clk`) depends on order --
   assert relative to a value read first. f64 `==` on a computed value is
   false (1/3 vs 0.333, Newton sqrt 25.000000000167777): tolerance, L5. A
   raw array where a fn takes a model struct needs the spec's constructor
   (`load_ternary_weights(w)`); `[x] ** N` for repeats. Args swapped in a
   `mac(acc, a, w)` call compile and give wrong numbers -- compute the
   expected value by hand from the fn body before pinning it. A whole-array
   `then r.outputs == [..]` claim: expand per element, computed. A
   `sed`-style bulk fix also rewrites the `// original:` comment -- anchor
   on the test name. "no member named" is UNDECLARED, not CODEGEN.
   Ratchet, not absolute (2026-10-05): a full sweep has ~240 known-false
   specs, so a bare exit 1 says nothing. `tri lab-exec --ratchet <spec>...`
   compares FALSE + UNDECLARED per spec with
   `docs/reports/lab_exec_false.json` and fails only on a rise;
   `--bless` writes the run's counts (lower them after a fix, in the same
   commit); `--from-log FILE` replays a saved run without the lab. A
   "RATCHET UP" right after the detector itself improved (signal crashes:
   bellman_ford, self_attention, multi_head_attention) is detection, not a
   regression -- bless it and say so in the commit.
   A fall while CODEGEN stops the compile is "masked", not better
   (bellman_ford `&e` -> `[]Edge` first went 1 -> 0 that way).
   Vacuous family: `tri stub-census [--list] [--kind no-input|void|stub]`
   lists tests/invariants whose `then` is `result != undefined` or `true`
   (514 in 124 specs, 2026-10-05). `no-input` alone = give real values,
   computed by hand (elu_activation: 4 tests, 3 invariants pass); `void` =
   return something observable (bellman_ford -> i64, NEG_CYCLE); `stub` =
   owner decision, do not invent an implementation. `math::exp` is
   UNDECLARED in the Zig; `@exp(x)` works. A read-only slice param is
   `[]const T` -- `&e` of a `given` value is `*const [N]T`.

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
| `invariant x:` at column 0 then an indented body | `invariant x` (no colon); a one-line `invariant x: true` is kept but VACUOUS |
| `target latency < t` inside a **test** | `// Target: ...` comment -- the whole test is dropped otherwise |
| `f(x).field = #[0, 0]` (prose equality on an array) | `given r = f(x)` then `assert r.field[0] == 0 && ...` |
| `forall s : T, pred(s)` (quantifier over a type) | NOT lowered (VACUOUS). Check the producer: `assert pred(make(a)) && pred(make(b))` |
| `assert f(n) = closed form for all n` | instantiate at 3-4 values, comment keeps the formula |
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
