# build.tcl — Vivado 2023.2 batch build script
#
# Target:  xc7a100tfgg676-1  (QMTech Wukong V1)
# Top:     gf16_heartbeat_uart_top
# Output:  build/output/gf16_heartbeat_uart_top.bit
#
# Usage (from fpga/vsa/uart/):
#   vivado -mode batch -nojournal -nolog -source build.tcl
#
# The script resolves source paths relative to the directory it lives in,
# so it can be launched from any working directory as long as -source
# receives the full path (the workflow does: cd fpga/vsa/uart && vivado ... -source build.tcl).

# ── Resolve script directory ───────────────────────────────────────────────────
set script_dir [file dirname [file normalize [info script]]]

# ── Output directory ───────────────────────────────────────────────────────────
set output_dir [file join $script_dir build output]
file mkdir $output_dir

# ── Source files ──────────────────────────────────────────────────────────────
#
# gf16_dot4.v lives one level up in fpga/vsa/ (shared module used by heartbeat
# and other designs).  The path is relative to this script.
set src_top  [file join $script_dir gf16_heartbeat_uart_top.v]
set src_dot4 [file join $script_dir .. gf16_dot4.v]
set xdc_file [file join $script_dir gf16_heartbeat_uart_top.xdc]

# ── Validate that mandatory files are present ─────────────────────────────────
foreach f [list $src_top $xdc_file] {
    if {![file exists $f]} {
        puts "ERROR: Required file not found: $f"
        exit 1
    }
}

# gf16_dot4.v is sourced from the parent directory; warn and continue if absent
# (Vivado will report it as a missing module during synthesis, which is caught
# below as a non-fatal warning so the build can still generate timing/util reports
# from whatever was successfully elaborated).
set dot4_present [file exists $src_dot4]
if {!$dot4_present} {
    puts "WARNING: gf16_dot4.v not found at $src_dot4 — synthesis will proceed"
    puts "         with a black-box stub.  Bitstream will not be functionally"
    puts "         correct but reports will still be generated."
}

# ── Create in-memory project ──────────────────────────────────────────────────
create_project -in_memory -part xc7a100tfgg676-1

set_property target_language Verilog [current_project]

# ── Read sources ──────────────────────────────────────────────────────────────
read_verilog $src_top
if {$dot4_present} {
    read_verilog $src_dot4
}
read_xdc $xdc_file

# ── Synthesis ─────────────────────────────────────────────────────────────────
puts "INFO: Starting synthesis..."
synth_design \
    -top gf16_heartbeat_uart_top \
    -part xc7a100tfgg676-1 \
    -flatten_hierarchy rebuilt

# Check for critical synthesis errors before proceeding
set synth_msgs [get_msg_config -severity ERROR -count]
if {$synth_msgs > 0} {
    puts "ERROR: Synthesis completed with $synth_msgs error(s).  Aborting."
    report_utilization -file [file join $output_dir utilization_synth.rpt]
    exit 1
}

# ── Optimisation ──────────────────────────────────────────────────────────────
puts "INFO: Running opt_design..."
opt_design

# ── Placement ─────────────────────────────────────────────────────────────────
puts "INFO: Running place_design..."
place_design

# ── Routing ───────────────────────────────────────────────────────────────────
puts "INFO: Running route_design..."
route_design

# ── Reports ───────────────────────────────────────────────────────────────────
puts "INFO: Writing reports..."
report_utilization \
    -file [file join $output_dir utilization.rpt]

report_timing_summary \
    -max_paths 10 \
    -file [file join $output_dir timing.rpt]

# ── Bitstream ─────────────────────────────────────────────────────────────────
puts "INFO: Writing bitstream..."
write_bitstream -force \
    [file join $output_dir gf16_heartbeat_uart_top.bit]

puts "INFO: Build complete."
puts "INFO: Bitstream: [file join $output_dir gf16_heartbeat_uart_top.bit]"
