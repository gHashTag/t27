# NOW -- One command asks every census whether it counted what it speaks about (2026-08-30)

## One command asks every census whether it counted what it speaks about (Refs #2864)

- Three defects in three passes had one shape: a command narrowed the set it spoke about and the narrowing was invisible, because the only number on screen was the command's own. Each was found by counting the population a DIFFERENT way and subtracting -- and each by accident.
- New `tri census audit` runs that subtraction for every census at once. It runs each census, reads the population off the line the census prints, and compares it with a counter written from scratch in the new file. A counter calling the census's own helper would agree by construction.
- Mutation-checked against the real history: restoring `find("forall ")` gives 922 against 923; restoring the hardcoded crate list gives 133 against 137. Both are the exact numbers of the defects already fixed, so the audit would have caught both.
- A census that stops printing its population FAILS the audit rather than leaving it silently -- proven by removing the line from mods orphan. A census that cannot run is reported as that, not as a disagreement.
- Rows carry their own hardness. `unparsed report` says 'specs TRACKED' while the counter walks the disk, so an untracked spec makes them differ and neither is wrong; that row reports and does not fail, with the reason printed beside it.
- A fourth row for `seals hollow` was built and REMOVED: its counter tested json text for spec_path while the census parses the same field, so planting a seal moved BOTH numbers to 1314 and the row stayed green. No input makes them disagree, and a control that cannot fail is not a control.
- Self-criticism: `rustfmt cli/tri/src/main.rs` follows the mod graph and reformatted five files I never touched -- the trap I have written down for cargo fmt, met through a different door. Reverted; the diff is main.rs +7 lines.
