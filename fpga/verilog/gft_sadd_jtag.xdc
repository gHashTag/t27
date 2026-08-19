# W830: the divided domain, DECLARED, at /4.
#
# `sadd` alone measures 24.59 MHz -- one operation deep, against 2.93 MHz for the
# five-deep perceptron (T568) and 7.16 for the four-term dot product (T551).
# Depth sets the period, so this design needs /4 where those needed /32 and /16.
#
# CFGMCLK is 70.77 MHz on the fastest die (T495), so slowclk is 17.69 MHz, a
# period of 56.5 ns, against a 24.59 MHz path: a 1.39x margin, STATED. W818
# withdrew a published verdict that rested on an unstated frequency.
create_clock -period 113.0 -name slowclk [get_nets slowclk]

# W843: was 56.5 ns (/4, 17.70 MHz). Measured 17.39-17.53 MHz across seeds --
# a miss. /8 gives 8.85 MHz, a period of 113.0 ns and a 2.0x margin against the
# 17.4 MHz this wrapper actually achieves. The verdict below is being re-measured
# at the slower clock; the previous one stood on no margin at all.
