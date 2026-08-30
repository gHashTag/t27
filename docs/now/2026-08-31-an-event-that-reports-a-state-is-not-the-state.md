# NOW -- An event that reports a state is not the state (2026-08-31)

## An event that reports a state is not the state

- an artifact republish notification twice cost a full re-read of a 2000-line file and found nothing lost: once it was my own publish, once it named a version my publish had already superseded
- the cheap check is two commands -- compare the version id the fetch returns against the one the notification named, and diff the body against your own copy
- when they DO differ the full read is right, and the heading count both ways is what proves nothing was silently dropped
- the section nearly went unwritten: the first append used an UNQUOTED heredoc, the shell ran its backticks, nothing was written and git status showed a clean tree
