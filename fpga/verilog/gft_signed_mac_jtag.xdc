# W839 (T602): the divided domain, DECLARED.
# Five GftSignedMac instances: two multiplies and an add in series. The FOUR-term
# version of this chain measured 7.16 MHz (T551), so the two-term one is at least
# that. CFGMCLK 70.77 MHz (T495) / 16 = 4.42 MHz, period 226.1 ns -- a 1.62x
# margin at worst, STATED. W818 withdrew a verdict that rested on an unstated
# frequency; this is what not repeating that looks like.
create_clock -period 226.1 -name slowclk [get_nets slowclk]
