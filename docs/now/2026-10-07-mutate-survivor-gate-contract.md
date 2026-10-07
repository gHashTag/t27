# NOW -- when `tri mutate spec` should fail on survivors, written in t27 before any Rust (2026-10-07)

## specs/tri/mutate/survivors.t27 (Refs #7303)

- Slice 1 of #7303. Today `tri mutate spec` exits 0 when mutants survive: on the lab, `specs/tri/mutate/lab.t27` printed `48 of 52 killed ... 4 survived` and exited 0. A script can gate on survivors only by grepping the text `SURVIVED`.
- This spec states the gate, and it was written before any Rust. Slice 2 (`docs/now/2026-10-07-mutate-survivor-gate-flags.md`, same branch) adds `--fail-on-survived` and `--accepted FILE` to `cli/tri/src/mutate.rs`, plus a test there that evaluates every assert row here against the Rust copy.
- The accepted file lists mutants the caller has judged equivalent, one per line, in the form the tool already prints them under SURVIVED or HUNG: `file:line [kind]`. The spec sees only four counts: survivors, hung mutants, of those the ones the file names, and file lines whose mutant ran and was killed.
- 5 functions:
  - `gate_on`: the flag or an accepted file turns the gate on. Without either, the exit code is today's.
  - `not_killed`: survivors plus hung mutants. A hang proves that no check noticed the mutant, so it is not a kill. Unviable mutants are counted nowhere, as in cargo-mutants.
  - `unaccepted`: not-killed mutants the file does not name. A count of more hits than mutants cannot be right, so the gate treats every one as unaccepted (it fails closed).
  - `survivor_exit`: with the gate on, an unaccepted survivor or hung mutant fails the run. So does an accepted line whose mutant is now killed, so the file cannot keep a claim that is no longer true.
  - `mutate_exit`: a red unmutated spec, or a mutant that could not run, fails first with the tool's 1, as today. Survivors are judged only on a whole run.
- Exit codes: 0 = pass, 1 = no verdict (the tool's error exit, as today), 2 = the gate failed. This follows cargo-mutants, whose documented codes are 2 for mutants the tests missed, 3 for timeouts and 4 for a failing baseline. Stryker exits 1 below its break threshold, the same code as a crash; a caller of `tri mutate spec` must be able to tell a test gap from a broken run. `lab_exit` in `specs/tri/mutate/lab.t27` passes the 2 through.
- Revised the same day, before any Rust: the first form failed the gate with the tool's 1 and did not count hung mutants. Both were weak points found by comparing with cargo-mutants.
- Results:
  - 5 functions, 3 constants, 10 tests and 1 invariant.
  - `zig test` on the `gen` output passes all 10, and `t27c test-report` counts 0 vacuous passes of 10.
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0. `cc -fsyntax-only` on the gen-c output gives 0 errors, the gen-rust output compiles with `rustc --test`, and `iverilog -g2012` accepts the gen-verilog output.
  - Negative control for the invariant: with `EXIT_SURVIVED` set to 1, `zig test` fails at compile time.
- Mutation testing:
  - `tri mutate spec` on the lab, whole file: 20 of 20 mutants killed, all by a failing test, 0 survived, 0 hung, 0 unviable.
  - 40 mutants made by hand cover each constant, the `||`, the `+` in `not_killed`, each guard's comparison and return value, each dropped guard, the subtraction and each final return. All 40 are killed, each by a failing assertion (read by hand for the 7 that drop a parameter's only use, so none is a compile error). The unmutated copy passes in the same harness.
