#!/usr/bin/env bash
# Loss of a loop tool must be impossible, not merely regrettable.
#
# Two scripts -- scripts/tri_loop/cost.py and scripts/tri_loop/diffbin.py -- were
# written, used to produce numbers that were then quoted in a pull request, and
# lost. They were never committed to any branch, and when the working copy was
# re-cloned they went with it. Six recovery routes were checked and all came back
# empty: dangling git objects, the reflog, shell history, CI artifacts, PR and
# issue comments, and the session snapshot. Every number they had produced became
# unreproducible in one stroke.
#
# The failure was not carelessness at the keyboard. It was that nothing in the
# repository could tell the difference between a tool that exists and a tool that
# exists only in one untracked working directory. This script is that difference.
#
# It fails when:
#   1. a required loop tool is missing from the working tree
#   2. a required loop tool exists but git does not track it -- the exact state
#      that preceded the loss
#   3. any file under scripts/tri_loop/ is untracked, so a NEW tool cannot be
#      quietly added and then lost the same way
#   4. a required tool is not executable via the dispatcher, i.e. `tri <cmd>`
#      does not resolve
#
# Not claimed: that this makes the tools correct, or that a tracked file cannot
# be deleted in a later commit. It closes the specific hole that swallowed these
# two files -- work that exists only outside version control -- and nothing wider.
set -uo pipefail

cd "$(dirname "$0")/../.." || exit 2

REQUIRED_TOOLS=(
  "scripts/tri_loop/triage.py"
  "scripts/tri_loop/cost.py"
  "scripts/tri_loop/diffbin.py"
  "scripts/tri_loop/damage.py"
  "scripts/tri_loop/damage_freeze.py"
  "scripts/tri_loop/damage_repair.py"
  "scripts/tri_loop/corpus_parse.py"
  "scripts/tri_loop/corpus_status.py"
  "scripts/tri_loop/diffmodes.py"
  "scripts/tri_loop/loop_rules.py"
  "scripts/tri_loop/gate_sweep.py"
)
REQUIRED_SUBCOMMANDS=(triage cost diffbin damage damage-freeze damage-repair \
                      corpus-parse corpus-status diffmodes loop-rules \
                      gate-sweep)

fail=0
note() { printf '  %s\n' "$1"; }

echo "loop-tools-tracked: the tools the measurement loop depends on"
echo

echo "1. present in the working tree"
for f in "${REQUIRED_TOOLS[@]}"; do
  if [[ -f "$f" ]]; then
    note "ok       $f"
  else
    note "MISSING  $f"
    fail=1
  fi
done

echo
echo "2. tracked by git (this is the check that would have caught the loss)"
for f in "${REQUIRED_TOOLS[@]}"; do
  [[ -f "$f" ]] || continue
  if git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
    note "tracked  $f"
  else
    note "UNTRACKED $f -- exists here and nowhere else; one re-clone loses it"
    fail=1
  fi
done

echo
echo "3. nothing under scripts/tri_loop/ is untracked"
untracked=$(git ls-files --others --exclude-standard -- scripts/tri_loop/ 2>/dev/null)
if [[ -z "$untracked" ]]; then
  note "ok       no untracked files under scripts/tri_loop/"
else
  while IFS= read -r u; do
    [[ -n "$u" ]] && note "UNTRACKED $u"
  done <<<"$untracked"
  fail=1
fi

echo
echo "4. reachable through the dispatcher"
# The dispatcher resolves `tri <cmd>` generically, to tri_loop/<cmd with - as _>.py.
# So the honest check is that the generic exec line is still there AND that the
# file each subcommand resolves to exists. An earlier draft of this check grepped
# scripts/tri for the literal command name, which passed for `tri triage` only
# because the word appears in a comment -- a check that reports the presence of
# its own documentation is worth nothing.
if grep -q 'exec python3 "\$LOOP_DIR/\${cmd//-/_}.py"' scripts/tri; then
  note "ok       generic loop dispatch line present"
else
  note "BROKEN   generic loop dispatch line is gone from scripts/tri"
  fail=1
fi
for c in "${REQUIRED_SUBCOMMANDS[@]}"; do
  target="scripts/tri_loop/${c//-/_}.py"
  if [[ -f "$target" ]]; then
    note "ok       tri $c -> $target"
  else
    note "UNROUTED tri $c -> $target does not exist"
    fail=1
  fi
done

echo
if [[ $fail -eq 0 ]]; then
  echo "PASS: every loop tool is present, tracked, and routed."
  echo "This says nothing about whether any of them is correct."
else
  echo "FAIL: a loop tool is missing, untracked, or unrouted."
  echo "An untracked tool is the state that already destroyed two of these"
  echo "scripts and every number they produced. Commit it before relying on"
  echo "its output for any claim."
fi
exit $fail
