/* Auto-generated from specs/base/types.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/base/types.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef TRITYPE_BASE_TYPES_H
#define TRITYPE_BASE_TYPES_H

#include <stdint.h>
#include <stdbool.h>

/* ========================================================================== */
/* Constants - Trit Values                                                     */
/* ========================================================================== */

#define TRIT_NEGONE  ((int8_t)-1)
#define TRIT_ZERO    ((int8_t)0)
#define TRIT_ONE     ((int8_t)1)

typedef int8_t Trit;

#define TRIT_NEG  ((Trit)-1)
#define TRIT_ZRO  ((Trit)0)
#define TRIT_POS  ((Trit)1)

/* ========================================================================== */
/* Constants - PackedTrit                                                      */
/* ========================================================================== */

#define PACKED_BITS_PER_TRIT  2
#define TRITS_PER_BYTE        8

#define PACKED_NEG   ((uint8_t)2)
#define PACKED_ZERO  ((uint8_t)0)
#define PACKED_ONE   ((uint8_t)1)

#define TRIT_MASK    ((uint8_t)0x03)

typedef uint8_t PackedTrit;

/* ========================================================================== */
/* Constants - TernaryWord                                                     */
/* ========================================================================== */

#define TRITS_PER_WORD  27
#define WORD_BYTES      5

typedef uint8_t TernaryWord[WORD_BYTES];

/* ========================================================================== */
/* Types                                                                       */
/* ========================================================================== */

typedef struct {
    Trit  value;
    bool  valid;
} UnpackResult;

/* ========================================================================== */
/* Function Declarations                                                       */
/* ========================================================================== */

Trit         trit_add(Trit a, Trit b);
Trit         trit_multiply(Trit a, Trit b);
Trit         trit_negate(Trit a);
uint8_t      trit_to_packed(Trit trit);
Trit         packed_to_trit(uint8_t packed);
PackedTrit   pack_trit(Trit trit, uint8_t position, PackedTrit packed);
UnpackResult unpack_trit(uint8_t position, PackedTrit packed);
void         ternary_word_pack(const Trit *src, uint8_t count, TernaryWord result);
void         ternary_word_unpack(const TernaryWord word, uint8_t count, Trit *result);
int8_t       trit_compare(Trit a, Trit b);
Trit         trit_min(Trit a, Trit b);
Trit         trit_max(Trit a, Trit b);
Trit         trit_abs(Trit a);
Trit         trit_from_i8(int8_t value);
Trit         trit_and(Trit a, Trit b);
Trit         trit_or(Trit a, Trit b);
Trit         trit_xor(Trit a, Trit b);
Trit         trit_not(Trit a);
Trit         trit_select(Trit condition, Trit a, Trit b);
uint8_t      packed_trit_count(PackedTrit packed, Trit value);
bool         packed_trit_all_equal(PackedTrit packed, Trit value);
bool         packed_trit_is_zero(PackedTrit packed);
bool         packed_trit_is_all_same(PackedTrit packed);
PackedTrit   packed_trit_nand(PackedTrit a, PackedTrit b);
PackedTrit   packed_trit_nor(PackedTrit a, PackedTrit b);
PackedTrit   packed_trit_xnor(PackedTrit a, PackedTrit b);
PackedTrit   packed_trit_shift_left(PackedTrit packed, uint8_t shift);
PackedTrit   packed_trit_shift_right(PackedTrit packed, uint8_t shift);
PackedTrit   packed_trit_rotate_left(PackedTrit packed, uint8_t rotate);
PackedTrit   packed_trit_rotate_right(PackedTrit packed, uint8_t rotate);
bool         ternary_word_is_zero(const TernaryWord word);
uint8_t      ternary_word_count(const TernaryWord word, Trit value);
bool         ternary_word_eq(const TernaryWord a, const TernaryWord b);
void         ternary_word_negate(const TernaryWord word, TernaryWord result);
bool         ternary_word_is_all_same(const TernaryWord word);

/* Test functions */
void test_trit_add_neg_plus_pos_equals_zero(void);
void test_trit_add_identity(void);
void test_trit_mul_neg_times_neg_equals_pos(void);
void test_trit_mul_zero_annihilates(void);
void test_pack_unpack_roundtrip(void);
void test_trit_negate_double_identity(void);
void test_trit_multiply_commutative(void);
void test_ternary_word_pack_unpack_roundtrip(void);

#endif /* TRITYPE_BASE_TYPES_H */
