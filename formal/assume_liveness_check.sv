// ============================================================================
// Meta-check: are assumptions actually active in the proof flow?
//
// Yosys's `sat` ignores $assume cells unless `-set-assumes` is passed. It is
// opt-in and silent: without the flag a harness still runs, still reports
// PROVED or REFUTED, and every `assume` in it is inert. A property that was
// meant to hold "given a compliant environment" is then being checked against
// an arbitrary one, and an inconclusive refutation looks like a finding.
//
// That cost a full wave here: an `arlen` anomaly was recorded as unexplained
// when the harness had simply never applied its own constraints.
//
// This module makes the failure mode observable. It assumes something
// unsatisfiable and asserts something manifestly false:
//
//   * assumptions ACTIVE   -> the assertion is vacuously true -> PROVED  (exit 0)
//   * assumptions IGNORED  -> the false assertion is reachable -> REFUTED (exit 1)
//
// So CI runs this expecting **exit 0**, and a green result here is what
// licenses reading the other harnesses' assumptions as meaningful. It is the
// tautology trick from Prop. 7 aimed at the tool rather than at the design.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module assume_liveness_check (
    input wire clk,
    input wire rst_n,
    input wire a
);
    always @(posedge clk) assume (1'b0);
    always @(posedge clk) if (rst_n) a_must_be_vacuous: assert (a == !a);
endmodule

`default_nettype wire
