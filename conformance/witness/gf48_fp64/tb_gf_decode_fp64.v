// tb_gf_decode_fp64.v -- iverilog testbench for gf_decode_param_fp64 (gf48).
// -----------------------------------------------------------------------------
// Independent second RTL witness for the gf48 SW-bitexact promotion. Reads a
// vector file "vectors_gf48.txt" whose lines are two hex tokens:
//     <hex_gf48_12nibbles> <hex_fp64_16nibbles>
// (produced by gen_vectors_fp64.py from the golden Fraction oracle). Drives each
// gf48 code into the DUT and compares fp64_out against the golden binary64
// pattern. NaN comparison is payload-agnostic (any quiet NaN passes). Prints the
// canonical line the local-agent harness greps for:
//     HW RESULT: <pass>/<total> bit-exact (fails=<fails>)
//
// NOTE: this is a SIMULATION witness (iverilog), NOT on-silicon Tier-E. It is
// the required INDEPENDENT second decoder distinct from the SW encoder/oracle.
//
// Author: Vasilev (gHashTag), ORCID 0009-0008-4294-6159, admin@t27.ai.
// -----------------------------------------------------------------------------
`timescale 1ns/1ps
`default_nettype none

module tb_gf_decode_fp64;
    localparam integer N = 48;
    localparam integer MAXV = 300000;

    reg  [N-1:0]  gf_in;
    wire [63:0]   fp64_out;
    wire          is_nan_o, is_inf_o, is_zero_o, is_subnormal_o;

    // combinational DUT (OUT_REG=0): clk/rst_n unused but wired for form.
    reg clk = 1'b0;
    reg rst_n = 1'b1;

    gf_decode_param_fp64 #(
        .N(48), .E(18), .M(29), .BIAS(131071), .OUT_REG(0)
    ) dut (
        .clk(clk), .rst_n(rst_n), .gf_in(gf_in),
        .fp64_out(fp64_out),
        .is_nan_o(is_nan_o), .is_inf_o(is_inf_o),
        .is_zero_o(is_zero_o), .is_subnormal_o(is_subnormal_o)
    );

    reg [N-1:0]  gf_vec  [0:MAXV-1];
    reg [63:0]   exp_vec [0:MAXV-1];
    integer n_vec;
    integer i;
    integer pass, fails;

    function is_qnan_bits;
        input [63:0] w;
        begin
            is_qnan_bits = (w[62:52] == 11'h7FF) && (w[51:0] != 52'b0);
        end
    endfunction

    // vector file loader: two hex tokens per line.
    integer fd, r;
    reg [N-1:0]  gf_tok;
    reg [63:0]   fp_tok;

    initial begin
        n_vec = 0;
        fd = $fopen("vectors_gf48.txt", "r");
        if (fd == 0) begin
            $display("ERROR: cannot open vectors_gf48.txt");
            $finish;
        end
        r = $fscanf(fd, "%h %h\n", gf_tok, fp_tok);
        while (r == 2) begin
            gf_vec[n_vec]  = gf_tok;
            exp_vec[n_vec] = fp_tok;
            n_vec = n_vec + 1;
            r = $fscanf(fd, "%h %h\n", gf_tok, fp_tok);
        end
        $fclose(fd);

        pass  = 0;
        fails = 0;
        for (i = 0; i < n_vec; i = i + 1) begin
            gf_in = gf_vec[i];
            #1;
            if (is_qnan_bits(exp_vec[i])) begin
                if (is_qnan_bits(fp64_out)) pass = pass + 1;
                else begin
                    fails = fails + 1;
                    if (fails <= 20)
                        $display("  MISMATCH gf=0x%012h got=0x%016h exp=0x%016h (NaN)",
                                 gf_vec[i], fp64_out, exp_vec[i]);
                end
            end else begin
                if (fp64_out === exp_vec[i]) pass = pass + 1;
                else begin
                    fails = fails + 1;
                    if (fails <= 20)
                        $display("  MISMATCH gf=0x%012h got=0x%016h exp=0x%016h",
                                 gf_vec[i], fp64_out, exp_vec[i]);
                end
            end
        end

        $display("HW RESULT: %0d/%0d bit-exact (fails=%0d)", pass, n_vec, fails);
        if (fails == 0) $display("gf48 FP64 iverilog witness: PASS");
        else            $display("gf48 FP64 iverilog witness: FAIL");
        $finish;
    end
endmodule
`default_nettype wire
