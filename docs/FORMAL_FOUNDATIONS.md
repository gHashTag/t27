# Formal Foundations

## Prop. 128

### Harness Evolution

Prop. 127b withdrew a false agreement. This makes the harness capable of producing a real one, and it now fails in a way that says why.

The harness evolution progressed through three distinct phases:

- **Wave 660**: Reported a false agreement (an uninitialised variable matching reference 0)
- **Wave 661**: Reported an unexplained X
- **Wave 662**: Reported a named missing precondition

Only the last of these represents a foundation.

### Reference Vector Improvement

The old vector was 9×(+1), 9×(0), 9×(−1), chosen so the reference accumulator would be exactly **0** — wrong under most indexing errors, and *also* what an unwritten counter reads. It could not distinguish a working engine from a silent harness.

The vector is now **27×(+1) against all-(+1) weights**: reference `acc = 27`, reference trit `TRIT_P`. Neither value is reachable by an uninitialised register.

This change ensures that an uninitialised register can no longer mimic a working engine, providing a more reliable test mechanism.

### MAC Capture Flag

`acc_seen` is assigned only under `mac_valid_q`, so it now carries a companion `saw_mac`, and the harness reports *"the MAC never produced a result — nothing was measured"* rather than comparing an initial value.

**Comparing an unassigned variable against a reference is not a measurement, and it looks exactly like a passing test.**

The measurement now shows:
- `saw_mac = 1`, one MAC result
- Engine `acc = 0` against reference 27

This is a genuine measurement.

### Probe Analysis

A probe on the weight path explains the measurement result:

**`weight_bram writes = 0`**

The prefetcher never writes a single word, so the MAC computes against an unwritten memory. The mismatch is explained by **absent weights, not a defect in the datapath**.

### Remaining Open Question

Whether the prefetch fails because this harness does not drive it correctly, or because the design does not start it. Prop. 125's sweep harness *did* raise a prefetch IRQ in some configurations, which suggests the path can work and points at this harness first. **A lead, not a conclusion.**

### What Has Actually Been Gained

| wave | harness reported |
|---|---|
| 660 | a **false agreement** (an uninitialised variable) |
| 661 | an **unexplained X** |
| 662 | a **named missing precondition** |

Only the last of those is a foundation.