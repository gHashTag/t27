# NOW -- Sort a debt list by what the file is for (2026-08-29)

## Sort a debt list by what the file is for (Refs #2804)

- an example is the worst place for a machine-specific path: it is the one kind of file whose purpose is to be copied
- set -e aborts on a failing command substitution before the guard clause below it runs -- my message never printed, and only running it outside a repository showed that
- thirty entries, three kinds: configuration to fix, records not to rewrite, experiments to leave. Only a reader can classify them, so it is written in the baseline
