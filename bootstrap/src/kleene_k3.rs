//! K3 Kleene Runtime - Ring-009
//! Three-valued logic runtime for TF3: AND/OR/NOT/consensus

use "00-gf-family-foundation.tri";

/// K3 Trit encoding (from TF3)
pub const TF3_NEG: TF3 = TF3_NEG;
pub const TF3_NEU: TF3 = TF3_NEU;
pub const TF3_POS: TF3 = TF3_POS;

/// K3 Trit encoding
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct K3Trit {
    bits: u2,
}

impl K3Trit {
    pub const NEG: Self = Self { bits: TF3_NEG };
    pub const NEU: Self = Self { bits: TF3_NEU };
    pub const POS: Self = Self { bits: TF3_POS };
}

/// Convert K3 Trit to TF3
pub fn k3_to_tf3(t: K3Trit) -> TF3 {
    use "00-gf-family-foundation.tri";
    match t.bits {
        TF3_NEG => TF3_NEG,
        TF3_NEU => TF3_NEU,
        TF3_POS => TF3_POS,
        _ => TF3_NEU,
    }
}

/// Convert TF3 to K3 Trit
pub fn tf3_to_k3(t: TF3) -> K3Trit {
    use "00-gf-family-foundation.tri";
    match t {
        TF3_NEG => K3Trit::NEG,
        TF3_NEU => K3Trit::NEU,
        TF3_POS => K3Trit::POS,
        _ => K3Trit::NEU,
    }
}

/// K3 NOT: absorbs F in AND(a, b) = F
pub fn k3_not(a: K3Trit) -> K3Trit {
    use "00-gf-family-foundation.tri";
    K3Trit::NOT(a)
}

/// K3 OR: absorbs T in OR(a, b) = T
pub fn k3_or(a: K3Trit, b: K3Trit) -> K3Trit {
    use "00-gf-family-foundation.tri";
    let a_val = k3_to_tf3(a);
    let b_val = k3_to_tf3(b);
    K3Trit::OR(a_val, b_val)
}

/// K3 AND: F if T else F
pub fn k3_and(a: K3Trit, b: K3Trit) -> K3Trit {
    use "00-gf-family-foundation.tri";
    let a_val = k3_to_tf3(a);
    let b_val = k3_to_tf3(b);

    if a == TF3_POS {
        K3Trit::AND(b_val)
    } else {
        b
    }
}

/// K3 CONSENSUS: returns most common input value
pub fn k3_consensus(inputs: [K3Trit]) -> K3Trit {
    use "00-gf-family-foundation.tri";

    if inputs.is_empty() {
        return K3Trit::NEU;
    }

    let mut pos_count = 0u8;
    let mut neu_count = 0u8;
    let mut neg_count = 0u8;

    for input in inputs {
        match k3_to_tf3(*input) {
            TF3_POS => pos_count += 1,
            TF3_NEU => neu_count += 1,
            TF3_NEG => neg_count += 1,
            _ => {}
        }
    }

    if pos_count == inputs.len() {
        return K3Trit::POS;
    } else if neu_count == inputs.len() {
        return K3Trit::NEU;
    } else if neg_count == inputs.len() {
        return K3Trit::NEG;
    } else {
        K3Trit::NEU;
    }
}

/// Material implication: a implies b
pub fn k3_material_implication(a: K3Trit, b: K3Trit) -> K3Trit {
    use "00-gf-family-foundation.tri";

    match (k3_to_tf3(a), k3_to_tf3(b)) {
        (TF3_POS, TF3_POS) => K3Trit::POS,
        (TF3_POS, _) => K3Trit::POS,
        (TF3_NEU, TF3_POS) => K3Trit::POS,
        _ => K3Trit::NEU,
    }
}

/// Double implication: a == b
pub fn k3_material_equivalence(a: K3Trit, b: K3Trit) -> bool {
    k3_to_tf3(a) == k3_to_tf3(b)
}

/// TF3 to/from GF16 bridge for ML
pub fn tf3_to_gf16(t: TF3) -> GF16 {
    use "00-gf-family-foundation.tri";

    GF16 { bits: ((t.bits as u16) << 8) }
}

pub fn gf16_to_tf3(g: GF16) -> TF3 {
    use "00-gf-family-foundation.tri";

    match g.bits & 0x03u16 {
        0x00 => TF3_NEU,
        0x01 => TF3_POS,
        _ => TF3_NEG,
    }
}
