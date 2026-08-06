// gf_decode_param_fp64.v  (matches the bit-exact-proven rtl_bit_model_fp64.py)
// -----------------------------------------------------------------------------
// Parametric GoldenFloat GF{N} decode module with IEEE binary64 output. This is
// the FP64-target sibling of fpga/openxc7-synth/gf_decode_param.v (FP32 target,
// Phase-A gf4..gf32). It exists to reach the wide GF rungs whose mantissa does
// NOT fit into FP32 (M>23) but DOES fit into FP64 (M<=52). gf48 (S1 E18 M29,
// BIAS=131071) is the first such rung: M=29 <= 52, so its full normal range
// decodes into binary64 WITHOUT mantissa rounding (pure zero-pad widen); only
// the deep-underflow FP64-subnormal path (true_exp < -1022) needs guard/round/
// sticky against the FP64 subnormal LSB 2^-1074.
//
// Decode law (5 classes, HAS_INF semantics) -- identical to the FP32 module:
//   exp == EXP_MAX, mant == 0   -> +-Inf
//   exp == EXP_MAX, mant != 0   -> quiet NaN
//   exp == 0,       mant == 0   -> +-0
//   exp == 0,       mant != 0   -> subnormal: (-1)^s * mant/2^M * 2^(1-BIAS)
//   otherwise (normal)          -> (-1)^s * (1+mant/2^M) * 2^(exp-BIAS)
//
// Output: IEEE-754 binary64. Faithful for GF formats with M<=52 (gf48, gf64).
// For M>52 (gf96/128/256/512/1024) binary64 CANNOT hold the mantissa -- this
// module MUST NOT be instantiated for those, and MUST NOT be claimed as FP64
// decode HW for them (they remain SW-only conformance, [open hypothesis HW]).
//
// Both fixed-width fixes from the FP32 iverilog witness (04.07) are carried over:
//   FIX #1 (widen-before-shift): zero-extend pack_frac to the full result width
//           BEFORE the left shift, so no high significant bits are truncated by
//           Verilog's "shift result width == left-operand width" rule.
//   FIX #2 (OOB-safe wide sub_shifted): sub_shifted is declared [63:0] (not
//           [M:0]) so the read of sub_shifted[51:0] on the FP64-subnormal path
//           never falls out of bounds (returns valid zero-extended bits).
//
// This file is a Phase-B design deliverable for the LOCAL AGENT to run under
// iverilog on trinity-fpga (the sandbox has NO iverilog/yosys/vivado). Its
// semantics are proven ahead of simulation by rtl_bit_model_fp64.py (a bit-exact
// fixed-width Python model of THIS exact datapath) checked against the golden
// Fraction oracle gf48_decode_ref.py: 224255/224255 bit-exact, fails=0
// (5-class + boundary + full-exponent stress sweep + deep-underflow + 200k
// random). The iverilog run is the required INDEPENDENT second RTL witness;
// synthesis/PnR/flash on AX7203 = [TREBUET DEISTVIYA POLZOVATELYA] and is NOT
// claimed here (encoding != compute != FPGA-HW).
//
// Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
// -----------------------------------------------------------------------------

module gf_decode_param_fp64 #(
    parameter integer N        = 48,      // total GF width (1 + E + M)
    parameter integer E        = 18,      // GF exponent width
    parameter integer M        = 29,      // GF mantissa width (MUST be <= 52)
    parameter integer BIAS     = 131071,  // GF exponent bias
    parameter integer OUT_REG  = 0        // 0 = combinational, 1 = registered
) (
    input  wire                 clk,
    input  wire                 rst_n,
    input  wire [N-1:0]         gf_in,
    output wire [63:0]          fp64_out,
    output wire                 is_nan_o,
    output wire                 is_inf_o,
    output wire                 is_zero_o,
    output wire                 is_subnormal_o
);
    // synthesis translate_off
    initial begin
        if (N !== (1 + E + M)) begin
            $display("ERROR gf_decode_param_fp64: N != 1+E+M (N=%0d E=%0d M=%0d)", N, E, M);
            $finish;
        end
        if (M > 52) begin
            $display("ERROR gf_decode_param_fp64: M=%0d > 52 -- binary64 cannot hold the mantissa (extended rung, SW-only). Do NOT instantiate.", M);
            $finish;
        end
    end
    // synthesis translate_on

    localparam [E-1:0] EXP_MAX = {E{1'b1}};
    localparam integer FP64_EBIAS        = 1023;
    localparam integer FP64_MANT         = 52;
    localparam integer FP64_MIN_NORM_EXP = -1022;  // smallest true exp for FP64 normal
    localparam integer FP64_SUB_LSB_EXP  = -1074;  // exponent of FP64 subnormal LSB (2^-1074)

    // Signed working width for exponent math. gf48 BIAS=131071, exp in
    // [0, 262143] -> true_exp in [-131071, 131072]; plus FP64 constants
    // (~-1074). 40 bits of signed headroom is ample.
    localparam integer EXP_CALC_W = 40;

    // ---- field extraction ----
    wire               sign_in = gf_in[N-1];
    wire [E-1:0]       exp_in  = gf_in[N-2 -: E];
    wire [M-1:0]       mant_in = gf_in[M-1:0];

    wire is_exp_zero  = (exp_in == {E{1'b0}});
    wire is_exp_max   = (exp_in == EXP_MAX);
    wire is_mant_zero = (mant_in == {M{1'b0}});

    wire cls_zero      = is_exp_zero  &&  is_mant_zero;
    wire cls_subnormal = is_exp_zero  && !is_mant_zero;
    wire cls_inf       = is_exp_max   &&  is_mant_zero;
    wire cls_nan       = is_exp_max   && !is_mant_zero;
    wire cls_normal    = !is_exp_zero && !is_exp_max;

    // ---- leading-zero-count for GF-subnormal renormalization ----
    function integer clz_m;
        input [M-1:0] v;
        integer i;
        begin
            clz_m = M;
            for (i = 0; i < M; i = i + 1) begin
                if (v[M-1-i] && (clz_m == M))
                    clz_m = i;
            end
        end
    endfunction

    wire signed [31:0] lzc_s = clz_m(mant_in);

    wire signed [EXP_CALC_W-1:0] sub_true_exp =
        ($signed(1) - BIAS) - (lzc_s + 32'sd1);
    wire [M-1:0] sub_frac_bits = (mant_in << (lzc_s[7:0] + 8'd1));

    wire signed [EXP_CALC_W-1:0] norm_true_exp =
        $signed({1'b0, exp_in}) - BIAS;

    wire signed [EXP_CALC_W-1:0] pack_true_exp = cls_subnormal ? sub_true_exp : norm_true_exp;
    wire [M-1:0]                 pack_frac     = cls_subnormal ? sub_frac_bits : mant_in;

    // -------------------------------------------------------------------
    // Attempt 1: FP64 NORMAL packer. For M<=52 this is a pure zero-pad widen
    // (no mantissa rounding). FIX #1 (widen-before-shift) applied.
    // -------------------------------------------------------------------
    localparam integer WIDE = (M > FP64_MANT) ? M : FP64_MANT;
    wire [WIDE:0] norm_widen_result;
    generate
        if (M <= FP64_MANT) begin : g_widen
            wire [WIDE:0] pf_wide = { {(WIDE-M+1){1'b0}}, pack_frac };
            assign norm_widen_result = pf_wide << (FP64_MANT - M);
        end else begin : g_narrow
            // Unreachable for M<=52 (guarded by initial $finish); kept for form.
            wire [M-FP64_MANT-1:0] lost_bits = pack_frac[M-FP64_MANT-1:0];
            wire                   g_bit     = pack_frac[M-FP64_MANT-1];
            wire                   s_bit     = |pack_frac[M-FP64_MANT-2:0];
            wire [FP64_MANT-1:0]   trunc     = pack_frac[M-1 -: FP64_MANT];
            wire round_up = g_bit && (s_bit || trunc[0]);
            wire [FP64_MANT:0] rounded = {1'b0, trunc} + (round_up ? 1'b1 : 1'b0);
            assign norm_widen_result = { {(WIDE-FP64_MANT){1'b0}}, rounded };
        end
    endgenerate
    wire        norm_carry  = norm_widen_result[FP64_MANT];
    wire [51:0] norm_mant52 = norm_widen_result[51:0];
    wire signed [EXP_CALC_W-1:0] norm_exp_final = pack_true_exp + norm_carry + FP64_EBIAS;

    wire is_fp64_normal_candidate = (pack_true_exp >= FP64_MIN_NORM_EXP);
    wire norm_overflow  = is_fp64_normal_candidate && (norm_exp_final >= 2047);
    wire norm_takes_normal_path = is_fp64_normal_candidate && !norm_overflow && (norm_exp_final >= 1);
    wire signed [EXP_CALC_W-1:0] corrected_true_exp = pack_true_exp + norm_carry;

    // -------------------------------------------------------------------
    // Attempt 2: FP64 SUBNORMAL packer (gradual underflow, RNE+sticky).
    // full_sig = implicit '1' + pack_frac (M frac bits); exact value =
    // full_sig * 2^(eff_true_exp - M). Express in units of 2^-1074:
    //   shift = FP64_SUB_LSB_EXP - (eff_true_exp - M)   (>=1 in this domain)
    // FIX #2 (OOB-safe): sub_shifted declared [63:0].
    // -------------------------------------------------------------------
    wire signed [EXP_CALC_W-1:0] eff_true_exp_for_sub =
        is_fp64_normal_candidate ? corrected_true_exp : pack_true_exp;
    wire [M:0] full_sig = {1'b1, pack_frac};
    localparam integer WIDTH_FULL = M + 1;
    wire signed [EXP_CALC_W-1:0] shift_s =
        FP64_SUB_LSB_EXP - (eff_true_exp_for_sub - M);

    // Defensive clamp: valid rounding window is [1, WIDTH_FULL+1]; beyond that
    // the whole significand is below the sticky window -> flush to zero.
    localparam integer MAXSH = WIDTH_FULL + 2;
    wire [31:0] shift_clamped = (shift_s < 0) ? 32'd0 :
                                (shift_s > MAXSH) ? MAXSH[31:0] : shift_s[31:0];

    wire [63:0] sub_shifted = (shift_s <= 0)          ? (full_sig << (-shift_s)) :
                              (shift_clamped >= MAXSH) ? 64'd0 :
                                                         (full_sig >> shift_clamped);

    // guard = bit just below the retained LSB; sticky = OR of everything below guard.
    wire        sub_guard = (shift_clamped >= 1 && shift_clamped <= WIDTH_FULL)
                              ? full_sig[shift_clamped-1] : 1'b0;
    wire [M:0]  sub_sticky_mask = (shift_clamped >= 2)
                              ? (({(M+1){1'b1}}) >> (WIDTH_FULL - (shift_clamped-1)))
                              : {(M+1){1'b0}};
    wire        sub_sticky = (shift_clamped >= 2) ? (|(full_sig & sub_sticky_mask)) : 1'b0;

    wire [53:0] sub_mant_pre = {2'b0, sub_shifted[51:0]};
    wire        sub_round_up = sub_guard && (sub_sticky || sub_shifted[0]);
    wire [53:0] sub_mant_rounded = sub_mant_pre + (sub_round_up ? 54'd1 : 54'd0);
    wire        sub_carry_to_normal = sub_mant_rounded[52];
    wire [51:0] sub_mant52 = sub_mant_rounded[51:0];

    // -------------------------------------------------------------------
    // Result composition
    // -------------------------------------------------------------------
    reg [63:0] fp64_r;
    localparam [63:0] FP64_QNAN    = 64'h7FF8000000000001;
    localparam [63:0] FP64_POS_INF = 64'h7FF0000000000000;
    localparam [63:0] FP64_NEG_INF = 64'hFFF0000000000000;

    always @(*) begin
        fp64_r = 64'h0000000000000000;
        if (cls_nan) begin
            fp64_r = FP64_QNAN;
        end else if (cls_inf) begin
            fp64_r = sign_in ? FP64_NEG_INF : FP64_POS_INF;
        end else if (cls_zero) begin
            fp64_r = {sign_in, 63'b0};
        end else if (norm_overflow) begin
            fp64_r = sign_in ? FP64_NEG_INF : FP64_POS_INF;
        end else if (norm_takes_normal_path) begin
            fp64_r = {sign_in, norm_exp_final[10:0], norm_mant52};
        end else begin
            if (sub_carry_to_normal) begin
                fp64_r = {sign_in, 11'd1, 52'b0}; // rounded up to smallest FP64 normal
            end else begin
                fp64_r = {sign_in, 11'b0, sub_mant52};
            end
        end
    end

    generate
        if (OUT_REG != 0) begin : g_reg
            reg [63:0] fp64_q;
            reg        nan_q, inf_q, zero_q, sub_q;
            always @(posedge clk) begin
                if (!rst_n) begin
                    fp64_q <= 64'b0;
                    nan_q  <= 1'b0; inf_q  <= 1'b0; zero_q <= 1'b0; sub_q  <= 1'b0;
                end else begin
                    fp64_q <= fp64_r;
                    nan_q  <= cls_nan; inf_q  <= cls_inf;
                    zero_q <= cls_zero; sub_q  <= cls_subnormal;
                end
            end
            assign fp64_out       = fp64_q;
            assign is_nan_o       = nan_q;
            assign is_inf_o       = inf_q;
            assign is_zero_o      = zero_q;
            assign is_subnormal_o = sub_q;
        end else begin : g_comb
            assign fp64_out       = fp64_r;
            assign is_nan_o       = cls_nan;
            assign is_inf_o       = cls_inf;
            assign is_zero_o      = cls_zero;
            assign is_subnormal_o = cls_subnormal;
        end
    endgenerate

endmodule
