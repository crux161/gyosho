use clap::{Parser as ClapParser, ValueEnum};
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};
use logos::Logos;

use sumic::lexer::Token;
use sumic::parser::Parser;
use sumic::codegen::{MetalGenerator, WgslGenerator, MarkdownGenerator, CodeGenerator}; // Added WgslGenerator

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output Format
    #[arg(short, long, value_enum, default_value_t = Target::Wgsl)]
    format: Target,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Target {
    Metal,
    Wgsl,
    Markdown,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("--- SumiC Compiler ---");

    // 1. Read
    let raw_source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read {:?}", args.input))?;

    // 2. Lex
    let lexer = Token::lexer(&raw_source);
    let tokens: Vec<_> = lexer.filter_map(|r| r.ok()).collect(); // Simplified for brevity

    // 3. Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| anyhow::anyhow!("{}", e))?;

    // 4. Generate
    let code = match args.format {
        Target::Metal => {
            println!("⚙️ Generating Metal...");
            MetalGenerator::new(false).generate(&ast)
        },
        Target::Wgsl => {
            println!("⚙️ Generating WGSL...");
            WgslGenerator::new().generate(&ast)
        },
        Target::Markdown => {
            println!("📄 Generating Docs...");
            MarkdownGenerator.generate(&ast)
        }
    };

    // 5. Output
    if let Some(out_path) = args.output {
        fs::write(&out_path, &code)?;
        println!("💾 Saved to {:?}", out_path);
    } else {
        println!("\n{}\n", code);
    }

    Ok(())
}
