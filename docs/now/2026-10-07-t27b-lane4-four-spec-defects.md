# NOW -- four t27b blockers that were spec defects: dqn, hex, gen_image, args (2026-10-07)

## specs rewritten from their sources, with real tests (Closes #7372)

- Four specs were `blocked` on the t27b ledger because of something in the spec, and the reference passed each of them on tests that checked nothing.
  - `ml/rl/dqn.t27`: its tests called `default_input()` and `valid_input()`, which are declared nowhere.
  - `tri/crypto/hex.t27`: `encode` and `decode` returned void, and their tests compared that void result with `undefined`.
  - `port/trinity/src/tri/gen_image.t27`: its three fns had the body `undefined;` over `*std.mem.Allocator`.
  - `tri/utils/args.t27`: one `_ = allocator;` fn, and one test, `then true`.
- dqn is the mutation-checked restoration from `specs/algo/dqn.tri`, from PR #5084 (branch `loop/auto-2026-09-29`), unchanged.
- hex follows trinity `src/tri/gen_hex.zig`. `encode` and `decode` write into a caller buffer and return the byte count. They return `NONE` for a short buffer, an odd length or a non-hex char.
  - Tests: the Zig's `deadbeef` vectors, RFC 4648 BASE16, all 256 byte values in both cases, 11 refused inputs, and the short-buffer cases.
- gen_image follows trinity `src/tri/gen_image.zig`. `Image_init` clears the first width * height pixels of a caller buffer to black. `Image_setPixel` writes `y * width + x` inside the bounds and ignores the call outside them. `deinit` has nothing left to free, so it is gone.
- args states the rule by which the Zig parser sorts each argv word: `--`, long option, `=` split, short option, short group, positional, and when an option takes the next word as its value.
  - It does not port the Zig's lone-`-` defect. The Zig's first pass counts `-` as an option, and its second pass stores it as a positional, one past the array.
- Hand mutants, run with `t27c test-report` on the Railway t27b lab:
  - hex: 10 of 10 killed.
  - args: 11 of 11 killed, after a test for a 3-char short group was added for the one that survived.
  - gen_image: 6 of 6 non-equivalent mutants killed.
- t27b `test --check` on the lab:
  - hex: 14 of 14 pass.
  - args: 5 of 5 pass.
  - gen_image: 7 of 7 pass, with 13 runtime asserts.
  - dqn: now stops at `ExprCall(@max)` (line 82) and `@abs`, which are backend work in the same lane.
- Ledger: hex, args and gen_image go from `blocked` to `pass`, and dqn's blocker becomes `ExprCall(@max)`. Pass goes 530 -> 533, not-pass 28 -> 25, and `max_not_pass` 28 -> 25.
- Resealed on the lab with t27c built from c6517ff. The seals are `crypto_TriHex`/`TriHex`, `rl_Dqn`/`Dqn`, `utils_TriArgs` and the two `[]const u8` seals that name args.t27, plus a new seal for gen_image.
