mod parser;

use std::env;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec_path = if args.len() > 1 { &args[1] } else { "specs/00-gf-family-foundation.tri" };

    println!("Parsing spec file: {}", spec_path);
    let result = parser::parse(spec_path);

    match result {
        Ok(ast) => {
            println!("\n=== Parse Successful ===");
            println!("Modules: {}", ast.modules.len());
            for module in &ast.modules {
                println!("\n  Module: {}", module.name);
                println!("    Version: {:?}", module.version);
                println!("    Ring: {:?}", module.ring);
                println!("    Seal: {:?}", module.seal);
                println!("    Source of Truth: {:?}", module.source_of_truth);
                println!("    Imports: {:?}", module.imports);
                println!("    Sections: {}", module.sections.len());

                for section in &module.sections {
                    println!("      Section: {:?}", section.kind);
                    match &section.content {
                        parser::SectionContent::ConstantsSection { constants } => {
                            println!("        Constants: {}", constants.len());
                        }
                        parser::SectionContent::TypesSection { types } => {
                            println!("        Types: {}", types.len());
                        }
                        parser::SectionContent::FunctionsSection { functions } => {
                            println!("        Functions: {}", functions.len());
                        }
                        parser::SectionContent::InvariantsSection { invariants } => {
                            println!("        Invariants: {}", invariants.len());
                        }
                        parser::SectionContent::TestsSection { tests } => {
                            println!("        Tests: {}", tests.len());
                        }
                        parser::SectionContent::BenchmarksSection { benchmarks } => {
                            println!("        Benchmarks: {}", benchmarks.len());
                        }
                        parser::SectionContent::NumericTowerSection { rules } => {
                            println!("        Numeric Tower Rules: {}", rules.len());
                        }
                        parser::SectionContent::PromotionSection { rules } => {
                            println!("        Promotion Rules: {}", rules.len());
                        }
                        parser::SectionContent::ExperienceSection { hooks } => {
                            println!("        Experience Hooks: {}", hooks.len());
                        }
                        parser::SectionContent::ImportsSection { imports } => {
                            println!("        Imports: {}", imports.len());
                        }
                    }
                }
            }

            println!("\nErrors: {}", ast.errors.len());
            for err in &ast.errors {
                println!("  {:?}", err);
            }
        }
        Err(e) => {
            eprintln!("\n=== Parse Error ===");
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
