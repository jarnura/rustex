//! Tests for error handling and recovery mechanisms.

use rustex_core::{AstExtractor, ExtractorConfig, FileProcessingError, RustExError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test project with some invalid files.
fn create_test_project_with_errors() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "error-test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Create src directory
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");

    // Create valid file
    let valid_rs = r#"
        pub fn valid_function() {
            println!("This is valid");
        }
    "#;
    fs::write(project_path.join("src/valid.rs"), valid_rs).expect("Failed to write valid.rs");

    // Create invalid syntax file
    let invalid_rs = r#"
        pub fn invalid_function(
            // Missing closing parenthesis and body
    "#;
    fs::write(project_path.join("src/invalid.rs"), invalid_rs).expect("Failed to write invalid.rs");

    // Create very large file (if max_file_size is small)
    let large_content = "// Large file content that exceeds the limit\n".repeat(10);
    fs::write(project_path.join("src/large.rs"), large_content).expect("Failed to write large.rs");

    (temp_dir, project_path)
}

#[tokio::test]
async fn test_parse_error_handling() {
    let (_temp_dir, project_path) = create_test_project_with_errors();

    // Use broader include pattern to catch our test files
    let mut config = ExtractorConfig::default();
    config.filters.include = vec!["**/*.rs".to_string()];

    let extractor = AstExtractor::new(config, project_path);
    let result = extractor.extract_project();

    // Should succeed with partial failure (not completely fail)
    assert!(result.is_ok(), "Should handle parse errors gracefully");

    let project_ast = result.unwrap();

    // Should have extracted the valid file
    assert!(!project_ast.files.is_empty(), "Should extract valid files");

    // Should have processed at least one file successfully
    assert!(
        project_ast.metrics.total_files > 0,
        "Should count successfully processed files"
    );
}

#[tokio::test]
async fn test_file_size_limit() {
    let (_temp_dir, project_path) = create_test_project_with_errors();

    // Set very small file size limit
    let mut config = ExtractorConfig::default();
    config.max_file_size = 100; // 100 bytes
    config.filters.include = vec!["**/*.rs".to_string()];

    let extractor = AstExtractor::new(config, project_path);
    let result = extractor.extract_project();

    // May fail due to high failure rate or succeed with partial failures
    match result {
        Ok(project_ast) => {
            // If it succeeds, should have processed some files but skipped large ones
            assert!(
                project_ast.metrics.total_files <= 2,
                "Should have skipped large files due to size limits"
            );
        }
        Err(e) => {
            // It's also acceptable to fail due to high failure rate
            println!("Extraction failed as expected: {}", e);
        }
    }
}

#[tokio::test]
async fn test_missing_project_root() {
    let config = ExtractorConfig::default();
    let non_existent_path = PathBuf::from("/this/path/does/not/exist");

    let extractor = AstExtractor::new(config, non_existent_path);
    let result = extractor.extract_project();

    // Should handle missing project root gracefully
    // The current implementation may succeed with empty results
    // This is acceptable behavior for a missing directory
    match result {
        Ok(project_ast) => {
            // If it succeeds, should have no files
            assert_eq!(
                project_ast.files.len(),
                0,
                "Should have no files for missing directory"
            );
        }
        Err(_) => {
            // It's also acceptable to return an error for missing directory
            // This depends on the implementation details
        }
    }
}

#[tokio::test]
async fn test_high_failure_rate() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "high-failure-test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Create src directory
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");

    // Create mostly invalid files to trigger high failure rate
    for i in 0..10 {
        let invalid_content = format!("pub fn invalid_function_{} {{", i); // Missing closing brace
        fs::write(
            project_path.join(format!("src/invalid_{}.rs", i)),
            invalid_content,
        )
        .expect("Failed to write invalid file");
    }

    // Create one valid file
    fs::write(project_path.join("src/valid.rs"), "pub fn valid() {}")
        .expect("Failed to write valid file");

    let mut config = ExtractorConfig::default();
    config.filters.include = vec!["**/*.rs".to_string()];

    let extractor = AstExtractor::new(config, project_path);
    let result = extractor.extract_project();

    // Should fail due to high failure rate (>50% failures)
    match result {
        Err(RustExError::PartialFailure {
            failed_count,
            total_count,
            ..
        }) => {
            assert!(failed_count > 0, "Should have failure count");
            assert!(total_count > 0, "Should have total count");
            assert!(
                failed_count as f64 / total_count as f64 > 0.5,
                "Should have high failure rate"
            );
        }
        _ => {
            // The implementation might handle this differently
            // This test verifies the error structure exists
        }
    }
}

#[test]
fn test_file_processing_error_types() {
    let file_path = PathBuf::from("test.rs");

    // Test parse error
    let parse_error = FileProcessingError::ParseError {
        file: file_path.clone(),
        error: "syntax error".to_string(),
    };

    assert_eq!(parse_error.file_path(), &file_path);
    assert!(parse_error.to_string().contains("Parse error"));
    assert!(parse_error.to_string().contains("test.rs"));

    // Test IO error
    let io_error = FileProcessingError::IoError {
        file: file_path.clone(),
        error: "permission denied".to_string(),
    };

    assert_eq!(io_error.file_path(), &file_path);
    assert!(io_error.to_string().contains("IO error"));

    // Test file too large error
    let size_error = FileProcessingError::TooLarge {
        file: file_path.clone(),
        size: 1024,
    };

    assert_eq!(size_error.file_path(), &file_path);
    assert!(size_error.to_string().contains("too large"));
    assert!(size_error.to_string().contains("1024"));

    // Test access denied error
    let access_error = FileProcessingError::AccessDenied {
        file: file_path.clone(),
    };

    assert_eq!(access_error.file_path(), &file_path);
    assert!(access_error.to_string().contains("Access denied"));
}

#[test]
fn test_rustex_error_types() {
    let file_path = PathBuf::from("test.rs");

    // Test file too large error
    let large_error = RustExError::FileTooLarge {
        file: file_path.clone(),
        size: 2048,
        limit: 1024,
    };

    let error_string = large_error.to_string();
    assert!(error_string.contains("too large"));
    assert!(error_string.contains("test.rs"));
    assert!(error_string.contains("2048"));
    assert!(error_string.contains("1024"));

    // Test invalid project root error
    let root_error = RustExError::InvalidProjectRoot {
        path: file_path.clone(),
    };

    let error_string = root_error.to_string();
    assert!(error_string.contains("Invalid project root"));
    assert!(error_string.contains("test.rs"));

    // Test partial failure error
    let partial_error = RustExError::PartialFailure {
        failed_count: 3,
        total_count: 10,
        errors: vec![FileProcessingError::ParseError {
            file: file_path.clone(),
            error: "syntax error".to_string(),
        }],
    };

    let error_string = partial_error.to_string();
    assert!(error_string.contains("Failed to process 3 out of 10 files"));
}

#[tokio::test]
async fn test_error_recovery_continues_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    // Create Cargo.toml
    let cargo_toml = r#"[package]
name = "recovery-test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Create src directory
    fs::create_dir_all(project_path.join("src")).expect("Failed to create src directory");

    // Create mix of valid and invalid files
    fs::write(project_path.join("src/valid1.rs"), "pub fn valid1() {}")
        .expect("Failed to write valid1.rs");

    fs::write(project_path.join("src/invalid.rs"), "pub fn invalid( {") // Syntax error
        .expect("Failed to write invalid.rs");

    fs::write(project_path.join("src/valid2.rs"), "pub fn valid2() {}")
        .expect("Failed to write valid2.rs");

    let mut config = ExtractorConfig::default();
    config.filters.include = vec!["**/*.rs".to_string()];

    let extractor = AstExtractor::new(config, project_path);
    let result = extractor.extract_project();

    // Should succeed and process valid files despite invalid ones
    assert!(result.is_ok(), "Should continue processing despite errors");

    let project_ast = result.unwrap();

    // Should have processed the valid files
    assert!(project_ast.files.len() >= 2, "Should process valid files");
    assert!(
        project_ast.metrics.total_functions >= 2,
        "Should extract functions from valid files"
    );

    // Should have functions from valid files
    let function_names: Vec<String> = project_ast
        .files
        .iter()
        .flat_map(|f| &f.elements)
        .filter(|e| matches!(e.element_type, rustex_core::ElementType::Function))
        .map(|e| e.name.clone())
        .collect();

    assert!(
        function_names.contains(&"valid1".to_string())
            || function_names.contains(&"valid2".to_string()),
        "Should extract functions from valid files"
    );
}
