# ═══════════════════════════════════════════════════════════════════════════════════════════
# QMTech XC7A100T Constraints File
# FPGA: Xilinx Artix-7 XC7A100T-1FGG676C
# Board: QMTech XC7A100T FPGA Development Board
# ═════════════════════════════════════════════════════════════════════════════════════════════

# ═════════════════════════════════════════════════════════════════════════════════════════
# CLOCK
# ═════════════════════════════════════════════════════════════════════════════════════════

# 50 MHz system clock (E19 - standard for Artix-7)
set_property -dict {PACKAGE_PIN E19 IOSTANDARD LVCMOS33} [get_ports clk]
create_generated_clock -name clk_50MHz -source [get_ports clk] [get_pins */clk]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# RESET
# ═══════════════════════════════════════════════════════════════════════════════════════════

# Reset button (C12 - active low)
set_property -dict {PACKAGE_PIN C12 IOSTANDARD LVCMOS33} [get_ports rst_n]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# LEDS - Status Indicators
# ═══════════════════════════════════════════════════════════════════════════════════════════

# 4 LEDs (R5, T5, T8, T9 - active low)
set_property -dict {PACKAGE_PIN R5 IOSTANDARD LVCMOS33} [get_ports {led[0]}]
set_property -dict {PACKAGE_PIN T5 IOSTANDARD LVCMOS33} [get_ports {led[1]}]
set_property -dict {PACKAGE_PIN T8 IOSTANDARD LVCMOS33} [get_ports {led[2]}]
set_property -dict {PACKAGE_PIN T9 IOSTANDARD LVCMOS33} [get_ports {led[3]}]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# SWITCHES - Input
# ═════════════════════════════════════════════════════════════════════════════════════════

# 4 switches (A15, C16, C15, P15)
set_property -dict {PACKAGE_PIN A15 IOSTANDARD LVCMOS33} [get_ports {switch[0]}]
set_property -dict {PACKAGE_PIN C16 IOSTANDARD LVCMOS33} [get_ports {switch[1]}]
set_property -dict {PACKAGE_PIN C15 IOSTANDARD LVCMOS33} [get_ports {switch[2]}]
set_property -dict {PACKAGE_PIN P15 IOSTANDARD LVCMOS33} [get_ports {switch[3]}]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# BUTTONS
# ═══════════════════════════════════════════════════════════════════════════════════════════

# 4 buttons (D9, C9, B9, B8 - active low)
set_property -dict {PACKAGE_PIN D9 IOSTANDARD LVCMOS33} [get_ports {btn[0]}]
set_property -dict {PACKAGE_PIN C9 IOSTANDARD LVCMOS33} [get_ports {btn[1]}]
set_property -dict {PACKAGE_PIN B9 IOSTANDARD LVCMOS33} [get_ports {btn[2]}]
set_property -dict {PACKAGE_PIN B8 IOSTANDARD LVCMOS33} [get_ports {btn[3]}]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# UART - Communication with Host
# ═══════════════════════════════════════════════════════════════════════════════════════════

# UART (TX=K20, RX=L20 - at 115200 baud)
set_property -dict {PACKAGE_PIN K20 IOSTANDARD LVCMOS33} [get_ports uart_tx]
set_property -dict {PACKAGE_PIN L20 IOSTANDARD LVCMOS33 PULLUP true} [get_ports uart_rx]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# SPI - External Peripheral Communication
# ═══════════════════════════════════════════════════════════════════════════════════════════

# SPI (CS=G13, SCK=K13, MOSI=H13, MISO=J13)
set_property -dict {PACKAGE_PIN G13 IOSTANDARD LVCMOS33} [get_ports spi_cs]
set_property -dict {PACKAGE_PIN K13 IOSTANDARD LVCMOS33} [get_ports spi_sck]
set_property -dict {PACKAGE_PIN H13 IOSTANDARD LVCMOS33} [get_ports spi_mosi]
set_property -dict {PACKAGE_PIN J13 IOSTANDARD LVCMOS33 PULLUP true} [get_ports spi_miso]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# GPIO - Pmod ports (optional expansion)
# ═══════════════════════════════════════════════════════════════════════════════════════════

# Pmod JA (J1, L1, M1, N1 - top row)
# set_property -dict {PACKAGE_PIN J1 IOSTANDARD LVCMOS33} [get_ports {gpio[0]}]
# set_property -dict {PACKAGE_PIN L1 IOSTANDARD LVCMOS33} [get_ports {gpio[1]}]
# set_property -dict {PACKAGE_PIN M1 IOSTANDARD LVCMOS33} [get_ports {gpio[2]}]
# set_property -dict {PACKAGE_PIN N1 IOSTANDARD LVCMOS33} [get_ports {gpio[3]}]

# Pmod JB (A14, B14, C14, D14)
# set_property -dict {PACKAGE_PIN A14 IOSTANDARD LVCMOS33} [get_ports {gpio[4]}]
# set_property -dict {PACKAGE_PIN B14 IOSTANDARD LVCMOS33} [get_ports {gpio[5]}]
# set_property -dict {PACKAGE_PIN C14 IOSTANDARD LVCMOS33} [get_ports {gpio[6]}]
# set_property -dict {PACKAGE_PIN D14 IOSTANDARD LVCMOS33} [get_ports {gpio[7]}]

# ═══════════════════════════════════════════════════════════════════════════════════════════
# JTAG - Programming Interface (automatic)
# ═══════════════════════════════════════════════════════════════════════════════════════════
# JTAG pins are automatically configured for programming

# ═══════════════════════════════════════════════════════════════════════════════════════════
# DDR3 Memory (if available on QMTech board)
# ═══════════════════════════════════════════════════════════════════════════════════════════
# DDR3 pins - Uncomment if using external DDR3
# Note: QMTech XC7A100T may have DDR3 depending on variant
