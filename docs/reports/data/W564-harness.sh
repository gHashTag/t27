#!/bin/zsh
REPO=/Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a
OUT=/tmp/w560/results8.tsv
: > "$OUT"
n=0
while read -r f; do
  n=$((n+1))
  base=$(basename "$f" .t27)
  zf="/tmp/w560/r2_${base}_$n.zig"
  if ! "$REPO/target/release/t27c" gen "$f" > "$zf" 2>/dev/null || [ ! -s "$zf" ]; then
    printf '%s\tGEN_FAIL\t0\n' "$f" >> "$OUT"; continue
  fi
  log=$(cd /tmp/w560 && zig test "$zf" 2>&1)
  # A compile error is authoritative: zig prints "error:" with a file:line prefix.
  if echo "$log" | grep -qE '^[^ ]+\.zig:[0-9]+:[0-9]+: error:'; then
    printf '%s\tCOMPILE_FAIL\t0\n' "$f" >> "$OUT"
  elif echo "$log" | grep -qE "All [0-9]+ tests passed"; then
    p=$(echo "$log" | grep -oE "All [0-9]+ tests passed" | grep -oE "[0-9]+")
    printf '%s\tALL_PASS\t%s\n' "$f" "$p" >> "$OUT"
  elif echo "$log" | grep -qE "panic: assertion failed|terminated with signal ABRT|[0-9]+ passed; [0-9]+ failed"; then
    p=$(echo "$log" | grep -cE "\.\.\.OK$")
    printf '%s\tTEST_FAIL\t%s\n' "$f" "$p" >> "$OUT"
  else
    printf '%s\tUNKNOWN\t0\n' "$f" >> "$OUT"
  fi
done < /tmp/bdd_ok.txt
