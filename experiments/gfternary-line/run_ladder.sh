#!/bin/zsh
# Synthesise every rung of the GFTernary line through yosys for xc7 and report
# the placed-cell counts.  Timeout on every step (loop invariant).
set -u
W=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w714
N=${N:-64}; M=${M:-8}; ACC=${ACC:-12}; SECS=${SECS:-240}
mkdir -p $W/rtl $W/log
printf "%-8s %-8s %8s %8s %8s %8s %8s\n" arm levels LUT CARRY4 DSP48 MUXF7 secs
for arm in gft0 q4 gft1 gft2 gft3 gft4; do
  v=$W/rtl/$arm.v
  perl -e 'alarm 60; exec @ARGV' python3 $W/gen_ladder.py --arm $arm -n $N -m $M --acc $ACC > $v 2>$W/log/$arm.gen
  if [ ! -s $v ]; then printf "%-8s GEN-FAIL\n" $arm; continue; fi
  lv=$(head -1 $v | grep -oE 'levels=[0-9]+' | cut -d= -f2)
  t0=$(python3 -c 'import time;print(time.time())')
  perl -e 'alarm '"$SECS"'; exec @ARGV' yosys -p \
    "read_verilog -sv $v; synth_xilinx -family xc7 -top layer_$arm -flatten; stat" \
    -l $W/log/$arm.yosys > /dev/null 2>&1
  rc=$?
  t1=$(python3 -c 'import time;print(time.time())')
  secs=$(python3 -c "print(f'{$t1-$t0:.1f}')")
  if [ $rc -ne 0 ]; then printf "%-8s %-8s %8s   rc=%s\n" $arm "$lv" YOSYS-FAIL $rc; continue; fi
  # Read the counts from the stat block ONLY.  Never from a head/tail line count.
  read lut car dsp mux <<< $(python3 - $W/log/$arm.yosys <<'PY'
import sys,re
txt=open(sys.argv[1]).read()
def cell(name):
    m=re.findall(r'^\s+(\d+)\s+%s\s*$'%re.escape(name), txt, re.M)
    return sum(int(x) for x in m) if m else 0
lut=sum(cell("LUT%d"%k) for k in range(1,7))
if "Printing statistics" not in txt: print("NOSTAT NOSTAT NOSTAT NOSTAT"); raise SystemExit
print(lut, cell("CARRY4"), cell("DSP48E1"), cell("MUXF7"))
PY
)
  printf "%-8s %-8s %8s %8s %8s %8s %8s\n" $arm "$lv" "$lut" "$car" "$dsp" "$mux" "$secs"
done
