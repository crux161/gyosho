use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::fs;
use anyhow::{Context, Result};

// --- CLI Definitions ---

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The input shader file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// The output file path (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The Kantei Hardware Grade to target
    #[arg(short, long, value_enum, default_value_t = TargetGrade::Brush)]
    grade: TargetGrade,

    /// Enable Shadertoy Compatibility Shim
    #[arg(long, default_value_t = false)]
    shadertoy: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum TargetGrade {
    Ink, Paper, Brush, Gold,
}

// --- The Shadertoy Boilerplate ---

// This mimics the hidden environment Shadertoy provides.
const SHADERTOY_HEADER: &str = r#"
#version 450
layout(location = 0) out vec4 outputColor;

// The Standard Shadertoy Uniforms (SumiContext)
layout(set = 0, binding = 0) uniform SumiContext {
    vec3 iResolution;
    float iTime;
    float iTimeDelta;
    float iFrame;
    vec4 iMouse;
    vec4 iDate;
};
"#;

// This calls the user's mainImage() function.
const SHADERTOY_FOOTER: &str = r#"
void main() {
    mainImage(outputColor, gl_FragCoord.xy);
}
"#;

// --- Main Execution ---
fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- SumiC Compiler ---");
    println!("Reading: {:?}", args.input);

    // 1. Read & Shim
    let raw_source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read shader file: {:?}", args.input))?;

    let final_source = if args.shadertoy {
        println!("✨ Applying Shadertoy Shim...");
        format!("{}\n{}\n{}", SHADERTOY_HEADER, raw_source, SHADERTOY_FOOTER)
    } else {
        raw_source
    };

    // 2. Compile (Frontend)
    let module = compile_frontend(&final_source)?;
    println!("✅ Syntax Verified.");

    // 3. Generate Output (Backend)
    // For now, we default to Metal (since you are on a Mac)
    if let Some(out_path) = args.output {
        println!("⚙️ Generating Metal (MSL)...");
        let msl_code = compile_backend_msl(&module)?;
        
        fs::write(&out_path, msl_code)
            .with_context(|| format!("Failed to write output to {:?}", out_path))?;
            
        println!("💾 Saved to: {:?}", out_path);
    } else {
        println!("⚠️ No output file specified. Dry run complete.");
    }
    
    Ok(())
}

/// FRONTEND: Text -> IR (Intermediate Representation)
fn compile_frontend(source: &str) -> Result<naga::Module> {
    let mut parser = naga::front::glsl::Frontend::default();
    let options = naga::front::glsl::Options::from(naga::ShaderStage::Fragment);
    
    // Parse to Module
    let module = parser.parse(&options, source)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        
    Ok(module)
}

/// BACKEND: IR -> Metal Shading Language
fn compile_backend_msl(module: &naga::Module) -> Result<String> {
    // 1. Create the Validation Context
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(module)
        .map_err(|e| anyhow::anyhow!("Validation Error: {:?}", e))?;

    // 2. Configure Metal Options (Translation choices)
    let options = naga::back::msl::Options {
        lang_version: (2, 0),
        ..Default::default()
    };
    
    // 3. Configure Pipeline Options (Resource binding layout)
    // FIX: This is the new required struct
    let pipeline_options = naga::back::msl::PipelineOptions::default();
    
    // 4. Translate (Now with 4 arguments)
    let (msl, _) = naga::back::msl::write_string(module, &info, &options, &pipeline_options)
        .map_err(|e| anyhow::anyhow!("Metal Generation Error: {:?}", e))?;
        
    Ok(msl)
}
