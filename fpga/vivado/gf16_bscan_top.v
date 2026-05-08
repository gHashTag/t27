module gf16_bscan_top (
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

    wire bscan_tck, bscan_tdi, bscan_sel, bscan_capture, bscan_shift, bscan_update;
    reg user_tdo;

    BSCANE2 #(.JTAG_CHAIN(2)) bscan_inst (
        .CAPTURE(bscan_capture),
        .DRCK(),
        .RESET(),
        .RUNTEST(),
        .SEL(bscan_sel),
        .SHIFT(bscan_shift),
        .TCK(bscan_tck),
        .TDI(bscan_tdi),
        .TMS(),
        .UPDATE(bscan_update),
        .TDO(user_tdo)
    );

    reg [127:0] sr;
    reg [15:0] a0_r, a1_r, a2_r, a3_r;
    reg [15:0] b0_r, b1_r, b2_r, b3_r;
    reg [15:0] result_r;
    reg valid_r;

    wire [15:0] dot_result;
    gf16_dot4 u_dot4 (
        .a0(a0_r), .a1(a1_r), .a2(a2_r), .a3(a3_r),
        .b0(b0_r), .b1(b1_r), .b2(b2_r), .b3(b3_r),
        .result(dot_result)
    );

    always @(posedge bscan_tck) begin
        if (bscan_sel) begin
            if (bscan_capture) begin
                sr[15:0] <= result_r;
                sr[31:16] <= 16'h0000;
                sr[47:32] <= 16'h0000;
                sr[63:48] <= 16'h0000;
                sr[79:64] <= 16'h0000;
                sr[95:80] <= 16'h0000;
                sr[111:96] <= 16'h0000;
                sr[127:112] <= {15'h0000, valid_r};
            end else if (bscan_shift) begin
                sr <= {bscan_tdi, sr[127:1]};
            end else if (bscan_update) begin
                a0_r <= sr[15:0];
                a1_r <= sr[31:16];
                a2_r <= sr[47:32];
                a3_r <= sr[63:48];
                b0_r <= sr[79:64];
                b1_r <= sr[95:80];
                b2_r <= sr[111:96];
                b3_r <= sr[127:112];
                result_r <= dot_result;
                valid_r <= 1'b1;
            end
        end
    end

    assign user_tdo = sr[0];

    assign led_r23 = ~counter[20];
    assign led_t23 = ~(valid_r ^ counter[19]);

endmodule
