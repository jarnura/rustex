//! Documentation Generator Example
//! 
//! This example shows how to use RustEx to generate comprehensive
//! documentation for a Rust project, including API documentation
//! and metrics summaries.

use rustex_core::{
    AstExtractor, ExtractorConfig, ConfigUseCase, 
    ElementType, Visibility
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 RustEx Documentation Generator");
    println!("════════════════════════════════");

    // Use documentation template configuration
    let config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    println!("🔍 Analyzing project...");
    let project_ast = extractor.extract_project()?;
    
    // Generate comprehensive documentation
    let mut doc = String::new();
    
    // Header
    doc.push_str(&format!("# {} Documentation\n\n", project_ast.project.name));
    
    // Project overview
    doc.push_str("## Overview\n\n");
    doc.push_str(&format!("**Version:** {}\n", project_ast.project.version));
    doc.push_str(&format!("**Rust Edition:** {}\n", project_ast.project.rust_edition));
    doc.push_str(&format!("**Generated:** {}\n\n", 
        project_ast.extracted_at.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Project metrics
    doc.push_str("## Project Metrics\n\n");
    doc.push_str(&format!("- **Total Files:** {}\n", project_ast.metrics.total_files));
    doc.push_str(&format!("- **Total Lines:** {}\n", project_ast.metrics.total_lines));
    doc.push_str(&format!("- **Functions:** {}\n", project_ast.metrics.total_functions));
    doc.push_str(&format!("- **Structs:** {}\n", project_ast.metrics.total_structs));
    doc.push_str(&format!("- **Enums:** {}\n", project_ast.metrics.total_enums));
    doc.push_str(&format!("- **Traits:** {}\n", project_ast.metrics.total_traits));
    doc.push_str(&format!("- **Average Complexity:** {:.2}\n\n", project_ast.metrics.complexity_average));
    
    // Table of contents for modules
    doc.push_str("## Table of Contents\n\n");
    let mut modules: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for file in &project_ast.files {
        for element in &file.elements {
            if element.element_type == ElementType::Module {
                modules.insert(element.hierarchy.qualified_name.clone());
            }
        }
    }
    
    for module in &modules {
        doc.push_str(&format!("- [{}](#{})\n", module, 
            module.replace("::", "").replace("_", "-").to_lowercase()));
    }
    doc.push('\n');
    
    // Public API section
    doc.push_str("## Public API\n\n");
    
    for file in &project_ast.files {
        let public_items: Vec<_> = file.elements.iter()
            .filter(|e| e.visibility == Visibility::Public)
            .collect();
            
        if !public_items.is_empty() {
            doc.push_str(&format!("### {}\n\n", file.relative_path.display()));
            
            // Group by element type
            let mut functions = Vec::new();
            let mut structs = Vec::new();
            let mut enums = Vec::new();
            let mut traits = Vec::new();
            let mut others = Vec::new();
            
            for element in public_items {
                match element.element_type {
                    ElementType::Function => functions.push(element),
                    ElementType::Struct => structs.push(element),
                    ElementType::Enum => enums.push(element),
                    ElementType::Trait => traits.push(element),
                    _ => others.push(element),
                }
            }
            
            // Document structs
            if !structs.is_empty() {
                doc.push_str("#### Structs\n\n");
                for element in structs {
                    document_element(&mut doc, element);
                }
            }
            
            // Document enums
            if !enums.is_empty() {
                doc.push_str("#### Enums\n\n");
                for element in enums {
                    document_element(&mut doc, element);
                }
            }
            
            // Document traits
            if !traits.is_empty() {
                doc.push_str("#### Traits\n\n");
                for element in traits {
                    document_element(&mut doc, element);
                }
            }
            
            // Document functions
            if !functions.is_empty() {
                doc.push_str("#### Functions\n\n");
                for element in functions {
                    document_element(&mut doc, element);
                }
            }
            
            // Document other elements
            if !others.is_empty() {
                doc.push_str("#### Other Items\n\n");
                for element in others {
                    document_element(&mut doc, element);
                }
            }
        }
    }
    
    // Implementation details section (private items if included)
    let private_items: Vec<_> = project_ast.files.iter()
        .flat_map(|f| &f.elements)
        .filter(|e| e.visibility != Visibility::Public)
        .collect();
    
    if !private_items.is_empty() {
        doc.push_str("## Implementation Details\n\n");
        doc.push_str("*This section documents private implementation details.*\n\n");
        
        // Group private items by file
        let mut private_by_file: std::collections::BTreeMap<&PathBuf, Vec<_>> = 
            std::collections::BTreeMap::new();
        
        for file in &project_ast.files {
            let file_private_items: Vec<_> = file.elements.iter()
                .filter(|e| e.visibility != Visibility::Public)
                .collect();
            
            if !file_private_items.is_empty() {
                private_by_file.insert(&file.relative_path, file_private_items);
            }
        }
        
        for (file_path, items) in private_by_file {
            doc.push_str(&format!("### {} (Private)\n\n", file_path.display()));
            
            for element in items {
                doc.push_str(&format!("- **{}** `{}` ", 
                    format!("{:?}", element.element_type), element.name));
                
                if let Some(complexity) = element.complexity {
                    doc.push_str(&format!("*(complexity: {})*", complexity));
                }
                doc.push('\n');
                
                if !element.doc_comments.is_empty() {
                    doc.push_str(&format!("  {}\n", 
                        element.doc_comments.join(" ").trim()));
                }
            }
            doc.push('\n');
        }
    }
    
    // Complexity analysis
    doc.push_str("## Complexity Analysis\n\n");
    
    let mut complexity_distribution: std::collections::BTreeMap<u32, u32> = 
        std::collections::BTreeMap::new();
    let mut high_complexity_items = Vec::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            if let Some(complexity) = element.complexity {
                *complexity_distribution.entry(complexity).or_insert(0) += 1;
                
                if complexity > 5 {
                    high_complexity_items.push((
                        element.name.clone(),
                        complexity,
                        file.relative_path.clone(),
                        element.element_type.clone(),
                    ));
                }
            }
        }
    }
    
    // Complexity distribution table
    doc.push_str("### Complexity Distribution\n\n");
    doc.push_str("| Complexity | Count |\n");
    doc.push_str("|------------|-------|\n");
    for (complexity, count) in complexity_distribution {
        doc.push_str(&format!("| {} | {} |\n", complexity, count));
    }
    doc.push('\n');
    
    // High complexity items
    if !high_complexity_items.is_empty() {
        high_complexity_items.sort_by_key(|(_, complexity, _, _)| *complexity);
        
        doc.push_str("### High Complexity Items (>5)\n\n");
        doc.push_str("| Item | Type | Complexity | File |\n");
        doc.push_str("|------|------|------------|------|\n");
        
        for (name, complexity, file, element_type) in high_complexity_items.iter().rev() {
            doc.push_str(&format!("| `{}` | {:?} | {} | {} |\n", 
                name, element_type, complexity, file.display()));
        }
        doc.push('\n');
    }
    
    // Cross-references section
    if !project_ast.cross_references.is_empty() {
        doc.push_str("## Cross-References\n\n");
        doc.push_str(&format!("Total cross-references: {}\n\n", 
            project_ast.cross_references.len()));
        
        // Group by reference type
        let mut ref_counts: std::collections::HashMap<String, u32> = 
            std::collections::HashMap::new();
        
        for cross_ref in &project_ast.cross_references {
            *ref_counts.entry(format!("{:?}", cross_ref.reference_type))
                       .or_insert(0) += 1;
        }
        
        doc.push_str("### Reference Types\n\n");
        for (ref_type, count) in ref_counts {
            doc.push_str(&format!("- **{}**: {} references\n", ref_type, count));
        }
        doc.push('\n');
    }
    
    // Footer
    doc.push_str("---\n\n");
    doc.push_str(&format!("*Generated by RustEx on {}*\n", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Save documentation
    let output_file = "PROJECT_DOCUMENTATION.md";
    std::fs::write(output_file, doc)?;
    
    println!("✅ Documentation generated successfully!");
    println!("📄 Output file: {}", output_file);
    println!("📊 Documented {} public items across {} files", 
        project_ast.files.iter()
            .flat_map(|f| &f.elements)
            .filter(|e| e.visibility == Visibility::Public)
            .count(),
        project_ast.files.len());
    
    Ok(())
}

fn document_element(doc: &mut String, element: &rustex_core::CodeElement) {
    doc.push_str(&format!("##### `{}`\n\n", element.name));
    
    // Add documentation
    if !element.doc_comments.is_empty() {
        for comment in &element.doc_comments {
            doc.push_str(&format!("> {}\n", comment));
        }
        doc.push('\n');
    }
    
    // Add signature
    if let Some(signature) = &element.signature {
        doc.push_str("```rust\n");
        doc.push_str(signature);
        doc.push_str("\n```\n\n");
    }
    
    // Add complexity info
    if let Some(complexity) = element.complexity {
        doc.push_str(&format!("**Complexity:** {}\n\n", complexity));
    }
    
    // Add location info
    doc.push_str(&format!("**Location:** {}:{}\n\n", 
        element.location.file_path.display(), 
        element.location.line_start));
}