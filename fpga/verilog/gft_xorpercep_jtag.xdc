# W828 (T568): the divided domain, DECLARED, at /32.
#
# The perceptron chain measures 2.93 MHz -- relu, multiply, add, multiply, add in
# series. It is SMALLER than gft_signed_dot4 (10,893 LUT against 12,724) and 2.4x
# slower, because depth and not width sets the period.
#
# CFGMCLK is 70.77 MHz on the fastest die (T495), so slowclk at /32 is 2.21 MHz,
# a period of 452.2 ns, against a 2.93 MHz path: a 1.33x margin, STATED. W818
# withdrew a published silicon verdict that rested on an unstated frequency.
create_clock -period 452.2 -name slowclk [get_nets slowclk]
