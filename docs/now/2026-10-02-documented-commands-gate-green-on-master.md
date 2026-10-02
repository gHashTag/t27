# NOW -- Documented commands gate green on master (2026-10-02)

## Read only command syntax as a sibling-script invocation (Refs #5497)

- `Documented t27c subcommands exist` fails on master: its standalone matcher read every hyphenated `tri-` word as a missing `scripts/tri-<name>`, which gave 1004 mentions under 113 names. They are skill names (`tri-pipeline`), agent ids (`tri-doctor`), sibling repositories (`../tri-net/src/lib.rs`) and the adjective `tri-valued`. None of them is a command.
- Take the repair from #5473. A standalone name counts only in command position: at the start of a line, optionally after `$ ` or `> `, or after a backtick, and followed by an option or the end of the line. The scan reads the tracked files from `git ls-files` instead of walking the working tree. Explicit `scripts/tri-*` paths are matched as before.
- On master d5f22155 the check now passes: 11 sibling references read, 5 excused as declared, 141 dead `tri` mentions at the recorded ceiling. `--self-check` passes, including 11 new sibling controls.
