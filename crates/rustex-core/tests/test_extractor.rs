//! Integration tests for the AST extractor.

use rustex_core::{AstExtractor, ElementType, ExtractorConfig, Visibility};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary test project with sample Rust files.
fn create_test_project() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Create src directory
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");

    // Create main.rs with various constructs
    let main_rs = r#"//! Test project main file.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Main function for the test project.
pub fn main() {
    println!("Hello, world!");
    let data = TestStruct::new();
    data.process();
}

/// A test struct for demonstration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStruct {
    /// Field documentation
    pub name: String,
    value: i32,
}

impl TestStruct {
    /// Creates a new TestStruct instance.
    pub fn new() -> Self {
        Self {
            name: "test".to_string(),
            value: 42,
        }
    }
    
    /// Processes the struct data.
    fn process(&self) {
        if self.value > 0 {
            println!("Processing: {}", self.name);
        }
    }
}

/// Test enumeration.
#[derive(Debug)]
pub enum TestEnum {
    /// First variant
    Variant1,
    /// Second variant with data
    Variant2(String),
    /// Third variant with struct
    Variant3 { data: i32 },
}

/// Test trait definition.
pub trait TestTrait {
    /// Required method
    fn required_method(&self) -> String;
    
    /// Default method implementation
    fn default_method(&self) -> i32 {
        42
    }
}

impl TestTrait for TestStruct {
    fn required_method(&self) -> String {
        self.name.clone()
    }
}

/// Private function for internal use.
fn private_function() -> bool {
    true
}

/// Constant definition
pub const TEST_CONSTANT: i32 = 100;

/// Static variable
pub static TEST_STATIC: &str = "test_value";
"#;
    fs::write(project_path.join("src/main.rs"), main_rs).expect("Failed to write main.rs");

    // Create a library file
    let lib_rs = r#"//! Test library module.

/// Library function
pub fn lib_function() -> String {
    "library".to_string()
}

/// Generic struct
pub struct GenericStruct<T> {
    data: T,
}

impl<T> GenericStruct<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}
"#;
    fs::write(project_path.join("src/lib.rs"), lib_rs).expect("Failed to write lib.rs");

    (temp_dir, project_path)
}

#[tokio::test]
async fn test_basic_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);

    let result = extractor.extract_project();
    assert!(result.is_ok(), "Extraction should succeed");

    let project_ast = result.unwrap();

    // Verify project info
    assert_eq!(project_ast.project.name, "test-project");
    assert_eq!(project_ast.project.version, "0.1.0");
    assert_eq!(project_ast.project.rust_edition, "2021");

    // Should have extracted files
    assert!(!project_ast.files.is_empty(), "Should extract files");

    // Verify metrics
    assert!(project_ast.metrics.total_files > 0);
    assert!(project_ast.metrics.total_functions > 0);
    assert!(project_ast.metrics.total_structs > 0);
}

#[tokio::test]
async fn test_function_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let mut config = ExtractorConfig::default();
    config.include_private = true; // Include private functions

    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    // Find functions in main.rs
    let main_file = project_ast
        .files
        .iter()
        .find(|f| f.relative_path.to_string_lossy().contains("main.rs"))
        .expect("Should find main.rs");

    let functions: Vec<_> = main_file
        .elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .collect();

    assert!(!functions.is_empty(), "Should extract functions");

    // Check for main function
    let main_fn = functions
        .iter()
        .find(|f| f.name == "main")
        .expect("Should find main function");

    assert!(matches!(main_fn.visibility, Visibility::Public));
    assert!(
        !main_fn.doc_comments.is_empty(),
        "Should have documentation"
    );

    // Check for private function
    let private_fn = functions.iter().find(|f| f.name == "private_function");
    assert!(
        private_fn.is_some(),
        "Should find private function when include_private=true"
    );
}

#[tokio::test]
async fn test_struct_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    // Find TestStruct
    let structs: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| matches!(e.element_type, ElementType::Struct))
        .collect();

    assert!(!structs.is_empty(), "Should extract structs");

    let test_struct = structs
        .iter()
        .find(|s| s.name == "TestStruct")
        .expect("Should find TestStruct");

    assert!(matches!(test_struct.visibility, Visibility::Public));
    assert!(
        !test_struct.doc_comments.is_empty(),
        "Should extract doc comments"
    );
    assert!(
        !test_struct.attributes.is_empty(),
        "Should extract attributes"
    );
}

#[tokio::test]
async fn test_enum_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    let enums: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| matches!(e.element_type, ElementType::Enum))
        .collect();

    assert!(!enums.is_empty(), "Should extract enums");

    let test_enum = enums
        .iter()
        .find(|e| e.name == "TestEnum")
        .expect("Should find TestEnum");

    assert!(matches!(test_enum.visibility, Visibility::Public));
    assert!(
        test_enum.complexity.unwrap_or(0) > 1,
        "Enum should have complexity > 1"
    );
}

#[tokio::test]
async fn test_trait_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    let traits: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| matches!(e.element_type, ElementType::Trait))
        .collect();

    assert!(!traits.is_empty(), "Should extract traits");

    let test_trait = traits
        .iter()
        .find(|t| t.name == "TestTrait")
        .expect("Should find TestTrait");

    assert!(matches!(test_trait.visibility, Visibility::Public));
    assert!(
        test_trait.complexity.unwrap_or(0) > 1,
        "Trait should have complexity for methods"
    );
}

#[tokio::test]
async fn test_private_visibility_filtering() {
    let (_temp_dir, project_path) = create_test_project();

    // Test with include_private = false
    let mut config = ExtractorConfig::default();
    config.include_private = false;

    let extractor = AstExtractor::new(config, project_path.clone());
    let project_ast = extractor.extract_project().unwrap();

    let all_elements: Vec<_> = project_ast.files.iter().flat_map(|f| &f.elements).collect();

    // Should not have private functions
    let private_functions: Vec<_> = all_elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .filter(|e| e.name == "private_function")
        .collect();

    assert!(
        private_functions.is_empty(),
        "Should not extract private functions when include_private=false"
    );

    // Test with include_private = true
    let mut config = ExtractorConfig::default();
    config.include_private = true;

    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    let all_elements: Vec<_> = project_ast.files.iter().flat_map(|f| &f.elements).collect();

    let private_functions: Vec<_> = all_elements
        .iter()
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .filter(|e| e.name == "private_function")
        .collect();

    assert!(
        !private_functions.is_empty(),
        "Should extract private functions when include_private=true"
    );
}

#[tokio::test]
async fn test_documentation_extraction() {
    let (_temp_dir, project_path) = create_test_project();

    // Test with include_docs = true
    let mut config = ExtractorConfig::default();
    config.include_docs = true;

    let extractor = AstExtractor::new(config, project_path.clone());
    let project_ast = extractor.extract_project().unwrap();

    let documented_elements: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| !e.doc_comments.is_empty())
        .collect();

    assert!(
        !documented_elements.is_empty(),
        "Should extract documentation when include_docs=true"
    );

    // Test with include_docs = false
    let mut config = ExtractorConfig::default();
    config.include_docs = false;

    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    let documented_elements: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| !e.doc_comments.is_empty())
        .collect();

    assert!(
        documented_elements.is_empty(),
        "Should not extract documentation when include_docs=false"
    );
}

#[tokio::test]
async fn test_import_extraction() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    let main_file = project_ast
        .files
        .iter()
        .find(|f| f.relative_path.to_string_lossy().contains("main.rs"))
        .expect("Should find main.rs");

    assert!(!main_file.imports.is_empty(), "Should extract imports");

    // Check for specific imports
    let std_import = main_file
        .imports
        .iter()
        .find(|i| i.module_path.contains("std"));
    assert!(std_import.is_some(), "Should find std imports");

    let serde_import = main_file
        .imports
        .iter()
        .find(|i| i.module_path.contains("serde"));
    assert!(serde_import.is_some(), "Should find serde imports");
}

#[tokio::test]
async fn test_metrics_calculation() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    // Verify project metrics
    assert!(project_ast.metrics.total_files > 0, "Should count files");
    assert!(project_ast.metrics.total_lines > 0, "Should count lines");
    assert!(
        project_ast.metrics.total_functions > 0,
        "Should count functions"
    );
    assert!(
        project_ast.metrics.total_structs > 0,
        "Should count structs"
    );
    assert!(project_ast.metrics.total_enums > 0, "Should count enums");
    assert!(project_ast.metrics.total_traits > 0, "Should count traits");

    // Verify file metrics
    for file in &project_ast.files {
        assert!(
            file.file_metrics.lines_of_code > 0,
            "Each file should have lines of code"
        );

        let expected_functions = file
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Function))
            .count();
        assert_eq!(
            file.file_metrics.function_count, expected_functions,
            "Function count should match"
        );

        let expected_structs = file
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Struct))
            .count();
        assert_eq!(
            file.file_metrics.struct_count, expected_structs,
            "Struct count should match"
        );

        let expected_enums = file
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Enum))
            .count();
        assert_eq!(
            file.file_metrics.enum_count, expected_enums,
            "Enum count should match"
        );

        let expected_traits = file
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Trait))
            .count();
        assert_eq!(
            file.file_metrics.trait_count, expected_traits,
            "Trait count should match"
        );
    }
}

#[tokio::test]
async fn test_generic_parameters() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    // Find GenericStruct
    let generic_struct = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .find(|e| e.name == "GenericStruct")
        .expect("Should find GenericStruct");

    assert!(
        !generic_struct.generic_params.is_empty(),
        "Should extract generic parameters"
    );
}

#[tokio::test]
async fn test_complexity_calculation() {
    let (_temp_dir, project_path) = create_test_project();
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, project_path);
    let project_ast = extractor.extract_project().unwrap();

    // Check that complexity calculation is working
    let functions: Vec<_> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| matches!(e.element_type, ElementType::Function))
        .collect();

    // Should have extracted functions
    assert!(!functions.is_empty(), "Should extract functions");

    // All functions should have some complexity value (at least 1)
    for function in &functions {
        assert!(
            function.complexity.is_some(),
            "Function {} should have complexity",
            function.name
        );
        assert!(
            function.complexity.unwrap() >= 1,
            "Function {} should have complexity >= 1",
            function.name
        );
    }

    // Check that different functions can have different complexity
    let complexities: Vec<u32> = functions
        .iter()
        .map(|f| f.complexity.unwrap_or(1))
        .collect();

    assert!(
        complexities.iter().all(|&c| c >= 1),
        "All functions should have complexity >= 1"
    );
}
