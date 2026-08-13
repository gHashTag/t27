#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# scripts/check-runaway-processes.sh
#
# T83/T98: a sweep whose `vvp` call carried no `timeout=` left a simulation
# running for 5h47m at 88% CPU, AFTER the enclosing job had reported completion.
# It was killed. Three hours later the SAME parent had spawned its successor:
#
#     PID 91632   03:00:50   99.1%   vvp
#     PID  8942   08:49:07    0.0%   Python   <- the parent, BLOCKED, waiting
#
# T98's finding: killing the child of an unbounded loop is a symptom whose
# removal is INDISTINGUISHABLE FROM A CURE. The loop advances and hangs again,
# and the observable -- one process at 99% CPU -- looks identical before and
# after. The diagnostic is the PAIRING: a parent at ~0% CPU whose child burns a
# core. Neither half is diagnostic alone.
#
# T94's argument: a lesson protects only the measurements taken after it, and
# only those the author connects to it. The remedy for a class of error is a
# check that runs. This is that check for T98, as
# scripts/check-pagination-truncation.sh is for T90/T91.
#
# Usage:
#   scripts/check-runaway-processes.sh [max-minutes] [--kill]
#
#   max-minutes   wall-clock bound for a compute child (default 30)
#   --kill        terminate the SOURCE (the blocked parent) and its child.
#                 Without it, the script only reports.
#
# A first draft flagged any watched process past the threshold regardless of its
# own CPU, so the BLOCKED PARENT -- at 0.0% by definition -- was itself reported
# as a runaway, with `launchd` named as its source. A runaway BURNS a core; that
# is half the signature and the draft omitted it. A negative control found this,
# not review.
#
# Exit codes:
#   0  nothing found
#   1  runaways found (and killed, if --kill was given)
#
# phi^2 + 1/phi^2 = 3 | TRINITY

set -uo pipefail

MAX_MIN="${1:-30}"
DO_KILL="${2:-}"
# A runaway burns a core. A long-lived idle process does not, and flagging it is
# a false positive -- see the note above.
MIN_CPU="${MIN_CPU:-50}"

# Tools this project spawns that can run unbounded. A generated testbench that
# does not terminate is the observed case; the others are here because the same
# no-timeout mistake applies to any of them.
# `python` is here because the BLOCKED PARENT in T98 was Python -- the sweeps
# this project writes are Python, and a Python child can hang exactly as a vvp
# child can. It matches broadly, which is why the wall-clock threshold and the
# blocked-parent signature, not this list, are what discriminate.
WATCH='vvp|iverilog|yosys|nextpnr|t27c|zig|[Pp]ython'

# Processes we must never touch: not ours, and killing a system daemon is the
# user's call, not the agent's. ReportCrash and trustd have both been observed
# spinning on this machine and are deliberately excluded.
NEVER='ReportCrash|trustd|mds|Spotlight|kernel_task|WindowServer|launchd'

etime_to_min() {
    # ps ELAPSED is [[dd-]hh:]mm:ss
    local e="$1" d=0 h=0 m=0 s=0
    if [[ "$e" == *-* ]]; then d="${e%%-*}"; e="${e#*-}"; fi
    local -a f
    IFS=: read -ra f <<< "$e"
    case "${#f[@]}" in
        3) h="${f[0]}"; m="${f[1]}"; s="${f[2]}" ;;
        2) m="${f[0]}"; s="${f[1]}" ;;
        *) return 1 ;;
    esac
    echo $(( 10#$d * 1440 + 10#$h * 60 + 10#$m ))
}

found=0

while read -r pid ppid etime pcpu comm; do
    [ -z "${pid:-}" ] && continue
    [[ "$comm" =~ $NEVER ]] && continue
    [[ "$comm" =~ $WATCH ]] || continue

    mins=$(etime_to_min "$etime") || continue
    [ "$mins" -lt "$MAX_MIN" ] && continue

    # Half the signature: it must actually be burning a core.
    hot=$(awk -v c="$pcpu" -v t="$MIN_CPU" 'BEGIN{print (c+0 >= t+0) ? 1 : 0}')
    [ "$hot" = "1" ] || continue

    found=1

    # T98's signature: is the PARENT blocked at ~0% CPU while this child burns?
    pinfo=$(ps -o pid=,etime=,pcpu=,comm= -p "$ppid" 2>/dev/null | tr -s ' ')
    pcpu_parent=$(echo "$pinfo" | awk '{print $3}')
    pcomm=$(echo "$pinfo" | awk '{print $4}')
    petime=$(echo "$pinfo" | awk '{print $2}')

    echo "RUNAWAY  pid=$pid  ${etime} (${mins}m)  ${pcpu}%  $comm"

    # `launchd` and other NEVER processes must not be named as a SOURCE either:
    # a child reparented to init is orphaned, not driven by a loop.
    if [[ "${pcomm:-}" =~ $NEVER ]] || [ "$ppid" = "1" ]; then
        echo "  parent is $pcomm (pid $ppid) -- this process is ORPHANED or"
        echo "  system-owned; there is no loop to kill. Reporting only."
        echo
        continue
    fi

    if [ -n "$pinfo" ] && [ -n "${pcpu_parent:-}" ]; then
        # A parent below 5% CPU that has been alive longer than the child is
        # blocked waiting on it -- the unbounded-loop signature.
        low=$(awk -v c="$pcpu_parent" 'BEGIN{print (c+0 < 5.0) ? 1 : 0}')
        if [ "$low" = "1" ]; then
            echo "  SOURCE  pid=$ppid  ${petime}  ${pcpu_parent}%  $pcomm"
            echo "          parent is BLOCKED (<5% CPU) -- killing the child alone"
            echo "          only advances the loop to its next hang (T98)."
            if [ "$DO_KILL" = "--kill" ]; then
                kill "$ppid" 2>/dev/null && echo "          killed source $ppid"
                sleep 1
                kill "$pid" 2>/dev/null && echo "          killed child  $pid"
            fi
        else
            echo "  parent pid=$ppid is at ${pcpu_parent}% -- not the blocked-loop signature;"
            echo "  this may be legitimate long-running work. Reporting only."
            if [ "$DO_KILL" = "--kill" ]; then
                echo "          NOT killing: no source identified."
            fi
        fi
    else
        echo "  parent $ppid is gone -- this process is ORPHANED."
        if [ "$DO_KILL" = "--kill" ]; then
            kill "$pid" 2>/dev/null && echo "          killed orphan $pid"
        fi
    fi
    echo
done < <(ps -axo pid=,ppid=,etime=,pcpu=,comm= 2>/dev/null | tr -s ' ' | sed 's/^ //')

if [ "$found" = "0" ]; then
    echo "OK  no watched process older than ${MAX_MIN}m"
    exit 0
fi

exit 1
