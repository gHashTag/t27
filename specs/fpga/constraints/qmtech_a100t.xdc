################################################################################
# QMTECH XC7A100T (Wukong Board) XDC Constraints File
# For ZeroDSP FPGA Implementation
# φ² + 1/φ² = 3 | TRINITY
################################################################################
# Board: QMTECH XC7A100T-324 Core Board + Wukong Expansion
# FPGA:  Xilinx Artix-7 XC7A100T-CSG324
# Clock: 12 MHz input clock
################################################################################

################################################################################
# Clock Constraints
################################################################################

# System clock - 12MHz input
set_property -dict { PACKAGE_PIN E3    IOSTANDARD LVCMOS33 } [get_ports clk]
create_clock -add -name sys_clk -period 83.333 -waveform {0 41.666} [get_ports clk]

# Reset button (active low)
set_property -dict { PACKAGE_PIN C18   IOSTANDARD LVCMOS33 } [get_ports rst_n]

################################################################################
# UART Signals (CP2102 USB-UART bridge)
################################################################################

# UART RX (FPGA receives from USB-UART)
set_property -dict { PACKAGE_PIN T14   IOSTANDARD LVCMOS33 } [get_ports uart_rx[0]]
set_property -dict { PACKAGE_PIN U14   IOSTANDARD LVCMOS33 } [get_ports uart_rx[1]]
set_property -dict { PACKAGE_PIN V14   IOSTANDARD LVCMOS33 } [get_ports uart_rx[2]]
set_property -dict { PACKAGE_PIN V13   IOSTANDARD LVCMOS33 } [get_ports uart_rx[3]]
set_property -dict { PACKAGE_PIN W14   IOSTANDARD LVCMOS33 } [get_ports uart_rx[4]]
set_property -dict { PACKAGE_PIN W13   IOSTANDARD LVCMOS33 } [get_ports uart_rx[5]]
set_property -dict { PACKAGE_PIN V12   IOSTANDARD LVCMOS33 } [get_ports uart_rx[6]]
set_property -dict { PACKAGE_PIN U12   IOSTANDARD LVCMOS33 } [get_ports uart_rx[7]]

# UART TX (FPGA transmits to USB-UART)
set_property -dict { PACKAGE_PIN T15   IOSTANDARD LVCMOS33 } [get_ports uart_tx[0]]
set_property -dict { PACKAGE_PIN U15   IOSTANDARD LVCMOS33 } [get_ports uart_tx[1]]
set_property -dict { PACKAGE_PIN V15   IOSTANDARD LVCMOS33 } [get_ports uart_tx[2]]
set_property -dict { PACKAGE_PIN W15   IOSTANDARD LVCMOS33 } [get_ports uart_tx[3]]
set_property -dict { PACKAGE_PIN V16   IOSTANDARD LVCMOS33 } [get_ports uart_tx[4]]
set_property -dict { PACKAGE_PIN U16   IOSTANDARD LVCMOS33 } [get_ports uart_tx[5]]
set_property -dict { PACKAGE_PIN V17   IOSTANDARD LVCMOS33 } [get_ports uart_tx[6]]
set_property -dict { PACKAGE_PIN U17   IOSTANDARD LVCMOS33 } [get_ports uart_tx[7]]

################################################################################
# LED Outputs (8 LEDs on Wukong board)
################################################################################

set_property -dict { PACKAGE_PIN H17   IOSTANDARD LVCMOS33 } [get_ports { led[0] }]
set_property -dict { PACKAGE_PIN K15   IOSTANDARD LVCMOS33 } [get_ports { led[1] }]
set_property -dict { PACKAGE_PIN J13   IOSTANDARD LVCMOS33 } [get_ports { led[2] }]
set_property -dict { PACKAGE_PIN N14   IOSTANDARD LVCMOS33 } [get_ports { led[3] }]
set_property -dict { PACKAGE_PIN R18   IOSTANDARD LVCMOS33 } [get_ports { led[4] }]
set_property -dict { PACKAGE_PIN V17   IOSTANDARD LVCMOS33 } [get_ports { led[5] }]
set_property -dict { PACKAGE_PIN U17   IOSTANDARD LVCMOS33 } [get_ports { led[6] }]
set_property -dict { PACKAGE_PIN U18   IOSTANDARD LVCMOS33 } [get_ports { led[7] }]

################################################################################
# MAC Result Output (debug)
################################################################################

set_property -dict { PACKAGE_PIN T13   IOSTANDARD LVCMOS33 } [get_ports mac_done]
set_property -dict { PACKAGE_PIN T10   IOSTANDARD LVCMOS33 } [get_ports { mac_result[0] }]
set_property -dict { PACKAGE_PIN T11   IOSTANDARD LVCMOS33 } [get_ports { mac_result[1] }]
set_property -dict { PACKAGE_PIN U10   IOSTANDARD LVCMOS33 } [get_ports { mac_result[2] }]
set_property -dict { PACKAGE_PIN U11   IOSTANDARD LVCMOS33 } [get_ports { mac_result[3] }]
set_property -dict { PACKAGE_PIN V11   IOSTANDARD LVCMOS33 } [get_ports { mac_result[4] }]
set_property -dict { PACKAGE_PIN W11   IOSTANDARD LVCMOS33 } [get_ports { mac_result[5] }]
set_property -dict { PACKAGE_PIN W12   IOSTANDARD LVCMOS33 } [get_ports { mac_result[6] }]
set_property -dict { PACKAGE_PIN V12   IOSTANDARD LVCMOS33 } [get_ports { mac_result[7] }]
set_property -dict { PACKAGE_PIN U12   IOSTANDARD LVCMOS33 } [get_ports { mac_result[8] }]
set_property -dict { PACKAGE_PIN V13   IOSTANDARD LVCMOS33 } [get_ports { mac_result[9] }]
set_property -dict { PACKAGE_PIN W13   IOSTANDARD LVCMOS33 } [get_ports { mac_result[10] }]
set_property -dict { PACKAGE_PIN W14   IOSTANDARD LVCMOS33 } [get_ports { mac_result[11] }]
set_property -dict { PACKAGE_PIN V14   IOSTANDARD LVCMOS33 } [get_ports { mac_result[12] }]
set_property -dict { PACKAGE_PIN U14   IOSTANDARD LVCMOS33 } [get_ports { mac_result[13] }]
set_property -dict { PACKAGE_PIN V15   IOSTANDARD LVCMOS33 } [get_ports { mac_result[14] }]
set_property -dict { PACKAGE_PIN W15   IOSTANDARD LVCMOS33 } [get_ports { mac_result[15] }]
set_property -dict { PACKAGE_PIN W16   IOSTANDARD LVCMOS33 } [get_ports { mac_result[16] }]
set_property -dict { PACKAGE_PIN W17   IOSTANDARD LVCMOS33 } [get_ports { mac_result[17] }]
set_property -dict { PACKAGE_PIN V16   IOSTANDARD LVCMOS33 } [get_ports { mac_result[18] }]
set_property -dict { PACKAGE_PIN U16   IOSTANDARD LVCMOS33 } [get_ports { mac_result[19] }]
set_property -dict { PACKAGE_PIN V17   IOSTANDARD LVCMOS33 } [get_ports { mac_result[20] }]
set_property -dict { PACKAGE_PIN U17   IOSTANDARD LVCMOS33 } [get_ports { mac_result[21] }]
set_property -dict { PACKAGE_PIN V18   IOSTANDARD LVCMOS33 } [get_ports { mac_result[22] }]
set_property -dict { PACKAGE_PIN W18   IOSTANDARD LVCMOS33 } [get_ports { mac_result[23] }]
set_property -dict { PACKAGE_PIN W19   IOSTANDARD LVCMOS33 } [get_ports { mac_result[24] }]
set_property -dict { PACKAGE_PIN W20   IOSTANDARD LVCMOS33 } [get_ports { mac_result[25] }]
set_property -dict { PACKAGE_PIN W22   IOSTANDARD LVCMOS33 } [get_ports { mac_result[26] }]
set_property -dict { PACKAGE_PIN U21   IOSTANDARD LVCMOS33 } [get_ports { mac_result[27] }]
set_property -dict { PACKAGE_PIN U22   IOSTANDARD LVCMOS33 } [get_ports { mac_result[28] }]
set_property -dict { PACKAGE_PIN V21   IOSTANDARD LVCMOS33 } [get_ports { mac_result[29] }]
set_property -dict { PACKAGE_PIN V22   IOSTANDARD LVCMOS33 } [get_ports { mac_result[30] }]
set_property -dict { PACKAGE_PIN W21   IOSTANDARD LVCMOS33 } [get_ports { mac_result[31] }]

################################################################################
# Pmod Header A (J10) - Available for expansion
################################################################################

set_property -dict { PACKAGE_PIN G8    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[0] }]
set_property -dict { PACKAGE_PIN G7    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[1] }]
set_property -dict { PACKAGE_PIN G5    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[2] }]
set_property -dict { PACKAGE_PIN G6    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[3] }]
set_property -dict { PACKAGE_PIN D5    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[4] }]
set_property -dict { PACKAGE_PIN D6    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[5] }]
set_property -dict { PACKAGE_PIN E6    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[6] }]
set_property -dict { PACKAGE_PIN E5    IOSTANDARD LVCMOS33 } [get_ports { pmod_a[7] }]

################################################################################
# Pmod Header B (J11) - Available for expansion
################################################################################

set_property -dict { PACKAGE_PIN A5    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[0] }]
set_property -dict { PACKAGE_PIN A4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[1] }]
set_property -dict { PACKAGE_PIN F4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[2] }]
set_property -dict { PACKAGE_PIN H4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[3] }]
set_property -dict { PACKAGE_PIN B5    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[4] }]
set_property -dict { PACKAGE_PIN B4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[5] }]
set_property -dict { PACKAGE_PIN G4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[6] }]
set_property -dict { PACKAGE_PIN J4    IOSTANDARD LVCMOS33 } [get_ports { pmod_b[7] }]

################################################################################
# Configuration Options
################################################################################

# Configuration pins - don't touch
set_property CONFIG_VOLTAGE 3.3 [current_design]
set_property CFGBVS VCCO [current_design]

################################################################################
# Timing Constraints
################################################################################

# Async input registers for metastability prevention
set_false_path -from [get_ports rst_n] -to [all_registers]

# UART baud rate constraint (115200 baud @ 12MHz = ~104 clocks per bit)
create_clock -add -name uart_clk -period 9600 [get_ports uart_rx[*]]

################################################################################
# I/O Standards
################################################################################

# Default I/O standard for all ports
set_property IOSTANDARD LVCMOS33 [current_design]

################################################################################
# DDR3 Memory Interface (optional - not used in basic design)
################################################################################

# DDR3 pins are pre-assigned on QMTECH board
# These are typically on pins:
# - DQS: T6, T8, R7, R9, W4, W8, U4, U8
# - DQSn: U6, U7, T7, R8, Y4, Y8, V4, V8
# - Address: M2, M1, M3, L4, L5, L6, K1, K2, J1, J2, H1, H2, F2
# - Control: N4, N5, N6, P5, P6, M4, M5, M6
# - Data: (see board schematic for full pinout)

