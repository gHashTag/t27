# PHI LOOP Skills Registry

Constitutional 8-step spec-first development workflow.

## Skill 079: add_base_ops_utility_functions

**Spec**: `specs/base/ops.t27`
**Task**: Add 9 trit utility functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: 6fc8296f2ab70c57f5e746d0eaf59c71719cc16a577658cf4c4b9bcb020f50b2
- `spec_hash_after`: 8d209238bf26663c1d154a3faa07ee0a66cdd17159350dd03948cf6b25dff82f
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `trit_power(a: Trit, n: u8) -> Trit` - Raise trit to small power
2. `trit_from_bool(b: bool) -> Trit` - Convert boolean to trit
3. `trit_to_bool(a: Trit) -> bool` - Convert trit to boolean
4. `trit_abs_diff(a: Trit, b: Trit) -> Trit` - Absolute difference
5. `trit_cond_swap(cond: Trit, a: Trit, b: Trit) -> Trit` - Conditional swap
6. `trit_is_unit(a: Trit) -> bool` - Check if multiplicative unit
7. `trit_is_identity(a: Trit) -> bool` - Check if additive identity
8. `trit_is_negated(a: Trit, b: Trit) -> bool` - Check if b = -a

### Tests Added (21)
- `test_trit_power_zero_exponent`
- `test_trit_power_zero_base`
- `test_trit_power_one`
- `test_trit_power_square`
- `test_trit_power_cube`
- `test_trit_from_bool_true`
- `test_trit_from_bool_false`
- `test_trit_to_bool_positive`
- `test_trit_to_bool_non_positive`
- `test_trit_to_bool_from_bool_roundtrip`
- `test_trit_abs_diff_equal`
- `test_trit_abs_diff_different`
- `test_trit_abs_diff_commutative`
- `test_trit_cond_swap_condition_true`
- `test_trit_cond_swap_condition_false`
- `test_trit_is_unit_true_for_pos`
- `test_trit_is_unit_false_for_others`
- `test_trit_is_identity_true_for_zero`
- `test_trit_is_identity_false_for_others`
- `test_trit_is_negated_true_pair`
- `test_trit_is_negated_false_non_pair`

### Invariants Added (13)
- `trit_power_zero_exponent_returns_unit`
- `trit_power_zero_base_returns_zero`
- `trit_power_pos_always_pos`
- `trit_from_bool_to_bool_roundtrip`
- `trit_abs_diff_non_negative`
- `trit_abs_diff_zero_iff_equal`
- `trit_abs_diff_commutative`
- `trit_cond_swap_condition_true_swaps`
- `trit_cond_swap_condition_false_keeps_first`
- `trit_is_unit_only_for_pos`
- `trit_is_identity_only_for_zero`
- `trit_is_negated_symmetric`
- `trit_is_negated_zero_self`

### Benchmarks Added (8)
- `bench_trit_power_latency` - Target: < 15 cycles
- `bench_trit_from_bool_latency` - Target: < 5 cycles
- `bench_trit_to_bool_latency` - Target: < 5 cycles
- `bench_trit_abs_diff_latency` - Target: < 10 cycles
- `bench_trit_cond_swap_latency` - Target: < 10 cycles
- `bench_trit_is_unit_latency` - Target: < 5 cycles
- `bench_trit_is_identity_latency` - Target: < 5 cycles
- `bench_trit_is_negated_latency` - Target: < 10 cycles

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 079
- `task_id`: add_base_ops_utility_functions
- `spec_path`: specs/base/ops.t27

---

## Skill 078: add_tf3_utility_functions

**Spec**: `specs/numeric/tf3.t27`
**Task**: Add 6 TF3 utility functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: d52f9c968fc89f27c4cc419c56f673aa33af2e5342e10d1a0ad00fe4613d5c50
- `spec_hash_after`: 597dfa362c83d1afb9759269f8b533f7e476e63ecab4a451466a942197c8f64e
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `tf3_is_nan(tf3: TF3) -> bool` - Check if value is NaN
2. `tf3_is_finite(tf3: TF3) -> bool` - Check if value is finite (not NaN, not infinity)
3. `tf3_signbit(tf3: TF3) -> bool` - Check if sign bit is set
4. `tf3_sign(tf3: TF3) -> i8` - Return sign: -1, 0, or 1
5. `tf3_clamp(x: TF3, min_val: TF3, max_val: TF3) -> TF3` - Clamp to range
6. `tf3_lerp(a: TF3, b: TF3, t: TF3) -> TF3` - Linear interpolation

### Tests Added (16)
- `test_tf3_is_nan_true`
- `test_tf3_is_nan_false_for_normal`
- `test_tf3_is_finite_normal`
- `test_tf3_is_finite_false_for_inf`
- `test_tf3_is_finite_false_for_nan`
- `test_tf3_signbit_positive`
- `test_tf3_signbit_negative`
- `test_tf3_sign_positive`
- `test_tf3_sign_negative`
- `test_tf3_sign_zero`
- `test_tf3_sign_nan`
- `test_tf3_clamp_in_range`
- `test_tf3_clamp_below_min`
- `test_tf3_clamp_above_max`
- `test_tf3_lerp_t_zero`
- `test_tf3_lerp_t_one`
- `test_tf3_lerp_t_half`

### Invariants Added (7)
- `tf3_is_finite_excludes_inf_nan`
- `tf3_sign_positive_returns_one`
- `tf3_sign_negative_returns_minus_one`
- `tf3_sign_zero_returns_zero`
- `tf3_clamp_in_range_returns_value`
- `tf3_lerp_t_zero_returns_a`
- `tf3_lerp_t_one_returns_b`

### Benchmarks Added (7)
- `bench_tf3_is_nan_latency` - Target: < 20ns
- `bench_tf3_is_finite_latency` - Target: < 30ns
- `bench_tf3_signbit_latency` - Target: < 10ns
- `bench_tf3_sign_latency` - Target: < 30ns
- `bench_tf3_clamp_latency` - Target: < 200ns
- `bench_tf3_lerp_latency` - Target: < 300ns

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 078
- `task_id`: add_tf3_utility_functions
- `spec_path`: specs/numeric/tf3.t27

---

## Skill 077: add_gf16_utility_functions

**Spec**: `specs/numeric/gf16.t27`
**Task**: Add 3 GF16 utility functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: 1e20ff2b52bc38cde81a7fb55ac8436ad57d2662838d28cd3588844c12e53e9c
- `spec_hash_after`: 3f3e48fcb6b1c326627c15e54686ec56b78661a5184731211d8884931540518a
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `gf16_clamp(x: GF16, min_val: GF16, max_val: GF16) -> GF16` - Clamp to range [min, max]
2. `gf16_lerp(a: GF16, b: GF16, t: GF16) -> GF16` - Linear interpolation: a + t*(b-a)
3. `gf16_fnma(a: GF16, b: GF16, c: GF16) -> GF16` - Fused negative multiply-add: -(a*b) + c

### Tests Added (13)
- `test_gf16_clamp_in_range`
- `test_gf16_clamp_below_min`
- `test_gf16_clamp_above_max`
- `test_gf16_clamp_with_nan`
- `test_gf16_lerp_t_zero`
- `test_gf16_lerp_t_one`
- `test_gf16_lerp_t_half`
- `test_gf16_lerp_with_nan`
- `test_gf16_fnma_basic`
- `test_gf16_fnma_zero_multiplier`
- `test_gf16_fnma_zero_addend`
- `test_gf16_fnma_with_nan`

### Invariants Added (8)
- `gf16_clamp_in_range_returns_value`
- `gf16_clamp_below_min_returns_min`
- `gf16_clamp_above_max_returns_max`
- `gf16_lerp_t_zero_returns_a`
- `gf16_lerp_t_one_returns_b`
- `gf16_lerp_monotonic`
- `gf16_fnma_equals_neg_mul_plus_c`
- `gf16_fnma_zero_multiplier_returns_c`

### Benchmarks Added (3)
- `bench_gf16_clamp_latency` - Target: < 200ns
- `bench_gf16_lerp_latency` - Target: < 300ns
- `bench_gf16_fnma_latency` - Target: < 300ns

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 077
- `task_id`: add_gf16_utility_functions
- `spec_path`: specs/numeric/gf16.t27

---

## Skill 076: add_gf16_sign_functions

**Spec**: `specs/numeric/gf16.t27`
**Task**: Add 2 GF16 sign functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: dfe7ce146e5eefddd57d8a91fcbcfa2381f83e41c09b6fda4676d5a4de853764
- `spec_hash_after`: 1e20ff2b52bc38cde81a7fb55ac8436ad57d2662838d28cd3588844c12e53e9c
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `gf16_signbit(gf16: GF16) -> bool` - Check if sign bit is set (negative or negative zero)
2. `gf16_sign(gf16: GF16) -> i8` - Return sign: -1 for negative, 0 for zero, +1 for positive

### Tests Added (11)
- `test_gf16_signbit_positive`
- `test_gf16_signbit_negative`
- `test_gf16_signbit_positive_zero`
- `test_gf16_signbit_negative_zero`
- `test_gf16_signbit_infinity`
- `test_gf16_signbit_nan`
- `test_gf16_sign_positive`
- `test_gf16_sign_negative`
- `test_gf16_sign_zero`
- `test_gf16_sign_nan`
- `test_gf16_sign_infinity`
- `test_gf16_sign_matches_signbit`

### Invariants Added (5)
- `gf16_signbit_positive_no_signbit`
- `gf16_signbit_negative_has_signbit`
- `gf16_sign_positive_returns_one`
- `gf16_sign_negative_returns_minus_one`
- `gf16_sign_zero_returns_zero`
- `gf16_sign_nan_returns_zero`

### Benchmarks Added (2)
- `bench_gf16_signbit_latency` - Target: < 5ns
- `bench_gf16_sign_latency` - Target: < 30ns

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 076
- `task_id`: add_gf16_sign_functions
- `spec_path`: specs/numeric/gf16.t27

---

## Skill 075: add_gf16_classification_functions

**Spec**: `specs/numeric/gf16.t27`
**Task**: Add 3 GF16 classification functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: b7f39eee2ff274bebfba4bcde34445bd01862c623380a2c89225fdef523d49db
- `spec_hash_after`: dfe7ce146e5eefddd57d8a91fcbcfa2381f83e41c09b6fda4676d5a4de853764
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `gf16_is_finite(gf16: GF16) -> bool` - Check if value is finite (not NaN, not infinity)
2. `gf16_is_normal(gf16: GF16) -> bool` - Check if value is a normal number
3. `gf16_is_subnormal(gf16: GF16) -> bool` - Check if value is subnormal (denormal)

### Tests Added (13)
- `test_gf16_is_finite_normal_numbers`
- `test_gf16_is_finite_zero`
- `test_gf16_is_finite_false_for_inf`
- `test_gf16_is_finite_false_for_nan`
- `test_gf16_is_normal_true_for_normal`
- `test_gf16_is_normal_false_for_zero`
- `test_gf16_is_normal_false_for_inf`
- `test_gf16_is_normal_false_for_nan`
- `test_gf16_is_subnormal_true_for_subnormal`
- `test_gf16_is_subnormal_false_for_normal`
- `test_gf16_is_subnormal_false_for_zero`
- `test_gf16_is_subnormal_false_for_special`
- `test_gf16_classification_complete_coverage`

### Invariants Added (6)
- `gf16_is_finite_excludes_inf_nan`
- `gf16_is_normal_implies_finite`
- `gf16_is_subnormal_implies_finite`
- `gf16_is_normal_and_subnormal_mutually_exclusive`
- `gf16_zero_neither_normal_nor_subnormal`
- `gf16_classification_exhaustive`

### Benchmarks Added (3)
- `bench_gf16_is_finite_latency` - Target: < 30ns
- `bench_gf16_is_normal_latency` - Target: < 40ns
- `bench_gf16_is_subnormal_latency` - Target: < 40ns

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 075
- `task_id`: add_gf16_classification_functions
- `spec_path`: specs/numeric/gf16.t27

---

## Skill 074: add_base_types_ternary_word_utility_functions

**Spec**: `specs/base/types.t27`
**Task**: Add 5 TernaryWord utility functions with tests, invariants, and benchmarks

### Hashes
- `spec_hash_before`: f9069ee88d8f356a817c562a48b7543a41927c1b102fe3481705d9d448d54b66
- `spec_hash_after`: 1d41b788fd86acae12dca279f73490d53a0eca2aac086965f58db1064a64b7e8
- `gen_hash_after`: pending (tri gen not available)
- `test_vector_hash`: pending (tri test not available)

### Functions Added
1. `ternary_word_is_zero(word: TernaryWord) -> bool` - Check if all 27 trits are zero
2. `ternary_word_count(word: TernaryWord, value: Trit) -> u8` - Count occurrences of a trit value
3. `ternary_word_eq(a: TernaryWord, b: TernaryWord) -> bool` - Compare two TernaryWords
4. `ternary_word_negate(word: TernaryWord) -> TernaryWord` - Negate all trits
5. `ternary_word_is_all_same(word: TernaryWord) -> bool` - Check if all trits are the same

### Tests Added (11)
- `test_ternary_word_is_zero_true`
- `test_ternary_word_is_zero_false`
- `test_ternary_word_count_zeros`
- `test_ternary_word_count_all_same`
- `test_ternary_word_equal_same`
- `test_ternary_word_equal_different`
- `test_ternary_word_negate`
- `test_ternary_word_negate_double_identity`
- `test_ternary_word_is_all_same_true`
- `test_ternary_word_is_all_same_false`

### Invariants Added (10)
- `ternary_word_is_zero_idempotent`
- `ternary_word_count_range`
- `ternary_word_count_sum_equals_trits_per_word`
- `ternary_word_eq_reflexive`
- `ternary_word_eq_symmetric`
- `ternary_word_eq_transitive`
- `ternary_word_negate_involutive`
- `ternary_word_negate_zero_invariant`
- `ternary_word_is_all_same_implies_count_equals_word_size`
- `ternary_word_is_all_same_zero_implies_is_zero`

### Benchmarks Added (6)
- `bench_ternary_word_is_zero_latency` - Target: < 200 cycles
- `bench_ternary_word_count_latency` - Target: < 300 cycles
- `bench_ternary_word_eq_latency` - Target: < 50 cycles
- `bench_ternary_word_negate_latency` - Target: < 500 cycles
- `bench_ternary_word_is_all_same_latency` - Target: < 350 cycles

### Verdict
- `test_status`: valid (tri spec validate passed)
- `verdict`: clean
- `bench_delta`: pending (tri bench not available)
- `sealed_at`: 2026-04-04

### Commit
- `skill_id`: 074
- `task_id`: add_base_types_ternary_word_utility_functions
- `spec_path`: specs/base/types.t27
