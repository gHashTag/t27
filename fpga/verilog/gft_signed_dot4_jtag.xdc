# W821 (T551): the divided domain, DECLARED.
#
# All four probes drive live operands, so the design is 12,615 LUT / 2,017 CARRY4
# and nextpnr measures its critical path at 7.16 MHz -- against 53.27 MHz when
# only one operand moved and 97% of the DUT folded away. The whole wrapper runs
# on CFGMCLK/16 through a BUFG.
#
# CFGMCLK measured at 70.77 MHz on the fastest of the three dice (T495), so
# slowclk is 4.42 MHz, a period of 226.1 ns, against a 7.16 MHz path: a 1.62x
# margin, STATED rather than assumed. W818 withdrew a published silicon verdict
# that rested on an unstated frequency; this is what not repeating that looks
# like.
create_clock -period 226.1 -name slowclk [get_nets slowclk]
