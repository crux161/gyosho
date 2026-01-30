use clap::{Parser as ClapParser, ValueEnum};
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};
use logos::Logos;

use sumic::lexer::Token;
use sumic::parser::Parser;
use sumic::codegen::{MetalGenerator, WgslGenerator, MarkdownGenerator, CodeGenerator};
use sumic::preprocessor::Preprocessor; // Ensure this is imported

#[derive(ClapParser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

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

    // 1. Preprocess (Resolve Includes)
    println!("🔍 Preprocessing...");
    let mut preprocessor = Preprocessor::new();
    
    // FIX: We capture the output here...
    let preprocessed_source = preprocessor.process(&args.input)
        .with_context(|| format!("Failed to preprocess {:?}", args.input))?;

    // 2. Lex
    // ...and we MUST pass that specific variable to the lexer!
    let lexer = Token::lexer(&preprocessed_source);
    
    let mut tokens = Vec::new();
    for (result, span) in lexer.spanned() {
        match result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                // Optional: You could print the span to debug specific invalid tokens
                // eprintln!("Lexer Error at {:?}", span);
            }
        }
    }

    // 3. Parse
    println!("🏗️ Parsing...");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("Parser Error: {}", e))?;

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
