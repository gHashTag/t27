# NOW -- A flag's value became the corpus, and the report could not say so (2026-08-24)

## A flag's value became the corpus, and the report could not say so (Closes #2161)

- tri damage --json PATH scanned PATH as the corpus: the parser stripped flags but not their values, so five valued flags across two tools redirected the scan to a path with no specs.
- What hid it was the output. 'damaged lines: 0 in 0 files' is the same line the tool prints over a clean 650-spec corpus, because the 0 counts files WITH damage. A scan of nothing and a scan that found nothing were byte-identical. The file count is now printed and an empty scan says so loudly.
- Six negative fixtures from #2161 salvaged and gated: they were written to hold false positives the detector used to produce, the tool landed through other PRs, and the control had never run. Two of my three controls on the new gate found defects in the gate itself.
