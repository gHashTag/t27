# NOW -- The elaboration ratchet was right and my ruler was wrong (2026-08-29)

## The elaboration ratchet was right and my ruler was wrong (Refs #2754)

- keeping the FnDecl made gen-verilog emit assert inside a task; a fn spelled as a test now parses to a TestBlock and every backend already knows what that is
- I measured with bare iverilog while the gate runs -g2012 -DSIMULATION: two rulers, opposite verdicts about the same file
- elaboration errors back to 176 = baseline, discard unchanged at 30408
