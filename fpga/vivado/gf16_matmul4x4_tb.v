module gf16_matmul4x4_tb;

    reg  [15:0] a00, a01, a02, a03;
    reg  [15:0] a10, a11, a12, a13;
    reg  [15:0] a20, a21, a22, a23;
    reg  [15:0] a30, a31, a32, a33;
    reg  [15:0] b00, b01, b02, b03;
    reg  [15:0] b10, b11, b12, b13;
    reg  [15:0] b20, b21, b22, b23;
    reg  [15:0] b30, b31, b32, b33;
    wire [15:0] c00, c01, c02, c03;
    wire [15:0] c10, c11, c12, c13;
    wire [15:0] c20, c21, c22, c23;
    wire [15:0] c30, c31, c32, c33;

    gf16_matmul4x4 uut (.*);

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
    integer i, j;

    reg [15:0] mat_a [0:3][0:3];
    reg [15:0] mat_b [0:3][0:3];
    reg [15:0] mat_c [0:3][0:3];

    task load_matrices;
        begin
            a00 = mat_a[0][0]; a01 = mat_a[0][1]; a02 = mat_a[0][2]; a03 = mat_a[0][3];
            a10 = mat_a[1][0]; a11 = mat_a[1][1]; a12 = mat_a[1][2]; a13 = mat_a[1][3];
            a20 = mat_a[2][0]; a21 = mat_a[2][1]; a22 = mat_a[2][2]; a23 = mat_a[2][3];
            a30 = mat_a[3][0]; a31 = mat_a[3][1]; a32 = mat_a[3][2]; a33 = mat_a[3][3];
            b00 = mat_b[0][0]; b01 = mat_b[0][1]; b02 = mat_b[0][2]; b03 = mat_b[0][3];
            b10 = mat_b[1][0]; b11 = mat_b[1][1]; b12 = mat_b[1][2]; b13 = mat_b[1][3];
            b20 = mat_b[2][0]; b21 = mat_b[2][1]; b22 = mat_b[2][2]; b23 = mat_b[2][3];
            b30 = mat_b[3][0]; b31 = mat_b[3][1]; b32 = mat_b[3][2]; b33 = mat_b[3][3];
        end
    endtask

    task capture_result;
        begin
            mat_c[0][0] = c00; mat_c[0][1] = c01; mat_c[0][2] = c02; mat_c[0][3] = c03;
            mat_c[1][0] = c10; mat_c[1][1] = c11; mat_c[1][2] = c12; mat_c[1][3] = c13;
            mat_c[2][0] = c20; mat_c[2][1] = c21; mat_c[2][2] = c22; mat_c[2][3] = c23;
            mat_c[3][0] = c30; mat_c[3][1] = c31; mat_c[3][2] = c32; mat_c[3][3] = c33;
        end
    endtask

    task check_element;
        input integer row;
        input integer col;
        input real expected;
        input [255:0] label;
        real got_r;
        real diff;
        real abs_exp;
        begin
            got_r = decode_gf16(mat_c[row][col]);
            abs_exp = (expected < 0.0) ? -expected : expected;
            if (abs_exp < 0.001) abs_exp = 0.001;
            diff = (got_r > expected) ? (got_r - expected) : (expected - got_r);
            if (expected == 0.0 && got_r == 0.0) begin
                pass_count = pass_count + 1;
            end else if (diff < 0.1 * abs_exp + 0.1) begin
                pass_count = pass_count + 1;
            end else begin
                fail_count = fail_count + 1;
                $display("FAIL %0s [%0d][%0d]: expected=%f got=%f", label, row, col, expected, got_r);
            end
        end
    endtask

    initial begin
        pass_count = 0;
        fail_count = 0;

        $display("=== GF16 Matmul4x4 Tests ===");

        // Test 1: Identity * Identity = Identity
        for (i = 0; i < 4; i = i + 1)
            for (j = 0; j < 4; j = j + 1) begin
                mat_a[i][j] = (i == j) ? 16'h3E00 : 16'h0000;
                mat_b[i][j] = (i == j) ? 16'h3E00 : 16'h0000;
            end
        load_matrices();
        #10;
        capture_result();
        for (i = 0; i < 4; i = i + 1)
            for (j = 0; j < 4; j = j + 1)
                check_element(i, j, (i == j) ? 1.0 : 0.0, "I*I");

        // Test 2: A * I = A, A = [[1,2,3,4],[5,6,7,8],[9,10,11,12],[13,14,15,16]]
        for (i = 0; i < 4; i = i + 1)
            for (j = 0; j < 4; j = j + 1) begin
                mat_a[i][j] = encode_gf16(i * 4 + j + 1);
                mat_b[i][j] = (i == j) ? 16'h3E00 : 16'h0000;
            end
        load_matrices();
        #10;
        capture_result();
        for (i = 0; i < 4; i = i + 1)
            for (j = 0; j < 4; j = j + 1)
                check_element(i, j, i * 4 + j + 1.0, "A*I");

        // Test 3: A * A, C[0][0]=90, C[0][1]=100, C[3][3]=600
        for (i = 0; i < 4; i = i + 1)
            for (j = 0; j < 4; j = j + 1) begin
                mat_a[i][j] = encode_gf16(i * 4 + j + 1);
                mat_b[i][j] = encode_gf16(i * 4 + j + 1);
            end
        load_matrices();
        #10;
        capture_result();
        check_element(0, 0, 90.0,  "A*A");
        check_element(0, 1, 100.0, "A*A");
        check_element(3, 3, 600.0, "A*A");

        $display("Results: %0d pass, %0d fail", pass_count, fail_count);
        if (fail_count > 0) $display("SOME TESTS FAILED");
        else $display("ALL TESTS PASSED");
        $finish;
    end

endmodule
