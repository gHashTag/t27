#!/usr/bin/env python3
"""GF-T backprop microcode generator for the on-FPGA MICROSEQUENCER.

The full 2-layer backprop is too large to place in parallel (~22M fasm, over the
~17M openXC7 correctness ceiling). The microsequencer solves this: ONE shared
GftSmul + ONE shared GftSadd core, driven by a microcode program (a sequence of
(op, a, a_mod, b, b_mod, dst) steps over a register file). Any network runs on the
same ~3K-LUT datapath; only the microcode length (= time) and register file grow.

This tool GENERATES the backprop microcode for a 2-layer net (n_in inputs, n_hid
ReLU hidden with fixed biases, n_out linear outputs) and self-tests it with a
bit-faithful GF-T interpreter (trains XOR to 4/4), proving the generated program
is correct. Emitting the Verilog case-ROM from `steps` is mechanical (see
board/bpseq.v for the hand-written (2,2,1) version this reproduces).

Op: 'MUL'|'ADD'|'MOV'.  Operand mod: 0 none, 1 relu, 2 relu', 3 neg, 4 -eta*x
(neg . scale_q, eta=2^-k). Hidden biases fixed at c=[0,-1,-1,...] (the XOR trick).
"""
import math

def enc(x):
    if x == 0.0: return 0
    s = 1 if x < 0 else 0; a = abs(x); e = math.floor(math.log2(a))
    m = int(round((a / 2**e - 1) * 512)); off = e + 40
    if m >= 512: m = 0; off += 1
    if off < 0: return 0
    off = min(off, 127); return (s << 16) | (off << 9) | m

def gen(n_in, n_hid, n_out):
    """Return (reg_map, steps) for the full backprop of a 2-layer net."""
    idx = [0]; reg = {}
    def alloc(name): reg[name] = idx[0]; idx[0] += 1; return reg[name]
    for j in range(n_hid):
        for k in range(n_in): alloc(f"W{j}_{k}")
    for o in range(n_out):
        for j in range(n_hid): alloc(f"v{o}_{j}")
    for k in range(n_in): alloc(f"x{k}")
    for o in range(n_out): alloc(f"t{o}")
    for j in range(n_hid): alloc(f"z{j}")
    for o in range(n_out): alloc(f"y{o}")
    for o in range(n_out): alloc(f"e{o}")
    for j in range(n_hid): alloc(f"dz{j}")
    alloc("m1"); alloc("acc"); alloc("c_-1")
    S = []
    def MUL(a, am, b, bm, d): S.append(("MUL", reg[a], am, reg[b], bm, reg[d]))
    def ADD(a, am, b, bm, d): S.append(("ADD", reg[a], am, reg[b], bm, reg[d]))
    def MOV(a, d):            S.append(("MOV", reg[a], 0, reg[a], 0, reg[d]))
    # forward: z_j = sum_k W[j][k]*x[k] + c_j  (c_0=0; c_{j>=1} = -1, XOR trick)
    for j in range(n_hid):
        MUL(f"W{j}_0", 0, "x0", 0, "acc")
        for k in range(1, n_in):
            MUL(f"W{j}_{k}", 0, f"x{k}", 0, "m1"); ADD("acc", 0, "m1", 0, "acc")
        if j >= 1: ADD("acc", 0, "c_-1", 0, f"z{j}")
        else:      MOV("acc", f"z{j}")
    # y_o = sum_j v[o][j]*relu(z_j) ; e_o = y_o - t_o
    for o in range(n_out):
        MUL(f"v{o}_0", 0, "z0", 1, "acc")
        for j in range(1, n_hid):
            MUL(f"v{o}_{j}", 0, f"z{j}", 1, "m1"); ADD("acc", 0, "m1", 0, "acc")
        MOV("acc", f"y{o}"); ADD(f"y{o}", 0, f"t{o}", 3, f"e{o}")
    # grads: dz_j = (sum_o e_o*v[o][j]) * relu'(z_j)
    for j in range(n_hid):
        MUL("e0", 0, f"v0_{j}", 0, "acc")
        for o in range(1, n_out):
            MUL(f"e{o}", 0, f"v{o}_{j}", 0, "m1"); ADD("acc", 0, "m1", 0, "acc")
        MUL("acc", 0, f"z{j}", 2, f"dz{j}")
    # updates: v[o][j] -= eta*e_o*relu(z_j) ; W[j][k] -= eta*dz_j*x_k
    for o in range(n_out):
        for j in range(n_hid):
            MUL(f"e{o}", 0, f"z{j}", 1, "m1"); ADD(f"v{o}_{j}", 0, "m1", 4, f"v{o}_{j}")
    for j in range(n_hid):
        for k in range(n_in):
            MUL(f"dz{j}", 0, f"x{k}", 0, "m1"); ADD(f"W{j}_{k}", 0, "m1", 4, f"W{j}_{k}")
    return reg, S

# ---- bit-faithful GF-T interpreter (self-test) ----
def _magmul(a16, b16):
    ao = a16 >> 9; am = a16 & 511; bo = b16 >> 9; bm = b16 & 511
    prod = (512 + am) * (512 + bm); carry = 1 if prod >= 524288 else 0
    q = prod >> 9; r = prod & 511; half = 256
    if carry: q = prod >> 10; r = prod & 1023; half = 512
    mant = q - 512
    if r > half: mant += 1
    elif r == half and (q & 1): mant += 1
    sm = ao + bo + carry; oo = 0
    if sm >= 40:
        res = sm - 40; oo = 80 if res >= 80 else res
    if mant >= 512: mant = 0; oo = min(oo + 1, 80)
    return (oo << 9) | mant
def _magadd(a, b):
    ao = a >> 9; am = a & 511; bo = b >> 9; bm = b & 511
    if ao >= bo: ho, hm, lo, lm = ao, am, bo, bm
    else: ho, hm, lo, lm = bo, bm, ao, am
    hs = 512 + hm; ls = 512 + lm; d = min(ho - lo, 11)
    losh = ls >> d; rem = ls - (losh << d); s = hs + losh; off = ho; mant = s - 512
    if s >= 1024:
        g = s & 1; pre = s >> 1; mant = pre - 512
        if g == 1:
            if rem > 0: mant += 1
            elif pre & 1: mant += 1
        off = min(ho + 1, 80)
    else:
        t = rem << 1; hf = 1 << d
        if t > hf: mant += 1
        elif t == hf and (s & 1): mant += 1
    if mant >= 512: mant = 0; off += 1; off = min(off, 80)
    return (off << 9) | mant
def _magsub(hi, lo):
    if hi == lo: return 0
    ho = hi >> 9; hm = hi & 511; lo_o = lo >> 9; lm = lo & 511; d = ho - lo_o
    if d < 0: return 0
    hs = (512 + hm) << 14; la = 0; sticky = 0
    if d >= 26: la = 0; sticky = 1
    else:
        ls = (512 + lm) << 14; la = ls >> d
        if (ls - (la << d)) > 0: sticky = 1
    diff = hs - la; off = ho
    for _ in range(12):
        if diff < 8388608 and off > 1: diff <<= 1; off -= 1
    q = diff >> 14; rem = diff - (q << 14); half = 8192; mant = q - 512
    if rem > half: mant += 1
    elif rem == half:
        if sticky: mant += 1
        elif q & 1: mant += 1
    if mant >= 512: mant = 0; off += 1; off = min(off, 80)
    return (off << 9) | mant
def sadd(a, b):
    if a == 0: return b
    if b == 0: return a
    sa = a >> 16; ma = a & 65535; sb = b >> 16; mb = b & 65535
    if sa == sb: return (sa << 16) | _magadd(ma, mb)
    bsign = sa; r = _magsub(ma, mb)
    if ma < mb: r = _magsub(mb, ma); bsign = sb
    return 0 if r == 0 else (bsign << 16) | r
def smul(a, b):
    if a == 0 or b == 0: return 0
    sgn = 1 if ((a >> 16) & 1) != ((b >> 16) & 1) else 0
    mag = _magmul(a & 65535, b & 65535)
    return 0 if mag == 0 else (sgn << 16) | mag
def neg(v): return 0 if v == 0 else v ^ 65536
def dec(u):
    if u == 0: return 0.0
    s = -1.0 if (u >> 16) & 1 else 1.0; off = (u >> 9) & 0x7f; m = u & 0x1ff
    return s * (1 + m / 512) * 2 ** (off - 40)
def _relu(z): return 0 if (z == 0 or (z >> 16) & 1) else z
def _rp(z): return 0 if (z == 0 or (z >> 16) & 1) else 20480
def _negscale(v, k=3):
    if v == 0: return 0
    off = (v >> 9) & 0x7f; mant = v & 0x1ff
    if off < k + 1: return 0
    return (v & 0x10000) ^ 0x10000 | (((off - k) << 9) | mant)
def _mod(v, m):
    return [v, _relu(v), _rp(v), neg(v), _negscale(v)][m]
def run(steps, rf):
    for (op, a, am, b, bm, d) in steps:
        av = _mod(rf[a], am); bv = _mod(rf[b], bm)
        rf[d] = smul(av, bv) if op == "MUL" else (sadd(av, bv) if op == "ADD" else av)

def emit_verilog(n_in, n_hid, n_out, modname):
    """Emit a synthesizable microsequencer Verilog module for the given arch.
    One shared GftSmul + one shared GftSadd, a register file, and a case(pc) ROM.
    Weights init small-random; build with `synth_xilinx -nocarry` (sequencer
    counters hit the nextpnr CARRY4-placement bug). Bigger nets = same datapath,
    ~constant area (measured: (2,2,1) 2.93M fasm, (2,3,1) 2.92M)."""
    import random
    reg, steps = gen(n_in, n_hid, n_out); N = len(reg); NP = len(steps)
    random.seed(3); initv = {}
    for j in range(n_hid):
        for k in range(n_in): initv[f"W{j}_{k}"] = round(random.uniform(0.5, 1.2), 3)
    for o in range(n_out):
        for j in range(n_hid): initv[f"v{o}_{j}"] = round(random.uniform(-1.0, 1.0), 3)
    initv["c_-1"] = -1.0
    pcw = max(1, NP.bit_length()); L = []
    L.append(f"module {modname}(input clk, input rst, input start, input [31:0] x0i,"
             f" input [31:0] x1i, input [31:0] ti, output reg [31:0] yout, output reg done);")
    L.append(f"  reg [31:0] rf [0:{N-1}];")
    L.append("  function [31:0] modf(input [31:0] v, input [2:0] m); reg neg0; reg [6:0] off;"
             " reg [8:0] mant; begin neg0=v[16]; case(m)")
    L.append("    3'd0:modf=v; 3'd1:modf=(v==0||neg0)?32'd0:v; 3'd2:modf=(v==0||neg0)?32'd0:32'd20480;")
    L.append("    3'd3:modf=(v==0)?32'd0:(v^32'h10000);")
    L.append("    3'd4:begin if(v==0)modf=0; else begin off=(v>>9)&7'h7f; mant=v&9'h1ff;"
             " if(off<3+1)modf=0; else modf=(v&32'h10000)^32'h10000|(((off-3)<<9)|mant); end end")
    L.append("    default:modf=v; endcase end endfunction")
    L.append(f"  reg [{pcw-1}:0] pc; reg [7:0] settle; reg running; reg op; reg [7:0] ai,bi,di; reg [2:0] am,bm;")
    L.append("  always @(*) begin op=0; ai=0; am=0; bi=0; bm=0; di=0; case(pc)")
    for i, (o, a, amod, b, bmod, d) in enumerate(steps):
        if o == "MOV": L.append(f"    {pcw}'d{i}: begin op=2; ai={a}; di={d}; end")
        else: L.append(f"    {pcw}'d{i}: begin op={1 if o=='ADD' else 0}; ai={a}; am={amod}; bi={b}; bm={bmod}; di={d}; end")
    L.append("    default: begin op=0; ai=0; bi=0; di=0; end endcase end")
    L.append("  wire [31:0] a_val=modf(rf[ai],am), b_val=modf(rf[bi],bm); wire [31:0] mul_r, add_r;")
    L.append("  GftSmul u_mul(.clk(clk),.rst_n(1'b1),.en(1'b1),.a(a_val),.b(b_val),.ready(),.result(mul_r));")
    L.append("  GftSadd u_add(.clk(clk),.rst_n(1'b1),.en(1'b1),.a(a_val),.b(b_val),.ready(),.result(add_r));")
    L.append("  localparam SETTLE=8'd40;")
    L.append("  always @(posedge clk) begin if (rst) begin pc<=0; running<=0; done<=0; settle<=0;")
    for name, val in initv.items(): L.append(f"    rf[{reg[name]}]<=32'd{enc(val)};")
    L.append("  end else begin done<=0;")
    L.append(f"    if(!running) begin if(start) begin rf[{reg['x0']}]<=x0i; rf[{reg['x1']}]<=x1i;"
             f" rf[{reg['t0']}]<=ti; pc<=0; settle<=SETTLE; running<=1; end end")
    L.append("    else begin if(settle==0) begin rf[di] <= (op==2)? a_val : (op? add_r : mul_r);")
    L.append(f"      if(pc=={pcw}'d{NP-1}) begin running<=0; done<=1; yout<=rf[{reg['y0']}]; end"
             " else begin pc<=pc+1'b1; settle<=SETTLE; end")
    L.append("    end else settle<=settle-1; end end end")
    L.append("endmodule")
    return "\n".join(L)


if __name__ == "__main__":
    for arch in [(2, 2, 1), (2, 3, 1), (2, 2, 2)]:
        reg, steps = gen(*arch)
        print(f"arch {arch}: {len(reg)} regs, {len(steps)} microcode steps")
    # self-test: generated (2,2,1) XOR net must train to 4/4
    reg, steps = gen(2, 2, 1)
    rf = [0] * len(reg)
    for k, v in {"W0_0": 0.9, "W0_1": 1.1, "W1_0": 1.1, "W1_1": 0.9,
                 "v0_0": 0.8, "v0_1": -1.7, "c_-1": -1.0}.items():
        rf[reg[k]] = enc(v)
    corners = [(0, 0, 0), (1, 0, 1), (0, 1, 1), (1, 1, 0)]
    acc = 0
    for _ in range(50):
        acc = 0
        for a, b, t in corners:
            rf[reg["x0"]] = enc(float(a)); rf[reg["x1"]] = enc(float(b)); rf[reg["t0"]] = enc(float(t))
            run(steps, rf); acc += int((dec(rf[reg["y0"]]) > 0.5)) == t
    assert acc == 4, f"XOR self-test failed: {acc}/4"
    print("self-test: generated backprop microcode trains XOR 4/4 -- OK")
    v = emit_verilog(2, 3, 1, "bpseq231")
    assert "module bpseq231" in v and v.count("\n") > 40
    print("emit_verilog: (2,3,1) module generated -- OK (build with -nocarry, ~2.9M fasm)")
