//! Advanced complexity analysis plugin.

use serde::{Deserialize, Serialize};
use rustex_core::{ElementType, ComplexityLevel};
use crate::core::{Plugin, PluginInfo, PluginPhase, PluginContext, PluginOutput, PluginError};
use crate::plugin_info;

/// Configuration for the complexity analyzer plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityConfig {
    /// Whether to include detailed complexity breakdown
    pub include_breakdown: bool,
    
    /// Whether to calculate code smells based on complexity
    pub detect_code_smells: bool,
    
    /// Threshold for high complexity warning
    pub high_complexity_threshold: u32,
    
    /// Threshold for very high complexity warning
    pub very_high_complexity_threshold: u32,
    
    /// Whether to suggest refactoring for complex functions
    pub suggest_refactoring: bool,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            include_breakdown: true,
            detect_code_smells: true,
            high_complexity_threshold: 20,
            very_high_complexity_threshold: 50,
            suggest_refactoring: true,
        }
    }
}

/// Advanced complexity analysis plugin.
pub struct ComplexityAnalyzer {
    config: ComplexityConfig,
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self {
            config: ComplexityConfig::default(),
        }
    }
}

impl ComplexityAnalyzer {
    /// Create a new complexity analyzer with custom configuration.
    pub fn with_config(config: ComplexityConfig) -> Self {
        Self { config }
    }
    
    /// Analyze complexity patterns across the project.
    fn analyze_project_complexity(&self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        let elements = context.all_elements();
        let function_elements: Vec<&rustex_core::CodeElement> = elements.iter()
            .filter(|e| e.element_type == ElementType::Function)
            .copied()
            .collect();
            
        if function_elements.is_empty() {
            return Ok(output);
        }
        
        // Calculate aggregate metrics
        let total_functions = function_elements.len();
        let complex_functions = function_elements.iter()
            .filter(|e| {
                if let Some(metrics) = &e.complexity_metrics {
                    metrics.overall_score() > self.config.high_complexity_threshold
                } else {
                    false
                }
            })
            .count();
            
        let very_complex_functions = function_elements.iter()
            .filter(|e| {
                if let Some(metrics) = &e.complexity_metrics {
                    metrics.overall_score() > self.config.very_high_complexity_threshold
                } else {
                    false
                }
            })
            .count();
        
        // Calculate averages
        let avg_complexity: f64 = function_elements.iter()
            .filter_map(|e| e.complexity_metrics.as_ref())
            .map(|m| m.overall_score() as f64)
            .sum::<f64>() / total_functions as f64;
            
        let avg_cyclomatic: f64 = function_elements.iter()
            .filter_map(|e| e.complexity_metrics.as_ref())
            .map(|m| m.cyclomatic as f64)
            .sum::<f64>() / total_functions as f64;
            
        let avg_cognitive: f64 = function_elements.iter()
            .filter_map(|e| e.complexity_metrics.as_ref())
            .map(|m| m.cognitive as f64)
            .sum::<f64>() / total_functions as f64;
        
        // Add metrics to output
        output.add_metric("total_functions", total_functions as f64);
        output.add_metric("complex_functions", complex_functions as f64);
        output.add_metric("very_complex_functions", very_complex_functions as f64);
        output.add_metric("complexity_ratio", complex_functions as f64 / total_functions as f64);
        output.add_metric("avg_complexity", avg_complexity);
        output.add_metric("avg_cyclomatic", avg_cyclomatic);
        output.add_metric("avg_cognitive", avg_cognitive);
        
        // Add complexity breakdown if requested
        if self.config.include_breakdown {
            let mut complexity_distribution = std::collections::HashMap::new();
            
            for element in &function_elements {
                if let Some(metrics) = &element.complexity_metrics {
                    let level = metrics.complexity_level();
                    let level_name = match level {
                        ComplexityLevel::Low => "low",
                        ComplexityLevel::Medium => "medium", 
                        ComplexityLevel::High => "high",
                        ComplexityLevel::VeryHigh => "very_high",
                    };
                    
                    *complexity_distribution.entry(level_name).or_insert(0) += 1;
                }
            }
            
            for (level, count) in complexity_distribution {
                output.add_metric(&format!("complexity_{}", level), count as f64);
            }
        }
        
        // Detect code smells if requested
        if self.config.detect_code_smells {
            self.detect_complexity_smells(&function_elements, &mut output)?;
        }
        
        // Add refactoring suggestions if requested
        if self.config.suggest_refactoring {
            self.suggest_refactoring(&function_elements, &mut output)?;
        }
        
        Ok(output)
    }
    
    /// Detect code smells based on complexity metrics.
    fn detect_complexity_smells(
        &self,
        functions: &[&rustex_core::CodeElement],
        output: &mut PluginOutput,
    ) -> Result<(), PluginError> {
        let mut smell_count = 0;
        
        for function in functions {
            if let Some(metrics) = &function.complexity_metrics {
                let mut smells = Vec::new();
                
                // God function smell
                if metrics.lines_of_code > 100 {
                    smells.push("god_function");
                }
                
                // Too many parameters smell
                if metrics.parameter_count > 7 {
                    smells.push("too_many_parameters");
                }
                
                // Deep nesting smell
                if metrics.nesting_depth > 5 {
                    smells.push("deep_nesting");
                }
                
                // High cognitive complexity smell
                if metrics.cognitive > metrics.cyclomatic * 2 {
                    smells.push("cognitive_overload");
                }
                
                if !smells.is_empty() {
                    smell_count += smells.len();
                    
                    output.add_metadata(
                        format!("code_smell_{}", function.name),
                        serde_json::json!({
                            "function": function.name,
                            "smells": smells,
                            "location": {
                                "file": function.location.file_path.to_string_lossy(),
                                "line": function.location.line_start
                            }
                        })
                    );
                }
            }
        }
        
        output.add_metric("code_smells_detected", smell_count as f64);
        
        if smell_count > 0 {
            output.add_message(
                crate::core::plugin::MessageLevel::Warning,
                format!("Detected {} code smell(s) related to complexity", smell_count)
            );
        }
        
        Ok(())
    }
    
    /// Suggest refactoring for complex functions.
    fn suggest_refactoring(
        &self,
        functions: &[&rustex_core::CodeElement],
        output: &mut PluginOutput,
    ) -> Result<(), PluginError> {
        let complex_functions: Vec<_> = functions.iter()
            .filter(|f| {
                if let Some(metrics) = &f.complexity_metrics {
                    metrics.overall_score() > self.config.high_complexity_threshold
                } else {
                    false
                }
            })
            .collect();
            
        if complex_functions.is_empty() {
            return Ok(());
        }
        
        for function in &complex_functions {
            if let Some(metrics) = &function.complexity_metrics {
                let mut suggestions = Vec::new();
                
                // Suggest breaking down large functions
                if metrics.lines_of_code > 50 {
                    suggestions.push("Consider breaking this function into smaller, more focused functions");
                }
                
                // Suggest reducing parameters
                if metrics.parameter_count > 5 {
                    suggestions.push("Consider using a configuration struct to reduce parameter count");
                }
                
                // Suggest reducing nesting
                if metrics.nesting_depth > 3 {
                    suggestions.push("Consider extracting nested logic into separate functions");
                }
                
                // Suggest reducing cognitive complexity
                if metrics.cognitive > 15 {
                    suggestions.push("Consider simplifying the logic flow to reduce cognitive complexity");
                }
                
                if !suggestions.is_empty() {
                    output.add_metadata(
                        format!("refactoring_suggestion_{}", function.name),
                        serde_json::json!({
                            "function": function.name,
                            "complexity_score": metrics.overall_score(),
                            "suggestions": suggestions,
                            "location": {
                                "file": function.location.file_path.to_string_lossy(),
                                "line": function.location.line_start
                            }
                        })
                    );
                }
            }
        }
        
        output.add_metric("refactoring_suggestions", complex_functions.len() as f64);
        
        if !complex_functions.is_empty() {
            output.add_message(
                crate::core::plugin::MessageLevel::Info,
                format!("Generated refactoring suggestions for {} complex function(s)", complex_functions.len())
            );
        }
        
        Ok(())
    }
}

impl Plugin for ComplexityAnalyzer {
    fn info(&self) -> PluginInfo {
        plugin_info!(
            "complexity-analyzer",
            "0.1.0", 
            "Advanced complexity analysis and code smell detection",
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
        self.analyze_project_complexity(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::{ProjectInfo, ExtractorConfig, CodeElement, CodeLocation, ComplexityMetrics, HalsteadMetrics, Visibility};
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[allow(dead_code)] // Used in tests
    fn create_test_element(name: &str, complexity_score: u32) -> CodeElement {
        use rustex_core::{ElementHierarchy, ElementNamespace};
        
        CodeElement {
            id: format!("Function_{}_{}", name, 1),
            element_type: ElementType::Function,
            name: name.to_string(),
            signature: Some(format!("fn {}()", name)),
            visibility: Visibility::Public,
            doc_comments: vec![],
            inline_comments: vec![],
            location: CodeLocation {
                line_start: 1,
                line_end: 10,
                char_start: 0,
                char_end: 100,
                file_path: PathBuf::from("test.rs"),
            },
            attributes: vec![],
            complexity: Some(complexity_score),
            complexity_metrics: Some(ComplexityMetrics {
                cyclomatic: complexity_score,
                cognitive: complexity_score,
                halstead: HalsteadMetrics::default(),
                nesting_depth: 2,
                lines_of_code: 50,
                parameter_count: 3,
                return_count: 1,
            }),
            dependencies: vec![],
            generic_params: vec![],
            metadata: HashMap::new(),
            hierarchy: ElementHierarchy::new_root(
                "crate::test".to_string(),
                format!("crate::test::{}", name),
                ElementNamespace::new(
                    name.to_string(),
                    format!("crate::test::{}", name),
                    &Visibility::Public,
                ),
            ),
        }
    }

    #[test]
    fn test_complexity_analyzer() {
        let analyzer = ComplexityAnalyzer::default();
        
        let project_info = ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        };
        
        let config = ExtractorConfig::default();
        let mut metadata = HashMap::new();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        let result = analyzer.post_project(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complexity_config_deserialization() {
        let config_json = serde_json::json!({
            "include_breakdown": false,
            "detect_code_smells": true,
            "high_complexity_threshold": 30,
            "very_high_complexity_threshold": 60,
            "suggest_refactoring": true
        });
        
        let config: ComplexityConfig = serde_json::from_value(config_json).unwrap();
        assert!(!config.include_breakdown);
        assert!(config.detect_code_smells);
        assert_eq!(config.high_complexity_threshold, 30);
    }
}