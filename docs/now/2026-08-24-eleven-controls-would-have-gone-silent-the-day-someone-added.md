# NOW -- Eleven controls would have gone silent the day someone added an import (2026-08-24)

## Eleven controls would have gone silent the day someone added an import (Closes #2161)

- Injected one unused sibling import into each self-planting gate and re-ran its own self-check: check_specs_parse broke 5 controls, check_catalog_integrity 4, check_gate_preconditions 2. All eleven report stdout empty and 'expected text absent', which reads as a broken gate on a day when only the harness is broken.
- plant(script, dest) in _prereq copies the script plus every sibling it imports, transitively, including _prereq itself. Six planting sites converted; a guard in check_gate_preconditions reports BARE with file and line, and goes red when one bare copy is reintroduced.
- My first grep keyed on the literal 'shutil.copy(__file__' and found four gates. The guard keyed on the destination being a planted tools/ directory and found six — two used a variable. Same mistake as the widths sweep, one iteration later.
- check_seal_coverage already had its own plant() building a whole world; the shared import shadowed it and the planted world became the script's own path. Imported under an alias. Grep the file for a name before binding it.
