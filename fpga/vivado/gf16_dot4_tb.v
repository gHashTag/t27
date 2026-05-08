module gf16_dot4_tb;

    reg  [15:0] a0, a1, a2, a3;
    reg  [15:0] b0, b1, b2, b3;
    wire [15:0] result;

    gf16_dot4 uut (.*);

    function real decode_gf16;
        input [15:0] v;
        reg sign;
        reg [5:0] exp;
        reg [8:0] mant;
        real r;
        integer i;
        begin
            sign = v[15];
            exp  = v[14:9];
            mant = v[8:0];
            if (exp == 6'd0 && mant == 9'd0)
                r = 0.0;
            else if (exp == 6'd63 && mant == 9'd0)
                r = 1.0e30;
            else if (exp == 6'd63)
                r = 0.0;
            else begin
                r = 1.0 + mant / 512.0;
                if (exp >= 31) begin
                    for (i = 0; i < (exp - 31); i = i + 1)
                        r = r * 2.0;
                end else begin
                    for (i = 0; i < (31 - exp); i = i + 1)
                        r = r / 2.0;
                end
            end
            if (sign) r = -r;
            decode_gf16 = r;
        end
    endfunction

    function [15:0] encode_gf16;
        input real v;
        reg sign;
        reg [5:0] exp;
        reg [8:0] mant;
        real abs_v;
        real frac;
        integer shifted;
        begin
            if (v == 0.0) begin
                encode_gf16 = (v < 0) ? 16'h8000 : 16'h0000;
            end else begin
                sign = (v < 0) ? 1'b1 : 1'b0;
                abs_v = (v < 0) ? -v : v;
                exp = 31;
                while (abs_v >= 2.0 && exp < 62) begin
                    abs_v = abs_v / 2.0;
                    exp = exp + 1;
                end
                while (abs_v < 1.0 && exp > 1) begin
                    abs_v = abs_v * 2.0;
                    exp = exp - 1;
                end
                frac = abs_v - 1.0;
                shifted = frac * 512.0 + 0.5;
                if (shifted >= 512) shifted = 511;
                mant = shifted[8:0];
                encode_gf16 = {sign, exp, mant};
            end
        end
    endfunction

    integer pass_count, fail_count;

    task check;
        input real expected;
        input [255:0] label;
        real got_r;
        real diff;
        real abs_exp;
        begin
            got_r = decode_gf16(result);
            abs_exp = (expected < 0.0) ? -expected : expected;
            if (abs_exp < 0.001) abs_exp = 0.001;
            diff = (got_r > expected) ? (got_r - expected) : (expected - got_r);
            if (expected == 0.0 && got_r == 0.0) begin
                pass_count = pass_count + 1;
            end else if (diff < 0.1 * abs_exp + 0.1) begin
                pass_count = pass_count + 1;
            end else begin
                fail_count = fail_count + 1;
                $display("FAIL %0s: result=%h expected=%f got=%f", label, result, expected, got_r);
            end
        end
    endtask

    real r_expected;

    initial begin
        pass_count = 0;
        fail_count = 0;

        $display("=== GF16 Dot4 Tests ===");

        // [1,1,1,1] . [1,1,1,1] = 4
        a0 = 16'h3E00; a1 = 16'h3E00; a2 = 16'h3E00; a3 = 16'h3E00;
        b0 = 16'h3E00; b1 = 16'h3E00; b2 = 16'h3E00; b3 = 16'h3E00;
        #10 check(4.0, "ones.dot4");

        // [1,2,3,4] . [1,2,3,4] = 1+4+9+16 = 30
        a0 = 16'h3E00; a1 = 16'h4000; a2 = 16'h4100; a3 = 16'h4200;
        b0 = 16'h3E00; b1 = 16'h4000; b2 = 16'h4100; b3 = 16'h4200;
        #10 check(30.0, "1234.dot4");

        // [0.5, 0.5, 0.5, 0.5] . [2, 2, 2, 2] = 4
        a0 = 16'h3C00; a1 = 16'h3C00; a2 = 16'h3C00; a3 = 16'h3C00;
        b0 = 16'h4000; b1 = 16'h4000; b2 = 16'h4000; b3 = 16'h4000;
        #10 check(4.0, "0.5x2.dot4");

        // [0, 0, 0, 0] . [1, 1, 1, 1] = 0
        a0 = 16'h0000; a1 = 16'h0000; a2 = 16'h0000; a3 = 16'h0000;
        b0 = 16'h3E00; b1 = 16'h3E00; b2 = 16'h3E00; b3 = 16'h3E00;
        #10 check(0.0, "zeros.dot4");

        // [-1, 1, -1, 1] . [1, 1, 1, 1] = 0
        a0 = 16'hBE00; a1 = 16'h3E00; a2 = 16'hBE00; a3 = 16'h3E00;
        b0 = 16'h3E00; b1 = 16'h3E00; b2 = 16'h3E00; b3 = 16'h3E00;
        #10 check(0.0, "neg_pos.dot4");

        // [100, 200, 50, 25] . [1, 2, 3, 4] = 100+400+150+100 = 750
        r_expected = 750.0;
        a0 = encode_gf16(100.0); a1 = encode_gf16(200.0);
        a2 = encode_gf16(50.0);  a3 = encode_gf16(25.0);
        b0 = encode_gf16(1.0);   b1 = encode_gf16(2.0);
        b2 = encode_gf16(3.0);   b3 = encode_gf16(4.0);
        #10 check(r_expected, "large.dot4");

        $display("Results: %0d pass, %0d fail", pass_count, fail_count);
        if (fail_count > 0) $display("SOME TESTS FAILED");
        else $display("ALL TESTS PASSED");
        $finish;
    end

endmodule
