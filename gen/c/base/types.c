/* Auto-generated from specs/base/types.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/base/types.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "types.h"
#include <string.h>
#include <assert.h>

/* ========================================================================== */
/* Function Implementations                                                    */
/* ========================================================================== */

Trit trit_add(Trit a, Trit b) {
    switch (a) {
        case TRIT_NEG:
            switch (b) {
                case TRIT_NEG:  return TRIT_NEG;
                case TRIT_ZRO:  return TRIT_NEG;
                case TRIT_POS:  return TRIT_ZRO;
                default:        return TRIT_ZRO;
            }
        case TRIT_ZRO:
            return b;
        case TRIT_POS:
            switch (b) {
                case TRIT_NEG:  return TRIT_ZRO;
                case TRIT_ZRO:  return TRIT_POS;
                case TRIT_POS:  return TRIT_POS;
                default:        return TRIT_ZRO;
            }
        default:
            return TRIT_ZRO;
    }
}

Trit trit_multiply(Trit a, Trit b) {
    switch (a) {
        case TRIT_NEG:
            switch (b) {
                case TRIT_NEG:  return TRIT_POS;
                case TRIT_ZRO:  return TRIT_ZRO;
                case TRIT_POS:  return TRIT_NEG;
                default:        return TRIT_ZRO;
            }
        case TRIT_ZRO:
            return TRIT_ZRO;
        case TRIT_POS:
            return b;
        default:
            return TRIT_ZRO;
    }
}

Trit trit_negate(Trit a) {
    switch (a) {
        case TRIT_NEG:  return TRIT_POS;
        case TRIT_ZRO:  return TRIT_ZRO;
        case TRIT_POS:  return TRIT_NEG;
        default:        return TRIT_ZRO;
    }
}

uint8_t trit_to_packed(Trit trit) {
    switch (trit) {
        case TRIT_NEG:  return PACKED_NEG;
        case TRIT_ZRO:  return PACKED_ZERO;
        case TRIT_POS:  return PACKED_ONE;
        default:        return PACKED_ZERO;
    }
}

Trit packed_to_trit(uint8_t packed) {
    switch (packed & TRIT_MASK) {
        case 2:  return TRIT_NEG;
        case 0:  return TRIT_ZRO;
        case 1:  return TRIT_POS;
        default: return TRIT_ZRO;
    }
}

PackedTrit pack_trit(Trit trit, uint8_t position, PackedTrit packed) {
    if (position >= TRITS_PER_BYTE) {
        return 0xFF;
    }

    uint8_t encoding = trit_to_packed(trit);
    uint8_t bit_pos = position * PACKED_BITS_PER_TRIT;
    uint8_t mask = ~(TRIT_MASK << bit_pos);
    uint8_t result = packed & mask;
    result |= (encoding << bit_pos);

    return result;
}

UnpackResult unpack_trit(uint8_t position, PackedTrit packed) {
    UnpackResult res;
    if (position >= TRITS_PER_BYTE) {
        res.value = TRIT_ZRO;
        res.valid = false;
        return res;
    }

    uint8_t bit_pos = position * PACKED_BITS_PER_TRIT;
    uint8_t encoding = (packed >> bit_pos) & TRIT_MASK;
    res.value = packed_to_trit(encoding);
    res.valid = true;

    return res;
}

void ternary_word_pack(const Trit *src, uint8_t count, TernaryWord result) {
    memset(result, 0, WORD_BYTES);

    if (count > TRITS_PER_WORD) {
        memset(result, 0xFF, WORD_BYTES);
        return;
    }

    uint8_t n = count < TRITS_PER_WORD ? count : TRITS_PER_WORD;
    for (uint8_t i = 0; i < n; i++) {
        uint8_t byte_idx = i / TRITS_PER_BYTE;
        uint8_t trit_pos = i % TRITS_PER_BYTE;
        result[byte_idx] = pack_trit(src[i], trit_pos, result[byte_idx]);
    }
}

void ternary_word_unpack(const TernaryWord word, uint8_t count, Trit *result) {
    uint8_t n = count < TRITS_PER_WORD ? count : TRITS_PER_WORD;
    for (uint8_t i = 0; i < n; i++) {
        uint8_t byte_idx = i / TRITS_PER_BYTE;
        uint8_t trit_pos = i % TRITS_PER_BYTE;
        UnpackResult unpacked = unpack_trit(trit_pos, word[byte_idx]);
        result[i] = unpacked.valid ? unpacked.value : TRIT_ZRO;
    }
}

int8_t trit_compare(Trit a, Trit b) {
    if (a == b) return 0;
    if (a == TRIT_NEG || (a == TRIT_ZRO && b == TRIT_POS)) return -1;
    return 1;
}

Trit trit_min(Trit a, Trit b) {
    return (a == TRIT_NEG || (a == TRIT_ZRO && b == TRIT_POS)) ? a : b;
}

Trit trit_max(Trit a, Trit b) {
    return (a == TRIT_POS || (a == TRIT_ZRO && b == TRIT_NEG)) ? a : b;
}

Trit trit_abs(Trit a) {
    return (a == TRIT_NEG) ? TRIT_POS : a;
}

Trit trit_from_i8(int8_t value) {
    switch (value) {
        case -1: return TRIT_NEG;
        case  0: return TRIT_ZRO;
        case  1: return TRIT_POS;
        default: return TRIT_ZRO;
    }
}

Trit trit_and(Trit a, Trit b) {
    if (a == TRIT_POS) return b;
    if (a == TRIT_ZRO) return (b == TRIT_ZRO) ? TRIT_ZRO : TRIT_NEG;
    return TRIT_NEG;
}

Trit trit_or(Trit a, Trit b) {
    if (a == TRIT_POS) return TRIT_POS;
    if (a == TRIT_ZRO) return (b == TRIT_ZRO) ? TRIT_ZRO : TRIT_POS;
    return (b == TRIT_NEG) ? TRIT_NEG : b;
}

Trit trit_xor(Trit a, Trit b) {
    if (a == b) return (a == TRIT_NEG) ? TRIT_NEG : TRIT_ZRO;
    return TRIT_POS;
}

Trit trit_not(Trit a) {
    return (a == TRIT_POS) ? TRIT_ZRO : TRIT_POS;
}

Trit trit_select(Trit condition, Trit a, Trit b) {
    return (condition == TRIT_POS) ? a : b;
}

uint8_t packed_trit_count(PackedTrit packed, Trit value) {
    uint8_t count = 0;
    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        UnpackResult unpacked = unpack_trit(i, packed);
        if (unpacked.valid && unpacked.value == value) {
            count++;
        }
    }
    return count;
}

bool packed_trit_all_equal(PackedTrit packed, Trit value) {
    return packed_trit_count(packed, value) == TRITS_PER_BYTE;
}

bool packed_trit_is_zero(PackedTrit packed) {
    return packed_trit_all_equal(packed, TRIT_ZRO);
}

bool packed_trit_is_all_same(PackedTrit packed) {
    Trit first = unpack_trit(0, packed).value;
    return packed_trit_all_equal(packed, first);
}

PackedTrit packed_trit_nand(PackedTrit a, PackedTrit b) {
    PackedTrit result = 0;
    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        Trit a_trit = unpack_trit(i, a).value;
        Trit b_trit = unpack_trit(i, b).value;
        Trit and_result = trit_and(a_trit, b_trit);
        Trit nand_result = trit_not(and_result);
        result = pack_trit(nand_result, i, result);
    }
    return result;
}

PackedTrit packed_trit_nor(PackedTrit a, PackedTrit b) {
    PackedTrit result = 0;
    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        Trit a_trit = unpack_trit(i, a).value;
        Trit b_trit = unpack_trit(i, b).value;
        Trit or_result = trit_or(a_trit, b_trit);
        Trit nor_result = trit_not(or_result);
        result = pack_trit(nor_result, i, result);
    }
    return result;
}

PackedTrit packed_trit_xnor(PackedTrit a, PackedTrit b) {
    PackedTrit result = 0;
    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        Trit a_trit = unpack_trit(i, a).value;
        Trit b_trit = unpack_trit(i, b).value;
        Trit xor_result = trit_xor(a_trit, b_trit);
        Trit xnor_result = trit_not(xor_result);
        result = pack_trit(xnor_result, i, result);
    }
    return result;
}

PackedTrit packed_trit_shift_left(PackedTrit packed, uint8_t shift) {
    if (shift == 0) return packed;
    if (shift >= TRITS_PER_BYTE) return 0;

    PackedTrit result = 0;
    for (uint8_t i = shift; i < TRITS_PER_BYTE; i++) {
        uint8_t src_pos = i - shift;
        Trit src_trit = unpack_trit(src_pos, packed).value;
        result = pack_trit(src_trit, i, result);
    }
    return result;
}

PackedTrit packed_trit_shift_right(PackedTrit packed, uint8_t shift) {
    if (shift == 0) return packed;
    if (shift >= TRITS_PER_BYTE) return 0;

    PackedTrit result = 0;
    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        uint8_t dst_pos = i + shift;
        if (dst_pos < TRITS_PER_BYTE) {
            Trit src_trit = unpack_trit(i, packed).value;
            result = pack_trit(src_trit, dst_pos, result);
        }
    }
    return result;
}

PackedTrit packed_trit_rotate_left(PackedTrit packed, uint8_t rotate) {
    if (rotate == 0) return packed;

    uint8_t shift = rotate % TRITS_PER_BYTE;
    PackedTrit result = 0;

    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        uint8_t src_pos = (i + TRITS_PER_BYTE - shift) % TRITS_PER_BYTE;
        Trit src_trit = unpack_trit(src_pos, packed).value;
        result = pack_trit(src_trit, i, result);
    }

    return result;
}

PackedTrit packed_trit_rotate_right(PackedTrit packed, uint8_t rotate) {
    if (rotate == 0) return packed;

    uint8_t shift = rotate % TRITS_PER_BYTE;
    PackedTrit result = 0;

    for (uint8_t i = 0; i < TRITS_PER_BYTE; i++) {
        uint8_t src_pos = (i + shift) % TRITS_PER_BYTE;
        Trit src_trit = unpack_trit(src_pos, packed).value;
        result = pack_trit(src_trit, i, result);
    }

    return result;
}

bool ternary_word_is_zero(const TernaryWord word) {
    for (uint8_t i = 0; i < TRITS_PER_WORD; i++) {
        uint8_t byte_idx = i / TRITS_PER_BYTE;
        uint8_t trit_pos = i % TRITS_PER_BYTE;
        UnpackResult unpacked = unpack_trit(trit_pos, word[byte_idx]);
        if (unpacked.valid && unpacked.value != TRIT_ZRO) {
            return false;
        }
    }
    return true;
}

uint8_t ternary_word_count(const TernaryWord word, Trit value) {
    uint8_t count = 0;
    for (uint8_t i = 0; i < TRITS_PER_WORD; i++) {
        uint8_t byte_idx = i / TRITS_PER_BYTE;
        uint8_t trit_pos = i % TRITS_PER_BYTE;
        UnpackResult unpacked = unpack_trit(trit_pos, word[byte_idx]);
        if (unpacked.valid && unpacked.value == value) {
            count++;
        }
    }
    return count;
}

bool ternary_word_eq(const TernaryWord a, const TernaryWord b) {
    for (uint8_t i = 0; i < WORD_BYTES; i++) {
        if (a[i] != b[i]) return false;
    }
    return true;
}

void ternary_word_negate(const TernaryWord word, TernaryWord result) {
    memset(result, 0, WORD_BYTES);
    for (uint8_t i = 0; i < TRITS_PER_WORD; i++) {
        uint8_t byte_idx = i / TRITS_PER_BYTE;
        uint8_t trit_pos = i % TRITS_PER_BYTE;
        UnpackResult unpacked = unpack_trit(trit_pos, word[byte_idx]);
        Trit negated = trit_negate(unpacked.value);
        result[byte_idx] = pack_trit(negated, trit_pos, result[byte_idx]);
    }
}

bool ternary_word_is_all_same(const TernaryWord word) {
    Trit first = unpack_trit(0, word[0]).value;
    return ternary_word_count(word, first) == TRITS_PER_WORD;
}

/* ========================================================================== */
/* Test Functions                                                              */
/* ========================================================================== */

void test_trit_add_neg_plus_pos_equals_zero(void) {
    assert(trit_add(TRIT_NEG, TRIT_POS) == TRIT_ZRO);
}

void test_trit_add_identity(void) {
    assert(trit_add(TRIT_ZRO, TRIT_NEG) == TRIT_NEG);
    assert(trit_add(TRIT_ZRO, TRIT_ZRO) == TRIT_ZRO);
    assert(trit_add(TRIT_ZRO, TRIT_POS) == TRIT_POS);
}

void test_trit_mul_neg_times_neg_equals_pos(void) {
    assert(trit_multiply(TRIT_NEG, TRIT_NEG) == TRIT_POS);
}

void test_trit_mul_zero_annihilates(void) {
    assert(trit_multiply(TRIT_ZRO, TRIT_NEG) == TRIT_ZRO);
    assert(trit_multiply(TRIT_ZRO, TRIT_ZRO) == TRIT_ZRO);
    assert(trit_multiply(TRIT_ZRO, TRIT_POS) == TRIT_ZRO);
}

void test_pack_unpack_roundtrip(void) {
    Trit trits[] = { TRIT_NEG, TRIT_ZRO, TRIT_POS };
    for (int i = 0; i < 3; i++) {
        PackedTrit packed = pack_trit(trits[i], 3, 0);
        UnpackResult unpacked = unpack_trit(3, packed);
        assert(unpacked.value == trits[i]);
        assert(unpacked.valid);
    }
}

void test_trit_negate_double_identity(void) {
    Trit trits[] = { TRIT_NEG, TRIT_ZRO, TRIT_POS };
    for (int i = 0; i < 3; i++) {
        assert(trit_negate(trit_negate(trits[i])) == trits[i]);
    }
}

void test_trit_multiply_commutative(void) {
    Trit trits[] = { TRIT_NEG, TRIT_ZRO, TRIT_POS };
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            assert(trit_multiply(trits[i], trits[j]) == trit_multiply(trits[j], trits[i]));
        }
    }
}

void test_ternary_word_pack_unpack_roundtrip(void) {
    Trit src[] = { TRIT_NEG, TRIT_ZRO, TRIT_POS, TRIT_NEG, TRIT_ZRO, TRIT_POS };
    TernaryWord packed;
    ternary_word_pack(src, 6, packed);
    Trit unpacked[6];
    ternary_word_unpack(packed, 6, unpacked);
    for (int i = 0; i < 6; i++) {
        assert(src[i] == unpacked[i]);
    }
}
