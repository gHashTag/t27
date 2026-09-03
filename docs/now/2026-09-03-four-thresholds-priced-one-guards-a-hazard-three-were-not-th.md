# NOW -- Four thresholds priced: one guards a hazard, three were not the class (2026-09-03)

## Four thresholds priced: one guards a hazard, three were not the class (Refs #2994)

- quant.rs depth>8: instrumented over the live corpus -- 2078 calls, maximum depth reached 1, guard taken 0 times, and the census is identical at 1/2/4/6/8/12/16/32/64. It decides nothing about any published number.
- It is kept because removing it STACK-OVERFLOWS on a struct whose field is itself, which #2949 established exists. A guard nobody had run is now run by a test, plus a finite-chain counter-example so the cap is not 'give up on anything nested'.
- red.rs at_least: n>=30 and per_page=30 were two literals linked only by prose. One constant now, with a test that reads the page size back OUT of the URL and checks the count at which the marker flips.
