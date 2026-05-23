# XDC constraints — JTAG-to-SPI proxy bitstream for QMTech XC7A100T-FGG676
#
# Refs:
#   UG475  "7 Series FPGAs Packaging and Pinout" — dedicated config pin map.
#   UG470  "7 Series FPGAs Configuration"        — SPI BUSWIDTH / persistence.
#   PR #663 trabucayre/openFPGALoader — FGG676 spiOverJtag variants.

# ----------------------------------------------------------------------
# Dedicated configuration pins (FGG676 package, per UG475 Table 1-58).
# These are the same SPI net names that STARTUPE2 / dedicated bank drives.
# ----------------------------------------------------------------------
set_property LOC C8  [get_ports cs_n]    ; # FCS_B
set_property LOC B19 [get_ports mosi]    ; # MOSI / DQ0
set_property LOC A18 [get_ports miso]    ; # DIN  / DQ1

set_property IOSTANDARD LVCMOS33 [get_ports {cs_n mosi miso}]

# ----------------------------------------------------------------------
# Bitstream properties — minimal proxy, single-line SPI, no compression
# is required but enable for smaller footprint. UNUSEDPIN PULLNONE so we
# do not back-drive the host board's pull-ups during transient config.
# ----------------------------------------------------------------------
set_property BITSTREAM.GENERAL.COMPRESS TRUE       [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLNONE   [current_design]
set_property BITSTREAM.CONFIG.SPI_BUSWIDTH 1       [current_design]
set_property CFGBVS VCCO                           [current_design]
set_property CONFIG_VOLTAGE 3.3                    [current_design]
