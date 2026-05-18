# 22FDX TOPS/W Projection Methodology

> **READ FIRST:** Every number in this document is a **projection**, not a
> measurement on silicon. No die targeting 22FDX has been received, brought
> up, or characterised. The purpose of this document is to make the
> projection method itself inspectable, so that when (and if) silicon
> arrives, the gap between projection and measurement is auditable
> line-by-line.
>
> **R5-HONEST:** Any reader who reaches a section break without seeing the
> word "projection" in the previous paragraph should treat that omission as
> a bug and file an issue.

---

## 1. Why 22FDX

GlobalFoundries' 22FDX (22 nm fully-depleted SOI with adaptive body bias)
is named here because it is the smallest, fully-public PDK at which a
TRI-NET-class ternary mesh would still benefit from body-bias techniques
(W47 RBB, W48 FBB-active, W49 CapBoost in `trios-coq/Physics/`). It is
*not* selected because we have access to 22FDX shuttles; we do not, and
this document does not assume we will.

Other plausible PDKs (Sky130, IHP-SG13G2, IHP-SG13S, TSMC N28HPC+,
SMIC 28HKC) would change the absolute numbers but not the projection
method. The method here is the contribution; the chosen PDK is the
worked example.

---

## 2. What we are projecting

The TOPS/W projection envelope, for a single tile of the gamma surface
(32 PEs), running INT1.58 inference, **assuming** all of:

- LUT-NPU operator `OP_LUT_NPU = 0xE3` carries the inner loop;
- AVS-48 voltage stacking microcode is engaged
  (`L2_BG_AVS96_STEP_GATE` extension on top);
- Sub-V_T weak-inversion clock domain at V = 0.30 V available
  (`OP_SUBTH_CLK = 0xE4`);
- Triple-Deck RBB / FBB / CapBoost engaged (W47..W49 Coq lemmas);
- Activity factor 0.5 (industry-conservative, see references).

These are *spec-level* assumptions backed by Coq lemmas in this repo.
None of them is a silicon claim.

---

## 3. Confidence-level scheme

Every projected figure is tagged with a confidence band:

| Band   | Meaning                                                                              |
|--------|--------------------------------------------------------------------------------------|
| `C1`   | Algebra-bound: derived directly from a Coq-proven identity; no PDK assumption.       |
| `C2`   | Toolchain-bound: derived from synthesis on an open PDK (Sky130 / Yosys) at this repo.|
| `C3`   | Scaling-bound: PDK-to-PDK scaling rules applied to a `C2` number; cite the rule.     |
| `C4`   | Vendor-cited: backed by a published 22FDX datapoint from GF or a peer-reviewed paper.|
| `C5`   | Speculative: no `C1..C4` backing; included only to show envelope and labelled red.   |

A reader is entitled to ignore any `C5` row and most `C3` rows when
forming an opinion about silicon.

---

## 4. Baseline (current in-repo)

The `STATUS.md` ladder shows the gamma mesh at `SIM` level (Verilog
generated, simulation passes). No `SYNTH` row exists for a 22FDX cell
library because no 22FDX cell library is integrated into the build. The
publicly cited baseline numbers used as projection anchors are:

| Anchor               | Value                | Source / band                          |
|----------------------|----------------------|----------------------------------------|
| W34 baseline TOPS/W  | 225                  | `NOW.md` Wave-35 row; band `C2` (open) |
| W35 LUT-NPU lift     | x1.20                | `NOW.md` Wave-35; Coq Qed; band `C1`   |
| W36 AVS-48 lift      | TOPS/W >= 297        | `NOW.md` Wave-36 W-104-B; band `C1`    |
| W37 Sub-V_T lift     | TOPS/W >= 350        | `NOW.md` Wave-37 W-104-C; band `C1`    |
| W47 RBB lift         | TOPS/W +1.5%         | Coq lemma `rbb_*`; band `C1`           |
| W48 FBB-active lift  | TOPS/W +1.5..1.9%    | Coq lemma `fbb_active_tops_w_lift_*`   |
| W49 CapBoost lift    | TOPS/W +0.7..0.9%    | Coq lemma `cap_boost_tops_w_lift_*`    |

All of those are derived from in-repo Coq lemmas and are **algebra-bound**
(`C1`). They are NOT silicon measurements.

---

## 5. 22FDX scaling assumptions (`C3` to `C4`)

To reach a 22FDX projection from a Sky130-class baseline, we apply these
scaling rules:

| Rule                                                           | Band | Notes                                              |
|---------------------------------------------------------------|------|----------------------------------------------------|
| Dynamic-energy / op scales with `(V_22 / V_130)^2`            | `C4` | textbook CMOS, cite Rabaey 2003                    |
| Cap / op shrinks with `(L_22 / L_130)`, capped at 2x          | `C3` | conservative; finFET-scaling literature is mixed   |
| Leakage / op increases at low V_DD; offset by RBB at idle     | `C3` | Tschanz JSSC 2002; matches W47 RBB lemma           |
| Forward body bias at active path reduces delay ~12%           | `C4` | Mukhopadhyay 2009 + W48 FBB lemma                  |
| 22FDX V_DD nominal 0.8 V; subthreshold 0.4 V                  | `C4` | GF 22FDX datasheet                                 |
| f_max derating at subthreshold: x0.5 vs nominal               | `C1` | W37 lemma `subth_freq_derating_factor_2`           |

A worked propagation of these rules onto the in-repo anchors yields a
**projected** TOPS/W envelope at 22FDX of:

```
   nominal V_DD, no body bias :       350  - 420   TOPS/W   (band C3)
   nominal V_DD, with TripleDeck:     400  - 490   TOPS/W   (band C3)
   subthreshold V_DD, full stack:    >=600 -- 800  TOPS/W   (band C3+C4 mix)
```

No assertion is made that 22FDX silicon would deliver any of these. The
purpose of the table is to **make the method auditable** before silicon
exists. When silicon exists, this table is what gets falsified
line-by-line.

---

## 6. Falsification policy

Each row above is associated with a falsification witness in the Coq
ledger:

- W34 baseline: `Trinity-loss sparsity >= 0.5 @ batch=1` (W-104-A).
- W36 AVS-48: `eta >= 0.93 => TOPS/W >= 297` (W-104-B; `avs_w104_b_witness`).
- W37 Sub-V_T: `V=0.30 + AVS48 + LUT-NPU => TOPS/W >= 350` (W-104-C;
  `subth_w104_c_witness`).

A 22FDX measurement that falsifies any of these will be reported and the
Coq lemma adjusted (or, more likely, the assumption set behind the lemma
narrowed). That is the deal.

---

## 7. What this document does NOT do

- It does not state a measured 22FDX TOPS/W number.
- It does not commit to a 22FDX tape-out.
- It does not compare 22FDX projections against any commercial product.
- It does not name a date for silicon.

If the reader sees any of those done elsewhere on the basis of this
document, that is a misreading and should be reported as an issue.

---

## 8. Cross-links

- `NOW.md` Waves W34..W49 -- the running ledger of lifts.
- `STATUS.md` -- readiness ladder; no SYNTH or GDS at 22FDX.
- `BENCHMARKS.md` -- restrained posture; what is and isn't measured.
- `COMPETITORS.md` -- no parity claim against any commercial NPU.
- `trios-coq/Physics/` -- Coq lemmas that anchor the `C1` rows.
- `docs/TRI_NET_WHITEPAPER.md` -- the line's positioning.
- `tt-trinity-euler` / `tt-trinity-gamma` (chip repos) -- silicon
  targeting decisions live there, not here.

---

## 9. References (external)

- Rabaey, Chandrakasan, Nikolic, *Digital Integrated Circuits*, 2003.
- Tschanz et al., "Adaptive Body Bias for Reducing Impacts of Die-to-Die
  and Within-Die Parameter Variations on Microprocessor Frequency and
  Leakage", JSSC 2002.
- Mukhopadhyay et al., "Modeling and Analysis of Loading Effect in
  Leakage of Nano-Scaled Bulk-CMOS Logic Circuits", 2009.
- Larsson and Svensson, "Noise in Digital Dynamic CMOS Circuits", 1994.
- Jiang et al., capacitive supply decoupling, 2018.
- GlobalFoundries 22FDX product brief (vendor page; cite at use).

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
