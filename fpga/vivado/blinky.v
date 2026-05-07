module blinky (
    input  wire clk,
    output wire led5,
    output wire led6
);

    reg [26:0] counter = 0;

    always @(posedge clk) begin
        counter <= counter + 1;
    end

    assign led5 = ~counter[23];
    assign led6 = ~counter[22];

endmodule
