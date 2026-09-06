# NOW -- Two tools that advised against their own bench (2026-09-06)

## Two tools that advised against their own bench (Closes #3371)

- `tri fpga program-flash` advertised `--enable-quad` with `--part` defaulting to the exact board whose SSOT records the flag ABORTS the command (measured, W396/E4). Zero guards near the flag. `quad_refused()` now refuses both flags on that part, quoting the SSOT.
- Worse than the flag: on `--spi-buswidth 1` -- the SSOT-canonical path all three internal callers hardcode -- it printed 'ensure the flash QE bit matches' about a bit the SSOT says does not exist on this part. Now silent on x1.
- `tri gates dead` printed unconditionally that a suppressed workflow's last step is 'forbidden by this repository's own ruleset'. #3324 removed that step; the only `git push` left in `brain-seal-refresh.yml` is inside the comment explaining the removal. Three further assertions in gates.rs stated the old cause.
- `# tri:cause-removed <reason>` gives the classifier its fourth state; the row is listed with its reason and the blanket footnote is scoped to the rows that explain nothing -- 3 of 4 today.
