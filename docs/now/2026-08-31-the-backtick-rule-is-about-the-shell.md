# NOW -- The backtick rule is about the shell, not about heredocs (2026-08-31)

## Correcting the previous commit message, and closing the class (Refs #2987)

- the commit message for `tri vsim funnel` was passed inline to `git commit -m "..."`; double quotes do not stop command substitution, so the span naming the `silent` row was executed and removed, leaving `*  is its own row.` and a `command not found` on stderr
- the message stands with that hole: the commit is pushed and this repository forbids force-pushing, so the correction lives here and in the pull request body rather than in a rewritten history
- the missing word is `silent` -- the row for a spec that ran, exited 0 and printed no verdict line at all
- ci-gates 418 named the heredoc; this went through a different door, so 427 names the class instead: prose containing a backtick never reaches the shell as an argument -- `git commit -F -` with a QUOTED heredoc, or a file
- third occurrence of this class in this repository's log, which by its own rule means the cure was wrong rather than that this is another case
