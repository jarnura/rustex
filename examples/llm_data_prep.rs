//! LLM Training Data Preparation Example
//! 
//! This example demonstrates how to use RustEx to prepare high-quality
//! training data for language models, including chunking, formatting,
//! and metadata generation.

use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase, ElementType, Visibility};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 RustEx LLM Training Data Preparation");
    println!("══════════════════════════════════════");

    // Use LLM training configuration
    let config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    println!("📊 Extracting and processing project...");
    let project_ast = extractor.extract_project()?;
    
    // Generate different types of training data
    let training_data = generate_training_data(&project_ast)?;
    
    // Save training data in various formats
    save_training_data(&training_data)?;
    
    // Generate statistics
    print_statistics(&training_data);
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingDataset {
    metadata: DatasetMetadata,
    examples: Vec<TrainingExample>,
    chunks: Vec<CodeChunk>,
    qa_pairs: Vec<QAPair>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatasetMetadata {
    name: String,
    version: String,
    language: String,
    domain: String,
    total_examples: usize,
    total_chunks: usize,
    total_qa_pairs: usize,
    complexity_distribution: std::collections::HashMap<String, usize>,
    generated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingExample {
    id: String,
    example_type: String,
    input: String,
    output: String,
    metadata: ExampleMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExampleMetadata {
    element_type: String,
    element_name: String,
    file_path: String,
    complexity: Option<u32>,
    visibility: String,
    has_documentation: bool,
    difficulty_level: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeChunk {
    id: String,
    content: String,
    context: String,
    metadata: ChunkMetadata,
    embedding_hint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChunkMetadata {
    file_path: String,
    element_types: Vec<String>,
    complexity_range: (u32, u32),
    token_count: usize,
    semantic_category: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct QAPair {
    id: String,
    question: String,
    answer: String,
    context: String,
    difficulty: String,
    category: String,
}

fn generate_training_data(project_ast: &rustex_core::ProjectAst) -> Result<TrainingDataset, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();
    let mut chunks = Vec::new();
    let mut qa_pairs = Vec::new();
    let mut complexity_distribution = std::collections::HashMap::new();
    
    let mut example_id = 0;
    let mut chunk_id = 0;
    let mut qa_id = 0;
    
    for file in &project_ast.files {
        for element in &file.elements {
            // Generate code examples
            if let Some(training_example) = create_training_example(element, file, &mut example_id) {
                // Track complexity distribution
                let complexity_level = get_complexity_level(element.complexity);
                *complexity_distribution.entry(complexity_level).or_insert(0) += 1;
                
                examples.push(training_example);
            }
            
            // Generate code chunks for RAG
            if should_include_in_chunks(element) {
                if let Some(chunk) = create_code_chunk(element, file, &mut chunk_id) {
                    chunks.push(chunk);
                }
            }
            
            // Generate Q&A pairs from documentation
            if !element.doc_comments.is_empty() {
                qa_pairs.extend(create_qa_pairs(element, file, &mut qa_id));
            }
        }
    }
    
    // Create dataset metadata
    let metadata = DatasetMetadata {
        name: project_ast.project.name.clone(),
        version: project_ast.project.version.clone(),
        language: "rust".to_string(),
        domain: "systems_programming".to_string(),
        total_examples: examples.len(),
        total_chunks: chunks.len(),
        total_qa_pairs: qa_pairs.len(),
        complexity_distribution,
        generated_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(TrainingDataset {
        metadata,
        examples,
        chunks,
        qa_pairs,
    })
}

fn create_training_example(
    element: &rustex_core::CodeElement,
    file: &rustex_core::FileAst,
    id: &mut usize,
) -> Option<TrainingExample> {
    let example_type = match element.element_type {
        ElementType::Function => "function_implementation",
        ElementType::Struct => "struct_definition",
        ElementType::Enum => "enum_definition",
        ElementType::Trait => "trait_definition",
        _ => return None,
    };
    
    // Create different types of training examples
    let (input, output) = match element.element_type {
        ElementType::Function => create_function_example(element),
        ElementType::Struct => create_struct_example(element),
        ElementType::Enum => create_enum_example(element),
        ElementType::Trait => create_trait_example(element),
        _ => return None,
    };
    
    let difficulty_level = determine_difficulty_level(element);
    let tags = generate_tags(element);
    
    let metadata = ExampleMetadata {
        element_type: format!("{:?}", element.element_type),
        element_name: element.name.clone(),
        file_path: file.relative_path.to_string_lossy().to_string(),
        complexity: element.complexity,
        visibility: format!("{:?}", element.visibility),
        has_documentation: !element.doc_comments.is_empty(),
        difficulty_level,
        tags,
    };
    
    *id += 1;
    Some(TrainingExample {
        id: format!("example_{}", id),
        example_type: example_type.to_string(),
        input,
        output,
        metadata,
    })
}

fn create_function_example(element: &rustex_core::CodeElement) -> (String, String) {
    let input = if !element.doc_comments.is_empty() {
        format!(
            "Given this Rust function documentation:\n\n{}\n\nGenerate the function signature:",
            element.doc_comments.join("\n")
        )
    } else {
        format!("Generate documentation for this Rust function:\n\n```rust\n{}\n```", 
                element.signature.as_ref().unwrap_or(&element.name))
    };
    
    let output = if !element.doc_comments.is_empty() && element.signature.is_some() {
        element.signature.as_ref().unwrap().clone()
    } else if !element.doc_comments.is_empty() {
        element.doc_comments.join("\n")
    } else {
        format!("/// Function: {}\npub fn {}() {{\n    // Implementation\n}}", 
                element.name, element.name)
    };
    
    (input, output)
}

fn create_struct_example(element: &rustex_core::CodeElement) -> (String, String) {
    let input = format!(
        "Create a Rust struct named '{}' with appropriate documentation:",
        element.name
    );
    
    let output = if let Some(signature) = &element.signature {
        if !element.doc_comments.is_empty() {
            format!("{}\n{}", element.doc_comments.join("\n"), signature)
        } else {
            signature.clone()
        }
    } else {
        format!("/// Struct: {}\npub struct {} {{\n    // fields\n}}", 
                element.name, element.name)
    };
    
    (input, output)
}

fn create_enum_example(element: &rustex_core::CodeElement) -> (String, String) {
    let input = format!(
        "Design a Rust enum named '{}' with variants and documentation:",
        element.name
    );
    
    let output = if let Some(signature) = &element.signature {
        if !element.doc_comments.is_empty() {
            format!("{}\n{}", element.doc_comments.join("\n"), signature)
        } else {
            signature.clone()
        }
    } else {
        format!("/// Enum: {}\npub enum {} {{\n    // variants\n}}", 
                element.name, element.name)
    };
    
    (input, output)
}

fn create_trait_example(element: &rustex_core::CodeElement) -> (String, String) {
    let input = format!(
        "Define a Rust trait named '{}' with methods and documentation:",
        element.name
    );
    
    let output = if let Some(signature) = &element.signature {
        if !element.doc_comments.is_empty() {
            format!("{}\n{}", element.doc_comments.join("\n"), signature)
        } else {
            signature.clone()
        }
    } else {
        format!("/// Trait: {}\npub trait {} {{\n    // methods\n}}", 
                element.name, element.name)
    };
    
    (input, output)
}

fn create_code_chunk(
    element: &rustex_core::CodeElement,
    file: &rustex_core::FileAst,
    id: &mut usize,
) -> Option<CodeChunk> {
    let content = if let Some(signature) = &element.signature {
        if !element.doc_comments.is_empty() {
            format!("{}\n\n{}", element.doc_comments.join("\n"), signature)
        } else {
            signature.clone()
        }
    } else {
        element.name.clone()
    };
    
    let context = format!(
        "File: {}, Module: {}, Element: {} ({})",
        file.relative_path.display(),
        element.hierarchy.module_path,
        element.name,
        format!("{:?}", element.element_type)
    );
    
    let complexity_range = if let Some(complexity) = element.complexity {
        (complexity, complexity)
    } else {
        (0, 0)
    };
    
    let semantic_category = match element.element_type {
        ElementType::Function => "function_definition",
        ElementType::Struct => "data_structure",
        ElementType::Enum => "data_structure",
        ElementType::Trait => "interface_definition",
        ElementType::Module => "module_organization",
        _ => "other",
    }.to_string();
    
    let embedding_hint = format!(
        "Rust {} {} with complexity {} in module {}",
        format!("{:?}", element.element_type).to_lowercase(),
        element.name,
        element.complexity.unwrap_or(0),
        element.hierarchy.module_path
    );
    
    let metadata = ChunkMetadata {
        file_path: file.relative_path.to_string_lossy().to_string(),
        element_types: vec![format!("{:?}", element.element_type)],
        complexity_range,
        token_count: estimate_token_count(&content),
        semantic_category,
    };
    
    *id += 1;
    Some(CodeChunk {
        id: format!("chunk_{}", id),
        content,
        context,
        metadata,
        embedding_hint,
    })
}

fn create_qa_pairs(
    element: &rustex_core::CodeElement,
    file: &rustex_core::FileAst,
    id: &mut usize,
) -> Vec<QAPair> {
    let mut qa_pairs = Vec::new();
    
    if element.doc_comments.is_empty() {
        return qa_pairs;
    }
    
    let documentation = element.doc_comments.join("\n");
    let context = format!(
        "Element: {} in {}, Type: {:?}",
        element.name,
        file.relative_path.display(),
        element.element_type
    );
    
    // Generate different types of Q&A pairs
    
    // 1. What does this function/struct/etc. do?
    *id += 1;
    qa_pairs.push(QAPair {
        id: format!("qa_{}", id),
        question: format!("What does the {} `{}` do?", 
                         format!("{:?}", element.element_type).to_lowercase(), 
                         element.name),
        answer: documentation.clone(),
        context: context.clone(),
        difficulty: determine_difficulty_level(element),
        category: "functionality".to_string(),
    });
    
    // 2. How to use this element?
    if element.visibility == Visibility::Public {
        *id += 1;
        qa_pairs.push(QAPair {
            id: format!("qa_{}", id),
            question: format!("How do you use the {} `{}`?", 
                             format!("{:?}", element.element_type).to_lowercase(), 
                             element.name),
            answer: if let Some(signature) = &element.signature {
                format!("```rust\n{}\n```\n\n{}", signature, documentation)
            } else {
                documentation.clone()
            },
            context: context.clone(),
            difficulty: determine_difficulty_level(element),
            category: "usage".to_string(),
        });
    }
    
    // 3. What are the parameters/fields? (for functions/structs)
    if element.element_type == ElementType::Function && element.signature.is_some() {
        *id += 1;
        qa_pairs.push(QAPair {
            id: format!("qa_{}", id),
            question: format!("What are the parameters of the function `{}`?", element.name),
            answer: format!("Function signature: {}\n\n{}", 
                           element.signature.as_ref().unwrap(), 
                           documentation),
            context: context.clone(),
            difficulty: "intermediate".to_string(),
            category: "parameters".to_string(),
        });
    }
    
    qa_pairs
}

fn should_include_in_chunks(element: &rustex_core::CodeElement) -> bool {
    // Include public items and well-documented private items
    element.visibility == Visibility::Public || !element.doc_comments.is_empty()
}

fn get_complexity_level(complexity: Option<u32>) -> String {
    match complexity {
        Some(c) if c <= 2 => "simple".to_string(),
        Some(c) if c <= 5 => "moderate".to_string(),
        Some(c) if c <= 10 => "complex".to_string(),
        Some(_) => "very_complex".to_string(),
        None => "unknown".to_string(),
    }
}

fn determine_difficulty_level(element: &rustex_core::CodeElement) -> String {
    let complexity = element.complexity.unwrap_or(1);
    let has_generics = !element.generic_params.is_empty();
    let has_docs = !element.doc_comments.is_empty();
    
    if complexity <= 2 && !has_generics {
        "beginner".to_string()
    } else if complexity <= 5 && has_docs {
        "intermediate".to_string()
    } else if complexity > 10 || has_generics {
        "advanced".to_string()
    } else {
        "intermediate".to_string()
    }
}

fn generate_tags(element: &rustex_core::CodeElement) -> Vec<String> {
    let mut tags = vec![
        format!("{:?}", element.element_type).to_lowercase(),
        format!("{:?}", element.visibility).to_lowercase(),
    ];
    
    if !element.generic_params.is_empty() {
        tags.push("generics".to_string());
    }
    
    if !element.doc_comments.is_empty() {
        tags.push("documented".to_string());
    }
    
    if let Some(complexity) = element.complexity {
        if complexity > 5 {
            tags.push("complex".to_string());
        }
    }
    
    if !element.attributes.is_empty() {
        tags.push("attributes".to_string());
    }
    
    tags
}

fn estimate_token_count(text: &str) -> usize {
    // Rough estimation: ~4 characters per token
    (text.len() / 4).max(1)
}

fn save_training_data(dataset: &TrainingDataset) -> Result<(), Box<dyn std::error::Error>> {
    // Save complete dataset
    let dataset_json = serde_json::to_string_pretty(dataset)?;
    std::fs::write("llm-training-dataset.json", dataset_json)?;
    
    // Save examples in JSONL format (common for training)
    let mut examples_jsonl = String::new();
    for example in &dataset.examples {
        examples_jsonl.push_str(&serde_json::to_string(example)?);
        examples_jsonl.push('\n');
    }
    std::fs::write("training-examples.jsonl", examples_jsonl)?;
    
    // Save Q&A pairs separately
    let qa_json = serde_json::to_string_pretty(&dataset.qa_pairs)?;
    std::fs::write("qa-pairs.json", qa_json)?;
    
    // Save chunks for RAG
    let chunks_json = serde_json::to_string_pretty(&dataset.chunks)?;
    std::fs::write("rag-chunks.json", chunks_json)?;
    
    // Save metadata
    let metadata_json = serde_json::to_string_pretty(&dataset.metadata)?;
    std::fs::write("dataset-metadata.json", metadata_json)?;
    
    Ok(())
}

fn print_statistics(dataset: &TrainingDataset) {
    use colored::*;
    
    println!("\n📊 Training Data Statistics");
    println!("═══════════════════════════");
    
    println!("📁 Dataset: {}", dataset.metadata.name.cyan());
    println!("🏷️  Version: {}", dataset.metadata.version);
    println!("🦀 Language: {}", dataset.metadata.language);
    println!("🎯 Domain: {}", dataset.metadata.domain);
    
    println!("\n📈 Data Counts:");
    println!("  • Training Examples: {}", dataset.metadata.total_examples.to_string().yellow());
    println!("  • Code Chunks: {}", dataset.metadata.total_chunks.to_string().yellow());
    println!("  • Q&A Pairs: {}", dataset.metadata.total_qa_pairs.to_string().yellow());
    
    println!("\n🧠 Complexity Distribution:");
    for (level, count) in &dataset.metadata.complexity_distribution {
        let bar = "█".repeat((*count / 10).max(1).min(50));
        println!("  {}: {} {}", level, count, bar);
    }
    
    // Example type distribution
    let mut example_types = std::collections::HashMap::new();
    for example in &dataset.examples {
        *example_types.entry(example.example_type.clone()).or_insert(0) += 1;
    }
    
    println!("\n🔍 Example Types:");
    for (example_type, count) in example_types {
        println!("  • {}: {}", example_type, count);
    }
    
    // Difficulty distribution
    let mut difficulty_levels = std::collections::HashMap::new();
    for example in &dataset.examples {
        *difficulty_levels.entry(example.metadata.difficulty_level.clone()).or_insert(0) += 1;
    }
    
    println!("\n📚 Difficulty Levels:");
    for (level, count) in difficulty_levels {
        println!("  • {}: {}", level, count);
    }
    
    println!("\n✅ Training data preparation complete!");
    println!("📄 Files generated:");
    println!("  • llm-training-dataset.json (complete dataset)");
    println!("  • training-examples.jsonl (examples in JSONL format)");
    println!("  • qa-pairs.json (question-answer pairs)");
    println!("  • rag-chunks.json (chunks for RAG systems)");
    println!("  • dataset-metadata.json (metadata and statistics)");
}