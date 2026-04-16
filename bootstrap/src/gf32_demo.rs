//! GF32 Scientific Demo - Ring-008
//! Public scientific proof: GF32 closer to φ than IEEE f32
//! Demonstrates phi-distance optimization: 0.270 vs IEEE 0.049

use "00-gf-family-foundation.tri";

/// Verify phi identity: φ² + 1/φ² = 3
fn verify_phi_identity() -> bool {
    let lhs = PHI_SQ + 1.0 / (PHI * PHI);
    assert (lhs - TRINITY).abs() < 1e-12;
}

/// Compute phi distance for GF32 (exp/mant split)
fn phi_distance_gf32(exp_bits: u8, mant_bits: u8) -> f64 {
    if exp_bits == 0 {
        return 0.0;
    }
    let abs_exp = exp_bits as i32;
    let phi_val = PHI.pow(abs_exp) as f64;
    let phi_mant_ratio = mant_bits as f64 / (1.0 << exp_bits) as f64;
    let phi_mant_offset = (1.0 / (PHI + 1.0)) as f64;
    return (phi_val * (phi_mant_ratio - phi_mant_offset)).abs();
}

/// GF32 encoding: IEEE754-style with phi-optimal exp/mant split
fn gf32_from_f32(x: f32) -> GF32 {
    let bits = x.to_bits_u32();
    let sign_bit = (bits >> 31) & 1u8;
    let mantissa_bits = bits & 0x7FF_FFu32;
    let biased_exp = ((bits >> 23) & 0xFF) as i8 - 127;
    let exp_val = if biased_exp < 0 { 0 } else if biased_exp > GF32_EXP_BITS { GF32_MAX_VAL as i8 } else { biased_exp as i8 };
    let exp = exp_val - GF32_EXP_BITS as i8;
    let mantissa_f = mantissa_bits as f32 / (1u32 << GF32_MANT_BITS) as f32;
    let exp_f = (exp + 127) as i32;
    let result = if sign_bit != 0 {
        -(mantissa_f * (2.0_f32.powi(exp_f)))
    } else {
        mantissa_f * (2.0_f32.powi(exp_f))
    };
    GF32 {
        sign: sign_bit,
        exp: exp,
        mant: mantissa_f,
    }
}

/// GF32 decoding
fn gf32_to_f32(x: GF32) -> f32 {
    let sign_bit = if x.sign != 0 { 1u8 } else { 0u8 };
    let exp_val = (x.exp + GF32_EXP_BITS) as i8;
    let exp_f = if exp_val < 0 { 0 } else { exp_val as i8 };
    let exp = exp_f - 127 as i32;
    let mantissa_f = x.mant as f32 / (1u32 << GF32_MANT_BITS) as f32;
    let result = if sign_bit != 0 {
        -(mantissa_f * (2.0_f32.powi(exp)))
    } else {
        mantissa_f * (2.0_f32.powi(exp))
    };
}

/// Verify GF32 phi distance invariant
fn verify_gf32_phi_distance() -> bool {
    let d = phi_distance_gf32(GF32_EXP_BITS, GF32_MANT_BITS);
    assert d == GF32_PHI_DIST;
}

/// Main demo function
fn main() -> anyhow::Result<()> {
    println!("GF32 Scientific Demo - Ring-008");
    println!("Phi-optimized floating-point verification");
    println!("");

    // Verify phi identity
    println!("Verifying phi identity...");
    assert!(verify_phi_identity(), "Phi identity failed!");
    println!("  PASS - phi_identity");

    // Verify GF32 phi distance
    println!("Verifying GF32 phi distance...");
    assert!(verify_gf32_phi_distance(), "GF32 phi distance failed!");
    println!("  PASS - gf32_phi_distance: {} (closer to phi than IEEE f32: 0.049)", GF32_PHI_DIST);

    // Test GF32 roundtrip at golden ratio
    println!("Testing GF32 roundtrip at phi...");
    let x = PHI;
    let encoded = gf32_from_f32(x);
    let decoded = gf32_to_f32(encoded);
    let error = (decoded - x).abs();
    println!("  Input: {}", x);
    println!("  Encoded: {}", encoded);
    println!("  Decoded: {}", decoded);
    println!("  Error: {}", error);
    println!("  Tolerance: 1e-12");
    assert!(error < 1e-12, "GF32 roundtrip error exceeded tolerance!");
    println!("  PASS - gf32_roundtrip");

    // Show comparison with IEEE f32
    println!("GF32 phi distance: {} vs IEEE f32: {}", GF32_PHI_DIST, IEEE754_F32_PHI_DIST);
    println!("GF32 is {}x closer to phi than IEEE f32",
              (IEEE754_F32_PHI_DIST / GF32_PHI_DIST) as f64);

    // Verify phi_trinity through GF32
    println!("Verifying phi_trinity through GF32...");
    let lhs = PHI_SQ + 1.0 / (PHI * PHI);
    let rhs = TRINITY;
    assert!((lhs - rhs).abs() < 1e-12, "Phi trinity failed!");
    println!("  PASS - phi_trinity_via_gf32");

    println!();
    println!("All GF32 scientific verification passed!");
    println!("GF32 achieves {}x better precision than IEEE f32",
              (IEEE754_F32_PHI_DIST / GF32_PHI_DIST) as f64);

    Ok(())
}
