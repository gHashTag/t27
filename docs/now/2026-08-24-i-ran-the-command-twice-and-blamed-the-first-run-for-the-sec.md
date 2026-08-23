# NOW -- I ran the command twice and blamed the first run for the second's answer (2026-08-24)

## I ran the command twice and blamed the first run for the second's answer (Closes #2161)

- Saw VERDICT: WAIT and rc=0 and concluded the exit-code fix was broken. It was not: I ran the command twice, once for output and once for the exit code, and in the second between them the last check completed.
- Two invocations of a command that reads a live external system are two measurements of two different states. Written on adjacent lines they read as one. Capture both from one call: out=$(cmd 2>&1); rc=$?
