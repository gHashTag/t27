# NOW -- A budget half the cost, on the half of the page nobody runs (2026-09-04)

## `gates dead` needs 899 s and was given 420 (Refs #2994)

- `tri whats-open --all` returned **`TIMEOUT after 420s`** for `gates dead`. Measured: that command over its default fleet takes **899 s** -- the budget was **less than half its cost**
- so `--all` has never printed this instrument's answer, and the answer is not small: **15 workflows have never succeeded, across 8875 runs**, the top three at 1983 / 1980 / 1541 runs
- **a budget under the measured cost does not make a slow instrument fast; it makes a working instrument unreadable** -- and honestly-looking, since `TIMEOUT` sits where a number belongs and the page reads as complete
- the tool's own prose said `dead` *"takes over four minutes"* where the measurement is **fifteen**. Both now carry the measured number and its date
- **the fleet was two lists:** `gates dead` defaulted to **3** repositories and `red now` to **4**, both doc comments calling it "the three/four this fleet uses". The difference is `gHashTag/ghashtag.github.io`
- cost of the divergence measured before closing it: that repository has **0** workflows with a file and >= 50 runs at zero successes, and reading it adds **7 s**. The gap hid nothing today -- and the next dead workflow there would have been invisible to the command whose subject is dead workflows
- one `fleet_repos()`, both callers on it. Three tests: it holds the disputed repository, every entry is `owner/repo`, no slug appears twice
- census re-blessed: the move is an ADDRESS, `red.rs:159 -> 152` from deleting seven lines of literal. Buckets identical at 25 sites, 8 / 0 / 3 / 2
