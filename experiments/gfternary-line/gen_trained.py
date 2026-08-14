#!/usr/bin/env python3
"""W752: emit Verilog from TRAINED weights. The first such path in this project.

Every prior generator drew its weights from random.Random(seed). This one reads
the trainer's export, so the silicon computes the function that was measured on
UNSW-NB15 rather than a structurally-similar stand-in.

DIE A carries a 593-bit shift register because a row does not fit the 31-bit JTAG
payload (T324): twenty UPDATEs load it, 31 bits at a time, newest bits at the
bottom. Dies R and B take 32-bit symbol vectors in one pass.
"""
import argparse, json, sys
from collections import Counter

def table(weights, in_bits, thr=2):
    F=len(weights); out=[]
    for pat in range(1<<(F*in_bits)):
        acc=0
        for j in range(F):
            v=(pat>>(j*in_bits))&((1<<in_bits)-1)
            x=(1 if v else -1) if in_bits==1 else (0 if v==0 else (1 if v==1 else -1))
            acc+=weights[j]*x
        out.append(1 if acc>thr else (3 if acc<-thr else 0))
    return out

def emit_adder(role, idx, w, in_bits, n_in, chain=3):
    """W752: the OUTPUT layer cannot be a truth table.

    A single decision neuron reads every hidden symbol -- fan-in 16 over 2-bit
    inputs is 32 bits, i.e. a table of 4.3 BILLION entries. The generator hung on
    it, which is how the defect was found. The truth-table trick pays for a WIDE
    layer, where its 2 LUT/neuron is multiplied by the width; the final decision
    is ONE neuron and an adder tree over 16 small integers is a few dozen LUT.

    THIS ALSO MEANS every area figure this project has published -- 128 LUT,
    770 LUT -- counted HIDDEN layers only and silently omitted the output.
    """
    n=len(idx[0])
    L=["`default_nettype none",
       f"// die '{role}': the TRAINED output neuron, fan-in {n}, as an ADDER TREE.",
       f"module trained_{role} #(parameter integer JTAG_CHAIN_N = {chain});",
       "    wire cfgmclk, eos;",
       '    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (',
       "        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(eos), .PREQ(),",
       "        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),",
       "        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),",
       "        .USRDONEO(1'b1), .USRDONETS(1'b1));",
       "    wire tck, tdi, sel, shift, update, capture;",
       "    reg [31:0] sr = 32'd0;",
       "    wire [31:0] outw;",
       "    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (",
       "        .CAPTURE(capture), .DRCK(), .RESET(), .RUNTEST(), .SEL(sel), .SHIFT(shift),",
       "        .TCK(tck), .TDI(tdi), .TMS(), .UPDATE(update), .TDO(sr[0]));",
       "    always @(posedge tck)",
       "        if (sel) begin",
       "            if (capture) sr <= outw;",
       "            else if (shift) sr <= {tdi, sr[31:1]};",
       "        end",
       f"    reg [{n_in*in_bits-1}:0] inw = {n_in*in_bits}'d0;",
       f"    always @(posedge tck) if (sel && update) inw <= sr[{n_in*in_bits-1}:0];"]
    terms=[]
    for j,(p_,cw) in enumerate(zip(idx[0], w[0])):
        if cw==0: continue
        lo=p_*in_bits
        # a ternary symbol: 2'b00 -> 0, 2'b01 -> +1, 2'b11 -> -1
        # The WEIGHT may be negative, so the literal must carry magnitude only and
        # the sign must live in the expression -- `-12'sd-4` is a syntax error and
        # yosys says so at the line, not at the cause.
        mag = abs(cw); pos = f"12'sd{mag}" if cw > 0 else f"-12'sd{mag}"
        neg = f"-12'sd{mag}" if cw > 0 else f"12'sd{mag}"
        L.append(f"    wire signed [11:0] t{j} = (inw[{lo+1}:{lo}] == 2'b01) ? {pos} :"
                 f" (inw[{lo+1}:{lo}] == 2'b11) ? {neg} : 12'sd0;")
        terms.append(f"t{j}")
    expr = " + ".join(terms) if terms else "12'sd0"
    L.append("    wire signed [11:0] acc = " + expr + ";")
    L.append("    assign outw = {16'hA5A5, 14'd0, (acc > 0) ? 2'b01 : 2'b11};")
    L.append("endmodule")
    return "\n".join(L)


def emit(role, idx, w, in_bits, n_in, chain=3, shift_in=False):
    n_out=len(idx)
    L=["`default_nettype none",
       f"// die '{role}': {n_out} TRAINED sparse ternary neurons, fan-in {len(idx[0])},",
       f"// {in_bits}-bit inputs, {n_in} of them. Weights exported from the trainer.",
       f"module trained_{role} #(parameter integer JTAG_CHAIN_N = {chain});",
       "    wire cfgmclk, eos;",
       '    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (',
       "        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(eos), .PREQ(),",
       "        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),",
       "        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),",
       "        .USRDONEO(1'b1), .USRDONETS(1'b1));",
       "    wire tck, tdi, sel, shift, update, capture;",
       "    reg [31:0] sr = 32'd0;",
       "    wire [31:0] outw;",
       "    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (",
       "        .CAPTURE(capture), .DRCK(), .RESET(), .RUNTEST(), .SEL(sel), .SHIFT(shift),",
       "        .TCK(tck), .TDI(tdi), .TMS(), .UPDATE(update), .TDO(sr[0]));",
       "    always @(posedge tck)",
       "        if (sel) begin",
       "            if (capture) sr <= outw;",
       "            else if (shift) sr <= {tdi, sr[31:1]};",
       "        end",
       f"    reg [{n_in*in_bits-1}:0] inw = {n_in*in_bits}'d0;"]
    if shift_in:
        L.append(f"    // 31 new bits per UPDATE, oldest toward the top: twenty passes")
        L.append(f"    // load a full 593-bit row.")
        L.append(f"    always @(posedge tck) if (sel && update)")
        L.append(f"        inw <= {{inw[{n_in*in_bits-32}:0], sr[30:0]}};")
    else:
        L.append("    always @(posedge tck) if (sel && update) inw <= sr[%d:0];" % (n_in*in_bits-1))
    for o in range(n_out):
        picks=idx[o]; ww=w[o]; tbl=table(ww,in_bits)
        sel_bits=", ".join(f"inw[{p*in_bits+b}]" for p in reversed(picks) for b in reversed(range(in_bits)))
        L.append(f"    wire [{len(picks)*in_bits-1}:0] s{o} = {{{sel_bits}}};")
        L.append(f"    reg [1:0] r{o};")
        L.append(f"    always @* case (s{o})")
        common=Counter(tbl).most_common(1)[0][0]
        for pat,v in enumerate(tbl):
            if v!=common: L.append(f"        {len(picks)*in_bits}'d{pat}: r{o} = 2'd{v};")
        L.append(f"        default: r{o} = 2'd{common};")
        L.append("    endcase")
    parts=" , ".join(f"r{o}" for o in reversed(range(n_out)))
    if 2*n_out<32:
        L.append(f"    assign outw = {{{32-2*n_out}'hA5A5, {parts}}};" if 32-2*n_out==16
                 else f"    assign outw = {{{32-2*n_out}'d0, {parts}}};")
    else:
        L.append(f"    assign outw = {{{parts}}};")
    L.append("endmodule")
    return "\n".join(L)

if __name__=="__main__":
    ap=argparse.ArgumentParser()
    ap.add_argument("--net",required=True); ap.add_argument("--layer",type=int,required=True)
    ap.add_argument("--role",required=True); ap.add_argument("--chain",type=int,default=3)
    a=ap.parse_args()
    net=json.load(open(a.net))
    idx=net["idx"][a.layer]; w=net["w"][a.layer]
    in_bits = 1 if a.layer==0 else 2
    n_in = 593 if a.layer==0 else 16
    # THE SIX-BIT RULE decides the FORM, not just the fan-in: a neuron reading
    # more than six bits cannot be a table, and the output neuron never can.
    if len(idx) == 1 or len(idx[0])*in_bits > 12:
        print(emit_adder(a.role, idx, w, in_bits, n_in, a.chain))
    else:
        print(emit(a.role, idx, w, in_bits, n_in, a.chain, shift_in=(a.layer==0)))
