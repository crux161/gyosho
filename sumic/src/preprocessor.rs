use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use regex::Regex;

pub struct Preprocessor {
    included_files: Vec<PathBuf>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self { included_files: Vec::new() }
    }

    pub fn process(&mut self, file_path: &Path) -> Result<String> {
        self.process_recursive(file_path, 0)
    }

    fn process_recursive(&mut self, file_path: &Path, depth: usize) -> Result<String> {
        if depth > 10 {
            anyhow::bail!("Include depth limit exceeded (cycle detected?): {:?}", file_path);
        }

        let canonical = fs::canonicalize(file_path)
            .with_context(|| format!("Failed to resolve path: {:?}", file_path))?;

        // Simple cycle detection
        if self.included_files.contains(&canonical) {
             // Already included, usually we'd skip or error. 
             // For simple headers, skipping avoids duplication.
             return Ok(String::new()); 
        }
        self.included_files.push(canonical.clone());

        let content = fs::read_to_string(&canonical)
            .with_context(|| format!("Failed to read file: {:?}", canonical))?;

        let base_dir = canonical.parent().unwrap_or(Path::new("."));
        let include_regex = Regex::new(r#"#include\s+"([^"]+)""#).unwrap();

        let mut final_output = String::new();
        let mut last_pos = 0;

        for cap in include_regex.captures_iter(&content) {
            let match_str = cap.get(0).unwrap();
            let rel_path = cap.get(1).unwrap().as_str();
            
            // Append text before the #include
            final_output.push_str(&content[last_pos..match_str.start()]);
            
            // Resolve and process the included file
            let target_path = base_dir.join(rel_path);
            println!("  🔗 Including: {:?}", rel_path);
            let included_content = self.process_recursive(&target_path, depth + 1)?;
            
            final_output.push_str(&included_content);
            last_pos = match_str.end();
        }

        // Append remaining text
        final_output.push_str(&content[last_pos..]);
        
        Ok(final_output)
    }
}
