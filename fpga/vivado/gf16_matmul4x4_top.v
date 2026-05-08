module gf16_matmul4x4_top (
    output wire led_r23,
    output wire led_t23
);

    (* KEEP = "TRUE" *) wire osc;
    (* KEEP = "TRUE" *) wire chain [19:0];
    reg [22:0] counter = 0;

    assign chain[0] = ~chain[19];
    genvar i;
    generate
        for (i = 1; i < 20; i = i + 1) begin : inv_chain
            (* KEEP = "TRUE" *) LUT1 #(.INIT(2'b01)) inv (
                .I0(chain[i-1]),
                .O(chain[i])
            );
        end
    endgenerate
    assign osc = chain[19];

    always @(posedge osc) begin
        counter <= counter + 1;
    end

    wire [15:0] c00, c01, c02, c03;
    wire [15:0] c10, c11, c12, c13;
    wire [15:0] c20, c21, c22, c23;
    wire [15:0] c30, c31, c32, c33;

    gf16_matmul4x4 u_matmul (
        .a00(16'h3E00), .a01(16'h4000), .a02(16'h4100), .a03(16'h4200),
        .a10(16'h4300), .a11(16'h4380), .a12(16'h4400), .a13(16'h4440),
        .a20(16'h4480), .a21(16'h44C0), .a22(16'h4500), .a23(16'h4520),
        .a30(16'h4540), .a31(16'h4560), .a32(16'h4580), .a33(16'h45A0),
        .b00(16'h3E00), .b01(16'h0000), .b02(16'h0000), .b03(16'h0000),
        .b10(16'h0000), .b11(16'h3E00), .b12(16'h0000), .b13(16'h0000),
        .b20(16'h0000), .b21(16'h0000), .b22(16'h3E00), .b23(16'h0000),
        .b30(16'h0000), .b31(16'h0000), .b32(16'h0000), .b33(16'h3E00),
        .c00(c00), .c01(c01), .c02(c02), .c03(c03),
        .c10(c10), .c11(c11), .c12(c12), .c13(c13),
        .c20(c20), .c21(c21), .c22(c22), .c23(c23),
        .c30(c30), .c31(c31), .c32(c32), .c33(c33)
    );

    wire diag_ok = (c00 == 16'h3E00) && (c11 == 16'h3E00) &&
                   (c22 == 16'h3E00) && (c33 == 16'h3E00);
    wire off_zero = (c01 == 16'h0000) && (c02 == 16'h0000) &&
                    (c03 == 16'h0000) && (c10 == 16'h0000);

    assign led_r23 = ~counter[20];
    assign led_t23 = ~(diag_ok & off_zero ^ counter[19]);

endmodule
