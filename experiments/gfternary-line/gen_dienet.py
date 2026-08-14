#!/usr/bin/env python3
"""W750: a ternary network split across DICE, with real data crossing the gap.

Every silicon result this project has is a design that computes on a free-running
LFSR and reports a frozen parity. Nothing has ever carried a value that ANOTHER
die produced. This does.

  die A: 32-bit input word (written over JTAG) -> 16 sparse ternary neurons
         -> a 32-bit symbol vector, read back over JTAG
  die B: that 32-bit symbol vector (written over JTAG) -> one ternary decision

The host is the wire and does NO arithmetic on the payload: it shifts 32 bits out
of A and the same 32 bits into B. Both roles are truth tables, so neither die
holds an adder -- the whole two-layer network is combinational LUTs plus the
BSCANE2 shift register.

THE PROTOCOL is link_relay.py's: CAPTURE loads {magic, payload}, SHIFT walks it
out while the next word walks in, UPDATE latches it. One DR pass is therefore one
write AND the previous read, so a write-then-read is two passes. And the Exit1-DR
transition clocks one extra bit, so the word written must be pre-shifted left by
one -- measured in W720, not guessed.
"""
import argparse, random

def neuron_table(weights, thr, in_bits):
    F = len(weights); out = []
    for pat in range(1 << (F*in_bits)):
        acc = 0
        for j in range(F):
            v = (pat >> (j*in_bits)) & ((1 << in_bits)-1)
            x = (1 if v else -1) if in_bits == 1 else (0 if v == 0 else (1 if v == 1 else -1))
            acc += weights[j]*x
        out.append(1 if acc > thr else (3 if acc < -thr else 0))
    return out

def emit_role(role, n_in, n_out, fanin, in_bits, seed, levels, chain=3):
    rnd = random.Random(seed)
    L = [f"`default_nettype none",
         f"// die role '{role}': {n_out} sparse ternary neurons, fan-in {fanin},",
         f"// {in_bits}-bit inputs, driven and read ENTIRELY over JTAG. Refs #1959",
         f"module dienet_{role} #(parameter integer JTAG_CHAIN_N = {chain});",
         "    wire cfgmclk, eos;",
         '    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (',
         "        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(eos), .PREQ(),",
         "        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),",
         "        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),",
         "        .USRDONEO(1'b1), .USRDONETS(1'b1));",
         "    wire tck, tdi, sel, shift, update, capture;",
         "    reg [31:0] sr = 32'd0;",
         "    reg [31:0] inw = 32'd0;          // latched input word",
         "    wire [31:0] outw;",
         "    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (",
         "        .CAPTURE(capture), .DRCK(), .RESET(), .RUNTEST(), .SEL(sel), .SHIFT(shift),",
         "        .TCK(tck), .TDI(tdi), .TMS(), .UPDATE(update), .TDO(sr[0]));",
         "    always @(posedge tck)",
         "        if (sel) begin",
         "            if (capture) sr <= outw;",
         "            else if (shift) sr <= {tdi, sr[31:1]};",
         "        end",
         "    always @(posedge tck) if (sel && update) inw <= sr;",
    ]
    # the neurons: combinational truth tables over the latched input word
    for o in range(n_out):
        picks = rnd.sample(range(n_in), min(fanin, n_in))
        w = [rnd.choice(levels) for _ in picks]
        tbl = neuron_table(w, 2, in_bits)
        sel_bits = ", ".join(f"inw[{p*in_bits+b}]" for p in reversed(picks) for b in reversed(range(in_bits)))
        L.append(f"    wire [{len(picks)*in_bits-1}:0] s{o} = {{{sel_bits}}};")
        L.append(f"    reg [1:0] r{o};")
        L.append(f"    always @* case (s{o})")
        from collections import Counter
        common = Counter(tbl).most_common(1)[0][0]
        for pat, v in enumerate(tbl):
            if v != common:
                L.append(f"        {len(picks)*in_bits}'d{pat}: r{o} = 2'd{v};")
        L.append(f"        default: r{o} = 2'd{common};")
        L.append("    endcase")
    # pack the symbols into the read word, magic in the high nibble so a dead die
    # or a wrong chain cannot look like a plausible answer
    parts = " , ".join(f"r{o}" for o in reversed(range(n_out)))
    if 2*n_out < 32:
        L.append(f"    assign outw = {{{32-2*n_out}'h{'A5A5A5A'[:max(1,(32-2*n_out)//4)]}, {parts}}};")
    else:
        L.append(f"    assign outw = {{{parts}}};")
    L.append("endmodule")
    return "\n".join(L)

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--role", required=True)          # 'a' or 'b'
    ap.add_argument("-n", type=int, default=32); ap.add_argument("-m", type=int, default=16)
    ap.add_argument("--fanin", type=int, default=5); ap.add_argument("--inbits", type=int, default=1)
    ap.add_argument("--seed", type=int, default=1)
    a = ap.parse_args()
    print(emit_role(a.role, a.n, a.m, a.fanin, a.inbits, a.seed, [-4,-2,-1,1,2,4]))
