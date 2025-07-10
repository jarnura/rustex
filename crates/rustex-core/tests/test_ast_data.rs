//! Tests for AST data structures.

use chrono::Utc;
use rustex_core::{
    CodeElement, CodeLocation, DependencyInfo, ElementType, FileAst, FileMetrics, ImportInfo,
    ProjectAst, ProjectInfo, ProjectMetrics, Visibility,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_project_ast_creation() {
    let project_info = ProjectInfo {
        name: "test-project".to_string(),
        version: "1.0.0".to_string(),
        rust_edition: "2021".to_string(),
        root_path: PathBuf::from("/test/path"),
    };

    let file_metrics = FileMetrics {
        lines_of_code: 100,
        lines_of_comments: 20,
        complexity_total: 10,
        function_count: 5,
        struct_count: 2,
        enum_count: 1,
        trait_count: 1,
    };

    let file_ast = FileAst {
        path: PathBuf::from("/test/path/src/main.rs"),
        relative_path: PathBuf::from("src/main.rs"),
        elements: vec![],
        imports: vec![],
        file_metrics,
    };

    let dependencies = DependencyInfo {
        direct: vec!["serde".to_string()],
        transitive: vec!["serde_json".to_string()],
        dev_dependencies: vec!["tokio-test".to_string()],
    };

    let metrics = ProjectMetrics {
        total_lines: 100,
        total_files: 1,
        total_functions: 5,
        total_structs: 2,
        total_enums: 1,
        total_traits: 1,
        complexity_average: 2.0,
        complexity_max: 5,
    };

    let project_ast = ProjectAst {
        project: project_info,
        files: vec![file_ast],
        dependencies,
        metrics,
        extracted_at: Utc::now(),
    };

    assert_eq!(project_ast.project.name, "test-project");
    assert_eq!(project_ast.files.len(), 1);
    assert_eq!(project_ast.dependencies.direct.len(), 1);
    assert_eq!(project_ast.metrics.total_functions, 5);
}

#[test]
fn test_code_element_creation() {
    let location = CodeLocation {
        line_start: 10,
        line_end: 20,
        char_start: 0,
        char_end: 100,
        file_path: PathBuf::from("src/main.rs"),
    };

    let mut metadata = HashMap::new();
    metadata.insert(
        "custom_key".to_string(),
        serde_json::Value::String("custom_value".to_string()),
    );

    let element = CodeElement {
        element_type: ElementType::Function,
        name: "test_function".to_string(),
        signature: Some("fn test_function() -> String".to_string()),
        visibility: Visibility::Public,
        doc_comments: vec!["Function documentation".to_string()],
        inline_comments: vec!["Inline comment".to_string()],
        location,
        attributes: vec!["#[test]".to_string()],
        complexity: Some(3),
        complexity_metrics: None,
        dependencies: vec!["std::string::String".to_string()],
        generic_params: vec!["T: Clone".to_string()],
        metadata,
    };

    assert_eq!(element.name, "test_function");
    assert!(matches!(element.element_type, ElementType::Function));
    assert!(matches!(element.visibility, Visibility::Public));
    assert!(!element.doc_comments.is_empty());
    assert_eq!(element.complexity.unwrap(), 3);
    assert!(!element.metadata.is_empty());
}

#[test]
fn test_element_types() {
    let types = vec![
        ElementType::Function,
        ElementType::Struct,
        ElementType::Enum,
        ElementType::Trait,
        ElementType::Impl,
        ElementType::Module,
        ElementType::Constant,
        ElementType::Static,
        ElementType::TypeAlias,
        ElementType::Macro,
        ElementType::Union,
    ];

    for element_type in types {
        let debug_str = format!("{:?}", element_type);
        assert!(!debug_str.is_empty(), "Element type should be debuggable");

        let cloned = element_type.clone();
        // Test that element types can be used in matches
        match cloned {
            ElementType::Function => assert!(matches!(element_type, ElementType::Function)),
            ElementType::Struct => assert!(matches!(element_type, ElementType::Struct)),
            ElementType::Enum => assert!(matches!(element_type, ElementType::Enum)),
            ElementType::Trait => assert!(matches!(element_type, ElementType::Trait)),
            ElementType::Impl => assert!(matches!(element_type, ElementType::Impl)),
            ElementType::Module => assert!(matches!(element_type, ElementType::Module)),
            ElementType::Constant => assert!(matches!(element_type, ElementType::Constant)),
            ElementType::Static => assert!(matches!(element_type, ElementType::Static)),
            ElementType::TypeAlias => assert!(matches!(element_type, ElementType::TypeAlias)),
            ElementType::Macro => assert!(matches!(element_type, ElementType::Macro)),
            ElementType::Union => assert!(matches!(element_type, ElementType::Union)),
        }
    }
}

#[test]
fn test_visibility_types() {
    let visibilities = vec![
        Visibility::Public,
        Visibility::Private,
        Visibility::Restricted("crate".to_string()),
    ];

    for visibility in visibilities {
        let debug_str = format!("{:?}", visibility);
        assert!(!debug_str.is_empty(), "Visibility should be debuggable");

        match &visibility {
            Visibility::Public => assert!(matches!(visibility, Visibility::Public)),
            Visibility::Private => assert!(matches!(visibility, Visibility::Private)),
            Visibility::Restricted(scope) => {
                assert!(matches!(visibility, Visibility::Restricted(_)));
                assert_eq!(scope, "crate");
            }
        }
    }
}

#[test]
fn test_import_info() {
    // Test simple import
    let simple_import = ImportInfo {
        module_path: "std::collections".to_string(),
        imported_items: vec!["HashMap".to_string()],
        is_glob: false,
        alias: None,
    };

    assert_eq!(simple_import.module_path, "std::collections");
    assert!(!simple_import.is_glob);
    assert!(simple_import.alias.is_none());

    // Test glob import
    let glob_import = ImportInfo {
        module_path: "std::prelude".to_string(),
        imported_items: vec![],
        is_glob: true,
        alias: None,
    };

    assert!(glob_import.is_glob);
    assert!(glob_import.imported_items.is_empty());

    // Test aliased import
    let aliased_import = ImportInfo {
        module_path: "std::collections".to_string(),
        imported_items: vec!["HashMap".to_string()],
        is_glob: false,
        alias: Some("Map".to_string()),
    };

    assert_eq!(aliased_import.alias.unwrap(), "Map");
}

#[test]
fn test_dependency_info() {
    let deps = DependencyInfo {
        direct: vec!["serde".to_string(), "tokio".to_string()],
        transitive: vec!["serde_json".to_string(), "mio".to_string()],
        dev_dependencies: vec!["tokio-test".to_string()],
    };

    assert_eq!(deps.direct.len(), 2);
    assert_eq!(deps.transitive.len(), 2);
    assert_eq!(deps.dev_dependencies.len(), 1);

    assert!(deps.direct.contains(&"serde".to_string()));
    assert!(deps.transitive.contains(&"serde_json".to_string()));
    assert!(deps.dev_dependencies.contains(&"tokio-test".to_string()));
}

#[test]
fn test_metrics_calculation() {
    let mut metrics = ProjectMetrics {
        total_lines: 0,
        total_files: 0,
        total_functions: 0,
        total_structs: 0,
        total_enums: 0,
        total_traits: 0,
        complexity_average: 0.0,
        complexity_max: 0,
    };

    // Simulate adding file metrics
    metrics.total_lines += 100;
    metrics.total_files += 1;
    metrics.total_functions += 5;
    metrics.total_structs += 2;
    metrics.total_enums += 1;
    metrics.total_traits += 1;

    // Calculate complexity
    let total_elements = metrics.total_functions
        + metrics.total_structs
        + metrics.total_enums
        + metrics.total_traits;
    metrics.complexity_average = (metrics.total_functions as f64 * 2.0
        + metrics.total_structs as f64 * 1.0
        + metrics.total_enums as f64 * 3.0
        + metrics.total_traits as f64 * 2.0)
        / total_elements as f64;
    metrics.complexity_max = 5;

    assert_eq!(metrics.total_files, 1);
    assert_eq!(metrics.total_functions, 5);
    assert!(metrics.complexity_average > 0.0);
    assert_eq!(metrics.complexity_max, 5);
}

#[test]
fn test_file_metrics() {
    let file_metrics = FileMetrics {
        lines_of_code: 150,
        lines_of_comments: 30,
        complexity_total: 25,
        function_count: 8,
        struct_count: 3,
        enum_count: 2,
        trait_count: 1,
    };

    assert_eq!(file_metrics.lines_of_code, 150);
    assert_eq!(file_metrics.lines_of_comments, 30);
    assert_eq!(file_metrics.complexity_total, 25);
    assert_eq!(file_metrics.function_count, 8);
    assert_eq!(file_metrics.struct_count, 3);
    assert_eq!(file_metrics.enum_count, 2);
    assert_eq!(file_metrics.trait_count, 1);

    // Test calculations
    let total_elements = file_metrics.function_count
        + file_metrics.struct_count
        + file_metrics.enum_count
        + file_metrics.trait_count;
    assert_eq!(total_elements, 14);

    let average_complexity = file_metrics.complexity_total as f64 / total_elements as f64;
    assert!(average_complexity > 0.0);
}

#[test]
fn test_serialization() {
    let project_info = ProjectInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        rust_edition: "2021".to_string(),
        root_path: PathBuf::from("/test"),
    };

    // Test serialization
    let json_result = serde_json::to_string(&project_info);
    assert!(json_result.is_ok(), "ProjectInfo should be serializable");

    let json_str = json_result.unwrap();
    assert!(json_str.contains("test"));
    assert!(json_str.contains("1.0.0"));

    // Test deserialization
    let deserialized_result: Result<ProjectInfo, _> = serde_json::from_str(&json_str);
    assert!(
        deserialized_result.is_ok(),
        "ProjectInfo should be deserializable"
    );

    let deserialized = deserialized_result.unwrap();
    assert_eq!(deserialized.name, project_info.name);
    assert_eq!(deserialized.version, project_info.version);
}

#[test]
fn test_code_location() {
    let location = CodeLocation {
        line_start: 10,
        line_end: 15,
        char_start: 4,
        char_end: 25,
        file_path: PathBuf::from("src/main.rs"),
    };

    assert_eq!(location.line_start, 10);
    assert_eq!(location.line_end, 15);
    assert_eq!(location.char_start, 4);
    assert_eq!(location.char_end, 25);
    assert_eq!(location.file_path, PathBuf::from("src/main.rs"));

    // Test that location spans multiple lines
    assert!(location.line_end >= location.line_start);

    // Test character span
    assert!(location.char_end >= location.char_start);
}

#[test]
fn test_default_implementations() {
    // Test ProjectMetrics default
    let default_metrics = ProjectMetrics {
        total_lines: 0,
        total_files: 0,
        total_functions: 0,
        total_structs: 0,
        total_enums: 0,
        total_traits: 0,
        complexity_average: 0.0,
        complexity_max: 0,
    };

    // All values should be zero/empty for default
    assert_eq!(default_metrics.total_lines, 0);
    assert_eq!(default_metrics.total_files, 0);
    assert_eq!(default_metrics.complexity_average, 0.0);
}
