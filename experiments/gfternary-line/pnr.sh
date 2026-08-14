#!/bin/zsh
# Full P&R for every GFTernary rung: yosys -> nextpnr -> fasm -> bitstream.
# Records the PLACED cell counts and Fmax, neither of which the ladder has ever
# had. Timeout on every step.
set -u
R=/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a
W=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w716
L=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w714
PNR=/Users/playom/t27/build/fpga/openxc7/nextpnr-openxc7/build/nextpnr-xilinx
CHIPDB=$R/build/fpga/openxc7/xc7a200tfbg676-1.bin
DB=$R/build/fpga/openxc7/prjxray-db/artix7
XR=/Users/playom/t27/build/fpga/openxc7/prjxray
N=${N:-64}; M=${M:-8}; ACC=${ACC:-12}
mkdir -p $W/rtl $W/log $W/out
printf "%-7s %8s %8s %8s %8s %9s %10s %8s\n" arm yLUT pLUT pCARRY pFF Fmax_MHz bit_bytes secs
for arm in gft0 gft1 gft2 gft3 gft4; do
  t0=$(python3 -c 'import time;print(time.time())')
  v=$W/rtl/layer_$arm.v; wv=$W/rtl/gft_${arm}_jtag.v
  perl -e 'alarm 60; exec @ARGV' python3 $L/gen_ladder.py --arm $arm -n $N -m $M --acc $ACC > $v 2>/dev/null
  perl -e 'alarm 60; exec @ARGV' python3 $W/wrap.py $arm $M $ACC > $wv 2>/dev/null
  [ -s $v ] && [ -s $wv ] || { printf "%-7s GEN-FAIL\n" $arm; continue; }
  J=$W/out/$arm.json; F=$W/out/$arm.fasm; B=$W/out/$arm.bit
  rm -f $J $F $B
  perl -e 'alarm 600; exec @ARGV' yosys -p \
    "read_verilog -sv $v $wv; synth_xilinx -family xc7 -top gft_${arm}_jtag -flatten; write_json $J" \
    -l $W/log/$arm.yosys >/dev/null 2>&1
  [ -s $J ] || { printf "%-7s YOSYS-FAIL\n" $arm; continue; }
  perl -e 'alarm 1200; exec @ARGV' $PNR --chipdb $CHIPDB --json $J --fasm $F >$W/log/$arm.pnr 2>&1
  [ -s $F ] || { printf "%-7s PNR-FAIL\n" $arm; continue; }
  perl -e 'alarm 600; exec @ARGV' python3 $XR/utils/fasm2frames.py --db-root $DB \
    --part xc7a200tfbg676-1 $F > $W/out/$arm.frames 2>$W/log/$arm.frames.err
  if [ -s $W/out/$arm.frames ]; then
    perl -e 'alarm 300; exec @ARGV' xc7frames2bit --part_file $DB/xc7a200tfbg676-1/part.yaml \
      --part_name xc7a200tfbg676-1 --frm_file $W/out/$arm.frames --output_file $B >/dev/null 2>&1
  fi
  t1=$(python3 -c 'import time;print(time.time())')
  python3 - $W/log/$arm.yosys $W/log/$arm.pnr $B $arm "$(python3 -c "print(f'{$t1-$t0:.0f}')")" <<'PY'
import sys,re,os
ylog,plog,bit,arm,secs=sys.argv[1:6]
yt=open(ylog).read()
def yc(n):
    m=re.findall(r'^\s+(\d+)\s+%s\s*$'%re.escape(n), yt, re.M); return sum(map(int,m)) if m else 0
ylut=sum(yc("LUT%d"%i) for i in range(1,7))
pt=open(plog).read() if os.path.exists(plog) else ""
def pc(name):
    m=re.search(r'^Info:\s+%s:\s+(\d+)/'%re.escape(name), pt, re.M)
    return int(m.group(1)) if m else 0
fm=re.findall(r'Max frequency for clock\s+\'?[^\']*\'?:\s+([\d.]+)\s*MHz', pt)
fmax=max(map(float,fm)) if fm else 0.0
bl=os.path.getsize(bit) if os.path.exists(bit) else 0
print(f"{arm:<7} {ylut:>8} {pc('SLICE_LUTX'):>8} {pc('CARRY4'):>8} {pc('SLICE_FFX'):>8} {fmax:>9.1f} {bl:>10} {secs:>8}")
PY
done
