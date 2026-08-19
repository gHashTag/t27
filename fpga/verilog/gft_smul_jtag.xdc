# W839 (T602): the divided domain, DECLARED.
# Six GftSmul instances, all live-driven. A single multiply is the same order as
# gft_sadd's 24.59 MHz (T568). CFGMCLK 70.77 MHz (T495) / 8 = 8.85 MHz, period
# 113.0 ns -- a 2.8x margin against 24.59 MHz, STATED rather than assumed.
create_clock -period 113.0 -name slowclk [get_nets slowclk]
