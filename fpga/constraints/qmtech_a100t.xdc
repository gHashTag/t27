# Trinity/t27 FPGA Constraints
# XC7A100T-FGG484 Device Pinout

## Clock Constraints
create_clock -name clk -period 8.33 [get_ports]

## I/O Constraints
set_property -name "MISO" -dict {DONT_TOUCH true}
set_property -name "MOSI" -dict {DONT_TOUCH true}

## Package Constraints
set_property -name "UCF" -dict {USER_BAUD_RATE 9600}

## Notes
- MAC modules use LUTs only (no DSPs)
- Target clock: ~92 MHz achievable with 16x prescaler

## Pin Mapping (to be defined in top_level spec)
# CLKA14, CLKA13, CLKA15: SPI CS, CLK
# CLKA12, CLKA10: SPI MISO, SPI MOSI, SPI CLK
# CLKA7, CLKA8, CLKA6, CLKA5: UART TX, UART RX
# CLKA1, CLKA4, CLKA3, CLKA2: UART CTS, UART RTS
