# NOW -- tri gates tests: a dead test and a phantom test cancel in every total (2026-09-05)

## tri gates tests: a dead test and a phantom test cancel in every total (Refs #3191)

- commit f7c1ff5 shipped one test that stopped running and one registered twice; both totals stayed unchanged, which is why the check that counted attributes against functions read a clean 11 against 11
- new tri gates tests --gate pairs them PER FUNCTION; measured across 57 files of cli/: exactly 1 of each, both real, 3 assert-bearing fixtures correctly silent; positive control is exit 1 on f7c1ff5 and exit 0 on the repair
- tri red asks for the last success on EVERY red row now, not only truncated ones, so a row can say it was never green on the branch -- 44 of the 50 in trinity-fpga had never once succeeded
- tri gates mutate counted every `# mutant-equivalent:` marker as in scope while its numerator only counted claims whose line was a real site in the direction run; all 8 markers in tools/ bind to comparisons or assignments and the default operator only mutates `return 1..4`, so a default run printed "each mutant survived" with zero mutants built. Claims are now partitioned and the untested ones named
