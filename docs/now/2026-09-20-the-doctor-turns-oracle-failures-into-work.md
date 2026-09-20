# NOW -- The doctor turns what the oracle measured into work the swarm can take (2026-09-20)

## 388 broken specs were a list nobody could act on; they are 27 clusters, and each is a task (Closes #4299)

- A full oracle run on master today: 425 PASS, 388 NOCOMPILE, 23 TESTFAIL, 1 TIMEOUT over 837 specs. The nightly has printed that reading for days and nothing consumed it, because a list of 388 failures is not work anyone can pick up.
- They cluster by the error the compiler printed: 173 specs fail with `use of undeclared identifier`, 23 abort under test, 22 with `expected 'X', found 'X'`, 14 with `expected 'X' after field`, 9 with `expected type expression`. 27 clusters of three or more specs in all.
- `tools/queen/feed_defects.py` opens one issue per cluster: the error, the command that reproduces it WITH its real output, every spec it affects, and criteria the tool measured rather than typed. It refuses a cluster that already has an issue, because a fuel line that duplicates its own work buries the swarm.
- The normalisation is the whole trick and is covered by a self-test: two different identifiers are one defect, two different errors are not, and `PASS` and `NOGEN` are not defects at all.
- The command in the issue is quoted as `t27c`, never as the path on the machine that measured it - that mistake put `/Users/playom/t27/target/release/t27c` into 137 open issues and made every one of them unrunnable in a container.
- Weekly, not nightly: the oracle is a full corpus build and a cluster does not change between Tuesdays. `oracle-nightly.yml` keeps the daily reading.
