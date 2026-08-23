# NOW -- Seventeen tri subcommands were unreachable, and a lost skill section came back (2026-08-23)

## Seventeen tri subcommands were unreachable, and a lost skill section came back (Closes #2559)

- scripts/tri forwarded every unhandled subcommand to t27c, so the 17 implemented in the Rust cli/tri binary could not be run by their documented names. now was one: the now-sync gate prints './scripts/tri now add ...' as its own remediation and that answered 'unrecognized subcommand'.
- The front door now asks each binary what it implements instead of carrying a baked-in list; only commands t27c lacks are rerouted, and all 8 shared names produce byte-identical output before and after.
- SKILL.md section 63 was overwritten by section 64 in an unrelated commit and had been missing for several sessions; restored from git and section 74 records how a numbering gap tells you whether anything was actually lost.
