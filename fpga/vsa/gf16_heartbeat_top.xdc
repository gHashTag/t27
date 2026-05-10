set_property LOC R23 [get_ports led_d5]
set_property IOSTANDARD LVCMOS33 [get_ports led_d5]

set_property LOC T23 [get_ports led_d6]
set_property IOSTANDARD LVCMOS33 [get_ports led_d6]

set_property LOC J26 [get_ports led_j26]
set_property IOSTANDARD LVCMOS33 [get_ports led_j26]

set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]
set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 4 [current_design]
set_property BITSTREAM.CONFIG.CONFIGRATE 33 [current_design]
