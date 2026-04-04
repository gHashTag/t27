// Auto-generated from compiler/runtime/runtime.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/runtime/runtime.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// T27 Runtime Core (Hardware)
// ============================================================================
// Minimal runtime controller: stack pointer, cycle counter, thread state

module t27_runtime_core #(
    parameter STACK_SIZE   = 4096,
    parameter MAX_THREADS  = 8,
    parameter DATA_WIDTH   = 32,
    parameter ADDR_WIDTH   = 12
) (
    input  wire                  clk,
    input  wire                  rst_n,
    input  wire                  exec_start,
    input  wire [ADDR_WIDTH-1:0] entry_addr,
    output reg  [ADDR_WIDTH-1:0] stack_ptr,
    output reg  [31:0]           cycle_counter,
    output reg  [31:0]           instr_counter,
    output reg  [1:0]            thread_state,  // 0=idle, 1=running, 2=blocked
    output reg                   exec_done,
    output reg  [7:0]            exit_code
);

    localparam S_IDLE    = 2'd0;
    localparam S_RUNNING = 2'd1;
    localparam S_BLOCKED = 2'd2;

    reg [1:0] state;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state         <= S_IDLE;
            stack_ptr     <= STACK_SIZE[ADDR_WIDTH-1:0];
            cycle_counter <= 32'd0;
            instr_counter <= 32'd0;
            thread_state  <= S_IDLE;
            exec_done     <= 1'b0;
            exit_code     <= 8'd0;
        end else begin
            cycle_counter <= cycle_counter + 32'd1;

            case (state)
                S_IDLE: begin
                    exec_done <= 1'b0;
                    if (exec_start) begin
                        state        <= S_RUNNING;
                        thread_state <= S_RUNNING;
                        stack_ptr    <= STACK_SIZE[ADDR_WIDTH-1:0];
                    end
                end

                S_RUNNING: begin
                    instr_counter <= instr_counter + 32'd1;
                    // Simplified: execute one instruction per cycle
                    // In full implementation, fetch-decode-execute pipeline
                    state        <= S_IDLE;
                    thread_state <= S_IDLE;
                    exec_done    <= 1'b1;
                    exit_code    <= 8'd0;
                end

                default: state <= S_IDLE;
            endcase
        end
    end

endmodule
