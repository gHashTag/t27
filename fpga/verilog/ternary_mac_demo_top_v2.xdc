# Constraints for ternary_mac_demo_top_v2 on QMTech Wukong V1 / XC7A200T-FGG676
# Board SSOT: fpga/HARDWARE_SSOT.md
#
# Unlike ternary_mac_demo_top.xdc this file needs no ALLOW_COMBINATORIAL_LOOPS
# and no CLOCK_DEDICATED_ROUTE override: the clock is STARTUPE2/CFGMCLK, a real
# primitive on a real clock net, not a ring oscillator closed through the fabric.

set_property -dict { PACKAGE_PIN R23 IOSTANDARD LVCMOS33 } [get_ports led_r23]
set_property -dict { PACKAGE_PIN T23 IOSTANDARD LVCMOS33 } [get_ports led_t23]

# CFGMCLK is nominally 65 MHz on 7-series (Xilinx UG470, "Internal
# Configuration Clock"); its tolerance is wide (roughly 50-80 MHz), so this
# constraint is deliberately pessimistic. The datapath runs one step per 2^24
# clocks and has no timing-critical path, but constraining the clock keeps the
# design honest and lets the tools report a real slack number.
create_clock -period 12.500 -name cfgmclk [get_pins startup/CFGMCLK]

set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
