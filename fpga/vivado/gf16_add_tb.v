module gf16_add_tb;

    reg  [15:0] a, b;
    wire [15:0] result;

    gf16_add uut (.a(a), .b(b), .result(result));

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
            end else if (diff < 0.1 * abs_exp + 0.01) begin
                pass_count = pass_count + 1;
            end else begin
                fail_count = fail_count + 1;
                $display("FAIL %0s: a=%h b=%h result=%h expected=%f got=%f", label, a, b, result, expected, got_r);
            end
        end
    endtask

    initial begin
        pass_count = 0;
        fail_count = 0;

        $display("=== GF16 Add Tests ===");

        // 1.0 + 1.0 = 2.0
        a = 16'h3E00; b = 16'h3E00;
        #10 check(2.0, "1+1");

        // 1.0 + 0 = 1.0
        a = 16'h3E00; b = 16'h0000;
        #10 check(1.0, "1+0");

        // 0 + 1.0 = 1.0
        a = 16'h0000; b = 16'h3E00;
        #10 check(1.0, "0+1");

        // 1.0 + 2.0 = 3.0
        a = 16'h3E00; b = 16'h4000;
        #10 check(3.0, "1+2");

        // 3.0 - 1.0 = 2.0
        a = 16'h4100; b = 16'hBE00;
        #10 check(2.0, "3-1");

        // 1.0 - 1.0 = 0
        a = 16'h3E00; b = 16'hBE00;
        #10 check(0.0, "1-1");

        // -1.0 + 2.0 = 1.0
        a = 16'hBE00; b = 16'h4000;
        #10 check(1.0, "-1+2");

        // 0.5 + 0.5 = 1.0
        a = 16'h3C00; b = 16'h3C00;
        #10 check(1.0, "0.5+0.5");

        // 0.5 + 0.25 = 0.75
        a = 16'h3C00; b = 16'h3A00;
        #10 check(0.75, "0.5+0.25");

        // 1.5 + 1.5 = 3.0
        a = 16'h3F00; b = 16'h3F00;
        #10 check(3.0, "1.5+1.5");

        // 100 + 200 = 300
        a = 16'h4B20; b = 16'h4D20;
        #10 check(300.0, "100+200");

        // NaN + 1 = NaN
        a = 16'hFE01; b = 16'h3E00;
        #10;
        if (result == 16'hFE01) pass_count = pass_count + 1;
        else begin fail_count = fail_count + 1; $display("FAIL NaN+1: got %h", result); end

        // Inf + 1 = Inf
        a = 16'h7E00; b = 16'h3E00;
        #10;
        if (result == 16'h7E00) pass_count = pass_count + 1;
        else begin fail_count = fail_count + 1; $display("FAIL Inf+1: got %h", result); end

        // Inf + (-Inf) = NaN
        a = 16'h7E00; b = 16'hFE00;
        #10;
        if (result == 16'hFE01) pass_count = pass_count + 1;
        else begin fail_count = fail_count + 1; $display("FAIL Inf+-Inf: got %h", result); end

        $display("Results: %0d pass, %0d fail", pass_count, fail_count);
        if (fail_count > 0) $display("SOME TESTS FAILED");
        else $display("ALL TESTS PASSED");
        $finish;
    end

endmodule
