# NOW -- A crash is loud, and it is still not a verdict (2026-08-24)

## A crash is loud, and it is still not a verdict (Closes #2161)

- Five gates crashed rather than passed in an empty tree. Three raised FileNotFoundError on a file the repository tracks — that is broken(), not a stack trace — and two are self-tests of another gate that raised ModuleNotFoundError when the subject was absent. All five now say which file is missing and that it is tracked.
- Yesterday's sweep planted each gate alone and recorded five tracebacks. Planting the whole tools/ directory, two of those five PASS: they had been dying on an import, not on the repository. My own probe carried the incomplete-planting defect I was fixing, and it silently changed a table I reasoned from.
- The BARE guard from yesterday missed t / 'tools/check_withdrawn_live.py' because it keyed on the destination spelled as a separate component. Widened, it then flagged a data-file copy. Both errors come from keying on one end: a plant is a copy OF A SCRIPT INTO a planted tools/, and the guard now requires both.
