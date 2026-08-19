# W832: the divided domain, DECLARED, at /4.
# `sadd` alone measures 24.59 MHz (T568); CFGMCLK is 70.77 MHz on the fastest
# die (T495), so slowclk is 17.69 MHz, period 56.5 ns -- a 1.39x margin, STATED.
create_clock -period 113.0 -name slowclk [get_nets slowclk]

# W844: was 56.5 ns (/4, 17.70 MHz) and MISSED it at 17.26 MHz. /8 gives
# 8.85 MHz, a period of 113.0 ns, a 2.0x margin against the 17.3 MHz measured.
