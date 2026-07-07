set part xc7a200tfgg676-1

set_property SEVERITY {Warning} [get_drc_checks LUTLP-1]

create_project -in_memory -part $part

read_verilog blinky.v
read_xdc blinky.xdc

synth_design -top blinky -part $part
opt_design
place_design
route_design

report_utilization
report_timing

write_bitstream -force blinky.bit

puts "DONE: bitstream written"
