// Simple parser test

extern crate t27_bootstrap_parser;

use std::env;

fn main() {
    let args: std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: simple_parse <spec_file>");
        std::process::exit(1);
    }

    let spec_path = &args[1];
    println!("Parsing: {}", spec_path);

    let result = t27_bootstrap_parser::parse(spec_path);

    match result {
        Ok(ast) => {
            println!("\n=== Parse Successful ===");
            println!("Modules: {}", ast.modules.len());
            for module in &ast.modules {
                println!("  Module: {}", module.name);
                for section in &module.sections {
                    println!("    {:?}", section.kind);
                }
            }
            println!("Errors: {}", ast.errors.len());
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
}
