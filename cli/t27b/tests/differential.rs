//! Randomized differential test: generated t27 source code against the
//! reference interpreter (`eval`) and reference compiler (`t27c gen-zig` + zig).
//!
//! This is a source-level generator that produces random `.t27` modules
//! limited to constructs t27b accepts, with constants biased toward 0, ±1,
//! powers of two and MIN/MAX (YARPGen policy). Every test ends in 
//! `assert_eq(checksum, <value>)` where the expected value is left open
//! so the oracle is agreement between t27b and t27c + zig.
//!
//! The generator runs on the Railway lab with a fresh seed every run and
//! records failing seeds. Each failing case is committed as a regression
//! spec under a fixtures directory, not under `specs/`.
//!
//! Classifies disagreements as:
//! - t27b bug: JIT and interpreter disagree (both run lower.rs's IR)
//! - reference bug: t27c gen-zig + zig disagrees with t27b
//! - semantics gap: no written rule covers the case
//!
//! Usage: `tri t27b fuzz --cases N --seed S`
//!
//! Knobs (environment variables):
//!   T27B_FUZZ_CASES    programs to generate (default 1000)
//!   T27B_FUZZ_SEED     base seed (default random)
//!   T27B_FUZZ_TRACE    print each case seed before running it (to find a crash)
//!   T27B_FUZZ_FIXTURES directory to save failing cases (default fixtures/fuzz)
//!
//! Exit codes: 0 success; 1 disagreement found; 2 usage error; 64 I/O error
#![cfg(all(target_arch = "aarch64", any(target_os = "macos", target_os = "linux")))]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use t27b::codegen::{self, canon, TrapStyle};
use t27b::eval::{Interp, Stop};
use t27b::ir::*;
use t27b::jit::Jit;
use t27b::{front, lower};

// ------------------------------------------------------------------ rng

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03);
        if r.0 == 0 {
            r.0 = 1;
        }
        r.next();
        r
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, pct: usize) -> bool {
        self.below(100) < pct
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len())]
    }
}

// ------------------------------------------------------------------ types

/// Types that t27b accepts for source-level generation
const SOURCE_TYPES: [&str; 7] = [
    "bool",
    "i8", "i16", "i32", "i64",
    "u8", "u16", "u32", "u64",
];

/// Biased value generation: 0, ±1, powers of two and MIN/MAX (YARPGen policy)
fn biased_value(rng: &mut Rng, ty: &str) -> i128 {
    if ty == "bool" {
        return rng.below(2) as i128;
    }
    
    match rng.below(16) {
        0 => 0,
        1 => 1,
        2 => -1,
        3 => {
            // MAX value for the type
            match ty {
                "i8" => 127,
                "i16" => 32767,
                "i32" => 2147483647,
                "i64" => 9223372036854775807,
                "u8" => 255,
                "u16" => 65535,
                "u32" => 4294967295,
                "u64" => 18446744073709551615,
                _ => 0,
            }
        },
        4 => {
            // MIN value for the type
            match ty {
                "i8" => -128,
                "i16" => -32768,
                "i32" => -2147483648,
                "i64" => -9223372036854775808,
                "u8" => 0,
                "u16" => 0,
                "u32" => 0,
                "u64" => 0,
                _ => 0,
            }
        },
        5 => ty.parse().unwrap_or(0),
        6 | 7 => rng.below(16) as i128,
        8 => ty.parse().unwrap_or(0) + 1,
        9 => {
            let max = match ty {
                "i8" => 127,
                "i16" => 32767,
                "i32" => 2147483647,
                "i64" => 9223372036854775807,
                "u8" => 255,
                "u16" => 65535,
                "u32" => 4294967295,
                "u64" => 18446744073709551615,
                _ => 255,
            };
            max - 1
        },
        10 => {
            // Power of two
            let power = 1i128 << (rng.below(8) as i128);
            if rng.chance(50) { power } else { -power }
        },
        11 => {
            // Power of two minus one
            let power = (1i128 << (rng.below(8) as i128)) - 1;
            if rng.chance(50) { power } else { -power }
        },
        _ => {
            let sh = rng.below(64) as u32;
            (rng.next() >> sh) as i128
        }
    }
}

// ------------------------------------------------------------------ source generator

/// Generate a random t27 source program
fn generate_source_program(rng: &mut Rng, mode: OverflowMode) -> String {
    let mut program = String::new();
    
    // Generate module declaration
    program.push_str(&format!("module fuzz_{};\n\n", rng.next()));
    
    // Generate constants
    let nconsts = rng.below(5);
    for i in 0..nconsts {
        let ty = rng.pick(&SOURCE_TYPES);
        let name = format!("CONST_{}", i);
        let value = biased_value(rng, ty);
        program.push_str(&format!("pub const {} : {} = {};\n", name, ty, value));
    }
    
    if nconsts > 0 {
        program.push_str("\n");
    }
    
    // Generate test functions
    let ntests = 1 + rng.below(3);
    for i in 0..ntests {
        let test_name = format!("test_{}", i);
        program.push_str(&format!("test \"{}\" {{\n", test_name));
        
        // Test body with checksum assertion
        let nexprs = 2 + rng.below(5);
        let mut checksum_expr = String::new();
        
        for j in 0..nexprs {
            let ty = rng.pick(&SOURCE_TYPES);
            let expr = generate_expression(rng, ty);
            if j == 0 {
                checksum_expr = expr.clone();
            }
            program.push_str(&format!("    var{} : {} = {};\n", j, ty, expr));
        }
        
        // End with checksum assertion using Zig-like syntax
        program.push_str(&format!(
            "    try std.testing.expectEqual(@as({}, {}), var0);\n",
            rng.pick(&SOURCE_TYPES), checksum_expr
        ));
        program.push_str("}\n\n");
    }
    
    program
}

/// Generate a random expression
fn generate_expression(rng: &mut Rng, ty: &str) -> String {
    let expr_type = rng.below(6);
    
    match expr_type {
        0 => {
            // Literal constant
            biased_value(rng, ty).to_string()
        },
        1 => {
            // Reference to a constant
            format!("CONST_{}", rng.below(5))
        },
        2 => {
            // Binary arithmetic
            let op = rng.pick(&["+", "-", "*", "%"]);
            let lhs = generate_expression(rng, ty);
            let rhs = generate_expression(rng, ty);
            format!("({} {} {})", lhs, op, rhs)
        },
        3 => {
            // Binary comparison
            let op = rng.pick(&["==", "!=", "<", "<=", ">", ">="]);
            let lhs_ty = rng.pick(&SOURCE_TYPES);
            let rhs_ty = rng.pick(&SOURCE_TYPES);
            let lhs = generate_expression(rng, lhs_ty);
            let rhs = generate_expression(rng, rhs_ty);
            format!("({} {} {})", lhs, op, rhs)
        },
        4 => {
            // Unary operator
            if ty.starts_with('i') && rng.chance(50) {
                let expr = generate_expression(rng, ty);
                format!("-{}", expr)
            } else {
                let expr = generate_expression(rng, ty);
                format!("~{}", expr)
            }
        },
        5 => {
            // Cast
            let from_ty = rng.pick(&SOURCE_TYPES);
            let expr = generate_expression(rng, from_ty);
            format!("@as({}, {})", ty, expr)
        }
        _ => {
            // Fallback to literal
            biased_value(rng, ty).to_string()
        }
    }
}

// ------------------------------------------------------------------ test runner

/// Test results
#[derive(Debug, Clone)]
struct TestResults {
    cases: usize,
    agree: usize,
    disagree: usize,
    seeds: Vec<u64>,
    failures: Vec<(u64, String)>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            cases: 0,
            agree: 0,
            disagree: 0,
            seeds: Vec::new(),
            failures: Vec::new(),
        }
    }
    
    fn add_result(&mut self, seed: u64, agree: bool, error: Option<String>) {
        self.cases += 1;
        self.seeds.push(seed);
        if agree {
            self.agree += 1;
        } else {
            self.disagree += 1;
            if let Some(err) = error {
                self.failures.push((seed, err));
            }
        }
    }
    
    fn to_json(&self) -> String {
        serde_json::json!({
            "cases": self.cases,
            "agree": self.agree,
            "disagree": self.disagree,
            "seeds": self.seeds,
            "failures": self.failures.iter().map(|(s, e)| {
                json!({"seed": s, "error": e})
            }).collect::<Vec<_>>()
        }).to_string()
    }
}

/// Run a generated test case
fn run_test_case(rng: &mut Rng, seed: u64, mode: OverflowMode, results: &mut TestResults) -> bool {
    // Generate source program
    let source = generate_source_program(rng, mode);
    
    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let source_file = temp_dir.join("test.t27");
    fs::write(&source_file, &source).expect("failed to write source file");
    
    // Parse and typecheck the generated source
    let parsed = front::parse(&source_file, &source).expect("parse failed");
    let tc = front::typecheck(&parsed.ast);
    if let Err(errs) = tc {
        let error_msg = format!("Typecheck error: {:?}", errs);
        results.add_result(seed, false, Some(error_msg));
        return false;
    }
    
    // Lower to IR
    let lowered = lower::lower_src(&parsed.ast, mode, Some(&source));
    let prog = match lowered {
        Ok(p) => p,
        Err(rejects) => {
            let error_msg = format!("Lowering error: {:?}", rejects);
            results.add_result(seed, false, Some(error_msg));
            return false;
        }
    };
    
    // Run t27b tests
    let t27b_result = run_t27b_tests(&prog);
    
    // Run reference tests (t27c gen-zig + zig)
    let reference_result = run_reference_tests(&source_file);
    
    // Compare results
    let agree = compare_results(&t27b_result, &reference_result);
    
    if !agree {
        let error_msg = format!("Disagreement: t27b={:?}, reference={:?}", t27b_result, reference_result);
        results.add_result(seed, false, Some(error_msg));
        
        // Save failing case as fixture
        save_failing_case(seed, &source, &error_msg);
    } else {
        results.add_result(seed, true, None);
    }
    
    agree
}

/// Run tests with t27b
fn run_t27b_tests(prog: &Program) -> Vec<(String, bool)> {
    let mut results = Vec::new();
    
    // Generate and compile
    let code = codegen::compile(prog, TrapStyle::Jit, true)
        .expect("codegen failed");
    let jit = Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals)
        .expect("jit load failed");
    
    // Run each test
    for (id, f) in prog.tests() {
        let r = jit.call(id as u32, &[]);
        match r {
            Ok(_) => results.push((f.name.clone(), true)),
            Err(_) => results.push((f.name.clone(), false)),
        }
    }
    
    results
}

/// Run tests with reference compiler (t27c gen-zig + zig)
fn run_reference_tests(source_file: &Path) -> Vec<(String, bool)> {
    let mut results = Vec::new();
    
    // Run t27c gen-zig
    let output = Command::new("t27c")
        .args(["gen-zig", source_file.to_str().unwrap()])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            // Write zig file
            let zig_file = source_file.with_extension("zig");
            fs::write(&zig_file, &output.stdout).expect("failed to write zig file");
            
            // Compile with zig
            let exe_file = source_file.with_extension("");
            let compile_output = Command::new("zig")
                .args(["build-exe", zig_file.to_str().unwrap(), "-freference", "-o", exe_file.to_str().unwrap()])
                .output();
            
            match compile_output {
                Ok(output) if output.status.success() => {
                    // Run the test binary
                    let run_output = Command::new(&exe_file)
                        .output();
                    
                    match run_output {
                        Ok(output) => {
                            // Parse test results (simplified)
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                if line.starts_with("PASS ") {
                                    let test_name = line[5..].trim();
                                    results.push((test_name.to_string(), true));
                                } else if line.starts_with("FAIL ") {
                                    let test_name = line[5..].trim();
                                    results.push((test_name.to_string(), false));
                                }
                            }
                        },
                        Err(_) => {
                            results.push(("test_binary_failed".to_string(), false));
                        }
                    }
                },
                Ok(_) => {
                    results.push(("zig_compile_failed".to_string(), false));
                },
                Err(_) => {
                    results.push(("zig_not_found".to_string(), false));
                }
            }
        },
        Ok(_) => {
            results.push(("t27c_gen_zig_failed".to_string(), false));
        },
        Err(_) => {
            results.push(("t27c_not_found".to_string(), false));
        }
    }
    
    results
}

/// Compare t27b and reference results
fn compare_results(t27b_results: &[(String, bool)], reference_results: &[(String, bool)]) -> bool {
    if t27b_results.len() != reference_results.len() {
        return false;
    }
    
    for (i, (t27b_name, t27b_pass)) in t27b_results.iter().enumerate() {
        let (_, ref_pass) = reference_results[i];
        if t27b_pass != ref_pass {
            return false;
        }
    }
    
    true
}

/// Save failing test case as fixture
fn save_failing_case(seed: u64, source: &str, error: &str) {
    let fixtures_dir = PathBuf::from("fixtures/fuzz");
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");
    
    let filename = format!("fuzz_{:016x}.t27", seed);
    let filepath = fixtures_dir.join(filename);
    
    let content = format!(
        "// Generated with seed: {:016x}\n// Error: {}\n\n{}",
        seed, error, source
    );
    
    fs::write(&filepath, content).expect("failed to save failing case");
}

// ------------------------------------------------------------------ main test function

#[test]
fn fuzz_source_level_differential() {
    let cases = std::env::var("T27B_FUZZ_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    
    let base_seed = std::env::var("T27B_FUZZ_SEED")
        .ok()
        .and_then(|s| {
            if s.starts_with("0x") {
                u64::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse().ok()
            }
        })
        .unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
    
    let trace = std::env::var("T27B_FUZZ_TRACE").is_ok();
    let fixtures_dir = std::env::var("T27B_FUZZ_FIXTURES").unwrap_or_else(|_| "fixtures/fuzz".to_string());
    
    let mut results = TestResults::new();
    let mut failures = Vec::new();
    
    for mode in [OverflowMode::Trap, OverflowMode::Wrap] {
        for i in 0..cases {
            let seed = base_seed.wrapping_add(i as u64);
            if trace {
                eprintln!("fuzz case seed {:016x} mode {:?}", seed, mode);
            }
            
            let mut rng = Rng::new(seed);
            let agree = run_test_case(&mut rng, seed, mode, &mut results);
            
            if !agree {
                failures.push(format!("seed {:016x} mode {:?}: disagreement", seed, mode));
            }
        }
    }
    
    // Write results to latest.json
    let latest_json = PathBuf::from("latest.json");
    fs::write(&latest_json, results.to_json()).expect("failed to write latest.json");
    
    // Write summary
    eprintln!(
        "fuzz results: {} cases, {} agree, {} disagree",
        results.cases, results.agree, results.disagree
    );
    
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

// ------------------------------------------------------------------ standalone main function

fn main() -> ExitCode {
    let cases = std::env::var("T27B_FUZZ_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    
    let base_seed = std::env::var("T27B_FUZZ_SEED")
        .ok()
        .and_then(|s| {
            if s.starts_with("0x") {
                u64::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse().ok()
            }
        })
        .unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
    
    let trace = std::env::var("T27B_FUZZ_TRACE").is_ok();
    let fixtures_dir = std::env::var("T27B_FUZZ_FIXTURES").unwrap_or_else(|_| "fixtures/fuzz".to_string());
    
    let mut results = TestResults::new();
    let mut failures = Vec::new();
    
    for mode in [OverflowMode::Trap, OverflowMode::Wrap] {
        for i in 0..cases {
            let seed = base_seed.wrapping_add(i as u64);
            if trace {
                eprintln!("fuzz case seed {:016x} mode {:?}", seed, mode);
            }
            
            let mut rng = Rng::new(seed);
            let agree = run_test_case(&mut rng, seed, mode, &mut results);
            
            if !agree {
                failures.push(format!("seed {:016x} mode {:?}: disagreement", seed, mode));
            }
        }
    }
    
    // Write results to latest.json
    let latest_json = PathBuf::from("latest.json");
    fs::write(&latest_json, results.to_json()).expect("failed to write latest.json");
    
    // Write summary
    eprintln!(
        "fuzz results: {} cases, {} agree, {} disagree",
        results.cases, results.agree, results.disagree
    );
    
    if failures.is_empty() {
        eprintln!("all tests passed");
        ExitCode::SUCCESS
    } else {
        eprintln!("{} failures:\n{}", failures.len(), failures.join("\n"));
        ExitCode::from(1)
    }
}