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
import tempfile
import subprocess
import sys
import os
import math

def enc(x):
    if x == 0.0: return 0
    s = 1 if x < 0 else 0; a = abs(x); e = math.floor(math.log2(a))
    m = int(round((a / 2**e - 1) * 512)); off = e + 40
    if m >= 512: m = 0; off += 1
    if off < 0: return 0
    off = min(off, 127); return (s << 16) | (off << 9) | m

def gen(n_in, n_hid, n_out):
    """Return (reg_map, steps) for the full backprop of a 2-layer net with TRAINABLE
    hidden biases b_j and output biases bo_o (a proper general 2-layer net)."""
    idx = [0]; reg = {}
    def alloc(name): reg[name] = idx[0]; idx[0] += 1; return reg[name]
    for j in range(n_hid):
        for k in range(n_in): alloc(f"W{j}_{k}")
    for j in range(n_hid): alloc(f"b{j}")
    for o in range(n_out):
        for j in range(n_hid): alloc(f"v{o}_{j}")
    for o in range(n_out): alloc(f"bo{o}")
    for k in range(n_in): alloc(f"x{k}")
    for o in range(n_out): alloc(f"t{o}")
    for j in range(n_hid): alloc(f"z{j}")
    for o in range(n_out): alloc(f"y{o}")
    for o in range(n_out): alloc(f"e{o}")
    for j in range(n_hid): alloc(f"dz{j}")
    alloc("m1"); alloc("acc")
    S = []
    def MUL(a, am, b, bm, d): S.append(("MUL", reg[a], am, reg[b], bm, reg[d]))
    def ADD(a, am, b, bm, d): S.append(("ADD", reg[a], am, reg[b], bm, reg[d]))
    # forward: z_j = W_j.x + b_j
    for j in range(n_hid):
        MUL(f"W{j}_0", 0, "x0", 0, "acc")
        for k in range(1, n_in):
            MUL(f"W{j}_{k}", 0, f"x{k}", 0, "m1"); ADD("acc", 0, "m1", 0, "acc")
        ADD("acc", 0, f"b{j}", 0, f"z{j}")
    # y_o = v_o . relu(z) + bo_o ; e_o = y_o - t_o
    for o in range(n_out):
        MUL(f"v{o}_0", 0, "z0", 1, "acc")
        for j in range(1, n_hid):
            MUL(f"v{o}_{j}", 0, f"z{j}", 1, "m1"); ADD("acc", 0, "m1", 0, "acc")
        ADD("acc", 0, f"bo{o}", 0, f"y{o}"); ADD(f"y{o}", 0, f"t{o}", 3, f"e{o}")
    # grads: dz_j = (sum_o e_o*v_oj) * relu'(z_j)
    for j in range(n_hid):
        MUL("e0", 0, f"v0_{j}", 0, "acc")
        for o in range(1, n_out):
            MUL(f"e{o}", 0, f"v{o}_{j}", 0, "m1"); ADD("acc", 0, "m1", 0, "acc")
        MUL("acc", 0, f"z{j}", 2, f"dz{j}")
    # updates: v -= eta*e*relu(z); bo -= eta*e; W -= eta*dz*x; b -= eta*dz
    for o in range(n_out):
        for j in range(n_hid):
            MUL(f"e{o}", 0, f"z{j}", 1, "m1"); ADD(f"v{o}_{j}", 0, "m1", 4, f"v{o}_{j}")
        ADD(f"bo{o}", 0, f"e{o}", 4, f"bo{o}")
    for j in range(n_hid):
        for k in range(n_in):
            MUL(f"dz{j}", 0, f"x{k}", 0, "m1"); ADD(f"W{j}_{k}", 0, "m1", 4, f"W{j}_{k}")
        ADD(f"b{j}", 0, f"dz{j}", 4, f"b{j}")
    return reg, S


def gen_deep(sizes):
    """Return (reg_map, steps) for the full backprop of an L-layer net of arbitrary
    DEPTH. `sizes` = [n_in, h1, h2, ..., n_out] (>=2 entries => >=1 weight layer).
    Hidden layers use ReLU; the output layer is linear (logits). Naming stays
    emit-compatible: inputs x{k}, targets t{o}, outputs y{o}. A 2-layer net
    [n_in, n_hid, n_out] is functionally identical to gen(n_in, n_hid, n_out)."""
    assert len(sizes) >= 2, "need at least [n_in, n_out]"
    L = len(sizes) - 1                      # number of weight layers
    idx = [0]; reg = {}
    def alloc(name): reg[name] = idx[0]; idx[0] += 1; return reg[name]
    # weights W{l}_{j}_{k}: layer l unit j <- layer l-1 unit k ; biases b{l}_{j}
    for l in range(1, L + 1):
        for j in range(sizes[l]):
            for k in range(sizes[l - 1]): alloc(f"W{l}_{j}_{k}")
    for l in range(1, L + 1):
        for j in range(sizes[l]): alloc(f"b{l}_{j}")
    for k in range(sizes[0]): alloc(f"x{k}")
    for o in range(sizes[L]): alloc(f"t{o}")
    for l in range(1, L):                   # hidden pre-activations z{l}_{j}
        for j in range(sizes[l]): alloc(f"z{l}_{j}")
    for o in range(sizes[L]): alloc(f"y{o}")          # linear outputs (== z^L)
    for o in range(sizes[L]): alloc(f"e{o}")          # output error = y - t = delta^L
    for l in range(1, L):                   # hidden deltas d{l}_{j}
        for j in range(sizes[l]): alloc(f"d{l}_{j}")
    alloc("m1"); alloc("acc")
    S = []
    def MUL(a, am, b, bm, d): S.append(("MUL", reg[a], am, reg[b], bm, reg[d]))
    def ADD(a, am, b, bm, d): S.append(("ADD", reg[a], am, reg[b], bm, reg[d]))
    def act(l, k):                          # activation reg + read-mod of layer l unit k
        return (f"x{k}", 0) if l == 0 else (f"z{l}_{k}", 1)   # input raw / hidden relu
    def dst_pre(l, j):                       # where layer l's pre-activation is stored
        return f"y{j}" if l == L else f"z{l}_{j}"
    def delta(l, u):                         # delta reg of layer l unit u
        return f"e{u}" if l == L else f"d{l}_{u}"
    # forward: z^l_j = W^l_j . a^{l-1} + b^l_j
    for l in range(1, L + 1):
        for j in range(sizes[l]):
            r0, m0 = act(l - 1, 0)
            MUL(f"W{l}_{j}_0", 0, r0, m0, "acc")
            for k in range(1, sizes[l - 1]):
                rk, mk = act(l - 1, k)
                MUL(f"W{l}_{j}_{k}", 0, rk, mk, "m1"); ADD("acc", 0, "m1", 0, "acc")
            ADD("acc", 0, f"b{l}_{j}", 0, dst_pre(l, j))
    # output error: e_o = y_o - t_o  (delta^L, linear output)
    for o in range(sizes[L]):
        ADD(f"y{o}", 0, f"t{o}", 3, f"e{o}")
    # hidden deltas (back-to-front): d^l_j = relu'(z^l_j) * sum_u W^{l+1}_{u,j} * delta^{l+1}_u
    for l in range(L - 1, 0, -1):
        for j in range(sizes[l]):
            MUL(f"W{l+1}_0_{j}", 0, delta(l + 1, 0), 0, "acc")
            for u in range(1, sizes[l + 1]):
                MUL(f"W{l+1}_{u}_{j}", 0, delta(l + 1, u), 0, "m1"); ADD("acc", 0, "m1", 0, "acc")
            MUL("acc", 0, f"z{l}_{j}", 2, f"d{l}_{j}")
    # updates: W^l_{j,k} -= eta*delta^l_j*a^{l-1}_k ; b^l_j -= eta*delta^l_j
    for l in range(1, L + 1):
        for j in range(sizes[l]):
            dj = delta(l, j)
            for k in range(sizes[l - 1]):
                rk, mk = act(l - 1, k)
                MUL(dj, 0, rk, mk, "m1"); ADD(f"W{l}_{j}_{k}", 0, "m1", 4, f"W{l}_{j}_{k}")
            ADD(f"b{l}_{j}", 0, dj, 4, f"b{l}_{j}")
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

def emit_verilog(n_in, n_hid, n_out, modname, clk_div=1, init=None):
    """Emit a synthesizable microsequencer Verilog module for the given arch.
    One shared GftSmul + one shared GftSadd, a register file, and a case(pc) ROM.
    Fully parametric interface: one x{k}i input port per input, one t{o}i target
    port per output, a packed yout ([32*n_out-1:0], y0 in the LSB word). Weights
    init small-random; build with `synth_xilinx -nocarry` (sequencer counters hit
    the nextpnr CARRY4-placement bug). Bigger nets = same datapath, ~constant area
    (measured: (2,2,1) 2.93M fasm, (2,3,1) 2.92M)."""
    import random
    # Fully parametric interface: one x{k}i port per input, one t{o}i port per
    # target, and a packed yout ([32*n_out-1:0], y0 in the LSB word). Every t{o} is
    # driven, so no target register is read uninitialized (the old multi-output bug).
    if n_in < 1 or n_hid < 1 or n_out < 1:
        raise ValueError(f"emit_verilog needs n_in,n_hid,n_out >= 1; got {(n_in, n_hid, n_out)}")
    reg, steps = gen(n_in, n_hid, n_out); N = len(reg); NP = len(steps)
    random.seed(3); initv = {}
    for j in range(n_hid):
        for k in range(n_in): initv[f"W{j}_{k}"] = round(random.uniform(0.5, 1.2), 3)
    for o in range(n_out):
        for j in range(n_hid): initv[f"v{o}_{j}"] = round(random.uniform(-1.0, 1.0), 3)
    for j in range(n_hid): initv[f"b{j}"] = round(random.uniform(-0.5, 0.5), 3)
    for o in range(n_out): initv[f"bo{o}"] = 0.0
    if init is not None:            # caller-supplied weights (e.g. an XOR near-solution)
        initv.update(init)
    return _emit_module(reg, steps, initv, n_in, n_out, modname, clk_div)


def emit_verilog_deep(sizes, modname, clk_div=1):
    """Emit the microsequencer for an arbitrary-DEPTH net (see gen_deep). `sizes` =
    [n_in, h1, ..., n_out]. Same one-smul/one-sadd datapath and parametric interface
    as emit_verilog; depth costs microcode steps (time), not area. A 2-entry-hidden
    2-layer `sizes` is equivalent to emit_verilog(n_in, n_hid, n_out)."""
    import random
    if len(sizes) < 2 or any(s < 1 for s in sizes):
        raise ValueError(f"emit_verilog_deep needs sizes=[n_in,...,n_out] all >=1; got {sizes}")
    L = len(sizes) - 1
    reg, steps = gen_deep(sizes)
    random.seed(3); initv = {}
    for l in range(1, L + 1):
        for j in range(sizes[l]):
            for k in range(sizes[l - 1]): initv[f"W{l}_{j}_{k}"] = round(random.uniform(-0.8, 0.8), 3)
    for l in range(1, L + 1):
        for j in range(sizes[l]): initv[f"b{l}_{j}"] = round(random.uniform(-0.5, 0.5), 3)
    return _emit_module(reg, steps, initv, sizes[0], sizes[-1], modname, clk_div)


def _emit_module(reg, steps, initv, n_in, n_out, modname, clk_div=1):
    """clk_div > 1 emits the SILICON-READY variant: the register file is forced to
    flip-flops (ram_style=registers -- distributed LUTRAM can't do the parallel weight
    init) and the sequencer steps once per clk_div cycles via a clock-enable, giving the
    shared combinational core ~clk_div x SETTLE cycles to settle (the open-source P&R
    can't express a multicycle constraint, so the deep muxed path is timing-relaxed;
    slowing the stepping is what lets it settle). Computed VALUES are identical to
    clk_div=1 -- only the timing changes -- so the bit-exact model check is unaffected.
    On real AX7203 silicon, clk_div=16 trains XOR 4/4, bit-exact to the model."""
    """Shared Verilog emitter: one GftSmul + one GftSadd + register file + case(pc)
    microcode ROM. Parametric ports (x{k}i / t{o}i / packed yout). Zero-inits the
    whole rf on reset (RTL == model, no x-propagation)."""
    N = len(reg); NP = len(steps)
    pcw = max(1, NP.bit_length()); L = []
    xports = ", ".join(f"input [31:0] x{k}i" for k in range(n_in))
    tports = ", ".join(f"input [31:0] t{o}i" for o in range(n_out))
    L.append(f"module {modname}(input clk, input rst, input start, {xports}, {tports},"
             f" output reg [{32*n_out-1}:0] yout, output reg done);")
    rs = '(* ram_style = "registers" *) ' if clk_div > 1 else ''
    L.append(f"  {rs}reg [31:0] rf [0:{N-1}];")
    L.append("  function [31:0] modf(input [31:0] v, input [2:0] m); reg neg0; reg [6:0] off;"
             " reg [8:0] mant; begin neg0=v[16]; case(m)")
    L.append("    3'd0:modf=v; 3'd1:modf=(v==0||neg0)?32'd0:v; 3'd2:modf=(v==0||neg0)?32'd0:32'd20480;")
    L.append("    3'd3:modf=(v==0)?32'd0:(v^32'h10000);")
    L.append("    3'd4:begin if(v==0)modf=0; else begin off=(v>>9)&7'h7f; mant=v&9'h1ff;"
             " if(off<3+1)modf=0; else modf=(v&32'h10000)^32'h10000|(((off-3)<<9)|mant); end end")
    L.append("    default:modf=v; endcase end endfunction")
    L.append(f"  reg [{pcw-1}:0] pc; reg [7:0] settle; reg running; reg op; reg [7:0] ai,bi,di; reg [2:0] am,bm; integer gi;")
    if clk_div >= 1:
        dcw = max(1, (clk_div - 1).bit_length())
        L.append(f"  reg [{dcw-1}:0] dc = 0; wire cen = (dc == {dcw}'d{clk_div-1});")
        L.append("  always @(posedge clk) dc <= dc + 1'b1;")
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
    L.append(f"    for(gi=0;gi<{N};gi=gi+1) rf[gi]<=32'd0;")  # zero scratch (match model's 0-init; no x-propagation)
    for name, val in initv.items(): L.append(f"    rf[{reg[name]}]<=32'd{enc(val)};")
    L.append("  end else begin done<=0;")
    loads = " ".join(f"rf[{reg[f'x{k}']}]<=x{k}i;" for k in range(n_in)) \
            + " " + " ".join(f"rf[{reg[f't{o}']}]<=t{o}i;" for o in range(n_out))
    L.append(f"    if(!running) begin if(start) begin {loads} pc<=0; settle<=SETTLE; running<=1; end end")
    ypack = "{" + ", ".join(f"rf[{reg[f'y{o}']}]" for o in range(n_out - 1, -1, -1)) + "}"
    step_gate = "else if (cen) begin" if clk_div > 1 else "else begin"  # step once per clk_div cycles
    L.append(f"    {step_gate} if(settle==0) begin rf[di] <= (op==2)? a_val : (op? add_r : mul_r);")
    L.append(f"      if(pc=={pcw}'d{NP-1}) begin running<=0; done<=1; yout<={ypack}; end"
             " else begin pc<=pc+1'b1; settle<=SETTLE; end")
    L.append("    end else settle<=settle-1; end end end")
    L.append("endmodule")
    return "\n".join(L)


def self_check():
    """Prove this file's sixteen asserts can fail, by planting faults they name.

    T124. The last gate in the tree with no negative control in any form. Its
    verdicts are `assert`s -- XOR trains to 4/4, the held-out classifier clears
    90%, the emitted Verilog carries the ports it claims -- and nothing showed
    that any of them could go red. Sixteen assertions and no evidence that they
    assert anything.

    Worth recording about the FORM: an assert delivers its verdict through a
    traceback, so exit 1 means both "XOR did not train" and "somebody typed a
    name wrong three lines up". Each case here demands the ASSERTION'S OWN
    MESSAGE, because the exit code cannot tell those apart -- which is the same
    reason every other control in this tree asserts text.

    The plants are copies of this file with one thing changed, run as whole
    programs. Nothing here can affect a live run: the child never sees
    --self-check, and the edits exist only in a temporary tree.
    """
    ok = True

    def spawned(label, edit, want_rc, expect, absent):
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            tools = os.path.join(td, "tools")
            os.makedirs(tools)
            src = open(os.path.abspath(__file__), encoding="utf-8").read()
            if edit:
                before = src
                src = edit(src)
                assert src != before, f"{label}: the plant changed nothing"
            me = os.path.join(tools, os.path.basename(__file__))
            open(me, "w", encoding="utf-8").write(src)
            r = subprocess.run([sys.executable, me], capture_output=True, text=True)
        out = r.stdout + r.stderr
        missing = [s for s in expect if s not in out]
        leaked = [s for s in absent if s in out]
        good = (r.returncode == want_rc) if want_rc == 0 else (r.returncode != 0)
        good = good and not missing and not leaked
        print(f"  {label:<44} " + (f"exit {r.returncode}, right assertion" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {'0' if want_rc == 0 else 'non-zero'})")
            if missing:
                print(f"       the assertion never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       said {out[-320:]!r}")

    # The clean direction first, or every case below passes for free on a file
    # that raises unconditionally.
    spawned("an unperturbed tree trains and emits", None, 0,
            ["generated XOR microcode trains 4/4", "emit_verilog: clk_div=16"],
            ["XOR self-test failed", "Traceback"])

    # The arithmetic the whole microsequencer rests on. A sign flip in the
    # shared multiplier makes the net stop converging, which is exactly what
    # `assert acc == 4` exists to notice.
    spawned("a broken multiplier stops XOR converging",
            lambda s: s.replace(
                "    return 0 if mag == 0 else (sgn << 16) | mag",
                "    return 0 if mag == 0 else ((1 - sgn) << 16) | mag", 1),
            1, ["XOR self-test failed"],
            ["emit_verilog: clk_div=16"])

    # And an emitter assertion, which is a different claim entirely: not that
    # the net learns, but that the Verilog says what this file says it says.
    # T124: the needle is ASSEMBLED, never written out. Spelled literally, its
    # first occurrence in this file would be THIS LINE -- so `str.replace(.., 1)`
    # edited the control's own source and left the assertion untouched, and the
    # case reported the gate as blind when nothing had been planted at all.
    #
    # check_duplicate_agreement.py carries a comment warning about exactly this,
    # written after the same thing happened there. I read that comment, wrote a
    # control, and reproduced the defect it describes in the same repository.
    port = "input [31:0] x" + "0i"
    spawned("a renamed port is caught by the emitter check",
            lambda s: s.replace(port, "input [31:0] xRENAMED", 1),
            1, ["AssertionError"],
            ["emit_verilog: clk_div=16"])

    print(f"  self-check: the training verdict and an emitter verdict both go red, "
          f"and a clean tree stays green = {ok}")
    return 0 if ok else 1


if __name__ == "__main__":
    if "--self-check" in sys.argv:
        sys.exit(self_check())
    for arch in [(2, 2, 1), (2, 3, 1), (2, 2, 2)]:
        reg, steps = gen(*arch)
        print(f"arch {arch}: {len(reg)} regs, {len(steps)} microcode steps")
    # self-test: generated (2,2,1) XOR net must train to 4/4
    import random
    reg, steps = gen(2, 2, 1)
    rf = [0] * len(reg)
    for k, v in {"W0_0": 0.9, "W0_1": 1.1, "W1_0": 1.1, "W1_1": 0.9,
                 "b0": 0.0, "b1": -1.0, "v0_0": 0.8, "v0_1": -1.7, "bo0": 0.0}.items():
        rf[reg[k]] = enc(v)
    corners = [(0, 0, 0), (1, 0, 1), (0, 1, 1), (1, 1, 0)]
    for _ in range(50):
        acc = 0
        for a, b, t in corners:
            rf[reg["x0"]] = enc(float(a)); rf[reg["x1"]] = enc(float(b)); rf[reg["t0"]] = enc(float(t))
            run(steps, rf); acc += int((dec(rf[reg["y0"]]) > 0.5)) == t
    assert acc == 4, f"XOR self-test failed: {acc}/4"
    print("self-test: generated XOR microcode trains 4/4 -- OK")
    # real-task self-test: (2,4,1) net on a NOISY nonlinear dataset, held-out generalization
    reg, steps = gen(2, 4, 1); rf = [0] * len(reg)
    random.seed(11)
    for j in range(4):
        for k in range(2): rf[reg[f"W{j}_{k}"]] = enc(round(random.uniform(-1, 1), 3))
        rf[reg[f"b{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    for j, v in enumerate([0.5, -0.5, 0.5, -0.5]): rf[reg[f"v0_{j}"]] = enc(v)
    random.seed(7)
    def _lab(a, b): return int((a > 0) != (b > 0))
    def _ds(n):
        d = []
        while len(d) < n:
            a = random.uniform(-1, 1); b = random.uniform(-1, 1)
            if abs(a) < 0.15 or abs(b) < 0.15: continue
            d.append((a, b, _lab(a, b)))
        return d
    tr_set, te_set = _ds(160), _ds(60)
    def _pred(a, b):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = 0
        run(steps, rf); y = dec(rf[reg["y0"]])
        for i in range(len(rf)): rf[i] = sav[i]
        return int(y > 0.5)
    for _ in range(60):
        for a, b, t in tr_set:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = enc(float(t)); run(steps, rf)
    te = sum(1 for a, b, t in te_set if _pred(a, b) == t)
    assert te >= int(0.9 * len(te_set)), f"real-task held-out too low: {te}/{len(te_set)}"
    print(f"self-test: (2,4,1) trains a noisy nonlinear task, held-out {te}/{len(te_set)} (>=90%) -- OK")
    v = emit_verilog(2, 3, 1, "bpseq231")
    assert "module bpseq231" in v and v.count("\n") > 40
    assert "for(gi=0;gi<" in v, "scratch registers must be zero-inited on reset"
    print("emit_verilog: (2,3,1) module generated -- OK (build with -nocarry, ~2.9M fasm)")
    # multi-output: (2,4,2) module emits one x/t port per input/output + packed yout
    v2 = emit_verilog(2, 4, 2, "bpseq242")
    assert "input [31:0] x0i, input [31:0] x1i" in v2
    assert "input [31:0] t0i, input [31:0] t1i" in v2
    assert "output reg [63:0] yout" in v2, "n_out=2 packs two 32-bit outputs"
    print("emit_verilog: (2,4,2) multi-output module generated -- OK (2 x/t ports, packed yout)")
    # multi-output LEARNING: (2,4,2) one-hot 2-class quadrant task, argmax over outputs
    reg, steps = gen(2, 4, 2); rf = [0] * len(reg)
    random.seed(5)
    for j in range(4):
        for k in range(2): rf[reg[f"W{j}_{k}"]] = enc(round(random.uniform(-1, 1), 3))
        rf[reg[f"b{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    for o in range(2):
        for j in range(4): rf[reg[f"v{o}_{j}"]] = enc(round(random.uniform(-1, 1), 3))
    random.seed(9)
    def _cls(a, b): return int((a > 0) != (b > 0))
    def _ds2(n):
        d = []
        while len(d) < n:
            a = random.uniform(-1, 1); b = random.uniform(-1, 1)
            if abs(a) < 0.15 or abs(b) < 0.15: continue
            d.append((a, b, _cls(a, b)))
        return d
    tr2, te2 = _ds2(160), _ds2(60)
    def _pred2(a, b):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b)
        rf[reg["t0"]] = 0; rf[reg["t1"]] = 0; run(steps, rf)
        ys = [dec(rf[reg[f"y{o}"]]) for o in range(2)]
        for i in range(len(rf)): rf[i] = sav[i]
        return 0 if ys[0] >= ys[1] else 1
    for _ in range(60):
        for a, b, c in tr2:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b)
            rf[reg["t0"]] = enc(1.0 if c == 0 else 0.0); rf[reg["t1"]] = enc(1.0 if c == 1 else 0.0)
            run(steps, rf)
    te = sum(1 for a, b, c in te2 if _pred2(a, b) == c)
    assert te >= int(0.9 * len(te2)), f"multi-output held-out too low: {te}/{len(te2)}"
    print(f"self-test: (2,4,2) multi-output one-hot classifier, held-out {te}/{len(te2)} (>=90%) -- OK")
    # DEEP: a 3-layer [2,4,3,1] net (arbitrary depth, backprop through 2 hidden
    # layers) learns the same noisy nonlinear task -- depth costs time, not area
    reg, steps = gen_deep([2, 4, 3, 1]); rf = [0] * len(reg)
    random.seed(3)
    for l in range(1, 4):
        for j in range([2, 4, 3, 1][l]):
            for k in range([2, 4, 3, 1][l - 1]): rf[reg[f"W{l}_{j}_{k}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
            rf[reg[f"b{l}_{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    random.seed(7)
    trd, ted = _ds(160), _ds(60)
    def _predd(a, b):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = 0
        run(steps, rf); y = dec(rf[reg["y0"]])
        for i in range(len(rf)): rf[i] = sav[i]
        return int(y > 0.5)
    for _ in range(60):
        for a, b, t in trd:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = enc(float(t)); run(steps, rf)
    te = sum(1 for a, b, t in ted if _predd(a, b) == t)
    assert te >= int(0.9 * len(ted)), f"deep [2,4,3,1] held-out too low: {te}/{len(ted)}"
    print(f"self-test: deep [2,4,3,1] (3-layer, 2 hidden) learns nonlinear task, held-out {te}/{len(ted)} (>=90%) -- OK")
    # deeper+wider: [2,5,3,1] (3-layer) generalises too -- the method scales, only the open
    # silicon flow's placement marginality does not (fixed by the Vivado closure kit).
    SZ = [2, 5, 3, 1]
    reg, steps = gen_deep(SZ); rf = [0] * len(reg)
    random.seed(3)
    for l in range(1, len(SZ)):
        for j in range(SZ[l]):
            for k in range(SZ[l - 1]): rf[reg[f"W{l}_{j}_{k}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
            rf[reg[f"b{l}_{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    random.seed(7); trw, tew = _ds(160), _ds(60)
    def _predw(a, b):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = 0
        run(steps, rf); y = dec(rf[reg["y0"]])
        for i in range(len(rf)): rf[i] = sav[i]
        return int(y > 0.5)
    for _ in range(60):
        for a, b, t in trw:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["t0"]] = enc(float(t)); run(steps, rf)
    te = sum(1 for a, b, t in tew if _predw(a, b) == t)
    assert te >= int(0.9 * len(tew)), f"deep [2,5,3,1] held-out too low: {te}/{len(tew)}"
    print(f"self-test: deep [2,5,3,1] (158 steps) learns nonlinear task, held-out {te}/{len(tew)} (>=90%) -- OK")
    # input-dimension scaling: a 3-INPUT net (3,5,1) learns a noisy 3-feature task (majority
    # of signs), proving the generator scales along inputs, not only depth/width.
    reg, steps = gen(3, 5, 1); rf = [0] * len(reg)
    random.seed(3)
    for j in range(5):
        for k in range(3): rf[reg[f"W{j}_{k}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
        rf[reg[f"b{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    for j in range(5): rf[reg[f"v0_{j}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
    def _maj(a, b, c): return int((int(a > 0) + int(b > 0) + int(c > 0)) >= 2)
    def _ds3(n, seed):
        random.seed(seed); d = []
        while len(d) < n:
            a, b, c = random.uniform(-1, 1), random.uniform(-1, 1), random.uniform(-1, 1)
            if min(abs(a), abs(b), abs(c)) < 0.15: continue
            d.append((a, b, c, _maj(a, b, c)))
        return d
    tr3, te3 = _ds3(200, 7), _ds3(60, 99)
    def _pred3(a, b, c):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["x2"]] = enc(c); rf[reg["t0"]] = 0
        run(steps, rf); y = dec(rf[reg["y0"]])
        for i in range(len(rf)): rf[i] = sav[i]
        return int(y > 0.5)
    for _ in range(80):
        for a, b, c, t in tr3:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b); rf[reg["x2"]] = enc(c); rf[reg["t0"]] = enc(float(t)); run(steps, rf)
    te = sum(1 for a, b, c, t in te3 if _pred3(a, b, c) == t)
    assert te >= int(0.9 * len(te3)), f"3-input (3,5,1) held-out too low: {te}/{len(te3)}"
    print(f"self-test: 3-input (3,5,1) learns a noisy 3-feature task, held-out {te}/{len(te3)} (>=90%) -- OK")
    # output-dimension scaling: a 3-CLASS argmax classifier (2,8,3) on angular sectors,
    # one-hot targets + argmax over three outputs -- proves scaling along outputs, not just
    # the 2-class case. Closes the scaling axes: inputs / depth / width / outputs.
    import math as _math
    reg, steps = gen(2, 8, 3); rf = [0] * len(reg)
    random.seed(3)
    for j in range(8):
        for k in range(2): rf[reg[f"W{j}_{k}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
        rf[reg[f"b{j}"]] = enc(round(random.uniform(-0.5, 0.5), 3))
    for o in range(3):
        for j in range(8): rf[reg[f"v{o}_{j}"]] = enc(round(random.uniform(-0.8, 0.8), 3))
    def _sector(a, b): return int(((_math.atan2(b, a) + _math.pi) / (2 * _math.pi)) * 3) % 3
    def _ds3c(n, seed):
        random.seed(seed); d = []
        while len(d) < n:
            a, b = random.uniform(-1, 1), random.uniform(-1, 1)
            if a * a + b * b < 0.1: continue
            d.append((a, b, _sector(a, b)))
        return d
    tr3c, te3c = _ds3c(240, 7), _ds3c(60, 99)
    def _pred3c(a, b):
        sav = rf[:]; rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b)
        for o in range(3): rf[reg[f"t{o}"]] = 0
        run(steps, rf); ys = [dec(rf[reg[f"y{o}"]]) for o in range(3)]
        for i in range(len(rf)): rf[i] = sav[i]
        return max(range(3), key=lambda o: ys[o])
    for _ in range(80):
        for a, b, c in tr3c:
            rf[reg["x0"]] = enc(a); rf[reg["x1"]] = enc(b)
            for o in range(3): rf[reg[f"t{o}"]] = enc(1.0 if o == c else 0.0)
            run(steps, rf)
    te = sum(1 for a, b, c in te3c if _pred3c(a, b) == c)
    assert te >= int(0.9 * len(te3c)), f"3-class (2,8,3) argmax held-out too low: {te}/{len(te3c)}"
    print(f"self-test: 3-class (2,8,3) argmax classifier learns angular sectors, held-out {te}/{len(te3c)} (>=90%) -- OK")
    vd = emit_verilog_deep([2, 4, 3, 1], "deep431")
    assert "module deep431" in vd and "for(gi=0;gi<" in vd
    print("emit_verilog_deep: [2,4,3,1] module generated -- OK")
    # silicon-ready variant: clk_div>1 adds ram_style=registers + a /N clock-enable
    # (the fix that trains XOR on real AX7203 silicon). Values are unchanged (only
    # timing), so the bit-exact model check is unaffected; default clk_div=1 is intact.
    vc = emit_verilog(2, 2, 1, "bpx", clk_div=16)
    assert 'ram_style = "registers"' in vc and "else if (cen)" in vc and "wire cen = (dc ==" in vc
    assert 'ram_style' not in emit_verilog(2, 2, 1, "bpx") and "else if (cen)" not in emit_verilog(2, 2, 1, "bpx")
    print("emit_verilog: clk_div=16 emits silicon-ready ram_style + /N clock-enable (default clk_div=1 intact) -- OK")
