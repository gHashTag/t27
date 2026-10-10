# NOW -- the merger-gate spec states the rule the merger runs (2026-10-07)

## specs/queen/merger_gate.t27 (Closes #7300)

- The spec's header said the runtime mirrors its functions "from t27c gen-js output". gen-js lowers no function bodies (it prints `fn red_check_passes was not emitted`), and the merger, `.github/workflows/auto-merge-ready-prs.yml`, is bash and jq that never reads the spec. The header now names `tools/bees/merger_gate_selftest.py` as the place where the two meet, and each new test names the scenario whose verdict it asserts.
- One verdict was the opposite of the merger's. The spec asserted `gate_open(false, true) == false`: an unreadable ruleset kept the gate shut even with every check green. The merger opens it, and its self-test pins that ("ruleset unreadable, every check green" is ready). It fails closed per check instead: with no ruleset every check counts as required, so no red is discountable ("ruleset unreadable: a discounted red check still blocks"). The spec now says the same through `counts_as_required`. If the stricter rule is wanted, it changes in the workflow first.
- Two rules of the merger were missing:
  - a check that has not concluded blocks (`check_passes`);
  - zero posted checks keep the gate shut ("zero checks is not zero failures").
- `gate_open` now takes the merger's three counts: posted, running and blocking.
- The spec has 5 functions (was 3) and 22 tests (was 13). `zig test` on the `gen` output passes all 22, and `t27c test-report` shows 0 vacuous passes of 22.
- `tri mutate spec` on the Railway lab killed 16 of 16 mutants, all by a failing test.
- 12 mutants made by hand cover what the tool does not flip: each `!`, the literal `true` passed for `concluded`, the bounds `posted > 0`, `running == 0` and `blocking == 0`, and both new guards dropped. Each was killed by the test written for it, and the unmutated copy passes in the same harness.
- The self-test has no scenario with zero posted checks. That is the #5777 lane's file and is noted on #7300, not edited here.
