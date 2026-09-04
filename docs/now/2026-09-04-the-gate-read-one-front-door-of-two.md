# NOW -- The gate read one front door of two (2026-09-04)

## The gate read one front door of two (Closes #3102)

- tri resolves 215 names across four surfaces; 97 live mentions of 24 names resolve on none of them -- tri git 23, tri spec 14, tri queen 9
- Fenced blocks were ungated and are twice as dense in dead names: 18.5% backticked against 36.5% fenced, and the README Quick Start still said ./scripts/tri gen-zig
- The fenced matcher is anchored to the start of the line: unanchored it reported 110 findings of which the majority were English -- t27c was, t27c is, tri binary
- The tri half reports under a down-only ceiling because the 24 names sit in 13 document families describing an intended product CLI; a red gate is a muted gate
- Five mutations: a new dead name fails, a removed one fails, disabling the fenced surface fails, dropping the t27c fallthrough fails the self-check, and a wrong TRI_BIN refuses with 2 instead of falling through
