// Auto-generated from compiler/runtime/commands.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/runtime/commands.t27
// phi^2 + phi^-2 = 3 | TRINITY

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


// ============================================================================
// Command Decoder (Hardware)
// ============================================================================
// Decodes 3-bit command ID to one-hot command enable signals

module t27_command_decoder (
    input  wire [2:0] cmd_id,
    output wire       cmd_spec,
    output wire       cmd_gen,
    output wire       cmd_compile,
    output wire       cmd_git,
    output wire       cmd_lint,
    output wire       cmd_skill,
    output wire       cmd_help
);

    assign cmd_spec    = (cmd_id == 3'd0);
    assign cmd_gen     = (cmd_id == 3'd1);
    assign cmd_compile = (cmd_id == 3'd2);
    assign cmd_git     = (cmd_id == 3'd3);
    assign cmd_lint    = (cmd_id == 3'd4);
    assign cmd_skill   = (cmd_id == 3'd5);
    assign cmd_help    = (cmd_id == 3'd6);

endmodule
