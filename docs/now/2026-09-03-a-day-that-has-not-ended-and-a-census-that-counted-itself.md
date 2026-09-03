# NOW -- A day that has not ended, and a census that counted itself (2026-09-03)

## `--as-of` accepted a future day and printed today's number under it (Refs #2994)

- `tri issues numbers --as-of 2027-01-01` printed **486** open issues -- today's count -- under the heading `AS OF 2027-01-01T23:59:59Z`. It looked anchored, read as history, and was a clock reading wearing next year's label
- GitHub does not object: `created:<=2027-01-01` composed with `closed:>2027-01-01` answers `486 + 0` without complaint. The query is well formed and its answer is worthless
- section 461 refused a MALFORMED date because a date silently becoming "today" is worse than no anchor. **A future date does exactly that while parsing perfectly**, so the refusal has to be about the day being CLOSED, not the string being well shaped. Today is refused for the same reason as tomorrow: its end is in the future, so the count differs from itself by evening
- the rule is `date >= today`; mutating it to `date > today` kills a test. The comparison needs a civil calendar, transcribed rather than invented, and the **century divisor is load-bearing on exactly two days in a hundred thousand** -- swap `36_524` for `36_525` and it invents `1900-02-29`. The discriminating dates were found by sweeping the mutant against an independent calendar
- one expectation in that test was wrong when written: `civil_from_days(20_699)` guessed 2026-09-04, and the answer is 2026-09-03. The code was right and the test was mine

## `--limit` was one of two spellings, and "four call sites" was wrong (Refs #2994)

- the previous entry closed four `--limit` sites and called the class closed. The same class is written `per_page=` in a URL, and that is **22 more lines**
- `tri gates fetches` now walks the crate: **24 fetch sites** -- 5 complete by `--paginate`, 2 reading the API's own `total_count`, 6 asking whether the page filled, 2 taking one row with no total, and **9 printing what they got** (`gates unmeasured`, `gates dead`, `red now`, four in `prcheck`)
- none bites today: 62 workflows against a page of 100, 35 check-runs against 100. The margin that matters is still 486 against 500
- **grepping one spelling and calling the class closed is the same error as counting one population and calling it the subject.** Section 462 is corrected in place

## The census counted itself, twice (Refs #2994)

- its first rule was "the line names `per_page=` and names `repos/`", and it matched **its own definition**, which names both. The fix is not an exception list: both needles must live in the SAME string literal, which a rule never has and a URL always does
- then it counted its own **test fixtures** -- 25 sites where a hand count said 24. Excluding test modules looked like one line, *everything after the first `#[cfg(test)]`*, and that was checked rather than assumed: **five files carry real top-level functions AFTER their test module**, `gates.rs` fifteen of them
- the attribute must be seen before the `mod`, or `main.rs`'s forty ordinary module declarations put the walker in test mode for the rest of the file, silently dropping every fetch after the first
- **what made both visible was the LIST, not the total.** A census that prints only its count cannot be checked; one that prints its members can. `--excluded` prints the other half: 25 lines name a spelling without fetching, against 24 that fetch
