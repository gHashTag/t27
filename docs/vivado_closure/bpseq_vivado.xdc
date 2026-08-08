## AX7203 (xc7a200tfbg484-1) — GF-T microsequencer trainer, commercial P&R closure.
## The one constraint the open flow (nextpnr-xilinx XDC = create_clock only) cannot
## express: the shared-core path is MULTICYCLE, so Vivado closes it deterministically
## and the seed-lottery disappears.

## --- board clock (200 MHz LVDS); Vivado propagates through the IBUFDS automatically ---
set_property -dict {PACKAGE_PIN R4 IOSTANDARD DIFF_SSTL15} [get_ports clk_p]
set_property -dict {PACKAGE_PIN T4 IOSTANDARD DIFF_SSTL15} [get_ports clk_n]
create_clock -period 5.000 -name sys_clk [get_ports clk_p]

## --- the shared-core datapath is a multicycle path ---------------------------------
## rf -> (modf) -> GftSmul/GftSadd -> rf. The microsequencer captures the result only
## once per (cen x settle) cycles, so this path never needs to meet 200 MHz single-cycle
## setup. Its real delay is ~47 ns (~10 cycles); a 16-cycle budget closes it with margin
## while every launch->capture gap in the design is >= 2560 cycles, so this is safe.
## Counters (pc/settle/dc) and the UART FSM are NOT rf_reg, so they stay single-cycle.
set_multicycle_path 16 -setup -from [get_cells -hier -filter {NAME =~ *rf_reg*}] -to [get_cells -hier -filter {NAME =~ *rf_reg*}]
set_multicycle_path 15 -hold  -from [get_cells -hier -filter {NAME =~ *rf_reg*}] -to [get_cells -hier -filter {NAME =~ *rf_reg*}]

## --- I/O -----------------------------------------------------------------------------
set_property -dict {PACKAGE_PIN P20 IOSTANDARD LVCMOS33} [get_ports uart_rx]
set_property -dict {PACKAGE_PIN N15 IOSTANDARD LVCMOS33} [get_ports uart_tx]
set_property -dict {PACKAGE_PIN B13 IOSTANDARD LVCMOS33} [get_ports {led[0]}]
set_property -dict {PACKAGE_PIN C13 IOSTANDARD LVCMOS33} [get_ports {led[1]}]
set_property -dict {PACKAGE_PIN D14 IOSTANDARD LVCMOS33} [get_ports {led[2]}]
set_property -dict {PACKAGE_PIN D15 IOSTANDARD LVCMOS33} [get_ports {led[3]}]
