# vivado_build.tcl — batch build the GF-T microsequencer trainer for the AX7203.
#   vivado -mode batch -source vivado_build.tcl
# Produces bpseq_vivado.bit with DETERMINISTIC timing closure (no seed-search):
# the multicycle constraint in bpseq_vivado.xdc lets a timing-driven P&R close the
# deep shared-core path that the open flow can only leave relaxed.
set part xc7a200tfbg484-1
set top  top
read_verilog [list uart_bpseq.v bpseq.v gft_smul.v gft_sadd.v]
read_xdc bpseq_vivado.xdc
synth_design -top $top -part $part
opt_design
place_design
phys_opt_design
route_design
report_timing_summary -file timing_summary.rpt
# Expect: WNS >= 0 (all paths met, including the multicycle shared-core path).
set wns [get_property SLACK [get_timing_paths -max_paths 1 -nworst 1 -setup]]
puts "=== WNS (setup worst slack) = $wns ns ==="
write_bitstream -force bpseq_vivado.bit
puts "=== wrote bpseq_vivado.bit — flash with openFPGALoader, no seed-search needed ==="
