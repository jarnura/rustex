# RustEx Examples

This directory contains comprehensive examples demonstrating the various capabilities of RustEx. Each example showcases different use cases, output formats, and configuration options.

## Quick Start

Run any example from the project root:

```bash
cargo run --example <example_name>
```

All examples are self-contained and will generate output files for inspection.

## Working Examples Overview

| Example | Description | Generates | Purpose |
|---------|-------------|-----------|----------|
| [`basic_usage`](#basic-usage) | AST extraction fundamentals | `basic-ast-output.json` | Learning core functionality |
| [`documentation_generator`](#documentation-generator) | Project documentation | `PROJECT_DOCUMENTATION.md` | API documentation |
| [`code_analyzer`](#code-analyzer) | Quality & complexity analysis | `code-analysis-report.md` | Code quality assessment |
| [`llm_data_prep`](#llm-data-prep) | LLM training data | Multiple JSON files | AI/ML workflows |
| [`rag_output_demo`](#rag-output-demo) | RAG-optimized formats | Multiple RAG files | Advanced AI applications |

## Basic Usage

### CLI Example

```bash
# Extract AST from current project
rustex extract --pretty

# Save to file with documentation
rustex extract --include-docs --output project-ast.json --pretty

# Generate readable documentation
rustex extract --format markdown --include-docs --output docs.md
```

### Library Example

```rust
// examples/basic_usage.rs
use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create default configuration
    let config = ExtractorConfig::default();
    
    // Create extractor for current directory
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    // Extract project AST
    let project_ast = extractor.extract_project()?;
    
    // Print basic information
    println!("Project: {}", project_ast.project.name);
    println!("Files: {}", project_ast.files.len());
    println!("Functions: {}", project_ast.metrics.total_functions);
    println!("Structs: {}", project_ast.metrics.total_structs);
    
    Ok(())
}
```

## Documentation Generation

### CLI Example

```bash
# Generate comprehensive documentation
rustex extract \
    --format markdown \
    --include-docs \
    --include-private \
    --output DOCUMENTATION.md

# Generate public API documentation only
rustex extract \
    --format markdown \
    --include-docs \
    --exclude "src/internal/**" \
    --output PUBLIC_API.md
```

### Configuration File

```toml
# examples/configs/documentation.toml
[extraction]
include_docs = true
include_private = true
output_format = "markdown"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["tests/**", "benches/**"]

[output.markdown]
include_toc = true
toc_depth = 3
syntax_highlighting = true
include_summary = true

[plugins]
enabled = ["doc-enhancer"]
```

### Library Example

```rust
// examples/documentation_generator.rs
use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase, ElementType, Visibility};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use documentation template
    let config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    // Generate custom documentation
    let mut doc = String::new();
    doc.push_str(&format!("# {} Documentation\n\n", project_ast.project.name));
    
    // Add project overview
    doc.push_str("## Overview\n\n");
    doc.push_str(&format!("Version: {}\n", project_ast.project.version));
    doc.push_str(&format!("Rust Edition: {}\n\n", project_ast.project.rust_edition));
    
    // Add metrics
    doc.push_str("## Project Metrics\n\n");
    doc.push_str(&format!("- **Files**: {}\n", project_ast.metrics.total_files));
    doc.push_str(&format!("- **Functions**: {}\n", project_ast.metrics.total_functions));
    doc.push_str(&format!("- **Structs**: {}\n", project_ast.metrics.total_structs));
    doc.push_str(&format!("- **Average Complexity**: {:.2}\n\n", project_ast.metrics.complexity_average));
    
    // Add public API
    doc.push_str("## Public API\n\n");
    for file in &project_ast.files {
        let public_items: Vec<_> = file.elements.iter()
            .filter(|e| e.visibility == Visibility::Public)
            .collect();
            
        if !public_items.is_empty() {
            doc.push_str(&format!("### {}\n\n", file.relative_path.display()));
            
            for element in public_items {
                doc.push_str(&format!("#### {} `{}`\n\n", 
                    format!("{:?}", element.element_type), element.name));
                
                if !element.doc_comments.is_empty() {
                    doc.push_str("**Documentation:**\n");
                    for comment in &element.doc_comments {
                        doc.push_str(&format!("> {}\n", comment));
                    }
                    doc.push('\n');
                }
                
                if let Some(signature) = &element.signature {
                    doc.push_str(&format!("```rust\n{}\n```\n\n", signature));
                }
            }
        }
    }
    
    // Save documentation
    std::fs::write("generated-docs.md", doc)?;
    println!("Documentation generated: generated-docs.md");
    
    Ok(())
}
```

## Code Analysis

### CLI Example

```bash
# Comprehensive code analysis
rustex extract \
    --format json \
    --include-private \
    --parse-deps \
    --plugins "complexity-analyzer" \
    --output analysis.json

# Generate metrics report
rustex metrics \
    --complexity \
    --loc \
    --output metrics.json
```

### Library Example

```rust
// examples/code_analyzer.rs
use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase};
use std::path::PathBuf;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig::for_use_case(ConfigUseCase::CodeAnalysis);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    // Analyze complexity distribution
    let mut complexity_distribution: HashMap<u32, u32> = HashMap::new();
    let mut high_complexity_functions = Vec::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            if let Some(complexity) = element.complexity {
                *complexity_distribution.entry(complexity).or_insert(0) += 1;
                
                if complexity > 10 {
                    high_complexity_functions.push((
                        element.name.clone(),
                        complexity,
                        file.relative_path.clone(),
                    ));
                }
            }
        }
    }
    
    // Sort by complexity
    high_complexity_functions.sort_by_key(|(_, complexity, _)| *complexity);
    
    // Generate report
    println!("🔍 Code Analysis Report");
    println!("═══════════════════════");
    
    println!("\n📊 Project Overview:");
    println!("  • Total files: {}", project_ast.metrics.total_files);
    println!("  • Total functions: {}", project_ast.metrics.total_functions);
    println!("  • Average complexity: {:.2}", project_ast.metrics.complexity_average);
    
    println!("\n📈 Complexity Distribution:");
    for (complexity, count) in complexity_distribution.iter() {
        let bar = "█".repeat((*count as usize).min(50));
        println!("  Complexity {}: {} {}", complexity, count, bar);
    }
    
    println!("\n⚠️  High Complexity Functions (>10):");
    for (name, complexity, file) in high_complexity_functions.iter().rev().take(10) {
        println!("  • {} (complexity: {}) in {}", name, complexity, file.display());
    }
    
    // Find files with most functions
    let mut file_function_counts: Vec<_> = project_ast.files.iter()
        .map(|f| (f.relative_path.clone(), f.file_metrics.function_count))
        .collect();
    file_function_counts.sort_by_key(|(_, count)| *count);
    
    println!("\n📁 Files with Most Functions:");
    for (file, count) in file_function_counts.iter().rev().take(5) {
        println!("  • {} ({} functions)", file.display(), count);
    }
    
    Ok(())
}
```

## LLM Data Preparation

### CLI Example

```bash
# Generate LLM training data
rustex extract \
    --format rag \
    --include-docs \
    --plugins "llm-optimizer" \
    --exclude "tests/**,benches/**" \
    --output llm-training-data.json

# Generate chunked data for fine-tuning
rustex extract \
    --format rag \
    --include-docs \
    --max-file-size 5MB \
    --output fine-tuning-chunks.json
```

### Configuration

```toml
# examples/configs/llm-training.toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = true
output_format = "rag"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs", "examples/**/*.rs"]
exclude = ["target/**", "tests/**", "benches/**"]

[output.rag]
max_chunk_size = 1000
chunk_overlap = 100
include_semantics = true
include_context = true

[plugins]
enabled = ["llm-optimizer"]

[plugins.llm-optimizer]
target_model = "gpt-4"
max_context_length = 8000
include_training_hints = true
```

### Library Example

```rust
// examples/llm_data_prep.rs
use rustex_core::{AstExtractor, ExtractorConfig, ConfigUseCase, ElementType};
use std::path::PathBuf;
use serde_json;

#[derive(serde::Serialize)]
struct LlmTrainingExample {
    input: String,
    output: String,
    metadata: serde_json::Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    let mut training_examples = Vec::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            if element.element_type == ElementType::Function 
               && !element.doc_comments.is_empty() {
                
                // Create function documentation example
                let input = format!(
                    "Generate documentation for this Rust function:\n\n```rust\n{}\n```",
                    element.signature.as_ref().unwrap_or(&element.name)
                );
                
                let output = element.doc_comments.join("\n");
                
                let example = LlmTrainingExample {
                    input,
                    output,
                    metadata: serde_json::json!({
                        "function_name": element.name,
                        "file": file.relative_path,
                        "complexity": element.complexity,
                        "element_type": "function",
                        "visibility": element.visibility
                    }),
                };
                
                training_examples.push(example);
            }
            
            if element.element_type == ElementType::Struct 
               && !element.doc_comments.is_empty() {
                
                // Create struct documentation example
                let input = format!(
                    "Explain this Rust struct:\n\n```rust\n{}\n```",
                    element.signature.as_ref().unwrap_or(&element.name)
                );
                
                let output = element.doc_comments.join("\n");
                
                let example = LlmTrainingExample {
                    input,
                    output,
                    metadata: serde_json::json!({
                        "struct_name": element.name,
                        "file": file.relative_path,
                        "element_type": "struct",
                        "visibility": element.visibility
                    }),
                };
                
                training_examples.push(example);
            }
        }
    }
    
    // Save training data
    let output = serde_json::to_string_pretty(&training_examples)?;
    std::fs::write("llm-training-data.jsonl", output)?;
    
    println!("Generated {} training examples", training_examples.len());
    println!("Training data saved to: llm-training-data.jsonl");
    
    Ok(())
}
```

## API Documentation

### Library Example

```rust
// examples/api_extractor.rs
use rustex_core::{AstExtractor, ExtractorConfig, ElementType, Visibility};
use std::path::PathBuf;
use serde_json;

#[derive(serde::Serialize)]
struct ApiEndpoint {
    name: String,
    signature: String,
    documentation: String,
    file: String,
    line: u32,
    parameters: Vec<String>,
    return_type: Option<String>,
    examples: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ExtractorConfig::default();
    config.include_docs = true;
    config.include_private = false;
    
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    let mut api_endpoints = Vec::new();
    
    for file in &project_ast.files {
        for element in &file.elements {
            if element.element_type == ElementType::Function 
               && element.visibility == Visibility::Public {
                
                let endpoint = ApiEndpoint {
                    name: element.name.clone(),
                    signature: element.signature.clone().unwrap_or_default(),
                    documentation: element.doc_comments.join("\n"),
                    file: file.relative_path.to_string_lossy().to_string(),
                    line: element.location.line_start,
                    parameters: element.generic_params.clone(),
                    return_type: extract_return_type(&element.signature),
                    examples: extract_examples(&element.doc_comments),
                };
                
                api_endpoints.push(endpoint);
            }
        }
    }
    
    // Generate OpenAPI-style documentation
    let api_doc = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": project_ast.project.name,
            "version": project_ast.project.version,
            "description": format!("API documentation for {}", project_ast.project.name)
        },
        "paths": api_endpoints.iter().map(|endpoint| {
            (endpoint.name.clone(), serde_json::json!({
                "get": {
                    "summary": endpoint.documentation.lines().next().unwrap_or(&endpoint.name),
                    "description": endpoint.documentation,
                    "responses": {
                        "200": {
                            "description": "Success"
                        }
                    }
                }
            }))
        }).collect::<serde_json::Map<_, _>>()
    });
    
    std::fs::write("api-documentation.json", serde_json::to_string_pretty(&api_doc)?)?;
    println!("API documentation generated: api-documentation.json");
    
    Ok(())
}

fn extract_return_type(signature: &Option<String>) -> Option<String> {
    signature.as_ref().and_then(|sig| {
        if let Some(arrow_pos) = sig.find("->") {
            Some(sig[arrow_pos + 2..].trim().to_string())
        } else {
            None
        }
    })
}

fn extract_examples(doc_comments: &[String]) -> Vec<String> {
    let mut examples = Vec::new();
    let mut in_example = false;
    let mut current_example = String::new();
    
    for comment in doc_comments {
        if comment.contains("# Example") || comment.contains("```") {
            if in_example && !current_example.is_empty() {
                examples.push(current_example.trim().to_string());
                current_example.clear();
            }
            in_example = !in_example;
        } else if in_example {
            current_example.push_str(comment);
            current_example.push('\n');
        }
    }
    
    if !current_example.is_empty() {
        examples.push(current_example.trim().to_string());
    }
    
    examples
}
```

## Integration Patterns

### Build Script Integration

```rust
// examples/build_integration.rs (for build.rs)
use rustex_core::{AstExtractor, ExtractorConfig};
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    
    match extractor.extract_project() {
        Ok(project_ast) => {
            // Generate code metrics for build output
            let metrics = serde_json::json!({
                "functions": project_ast.metrics.total_functions,
                "structs": project_ast.metrics.total_structs,
                "complexity": project_ast.metrics.complexity_average,
                "files": project_ast.metrics.total_files
            });
            
            std::fs::write("target/build-metrics.json", metrics.to_string()).ok();
            
            println!("cargo:warning=Project metrics: {} functions, {} structs, {:.1} avg complexity",
                    project_ast.metrics.total_functions,
                    project_ast.metrics.total_structs, 
                    project_ast.metrics.complexity_average);
        }
        Err(e) => {
            println!("cargo:warning=Failed to extract AST: {}", e);
        }
    }
}
```

### CI/CD Integration

```bash
#!/bin/bash
# examples/ci_integration.sh

# Extract and analyze code
echo "📊 Running RustEx analysis..."
rustex extract \
    --format json \
    --include-private \
    --plugins "complexity-analyzer" \
    --output ci-analysis.json

# Check for high complexity
COMPLEX_FUNCTIONS=$(jq '[.files[].elements[] | select(.complexity > 15)] | length' ci-analysis.json)

if [ "$COMPLEX_FUNCTIONS" -gt 0 ]; then
    echo "⚠️  Warning: Found $COMPLEX_FUNCTIONS functions with complexity > 15"
    jq -r '.files[].elements[] | select(.complexity > 15) | "  • \(.name) (complexity: \(.complexity))"' ci-analysis.json
fi

# Generate documentation
echo "📚 Generating documentation..."
rustex extract \
    --format markdown \
    --include-docs \
    --output docs/API.md

echo "✅ RustEx analysis complete"
```

### GitHub Actions Workflow

```yaml
# examples/.github/workflows/rustex.yml
name: RustEx Analysis

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  analyze:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install RustEx
      run: cargo install rustex-cli
    
    - name: Run RustEx Analysis
      run: |
        rustex extract \
          --format json \
          --include-private \
          --plugins "complexity-analyzer" \
          --output analysis.json
    
    - name: Check Code Quality
      run: |
        # Fail if average complexity is too high
        COMPLEXITY=$(jq '.metrics.complexity_average' analysis.json)
        if (( $(echo "$COMPLEXITY > 10.0" | bc -l) )); then
          echo "❌ Average complexity too high: $COMPLEXITY"
          exit 1
        fi
        
        # Warn about high-complexity functions
        jq -r '.files[].elements[] | select(.complexity > 15) | "Warning: \(.name) has complexity \(.complexity)"' analysis.json
    
    - name: Generate Documentation
      run: |
        rustex extract \
          --format markdown \
          --include-docs \
          --output docs/API.md
    
    - name: Upload Analysis Results
      uses: actions/upload-artifact@v3
      with:
        name: rustex-analysis
        path: |
          analysis.json
          docs/API.md
```

## Custom Analysis

### Advanced Library Usage

```rust
// examples/custom_analyzer.rs
use rustex_core::{AstExtractor, ExtractorConfig, CodeElementVisitor};
use std::path::PathBuf;
use std::collections::HashMap;
use syn::visit::Visit;

#[derive(Default)]
struct SecurityAnalyzer {
    unsafe_blocks: u32,
    external_calls: Vec<String>,
    potential_issues: Vec<String>,
}

impl<'ast> Visit<'ast> for SecurityAnalyzer {
    fn visit_expr_unsafe(&mut self, _node: &'ast syn::ExprUnsafe) {
        self.unsafe_blocks += 1;
        self.potential_issues.push("Unsafe block detected".to_string());
    }
    
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            let path_str = quote::quote!(#path).to_string();
            if path_str.contains("std::process") || path_str.contains("std::fs") {
                self.external_calls.push(path_str);
                self.potential_issues.push(format!("External system call: {}", path_str));
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, PathBuf::from("."));
    let project_ast = extractor.extract_project()?;
    
    let mut security_analyzer = SecurityAnalyzer::default();
    
    // Analyze each file for security issues
    for file in &project_ast.files {
        let content = std::fs::read_to_string(&file.path)?;
        if let Ok(syntax_tree) = syn::parse_file(&content) {
            security_analyzer.visit_file(&syntax_tree);
        }
    }
    
    // Generate security report
    println!("🔒 Security Analysis Report");
    println!("══════════════════════════");
    
    println!("📊 Summary:");
    println!("  • Unsafe blocks: {}", security_analyzer.unsafe_blocks);
    println!("  • External calls: {}", security_analyzer.external_calls.len());
    println!("  • Potential issues: {}", security_analyzer.potential_issues.len());
    
    if !security_analyzer.potential_issues.is_empty() {
        println!("\n⚠️  Potential Security Issues:");
        for issue in &security_analyzer.potential_issues {
            println!("  • {}", issue);
        }
    }
    
    if !security_analyzer.external_calls.is_empty() {
        println!("\n🌐 External System Calls:");
        let mut call_counts: HashMap<String, u32> = HashMap::new();
        for call in &security_analyzer.external_calls {
            *call_counts.entry(call.clone()).or_insert(0) += 1;
        }
        
        for (call, count) in call_counts {
            println!("  • {} (used {} times)", call, count);
        }
    }
    
    Ok(())
}
```

## Running the Examples

### Prerequisites

```bash
# Install RustEx
cargo install rustex-cli

# Or build from source
git clone https://github.com/your-username/rustex.git
cd rustex
cargo build --release
```

### Running CLI Examples

```bash
# Basic usage
rustex extract --pretty

# Documentation generation
rustex extract --format markdown --include-docs --output docs.md

# Code analysis
rustex extract --plugins "complexity-analyzer" --output analysis.json
```

### Running Library Examples

```bash
# Compile and run an example
cargo run --example basic_usage

# Run with a specific project
cargo run --example documentation_generator -- --path /path/to/project

# Run custom analyzer
cargo run --example custom_analyzer
```

### Using Configuration Examples

```bash
# Use a specific configuration
rustex extract --config examples/configs/documentation.toml

# Initialize with template
rustex config init --template llm-training --output my-config.toml
```

---

These examples demonstrate the flexibility and power of RustEx for various use cases. Feel free to modify and adapt them for your specific needs!