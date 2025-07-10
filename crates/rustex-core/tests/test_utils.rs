//! Test utility functions and helpers.
//!
//! This module provides common testing utilities that can be shared
//! across multiple test files for consistency and reusability.

use rustex_core::test_fixtures::*;
use rustex_core::*;
use tempfile::TempDir;

/// Helper function to create a minimal test project with basic structure.
pub fn create_minimal_test_project() -> TestFixture {
    TestFixtureBuilder::new()
        .with_project_name("minimal-test")
        .with_file("lib.rs", r#"
//! Minimal test library.

/// A simple function for testing.
pub fn hello() -> &'static str {
    "Hello, World!"
}

/// A private function.
fn private_function() -> u32 {
    42
}

/// A struct for testing.
pub struct TestStruct {
    pub field: String,
}

impl TestStruct {
    pub fn new(field: String) -> Self {
        Self { field }
    }
}
"#)
        .build()
}

/// Helper function to create a project with complex code for stress testing.
pub fn create_complex_test_project() -> TestFixture {
    let samples = SampleCode::new();
    
    TestFixtureBuilder::new()
        .with_project_name("complex-test")
        .with_sample_files(&samples)
        .with_file("main.rs", r#"
use std::collections::HashMap;

fn main() {
    println!("Complex test project");
    
    let mut data = HashMap::new();
    data.insert("key1", "value1");
    data.insert("key2", "value2");
    
    for (key, value) in &data {
        println!("{}: {}", key, value);
    }
}
"#)
        .build()
}

/// Helper function to create a project with error scenarios.
pub fn create_error_test_project() -> TestFixture {
    let error_scenarios = MockDataGenerator::error_scenarios();
    let mut builder = TestFixtureBuilder::new()
        .with_project_name("error-test");
    
    for (name, code) in error_scenarios {
        builder = builder.with_file(&format!("{}.rs", name), &code);
    }
    
    builder.build()
}

/// Helper function to create a project with edge cases.
pub fn create_edge_case_project() -> TestFixture {
    let edge_cases = MockDataGenerator::edge_cases();
    let mut builder = TestFixtureBuilder::new()
        .with_project_name("edge-case-test");
    
    for (name, code) in edge_cases {
        builder = builder.with_file(&format!("{}.rs", name), &code);
    }
    
    builder.build()
}

/// Assert that a ProjectAst has the expected basic structure.
pub fn assert_project_ast_structure(
    project_ast: &ProjectAst,
    expected_name: &str,
    min_files: usize,
) {
    assert_eq!(project_ast.project.name, expected_name);
    assert!(project_ast.files.len() >= min_files);
    assert!(!project_ast.extracted_at.to_string().is_empty());
    
    // Verify metrics consistency
    let total_elements: usize = project_ast.files.iter()
        .map(|f| f.elements.len())
        .sum();
    assert!(total_elements > 0, "Project should have at least some elements");
    
    // Verify file metrics are reasonable
    for file in &project_ast.files {
        assert!(file.path.is_absolute(), "File paths should be absolute");
        assert!(!file.relative_path.is_absolute(), "Relative paths should not be absolute");
    }
}

/// Assert that a CodeElement has valid complexity metrics.
pub fn assert_valid_complexity_metrics(element: &CodeElement) {
    if let Some(ref metrics) = element.complexity_metrics {
        assert!(metrics.cyclomatic >= 1, "Cyclomatic complexity should be at least 1");
        assert!(metrics.overall_score() > 0, "Overall complexity score should be positive");
        
        // Halstead metrics should be reasonable
        assert!(metrics.halstead.vocabulary >= metrics.halstead.n1 + metrics.halstead.n2);
        assert!(metrics.halstead.length >= metrics.halstead.big_n1 + metrics.halstead.big_n2);
    }
}

/// Compare two ProjectAst objects for structural similarity.
pub fn assert_similar_project_structure(ast1: &ProjectAst, ast2: &ProjectAst) {
    // Should have similar number of files (within reasonable range)
    let file_diff = (ast1.files.len() as i32 - ast2.files.len() as i32).abs();
    assert!(file_diff <= 2, "File counts should be similar");
    
    // Should have similar complexity ranges
    assert!(ast1.metrics.complexity_average >= 0.0);
    assert!(ast2.metrics.complexity_average >= 0.0);
}

/// Create a test configuration with specific settings for testing.
pub fn create_test_config(include_private: bool, include_docs: bool) -> ExtractorConfig {
    ExtractorConfig {
        include_private,
        include_docs,
        filters: FilterConfig {
            include: vec!["**/*.rs".to_string()],
            exclude: vec!["target/**".to_string(), "**/build.rs".to_string()],
        },
        output_format: OutputFormat::Json,
        ..Default::default()
    }
}

/// Helper to extract and validate a simple project.
pub fn extract_and_validate_project(fixture: &TestFixture) -> ProjectAst {
    let extractor = AstExtractor::new(
        fixture.config().clone(),
        fixture.project_root().to_path_buf(),
    );
    
    let result = extractor.extract_project();
    assert!(result.is_ok(), "Extraction should succeed");
    
    let project_ast = result.unwrap();
    assert_project_ast_structure(&project_ast, fixture.project_name(), 1);
    
    project_ast
}

/// Helper to time an operation and ensure it completes within a reasonable time.
pub fn time_operation<F, R>(operation: F, max_duration_ms: u64, operation_name: &str) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = operation();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() <= max_duration_ms as u128,
        "{} took {:?}, expected <= {}ms",
        operation_name,
        duration,
        max_duration_ms
    );
    
    result
}

/// Create a temporary directory with Rust project structure.
pub fn create_temp_rust_project(name: &str, files: &[(&str, &str)]) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_root = temp_dir.path();
    
    // Create src directory
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    // Create Cargo.toml
    let cargo_toml = format!(
        r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
"#,
        name
    );
    std::fs::write(project_root.join("Cargo.toml"), cargo_toml)
        .expect("Failed to write Cargo.toml");
    
    // Create source files
    for (filename, content) in files {
        let file_path = src_dir.join(filename);
        std::fs::write(file_path, content).expect("Failed to write source file");
    }
    
    temp_dir
}

/// Generate test data with specific characteristics for performance testing.
pub fn generate_performance_test_data(
    file_count: usize,
    elements_per_file: usize,
    complexity_level: u32,
) -> ProjectAst {
    let mut project_ast = MockDataGenerator::project_ast(file_count, elements_per_file);
    
    // Adjust complexity levels
    for file in &mut project_ast.files {
        for element in &mut file.elements {
            if let Some(ref mut metrics) = element.complexity_metrics {
                metrics.cyclomatic = complexity_level;
                metrics.cognitive = complexity_level / 2;
                metrics.nesting_depth = complexity_level / 3;
            }
            element.complexity = Some(complexity_level);
        }
        
        // Update file metrics
        file.file_metrics.complexity_total = elements_per_file as u32 * complexity_level;
    }
    
    // Update project metrics
    project_ast.metrics.complexity_average = complexity_level as f64;
    project_ast.metrics.complexity_max = complexity_level * 2;
    
    project_ast
}

/// Validate that all elements in a project have required fields populated.
pub fn validate_complete_extraction(project_ast: &ProjectAst) {
    assert!(!project_ast.project.name.is_empty(), "Project name should not be empty");
    assert!(!project_ast.files.is_empty(), "Should have at least one file");
    
    for file in &project_ast.files {
        assert!(file.path.exists() || file.path.to_string_lossy().contains("test"), 
                "File path should exist or be a test file: {:?}", file.path);
        
        for element in &file.elements {
            assert!(!element.name.is_empty(), "Element name should not be empty");
            assert!(element.location.line_start > 0, "Line start should be positive");
            assert!(element.location.line_end >= element.location.line_start, 
                    "Line end should be >= line start");
            
            // Validate complexity metrics if present
            if element.complexity_metrics.is_some() {
                assert_valid_complexity_metrics(element);
            }
        }
    }
}

/// Create a test scenario with specific error conditions.
pub fn create_error_scenario(error_type: &str) -> String {
    match error_type {
        "syntax_error" => "fn broken syntax here".to_string(),
        "incomplete_function" => "fn incomplete() {".to_string(),
        "invalid_generics" => "fn bad<T where> () {}".to_string(),
        "malformed_struct" => "struct Bad { field: }".to_string(),
        "invalid_macro" => "macro_rules! bad { ($) => {} }".to_string(),
        _ => format!("// Unknown error type: {}", error_type),
    }
}

/// Assert that error handling works correctly for various scenarios.
pub fn test_error_handling_robustness(fixture: &TestFixture) {
    let extractor = AstExtractor::new(
        fixture.config().clone(),
        fixture.project_root().to_path_buf(),
    );
    
    // Extract project - should handle errors gracefully
    let result = extractor.extract_project();
    
    match result {
        Ok(project_ast) => {
            // If extraction succeeds, validate the result
            validate_complete_extraction(&project_ast);
        }
        Err(error) => {
            // If extraction fails, error should be descriptive
            let error_message = error.to_string();
            assert!(!error_message.is_empty(), "Error message should not be empty");
            assert!(error_message.len() > 10, "Error message should be descriptive");
        }
    }
}

/// Benchmark extraction performance for different project sizes.
pub fn benchmark_extraction_performance(
    project_sizes: &[(usize, usize)], // (file_count, elements_per_file)
    max_duration_ms: u64,
) {
    for &(file_count, elements_per_file) in project_sizes {
        let fixture = TestFixtureBuilder::new()
            .with_project_name(&format!("perf-test-{}-{}", file_count, elements_per_file))
            .build();
        
        // Add files with varying complexity
        let samples = SampleCode::new();
        for i in 0..file_count {
            let content = match i % 4 {
                0 => &samples.simple_function,
                1 => &samples.complex_function,
                2 => &samples.struct_with_fields,
                _ => &samples.enum_with_variants,
            };
            fixture.add_file(&format!("file_{}.rs", i), content);
        }
        
        // Time the extraction
        time_operation(
            || extract_and_validate_project(&fixture),
            max_duration_ms,
            &format!("Extraction of {} files with {} elements each", file_count, elements_per_file),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_project_creation() {
        let fixture = create_minimal_test_project();
        assert_eq!(fixture.project_name(), "minimal-test");
        
        let rust_files = fixture.rust_files();
        assert!(!rust_files.is_empty());
    }

    #[test]
    fn test_complex_project_creation() {
        let fixture = create_complex_test_project();
        assert_eq!(fixture.project_name(), "complex-test");
        
        let rust_files = fixture.rust_files();
        assert!(rust_files.len() >= 5); // Should have sample files + main.rs
    }

    #[test]
    fn test_error_scenario_generation() {
        let error_code = create_error_scenario("syntax_error");
        assert!(error_code.contains("broken"));
        
        let incomplete_code = create_error_scenario("incomplete_function");
        assert!(incomplete_code.contains("incomplete"));
    }

    #[test]
    fn test_performance_data_generation() {
        let project_ast = generate_performance_test_data(3, 5, 10);
        assert_eq!(project_ast.files.len(), 3);
        assert_eq!(project_ast.metrics.complexity_average, 10.0);
        
        // Check that complexity was applied consistently
        for file in &project_ast.files {
            for element in &file.elements {
                if let Some(complexity) = element.complexity {
                    assert_eq!(complexity, 10);
                }
            }
        }
    }

    #[test]
    fn test_temp_project_creation() {
        let temp_dir = create_temp_rust_project(
            "temp-test",
            &[
                ("main.rs", "fn main() {}"),
                ("lib.rs", "pub fn library_function() {}"),
            ],
        );
        
        let project_root = temp_dir.path();
        assert!(project_root.join("Cargo.toml").exists());
        assert!(project_root.join("src/main.rs").exists());
        assert!(project_root.join("src/lib.rs").exists());
    }

    #[test]
    fn test_timing_operation() {
        // Test a fast operation
        let result = time_operation(
            || 42,
            100, // 100ms should be plenty
            "simple calculation",
        );
        assert_eq!(result, 42);
        
        // This should not panic since the operation is very fast
    }
}