# NOW -- tri now add could only write the autoclosing form (2026-08-24)

## tri now add could only write the autoclosing form (Refs #141)

- Refs #141. `tri now add` had one issue flag, `--closes N`, which stamps `(Closes #N)` -- and GitHub ACTS on that word. LOOP-RULES R11 bans autoclosing a long-lived tracking issue, so an entry that had to cite one could autoclose it (banned), hand-edit the generated file (the exact drift this command exists to prevent), or cite nothing. All three were taken at least once
- `--refs N` stamps `(Refs #N)` instead: the citation without the side effect. It `conflicts_with = "closes"`, because an entry carrying both suffixes says two different things about the same issue and clap can refuse that for free
- The suffix is now built by `issue_suffix()`, separate from the writer, so it is testable without a filesystem. The defect this guards against is a one-word difference between two strings that both look right in a diff -- `Closes` and `Refs` differ by nothing a reviewer reliably notices, and only one of them closes an issue
- This entry is the flag first use, and it cites issue 141 -- the now-coordination anchor, which must never be autoclosed. Before the flag, writing this very entry correctly was not possible with the command
