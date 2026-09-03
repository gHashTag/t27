# NOW -- Zero specs is not zero acceptance (2026-09-04)

## A run that produced no numbers must not print the format of a measurement

- `t27c corpus` gains an UNRESOLVED channel (#3025): a tool that could not be
  spawned, a capture file that could not be written, a child killed by a signal
  and a timeout are four distinct non-verdicts, and the first three refuse the
  run with exit 2 instead of falling to a column.
- The reason this was needed: `run_timed`'s failure to create a capture file met
  `== Some(0)` at the call site identically to a compile error. The ENOSPC run
  that opened the issue reported a corpus-wide collapse in acceptance and exited
  green.
- A refusal prints no percentages, writes no `--per-spec` table, and its JSON
  carries no acceptance key at all -- not `verilog_build: 0`, which reads as a
  measurement of zero rather than as the absence of one.
- A timeout alone does NOT refuse: it is a lower bound that a re-run on an idle
  machine can improve, unlike an absent tool or a full volume.
- Second door, found by reproducing rather than reading: a corpus over ZERO
  specs printed `{"specs":0,"verilog_build":0,...}` and exited 0. A mistyped
  `--specs-dir` reached it identically, because the walk opens the tree with
  `read_dir(..).else { continue }`. Both now refuse with `refused: no_specs`.
- Both halves are mutation-checked. Deleting the empty-population guard, and
  keeping it but re-adding one acceptance key to the refusal, each kill the new
  end-to-end test and move no other test in the file.
