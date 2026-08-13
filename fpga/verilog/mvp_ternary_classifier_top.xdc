# Constraints for mvp_ternary_classifier_top on QMTech Wukong V1 / XC7A200T-FGG676
# Board SSOT: fpga/HARDWARE_SSOT.md
#
# Same two LEDs and the same STARTUPE2/CFGMCLK clock as
# ternary_mac_demo_top_v2.xdc -- a real primitive on a real clock net, so no
# ALLOW_COMBINATORIAL_LOOPS and no CLOCK_DEDICATED_ROUTE override are needed.
# Refs #1959

set_property -dict { PACKAGE_PIN R23 IOSTANDARD LVCMOS33 } [get_ports led_r23]
set_property -dict { PACKAGE_PIN T23 IOSTANDARD LVCMOS33 } [get_ports led_t23]

# CFGMCLK is nominally 65 MHz on 7-series (Xilinx UG470, "Internal
# Configuration Clock") with a wide tolerance (roughly 50-80 MHz), so this
# constraint is deliberately pessimistic.  The classifier is purely
# combinational and the sequencer advances once per 2^24 clocks, so there is no
# timing-critical path -- but constraining the clock keeps the design honest and
# lets the tools report a real slack number.
#
# Addressed to the NET, not the pin: nextpnr-xilinx's XDC reader supports only
# `get_ports` and `get_nets` and errors on `get_pins`.  Vivado accepts either.
create_clock -period 12.500 -name cfgmclk [get_nets cfgmclk]

set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
