# NOW -- The gate's blindness was its bypass (2026-09-05)

## The gate's blindness was its bypass (Refs #3338)

- A required context whose whole body was an `echo` asserted nothing -- and also exempted
  every bot pull request from a requirement it could not enforce. Only the first was ever
  noticed, because an exemption produced by blindness looks exactly like no exemption
  being needed.
- Two sibling required contexts had been given a trusted-bot no-op in June, with the reason
  recorded: a SKIPPED required check never satisfies branch protection, so the bypass must
  be a step that PASSES. The third never got it, because it did not appear to need it.
- Measured: IS_BOT occurrences and conditional steps were 6/5, 3/2 and **0/0**. Eight open
  Dependabot pull requests: the three opened after the change were RED on that one context
  and green on the other three; the five opened before were green only because their run
  predated it.
- Proven end to end after the port: all eight report `check = SUCCESS` and the three that
  were red are mergeable.
- Cost a rewrite: the section was first written into an UNQUOTED heredoc, so every backtick
  ran as a command substitution and the code spans were eaten -- one of them printed
  `command not found: check`. `<<'MD'`, never `<<MD`. This is in my notes already.
