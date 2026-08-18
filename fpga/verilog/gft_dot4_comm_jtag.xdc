# W839 (T601): the divided domain, DECLARED.
#
# Three GftSignedDot4 instances with live operands. The single-instance critical
# path measured 7.16 MHz (T551), so the wrapper runs on CFGMCLK/16 through a
# BUFG. CFGMCLK is 70.77 MHz on the fastest die (T495), giving slowclk 4.42 MHz
# and a 226.1 ns period against a 7.16 MHz path -- a 1.62x margin, STATED.
#
# The margin matters more here than anywhere else on this bench: this wrapper
# exists to tell a settling race apart from an arithmetic fact, so a design that
# quietly failed timing would answer its own question wrongly.
create_clock -period 452.2 -name slowclk [get_nets slowclk]

# W844: was 226.1 ns (/16, 4.42 MHz). The measured margin in this family is
# ~1.07x, thinner than the placer's own seed-to-seed spread. /32 gives
# 2.21 MHz and a ~2.1x margin.
