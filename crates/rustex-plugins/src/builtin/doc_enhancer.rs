//! Documentation enhancement plugin.

use serde::{Deserialize, Serialize};
use rustex_core::ElementType;
use crate::core::{Plugin, PluginInfo, PluginPhase, PluginContext, PluginOutput, PluginError};
use crate::plugin_info;

/// Configuration for the documentation enhancer plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocEnhancerConfig {
    /// Whether to generate missing documentation
    pub generate_missing_docs: bool,
    
    /// Whether to enhance existing documentation
    pub enhance_existing_docs: bool,
    
    /// Whether to add cross-references
    pub add_cross_references: bool,
    
    /// Whether to generate example code
    pub generate_examples: bool,
    
    /// Documentation style to follow
    pub doc_style: DocStyle,
    
    /// Minimum documentation quality score
    pub min_quality_score: f64,
}

/// Documentation style options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DocStyle {
    /// Standard Rust documentation style
    #[default]
    Standard,
    /// Google style documentation
    Google,
    /// JSDoc style documentation
    JSDoc,
    /// Custom style
    Custom(String),
}

impl Default for DocEnhancerConfig {
    fn default() -> Self {
        Self {
            generate_missing_docs: true,
            enhance_existing_docs: true,
            add_cross_references: true,
            generate_examples: false,
            doc_style: DocStyle::Standard,
            min_quality_score: 0.7,
        }
    }
}

/// Documentation enhancement plugin.
#[derive(Default)]
pub struct DocEnhancer {
    config: DocEnhancerConfig,
}

impl DocEnhancer {
    /// Create a new documentation enhancer with custom configuration.
    pub fn with_config(config: DocEnhancerConfig) -> Self {
        Self { config }
    }
    
    /// Enhance documentation across the project.
    fn enhance_documentation(&self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        let elements = context.all_elements();
        
        // Analyze current documentation state
        let doc_analysis = self.analyze_documentation(&elements)?;
        
        // Generate missing documentation suggestions
        if self.config.generate_missing_docs {
            let missing_docs = self.find_missing_documentation(&elements)?;
            output.add_metadata("missing_documentation", missing_docs);
        }
        
        // Enhance existing documentation
        if self.config.enhance_existing_docs {
            let enhancements = self.suggest_documentation_enhancements(&elements)?;
            output.add_metadata("documentation_enhancements", enhancements);
        }
        
        // Add cross-references
        if self.config.add_cross_references {
            let cross_refs = self.generate_cross_references(&elements)?;
            output.add_metadata("cross_references", cross_refs);
        }
        
        // Generate examples
        if self.config.generate_examples {
            let examples = self.generate_documentation_examples(&elements)?;
            output.add_metadata("documentation_examples", examples);
        }
        
        // Add analysis metrics
        output.add_metadata("documentation_analysis", doc_analysis);
        
        // Calculate overall documentation metrics
        let total_elements = elements.len();
        let documented_elements = elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .count();
            
        let documentation_coverage = if total_elements > 0 {
            documented_elements as f64 / total_elements as f64
        } else {
            0.0
        };
        
        output.add_metric("documentation_coverage", documentation_coverage);
        output.add_metric("total_elements", total_elements as f64);
        output.add_metric("documented_elements", documented_elements as f64);
        output.add_metric("undocumented_elements", (total_elements - documented_elements) as f64);
        
        // Add quality metrics
        let avg_quality = self.calculate_average_documentation_quality(&elements);
        output.add_metric("avg_documentation_quality", avg_quality);
        
        output.add_message(
            crate::core::plugin::MessageLevel::Info,
            format!(
                "Documentation analysis complete: {:.1}% coverage, {:.2} avg quality",
                documentation_coverage * 100.0,
                avg_quality
            )
        );
        
        Ok(output)
    }
    
    /// Analyze current documentation state.
    fn analyze_documentation(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut analysis = std::collections::HashMap::new();
        
        // Group by element type and analyze
        let mut by_type: std::collections::HashMap<ElementType, Vec<&rustex_core::CodeElement>> = std::collections::HashMap::new();
        for element in elements {
            by_type.entry(element.element_type.clone()).or_default().push(element);
        }
        
        for (element_type, type_elements) in by_type {
            let type_name = format!("{:?}", element_type).to_lowercase();
            
            let documented = type_elements.iter()
                .filter(|e| !e.doc_comments.is_empty())
                .count();
                
            let public_documented = type_elements.iter()
                .filter(|e| e.visibility == rustex_core::Visibility::Public && !e.doc_comments.is_empty())
                .count();
                
            let public_total = type_elements.iter()
                .filter(|e| e.visibility == rustex_core::Visibility::Public)
                .count();
            
            analysis.insert(type_name, serde_json::json!({
                "total": type_elements.len(),
                "documented": documented,
                "coverage": documented as f64 / type_elements.len() as f64,
                "public_total": public_total,
                "public_documented": public_documented,
                "public_coverage": if public_total > 0 { 
                    public_documented as f64 / public_total as f64 
                } else { 
                    0.0 
                }
            }));
        }
        
        Ok(serde_json::to_value(analysis)?)
    }
    
    /// Find elements missing documentation.
    fn find_missing_documentation(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut missing = Vec::new();
        
        for element in elements {
            if element.doc_comments.is_empty() {
                // Prioritize public items
                let priority = match element.visibility {
                    rustex_core::Visibility::Public => "high",
                    rustex_core::Visibility::Restricted(_) => "medium",
                    rustex_core::Visibility::Private => "low",
                };
                
                missing.push(serde_json::json!({
                    "name": element.name,
                    "type": format!("{:?}", element.element_type),
                    "visibility": format!("{:?}", element.visibility),
                    "priority": priority,
                    "location": {
                        "file": element.location.file_path.to_string_lossy(),
                        "line": element.location.line_start
                    },
                    "suggested_documentation": self.generate_documentation_suggestion(element)
                }));
            }
        }
        
        Ok(serde_json::json!({
            "missing_count": missing.len(),
            "items": missing
        }))
    }
    
    /// Suggest enhancements for existing documentation.
    fn suggest_documentation_enhancements(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut enhancements = Vec::new();
        
        for element in elements {
            if !element.doc_comments.is_empty() {
                let quality_score = self.calculate_documentation_quality(element);
                
                if quality_score < self.config.min_quality_score {
                    let suggestions = self.generate_enhancement_suggestions(element, quality_score);
                    
                    enhancements.push(serde_json::json!({
                        "name": element.name,
                        "type": format!("{:?}", element.element_type),
                        "current_quality": quality_score,
                        "suggestions": suggestions,
                        "location": {
                            "file": element.location.file_path.to_string_lossy(),
                            "line": element.location.line_start
                        }
                    }));
                }
            }
        }
        
        Ok(serde_json::json!({
            "enhancement_count": enhancements.len(),
            "items": enhancements
        }))
    }
    
    /// Generate cross-references between elements.
    fn generate_cross_references(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut cross_refs = std::collections::HashMap::new();
        
        // Build a map of element names for lookup
        let element_map: std::collections::HashMap<&str, &rustex_core::CodeElement> = 
            elements.iter().map(|e| (e.name.as_str(), *e)).collect();
        
        for element in elements {
            let mut refs = Vec::new();
            
            // Look for references in dependencies
            for dep in &element.dependencies {
                if let Some(referenced_element) = element_map.get(dep.as_str()) {
                    refs.push(serde_json::json!({
                        "type": "dependency",
                        "target": dep,
                        "target_type": format!("{:?}", referenced_element.element_type),
                        "location": {
                            "file": referenced_element.location.file_path.to_string_lossy(),
                            "line": referenced_element.location.line_start
                        }
                    }));
                }
            }
            
            if !refs.is_empty() {
                cross_refs.insert(element.name.clone(), refs);
            }
        }
        
        Ok(serde_json::to_value(cross_refs)?)
    }
    
    /// Generate documentation examples.
    fn generate_documentation_examples(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let mut examples = Vec::new();
        
        for element in elements {
            if element.visibility == rustex_core::Visibility::Public {
                let example = self.generate_usage_example(element);
                
                examples.push(serde_json::json!({
                    "name": element.name,
                    "type": format!("{:?}", element.element_type),
                    "example": example,
                    "location": {
                        "file": element.location.file_path.to_string_lossy(),
                        "line": element.location.line_start
                    }
                }));
            }
        }
        
        Ok(serde_json::json!({
            "example_count": examples.len(),
            "items": examples
        }))
    }
    
    /// Calculate documentation quality score for an element.
    fn calculate_documentation_quality(&self, element: &rustex_core::CodeElement) -> f64 {
        if element.doc_comments.is_empty() {
            return 0.0;
        }
        
        let combined_docs = element.doc_comments.join(" ");
        let mut score = 0.0;
        let mut checks = 0.0;
        
        // Check for minimum length
        checks += 1.0;
        if combined_docs.len() > 20 {
            score += 1.0;
        }
        
        // Check for proper structure (function-specific)
        if element.element_type == ElementType::Function {
            checks += 3.0;
            
            // Check for description
            if combined_docs.to_lowercase().contains("return") {
                score += 1.0;
            }
            
            // Check for parameters (if function has parameters)
            if element.signature.as_ref().map(|s| s.contains('(')).unwrap_or(false) {
                if combined_docs.to_lowercase().contains("param") || 
                   combined_docs.contains("# Arguments") {
                    score += 1.0;
                }
            } else {
                score += 1.0; // No parameters to document
            }
            
            // Check for examples
            if combined_docs.contains("```") || combined_docs.contains("# Example") {
                score += 1.0;
            }
        } else {
            checks += 1.0;
            // For non-functions, just check for reasonable description
            if combined_docs.len() > 50 {
                score += 1.0;
            }
        }
        
        // Check for proper formatting
        checks += 1.0;
        if combined_docs.starts_with("///") || combined_docs.contains("# ") {
            score += 1.0;
        }
        
        score / checks
    }
    
    /// Calculate average documentation quality across all elements.
    fn calculate_average_documentation_quality(&self, elements: &[&rustex_core::CodeElement]) -> f64 {
        let documented_elements: Vec<_> = elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .collect();
            
        if documented_elements.is_empty() {
            return 0.0;
        }
        
        let total_quality: f64 = documented_elements.iter()
            .map(|e| self.calculate_documentation_quality(e))
            .sum();
            
        total_quality / documented_elements.len() as f64
    }
    
    /// Generate documentation suggestion for an element.
    fn generate_documentation_suggestion(&self, element: &rustex_core::CodeElement) -> String {
        match element.element_type {
            ElementType::Function => {
                format!(
                    "/// Brief description of what {} does.\n///\n/// # Arguments\n///\n/// * `param` - Description of parameter\n///\n/// # Returns\n///\n/// Description of return value",
                    element.name
                )
            }
            ElementType::Struct => {
                format!("/// A struct representing {}.", element.name)
            }
            ElementType::Enum => {
                format!("/// An enumeration representing different types of {}.", element.name)
            }
            ElementType::Trait => {
                format!("/// A trait defining the behavior for {}.", element.name)
            }
            _ => {
                format!("/// Documentation for {}.", element.name)
            }
        }
    }
    
    /// Generate enhancement suggestions for existing documentation.
    fn generate_enhancement_suggestions(&self, element: &rustex_core::CodeElement, quality_score: f64) -> Vec<String> {
        let mut suggestions = Vec::new();
        let combined_docs = element.doc_comments.join(" ");
        
        // Check for common missing elements
        if combined_docs.len() < 20 {
            suggestions.push("Add more detailed description".to_string());
        }
        
        if element.element_type == ElementType::Function {
            if !combined_docs.to_lowercase().contains("return") && 
               element.signature.as_ref().map(|s| !s.contains("()")).unwrap_or(false) {
                suggestions.push("Add return value documentation".to_string());
            }
            
            if element.signature.as_ref().map(|s| s.contains('(')).unwrap_or(false) &&
               !combined_docs.to_lowercase().contains("param") &&
               !combined_docs.contains("# Arguments") {
                suggestions.push("Add parameter documentation".to_string());
            }
            
            if !combined_docs.contains("```") && !combined_docs.contains("# Example") {
                suggestions.push("Add usage example".to_string());
            }
        }
        
        if quality_score < 0.3 {
            suggestions.push("Consider rewriting documentation for clarity".to_string());
        }
        
        suggestions
    }
    
    /// Generate usage example for an element.
    fn generate_usage_example(&self, element: &rustex_core::CodeElement) -> String {
        match element.element_type {
            ElementType::Function => {
                if let Some(signature) = &element.signature {
                    format!("```rust\n{}\n```", signature)
                } else {
                    format!("```rust\n{}();\n```", element.name)
                }
            }
            ElementType::Struct => {
                format!("```rust\nlet instance = {}::new();\n```", element.name)
            }
            ElementType::Enum => {
                format!("```rust\nlet value = {}::Variant;\n```", element.name)
            }
            _ => {
                format!("```rust\n// Usage of {}\n```", element.name)
            }
        }
    }
}

impl Plugin for DocEnhancer {
    fn info(&self) -> PluginInfo {
        plugin_info!(
            "doc-enhancer",
            "0.1.0",
            "Enhance and analyze code documentation quality",
            phases: [PluginPhase::PostProject]
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
        self.enhance_documentation(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::{ProjectInfo, ExtractorConfig, CodeElement, CodeLocation, Visibility};
    use std::path::PathBuf;
    use std::collections::HashMap;

    fn create_test_element_with_docs(name: &str, docs: Vec<String>) -> rustex_core::CodeElement {
        CodeElement {
            id: format!("Function_{}_{}", name, 1),
            element_type: ElementType::Function,
            name: name.to_string(),
            signature: Some(format!("fn {}()", name)),
            visibility: Visibility::Public,
            doc_comments: docs,
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
                format!("crate::test::{}", name),
                rustex_core::ElementNamespace::new(
                    name.to_string(),
                    format!("crate::test::{}", name),
                    &Visibility::Public,
                ),
            ),
        }
    }

    #[test]
    fn test_doc_enhancer() {
        let enhancer = DocEnhancer::default();
        
        let project_info = ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        };
        
        let config = ExtractorConfig::default();
        let mut metadata = HashMap::new();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        let result = enhancer.post_project(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_documentation_quality_calculation() {
        let enhancer = DocEnhancer::default();
        
        // Well-documented function
        let good_element = create_test_element_with_docs(
            "good_function",
            vec![
                "/// A well-documented function that does something useful.".to_string(),
                "/// ".to_string(),
                "/// # Arguments".to_string(),
                "/// ".to_string(),
                "/// * `param` - A parameter that does something".to_string(),
                "/// ".to_string(),
                "/// # Returns".to_string(),
                "/// ".to_string(),
                "/// Returns a value indicating success".to_string(),
            ]
        );
        
        let quality = enhancer.calculate_documentation_quality(&good_element);
        assert!(quality > 0.5);
        
        // Poorly documented function
        let poor_element = create_test_element_with_docs(
            "poor_function",
            vec!["/// Bad docs".to_string()]
        );
        
        let poor_quality = enhancer.calculate_documentation_quality(&poor_element);
        assert!(poor_quality < quality);
    }
}