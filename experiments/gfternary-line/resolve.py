#!/usr/bin/env python3
"""W746: the FULL ternary node, including the step that decides.

WHY THIS EXISTS. Every LUT count this project has published for the golden
ladder measured a layer that emits a Z[phi] PAIR (a, b) and stops. But a node
must produce a ternary SYMBOL, and that means evaluating sign(a + b*phi - theta).
phi is irrational, so the pair has to be resolved -- and the resolve is exactly
where a multiplier can reappear. Measuring the accumulate alone and calling it
"multiplier-free" is measuring the half of the node that was never in doubt.

THE FAIR COMPARISON. Both families end at the same place: m ternary symbols.
  - dyadic: value = a, shifted into the comparator frame. Costs nothing.
  - golden: value = (a << 16) + b * round(phi * 65536), ONE constant multiply
    per OUTPUT neuron (m of them), not per weight (n*m of them).
So the golden family pays a fixed toll of m resolves against a saving on every
one of its n*m accumulations. Which side wins is an empirical question, and this
is the file that asks it. The constant is left as a multiply so the synthesiser
picks its own shift-add decomposition -- hand-decomposing it would measure our
cleverness instead of the design.
"""
import argparse, sys

PHI_Q16 = 106039          # round(phi * 65536); relative error 2.7e-6


def emit(arm, m, acc_w, paired):
    o = ["`default_nettype none",
         f"// node_{arm}: layer + pair resolve + ternary threshold. paired={paired}",
         f"module node_{arm} (",
         "    input  wire [63:0] x,",
         f"    output wire [{2*m-1}:0] sym"]
    o.append(");")
    conn = []
    for j in range(m):
        o.append(f"    wire signed [{acc_w-1}:0] a{j};"); conn.append(f".a{j}(a{j})")
        if paired:
            o.append(f"    wire signed [{acc_w-1}:0] b{j};"); conn.append(f".b{j}(b{j})")
    o.append(f"    layer_{arm} inst (.x(x), {', '.join(conn)});")
    # THE RESOLVE. Same output frame for both families, so the comparison is fair.
    vw = acc_w + 18
    for j in range(m):
        if paired:
            o.append(f"    wire signed [{vw-1}:0] v{j} = ($signed(a{j}) <<< 16)"
                     f" + $signed(b{j}) * {PHI_Q16};")
        else:
            o.append(f"    wire signed [{vw-1}:0] v{j} = $signed(a{j}) <<< 16;")
    # Ternary decision against a FIXED integer threshold, in the resolved frame.
    thr = 2 << 16
    for j in range(m):
        o.append(f"    assign sym[{2*j+1}:{2*j}] = (v{j} > {vw}'sd{thr}) ? 2'b01 :"
                 f" (v{j} < -{vw}'sd{thr}) ? 2'b11 : 2'b00;")
    o.append("endmodule")
    return "\n".join(o)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", required=True); ap.add_argument("-m", type=int, default=8)
    ap.add_argument("--acc", type=int, default=16)
    ap.add_argument("--paired", action="store_true")
    a = ap.parse_args()
    print(emit(a.arm, a.m, a.acc, a.paired))
