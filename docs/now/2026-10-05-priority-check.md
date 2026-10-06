# NOW -- tri priority --check and the stdlib-shadow guard (2026-10-05)

## Label anomalies, and no loop tool named like a stdlib module (Closes #6378)

- `tri priority --check` prints each label anomaly and exits 1 when there is one: more criticals than CRITICAL_CAP, labels of two levels on one issue, a labelled issue with an open blocker. On gHashTag/t27 today: 0 anomalies in 864 open issues.
- The loop-tools-gate priority step now fails when a file in `scripts/tri_loop` has a stdlib module's name; a planted `queue.py` is caught. The first name of `tri priority` was `queue.py`, and it broke `tri stranded` in CI.
