# NOW -- The compiler stopped naming one machine (2026-08-29)

## The compiler stopped naming one machine (Refs #2804)

- four sites in bootstrap/src/service.rs hardcoded a developer home; the openXC7 root now comes from T27_OPENXC7 with NO default, because a default that is one machine's path is the literal wearing a fallback
- the existence loop in run_silicon checked three of the five paths the run needs; xr and venv were used twenty lines later untested, and their absence surfaced as sixty characters of stderr naming neither
- constids missing on one side now names WHICH side and both paths, held to the standard of the arm directly above it
- the devhome ratchet built last iteration caught its own first repair: FAIL fixed bootstrap/src/service.rs was 4. Baseline 33/51 -> 32/47
