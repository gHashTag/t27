set part xc7a200tfgg676-1

set_property SEVERITY {Warning} [get_drc_checks LUTLP-1]

create_project -in_memory -part $part

read_verilog gf16_mul.v
read_verilog gf16_top.v
read_xdc gf16_top.xdc

synth_design -top gf16_top -part $part
opt_design
place_design
route_design

report_utilization
report_timing

write_bitstream -force gf16_top.bit

puts "DONE: bitstream written"
