# NOW -- README: CLARA is a specification target, not a compliance claim (2026-09-03)

## README: CLARA is a specification target, not a compliance claim (Closes #3003)

- L231 read "targeting FPGA acceleration and DARPA CLARA compliance" and L361 read "a full DARPA CLARA-compliant reasoning pipeline" -- DARPA does not certify compliance, and t27 has no DARPA award or engagement, so both lines overclaimed a status that does not exist
- L231 now reads "an automated-reasoning pipeline written against the public DARPA CLARA solicitation as a specification target", with an explicit "No DARPA award or engagement" pointer to `CLARA_TRACEABILITY.md`
- L361 now reads "CLARA-style, not certified" -- tt-trinity-euler already used this exact phrasing; the fix brings t27 in line with it
- agreed wording, 2026-09-03, between dmitrii-f-t27 and gHashTag, as part of a six-repo pass ahead of investor due diligence (trinity #891, trinity-fpga, tt-trinity-gamma, gHashTag profile, zig-golden-float)
