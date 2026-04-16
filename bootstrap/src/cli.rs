//! Trinity Bootstrap CLI - Ring-013
//! Unified dispatcher for all CLI commands (pipeline, test, bench, experience, parse)
//! v0.1.0: φ-native CLI with tri experience save/load/list/diff/evolve

use anyhow::Result;
use std::env;
use std::fs;

/// CLI version (semantic versioning)
const MAJOR: u8 = 0;
const MINOR: u8 = 1;
const PATCH: u8 = 0;

/// Command kind enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    PIPELINE = 0,
    TEST = 1,
    BENCH = 2,
    PARSE = 3,
    EXPERIENCE_SAVE = 4,
    EXPERIENCE_LIST = 5,
    EXPERIENCE_DIFF = 6,
    EXPERIENCE_EVOLVE = 7,
    VERSION = 8,
    HELP = 9,
}

/// CLI command representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    kind: CommandKind,
    args_count: u8,
}

/// Parse arguments into structured Command
fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.is_empty() {
        return Err("Usage: tri <command> [args...]".to_string());
    }

    let first = &args[0].to_lowercase();

    match first.as_str() {
        "pipeline" => Ok(Command {
            kind: CommandKind::PIPELINE,
            args_count: args.len() as u8,
        }),
        "test" => Ok(Command {
            kind: CommandKind::TEST,
            args_count: args.len() as u8,
        }),
        "bench" => Ok(Command {
            kind: CommandKind::BENCH,
            args_count: args.len() as u8,
        }),
        "parse" => Ok(Command {
            kind: CommandKind::PARSE,
            args_count: args.len() as u8,
        }),
        "experience" => {
            if args.len() < 2 {
                return Err("experience requires <command> [save|list|diff|evolve]".to_string());
            }
            match args[1].to_lowercase().as_str() {
                "save" => Ok(Command {
                    kind: CommandKind::EXPERIENCE_SAVE,
                    args_count: args.len() as u8,
                }),
                "list" => Ok(Command {
                    kind: CommandKind::EXPERIENCE_LIST,
                    args_count: args.len() as u8,
                }),
                "diff" => Ok(Command {
                    kind: CommandKind::EXPERIENCE_DIFF,
                    args_count: args.len() as u8,
                }),
                "evolve" => Ok(Command {
                    kind: CommandKind::EXPERIENCE_EVOLVE,
                    args_count: args.len() as u8,
                }),
                _ => Err(format!("Unknown experience command: {}", args[1])),
            }
        },
        "--version" => Ok(Command {
            kind: CommandKind::VERSION,
            args_count: args.len() as u8,
        }),
        "--help" => Ok(Command {
            kind: CommandKind::HELP,
            args_count: args.len() as u8,
        }),
        _ => Err(format!("Unknown command: {}", args[0])),
    }
}

fn parse_args_and_dispatch(args: Vec<String>) -> Result<(), String> {
    let command = parse_args(&args)?;

    match command.kind {
        CommandKind::PIPELINE => pipeline_run(&args[1..])?,
        CommandKind::TEST => test_run(&args[1..])?,
        CommandKind::BENCH => bench_run(&args[1..])?,
        CommandKind::PARSE => parse_run(&args[1..])?,
        CommandKind::EXPERIENCE_SAVE => experience_save_run(&args[1..])?,
        CommandKind::EXPERIENCE_LIST => experience_list_run()?,
        CommandKind::EXPERIENCE_DIFF => experience_diff_run(&args[1..])?,
        CommandKind::EXPERIENCE_EVOLVE => experience_evolve_run()?,
        CommandKind::VERSION => version_run(),
        CommandKind::HELP => help_run(),
    }
}

/// Pipeline run: parse → codegen → execute
fn pipeline_run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("pipeline requires <spec-file.tri>".to_string());
    }

    let spec_file = &args[1];

    println!("[Pipeline] Parsing: {}", spec_file);
    println!("[Pipeline] Codegen...");
    println!("[Pipeline] Executing...");

    // Stubs for now (full pipeline to be implemented later)
    // TODO: Integrate with t27c parser, codegen, VM executor
    println!("[Pipeline] Complete: {} parsed → codegen → execute (stubs)", spec_file);

    Ok(())
}

/// Test run: execute tests from .tri spec
fn test_run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("test requires <spec-file.tri>".to_string());
    }

    let spec_file = &args[1];

    println!("[Test] Running tests: {}", spec_file);
    println!("[Test] {} / 8 passing (stubs for now)", spec_file);

    // Stubs for now (test runner to be implemented later)
    // TODO: Integrate with t27c parser, test runner, VM executor
    println!("[Test] Complete: {} tests passed (stubs for now)", spec_file);

    Ok(())
}

/// Benchmark run: run benchmarks from .tri spec
fn bench_run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("bench requires <spec-file.tri>".to_string());
    }

    let spec_file = &args[1];

    println!("[Bench] Running benchmarks: {}", spec_file);
    println!("[Bench] {} / 2 benchmarks (stubs for now)", spec_file);

    // Stubs for now (benchmark runner to be implemented later)
    // TODO: Integrate with t27c parser, benchmark runner, VM executor
    println!("[Bench] Complete: {} benchmarks (stubs for now)", spec_file);

    Ok(())
}

/// Parse: validate .tri syntax
fn parse_run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("parse requires <spec-file.tri>".to_string());
    }

    let spec_file = &args[1];

    println!("[Parse] Validating: {}", spec_file);

    // Stubs for now (t27c parser to be integrated later)
    // TODO: Integrate with t27c parser
    println!("[Parse] Complete: {} valid (stubs for now)", spec_file);

    Ok(())
}

/// Experience save: save skill/payload to .trinity/experience/
fn experience_save_run(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("save requires <skill> <payload>".to_string());
    }

    let skill = &args[1];
    let payload = &args[2];

    println!("[Save] Skill: {}", skill);
    println!("[Save] Payload: {}", payload);
    println!("[Save] Ring: 010");

    // Build JSON payload
    let timestamp = chrono::Utc::now().to_rfc3339();
    let json_payload = format!(
        r#"{{"skill": "{}", "payload": "{}", "timestamp": "{}", "ring": "010", "commit": "current"}}"#,
        skill, payload, timestamp
    );

    // Write to .trinity/experience/episodes.jsonl
    fs::create_dir_all(".trinity/experience")?;
    let path = ".trinity/experience/episodes.jsonl";
    let existing = fs::read_to_string(&path)
        .unwrap_or_else(|_| String::new())
        + &"\n"
        + &json_payload;

    fs::write(&path, &existing)?;
    println!("[Save] Saved: {}@{}", skill, timestamp);

    Ok(())
}

/// Experience list: list all experiences from .trinity/experience/
fn experience_list_run() -> Result<(), String> {
    println!("[List] All experiences:");
    println!("[List] To be implemented: read .trinity/experience/episodes.jsonl");

    // Stubs for now (experience list to be implemented later)
    // TODO: Integrate with bootstrap/src/experience.rs
    println!("[List] Complete (stubs for now)");

    Ok(())
}

/// Experience diff: compare old vs new benchmarks
fn experience_diff_run(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err("diff requires <skill> <old_bench> <new_bench>".to_string());
    }

    let skill = &args[1];
    let old_bench = args[2].parse::<f64>()
        .map_err(|e| format!("Failed to parse old_bench: {}", e))?;
    let new_bench = args[3].parse::<f64>()
        .map_err(|e| format!("Failed to parse new_bench: {}", e))?;

    let delta_pct = if old_bench > 0.0 {
        ((new_bench - old_bench).abs() / old_bench * 100.0)
    } else {
        0.0
    };

    let verdict = if new_bench >= 20000000.0 {
        "IMPROVED"
    } else if delta_pct > 10.0 {
        "DEGRADED"
    } else {
        "STABLE"
    };

    println!("[Diff] Skill: {}", skill);
    println!("[Diff] Old: {} ops/s", old_bench);
    println!("[Diff] New: {} ops/s", new_bench);
    println!("[Diff] Delta: {}%", delta_pct);
    println!("[Diff] Verdict: {}", verdict);

    // Stubs for now (experience diff to be implemented later)
    // TODO: Integrate with bootstrap/src/experience.rs
    println!("[Diff] Complete (stubs for now)");

    Ok(())
}

/// Experience evolve: ASHA+PBT - compare all skills across sessions
fn experience_evolve_run(_args: &[String]) -> Result<(), String> {
    println!("[Evolve] ASHA+PBT algorithm");
    println!("[Evolve] Comparing all skills across sessions...");
    println!("[Evolve] To be implemented: read .trinity/experience/episodes.jsonl");
    println!("[Evolve] ASHA+PBT (12 skills, improved/stable/degraded distribution)");

    // Stubs for now (experience evolve to be implemented later)
    // TODO: Integrate with bootstrap/src/experience.rs
    println!("[Evolve] Complete (stubs for now)");

    Ok(())
}

/// CLI version output
fn version_run() {
    println!("trinity v{}.{}.{} | φ²+1/φ²=3 | TRIB=0x54524942",
              MAJOR, MINOR, PATCH);
    println!();
}

/// Help display
fn help_run() {
    println!(r#"
tri v{}.{}.{} — Trinity Language CLI
φ² + 1/φ² = 3 | TRIB=0x54524942

COMMANDS:
  pipeline <file.tri>              Run .tri → .trib → execute (E2E pipeline)
  test     <file.tri>              Run tests from .tri spec
  bench    <file.tri>              Run benchmarks from .tri spec
  parse    <file.tri>              Validate .tri syntax
  experience save <skill> <payload>    Save experience episode
  experience list                    List all experiences
  experience diff <skill> <old> <new>    Compare benchmarks
  experience evolve                  ASHA+PBT: compare all skills

EXAMPLES:
  tri pipeline specs/00-gf-family-foundation.tri
  tri test specs/00-gf-family-foundation.tri
  tri bench specs/00-gf-family-foundation.tri
  tri experience save "ring-013-done" "v0.1.0 released"

OPTIONS:
  --help    Show this message
  --version Show semantic version

CURRENT VERSION: v0.1.0
"#);
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    parse_args_and_dispatch(&args)?;
}
