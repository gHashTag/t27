#!/bin/zsh
# W771: build a bitstream and DELETE THE INTERMEDIATES IMMEDIATELY.
#
# Waves W746-W771 filled 6.6 GB of scratchpad and eventually ENOSPC'd the whole
# machine, which broke the Bash tool itself -- the one needed to clean up. The
# artefacts are 20,000-line .frames and .fasm files that regenerate from the
# netlist in under a minute. Only the .bit is worth keeping, and only until the
# next build of the same design.
#
# Usage: fpga-build.sh <top> <verilog...>   -- writes <top>.bit beside the json
set -u
R=${R:-$(git rev-parse --show-toplevel)}
PNR=${PNR:-/Users/playom/t27/build/fpga/openxc7/nextpnr-openxc7/build/nextpnr-xilinx}
DB=$R/build/fpga/openxc7/prjxray-db/artix7
VPY=${VPY:-/Users/playom/t27/build/fpga/openxc7/venv/bin/python3}
XR=${XR:-/Users/playom/t27/build/fpga/openxc7/prjxray}
TOP=$1; shift
OUT=${OUT:-$(dirname $1)}
set -e
perl -e 'alarm 2400; exec @ARGV' yosys -p \
  "read_verilog -sv $*; synth_xilinx -family xc7 -nodsp -nosrl -top $TOP -flatten; stat; write_json $OUT/$TOP.json" \
  -l $OUT/$TOP.yosys >/dev/null 2>&1
# the guard from W755: refuse known-bad primitives rather than build a wrong bitstream
$R/target/release/t27c yostat $OUT/$TOP.yosys || { echo "GUARD: known-bad primitive"; exit 2; }
perl -e 'alarm 3000; exec @ARGV' $PNR --chipdb $R/build/fpga/openxc7/xc7a200tfbg676-1.bin \
  --json $OUT/$TOP.json --fasm $OUT/$TOP.fasm >$OUT/$TOP.pnr 2>&1
perl -e 'alarm 1800; exec @ARGV' $VPY $XR/utils/fasm2frames.py --db-root $DB \
  --part xc7a200tfbg676-1 $OUT/$TOP.fasm > $OUT/$TOP.frames 2>/dev/null
FR=$(wc -l < $OUT/$TOP.frames | tr -d ' ')
if [ "$FR" -lt 100 ]; then echo "GUARD: frames=$FR"; rm -f $OUT/$TOP.frames $OUT/$TOP.fasm; exit 3; fi
perl -e 'alarm 900; exec @ARGV' xc7frames2bit --part_file $DB/xc7a200tfbg676-1/part.yaml \
  --part_name xc7a200tfbg676-1 --frm_file $OUT/$TOP.frames --output_file $OUT/$TOP.bit >/dev/null 2>&1
# THE POINT OF THIS SCRIPT
rm -f $OUT/$TOP.frames $OUT/$TOP.fasm
echo "  $TOP.bit built; frames+fasm removed (frames were $FR lines)"
