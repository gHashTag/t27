# W819 (T544): the divided domain, DECLARED.
#
# `gft_train1_jtag.v` runs its whole datapath on `slowclk`, which a BUFG drives
# from bit 3 of a divider off CFGMCLK -- one sixteenth. T541 measured that the
# divider alone changes nothing in the timing report, because nextpnr never
# learns the ratio and applies the global `--freq` to every clock it discovers.
#
# CFGMCLK measured at 70.77 MHz on the fastest of the three dice (T495), so
# slowclk runs at 70.77 / 16 = 4.42 MHz, a period of 226.1 ns. The datapath was
# measured at 7.60 MHz (T541), which is a 1.72x margin -- stated here rather than
# assumed, because W818 withdrew a published silicon verdict that rested on an
# unstated frequency.
create_clock -period 226.1 -name slowclk [get_nets slowclk]
