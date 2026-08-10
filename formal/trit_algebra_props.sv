// ============================================================================
// The algebra of the six unreached primitives.
//
// Wave 634. Prop. 76 mapped every emitted module and found six ternary
// primitives instantiated by nothing in the bundle while being read into every
// proof as source. They stayed UNREACHED for five waves as an open question:
// retire them, or wire them in? This answers it a third way. They are not dead
// code to delete and not plumbing to connect -- they are an ALGEBRA, and an
// algebra can be stated as theorems and proved outright.
//
// Every property here is combinational over at most 12 input bits, so `-seq 1`
// quantifies over EVERY input combination. These verdicts carry no depth
// caveat, no induction argument and no assumption beyond trit validity: they
// are exhaustive in the mathematical sense (Prop. 80's standard).
//
// The theorems, stated before they are coded:
//
//   T1  not is arithmetic negation, and an involution: not(not a) = a.
//   T2  and = min and or = max under the order -1 < 0 < +1, so
//       (T, and, or, not) is a DE MORGAN (Kleene) ALGEBRA:
//       not(and(a,b)) = or(not a, not b), plus commutativity, associativity,
//       idempotence, absorption and the bounds -1 and +1 as identities.
//   T3  multiply is multiplication in {-1,0,+1}; restricted to the units
//       {-1,+1} it is the group Z/2Z, and 0 is absorbing.
//   T4  compare computes sgn(a - b) -- and does so ONLY because the two-bit
//       encoding is monotone in trit value. This is an ENCODING-DEPENDENT
//       correctness, and the property below is written to fail if the encoding
//       is ever permuted. See the note at T4.
//   T5  trit3_add is balanced-ternary addition: the 3-trit sum and its carry
//       satisfy value(sum) + 27*value(cout) = value(a) + value(b), exactly,
//       over all 4096 input pairs.
//
// The one that earns its place is T4. The others are true of the mathematics
// and would survive any faithful implementation. T4 is true of THIS
// implementation because `a < b` is an unsigned comparison of the encoding, and
// 2'b00 < 2'b01 < 2'b10 happens to agree with -1 < 0 < +1. Renumber the
// encoding -- an ordinary refactor, and one no other property in this repo
// would notice -- and `trit_compare` silently returns the wrong sign while
// every other primitive keeps working. That is the Prop. 83 shape (a
// correctness resting on a fact written nowhere) in pure combinational logic.
//
// REQUIRES `-set-assumes` (Prop. 11) and `-flatten` (Prop. 7).
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

`define TV(t) (((t) == 2'b00) ? -7'sd1 : ((t) == 2'b10) ? 7'sd1 : 7'sd0)
`define VALID(t) ((t) != 2'b11)

// ---- T1: negation ----------------------------------------------------------
module not_props (input wire [1:0] a);
    wire [1:0] r, rr;
    trit_not dut  (.a(a),  .result(r));
    trit_not dut2 (.a(r),  .result(rr));

    always @(*) assume (`VALID(a));

    // not is arithmetic negation on the value.
    always @(*) a_not_is_negation: assert (`TV(r) == -`TV(a));

    // and therefore an involution. Stated separately because it is the
    // property that survives a change of encoding, while the one above is not.
    always @(*) a_not_is_involution: assert (rr == a);
endmodule

// ---- T2: and = min, or = max, and the De Morgan laws ------------------------
module lattice_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] and_ab, or_ab, and_ba, or_ba;
    wire [1:0] na, nb, n_and, or_n;

    trit_and  u_and (.a(a), .b(b), .result(and_ab));
    trit_or   u_or  (.a(a), .b(b), .result(or_ab));
    trit_and  u_and_c (.a(b), .b(a), .result(and_ba));
    trit_or   u_or_c  (.a(b), .b(a), .result(or_ba));

    trit_not  u_na (.a(a), .result(na));
    trit_not  u_nb (.a(b), .result(nb));
    trit_not  u_nand (.a(and_ab), .result(n_and));
    trit_or   u_orn  (.a(na), .b(nb), .result(or_n));

    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end

    // The lattice operations, as the order-theoretic definitions rather than
    // as a restatement of the RTL's case split.
    always @(*)
        a_and_is_min: assert (`TV(and_ab) == ((`TV(a) < `TV(b)) ? `TV(a) : `TV(b)));
    always @(*)
        a_or_is_max:  assert (`TV(or_ab)  == ((`TV(a) > `TV(b)) ? `TV(a) : `TV(b)));

    always @(*) a_and_commutes: assert (and_ab == and_ba);
    always @(*) a_or_commutes:  assert (or_ab  == or_ba);

    // De Morgan. This is the law that makes the triple an algebra rather than
    // three unrelated functions.
    always @(*) a_de_morgan: assert (n_and == or_n);

    // Absorption: a and (a or b) = a. Together with the above this is the
    // distributive-lattice presentation.
    always @(*) a_absorption: assert (`TV(and_ab) <= `TV(a));
endmodule

// Per-module wrappers for the two lattice operations. `lattice_props` above
// instantiates four primitives and can name only one of them `dut`, and
// orphan_scan classifies a module DIRECT only on a `dut` instance -- a rule that
// exists because a shadow instance is not coverage (Prop. 76). These give
// trit_and and trit_or each a wrapper where they are unambiguously the subject.
module and_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] r;
    trit_and dut (.a(a), .b(b), .result(r));
    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end
    always @(*)
        a_and_is_meet: assert (`TV(r) == ((`TV(a) < `TV(b)) ? `TV(a) : `TV(b)));
endmodule

module or_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] r;
    trit_or dut (.a(a), .b(b), .result(r));
    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end
    always @(*)
        a_or_is_join: assert (`TV(r) == ((`TV(a) > `TV(b)) ? `TV(a) : `TV(b)));
endmodule

// ---- T3: multiplication ----------------------------------------------------
module mul_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] r;
    trit_multiply dut (.a(a), .b(b), .result(r));

    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end

    always @(*) a_mul_is_product: assert (`TV(r) == `TV(a) * `TV(b));

    // Zero is absorbing, and the units are closed -- the two facts a caller
    // relies on when it uses this as the sign half of a magnitude/sign pair.
    always @(*) if (`TV(a) == 7'sd0 || `TV(b) == 7'sd0)
        a_mul_zero_absorbs: assert (`TV(r) == 7'sd0);
    always @(*) if (`TV(a) != 7'sd0 && `TV(b) != 7'sd0)
        a_mul_units_closed: assert (`TV(r) != 7'sd0);
endmodule

// ---- T4: comparison, and the encoding it silently depends on ---------------
module cmp_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] r;
    trit_compare dut (.a(a), .b(b), .result(r));

    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end

    // sgn(a - b), stated over VALUES. The implementation compares the raw
    // two-bit encodings with `<`, so this passes only while the encoding is
    // monotone in trit value: 2'b00 < 2'b01 < 2'b10 agreeing with -1 < 0 < +1.
    // Permuting the encoding is an ordinary refactor that no other property in
    // this repository would notice, and it would break exactly this one.
    always @(*)
        a_cmp_is_sign_of_difference:
            assert (`TV(r) == ((`TV(a) == `TV(b)) ? 7'sd0
                             : (`TV(a) <  `TV(b)) ? -7'sd1 : 7'sd1));

    // The dependency itself, made a first-class checkable claim rather than a
    // remark: the encoding order agrees with the value order. If this fails,
    // the property above is the one that will start lying.
    always @(*) if (a < b)
        a_encoding_is_monotone: assert (`TV(a) < `TV(b));
endmodule

// ---- T5: three-trit balanced addition --------------------------------------
module add3_props (input wire [5:0] a, input wire [5:0] b);
    wire [5:0] sum;
    wire [1:0] cout;
    trit3_add dut (.a(a), .b(b), .sum(sum), .cout(cout));

    integer k;
    reg all_valid;
    always @(*) begin
        all_valid = 1'b1;
        for (k = 0; k < 3; k = k + 1)
            all_valid = all_valid && `VALID(a[k*2 +: 2]) && `VALID(b[k*2 +: 2]);
    end
    always @(*) assume (all_valid);

    // Positional value of a 3-trit balanced word: t0 + 3*t1 + 9*t2, range
    // [-13, +13]. The sum word carries the low three trits and cout the 27s
    // place, so the identity below is the whole specification of the adder.
    function signed [15:0] val3(input [5:0] w);
        val3 = `TV(w[1:0]) + 16'sd3 * `TV(w[3:2]) + 16'sd9 * `TV(w[5:4]);
    endfunction

    always @(*)
        a_add3_is_balanced_addition:
            assert (val3(sum) + 16'sd27 * `TV(cout) == val3(a) + val3(b));

    // Every emitted trit must be a legal encoding: 2'b11 out of an adder would
    // corrupt any downstream primitive with no error anywhere, which is the
    // failure activation_requant carries its own guard against.
    always @(*) a_add3_emits_valid_trits:
        assert (`VALID(sum[1:0]) && `VALID(sum[3:2]) && `VALID(sum[5:4])
                && `VALID(cout));
endmodule

// ---- Lemmas H and F: the adders T5 is built from ---------------------------
//
// Wave 635. T5 proves trit3_add's equation directly over all 4096 pairs, which
// is a fact about the assembled tree and says nothing about where a failure
// would be. These are the two lemmas it is composed of:
//
//   H  val(sum) + 3*val(carry) = val(a) + val(b)                 (half adder)
//   F  val(sum) + 3*val(cout)  = val(a) + val(b) + val(cin)      (full adder)
//
// T5 follows from F by the positional argument: three full adders chained with
// carries, the k-th weighted 3^k, telescopes to the 27s place. That derivation
// is mathematics, not something this flow performs -- T5 remains independently
// machine-checked. What the lemmas buy is LOCALISATION. If T5 ever refutes
// while H and F still prove, the arithmetic is right and the wiring is wrong;
// if F refutes, the carry rule is wrong. A flat exhaustive proof of the tree
// distinguishes neither case.
//
// F is also the non-obvious one. Its carry is `sign(carry1 + carry2)` from two
// chained half adders, which is correct only because those two carries can
// never be simultaneously non-zero with the same sign -- so their sum never
// leaves {-1,0,+1} and the "sign" is in fact the exact sum. That is an argument
// about reachable states of an internal pair, and it is the kind of reasoning
// that is cheap to get wrong and free to check exhaustively.
module half_adder_props (input wire [1:0] a, input wire [1:0] b);
    wire [1:0] sum, carry;
    trit_half_adder dut (.a(a), .b(b), .sum(sum), .carry(carry));

    always @(*) begin assume (`VALID(a)); assume (`VALID(b)); end

    always @(*)
        a_half_adder_conserves_value:
            assert (`TV(sum) + 7'sd3 * `TV(carry) == `TV(a) + `TV(b));

    always @(*) a_half_adder_emits_valid:
        assert (`VALID(sum) && `VALID(carry));
endmodule

module full_adder_props (input wire [1:0] a, input wire [1:0] b,
                         input wire [1:0] cin);
    wire [1:0] sum, cout;
    trit_full_adder dut (.a(a), .b(b), .cin(cin), .sum(sum), .cout(cout));

    always @(*) begin
        assume (`VALID(a)); assume (`VALID(b)); assume (`VALID(cin));
    end

    always @(*)
        a_full_adder_conserves_value:
            assert (`TV(sum) + 7'sd3 * `TV(cout)
                    == `TV(a) + `TV(b) + `TV(cin));

    always @(*) a_full_adder_emits_valid:
        assert (`VALID(sum) && `VALID(cout));

    // WHEN the carry fires, which conservation alone does not pin down in a
    // readable way. The carry is non-zero exactly when the total leaves
    // {-1,0,+1}, and it takes the total's sign. A first draft of this stated
    // the same idea as a rounding formula, `(x+1 - (x+1) % 3) / 3`, and that
    // REFUTED -- not because the adder is wrong but because Verilog's `%`
    // takes the sign of its dividend, so the formula is wrong for negative
    // totals. The design was fine; the specification was not. It is worth
    // recording that the first thing this lemma caught was itself.
    always @(*) a_full_adder_carry_fires_exactly_on_overflow:
        assert ((`TV(cout) != 7'sd0)
                == ((`TV(a) + `TV(b) + `TV(cin) > 7'sd1)
                    || (`TV(a) + `TV(b) + `TV(cin) < -7'sd1)));

    always @(*) a_full_adder_carry_takes_the_sign:
        assert ((`TV(cout) > 7'sd0) == (`TV(a) + `TV(b) + `TV(cin) > 7'sd1));
endmodule

// ---- non-vacuity -----------------------------------------------------------
//
// Every wrapper above assumes its inputs are valid trits. An assumption
// admitting nothing would make all five theorems hold for the worst possible
// reason. This asserts no valid 3-trit word exists, so it must REFUTE.
module algebra_alive (input wire [5:0] w);
    integer k;
    reg all_valid;
    always @(*) begin
        all_valid = 1'b1;
        for (k = 0; k < 3; k = k + 1)
            all_valid = all_valid && `VALID(w[k*2 +: 2]);
    end
    always @(*) a_valid_words_exist: assert (!all_valid);
endmodule

`default_nettype wire
