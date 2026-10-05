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
   First move on a `no-input` spec: `tri drop-vacuous spec...` (dry run),
   then `--write`. It deletes a vacuous block only when another non-vacuous
   test already calls the same fn, and lists the rest as KEEP = "write
   inputs by hand" (2026-10-05: 9 specs, 40 blocks; constants 12 -> 0, all
   17 tests pass). It also drops `use std;` (duplicate struct member 'std').
   More generator traps seen in http (1686b570f): `if (x) |end|` capture is
   not lowered (write `if (x != null) { const end = x.?; ...}`); a
   `struct { enum : [A, B] }` has no `.A` (write `enum { A, B }`);
   `@ptrFromNull` is not a Zig builtin; open slice `s[a..]` lowers to `s[a]`
   and `==` on `?[]const u8` to `std.mem.eql` without unwrap (CODEGEN, report).
   The struct-enum form is defect #3225 and has its own command:
   `tri enum-fix [--write]` (b6c6370f9: 11 specs). Then let the lab suite
   name the UNEXPECTED PASSes and remove exactly those ledger entries in
   the next commit -- a fixed declaration can still fail typecheck for
   another reason, so never remove an entry on the strength of the rewrite.
   `tri ledger-prune <sha> --write` does that removal from the lab log.
   enum-fix also catches the ml form `enum_type : "enum", values : ,`.
   Other forms of #3225:
   - `X = struct { variants : , }`: the names were lost. They still live in
     trinity specs/tri/*.tri, and `tri variants-restore [--write]` copies
     them back. A name with no source is listed MISSING; never guess it.
   - `name : T  # note,`: the comma is inside the comment (#5968). Write
     `name : T, // note`.
   - A `.tri` dependency entry (`module: math.x`, `functions:`) that the
     port turned into a struct is not a type. Make it a comment.
   - When typecheck refuses a spec, `tri lab-parse --why` prints the actual
     errors and skips the warnings.

   REPLAY TRAP (cross_entropy 206161a96): a number computed by hand from the
   body only proves the body does what it does. Its backward said
   (p - t)/p, off by one from -t/p, and four "real" tests agreed with it.
   For a claim that has a definition independent of the code, test that
   definition: a gradient against a central difference of forward, an
   inverse against round-trip, a sum against its closed form. Also mutate one
   expected value once and make sure lab-exec reports FALSE.
   Placeholder bodies (`return action; // Simple implementation for now`,
   void fns that compute and drop the value) are `stub`: owner decision.
   Native forms the lab proved here: `x as f32` (not `f32(x)`),
   `@max`/`@log` (not `f32.max`), and `usize` for anything assigned `.len`.

   ONLY-T27 (owner HARD RULE 2026-10-05 15:27Z): no new or edited hand-written
   Python/Rust/shell. The scripts/tri_loop/*.py tools already in PR #5084
   stay as they are until the owner decides, and you may still run them. A
   new tri command is written as a .t27 spec, or it is not written at all.

   LEAN COUPLING: making a spec lowerable can turn the `lean` gate red
   (`Rust=true, Lean theorem=false`). The cause is that
   proofs/lean4/.../Completeness.lean is a stale snapshot with no generator
   (#2747); for example exit_codes at 39a285c42. In that case:
   - revert the spec;
   - prune with `--keep <spec>`;
   - leave the Lean snapshot and the 77-entry cap to the owner.

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
suite gate: RATCHET CLEAN and the spec's count down. `enqueue` only drops a
request file: the sha shows in `/latest.json` `queue` after the watcher's next
poll (up to 180 s), so a sha missing 10 s later is NOT lost -- wait one poll
before enqueueing again (2026-10-05: b5a0aede, f32c8a32) -> bless the lower pin
(`t27c suite --ratchet --bless-expectations`, on the lab) -> commit `Refs #N`.

Never mass-reseal. After every commit, check `.gitattributes` for the stray
`.claude/skills/ci-gates/SKILL.md merge=union` line and revert it.

## 3. Checking that a math spec is TRUE, not only compiling (ml wave, 2026-10-05)

A spec that parses and typechecks can still be wrong. In this wave, 8 ml specs
compiled but carried the wrong math: gelu's exact backward divided by sqrt 2
instead of sqrt(2 pi); softmax's backward was only the diagonal; silu ignored
beta; tanh returned NaN for |x| > 44; BCE's sigmoid returned 3.146 at 0.
`tri lab-exec` runs tests on the lab, but each check needs a reference that
is independent of the code under test:

- **gradient**: a central difference of forward,
  `(f(x+h) - f(x-h)) / 2h` with h = 0.001 and tolerance 1e-3, in a fixture fn
  that builds the arrays (`const a : [3]f32 = [x, y, z]; var g : [3]f32 =
  [0.0] ** 3;`). For softmax this means the VJP:
  `p_i (g_i - sum g.p) / T`.
- **value**: a published number (GELU(1) = 0.8413447, tanh-GELU(1) = 0.8411920,
  sigmoid(1) = 0.73105858). Also a symmetry identity: gelu(x) - gelu(-x) = x,
  tanh odd, sigmoid(-x) = 1 - sigmoid(x).
- **saturation**: run at |x| = 50..200. This is what catches overflow NaN.
- **mutation**: put the old bug back once and confirm lab-exec turns FALSE.
  A test that stays green under the mutant proves nothing (REPLAY TRAP).

Zig has no `@tanh` or `@erf`, so write them out:
- stable tanh: `1.0 - 2.0 / (@exp(2.0 * z) + 1.0)`
- stable sigmoid: branch on sign, `e / (1 + e)` with `e = @exp(x)` for x < 0
- erf: Abramowitz and Stegun 7.1.26
- log: use `@log` and clip p to [eps, 1 - eps]. Never use a Taylor
  "log_approx".

Codegen trap (#6549): the Zig backend emits `-(a + b)` as `-a + b`. Keep the
natural form in the spec, comment the affected tests with the issue number, and
let them run FALSE. Never dodge it with a rewrite. The probe that isolates it is
a throwaway `zz_probe_*.t27` that defines only the suspect fn and one failing
test. lab-exec -v prints only the first zig message per group, so keep one
failing assert per probe. Delete the probe file before committing.

### 3a. Restoring a placeholder from its .tri (loss + optimizer waves, ae9d87364, 80a83524b)

- A placeholder that kept only the first parameter and returns void is restorable
  when `trinity/specs/algo/<name>.tri` gives the signature AND the formula. With
  no formula there, leave it as a stub; do not invent one.
- A spec cannot allocate. Where the .tri says `array(n, 0.0)`, take a
  caller-owned `[]f32` buffer and zero it (adagrad `init_state(buffer)`).
- Stateful fixtures: `var params : [1]f32 = [1.0]; var m : [1]f32 = [0.0];`,
  loop `k` steps, return one element. Use an `if (k == 0)` override for a
  gradient that changes after the first step.
- Empty array: `const none : [0]f32 = [];`. `[_]f32{}` is a CODEGEN error.
- The .tri itself can be wrong (sgd nesterov counts the momentum twice). Check
  against the published reference (PyTorch / the paper), follow that, and note
  the difference in the spec.
- Check optimizers with scale-free identities: Adam's and Adagrad's first step
  is lr·sign(g) for any |g|; RMSprop's first step is lr/sqrt(1-decay); a
  constant gradient moves Adam by lr per step.
- RL returns and GAE run backwards: `var t : usize = rewards.len; while (t > 0)
  { t = t - 1; ... }`. Test GAE at lambda 0 (= TD error), lambda 1 (= return
  minus V(s_0)) and one value in between (ppo_critic a00c6bb10).
- A local named `std` breaks the generated Zig ("local constant shadows
  declaration of 'std'"); call it `deviation` (ppo_clip_loss 7c5a5f336).
- The same for Zig's C-ABI primitive type names: a local `c_short` (also
  `c_int`, `c_long`, `c_char`, `isize`...) is CODEGEN "name shadows primitive"
  (lstm_single 33f4237c2). Name fixtures `c_small`, `w_small`.
- A .tri whose `formula:` is blank can still define the function when its
  `description:` is unambiguous ("Lookup token embeddings: W[token_id]" ->
  row copy, embedding_layer cd29bd3b8). Say in the header which line you
  read; if the description could mean two things, leave it NOT CHECKED.
- Two .t27 ports of one .tri (lstm_cell stacked, lstm_single per-gate) are
  restored separately; keep each spec's own weight layout and test it with
  an off-diagonal entry of THAT layout.
- Unary minus on a sum hits codegen bug #6549 (`-(a+b)` -> `-a+b`): write
  `0.0 - (a + b)`.
- Symmetric fixtures hide swaps: with i = f = s(1) an LSTM that swaps the
  input and forget gates still passes. Give every gate (term, argument) a
  different hand value, and test a weight layout with an OFF-diagonal entry
  so the transposed read gives a different answer (lstm_cell, rnn_cell).
- Structs of slices work in fixtures: `LSTMState{ .h = &h_out, .c = &c_out }`
  with `var` arrays for fields typed `[]f32`.
- `tri lab-exec` uses the t27c the lab built LAST, and prints "(lab t27c built
  at <sha>)". If that sha is a foreign branch (claude/t27c-gen-typecheck*),
  every spec may say "gen FAILED"; with a busy queue, big batches lose
  verdicts. Run one spec at a time and read the built-at sha before trusting
  a mass failure (2026-10-05, 47 false "gen FAILED").
- An old "implemented" body can contradict its .tri, not just be a stub:
  gru_cell blended h = (1-z) h_hat + z h_prev, the .tri says (1-z) h_prev +
  z h_hat. Before keeping any formula, diff it against the .tri formula AND
  behaviors, then pin the convention with a test whose swap fails
  (dee114441). A gate the .tri writes over `[h_prev, x]` puts h first.
- Read-only buffers in a struct of slices still need `var` arrays; a slice
  `wh[0..n]` of a var array passes a shorter length into a size predicate
  (gru_cell sizes_ok).

### 2026-10-06 tick notes (sac_critic, sac_actor, ppo_actor, flatten, mlp)
- Stochastic samplers: pass the noise draw in as an input (reparameterization); test the deterministic branch and one fixed-noise draw.
- Discrete action stored as f32: find its index with `(k as f32) == action[0]` in a `while` over k.
- `size_ok`/`state_ok`: add a negative case on each side (short AND long), or `==` -> `>=` survives.
- Open-slice mutation `x[i..]` hits codegen #6371 (emitted as `x[i]`): note it, do not report again.
- Layered forward without a scratch buffer: recompute each hidden unit inside the output loop (mlp 8b2b6baaa); test with one negative pre-activation per layer so ReLU is exercised.
- Run `git log origin/master -- <spec>` BEFORE editing, not after (mlp: checked late, both commits were in the branch).
- When the .tri formula and its own behavior/equation disagree, the real layout or the true derivative wins; write the conflict into the spec header (avgpool index, batchnorm grad_gamma, encoder_block residual).
- A mutation that survives because it is EQUIVALENT (avgpool kernel>=1, implied by stride>=1 and stride<=kernel) is documented in a comment, not "fixed" with a contrived test.
- A .tri that names undefined helpers (seq2seq encoder/decoder_step, encoder_block W1/activation): delete the port's invented or identity bodies, keep the types, check only the constraints, mark the rest NOT CHECKED. Removing an invented body is a deletion, not an invention.
- Recurrent passes without a scratch buffer: read h_prev from the previous output slot (zero at t=0) and update the cell in place, since c'[k] reads only c[k] (bilstm 55ed86a33). Use different parameters per direction and asymmetric inputs, so "not reversed" and "shared params" each fail.

### 2026-10-06 tick notes (layernorm .. attention_mechanism)
- `@sin`, `@cos`, `@exp`, `@log`, `@sqrt` all lower and run; `10000^x` is `@exp(@log(10000.0) * x)` (positional_encoding 9897ead9e).
- A PRNG inside a .tri (dropout) becomes an input slice `uniform[]`; test inference and one fixed draw (dropout fbd9a7ed8). With no random source at all (attention dropout), it is NOT CHECKED and forward is the inference path.
- Per-channel / per-head loops need fixtures with 2+ channels or heads, otherwise the "plane offset dropped" mutation survives (maxpool2d aa5b03528, attention e21cfd91f).
- Window bounds that mix padding and dilation need one padded+dilated test; a dilation-free bounds check survives everything else.
- A -inf mask is written as "masked entry gets weight 0" with the max taken over unmasked entries; native, no infinity literal (attention).
- Dropping the softmax max-shift is an EQUIVALENT mutation (shift-invariance): comment it, do not chase it.
- Gradients: compute expected coded sums with a throwaway float64 reference AND check that reference against central finite differences before trusting it; the reference is scratch, never committed (Only-t27 rule).
- A projection weight the .tri names but never defines (W_o, Wq/Wk/Wv): stop before it and say so in the header (attention: output = head concat; multi_head_attention 04b41636e: forward removed, constraints only).

### 2026-10-06 tick notes (specs/tri/sort family, e4f18a6c2)
- A .tri `allocator` parameter becomes caller-provided buffers (`output`, `scratch`, `counts`) plus a `buffers_ok` check; do not invent an allocator API.
- Array `==` is a gen-zig error ("operator == not allowed for type '[5]i64'"): compare through a coded sum Σ x[k]*(k+1), and add `* 1000 + code(input)` when the input must stay untouched.
- A sorted result hides the algorithm's choices (gap sequence, digit base, Lomuto tie side, min_run). Expose the step (`gap_pass`, `digit`, `digit_pass`, `partition`, `build_run`) and test it, or a mutation of the choice survives. A fixture with ties against the pivot is what kills `<` -> `<=`.
- The port and the .tri disagree (shell: Knuth vs Shell's n/2 gaps): the .tri wins; name the conflict in the header.
- usize loops: guard `p - 1` with `if (p > low)`; stop radix with `if (top / place < BASE) break;` before `place * BASE` overflows.
- Recursion (merge sort_range, quick sort_range) lowers and runs.
- Equivalent mutants seen here: an extra sift_down on a valid heap, `sort_range(lo, lo)`, `> x` -> `>= x + 1` on ints. Do not pick integer-equivalent mutations.
- Lab enqueue: the railway CLI now prints a config-migration notice; if the output lacks the `queued <sha>` line, re-run the enqueue.

### 2026-10-06 tick notes (KMP, Boyer-Moore, disjoint_set, polynomial, bezier)
- String literals pass as `[]const u8` and index to bytes; `text[i] as usize` indexes a `[256]usize` table. A struct can hold slices and a `[256]usize` array; `Type{ .a = x, .b = y }` returns it by value.
- Pointer params `ds: *DisjointSet` with `union(&ds, 0, 1)` lower and run; a fn named `union` is fine.
- Search functions returning `[]usize` become `out: []usize` + returned count; write only while `count < out.len`, and test a too-small out.
- Symmetric fixtures hide swapped operands: palindromic y control points let a reversed lerp survive (bezier). Give each fixture one asymmetric case.
- A `*_ok` length check needs a positive case at the exact length AND a negative case beside it, or `len - 1` -> `len` survives.
- Hand arithmetic in comments was wrong again (code(0,1,2,3) is 20, not 30): take every expected value from the /tmp reference, never from the head.
- A guard that the loop condition already implies (`m > text.len` before `while (pos + m <= text.len)`) is an equivalent mutant: delete the guard instead of testing it.
- A module-level backing buffer in place of an allocator is a defect (two inits alias): use caller buffers.

### 2026-10-06 tick notes (bellman_ford, graph_bfs, graph_dfs, topological_sort)
- A .tri that names `Graph` without defining it borrows the one .tri that does (tri_graph_bfs.tri adjacency lists); say so in the header. Each spec keeps its own local copy (no cross-spec import yet).
- Nested rows `var rows : [3][]usize = [&r0, &r1, &r2];` pass Zig but FAIL gen-verilog (W469: local array of slices not lowered) -> RATCHET FAIL. Use one flat buffer: `adj[v * width + k]` with a `width` field; test the row offset with two non-empty rows and the flat length in buffers_ok. lab-exec runs Zig only: a backend failure shows up only in the lab suite.
- A slice of a field slice is passed WITHOUT `&` (`code(r.distance[0..5])`); with `&` it is CODEGEN "found *[5]usize".
- BFS/Kahn: the caller's order buffer doubles as the FIFO queue (head/tail), so no queue buffer; DFS needs stack + a per-vertex edge cursor (UNSEEN = not visited).
- Mutating away a call whose bool result is unused gives CODEGEN, not a test: not a valid mutant.
- `<=` vs `<` on positions in is_valid is only told apart by a self-loop; add a self-loop fixture.
