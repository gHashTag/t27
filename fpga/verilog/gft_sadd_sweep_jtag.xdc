# W832: the divided domain, DECLARED, at /4.
# `sadd` alone measures 24.59 MHz (T568); CFGMCLK is 70.77 MHz on the fastest
# die (T495), so slowclk is 17.69 MHz, period 56.5 ns -- a 1.39x margin, STATED.
create_clock -period 56.5 -name slowclk [get_nets slowclk]
