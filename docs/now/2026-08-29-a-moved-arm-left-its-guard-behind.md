# NOW -- A moved arm left its guard behind (2026-08-29)

## A moved arm left its guard behind (Refs #2754)

- parse-complete --show printed the FALLBACK view, not the token view it is documented to print: the if fallbacks guard stayed at the old site when the arm moved above --show
- nothing failed -- both outputs are plausible, and the flag that selects between them was the one that stopped being read
- found while using --show for something else; the CLI has no test that a flag still selects what it names
