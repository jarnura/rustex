# RustEx API Reference

This document provides comprehensive API documentation for the RustEx library (`rustex-core`), enabling programmatic usage of the AST extraction functionality.

## Table of Contents

- [Getting Started](#getting-started)
- [Core Types](#core-types)
- [Main API](#main-api)
- [Configuration](#configuration)
- [AST Data Structures](#ast-data-structures)
- [Visitor Pattern](#visitor-pattern)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Getting Started

Add RustEx to your `Cargo.toml`:

```toml
[dependencies]
rustex-core = "0.1.0"
syn = { version = "2.0", features = ["full", "extra-traits", "visit"] }
```

Basic usage:

```rust
use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    println!("Extracted {} files", project_ast.files.len());
    Ok(())
}
```

## Core Types

### `AstExtractor`

The main entry point for AST extraction.

```rust
pub struct AstExtractor {
    config: ExtractorConfig,
    root_path: PathBuf,
}

impl AstExtractor {
    /// Create a new AST extractor with configuration and root path.
    pub fn new(config: ExtractorConfig, root_path: PathBuf) -> Self;
    
    /// Extract AST from the entire project.
    pub fn extract_project(&self) -> Result<ProjectAst, RustExError>;
    
    /// Extract AST from a single file.
    pub fn extract_file(&self, file_path: &Path) -> Result<FileAst, RustExError>;
    
    /// Discover all Rust files in the project.
    pub fn discover_rust_files(&self) -> Result<Vec<PathBuf>, RustExError>;
}
```

### `ExtractorConfig`

Configuration for AST extraction behavior.

```rust
pub struct ExtractorConfig {
    pub include_docs: bool,
    pub include_private: bool,
    pub parse_dependencies: bool,
    pub max_file_size: usize,
    pub output_format: OutputFormat,
    pub filters: FilterConfig,
    pub plugins: Vec<String>,
}

impl ExtractorConfig {
    /// Create default configuration.
    pub fn default() -> Self;
    
    /// Create configuration for specific use case.
    pub fn for_use_case(use_case: ConfigUseCase) -> Self;
    
    /// Load configuration from TOML file.
    pub fn from_toml_file(path: &Path) -> Result<Self, RustExError>;
    
    /// Save configuration to TOML file.
    pub fn to_toml_file(&self, path: &Path) -> Result<(), RustExError>;
    
    /// Validate configuration settings.
    pub fn validate(&self) -> Result<(), RustExError>;
}
```

## Main API

### Extracting from a Project

```rust
use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase};
use std::path::PathBuf;

// Create configuration for documentation generation
let config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);

// Create extractor
let extractor = AstExtractor::new(config, PathBuf::from("./my-project"));

// Extract the entire project
let project_ast = extractor.extract_project()?;

// Access project information
println!("Project: {}", project_ast.project.name);
println!("Files: {}", project_ast.files.len());
println!("Functions: {}", project_ast.metrics.total_functions);
```

### Extracting from Single Files

```rust
use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::{Path, PathBuf};

let config = ExtractorConfig::default();
let extractor = AstExtractor::new(config, PathBuf::from("."));

// Extract single file
let file_ast = extractor.extract_file(Path::new("src/lib.rs"))?;

// Access file elements
for element in &file_ast.elements {
    println!("Found {}: {}", element.element_type, element.name);
}
```

### Custom Configuration

```rust
use rustex_core::{ExtractorConfig, OutputFormat, FilterConfig};

let config = ExtractorConfig {
    include_docs: true,
    include_private: false,
    parse_dependencies: true,
    max_file_size: 5 * 1024 * 1024, // 5MB
    output_format: OutputFormat::Json,
    filters: FilterConfig {
        include: vec!["src/**/*.rs".to_string()],
        exclude: vec!["tests/**".to_string(), "benches/**".to_string()],
    },
    plugins: vec![],
};

let extractor = AstExtractor::new(config, PathBuf::from("."));
```

## Configuration

### Use Case Templates

```rust
use rustex_core::{ExtractorConfig, ConfigUseCase};

// Pre-configured for documentation generation
let doc_config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);

// Pre-configured for code analysis
let analysis_config = ExtractorConfig::for_use_case(ConfigUseCase::CodeAnalysis);

// Pre-configured for LLM training
let llm_config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);

// Pre-configured for testing
let test_config = ExtractorConfig::for_use_case(ConfigUseCase::Testing);
```

### File Filtering

```rust
use rustex_core::{FilterConfig, ExtractorConfig};

let filters = FilterConfig {
    include: vec![
        "src/**/*.rs".to_string(),
        "lib/**/*.rs".to_string(),
    ],
    exclude: vec![
        "target/**".to_string(),
        "tests/**".to_string(),
        "benches/**".to_string(),
    ],
};

let mut config = ExtractorConfig::default();
config.filters = filters;
```

## AST Data Structures

### `ProjectAst`

Root structure containing the entire project's AST.

```rust
pub struct ProjectAst {
    pub project: ProjectInfo,
    pub files: Vec<FileAst>,
    pub metrics: ProjectMetrics,
    pub cross_references: Vec<CrossReference>,
    pub extracted_at: DateTime<Utc>,
}
```

### `FileAst`

AST representation of a single Rust file.

```rust
pub struct FileAst {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub content_hash: String,
    pub file_metrics: FileMetrics,
    pub elements: Vec<CodeElement>,
    pub imports: Vec<ImportInfo>,
}
```

### `CodeElement`

Represents individual code elements (functions, structs, etc.).

```rust
pub struct CodeElement {
    pub id: String,
    pub element_type: ElementType,
    pub name: String,
    pub signature: Option<String>,
    pub visibility: Visibility,
    pub doc_comments: Vec<String>,
    pub inline_comments: Vec<String>,
    pub location: CodeLocation,
    pub attributes: Vec<String>,
    pub complexity: Option<u32>,
    pub complexity_metrics: Option<ComplexityMetrics>,
    pub dependencies: Vec<String>,
    pub generic_params: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub hierarchy: ElementHierarchy,
}
```

### `ElementType`

Enumeration of supported code elements.

```rust
pub enum ElementType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Macro,
    Const,
    Static,
    Type,
    Use,
}
```

### `Visibility`

Code element visibility levels.

```rust
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
    Module(String),
}
```

### `ComplexityMetrics`

Detailed complexity analysis for code elements.

```rust
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub halstead: HalsteadMetrics,
    pub nesting_depth: u32,
    pub lines_of_code: u32,
    pub parameter_count: u32,
    pub return_count: u32,
}

pub struct HalsteadMetrics {
    pub n1: u32,        // Number of distinct operators
    pub n2: u32,        // Number of distinct operands  
    pub big_n1: u32,    // Total number of operators
    pub big_n2: u32,    // Total number of operands
}
```

### `ElementHierarchy`

Hierarchical relationships between code elements.

```rust
pub struct ElementHierarchy {
    pub module_path: String,
    pub qualified_name: String,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub nesting_level: u32,
    pub namespace: ElementNamespace,
}

pub struct ElementNamespace {
    pub simple_name: String,
    pub canonical_path: String,
    pub aliases: Vec<String>,
    pub import_paths: Vec<String>,
    pub is_public: bool,
    pub visibility_scope: VisibilityScope,
}
```

## Visitor Pattern

### `CodeElementVisitor`

Customize AST traversal and element extraction.

```rust
use rustex_core::{CodeElementVisitor, ExtractorConfig};
use std::path::PathBuf;
use syn::visit::Visit;

let config = ExtractorConfig::default();
let file_path = PathBuf::from("src/lib.rs");
let mut visitor = CodeElementVisitor::new(file_path, &config);

// Parse Rust code
let code = std::fs::read_to_string("src/lib.rs")?;
let syntax_tree = syn::parse_file(&code)?;

// Visit the AST
visitor.visit_file(&syntax_tree);

// Extract elements
let elements = visitor.into_elements();
```

### Custom Visitors

Implement custom analysis by extending the visitor:

```rust
use syn::visit::Visit;
use syn::{ItemFn, ItemStruct};

struct CustomVisitor {
    functions: Vec<String>,
    structs: Vec<String>,
}

impl<'ast> Visit<'ast> for CustomVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.functions.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
    }
    
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.structs.push(node.ident.to_string());
        syn::visit::visit_item_struct(self, node);
    }
}
```

## Error Handling

### `RustExError`

Main error type for RustEx operations.

```rust
pub enum RustExError {
    Io(std::io::Error),
    ParseError(syn::Error),
    ConfigError(String),
    InvalidProjectRoot { path: PathBuf },
    PartialFailure {
        successful_count: usize,
        failed_count: usize,
        total_count: usize,
        errors: Vec<String>,
    },
    MaxFileSizeExceeded {
        file_path: PathBuf,
        file_size: u64,
        max_size: u64,
    },
}
```

### Error Handling Patterns

```rust
use rustex_core::{AstExtractor, RustExError};

match extractor.extract_project() {
    Ok(project_ast) => {
        println!("Successfully extracted {} files", project_ast.files.len());
    }
    Err(RustExError::PartialFailure { successful_count, failed_count, .. }) => {
        println!("Partial success: {}/{} files processed", 
                successful_count, successful_count + failed_count);
    }
    Err(RustExError::InvalidProjectRoot { path }) => {
        eprintln!("Invalid project root: {}", path.display());
    }
    Err(e) => {
        eprintln!("Extraction failed: {}", e);
    }
}
```

## Examples

### Basic Project Analysis

```rust
use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::PathBuf;

fn analyze_project() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig {
        include_docs: true,
        include_private: true,
        ..Default::default()
    };
    
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    // Print summary
    println!("Project: {}", project_ast.project.name);
    println!("Total functions: {}", project_ast.metrics.total_functions);
    println!("Average complexity: {:.2}", project_ast.metrics.complexity_average);
    
    // Find most complex functions
    let mut complex_functions = Vec::new();
    for file in &project_ast.files {
        for element in &file.elements {
            if let Some(complexity) = element.complexity {
                if complexity > 10 {
                    complex_functions.push((element.name.clone(), complexity));
                }
            }
        }
    }
    
    complex_functions.sort_by_key(|(_, complexity)| *complexity);
    println!("Most complex functions:");
    for (name, complexity) in complex_functions.iter().rev().take(5) {
        println!("  {} (complexity: {})", name, complexity);
    }
    
    Ok(())
}
```

### Custom Output Processing

```rust
use rustex_core::{AstExtractor, ExtractorConfig, ElementType};
use serde_json;

fn generate_api_documentation() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig {
        include_docs: true,
        include_private: false,
        ..Default::default()
    };
    
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    let mut api_functions = Vec::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            if element.element_type == ElementType::Function 
               && element.visibility == rustex_core::Visibility::Public {
                api_functions.push(serde_json::json!({
                    "name": element.name,
                    "signature": element.signature,
                    "documentation": element.doc_comments.join("\n"),
                    "file": file.relative_path,
                    "line": element.location.line_start
                }));
            }
        }
    }
    
    let api_doc = serde_json::json!({
        "project": project_ast.project.name,
        "version": project_ast.project.version,
        "api_functions": api_functions
    });
    
    std::fs::write("api-doc.json", serde_json::to_string_pretty(&api_doc)?)?;
    Ok(())
}
```

### Working with Hierarchical Data

```rust
use rustex_core::{AstExtractor, ExtractorConfig, ElementType};

fn print_module_structure() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    // Group elements by module
    let mut modules: std::collections::HashMap<String, Vec<&rustex_core::CodeElement>> = 
        std::collections::HashMap::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            let module_path = &element.hierarchy.module_path;
            modules.entry(module_path.clone())
                   .or_insert_with(Vec::new)
                   .push(element);
        }
    }
    
    // Print hierarchical structure
    for (module_path, elements) in modules {
        println!("Module: {}", module_path);
        
        for element in elements {
            let indent = "  ".repeat(element.hierarchy.nesting_level as usize);
            println!("{}├─ {:?}: {}", indent, element.element_type, element.name);
            
            // Print children
            for child_id in &element.hierarchy.children_ids {
                if let Some(child) = find_element_by_id(&project_ast, child_id) {
                    let child_indent = "  ".repeat((element.hierarchy.nesting_level + 1) as usize);
                    println!("{}├─ {:?}: {}", child_indent, child.element_type, child.name);
                }
            }
        }
        println!();
    }
    
    Ok(())
}

fn find_element_by_id<'a>(
    project_ast: &'a rustex_core::ProjectAst, 
    id: &str
) -> Option<&'a rustex_core::CodeElement> {
    for file in &project_ast.files {
        for element in &file.elements {
            if element.id == id {
                return Some(element);
            }
        }
    }
    None
}
```

### Configuration from Code

```rust
use rustex_core::{ExtractorConfig, FilterConfig, OutputFormat};

fn create_custom_config() -> ExtractorConfig {
    ExtractorConfig {
        include_docs: true,
        include_private: false,
        parse_dependencies: true,
        max_file_size: 10 * 1024 * 1024, // 10MB
        output_format: OutputFormat::Json,
        filters: FilterConfig {
            include: vec![
                "src/**/*.rs".to_string(),
                "lib/**/*.rs".to_string(),
            ],
            exclude: vec![
                "tests/**".to_string(),
                "benches/**".to_string(),
                "examples/**".to_string(),
            ],
        },
        plugins: vec![
            "complexity".to_string(),
            "documentation".to_string(),
        ],
    }
}
```

## Advanced Usage

### Performance Considerations

For large projects, consider these optimizations:

```rust
use rustex_core::{ExtractorConfig, FilterConfig};

// Process only essential files
let config = ExtractorConfig {
    filters: FilterConfig {
        include: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
        exclude: vec!["target/**".to_string()],
    },
    max_file_size: 1024 * 1024, // 1MB limit
    parse_dependencies: false,   // Skip dependency parsing
    ..Default::default()
};
```

### Integration with Build Systems

```rust
// In build.rs
use rustex_core::{AstExtractor, ExtractorConfig};

fn main() {
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, std::env::current_dir().unwrap());
    
    if let Ok(project_ast) = extractor.extract_project() {
        // Generate code documentation during build
        let docs = generate_docs(&project_ast);
        std::fs::write("target/generated-docs.md", docs).unwrap();
    }
}
```

## Thread Safety

RustEx is designed to be thread-safe:

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(ExtractorConfig::default());
let handles: Vec<_> = (0..4).map(|i| {
    let config = Arc::clone(&config);
    thread::spawn(move || {
        let extractor = AstExtractor::new((*config).clone(), 
                                         PathBuf::from(format!("project-{}", i)));
        extractor.extract_project()
    })
}).collect();

for handle in handles {
    match handle.join().unwrap() {
        Ok(project_ast) => println!("Extracted project with {} files", project_ast.files.len()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

For more examples and advanced usage patterns, see the [examples directory](../examples/) in the repository.