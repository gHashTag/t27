module blinky (
    input sys_clk,
    input sys_rst_n,
    output led_g20,
    output led_g21,
    output led_r23,
    output led_t23
);

reg [31:0] count;
reg r_led;

always @(posedge sys_clk or negedge sys_rst_n) begin
    if (!sys_rst_n)
        count <= 32'd0;
    else if (count == 32'd50_000_000)
        count <= 32'd0;
    else
        count <= count + 32'd1;
end

always @(posedge sys_clk or negedge sys_rst_n) begin
    if (!sys_rst_n)
        r_led <= 1'b0;
    else if (count < 32'd25_000_000)
        r_led <= 1'b1;
    else
        r_led <= 1'b0;
end

assign led_g20 = r_led;
assign led_g21 = ~r_led;
assign led_r23 = r_led;
assign led_t23 = ~r_led;

endmodule
