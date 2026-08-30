# NOW -- The measurement should not wait for the decision (2026-08-31)

## `tri vsim funnel`: how far each spec gets when its Verilog is RUN (Refs #2987)

- `t27c suite --icarus-simulate` walks `repo/specs/scratch`, gone since #2283, so the one arm that can catch a defect whose nature is that it COMPILES has had no targets; repairing that is half a line and half a decision about 260 stale baselines, and the decision is the owner's
- the measurement is not: `tri vsim funnel` walks the corpus through `t27c icarus-simulate`, one TMPDIR per spec because the compiler keys its scratch file on the BASENAME alone and this corpus shares stems
- measured at 487003fb9 over 650 specs: 69 refused by gen / 410 rejected by iverilog / 0 vvp / **10 report failures right now** / 106 produce a verdict / **55 exit 0 having said nothing** / 0 timeouts -- 650 of 650, and the command prints the mismatch if the parts ever stop summing
- `silent` is a separate row on purpose: `run_icarus_simulate` bails only on a FAILED line, so "checked everything and passed" and "checked nothing" are the same exit code
- it REFUSES when `iverilog` or `t27c` is absent rather than reporting a table of zeros, and asks the OS rather than matching an error message (ci-gates 426); it reports and never gates -- which of the 10 are compiler defects and which are spec defects is not a walker's question
