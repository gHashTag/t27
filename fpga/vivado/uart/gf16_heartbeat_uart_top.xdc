# QMTech Wukong V1 — XC7A100T-FGG676
# heartbeat + UART telemetry pin assignment

# LEDs (active-low on V1 base board)
set_property LOC R23 [get_ports led_d5]
set_property IOSTANDARD LVCMOS33 [get_ports led_d5]

set_property LOC T23 [get_ports led_d6]
set_property IOSTANDARD LVCMOS33 [get_ports led_d6]

set_property LOC J26 [get_ports led_j26]
set_property IOSTANDARD LVCMOS33 [get_ports led_j26]

# UART TX → J2 pin 5 (K20) — drives FT232RL RX
set_property LOC K20 [get_ports uart_tx]
set_property IOSTANDARD LVCMOS33 [get_ports uart_tx]
set_property DRIVE 8 [get_ports uart_tx]
set_property SLEW SLOW [get_ports uart_tx]

# Bitstream config (matches existing gf16_heartbeat_top.xdc + STARTUPCLK)
set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
set_property CFGBVS VCCO [current_design]
set_property CONFIG_VOLTAGE 3.3 [current_design]
set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 4 [current_design]
set_property BITSTREAM.CONFIG.CONFIGRATE 33 [current_design]
# StartupClk literal MUST be 'JtagClk' (case-sensitive; Vivado silently rejects 'JTAGCLK')
set_property BITSTREAM.STARTUP.STARTUPCLK JtagClk [current_design]
