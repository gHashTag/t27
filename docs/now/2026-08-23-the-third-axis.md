# NOW — the third axis (2026-08-23)

Two operators asked whether a gate can still reach its verdicts. The third asks whether it reaches the **right** one: a gate that fires its FAIL branch on a healthy tree and its OK branch on a broken one satisfies both of the others.

- **`--invert`.** `if C:` becomes `if not (C):`, on conditions whose body carries a verdict.

  **RETRACTED.** This bullet first read *"all thirteen gates: one survivor, `check_elab_ratchet`'s no-baseline branch — the same one both other operators leave."* The flag did not run the invert operator. `Direction::Invert` was declared, documented, and never constructed; `invert_sites()` had zero callers; `mutate()` chose `if loud { Loud } else { Silent }`. `--invert` printed an invert banner over a silent run, so the survivor reported was the *silent* survivor, and "the same one both other operators leave" was true by construction rather than by measurement.

  The real first measurement, after wiring the delegation: **33 invert mutants across 13 gates, 33 killed, no survivors.** The columns now visibly disagree where they must — `check_json_parses` has one silent site and no invertible condition, `check_vector_data` five inverts against four silents.

  What made this durable is worth naming: the wrong answer was *plausible*. A real measurement of the wrong thing agreed with the story I already believed.

- **Three axes now converge on one line.** That is the first result in this campaign where a new instrument found nothing new, and it is worth as much as the ones that did: the controls written over the last week hold up against a question they were not written for.

- **The scoping is the whole design.** Inverting a loop guard or a plumbing check makes the gate **crash**, and a control that reds on a traceback would score that as a kill — for the wrong reason, which is the mistake this command exists to catch. Only conditions whose body holds a `return 1..4`, a `raise SystemExit(N)` or a FAIL print are sites.

  Measured over 19 mutations on four gates: **19 killed by message, zero tracebacks.** The scoping does what it claims.

- **My verification probe was broken twice before it worked.** It mutated inside the control functions, which the Rust implementation excludes and my Python probe did not; and its verdict label conflated "not killed" with "killed by a crash", so it printed `killed by crash` for a mutant that had survived. Both wrong in the same run, and the first output looked like a finding about the tool.

  That is the seventh instrument of my own to be broken in this campaign, and it is the same lesson each time: **the probe written to check a tool needs the care the tool got, and usually gets less.**

Refs #2492

- **This note was first dated 2026-08-25 and the gate rejected it.** The real UTC date is 2026-08-23. I had been inferring the passage of days from the number of iterations rather than reading a clock, and two earlier notes carry the same invented drift.

  That is mechanism 8.2 — *the claim was true and the world moved* — turned inside out: the world did not move and I recorded that it had. The gate caught it, which is the gate doing precisely its job, and it is the first time in this campaign that an existing gate caught **me** rather than the other way round.
