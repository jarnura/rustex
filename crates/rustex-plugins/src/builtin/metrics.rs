//! Comprehensive metrics collection plugin.

use serde::{Deserialize, Serialize};
use rustex_core::ElementType;
use crate::core::{Plugin, PluginInfo, PluginPhase, PluginContext, PluginOutput, PluginError};
use crate::plugin_info;

/// Configuration for the metrics collector plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to collect detailed metrics
    pub detailed_metrics: bool,
    
    /// Whether to include historical trends (if available)
    pub include_trends: bool,
    
    /// Whether to calculate ratios and percentages
    pub calculate_ratios: bool,
    
    /// Whether to generate metric summaries
    pub generate_summaries: bool,
    
    /// Custom metrics to calculate
    pub custom_metrics: Vec<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            detailed_metrics: true,
            include_trends: false,
            calculate_ratios: true,
            generate_summaries: true,
            custom_metrics: vec![],
        }
    }
}

/// Comprehensive metrics collection plugin.
#[derive(Default)]
pub struct MetricsCollector {
    config: MetricsConfig,
}

impl MetricsCollector {
    /// Create a new metrics collector with custom configuration.
    pub fn with_config(config: MetricsConfig) -> Self {
        Self { config }
    }
    
    /// Collect comprehensive metrics from the project.
    fn collect_metrics(&self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        let elements = context.all_elements();
        
        // Basic count metrics
        self.collect_count_metrics(&elements, &mut output)?;
        
        // Complexity metrics
        self.collect_complexity_metrics(&elements, &mut output)?;
        
        // Size metrics
        self.collect_size_metrics(&elements, &mut output)?;
        
        // Visibility metrics
        self.collect_visibility_metrics(&elements, &mut output)?;
        
        // Documentation metrics
        self.collect_documentation_metrics(&elements, &mut output)?;
        
        // Dependency metrics
        self.collect_dependency_metrics(&elements, &mut output)?;
        
        // Detailed metrics if requested
        if self.config.detailed_metrics {
            self.collect_detailed_metrics(&elements, &mut output)?;
        }
        
        // Ratios and percentages if requested
        if self.config.calculate_ratios {
            self.calculate_ratio_metrics(&elements, &mut output)?;
        }
        
        // Generate summaries if requested
        if self.config.generate_summaries {
            let summary = self.generate_metric_summary(&elements)?;
            output.add_metadata("metrics_summary", summary);
        }
        
        // Custom metrics if configured
        for custom_metric in &self.config.custom_metrics {
            self.calculate_custom_metric(custom_metric, &elements, &mut output)?;
        }
        
        output.add_message(
            crate::core::plugin::MessageLevel::Info,
            format!("Collected {} metrics for {} elements", output.metrics.len(), elements.len())
        );
        
        Ok(output)
    }
    
    /// Collect basic count metrics.
    fn collect_count_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        // Total counts
        output.add_metric("total_elements", elements.len() as f64);
        
        // Count by type
        let mut type_counts = std::collections::HashMap::new();
        for element in elements {
            *type_counts.entry(element.element_type.clone()).or_insert(0) += 1;
        }
        
        for (element_type, count) in type_counts {
            let metric_name = format!("count_{:?}", element_type).to_lowercase();
            output.add_metric(&metric_name, count as f64);
        }
        
        Ok(())
    }
    
    /// Collect complexity-related metrics.
    fn collect_complexity_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        let function_elements: Vec<_> = elements.iter()
            .filter(|e| e.element_type == ElementType::Function)
            .collect();
            
        if function_elements.is_empty() {
            return Ok(());
        }
        
        // Collect complexity scores
        let complexity_scores: Vec<u32> = function_elements.iter()
            .filter_map(|e| e.complexity)
            .collect();
            
        if !complexity_scores.is_empty() {
            let total_complexity: u32 = complexity_scores.iter().sum();
            let avg_complexity = total_complexity as f64 / complexity_scores.len() as f64;
            let max_complexity = *complexity_scores.iter().max().unwrap_or(&0);
            let min_complexity = *complexity_scores.iter().min().unwrap_or(&0);
            
            output.add_metric("avg_complexity", avg_complexity);
            output.add_metric("max_complexity", max_complexity as f64);
            output.add_metric("min_complexity", min_complexity as f64);
            output.add_metric("total_complexity", total_complexity as f64);
        }
        
        // Detailed complexity metrics from complexity_metrics field
        let detailed_metrics: Vec<_> = function_elements.iter()
            .filter_map(|e| e.complexity_metrics.as_ref())
            .collect();
            
        if !detailed_metrics.is_empty() {
            // Cyclomatic complexity
            let cyclomatic_scores: Vec<u32> = detailed_metrics.iter().map(|m| m.cyclomatic).collect();
            let avg_cyclomatic = cyclomatic_scores.iter().sum::<u32>() as f64 / cyclomatic_scores.len() as f64;
            output.add_metric("avg_cyclomatic_complexity", avg_cyclomatic);
            
            // Cognitive complexity
            let cognitive_scores: Vec<u32> = detailed_metrics.iter().map(|m| m.cognitive).collect();
            let avg_cognitive = cognitive_scores.iter().sum::<u32>() as f64 / cognitive_scores.len() as f64;
            output.add_metric("avg_cognitive_complexity", avg_cognitive);
            
            // Nesting depth
            let nesting_depths: Vec<u32> = detailed_metrics.iter().map(|m| m.nesting_depth).collect();
            let avg_nesting = nesting_depths.iter().sum::<u32>() as f64 / nesting_depths.len() as f64;
            let max_nesting = *nesting_depths.iter().max().unwrap_or(&0);
            output.add_metric("avg_nesting_depth", avg_nesting);
            output.add_metric("max_nesting_depth", max_nesting as f64);
            
            // Lines of code
            let loc_values: Vec<u32> = detailed_metrics.iter().map(|m| m.lines_of_code).collect();
            let avg_loc = loc_values.iter().sum::<u32>() as f64 / loc_values.len() as f64;
            let max_loc = *loc_values.iter().max().unwrap_or(&0);
            output.add_metric("avg_lines_per_function", avg_loc);
            output.add_metric("max_lines_per_function", max_loc as f64);
            
            // Parameter counts
            let param_counts: Vec<u32> = detailed_metrics.iter().map(|m| m.parameter_count).collect();
            let avg_params = param_counts.iter().sum::<u32>() as f64 / param_counts.len() as f64;
            let max_params = *param_counts.iter().max().unwrap_or(&0);
            output.add_metric("avg_parameters_per_function", avg_params);
            output.add_metric("max_parameters_per_function", max_params as f64);
        }
        
        Ok(())
    }
    
    /// Collect size-related metrics.
    fn collect_size_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        // Calculate total estimated size
        let total_name_length: usize = elements.iter().map(|e| e.name.len()).sum();
        let total_signature_length: usize = elements.iter()
            .filter_map(|e| e.signature.as_ref())
            .map(|s| s.len())
            .sum();
            
        output.add_metric("total_name_characters", total_name_length as f64);
        output.add_metric("total_signature_characters", total_signature_length as f64);
        
        // Average name length
        if !elements.is_empty() {
            let avg_name_length = total_name_length as f64 / elements.len() as f64;
            output.add_metric("avg_name_length", avg_name_length);
        }
        
        // Generic parameters
        let total_generics: usize = elements.iter()
            .map(|e| e.generic_params.len())
            .sum();
        output.add_metric("total_generic_parameters", total_generics as f64);
        
        Ok(())
    }
    
    /// Collect visibility-related metrics.
    fn collect_visibility_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        let mut visibility_counts = std::collections::HashMap::new();
        
        for element in elements {
            let visibility_key = match &element.visibility {
                rustex_core::Visibility::Public => "public",
                rustex_core::Visibility::Private => "private",
                rustex_core::Visibility::Restricted(_) => "restricted",
            };
            
            *visibility_counts.entry(visibility_key).or_insert(0) += 1;
        }
        
        for (visibility, count) in visibility_counts {
            let metric_name = format!("visibility_{}", visibility);
            output.add_metric(&metric_name, count as f64);
        }
        
        Ok(())
    }
    
    /// Collect documentation-related metrics.
    fn collect_documentation_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        let documented_count = elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .count();
            
        let total_doc_lines: usize = elements.iter()
            .map(|e| e.doc_comments.len())
            .sum();
            
        let total_inline_comments: usize = elements.iter()
            .map(|e| e.inline_comments.len())
            .sum();
        
        output.add_metric("documented_elements", documented_count as f64);
        output.add_metric("total_doc_lines", total_doc_lines as f64);
        output.add_metric("total_inline_comments", total_inline_comments as f64);
        
        if !elements.is_empty() {
            let documentation_coverage = documented_count as f64 / elements.len() as f64;
            output.add_metric("documentation_coverage", documentation_coverage);
            
            let avg_doc_lines = total_doc_lines as f64 / elements.len() as f64;
            output.add_metric("avg_doc_lines_per_element", avg_doc_lines);
        }
        
        Ok(())
    }
    
    /// Collect dependency-related metrics.
    fn collect_dependency_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        let total_dependencies: usize = elements.iter()
            .map(|e| e.dependencies.len())
            .sum();
            
        let elements_with_deps = elements.iter()
            .filter(|e| !e.dependencies.is_empty())
            .count();
        
        output.add_metric("total_dependencies", total_dependencies as f64);
        output.add_metric("elements_with_dependencies", elements_with_deps as f64);
        
        if !elements.is_empty() {
            let avg_dependencies = total_dependencies as f64 / elements.len() as f64;
            output.add_metric("avg_dependencies_per_element", avg_dependencies);
        }
        
        // Find most connected elements
        if !elements.is_empty() {
            let max_dependencies = elements.iter()
                .map(|e| e.dependencies.len())
                .max()
                .unwrap_or(0);
            output.add_metric("max_dependencies_per_element", max_dependencies as f64);
        }
        
        Ok(())
    }
    
    /// Collect detailed metrics for in-depth analysis.
    fn collect_detailed_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        // Attribute usage
        let total_attributes: usize = elements.iter()
            .map(|e| e.attributes.len())
            .sum();
        output.add_metric("total_attributes", total_attributes as f64);
        
        // Metadata usage
        let elements_with_metadata = elements.iter()
            .filter(|e| !e.metadata.is_empty())
            .count();
        output.add_metric("elements_with_metadata", elements_with_metadata as f64);
        
        // Location spread analysis
        let unique_files: std::collections::HashSet<_> = elements.iter()
            .map(|e| &e.location.file_path)
            .collect();
        output.add_metric("unique_files", unique_files.len() as f64);
        
        if !elements.is_empty() {
            let avg_elements_per_file = elements.len() as f64 / unique_files.len() as f64;
            output.add_metric("avg_elements_per_file", avg_elements_per_file);
        }
        
        Ok(())
    }
    
    /// Calculate ratio and percentage metrics.
    fn calculate_ratio_metrics(&self, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        if elements.is_empty() {
            return Ok(());
        }
        
        let total = elements.len() as f64;
        
        // Function to total ratio
        let function_count = elements.iter()
            .filter(|e| e.element_type == ElementType::Function)
            .count() as f64;
        output.add_metric("function_ratio", function_count / total);
        
        // Public to total ratio
        let public_count = elements.iter()
            .filter(|e| e.visibility == rustex_core::Visibility::Public)
            .count() as f64;
        output.add_metric("public_ratio", public_count / total);
        
        // Complex to simple ratio
        let complex_count = elements.iter()
            .filter(|e| e.complexity.unwrap_or(1) > 10)
            .count() as f64;
        output.add_metric("complex_ratio", complex_count / total);
        
        // Documented to undocumented ratio
        let documented_count = elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .count() as f64;
        output.add_metric("documented_ratio", documented_count / total);
        
        Ok(())
    }
    
    /// Generate a comprehensive metric summary.
    fn generate_metric_summary(&self, elements: &[&rustex_core::CodeElement]) -> Result<serde_json::Value, PluginError> {
        let total_elements = elements.len();
        
        if total_elements == 0 {
            return Ok(serde_json::json!({
                "status": "empty_project",
                "message": "No elements found to analyze"
            }));
        }
        
        // Quality assessment
        let documented_ratio = elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .count() as f64 / total_elements as f64;
            
        let avg_complexity = elements.iter()
            .filter_map(|e| e.complexity)
            .sum::<u32>() as f64 / total_elements as f64;
        
        // Determine overall project health
        let health_score = self.calculate_health_score(documented_ratio, avg_complexity);
        let health_level = match health_score {
            s if s >= 0.8 => "excellent",
            s if s >= 0.6 => "good", 
            s if s >= 0.4 => "fair",
            _ => "needs_improvement",
        };
        
        Ok(serde_json::json!({
            "total_elements": total_elements,
            "documentation_coverage": documented_ratio,
            "average_complexity": avg_complexity,
            "health_score": health_score,
            "health_level": health_level,
            "recommendations": self.generate_recommendations(documented_ratio, avg_complexity)
        }))
    }
    
    /// Calculate custom metrics based on configuration.
    fn calculate_custom_metric(&self, metric_name: &str, elements: &[&rustex_core::CodeElement], output: &mut PluginOutput) -> Result<(), PluginError> {
        match metric_name {
            "trait_implementation_ratio" => {
                let trait_count = elements.iter()
                    .filter(|e| e.element_type == ElementType::Trait)
                    .count() as f64;
                let impl_count = elements.iter()
                    .filter(|e| e.element_type == ElementType::Impl)
                    .count() as f64;
                    
                let ratio = if trait_count > 0.0 { impl_count / trait_count } else { 0.0 };
                output.add_metric("trait_implementation_ratio", ratio);
            }
            "naming_consistency" => {
                let snake_case_count = elements.iter()
                    .filter(|e| e.name.contains('_'))
                    .count() as f64;
                let total = elements.len() as f64;
                let consistency = if total > 0.0 { snake_case_count / total } else { 0.0 };
                output.add_metric("naming_consistency", consistency);
            }
            _ => {
                // Unknown custom metric
                output.add_message(
                    crate::core::plugin::MessageLevel::Warning,
                    format!("Unknown custom metric: {}", metric_name)
                );
            }
        }
        
        Ok(())
    }
    
    /// Calculate overall project health score.
    fn calculate_health_score(&self, documentation_ratio: f64, avg_complexity: f64) -> f64 {
        // Simple health scoring algorithm
        let doc_score = documentation_ratio; // 0.0 to 1.0
        let complexity_score = (1.0 - (avg_complexity / 50.0).min(1.0)).max(0.0); // Inverse of complexity
        
        (doc_score * 0.4 + complexity_score * 0.6).clamp(0.0, 1.0)
    }
    
    /// Generate recommendations based on metrics.
    fn generate_recommendations(&self, documentation_ratio: f64, avg_complexity: f64) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if documentation_ratio < 0.5 {
            recommendations.push("Consider improving documentation coverage".to_string());
        }
        
        if avg_complexity > 20.0 {
            recommendations.push("Consider refactoring complex functions".to_string());
        }
        
        if documentation_ratio > 0.8 && avg_complexity < 10.0 {
            recommendations.push("Great job! Well-documented and maintainable code".to_string());
        }
        
        recommendations
    }
}

impl Plugin for MetricsCollector {
    fn info(&self) -> PluginInfo {
        plugin_info!(
            "metrics-collector",
            "0.1.0",
            "Comprehensive code metrics collection and analysis",
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
        self.collect_metrics(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::{ProjectInfo, ExtractorConfig};
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::default();
        
        let project_info = ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        };
        
        let config = ExtractorConfig::default();
        let mut metadata = HashMap::new();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        let result = collector.post_project(&context);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.metrics.contains_key("total_elements"));
    }

    #[test]
    fn test_health_score_calculation() {
        let collector = MetricsCollector::default();
        
        // Good project: high documentation, low complexity
        let good_score = collector.calculate_health_score(0.9, 5.0);
        assert!(good_score > 0.7);
        
        // Poor project: low documentation, high complexity
        let poor_score = collector.calculate_health_score(0.1, 30.0);
        assert!(poor_score < 0.5);
    }
}