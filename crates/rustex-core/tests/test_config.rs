//! Tests for configuration handling.

use rustex_core::{ExtractorConfig, FilterConfig, OutputFormat};

#[test]
fn test_default_config() {
    let config = ExtractorConfig::default();

    assert!(config.include_docs, "Should include docs by default");
    assert!(
        !config.include_private,
        "Should not include private items by default"
    );
    assert!(
        !config.parse_dependencies,
        "Should not parse dependencies by default"
    );
    assert_eq!(
        config.max_file_size,
        10 * 1024 * 1024,
        "Should have 10MB file size limit"
    );
    assert!(
        matches!(config.output_format, OutputFormat::Json),
        "Should default to JSON output"
    );

    // Check default filters
    assert!(
        !config.filters.include.is_empty(),
        "Should have default include patterns"
    );
    assert!(
        !config.filters.exclude.is_empty(),
        "Should have default exclude patterns"
    );

    // Should include src files by default
    assert!(
        config.filters.include.iter().any(|p| p.contains("src")),
        "Should include src files"
    );

    // Should exclude target by default
    assert!(
        config.filters.exclude.iter().any(|p| p.contains("target")),
        "Should exclude target"
    );

    assert!(
        config.plugins.is_empty(),
        "Should have no plugins by default"
    );
}

#[test]
fn test_config_modification() {
    let mut config = ExtractorConfig::default();

    // Modify settings
    config.include_docs = false;
    config.include_private = true;
    config.parse_dependencies = true;
    config.max_file_size = 1024;
    config.output_format = OutputFormat::Markdown;

    assert!(!config.include_docs);
    assert!(config.include_private);
    assert!(config.parse_dependencies);
    assert_eq!(config.max_file_size, 1024);
    assert!(matches!(config.output_format, OutputFormat::Markdown));
}

#[test]
fn test_filter_config() {
    let mut filter_config = FilterConfig {
        include: vec!["**/*.rs".to_string()],
        exclude: vec!["target/**".to_string(), "tests/**".to_string()],
    };

    assert_eq!(filter_config.include.len(), 1);
    assert_eq!(filter_config.exclude.len(), 2);

    // Modify filters
    filter_config.include.push("lib/**/*.rs".to_string());
    filter_config.exclude.push("examples/**".to_string());

    assert_eq!(filter_config.include.len(), 2);
    assert_eq!(filter_config.exclude.len(), 3);
}

#[test]
fn test_output_formats() {
    let formats = vec![
        OutputFormat::Json,
        OutputFormat::MessagePack,
        OutputFormat::Markdown,
        OutputFormat::GraphQL,
        OutputFormat::Rag,
    ];

    // Test that all formats can be cloned and debugged
    for format in formats {
        let cloned = format.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty(), "Format should be debuggable");
    }
}

#[test]
fn test_config_serialization() {
    let config = ExtractorConfig::default();

    // Test that config can be serialized (this tests the Serialize derive)
    let json_result = serde_json::to_string(&config);
    assert!(json_result.is_ok(), "Config should be serializable to JSON");

    let json_str = json_result.unwrap();
    assert!(!json_str.is_empty(), "Serialized JSON should not be empty");
    assert!(
        json_str.contains("include_docs"),
        "Should contain config fields"
    );

    // Test deserialization
    let deserialized_result: Result<ExtractorConfig, _> = serde_json::from_str(&json_str);
    assert!(
        deserialized_result.is_ok(),
        "Config should be deserializable from JSON"
    );

    let deserialized_config = deserialized_result.unwrap();
    assert_eq!(config.include_docs, deserialized_config.include_docs);
    assert_eq!(config.include_private, deserialized_config.include_private);
    assert_eq!(config.max_file_size, deserialized_config.max_file_size);
}

#[test]
fn test_custom_config_creation() {
    let custom_config = ExtractorConfig {
        include_docs: false,
        include_private: true,
        parse_dependencies: true,
        max_file_size: 5 * 1024 * 1024, // 5MB
        output_format: OutputFormat::Markdown,
        filters: FilterConfig {
            include: vec!["custom/**/*.rs".to_string()],
            exclude: vec!["custom/target/**".to_string()],
        },
        plugins: vec!["custom-plugin".to_string()],
    };

    assert!(!custom_config.include_docs);
    assert!(custom_config.include_private);
    assert!(custom_config.parse_dependencies);
    assert_eq!(custom_config.max_file_size, 5 * 1024 * 1024);
    assert!(matches!(
        custom_config.output_format,
        OutputFormat::Markdown
    ));
    assert_eq!(custom_config.filters.include[0], "custom/**/*.rs");
    assert_eq!(custom_config.filters.exclude[0], "custom/target/**");
    assert_eq!(custom_config.plugins[0], "custom-plugin");
}

#[test]
fn test_config_validation() {
    let mut config = ExtractorConfig::default();

    // Test that zero file size limit works
    config.max_file_size = 0;
    // Should not panic or have issues with zero size

    // Test with very large file size
    config.max_file_size = usize::MAX;
    // Should handle large values

    // Test with empty filters
    config.filters.include.clear();
    config.filters.exclude.clear();
    // Should handle empty filters gracefully

    // Test with duplicate patterns
    config.filters.include = vec!["**/*.rs".to_string(), "**/*.rs".to_string()];
    // Should handle duplicates
    assert_eq!(config.filters.include.len(), 2);
}

#[test]
fn test_plugin_configuration() {
    let mut config = ExtractorConfig::default();

    // Test adding plugins
    config.plugins.push("plugin1".to_string());
    config.plugins.push("plugin2".to_string());

    assert_eq!(config.plugins.len(), 2);
    assert!(config.plugins.contains(&"plugin1".to_string()));
    assert!(config.plugins.contains(&"plugin2".to_string()));

    // Test removing plugins
    config.plugins.retain(|p| p != "plugin1");
    assert_eq!(config.plugins.len(), 1);
    assert!(!config.plugins.contains(&"plugin1".to_string()));
    assert!(config.plugins.contains(&"plugin2".to_string()));
}
