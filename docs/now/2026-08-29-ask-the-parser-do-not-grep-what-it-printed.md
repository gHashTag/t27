# NOW -- Ask the parser, do not grep what it printed (2026-08-29)

## Ask the parser, do not grep what it printed (Refs #2754)

- every dropped token now carries the recovery that threw it away: bdd-block-fallback 23852 (78%), brace-body 4602, top-level-resync 1894, stray-brace and stmt-recovery the rest
- head token is not channel: given was the 4th-largest head at 2453 tokens and the fn arm that names it owed only 43 of them
- the fn arm recognised  by name and called skip_to_next_top_level on it; the shared clause parser already served test/invariant/bench
- zip over two parallel vectors truncated silently and hid two missed push sites -- 27 tokens; the accessor now refuses on a length mismatch
