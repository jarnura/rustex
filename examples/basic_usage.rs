//! Basic RustEx usage example
//! 
//! This example demonstrates the fundamental usage of RustEx for extracting
//! AST information from a Rust project.

use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RustEx Basic Usage Example");
    println!("════════════════════════════");

    // Create default configuration
    let config = ExtractorConfig::default();
    
    // Create extractor for current directory
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    // Extract project AST
    println!("📊 Extracting AST...");
    let project_ast = extractor.extract_project()?;
    
    // Print basic information
    println!("\n✅ Extraction Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("📦 Project: {}", project_ast.project.name);
    println!("📄 Version: {}", project_ast.project.version);
    println!("🦀 Rust Edition: {}", project_ast.project.rust_edition);
    
    println!("\n📈 Metrics:");
    println!("  • Files: {}", project_ast.files.len());
    println!("  • Total lines: {}", project_ast.metrics.total_lines);
    println!("  • Functions: {}", project_ast.metrics.total_functions);
    println!("  • Structs: {}", project_ast.metrics.total_structs);
    println!("  • Enums: {}", project_ast.metrics.total_enums);
    println!("  • Traits: {}", project_ast.metrics.total_traits);
    println!("  • Average complexity: {:.2}", project_ast.metrics.complexity_average);
    
    // Show file breakdown
    println!("\n📁 File Breakdown:");
    for file in &project_ast.files {
        println!("  • {} ({} elements)", 
                file.relative_path.display(), 
                file.elements.len());
    }
    
    // Show element types
    let mut element_counts = std::collections::HashMap::new();
    for file in &project_ast.files {
        for element in &file.elements {
            *element_counts.entry(format!("{:?}", element.element_type))
                           .or_insert(0) += 1;
        }
    }
    
    if !element_counts.is_empty() {
        println!("\n🔍 Element Types:");
        for (element_type, count) in element_counts {
            println!("  • {}: {}", element_type, count);
        }
    }
    
    // Show most complex functions
    let mut complex_functions: Vec<_> = project_ast.files.iter()
        .flat_map(|f| &f.elements)
        .filter_map(|e| e.complexity.map(|c| (&e.name, c)))
        .collect();
    
    complex_functions.sort_by_key(|(_, complexity)| *complexity);
    
    if !complex_functions.is_empty() {
        println!("\n🧠 Most Complex Functions:");
        for (name, complexity) in complex_functions.iter().rev().take(5) {
            println!("  • {} (complexity: {})", name, complexity);
        }
    }
    
    println!("\n🎉 Analysis complete!");
    
    Ok(())
}