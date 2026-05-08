set_property -dict { PACKAGE_PIN R23 IOSTANDARD LVCMOS33 } [get_ports led_r23]
set_property -dict { PACKAGE_PIN T23 IOSTANDARD LVCMOS33 } [get_ports led_t23]
set_property -dict { PACKAGE_PIN T14 IOSTANDARD LVCMOS33 } [get_ports uart_rx_pin]
set_property -dict { PACKAGE_PIN T15 IOSTANDARD LVCMOS33 } [get_ports uart_tx_pin]

set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
set_property CLOCK_DEDICATED_ROUTE FALSE [get_nets osc]
set_property ALLOW_COMBINATORIAL_LOOPS TRUE [get_nets osc]
