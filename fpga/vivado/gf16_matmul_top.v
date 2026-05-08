module gf16_matmul_top (
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

    wire [15:0] dot_result;

    gf16_dot4 u_dot (
        .a0(16'h3E00),
        .a1(16'h4000),
        .a2(16'h4100),
        .a3(16'h4200),
        .b0(16'h3E00),
        .b1(16'h4000),
        .b2(16'h4100),
        .b3(16'h4200),
        .result(dot_result)
    );

    wire dot_ok = (dot_result == 16'h47C0);

    assign led_r23 = ~counter[20];
    assign led_t23 = ~(dot_ok ^ counter[19]);

endmodule
