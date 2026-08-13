#!/usr/bin/env bash
# check-fpga-toolchain.sh -- refuse to start place-and-route on a toolchain that
# cannot produce a valid bitstream.
#
# WHY THIS EXISTS.  On 2026-08-14 the path from spec to board broke five times,
# and not once in the code.  Four of the five were invisible at the moment of
# breakage: a deleted binary looked like a stage that ran in 0.0 s, renumbered
# cables looked like a successful flash of the wrong board, a top-level wrapper
# that lived only in a scratch directory looked reproducible until it wasn't.
#
# The fifth is the reason this file is a script and not a paragraph:
# docs/fpga/LOCAL-BITSTREAM-FLOW.md already recorded the constids diagnosis and
# its fix, correctly, the previous day -- and it was applied BACKWARDS, because
# a recipe that must be remembered is a recipe that will eventually not be.
#
# Each check below states what it proves and exits non-zero when it fails.
# Refs #1959

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# The openXC7 fork, cloned to a PERSISTENT path.  Two traps are avoided here:
#   * `.../openxc7/nextpnr-xilinx` is a VENDORED COPY inside the t27 repo with
#     763 constids.  It builds, it runs, and it cannot place a design on this
#     database.  It is not a clone of the fork -- `git remote` inside it
#     reports gHashTag/t27.
#   * a build under a session scratchpad disappears when the session restarts,
#     and the vanished binary then reads as a stage that completed in 0.0 s.
NEXTPNR_SRC="${NEXTPNR_SRC:-/Users/playom/t27/build/fpga/openxc7/nextpnr-openxc7}"
CHIPDB="${CHIPDB:-$REPO_ROOT/build/fpga/openxc7/xc7a200tfbg676-1.bin}"
REF_CONSTIDS="${REF_CONSTIDS:-$REPO_ROOT/build/fpga/openxc7/constids.inc}"

fails=0
pass() { printf '  \033[32mOK\033[0m   %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$1"; fails=$((fails + 1)); }

md5_of() { md5 -q "$1" 2>/dev/null || md5sum "$1" 2>/dev/null | cut -d' ' -f1; }

echo "FPGA toolchain preflight"
echo

# ---- 1. The chip database ------------------------------------------------
# Proves: the 332 MB database is present.  Without it nextpnr aborts before it
# has read a single cell, in a tenth of a second, which reads as "instant
# success" to anything that times stages instead of checking exit codes.
if [ -f "$CHIPDB" ]; then
    pass "chipdb present ($(( $(wc -c < "$CHIPDB") / 1048576 )) MB)"
else
    fail "chipdb missing: $CHIPDB"
fi

# ---- 2. constids agreement ----------------------------------------------
# Proves: the binary's internal IDs are numbered the way the database expects.
# constids are ORDINAL -- each X(name) takes the next integer -- so a file with
# the same names in a different order produces a database-wide off-by-N and an
# assertion that advises regenerating 1.3 GB.  The real fix is this file.
SRC_CONSTIDS="$NEXTPNR_SRC/xilinx/constids.inc"
if [ ! -f "$REF_CONSTIDS" ]; then
    fail "reference constids missing: $REF_CONSTIDS"
elif [ ! -f "$SRC_CONSTIDS" ]; then
    fail "nextpnr source constids missing: $SRC_CONSTIDS"
elif [ "$(md5_of "$REF_CONSTIDS")" = "$(md5_of "$SRC_CONSTIDS")" ]; then
    pass "constids match the database ($(wc -l < "$REF_CONSTIDS" | tr -d ' ') lines)"
else
    fail "constids DIFFER -- P&R will abort. Fix: cp '$REF_CONSTIDS' '$SRC_CONSTIDS' && cmake --build '$NEXTPNR_SRC/build' -j8"
fi

# ---- 2b. The source is the FORK, not the vendored copy -------------------
# Proves: this tree came from openXC7/nextpnr-xilinx.  Matching constids alone
# is not enough -- the vendored copy can be made to pass check 2 by copying the
# reference file in, and it still fails at `Unable to constrain IO ... device
# does not have a pin named ''` because its pin tables are a different vintage.
# That failure is indistinguishable from a bad XDC, which is what makes it
# expensive: it sends you to debug a constraints file that is already correct.
origin_url="$(git -C "$NEXTPNR_SRC" remote get-url origin 2>/dev/null || echo '')"
case "$origin_url" in
    *openXC7*) pass "source is the openXC7 fork" ;;
    '')        fail "cannot read git origin of $NEXTPNR_SRC" ;;
    *)         fail "source is NOT the openXC7 fork (origin: $origin_url) -- clone it: git clone --depth 1 -b stable-backports https://github.com/openXC7/nextpnr-xilinx.git" ;;
esac

# ---- 3. The binary actually runs -----------------------------------------
# Proves: the binary exists AND executes.  `--version` is the cheapest call
# that distinguishes "absent" (127) from "present but broken" from "working".
PNR="$NEXTPNR_SRC/build/nextpnr-xilinx"
if [ ! -x "$PNR" ]; then
    fail "nextpnr-xilinx not executable: $PNR"
elif "$PNR" --version >/dev/null 2>&1; then
    pass "nextpnr-xilinx runs"
else
    fail "nextpnr-xilinx present but exits non-zero on --version"
fi

# ---- 4. Downstream tools -------------------------------------------------
for tool in yosys xc7frames2bit openFPGALoader iverilog; do
    if command -v "$tool" >/dev/null 2>&1; then pass "$tool on PATH"
    else fail "$tool NOT on PATH"; fi
done

# ---- 5. Boards, by ACTUAL bus position -----------------------------------
# Proves: how many cables are attached, and at which addresses TODAY.
# All three cables share serial 210512180081, so --ftdi-serial cannot pick one
# and the bus position is the only handle -- and it changes on every replug.
# Never hardcode a busdev-num; read it here.
echo
if command -v openFPGALoader >/dev/null 2>&1; then
    boards=$(openFPGALoader --scan-usb 2>/dev/null | awk '/0x0403:0x6014/ {print $1":"$2}')
    n=$(printf '%s\n' "$boards" | grep -c . || true)
    if [ "$n" -gt 0 ]; then
        pass "$n board(s) attached"
        printf '%s\n' "$boards" | while read -r b; do
            [ -n "$b" ] && printf '         --busdev-num %s\n' "$(echo "$b" | sed 's/^00*//;s/:00*/:/')"
        done
    else
        fail "no Digilent cables found (0x0403:0x6014)"
    fi
fi

echo
if [ "$fails" -eq 0 ]; then
    echo "PASS -- toolchain can produce and load a bitstream"
    exit 0
else
    echo "FAIL -- $fails check(s) failed; do not trust a bitstream built now"
    exit 1
fi
