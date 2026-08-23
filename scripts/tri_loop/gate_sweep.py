#!/usr/bin/env python3
"""tri gate-sweep -- what every gate does when there is nothing to check.

Plants a complete copy of tools/ into an empty tree, runs each gate there, and
classifies the result: PASS (exit 0 over nothing), VERDICT (non-zero with a
sentence naming the missing input), CRASH (non-zero through a traceback).

It exists because the probe was retyped by hand three iterations running, and
the third copy planted each gate ALONE -- so two gates died on an import and
were recorded as crashing on the repository, and that table justified work. A
probe kept in the tree is measured once and reused. A probe retyped each time
is a new instrument with new defects.

Reports only. Nothing here decides which PASS is legitimate: a self-test that
builds its own corpus belongs in that column, and so does a skip() that
`--require` turns fatal. A CRASH always belongs in neither -- it reports the
harness rather than the subject.

What this does NOT establish: that a gate which gives a verdict here gives the
RIGHT verdict on real inputs. Empty-tree behaviour is one question, and the
narrowest one.
"""
import os
import subprocess
import sys


def main(argv):
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    gate = os.path.join(root, "tools", "check_gate_preconditions.py")
    if not os.path.isfile(gate):
        print("tri gate-sweep: tools/check_gate_preconditions.py is missing.")
        print("  It carries the planting and the classification; this command is")
        print("  a front door to its --sweep mode and has nothing to run without it.")
        return 2
    return subprocess.run([sys.executable, gate, "--sweep", *argv], cwd=root).returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
