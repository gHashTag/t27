//! Trinity Bootstrap Compiler - Ring-011
//! Experience Save CLI (infinite memory for Trinity)
//! 5th Unfair Advantage: Collective intelligence via shared experience/

use anyhow::Result;
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        println!("tri experience CLI - Ring-011");
        println!("\nCommands:");
        println!("  save    <skill> <payload>    Save experience episode");
        println!("  list    List all experiences");
        println!("  diff    <skill> <old_bench> <new_bench>    Compare benchmarks");
        println!("  evolve  ASHA+PBT: compare all skills across sessions");
        std::process::exit(1);
    }

    let command = &args[0];

    match command.as_str() {
        "save" => {
            if args.len() < 3 {
                eprintln!("save requires: <skill> <payload>");
                return Err(anyhow::anyhow!("Missing arguments"));
            }
            let skill = &args[1];
            let payload = &args[2];

            println!("Saving experience:");
            println!("  skill: {}", skill);
            println!("  payload: {}", payload);
            println!("  ring: 010");
            println!("  commit: current");

            let experience = format!(
                "{{\"skill\": \"{}\", \"payload\": \"{}\", \"timestamp\": \"{}\", \"ring\": \"010\", \"commit\": \"current\"}}",
                skill, payload, chrono::Utc::now().to_rfc3339()
            );

            fs::create_dir_all(".trinity/experience")?;
            let path = ".trinity/experience/episodes.jsonl";
            let existing = fs::read_to_string(&path)
                .unwrap_or_else(|_| String::new())
                + &"\n"
                + &experience;

            fs::write(&path, &existing)?;
            println!("Experience saved!");
        }

        "list" => {
            println!("Experience List (stub):");
            println!("To be implemented: tri experience list");
            println!("Lists all entries from .trinity/experience/episodes.jsonl");
        }

        "diff" => {
            if args.len() < 4 {
                eprintln!("diff requires: <skill> <old_bench> <new_bench>");
                return Err(anyhow::anyhow!("Missing arguments"));
            }
            let skill = &args[1];
            let old_bench = args[2].parse::<f64>()
                .map_err(|e| anyhow::anyhow!("Failed to parse old_bench: {}", e))?;
            let new_bench = args[3].parse::<f64>()
                .map_err(|e| anyhow::anyhow!("Failed to parse new_bench: {}", e))?;

            let delta_pct = if *old_bench > 0.0 {
                ((new_bench - old_bench) / *old_bench * 100.0).abs()
            } else {
                0.0
            };

            let verdict = if *new_bench >= 20000000.0 {
                "IMPROVED"
            } else if delta_pct > 10.0 {
                "DEGRADED"
            } else {
                "STABLE"
            };

            println!("Skill: {}", skill);
            println!("Old: {} ops/s", old_bench);
            println!("New: {} ops/s", new_bench);
            println!("Delta: {}%", delta_pct);
            println!("Verdict: {}", verdict);
        }

        "evolve" => {
            println!("ASAP+PBT Evolve (stub):");
            println!("To be implemented: tri experience evolve");
            println!("Compares all skills across sessions using ASHA+PBT algorithm");
        }

        _ => {
            eprintln!("Unknown command: {}. Try 'save', 'list', 'diff', or 'evolve'", command);
            return Err(anyhow::anyhow!("Unknown command: {}", command));
        }
    }

    Ok(())
}
