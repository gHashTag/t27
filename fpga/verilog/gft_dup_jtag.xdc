# W840 (T607): the divided domain, DECLARED -- and for the first time on this
# bench, against a MEASURED Fmax rather than an assumed one.
#
# Five GftSmul instances. T603 repaired the nextpnr stage to report achieved
# frequency instead of the requested one, and design 12 (six instances of the
# same function) measured slowclk at 32.87 and 47.37 MHz against 8.85 MHz
# declared. This design is smaller, so 8.85 MHz stands with at least that margin.
create_clock -period 113.0 -name slowclk [get_nets slowclk]
