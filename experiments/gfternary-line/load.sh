#!/bin/zsh
set -u
R=/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a
W=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w716
BD=${BD:-1:4}
WRONG=$R/fpga/tools/bscan_spi_xc7a100t.bit
# The DONE bit is reported BY THE LOAD, not by --detect (which prints only idcode
# in openFPGALoader 1.1.1). Reading it from the wrong command is how the bracket
# came back as `?` on the first attempt.
load () {
  local log
  log=$(perl -e 'alarm 400; exec @ARGV' openFPGALoader --cable digilent_hs2 --busdev-num $BD "$1" 2>&1)
  if echo "$log" | grep -q "Done            0x0" || echo "$log" | grep -q "ID Error"; then echo 0
  elif echo "$log" | grep -q "done 1"; then echo 1
  else echo "?"; fi
}
printf "%-7s %4s %4s  %s\n" arm A1 B1 verdict
for arm in gft0 gft1 gft2 gft3 gft4; do
  a1=$(load $WRONG)
  b1=$(load $W/out/$arm.bit)
  v=$(perl -e 'alarm 200; exec @ARGV' python3 $R/tools/jtag/read_verdict.py --chain 3 2>&1 \
        | grep -oE "MAGIC PRESENT [^ ]+  const=[0-9]+  beat=[0-9]  ok=[0-9]" | head -1)
  [ -z "$v" ] && v="magic absent"
  printf "%-7s %4s %4s  %s\n" $arm "$a1" "$b1" "$v"
done
