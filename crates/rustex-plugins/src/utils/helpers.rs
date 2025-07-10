//! Helper utilities for plugin development.

use rustex_core::{CodeElement, ElementType};
use std::collections::HashMap;

/// Helper functions for working with code elements.
pub struct ElementHelpers;

impl ElementHelpers {
    /// Filter elements by type.
    pub fn filter_by_type<'a>(elements: &'a [&'a CodeElement], element_type: ElementType) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.element_type == element_type)
            .copied()
            .collect()
    }
    
    /// Filter elements by visibility.
    pub fn filter_by_visibility<'a>(elements: &'a [&'a CodeElement], visibility: rustex_core::Visibility) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.visibility == visibility)
            .copied()
            .collect()
    }
    
    /// Get public elements only.
    pub fn public_elements<'a>(elements: &'a [&'a CodeElement]) -> Vec<&'a CodeElement> {
        Self::filter_by_visibility(elements, rustex_core::Visibility::Public)
    }
    
    /// Get elements with documentation.
    pub fn documented_elements<'a>(elements: &'a [&'a CodeElement]) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| !e.doc_comments.is_empty())
            .copied()
            .collect()
    }
    
    /// Get elements without documentation.
    pub fn undocumented_elements<'a>(elements: &'a [&'a CodeElement]) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.doc_comments.is_empty())
            .copied()
            .collect()
    }
    
    /// Group elements by type.
    pub fn group_by_type<'a>(elements: &'a [&'a CodeElement]) -> HashMap<ElementType, Vec<&'a CodeElement>> {
        let mut groups = HashMap::new();
        for element in elements {
            groups.entry(element.element_type.clone())
                .or_insert_with(Vec::new)
                .push(*element);
        }
        groups
    }
    
    /// Group elements by file.
    pub fn group_by_file<'a>(elements: &'a [&'a CodeElement]) -> HashMap<String, Vec<&'a CodeElement>> {
        let mut groups = HashMap::new();
        for element in elements {
            let file_path = element.location.file_path.to_string_lossy().to_string();
            groups.entry(file_path)
                .or_insert_with(Vec::new)
                .push(*element);
        }
        groups
    }
    
    /// Get complex elements (complexity > threshold).
    pub fn complex_elements<'a>(elements: &'a [&'a CodeElement], threshold: u32) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.complexity.unwrap_or(1) > threshold)
            .copied()
            .collect()
    }
    
    /// Calculate average complexity.
    pub fn average_complexity(elements: &[&CodeElement]) -> f64 {
        if elements.is_empty() {
            return 0.0;
        }
        
        let total: u32 = elements.iter()
            .filter_map(|e| e.complexity)
            .sum();
            
        total as f64 / elements.len() as f64
    }
    
    /// Find elements with specific attributes.
    pub fn with_attribute<'a>(elements: &'a [&'a CodeElement], attribute: &str) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.attributes.iter().any(|attr| attr.contains(attribute)))
            .copied()
            .collect()
    }
    
    /// Find elements with dependencies.
    pub fn with_dependencies<'a>(elements: &'a [&'a CodeElement]) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| !e.dependencies.is_empty())
            .copied()
            .collect()
    }
    
    /// Find elements that depend on a specific element.
    pub fn dependent_on<'a>(elements: &'a [&'a CodeElement], target: &str) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.dependencies.contains(&target.to_string()))
            .copied()
            .collect()
    }
    
    /// Get elements within a line range.
    pub fn in_line_range<'a>(elements: &'a [&'a CodeElement], start: usize, end: usize) -> Vec<&'a CodeElement> {
        elements.iter()
            .filter(|e| e.location.line_start >= start && e.location.line_end <= end)
            .copied()
            .collect()
    }
}

/// Helper functions for calculating metrics.
pub struct MetricHelpers;

impl MetricHelpers {
    /// Calculate percentage.
    pub fn percentage(numerator: usize, denominator: usize) -> f64 {
        if denominator == 0 {
            0.0
        } else {
            (numerator as f64 / denominator as f64) * 100.0
        }
    }
    
    /// Calculate ratio.
    pub fn ratio(numerator: usize, denominator: usize) -> f64 {
        if denominator == 0 {
            0.0
        } else {
            numerator as f64 / denominator as f64
        }
    }
    
    /// Calculate median of a vector of numbers.
    pub fn median(mut values: Vec<f64>) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }
    
    /// Calculate standard deviation.
    pub fn std_deviation(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
            
        variance.sqrt()
    }
    
    /// Calculate quartiles (Q1, Q2/median, Q3).
    pub fn quartiles(mut values: Vec<f64>) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        
        let q2 = Self::median(values.clone());
        
        let q1_values = values[..len / 2].to_vec();
        let q1 = Self::median(q1_values);
        
        let q3_values = values[len.div_ceil(2)..].to_vec();
        let q3 = Self::median(q3_values);
        
        (q1, q2, q3)
    }
}

/// Helper functions for string processing.
pub struct StringHelpers;

impl StringHelpers {
    /// Convert camelCase to snake_case.
    pub fn camel_to_snake(input: &str) -> String {
        let mut result = String::new();
        for (i, ch) in input.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }
    
    /// Convert snake_case to camelCase.
    pub fn snake_to_camel(input: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;
        
        for ch in input.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    /// Extract words from a string (split on various delimiters).
    pub fn extract_words(input: &str) -> Vec<String> {
        input.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }
    
    /// Calculate similarity between two strings (simple Jaccard index).
    pub fn similarity(a: &str, b: &str) -> f64 {
        let words_a: std::collections::HashSet<_> = Self::extract_words(a).into_iter().collect();
        let words_b: std::collections::HashSet<_> = Self::extract_words(b).into_iter().collect();
        
        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }
        
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Helper functions for working with plugin configurations.
pub struct ConfigHelpers;

impl ConfigHelpers {
    /// Merge two JSON configurations.
    pub fn merge_configs(base: &serde_json::Value, override_config: &serde_json::Value) -> serde_json::Value {
        match (base, override_config) {
            (serde_json::Value::Object(base_map), serde_json::Value::Object(override_map)) => {
                let mut result = base_map.clone();
                for (key, value) in override_map {
                    if let Some(base_value) = base_map.get(key) {
                        result.insert(key.clone(), Self::merge_configs(base_value, value));
                    } else {
                        result.insert(key.clone(), value.clone());
                    }
                }
                serde_json::Value::Object(result)
            }
            _ => override_config.clone(),
        }
    }
    
    /// Get a configuration value with a default.
    pub fn get_config_value<T>(config: &serde_json::Value, key: &str, default: T) -> T 
    where 
        T: serde::de::DeserializeOwned + Clone,
    {
        config.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(default)
    }
    
    /// Validate required configuration keys.
    pub fn validate_required_keys(config: &serde_json::Value, required_keys: &[&str]) -> Result<(), String> {
        if let serde_json::Value::Object(map) = config {
            for key in required_keys {
                if !map.contains_key(*key) {
                    return Err(format!("Missing required configuration key: {}", key));
                }
            }
            Ok(())
        } else {
            Err("Configuration must be an object".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::{CodeElement, CodeLocation, Visibility};
    use std::path::PathBuf;

    fn create_test_element(name: &str, element_type: ElementType, complexity: Option<u32>) -> CodeElement {
        CodeElement {
            id: format!("{:?}_{}_{}", element_type, name, 1),
            element_type,
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
            complexity,
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
    fn test_element_filtering() {
        let elements = vec![
            create_test_element("func1", ElementType::Function, Some(5)),
            create_test_element("struct1", ElementType::Struct, Some(2)),
            create_test_element("func2", ElementType::Function, Some(15)),
        ];
        
        let element_refs: Vec<&CodeElement> = elements.iter().collect();
        
        let functions = ElementHelpers::filter_by_type(&element_refs, ElementType::Function);
        assert_eq!(functions.len(), 2);
        
        let complex = ElementHelpers::complex_elements(&element_refs, 10);
        assert_eq!(complex.len(), 1);
        assert_eq!(complex[0].name, "func2");
    }

    #[test]
    fn test_metric_calculations() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let median = MetricHelpers::median(values.clone());
        assert_eq!(median, 3.0);
        
        let (q1, q2, q3) = MetricHelpers::quartiles(values.clone());
        assert_eq!(q2, 3.0); // median
        assert!(q1 < q2 && q2 < q3);
        
        let std_dev = MetricHelpers::std_deviation(&values);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_string_helpers() {
        assert_eq!(StringHelpers::camel_to_snake("camelCase"), "camel_case");
        assert_eq!(StringHelpers::snake_to_camel("snake_case"), "snakeCase");
        
        let words = StringHelpers::extract_words("hello-world_test");
        assert_eq!(words, vec!["hello", "world", "test"]);
        
        let similarity = StringHelpers::similarity("hello world", "hello earth");
        assert!(similarity > 0.0 && similarity < 1.0);
    }

    #[test]
    fn test_config_helpers() {
        let base = serde_json::json!({
            "a": 1,
            "b": {"c": 2}
        });
        
        let override_config = serde_json::json!({
            "b": {"d": 3},
            "e": 4
        });
        
        let merged = ConfigHelpers::merge_configs(&base, &override_config);
        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"]["c"], 2);
        assert_eq!(merged["b"]["d"], 3);
        assert_eq!(merged["e"], 4);
    }
}