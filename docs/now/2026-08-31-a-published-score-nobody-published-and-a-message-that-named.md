# NOW -- A published score nobody published, and a message that named the wrong consequence (2026-08-31)

## A published score nobody published, and a message that named the wrong consequence (Refs #2964)

- tri competitors audit (new): 168 records in specs/igla/coder/benchmark.t27, 150 distinct names, 6 arXiv ids carried by two records each, 141 records citing no score at any metric and 144 stating pass_at_1: 0.0 -- the value compare_with_competitor subtracts, so those 144 yield our own score as the margin over them.
- The two zero-counts are different questions and are printed apart: 141 cite nothing anywhere, 3 more cite pass@10 alone. Reading either as the other is how one population gets counted twice.
- Attribution rule is printed beside the number: a paper belongs to a record only from the contiguous run of doc lines directly above its pub fn. The first version used a 400-character window and reported 16 double-entered papers; ten were the neighbouring record's citation. a_neighbours_citation_is_not_this_ones holds the rule.
- tri types redef printed a false consequence: that the consumer takes whichever copy the compiler kept. Measured: t27c gen-rust emits EVERY copy and the generated crate does not compile (rustc E0428). Nothing picks a copy. The message now states the measurement, and tri types redef --probe re-measures it -- swap in a generator that de-duplicates and the probe fails.
- quant.rs: a_second_forall_keyword_is_not_a_body had no test attribute and had never run; a_bare_forall_is_a_clause_with_no_binders carried two. One insertion had split an attribute from its function. Both repaired, the orphaned doc comment returned; the recovered test passes.
- The ruler that found it was the compiler, not a regex: a test that loses its attribute is reported as dead code. A regex over fn-with-an-assert-inside flagged 18, of which 16 were helpers.
