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
create_clock -period 226.1 -name slowclk [get_nets slowclk]
