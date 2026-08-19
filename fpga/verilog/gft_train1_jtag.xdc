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
create_clock -period 452.2 -name slowclk [get_nets slowclk]

# W844: was 226.1 ns (/16, 4.42 MHz), measured 4.82 MHz -- a 1.09x margin,
# thinner than the placer's own seed-to-seed spread. /32 gives 2.21 MHz and a
# 2.2x margin. T617: a thin margin is not why a clause fails, but it makes the
# failure indistinguishable from the one that matters.
