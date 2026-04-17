// Simple test for the parser

fn main() {
    let result = t27_bootstrap_parser::parse("specs/00-gf-family-foundation.tri");
    match result {
        Ok(ast) => {
            println!("Parse successful!");
            println!("Modules: {}", ast.modules.len());
            for module in &ast.modules {
                println!("  Module: {}", module.name);
                println!("    Sections: {}", module.sections.len());
                for section in &module.sections {
                    println!("      {:?}", section.kind);
                }
            }
            println!("Errors: {}", ast.errors.len());
            for err in &ast.errors {
                println!("  {:?}", err);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}
