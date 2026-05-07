set_property -dict { PACKAGE_PIN M21 IOSTANDARD LVCMOS33 } [get_ports clk]
set_property -dict { PACKAGE_PIN R23 IOSTANDARD LVCMOS33 } [get_ports led5]
set_property -dict { PACKAGE_PIN T23 IOSTANDARD LVCMOS33 } [get_ports led6]

set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
