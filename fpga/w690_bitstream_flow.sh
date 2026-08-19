set -u
REPO=/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a
O=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w690
PNR=/Users/playom/t27/build/fpga/openxc7/nextpnr-xilinx/build/nextpnr-xilinx
DB=$REPO/build/fpga/openxc7/prjxray-db/artix7
XR=/Users/playom/t27/build/fpga/openxc7/prjxray
VENV=/Users/playom/t27/build/fpga/openxc7/venv
cd "$O"

echo "### 1. yosys, deleting the \$print cells the spec's test blocks produce"
yosys -p "read_verilog $O/igla_mvp.v $REPO/fpga/verilog/mvp_ternary_classifier_check.v $REPO/fpga/verilog/mvp_ternary_classifier_jtag_noport.v; \
          synth_xilinx -family xc7 -top mvp_ternary_classifier_jtag_noport -flatten; \
          delete t:\$print; delete t:\$scopeinfo; \
          write_json $O/jtag.json" > "$O/yosys.log" 2>&1
echo "yosys rc=$?  json=$( [ -s $O/jtag.json ] && wc -c < $O/jtag.json || echo EMPTY )"
grep -E '^ERROR' "$O/yosys.log" | head -3
sed -n '/Printing statistics/,$p' "$O/yosys.log" | grep -E 'LUT[0-9]|FDRE|BSCANE2|STARTUPE2|Number of cells' | head -10

echo "### 2. nextpnr, no XDC"
"$PNR" --chipdb "$REPO/build/fpga/openxc7/xc7a200tfbg676-1.bin" \
       --json "$O/jtag.json" --write "$O/routed.json" --fasm "$O/jtag.fasm" \
       > "$O/pnr.log" 2>&1
echo "nextpnr rc=$?  fasm=$( [ -s $O/jtag.fasm ] && wc -l < $O/jtag.fasm || echo EMPTY ) lines"
grep -iE '^ERROR' "$O/pnr.log" | head -4

echo "### 3. the six BSCAN routing entries T141 said cannot be expressed"
grep -c 'BSCAN' "$O/jtag.fasm" 2>/dev/null | sed 's/^/  BSCAN FASM lines: /'
grep 'BSCAN' "$O/jtag.fasm" 2>/dev/null | head -10 | sed 's/^/    /'

echo "### 4. fasm2frames  <-- FORECAST W690-F1 DECIDED HERE"
PYTHONPATH=$XR "$VENV/bin/python" "$XR/utils/fasm2frames.py" \
     --db-root "$DB" --part xc7a200tfbg676-1 "$O/jtag.fasm" > "$O/jtag.frames" 2> "$O/f2f.log"
echo "fasm2frames rc=$?  frames=$( [ -s $O/jtag.frames ] && wc -l < $O/jtag.frames || echo EMPTY ) lines"
grep -iE 'FasmLookupError|error' "$O/f2f.log" | head -3

echo "### 5. bitstream, gated on non-empty frames (W690 trap 1)"
if [ -s "$O/jtag.frames" ]; then
  xc7frames2bit --part_file "$DB/xc7a200tfbg676-1/part.yaml" --part_name xc7a200tfbg676-1 \
                --frm_file "$O/jtag.frames" --output_file "$O/jtag.bit" > "$O/bit.log" 2>&1
  echo "xc7frames2bit rc=$?  bit=$(wc -c < "$O/jtag.bit") bytes"
else
  echo "SKIPPED -- frames empty"
fi
echo "### DONE"
