# NOW -- tri gates tests: a dead test and a phantom test cancel in every total (2026-09-05)

## tri gates tests: a dead test and a phantom test cancel in every total (Refs #3191)

- commit f7c1ff5 shipped one test that stopped running and one registered twice; both totals stayed unchanged, which is why the check that counted attributes against functions read a clean 11 against 11
- new tri gates tests --gate pairs them PER FUNCTION; measured across 57 files of cli/: exactly 1 of each, both real, 3 assert-bearing fixtures correctly silent; positive control is exit 1 on f7c1ff5 and exit 0 on the repair
- tri red asks for the last success on EVERY red row now, not only truncated ones, so a row can say it was never green on the branch -- 44 of the 50 in trinity-fpga had never once succeeded
