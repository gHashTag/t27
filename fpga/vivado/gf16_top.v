module gf16_top (
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

    wire [15:0] mul_result;

    gf16_mul u_mul (
        .a(16'h3E00),
        .b(16'h3E00),
        .result(mul_result)
    );

    wire mul_ok = (mul_result == 16'h3E00);

    assign led_r23 = ~counter[20];
    assign led_t23 = ~(mul_ok ^ counter[19]);

endmodule
