//! RAG Output Format Demo
//! 
//! This example demonstrates the RAG-optimized output formats for LLM training
//! and embedding applications.

use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase};
use rustex_formats::{
    RagFormatter, RagConfig, SemanticDepth, DocumentationQuality,
    format_as_rag_jsonl, format_for_embeddings,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 RustEx RAG Output Format Demo");
    println!("═══════════════════════════════════");

    // Extract AST from the current project
    let config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    println!("📊 Extracting project AST...");
    let project_ast = extractor.extract_project()?;
    
    // Demonstrate different RAG output formats
    demonstrate_basic_rag_format(&project_ast)?;
    demonstrate_custom_rag_config(&project_ast)?;
    demonstrate_jsonl_format(&project_ast)?;
    demonstrate_embedding_format(&project_ast)?;
    demonstrate_chunking_strategies(&project_ast)?;
    
    Ok(())
}

/// Demonstrate basic RAG format output.
fn demonstrate_basic_rag_format(project_ast: &rustex_core::ProjectAst) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔹 Basic RAG Format");
    println!("───────────────────");
    
    let formatter = RagFormatter::default();
    let rag_doc = formatter.format(project_ast)?;
    
    println!("✅ Generated RAG document with:");
    println!("  • {} chunks", rag_doc.chunks.len());
    println!("  • {} training examples", rag_doc.training_examples.len());
    println!("  • {} semantic categories", rag_doc.metadata.semantic_categories.len());
    
    // Save basic RAG format
    let json_output = rustex_formats::format_as_json(&rag_doc, true)?;
    std::fs::write("rag-basic.json", json_output)?;
    println!("📄 Saved to: rag-basic.json");
    
    // Print sample chunk information
    if let Some(first_chunk) = rag_doc.chunks.first() {
        println!("\n📝 Sample Chunk:");
        println!("  ID: {}", first_chunk.id);
        println!("  Element: {} ({})", 
                 first_chunk.metadata.element_name, 
                 first_chunk.metadata.element_type);
        println!("  Tokens: {}", first_chunk.metadata.token_count);
        println!("  Strategy: {:?}", first_chunk.metadata.embedding_strategy);
        println!("  Content preview: {}...", 
                 first_chunk.content.chars().take(100).collect::<String>());
    }
    
    Ok(())
}

/// Demonstrate custom RAG configuration.
fn demonstrate_custom_rag_config(project_ast: &rustex_core::ProjectAst) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔹 Custom RAG Configuration");
    println!("────────────────────────────");
    
    let custom_config = RagConfig {
        target_chunk_size: 256,  // Smaller chunks for faster embedding
        max_chunk_size: 512,
        min_chunk_size: 50,
        chunk_overlap: 25,
        generate_training_examples: true,
        max_training_examples_per_chunk: 2,
        semantic_analysis_depth: SemanticDepth::Deep,
        include_private_items: false,
        include_test_code: false,
        min_complexity_for_inclusion: Some(2),
        min_documentation_quality: DocumentationQuality::Basic,
        ..Default::default()
    };
    
    let formatter = RagFormatter::new(custom_config);
    let rag_doc = formatter.format(project_ast)?;
    
    println!("✅ Generated custom RAG document with:");
    println!("  • {} chunks (filtered)", rag_doc.chunks.len());
    println!("  • Target chunk size: 256 tokens");
    println!("  • Min complexity filter: 2");
    println!("  • Deep semantic analysis enabled");
    
    // Analyze chunk size distribution
    let token_counts: Vec<usize> = rag_doc.chunks.iter()
        .map(|c| c.metadata.token_count)
        .collect();
    
    if !token_counts.is_empty() {
        let avg_tokens = token_counts.iter().sum::<usize>() as f64 / token_counts.len() as f64;
        let min_tokens = token_counts.iter().min().unwrap();
        let max_tokens = token_counts.iter().max().unwrap();
        
        println!("\n📊 Chunk Statistics:");
        println!("  • Average tokens: {:.1}", avg_tokens);
        println!("  • Range: {} - {} tokens", min_tokens, max_tokens);
    }
    
    // Save custom configuration output
    let json_output = rustex_formats::format_as_json(&rag_doc, true)?;
    std::fs::write("rag-custom.json", json_output)?;
    println!("📄 Saved to: rag-custom.json");
    
    Ok(())
}

/// Demonstrate JSONL format for streaming processing.
fn demonstrate_jsonl_format(project_ast: &rustex_core::ProjectAst) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔹 JSONL Format (Streaming)");
    println!("───────────────────────────");
    
    let jsonl_output = format_as_rag_jsonl(project_ast)?;
    let line_count = jsonl_output.lines().count();
    
    println!("✅ Generated JSONL format with {} lines", line_count);
    println!("  • First line: metadata");
    println!("  • Remaining lines: individual chunks");
    println!("  • Suitable for streaming processing");
    
    // Save JSONL format
    std::fs::write("rag-chunks.jsonl", &jsonl_output)?;
    println!("📄 Saved to: rag-chunks.jsonl");
    
    // Show sample JSONL lines
    let lines: Vec<&str> = jsonl_output.lines().take(3).collect();
    if !lines.is_empty() {
        println!("\n📝 Sample JSONL lines:");
        for (i, line) in lines.iter().enumerate() {
            let preview = if line.len() > 100 {
                format!("{}...", &line[..100])
            } else {
                line.to_string()
            };
            println!("  Line {}: {}", i + 1, preview);
        }
    }
    
    Ok(())
}

/// Demonstrate embedding-optimized format.
fn demonstrate_embedding_format(project_ast: &rustex_core::ProjectAst) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔹 Embedding Format");
    println!("───────────────────");
    
    let embedding_inputs = format_for_embeddings(project_ast)?;
    
    println!("✅ Generated embedding inputs:");
    println!("  • {} embedding-ready texts", embedding_inputs.len());
    
    // Analyze embedding strategies
    let mut strategy_counts = std::collections::HashMap::new();
    for input in &embedding_inputs {
        let strategy_name = match input.metadata.embedding_strategy {
            rustex_formats::EmbeddingStrategy::Combined => "Combined",
            rustex_formats::EmbeddingStrategy::CodeOnly => "Code Only",
            rustex_formats::EmbeddingStrategy::DocumentationOnly => "Documentation Only",
            rustex_formats::EmbeddingStrategy::Specialized(_) => "Specialized",
        };
        *strategy_counts.entry(strategy_name).or_insert(0) += 1;
    }
    
    println!("\n📊 Embedding Strategies:");
    for (strategy, count) in strategy_counts {
        println!("  • {}: {} texts", strategy, count);
    }
    
    // Save embedding format
    let embedding_json = serde_json::to_string_pretty(&embedding_inputs)?;
    std::fs::write("rag-embeddings.json", embedding_json)?;
    println!("📄 Saved to: rag-embeddings.json");
    
    // Show sample embedding input
    if let Some(first_input) = embedding_inputs.first() {
        println!("\n📝 Sample Embedding Input:");
        println!("  ID: {}", first_input.id);
        println!("  Strategy: {:?}", first_input.metadata.embedding_strategy);
        println!("  Keywords: {:?}", first_input.metadata.retrieval_keywords.iter().take(5).collect::<Vec<_>>());
        println!("  Text preview: {}...", 
                 first_input.text.chars().take(150).collect::<String>());
    }
    
    Ok(())
}

/// Demonstrate different chunking strategies and their effects.
fn demonstrate_chunking_strategies(project_ast: &rustex_core::ProjectAst) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔹 Chunking Strategy Comparison");
    println!("───────────────────────────────");
    
    // Small chunks for embedding models
    let small_config = RagConfig {
        target_chunk_size: 128,
        max_chunk_size: 256,
        semantic_analysis_depth: SemanticDepth::Basic,
        ..Default::default()
    };
    
    // Large chunks for context-aware models
    let large_config = RagConfig {
        target_chunk_size: 1024,
        max_chunk_size: 2048,
        semantic_analysis_depth: SemanticDepth::Deep,
        ..Default::default()
    };
    
    let small_formatter = RagFormatter::new(small_config);
    let large_formatter = RagFormatter::new(large_config);
    
    let small_doc = small_formatter.format(project_ast)?;
    let large_doc = large_formatter.format(project_ast)?;
    
    println!("📊 Chunking Strategy Results:");
    println!("\n  Small Chunks (128-256 tokens):");
    println!("    • {} chunks generated", small_doc.chunks.len());
    println!("    • Average tokens: {:.1}", small_doc.metadata.chunk_size_stats.avg_tokens);
    println!("    • Use case: Embedding models, fine-grained retrieval");
    
    println!("\n  Large Chunks (1024-2048 tokens):");
    println!("    • {} chunks generated", large_doc.chunks.len());
    println!("    • Average tokens: {:.1}", large_doc.metadata.chunk_size_stats.avg_tokens);
    println!("    • Use case: Context-aware models, comprehensive analysis");
    
    // Save comparison results
    let small_json = rustex_formats::format_as_json(&small_doc, true)?;
    let large_json = rustex_formats::format_as_json(&large_doc, true)?;
    
    std::fs::write("rag-small-chunks.json", small_json)?;
    std::fs::write("rag-large-chunks.json", large_json)?;
    
    println!("\n📄 Saved comparison files:");
    println!("  • rag-small-chunks.json");
    println!("  • rag-large-chunks.json");
    
    // Analyze complexity distribution differences
    println!("\n📈 Complexity Distribution Comparison:");
    
    for (name, doc) in [("Small", &small_doc), ("Large", &large_doc)] {
        println!("  {} chunks:", name);
        for (level, count) in &doc.metadata.complexity_distribution {
            println!("    • {}: {}", level, count);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_formats::{RagFormatter, RagConfig};
    
    #[test]
    fn test_rag_formatter_creation() {
        let config = RagConfig::default();
        let formatter = RagFormatter::new(config);
        
        // Test that formatter is created successfully
        // (The actual functionality would be tested with real AST data)
        assert!(true); // Placeholder assertion
    }
    
    #[test]
    fn test_custom_rag_config() {
        let config = RagConfig {
            target_chunk_size: 512,
            generate_training_examples: false,
            semantic_analysis_depth: SemanticDepth::Basic,
            ..Default::default()
        };
        
        assert_eq!(config.target_chunk_size, 512);
        assert!(!config.generate_training_examples);
        assert!(matches!(config.semantic_analysis_depth, SemanticDepth::Basic));
    }
}