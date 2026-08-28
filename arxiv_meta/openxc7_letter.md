# Draft: message to the openXC7 contributors

Not sent. Two versions below — pick one. Neither asks for work; that goes
separately and later.

---

## Version A — for the closed group

**Subject: Fmax through our flow is a size proxy — worth a second corpus**

Hi all —

I filed https://github.com/openXC7/nextpnr-xilinx/issues/166 with something I'd
rather have found before publishing than after.

Benchmarking 21 numeric formats through Yosys 0.65 → nextpnr-xilinx 1743d0f →
prjxray on an XC7A200T, achieved frequency fits `Fmax ≈ 3174·LUT^(-0.648)` at
R² = 0.92 across a 41× span of area. Size explains 92% of frequency, so a
ranking by MHz/LUT is close to a ranking by smallness — which is exactly the
mistake I made: a design led my throughput table because it was the smallest
thing in it.

Second, smaller trap in the same corpus: Yosys constant-folded 15 of 28 on-die
assertion clauses before I measured their area, and the bias runs *with* design
size.

**@cavearr** — this one is aimed at you more than anyone. Your parity report
(#165) compares `himbaechel-xilinx-porting` against 0.9.3, and if any part of
that comparison ranks by frequency, the exponent above says the two revisions
have to be compared at matched area or not at all. If you already normalise for
that, I'd like to know how.

**@jasonzeng124** — adjacent to your DSP48E1 and LUT-truth-table work (#158,
#159): the folding finding is the same class of problem, an optimiser silently
changing what you think you measured. My corpus runs `-nodsp` throughout, so if
you want a DSP-free reference set to test inference against, it's there.

**@hansfbaier**, **@jrrk2** — mostly flagging in case either of you has a corpus
of placed designs lying around. One more fit is worth more than anything else I
can say about mine.

The honest part, which is in the issue too: no place-and-route log for my rows is
committed anywhere, so my frequencies are sourced to a document rather than to
tool output. That's a defect on my side and it's why I'm asking for a second
corpus rather than presenting a law.

Everything numeric recomputes in one command:
https://github.com/gHashTag/ternary-network-floats

— Dmitrii (gHashTag)

---

## Version B — shorter, as a comment on issue #166

@cavearr @jasonzeng124 @hansfbaier @jrrk2 — flagging this one directly because
it bears on how we compare revisions.

If any frequency-based comparison in the project ranks designs of different
sizes, the exponent above (−0.648, R² = 0.92 across 41× in area) says the
ranking is mostly measuring size. @cavearr, that includes the parity report in
#165 if it uses Fmax anywhere.

I'd rather be wrong about this than right. One fit on someone else's corpus
settles it either way — mine has no place-and-route logs behind it, which I've
said in the issue and which is the reason I'm asking.

---

## Third message — separate, later, after there is a reply

Do not merge this into either version above. A reviewer who is simultaneously
being asked for work stops being a reviewer.

> Separate note: I'm looking for paid work in this area — open FPGA toolchains,
> arithmetic datapaths, conformance and measurement infrastructure. If anyone
> knows of a group funding work on openXC7 or on numeric IP for 7-series, a
> pointer would be welcome.
