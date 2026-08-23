# NOW -- spi joins the executed set; my own "blocked" claim was wrong (2026-08-22)

## measured per file instead of asserted: three of the nine debt modules elaborate clean (Refs #2241)

- I wrote that the nine data-carrying non-executed modules were "blocked by
  #2410/#2413". Measured: spi, top_level and uart elaborate with ZERO errors --
  they were never blocked, only unmapped. bridge (17), power_analysis (14) and
  vcd_conformance_compare (20) do carry elaboration errors.
- spi is now the second executed module: the prescaler round trip
  (set_prescaler(code) -> get_prescaler_div() == divisor) runs 3/3. Honest
  scope: the (clk_freq, target_freq) half of those vectors is the vector's own
  arithmetic, no spec function maps frequencies to a code -- what executes is
  the dispatch chain that a dropped `match` once left as a stub (#1941).
- top_level's led_set vectors are ASPIRATIONAL: the spec has no LED function at
  all, so nothing can execute them as written.
- Negative control on the new renderer: a planted wrong divisor produces
  FAIL + exit 1; restoring the spec returns exit 0.
