# NOW -- The unused instrument was red from the day it landed (2026-08-24)

## The unused instrument was red from the day it landed (Closes #2161)

- Ran tri audit for the first time. It fails on master with 'FAIL lessons — 0 lessons, highest 0, no anomalies': zero lessons, no anomalies, and a failure, on one line.
- git log --all -S shows the **N. Title.** lesson format has never existed in the file the counter reads. This repository keeps theorems — 914 headings, counted correctly by the same command. The lesson ledger belongs to a wave loop elsewhere and the command arrived carrying it, so zero is the right answer.
- And the verdict was never the checker's: under set -o pipefail an empty grep exits 1, the pipeline inherits it, and the awk block's own exit 0 never reaches the caller. The exit code belonged to the search, not to the answer.
- Absence and anomaly no longer share a verdict. Verified both ways: two lessons planted at 1 and 3 give 'gap: 1 -> 3' and exit 1; restored, exit 0. tri audit now passes.
