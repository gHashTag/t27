#!/bin/bash
mod=$1
mkdir -p build reports

yosys << YOSYS_SCRIPT
read_verilog ${mod}.v
if [ -f gf_formats.v ]; then read_verilog gf_formats.v; fi
hierarchy -check -top ${mod}
proc
opt_clean -purge
stat
write_json build/${mod}.json
write_verilog build/${mod}_synth.v
YOSYS_SCRIPT
