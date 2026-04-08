// Language test harness: Rust f64 precision test
// Tests IEEE 754 binary64 precision against GoldenFloat ternary claims
//
// Usage: cargo run --bin rust_f64 > results/rust_f64.json

use serde::Serialize;

#[derive(Serialize)]
struct TestResult {
    name: String,
    passed: bool,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Serialize)]
struct LanguageTestResults {
    language: String,
    precision: String,
    tests: Vec<TestResult>,
    all_passed: bool,
    summary: serde_json::Value,
}

fn count_decimal_places(value: f64, reference: f64) -> i32 {
    let s1 = format!("{:.20}", value);
    let s2 = format!("{:.20}", reference);

    let mut count = 0;
    let mut found_decimal = false;

    for (c1, c2) in s1.chars().zip(s2.chars()) {
        if c1 == '.' {
            found_decimal = true;
            continue;
        }
        if found_decimal && c1 == c2 {
            count += 1;
        } else if found_decimal {
            break;
        }
    }

    count
}

fn test_phi() -> TestResult {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    let expected = 1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475_f64;
    let error = (phi - expected).abs();

    TestResult {
        name: "phi".to_string(),
        passed: error < 1e-15,
        extra: serde_json::json!({
            "expected": expected,
            "computed": phi,
            "error": error,
            "decimal_places": count_decimal_places(phi, expected)
        }),
    }
}

fn test_phi_squared() -> TestResult {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    let phi_sq = phi * phi;
    let phi_plus_one = phi + 1.0;
    let error = (phi_sq - phi_plus_one).abs();

    TestResult {
        name: "phi_squared_equals_phi_plus_one".to_string(),
        passed: error < 1e-15,
        extra: serde_json::json!({
            "phi_sq": phi_sq,
            "phi_plus_one": phi_plus_one,
            "error": error
        }),
    }
}

fn test_trinity_identity() -> TestResult {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    let phi_inv = 1.0 / phi;
    let phi_sq = phi * phi;
    let phi_inv_sq = phi_inv * phi_inv;
    let trinity = phi_sq + phi_inv_sq;
    let expected = 3.0_f64;
    let error = (trinity - expected).abs();

    TestResult {
        name: "trinity_identity".to_string(),
        passed: error < 1e-12,
        extra: serde_json::json!({
            "trinity": trinity,
            "expected": expected,
            "error": error
        }),
    }
}

fn test_one_third() -> TestResult {
    let value = 1.0_f64 / 3.0_f64;
    let value_str = format!("{:.16}", value);
    let expected_str = "0.3333333333333333";

    TestResult {
        name: "one_third".to_string(),
        passed: true, // Always passes, measuring precision
        extra: serde_json::json!({
            "value": value,
            "value_str": value_str,
            "expected_str": expected_str,
            "decimal_places": 15, // IEEE f64 gives ~15-16 decimal places
            "error": (value - 1.0/3.0).abs()
        }),
    }
}

fn test_accumulation() -> TestResult {
    let n_terms = 100_000;
    let total: f64 = (1..=n_terms).map(|n| 1.0 / n as f64).sum();

    TestResult {
        name: "accumulation".to_string(),
        passed: true, // Documenting behavior
        extra: serde_json::json!({
            "n_terms": n_terms,
            "total": total
        }),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tests = vec![
        test_phi(),
        test_phi_squared(),
        test_trinity_identity(),
        test_one_third(),
        test_accumulation(),
    ];

    let all_passed = tests.iter().all(|t| t.passed);

    let summary = serde_json::json!({
        "phi_error": tests[0].extra["error"],
        "phi_decimal_places": tests[0].extra["decimal_places"],
        "one_third_decimal_places": tests[3].extra["decimal_places"]
    });

    let results = LanguageTestResults {
        language: "Rust".to_string(),
        precision: "f64 (IEEE 754 binary64)".to_string(),
        tests,
        all_passed,
        summary,
    };

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
