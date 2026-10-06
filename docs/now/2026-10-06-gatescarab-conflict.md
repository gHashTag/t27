# NOW -- GateScarab declared twice: rename the SR-03 port's type (2026-10-06)

- Closes #6611. Master's Corpus ratchet is red with `+ GateScarab NEW conflict` after #6238 merged alongside the SR-04 port.
- `specs/port/trios/crates/trios-scarab-types/rings/SR-03/src/lib.t27`: `GateScarab` -> `GateScarabFour` (type and its functions). The rendered strings keep the Rust original's text.
- No ledger move, no seal touched.
- Same batch, second red: `duplicate-bodies` (`a body written under put is now copied [2] time(s)`). The 6-line byte-copy helper `put` is in two independent port modules, `ls.t27` and `gen_w384_lean.t27`. Each port module is self-contained, so the copy is deliberate. It is recorded with `python3 tools/dupe_scan.py --bless`: +1 line in `tools/duplicate_bodies_baseline.txt`, tool-written.
