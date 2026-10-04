# NOW -- C-backend tests ask cc which flags it takes (2026-10-04)

## Route every C syntax check through common::cc_check (Closes #5907, Part of #5906)

### What was read

- `Bootstrap Test Ratchet` on master 5432dedf7: 24 tests newly failing against the baseline. 23 of them are C-backend tests whose `cc` (gcc on ubuntu-latest) refused `-ferror-limit=0`, a clang-only flag hand-copied into 15 files. The 24th, `icarus_lowerable::corpus_classifier_matches_lean_completeness`, is #5910 and is not C.
- `bootstrap/tests/common/mod.rs::get_cc_args()` (1acab8481) was meant to fix this but no test used it, it did not compile, and its probe wrote `-.o` into the working directory.

### What changed

- `common/mod.rs` rewritten: `get_cc_args()` probes `-ferror-limit=0`, then `-fmax-errors=0`, and accepts a flag only on exit 0 with empty stderr (Apple clang takes `-fmax-errors=0` with a warning and exit 0). `cc_check()` panics when `cc` exits non-zero without naming a `file:line:col` error, which is `cc` refusing its own command line. `error_count()` also counts `fatal error:` lines.
- 17 `c_*.rs` files use `mod common;` and `common::cc_check`. Under gcc the count-shaped ones (9 files) used to count 0 errors without compiling anything; the contains-shaped ones went red on the word "error".
- `c_static_array_params.rs` compiles with `-c` (gcc reports a short `[static N]` argument only past the front end) and asks `cc` which warning a short argument gets, using hand-written C, instead of demanding clang's `-Warray-bounds`.

### What was verified

- 21 affected targets (489 tests), gcc 15 as `cc`: before 464 passed / 25 failed, after 488 / 1. Apple clang: 487 / 2 before, 488 / 1 after. Homebrew LLVM clang 21: all 18 C targets pass. Of the 25, 23 are the C tests; `corpus_unresolved` was fixed on master by #5909 meanwhile; the one failure left is #5910.
- Negative control (not committed): a generated header plus `int x = ;` must count >= 1 error. Before, gcc counted 0. After: gcc, Apple clang and LLVM 21 all count it.

### Not verified

- The full `cargo test -p t27c --no-fail-fast` was not run locally under gcc: the disk had under 3 GiB free. CI's ratchet run on this PR is that comparison.
- CI's gcc is older than gcc 15. Whether it warns on a short `[static 4]` argument at `-O0 -c` is read from the PR run, not assumed.
