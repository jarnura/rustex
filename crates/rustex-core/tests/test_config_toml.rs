//! Tests for TOML configuration system.

use rustex_core::{ConfigUseCase, ExtractorConfig, OutputFormat};
use tempfile::TempDir;

#[test]
fn test_toml_serialization_roundtrip() {
    let config = ExtractorConfig::default();

    // Serialize to TOML
    let toml_str = config
        .to_toml_string()
        .expect("Failed to serialize to TOML");

    // Deserialize back
    let deserialized =
        ExtractorConfig::from_toml_str(&toml_str).expect("Failed to deserialize from TOML");

    // Should be identical
    assert_eq!(config.include_docs, deserialized.include_docs);
    assert_eq!(config.include_private, deserialized.include_private);
    assert_eq!(config.parse_dependencies, deserialized.parse_dependencies);
    assert_eq!(config.max_file_size, deserialized.max_file_size);
    assert_eq!(config.filters.include, deserialized.filters.include);
    assert_eq!(config.filters.exclude, deserialized.filters.exclude);
    assert_eq!(config.plugins, deserialized.plugins);
}

#[test]
fn test_toml_file_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");

    let mut config = ExtractorConfig::default();
    config.include_private = true;
    config.max_file_size = 5 * 1024 * 1024; // 5MB
    config.filters.include = vec!["**/*.rs".to_string()];

    // Save to file
    config
        .to_toml_file(&config_path)
        .expect("Failed to save config");
    assert!(config_path.exists());

    // Load from file
    let loaded_config =
        ExtractorConfig::from_toml_file(&config_path).expect("Failed to load config");

    assert_eq!(config.include_private, loaded_config.include_private);
    assert_eq!(config.max_file_size, loaded_config.max_file_size);
    assert_eq!(config.filters.include, loaded_config.filters.include);
}

#[test]
fn test_example_config_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("example.toml");

    ExtractorConfig::create_example_config(&config_path).expect("Failed to create example config");

    assert!(config_path.exists());

    // Verify the example config can be loaded
    let config =
        ExtractorConfig::from_toml_file(&config_path).expect("Failed to load example config");

    assert!(config.include_docs);
    assert!(!config.include_private);
    assert!(!config.parse_dependencies);
    assert_eq!(config.max_file_size, 10 * 1024 * 1024);
}

#[test]
fn test_config_validation() {
    // Valid config
    let valid_config = ExtractorConfig::default();
    assert!(valid_config.validate().is_ok());

    // Invalid max_file_size (zero)
    let mut invalid_config = ExtractorConfig::default();
    invalid_config.max_file_size = 0;
    assert!(invalid_config.validate().is_err());

    // Invalid max_file_size (too large)
    let mut invalid_config = ExtractorConfig::default();
    invalid_config.max_file_size = 200 * 1024 * 1024; // 200MB
    assert!(invalid_config.validate().is_err());

    // Empty include patterns
    let mut invalid_config = ExtractorConfig::default();
    invalid_config.filters.include.clear();
    assert!(invalid_config.validate().is_err());

    // Conflicting include/exclude patterns
    let mut invalid_config = ExtractorConfig::default();
    invalid_config.filters.include = vec!["src/**/*.rs".to_string()];
    invalid_config.filters.exclude = vec!["src/**/*.rs".to_string()];
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_config_merge() {
    let mut base_config = ExtractorConfig::default();
    base_config.include_docs = false;
    base_config.max_file_size = 1024;

    let mut override_config = ExtractorConfig::default();
    override_config.include_docs = true;
    override_config.include_private = true;
    override_config.filters.include = vec!["test/**/*.rs".to_string()];

    base_config.merge_with(override_config);

    // Should have merged values
    assert!(base_config.include_docs); // From override
    assert!(base_config.include_private); // From override
    assert_eq!(
        base_config.filters.include,
        vec!["test/**/*.rs".to_string()]
    ); // From override
    assert_eq!(base_config.max_file_size, 1024); // Original value kept (override was default)
}

#[test]
fn test_use_case_configs() {
    // Documentation use case
    let doc_config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);
    assert!(doc_config.include_docs);
    assert!(!doc_config.include_private);
    assert!(matches!(doc_config.output_format, OutputFormat::Markdown));
    assert!(doc_config
        .filters
        .include
        .contains(&"src/**/*.rs".to_string()));
    assert!(doc_config
        .filters
        .include
        .contains(&"examples/**/*.rs".to_string()));

    // Code analysis use case
    let analysis_config = ExtractorConfig::for_use_case(ConfigUseCase::CodeAnalysis);
    assert!(analysis_config.include_docs);
    assert!(analysis_config.include_private);
    assert!(analysis_config.parse_dependencies);
    assert!(matches!(analysis_config.output_format, OutputFormat::Json));
    assert!(analysis_config
        .filters
        .include
        .contains(&"**/*.rs".to_string()));

    // LLM training use case
    let llm_config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);
    assert!(llm_config.include_docs);
    assert!(!llm_config.include_private);
    assert!(!llm_config.parse_dependencies);
    assert!(matches!(llm_config.output_format, OutputFormat::Rag));
    assert!(llm_config.filters.exclude.contains(&"tests/**".to_string()));

    // Testing use case
    let test_config = ExtractorConfig::for_use_case(ConfigUseCase::Testing);
    assert!(!test_config.include_docs);
    assert!(test_config.include_private);
    assert_eq!(test_config.max_file_size, 1024 * 1024); // 1MB for faster testing
    assert!(test_config
        .filters
        .include
        .contains(&"tests/**/*.rs".to_string()));
}

#[test]
fn test_load_from_standard_locations() {
    // This test is tricky since it depends on the filesystem
    // For now, just verify it returns a valid config
    let config = ExtractorConfig::load_from_standard_locations();
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_toml_handling() {
    let invalid_toml = r#"
        include_docs = "not a boolean"
        max_file_size = "not a number"
        [filters
        # Missing closing bracket
    "#;

    let result = ExtractorConfig::from_toml_str(invalid_toml);
    assert!(result.is_err());
}

#[test]
fn test_partial_toml_config() {
    // TOML with only some fields specified
    let partial_toml = r#"
        include_docs = false
        max_file_size = 5242880
        
        [filters]
        include = ["lib/**/*.rs"]
    "#;

    let config =
        ExtractorConfig::from_toml_str(partial_toml).expect("Failed to parse partial TOML");

    // Specified fields should be set
    assert!(!config.include_docs);
    assert_eq!(config.max_file_size, 5242880);
    assert_eq!(config.filters.include, vec!["lib/**/*.rs"]);

    // Unspecified fields should have defaults
    assert!(!config.include_private); // Default is false
    assert!(!config.parse_dependencies); // Default is false
    assert!(matches!(config.output_format, OutputFormat::Json)); // Default
}

#[test]
fn test_toml_with_comments() {
    let toml_with_comments = r#"
        # Main configuration
        include_docs = true  # Include documentation
        include_private = false  # Skip private items
        
        # File processing
        max_file_size = 10485760  # 10MB limit
        
        [filters]
        # Files to process
        include = ["src/**/*.rs", "examples/**/*.rs"]
        # Files to skip
        exclude = ["target/**", "build/**"]
        
        # No plugins for now
        plugins = []
    "#;

    let config = ExtractorConfig::from_toml_str(toml_with_comments)
        .expect("Failed to parse TOML with comments");

    assert!(config.include_docs);
    assert!(!config.include_private);
    assert_eq!(config.max_file_size, 10485760);
    assert_eq!(
        config.filters.include,
        vec!["src/**/*.rs", "examples/**/*.rs"]
    );
    assert_eq!(config.filters.exclude, vec!["target/**", "build/**"]);
    assert!(config.plugins.is_empty());
}

#[test]
fn test_complex_toml_structure() {
    let complex_toml = r#"
        include_docs = true
        include_private = true
        parse_dependencies = true
        max_file_size = 20971520
        output_format = "Markdown"
        
        plugins = [
            "complexity-analyzer",
            "doc-generator",
            "dependency-graph"
        ]
        
        [filters]
        include = [
            "src/**/*.rs",
            "lib/**/*.rs", 
            "examples/**/*.rs"
        ]
        exclude = [
            "target/**",
            "build/**",
            "*.tmp"
        ]
    "#;

    let config =
        ExtractorConfig::from_toml_str(complex_toml).expect("Failed to parse complex TOML");

    assert!(config.include_docs);
    assert!(config.include_private);
    assert!(config.parse_dependencies);
    assert_eq!(config.max_file_size, 20971520);
    assert!(matches!(config.output_format, OutputFormat::Markdown));

    assert_eq!(config.filters.include.len(), 3);
    assert!(config.filters.include.contains(&"src/**/*.rs".to_string()));
    assert!(config.filters.include.contains(&"lib/**/*.rs".to_string()));
    assert!(config
        .filters
        .include
        .contains(&"examples/**/*.rs".to_string()));

    assert_eq!(config.filters.exclude.len(), 3);
    assert!(config.filters.exclude.contains(&"target/**".to_string()));
    assert!(config.filters.exclude.contains(&"build/**".to_string()));
    assert!(config.filters.exclude.contains(&"*.tmp".to_string()));

    // Plugins should now be parsed correctly
    assert_eq!(config.plugins.len(), 3);
    assert!(config.plugins.contains(&"complexity-analyzer".to_string()));
    assert!(config.plugins.contains(&"doc-generator".to_string()));
    assert!(config.plugins.contains(&"dependency-graph".to_string()));
}

#[test]
fn test_plugins_parsing() {
    let simple_toml = r#"
        plugins = ["analyzer", "formatter"]
    "#;

    let config =
        ExtractorConfig::from_toml_str(simple_toml).expect("Failed to parse simple plugins TOML");

    println!("Simple plugins: {:?}", config.plugins);
    assert_eq!(config.plugins.len(), 2);
    assert!(config.plugins.contains(&"analyzer".to_string()));
    assert!(config.plugins.contains(&"formatter".to_string()));
}
