# NOW -- when `tri mutate spec` should fail on survivors, written in t27 before any Rust (2026-10-07)

## specs/tri/mutate/survivors.t27 (Refs #7303)

- Slice 1 of #7303. Today `tri mutate spec` exits 0 when mutants survive: on the lab, `specs/tri/mutate/lab.t27` printed `48 of 52 killed ... 4 survived` and exited 0. A script can gate on survivors only by grepping the text `SURVIVED`.
- This spec states the gate. The Rust does not exist yet, and the header says so. Slice 2 adds `--fail-on-survived` and `--accepted FILE` to `cli/tri/src/mutate.rs`, plus a test there that asserts each scenario here row by row.
- The accepted file lists survivors the caller has judged equivalent, one per line, in the form the tool already prints them: `file:line [kind]`. The spec sees only three counts: survivors, survivors the file names, and file lines whose mutant ran and was killed.
- 4 functions:
  - `gate_on`: the flag or an accepted file turns the gate on. Without either, the exit code is today's.
  - `unaccepted`: survivors the file does not name. A count of more hits than survivors cannot be right, so the gate treats every survivor as unaccepted (it fails closed).
  - `survivor_exit`: with the gate on, an unaccepted survivor fails the run. So does an accepted line whose mutant is now killed, so the file cannot keep a claim that is no longer true.
  - `mutate_exit`: a red unmutated spec, or a mutant that could not run, fails first, as today. Survivors are judged only on a whole run.
- The gate adds no exit code. Failure is the tool's existing 1, so `lab_exit` in `specs/tri/mutate/lab.t27` reads it as a failed tool.
- Results:
  - 4 functions, 2 constants, 9 tests and 1 invariant.
  - `zig test` on the `gen` output passes all 9, and `t27c test-report` counts 0 vacuous passes of 9.
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0. `cc -fsyntax-only` on the gen-c output gives 0 errors, the gen-rust output compiles with `rustc --test`, and `iverilog -g2012` accepts the gen-verilog output.
  - Negative control for the invariant: with `EXIT_FAILED` set to 2, `zig test` fails at compile time.
- Mutation testing:
  - `tri mutate spec` on the lab, whole file: 18 of 18 mutants killed, all by a failing test, 0 survived.
  - 26 mutants made by hand cover each constant, the `||`, each guard's comparison and return value, each dropped guard, the subtraction and each final return. All 26 are killed. The unmutated copy passes in the same harness.
