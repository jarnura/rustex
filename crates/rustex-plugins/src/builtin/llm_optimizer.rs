//! LLM output optimization plugin.

use serde::{Deserialize, Serialize};
use rustex_core::ElementType;
use crate::core::{Plugin, PluginInfo, PluginPhase, PluginContext, PluginOutput, PluginError};
use crate::plugin_info;

/// Configuration for the LLM optimizer plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOptimizerConfig {
    /// Target LLM model for optimization
    pub target_model: String,
    
    /// Maximum token limit for chunking
    pub max_tokens: usize,
    
    /// Whether to add semantic context
    pub add_semantic_context: bool,
    
    /// Whether to include usage examples
    pub include_examples: bool,
    
    /// Whether to optimize for embedding models
    pub optimize_for_embeddings: bool,
    
    /// Chunk overlap in tokens
    pub chunk_overlap: usize,
}

impl Default for LlmOptimizerConfig {
    fn default() -> Self {
        Self {
            target_model: "gpt-4".to_string(),
            max_tokens: 4000,
            add_semantic_context: true,
            include_examples: true,
            optimize_for_embeddings: false,
            chunk_overlap: 200,
        }
    }
}

/// LLM output optimization plugin.
#[derive(Default)]
pub struct LlmOptimizer {
    config: LlmOptimizerConfig,
}

impl LlmOptimizer {
    /// Create a new LLM optimizer with custom configuration.
    pub fn with_config(config: LlmOptimizerConfig) -> Self {
        Self { config }
    }
    
    /// Optimize the project AST for LLM consumption.
    fn optimize_for_llm(&self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        let elements = context.all_elements();
        
        // Create semantic chunks
        let chunks = self.create_semantic_chunks(&elements)?;
        
        // Add chunk metadata
        output.add_metadata("llm_chunks", serde_json::json!({
            "total_chunks": chunks.len(),
            "target_model": self.config.target_model,
            "max_tokens": self.config.max_tokens,
            "chunks": chunks
        }));
        
        // Generate context summaries
        if self.config.add_semantic_context {
            let context_summaries = self.generate_context_summaries(&elements)?;
            output.add_metadata("semantic_context", context_summaries);
        }
        
        // Generate usage examples
        if self.config.include_examples {
            let examples = self.generate_usage_examples(&elements)?;
            output.add_metadata("usage_examples", examples);
        }
        
        // Optimize for embeddings if requested
        if self.config.optimize_for_embeddings {
            let embedding_data = self.optimize_for_embeddings(&elements)?;
            output.add_metadata("embedding_optimization", embedding_data);
        }
        
        // Add optimization metrics
        output.add_metric("chunks_created", chunks.len() as f64);
        output.add_metric("avg_chunk_size", self.calculate_avg_chunk_size(&chunks));
        output.add_metric("token_efficiency", self.calculate_token_efficiency(&chunks));
        
        output.add_message(
            crate::core::plugin::MessageLevel::Info,
            format!("Optimized project for {} with {} chunks", self.config.target_model, chunks.len())
        );
        
        Ok(output)
    }
    
    /// Create semantic chunks for LLM processing.
    fn create_semantic_chunks(&self, elements: &[&rustex_core::CodeElement]) -> Result<Vec<LlmChunk>, PluginError> {
        let mut chunks = Vec::new();
        let mut current_chunk = LlmChunk::new();
        let mut current_tokens = 0;
        
        for element in elements {
            let element_tokens = self.estimate_tokens(element);
            
            // Start new chunk if adding this element would exceed token limit
            if current_tokens + element_tokens > self.config.max_tokens && !current_chunk.elements.is_empty() {
                chunks.push(current_chunk);
                current_chunk = LlmChunk::new();
                current_tokens = 0;
            }
            
            // Add element to current chunk
            current_chunk.add_element(element);
            current_tokens += element_tokens;
        }
        
        // Add final chunk if not empty
        if !current_chunk.elements.is_empty() {
            chunks.push(current_chunk);
        }
        
        // Add overlap between chunks if configured
        if self.config.chunk_overlap > 0 && chunks.len() > 1 {
            self.add_chunk_overlap(&mut chunks)?;
        }
        
        Ok(chunks)
    }
    
    /// Generate semantic context summaries.
    fn generate_context_summaries(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut summaries = std::collections::HashMap::new();
        
        // Group elements by type
        let mut by_type: std::collections::HashMap<ElementType, Vec<&rustex_core::CodeElement>> = std::collections::HashMap::new();
        for element in elements {
            by_type.entry(element.element_type.clone()).or_default().push(element);
        }
        
        // Generate summaries for each type
        for (element_type, type_elements) in by_type {
            let type_name = format!("{:?}", element_type).to_lowercase();
            
            let summary = match element_type {
                ElementType::Function => self.summarize_functions(&type_elements),
                ElementType::Struct => self.summarize_structs(&type_elements),
                ElementType::Enum => self.summarize_enums(&type_elements),
                ElementType::Trait => self.summarize_traits(&type_elements),
                _ => self.summarize_generic(&type_elements),
            };
            
            summaries.insert(type_name, summary);
        }
        
        Ok(serde_json::to_value(summaries)?)
    }
    
    /// Generate usage examples for key elements.
    fn generate_usage_examples(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut examples = Vec::new();
        
        // Find public functions and types that would benefit from examples
        for element in elements {
            if element.visibility == rustex_core::Visibility::Public {
                match element.element_type {
                    ElementType::Function => {
                        if let Some(signature) = &element.signature {
                            examples.push(serde_json::json!({
                                "type": "function",
                                "name": element.name,
                                "signature": signature,
                                "example": self.generate_function_example(element),
                                "description": self.extract_description(element)
                            }));
                        }
                    }
                    ElementType::Struct | ElementType::Enum => {
                        examples.push(serde_json::json!({
                            "type": format!("{:?}", element.element_type).to_lowercase(),
                            "name": element.name,
                            "example": self.generate_type_example(element),
                            "description": self.extract_description(element)
                        }));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(serde_json::to_value(examples)?)
    }
    
    /// Optimize content for embedding models.
    fn optimize_for_embeddings(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut embedding_docs = Vec::new();
        
        for element in elements {
            let doc = serde_json::json!({
                "id": format!("{:?}_{}", element.element_type, element.name),
                "content": self.create_embedding_content(element),
                "metadata": {
                    "type": format!("{:?}", element.element_type),
                    "name": element.name,
                    "file": element.location.file_path.to_string_lossy(),
                    "visibility": format!("{:?}", element.visibility),
                    "complexity": element.complexity.unwrap_or(1)
                }
            });
            
            embedding_docs.push(doc);
        }
        
        Ok(serde_json::json!({
            "documents": embedding_docs,
            "total_documents": embedding_docs.len(),
            "optimization_notes": [
                "Content optimized for semantic search",
                "Includes code context and documentation",
                "Metadata preserved for filtering"
            ]
        }))
    }
    
    /// Estimate token count for an element.
    fn estimate_tokens(&self, element: &rustex_core::CodeElement) -> usize {
        // Rough estimation: 1 token per 4 characters
        let content_length = element.name.len() + 
            element.signature.as_ref().map(|s| s.len()).unwrap_or(0) +
            element.doc_comments.iter().map(|c| c.len()).sum::<usize>() +
            element.inline_comments.iter().map(|c| c.len()).sum::<usize>();
            
        (content_length / 4).max(10) // Minimum 10 tokens per element
    }
    
    /// Calculate average chunk size.
    fn calculate_avg_chunk_size(&self, chunks: &[LlmChunk]) -> f64 {
        if chunks.is_empty() {
            return 0.0;
        }
        
        let total_elements: usize = chunks.iter().map(|c| c.elements.len()).sum();
        total_elements as f64 / chunks.len() as f64
    }
    
    /// Calculate token efficiency.
    fn calculate_token_efficiency(&self, chunks: &[LlmChunk]) -> f64 {
        if chunks.is_empty() {
            return 0.0;
        }
        
        let total_estimated_tokens: usize = chunks.iter()
            .map(|c| c.estimated_tokens)
            .sum();
            
        let max_possible_tokens = chunks.len() * self.config.max_tokens;
        
        if max_possible_tokens == 0 {
            0.0
        } else {
            total_estimated_tokens as f64 / max_possible_tokens as f64
        }
    }
    
    /// Add overlap between chunks.
    fn add_chunk_overlap(&self, chunks: &mut [LlmChunk]) -> Result<(), PluginError> {
        // Implementation would add overlapping elements between chunks
        // For now, just mark that overlap was considered
        for chunk in chunks.iter_mut() {
            chunk.has_overlap = true;
        }
        Ok(())
    }
    
    // Helper methods for generating summaries and examples
    fn summarize_functions(&self, functions: &[&rustex_core::CodeElement]) -> serde_json::Value {
        serde_json::json!({
            "count": functions.len(),
            "public_count": functions.iter().filter(|f| f.visibility == rustex_core::Visibility::Public).count(),
            "avg_complexity": functions.iter()
                .filter_map(|f| f.complexity)
                .sum::<u32>() as f64 / functions.len() as f64
        })
    }
    
    fn summarize_structs(&self, structs: &[&rustex_core::CodeElement]) -> serde_json::Value {
        serde_json::json!({
            "count": structs.len(),
            "public_count": structs.iter().filter(|s| s.visibility == rustex_core::Visibility::Public).count()
        })
    }
    
    fn summarize_enums(&self, enums: &[&rustex_core::CodeElement]) -> serde_json::Value {
        serde_json::json!({
            "count": enums.len(),
            "public_count": enums.iter().filter(|e| e.visibility == rustex_core::Visibility::Public).count()
        })
    }
    
    fn summarize_traits(&self, traits: &[&rustex_core::CodeElement]) -> serde_json::Value {
        serde_json::json!({
            "count": traits.len(),
            "public_count": traits.iter().filter(|t| t.visibility == rustex_core::Visibility::Public).count()
        })
    }
    
    fn summarize_generic(&self, elements: &[&rustex_core::CodeElement]) -> serde_json::Value {
        serde_json::json!({
            "count": elements.len()
        })
    }
    
    fn generate_function_example(&self, _element: &rustex_core::CodeElement) -> String {
        // Simplified example generation
        format!("// Example usage of {}\n{}", _element.name, _element.signature.as_deref().unwrap_or(""))
    }
    
    fn generate_type_example(&self, element: &rustex_core::CodeElement) -> String {
        format!("// Example usage of {}\nlet instance = {}::new();", element.name, element.name)
    }
    
    fn extract_description(&self, element: &rustex_core::CodeElement) -> String {
        if !element.doc_comments.is_empty() {
            element.doc_comments.join(" ")
        } else {
            format!("A {} named {}", format!("{:?}", element.element_type).to_lowercase(), element.name)
        }
    }
    
    fn create_embedding_content(&self, element: &rustex_core::CodeElement) -> String {
        let mut content = Vec::new();
        
        // Add signature/name
        if let Some(signature) = &element.signature {
            content.push(signature.clone());
        } else {
            content.push(element.name.clone());
        }
        
        // Add documentation
        if !element.doc_comments.is_empty() {
            content.extend(element.doc_comments.clone());
        }
        
        // Add context
        content.push(format!("Type: {:?}", element.element_type));
        content.push(format!("Visibility: {:?}", element.visibility));
        
        content.join("\n")
    }
}

/// Represents a semantic chunk for LLM processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChunk {
    pub elements: Vec<String>,
    pub estimated_tokens: usize,
    pub has_overlap: bool,
    pub chunk_type: String,
}

impl LlmChunk {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            estimated_tokens: 0,
            has_overlap: false,
            chunk_type: "semantic".to_string(),
        }
    }
    
    fn add_element(&mut self, element: &rustex_core::CodeElement) {
        self.elements.push(element.name.clone());
        // Update estimated tokens (simplified)
        self.estimated_tokens += element.name.len() / 4 + 10;
    }
}

impl Plugin for LlmOptimizer {
    fn info(&self) -> PluginInfo {
        plugin_info!(
            "llm-optimizer",
            "0.1.0",
            "Optimize AST output for LLM consumption and RAG applications",
            phases: [PluginPhase::PostProject, PluginPhase::PreFormat]
        )
    }
    
    fn initialize(&mut self, config: &serde_json::Value) -> Result<(), PluginError> {
        if !config.is_null() {
            self.config = serde_json::from_value(config.clone())
                .map_err(|e| PluginError::ConfigurationError(e.to_string()))?;
        }
        Ok(())
    }
    
    fn post_project(&self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        self.optimize_for_llm(context)
    }
    
    fn pre_format(&self, _context: &PluginContext) -> Result<PluginOutput, PluginError> {
        // Additional optimization before formatting
        let mut output = PluginOutput::new();
        
        output.add_metadata("llm_ready", true);
        output.add_metadata("optimization_timestamp", chrono::Utc::now().to_rfc3339());
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::{ProjectInfo, ExtractorConfig, CodeElement, CodeLocation, Visibility};
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[test]
    fn test_llm_optimizer() {
        let optimizer = LlmOptimizer::default();
        
        let project_info = ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        };
        
        let config = ExtractorConfig::default();
        let mut metadata = HashMap::new();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        let result = optimizer.post_project(&context);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.additional_metadata.contains_key("llm_chunks"));
    }

    #[test]
    fn test_token_estimation() {
        let optimizer = LlmOptimizer::default();
        
        let element = CodeElement {
            id: "Function_test_function_1".to_string(),
            element_type: ElementType::Function,
            name: "test_function".to_string(),
            signature: Some("fn test_function() -> i32".to_string()),
            visibility: Visibility::Public,
            doc_comments: vec!["A test function".to_string()],
            inline_comments: vec![],
            location: CodeLocation {
                line_start: 1,
                line_end: 5,
                char_start: 0,
                char_end: 50,
                file_path: PathBuf::from("test.rs"),
            },
            attributes: vec![],
            complexity: Some(1),
            complexity_metrics: None,
            dependencies: vec![],
            generic_params: vec![],
            metadata: HashMap::new(),
            hierarchy: rustex_core::ElementHierarchy::new_root(
                "crate::test".to_string(),
                "crate::test::test_function".to_string(),
                rustex_core::ElementNamespace::new(
                    "test_function".to_string(),
                    "crate::test::test_function".to_string(),
                    &Visibility::Public,
                ),
            ),
        };
        
        let tokens = optimizer.estimate_tokens(&element);
        assert!(tokens > 0);
    }
}