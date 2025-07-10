# RustEx User Guide

Welcome to RustEx! This comprehensive guide will walk you through everything you need to know to effectively use RustEx for extracting and analyzing Rust ASTs.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Configuration](#configuration)
- [Output Formats](#output-formats)
- [Examples](#examples)
- [Advanced Usage](#advanced-usage)
- [Use Cases](#use-cases)
- [Troubleshooting](#troubleshooting)

## Installation

### Option 1: Install from Crates.io (Recommended)

```bash
cargo install rustex-cli
```

### Option 2: Build from Source

```bash
git clone https://github.com/your-username/rustex.git
cd rustex
cargo build --release
# Binary will be available at target/release/rustex
```

### Option 3: Using cargo-binstall

```bash
cargo binstall rustex-cli
```

### Verify Installation

```bash
rustex --version
rustex --help
```

## Quick Start

### Extract AST from Your Project

Navigate to your Rust project directory and run:

```bash
# Basic extraction with pretty JSON output
rustex extract --pretty

# Save output to file
rustex extract --output my-project-ast.json --pretty

# Include documentation comments
rustex extract --include-docs --pretty
```

### Generate Readable Documentation

```bash
# Create markdown documentation
rustex extract --format markdown --output project-docs.md

# Include private items in documentation
rustex extract --format markdown --include-private --output full-docs.md
```

### Analyze Project Metrics

```bash
# Get basic project metrics
rustex metrics --complexity --loc

# Save detailed metrics
rustex metrics --complexity --loc --output metrics.json
```

## Command Reference

### `rustex extract`

Extracts AST information from Rust source files.

#### Options:

- `--format, -f`: Output format (`json`, `markdown`, `messagepack`, `graphql`, `rag`)
  - Default: `json`
- `--output, -o`: Output file path (if not specified, prints to stdout)
- `--include-docs`: Include documentation comments in output
- `--include-private`: Include private items (functions, structs, etc.)
- `--parse-deps`: Parse project dependencies
- `--max-file-size`: Maximum file size to process (default: 10MB)
- `--include`: File patterns to include (glob patterns, comma-separated)
- `--exclude`: File patterns to exclude (glob patterns, comma-separated)
- `--plugins`: Enable specific plugins (comma-separated)
- `--pretty`: Pretty-print JSON output

#### Examples:

```bash
# Extract all public items with documentation
rustex extract --include-docs --pretty

# Extract only specific files
rustex extract --include "src/lib.rs,src/main.rs" --pretty

# Exclude test files
rustex extract --exclude "tests/**,benches/**" --pretty

# Process dependencies too
rustex extract --parse-deps --include-docs --pretty

# Generate LLM-ready output
rustex extract --format rag --output llm-data.json
```

### `rustex deps`

Analyzes project dependencies.

#### Options:

- `--visualize`: Create dependency visualization
- `--output, -o`: Output file for visualization

#### Examples:

```bash
# Analyze dependencies
rustex deps

# Create dependency graph
rustex deps --visualize --output deps.svg
```

### `rustex metrics`

Calculates various project metrics.

#### Options:

- `--complexity`: Include complexity analysis
- `--loc`: Include lines of code metrics
- `--output, -o`: Output file for metrics

#### Examples:

```bash
# Basic metrics
rustex metrics

# Detailed analysis
rustex metrics --complexity --loc --output detailed-metrics.json
```

### `rustex config`

Manages configuration files.

#### Subcommands:

- `init`: Create a new configuration file
- `validate`: Validate existing configuration
- `show`: Display current configuration
- `template`: Generate configuration templates

#### Examples:

```bash
# Create default configuration
rustex config init

# Create configuration for LLM training
rustex config init --template llm-training

# Validate current configuration
rustex config validate

# Show current configuration
rustex config show
```

## Configuration

RustEx supports flexible configuration through TOML files. Configuration files are searched in this order:

1. File specified with `--config` flag
2. `rustex.toml` in project root
3. `.rustex.toml` in project root
4. User's home directory configuration

### Basic Configuration

Create `rustex.toml` in your project root:

```toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = false
max_file_size = "10MB"
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**", "benches/**"]

[plugins]
enabled = []

[output]
pretty_print = true
```

### Use Case Templates

RustEx provides pre-configured templates for common use cases:

#### Documentation Generation

```bash
rustex config init --template documentation
```

```toml
[extraction]
include_docs = true
include_private = true
parse_dependencies = false
output_format = "markdown"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**"]
```

#### Code Analysis

```bash
rustex config init --template code-analysis
```

```toml
[extraction]
include_docs = true
include_private = true
parse_dependencies = true
output_format = "json"

[plugins]
enabled = ["complexity", "metrics"]

[filters]
include = ["src/**/*.rs", "lib/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**"]
```

#### LLM Training

```bash
rustex config init --template llm-training
```

```toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = true
output_format = "rag"

[plugins]
enabled = ["llm-optimizer"]

[filters]
include = ["src/**/*.rs", "lib/**/*.rs", "examples/**/*.rs"]
exclude = ["target/**", "tests/**", "benches/**"]
```

## Output Formats

### JSON Format

Structured data perfect for programmatic processing:

```json
{
  "project": {
    "name": "my-crate",
    "version": "0.1.0",
    "rust_edition": "2021",
    "description": "A sample Rust crate"
  },
  "files": [
    {
      "path": "src/lib.rs",
      "relative_path": "src/lib.rs",
      "file_metrics": {
        "lines_of_code": 45,
        "function_count": 3,
        "struct_count": 1
      },
      "elements": [
        {
          "id": "Function_hello_world_1",
          "element_type": "Function",
          "name": "hello_world",
          "signature": "pub fn hello_world() -> String",
          "visibility": "Public",
          "doc_comments": ["Returns a greeting message"],
          "location": {
            "line_start": 10,
            "line_end": 12,
            "char_start": 0,
            "char_end": 45,
            "file_path": "src/lib.rs"
          },
          "complexity": 1,
          "complexity_metrics": {
            "cyclomatic": 1,
            "cognitive": 0,
            "halstead": {
              "n1": 2,
              "n2": 1,
              "big_n1": 2,
              "big_n2": 1
            }
          },
          "hierarchy": {
            "module_path": "crate",
            "qualified_name": "crate::hello_world",
            "parent_id": null,
            "children_ids": [],
            "nesting_level": 0
          }
        }
      ]
    }
  ],
  "metrics": {
    "total_files": 1,
    "total_lines": 45,
    "total_functions": 3,
    "total_structs": 1,
    "total_enums": 0,
    "total_traits": 0,
    "complexity_average": 2.3
  }
}
```

### Markdown Format

Human-readable documentation:

```markdown
# my-crate AST Analysis

**Version:** 0.1.0
**Rust Edition:** 2021

## Project Metrics

- **Total Files:** 1
- **Total Lines:** 45
- **Functions:** 3
- **Structs:** 1
- **Average Complexity:** 2.30

## Files

### src/lib.rs

#### Function `hello_world`

**Documentation:**
> Returns a greeting message

```rust
pub fn hello_world() -> String
```
```

### RAG Format

Optimized for Retrieval-Augmented Generation and LLM training:

```json
{
  "metadata": {
    "project_name": "my-crate",
    "version": "0.1.0",
    "chunking_strategy": "semantic",
    "embedding_strategy": "combined",
    "generated_at": "2024-01-15T10:30:00Z"
  },
  "chunks": [
    {
      "id": "chunk_1",
      "content": "/// Returns a greeting message\npub fn hello_world() -> String {\n    \"Hello, World!\".to_string()\n}",
      "metadata": {
        "element_type": "Function",
        "element_name": "hello_world",
        "file_path": "src/lib.rs",
        "complexity": 1,
        "context_window": 512,
        "semantic_tags": ["greeting", "string_operations"],
        "documentation_quality": "good"
      },
      "embeddings": {
        "code_vector": [0.1, 0.2, 0.3, "..."],
        "doc_vector": [0.4, 0.5, 0.6, "..."]
      }
    }
  ],
  "semantics": {
    "concepts": ["string_manipulation", "public_api"],
    "complexity_distribution": {
      "simple": 5,
      "moderate": 2,
      "complex": 1
    },
    "api_surface": {
      "public_functions": 3,
      "public_structs": 1,
      "public_traits": 0
    }
  },
  "training_examples": [
    {
      "input": "Write a function that returns a greeting message",
      "output": "pub fn hello_world() -> String {\n    \"Hello, World!\".to_string()\n}",
      "difficulty": "beginner",
      "category": "function_implementation"
    }
  ]
}
```

#### RAG Format Features

- **Intelligent Chunking**: Context-aware code segmentation with configurable chunk sizes
- **Semantic Analysis**: Automatic concept extraction and relationship mapping
- **Embedding Optimization**: Separate vectors for code and documentation content
- **Training Data Generation**: Automatic creation of instruction-following examples
- **Quality Assessment**: Documentation and code quality scoring
- **Metadata Enrichment**: Comprehensive context information for each chunk

## Examples

RustEx includes comprehensive examples demonstrating various use cases and output formats. These examples are ready to run and generate output files for inspection.

### Running Examples

All examples can be run from the project root using:

```bash
cargo run --example <example_name>
```

### Available Examples

#### 1. Basic Usage (`basic_usage.rs`)

**Purpose**: Demonstrates fundamental AST extraction and JSON output.

```bash
cargo run --example basic_usage
```

**What it does**:
- Extracts AST from the current project
- Generates clean JSON output with basic configuration
- Shows project structure and element hierarchy
- Demonstrates error handling and progress indicators

**Output**: Creates `basic-ast-output.json` with project structure.

#### 2. Documentation Generator (`documentation_generator.rs`)

**Purpose**: Generates comprehensive project documentation.

```bash
cargo run --example documentation_generator
```

**What it does**:
- Creates markdown documentation with API reference
- Includes complexity analysis and metrics
- Groups public API elements by type
- Generates cross-reference information

**Output**: Creates `PROJECT_DOCUMENTATION.md` with full documentation.

#### 3. Code Analyzer (`code_analyzer.rs`)

**Purpose**: Performs comprehensive code quality analysis.

```bash
cargo run --example code_analyzer
```

**What it does**:
- Analyzes complexity distribution across the project
- Identifies high-complexity functions for refactoring
- Calculates documentation coverage metrics
- Generates quality score and improvement recommendations
- Creates detailed analysis reports

**Output**: Creates `code-analysis-report.md` with detailed metrics.

#### 4. LLM Data Preparation (`llm_data_prep.rs`)

**Purpose**: Prepares training data for language models.

```bash
cargo run --example llm_data_prep
```

**What it does**:
- Generates training examples with input/output pairs
- Creates Q&A pairs from documentation
- Prepares code chunks for RAG systems
- Organizes data by difficulty level and categories
- Provides statistics on dataset composition

**Outputs**:
- `llm-training-dataset.json` (complete dataset)
- `training-examples.jsonl` (examples in JSONL format)
- `qa-pairs.json` (question-answer pairs)
- `rag-chunks.json` (chunks for RAG systems)
- `dataset-metadata.json` (metadata and statistics)

#### 5. RAG Output Demo (`rag_output_demo.rs`)

**Purpose**: Demonstrates RAG-optimized output formats.

```bash
cargo run --example rag_output_demo
```

**What it does**:
- Shows different chunking strategies (fixed, semantic, adaptive)
- Demonstrates embedding optimization techniques
- Creates semantic analysis and concept hierarchies
- Generates training examples for fine-tuning
- Exports in multiple formats (JSON, JSONL, embeddings)

**Outputs**:
- `rag-demo-output.json` (RAG format)
- `rag-demo-output.jsonl` (JSONL format)
- `rag-embeddings-output.json` (embedding-optimized)

### Example Configurations

Each example uses different configurations to demonstrate various use cases:

```rust
// Basic usage - minimal configuration
let config = ExtractorConfig::for_use_case(ConfigUseCase::Basic);

// Documentation - include docs and private items
let config = ExtractorConfig::for_use_case(ConfigUseCase::Documentation);

// Code analysis - comprehensive analysis
let config = ExtractorConfig::for_use_case(ConfigUseCase::CodeAnalysis);

// LLM training - optimized for AI consumption
let config = ExtractorConfig::for_use_case(ConfigUseCase::LlmTraining);

// RAG - specialized for retrieval systems
let config = ExtractorConfig::for_use_case(ConfigUseCase::Rag);
```

### Understanding Example Output

#### JSON Structure
All examples generate structured JSON that includes:
- **Project metadata**: Name, version, edition
- **File information**: Paths, metrics, elements
- **Element details**: Functions, structs, enums, traits
- **Complexity metrics**: Cyclomatic, cognitive, Halstead
- **Cross-references**: Dependencies and relationships

#### Markdown Documentation
Documentation examples create human-readable markdown with:
- **Table of contents**: Organized by modules and types
- **API reference**: Public functions, structs, and traits
- **Complexity analysis**: Distribution and high-complexity items
- **Implementation details**: Private items when included

#### RAG-Optimized Output
RAG examples demonstrate specialized formatting for:
- **Chunked content**: Context-aware code segmentation
- **Semantic analysis**: Concept extraction and relationships
- **Training examples**: Input/output pairs for fine-tuning
- **Embedding preparation**: Optimized vectors for retrieval

### Customizing Examples

You can modify the examples to test different configurations:

1. **Change file filters**:
```rust
config.filters.include = vec!["src/specific_module/**/*.rs".to_string()];
```

2. **Adjust complexity thresholds**:
```rust
config.complexity.max_complexity = 15;
```

3. **Enable different plugins**:
```rust
config.plugins.enabled = vec!["semantic-analyzer".to_string()];
```

4. **Modify output formats**:
```rust
let rag_config = RagConfig {
    chunking_strategy: ChunkingStrategy::Adaptive,
    max_chunk_size: 1024,
    // ... other options
};
```

### Example Best Practices

1. **Start with basic_usage** to understand core functionality
2. **Use documentation_generator** for project documentation
3. **Run code_analyzer** to identify improvement opportunities
4. **Try llm_data_prep** for AI/ML workflows
5. **Explore rag_output_demo** for advanced RAG applications

## Advanced Usage

### Custom File Filtering

Use glob patterns for precise file selection:

```bash
# Include only specific modules
rustex extract --include "src/lib.rs,src/api/**/*.rs"

# Exclude test and benchmark files
rustex extract --exclude "tests/**,benches/**,examples/**"

# Complex filtering
rustex extract \
  --include "src/**/*.rs,lib/**/*.rs" \
  --exclude "src/internal/**,lib/deprecated/**"
```

### Working with Large Projects

For large codebases, consider these optimizations:

```bash
# Increase file size limit
rustex extract --max-file-size 50MB

# Exclude dependencies to focus on your code
rustex extract --exclude "target/**,vendor/**"

# Use specific output format for faster processing
rustex extract --format messagepack --output fast-output.msgpack
```

### Plugin System

Enable plugins for enhanced analysis:

```bash
# Enable complexity analysis plugin
rustex extract --plugins complexity-analyzer

# Multiple plugins
rustex extract --plugins "complexity-analyzer,llm-optimizer,doc-enhancer"
```

### Configuration Validation

Always validate your configuration:

```bash
# Validate current configuration
rustex config validate

# Validate specific configuration file
rustex config validate --file custom-config.toml
```

## Use Cases

### 1. Code Documentation Generation

Generate comprehensive documentation for your Rust project:

```bash
# Create full project documentation
rustex extract \
  --format markdown \
  --include-docs \
  --include-private \
  --output PROJECT_DOCS.md

# Create public API documentation only
rustex extract \
  --format markdown \
  --include-docs \
  --exclude "src/internal/**" \
  --output PUBLIC_API.md
```

### 2. LLM Training Data Preparation

Prepare your Rust code for LLM training:

```bash
# Generate LLM-optimized output
rustex extract \
  --format rag \
  --include-docs \
  --plugins llm-optimizer \
  --output training-data.json

# Create chunked data for fine-tuning
rustex extract \
  --format rag \
  --include-docs \
  --exclude "tests/**,benches/**" \
  --output fine-tuning-data.json
```

### 3. Code Analysis and Metrics

Analyze code quality and complexity:

```bash
# Comprehensive code analysis
rustex extract \
  --format json \
  --include-private \
  --plugins "complexity-analyzer,metrics" \
  --output analysis.json

# Generate metrics report
rustex metrics \
  --complexity \
  --loc \
  --output metrics-report.json
```

### 4. API Documentation

Extract public API structure:

```bash
# Public API only
rustex extract \
  --format json \
  --include-docs \
  --exclude "src/internal/**,tests/**" \
  --output public-api.json

# Generate OpenAPI-style documentation
rustex extract \
  --format graphql \
  --include-docs \
  --output api-schema.graphql
```

### 5. Code Migration and Refactoring

Analyze code structure for migration:

```bash
# Extract all dependencies and structure
rustex extract \
  --include-private \
  --parse-deps \
  --plugins "complexity-analyzer" \
  --output migration-analysis.json

# Focus on complex functions for refactoring
rustex extract \
  --format json \
  --plugins complexity-analyzer \
  --output refactoring-candidates.json
```

## Troubleshooting

### Common Issues

#### 1. File Not Found Errors

**Problem:** RustEx can't find your Rust files.

**Solution:**
```bash
# Check your current directory
pwd

# Verify Cargo.toml exists
ls Cargo.toml

# Use absolute path if needed
rustex extract --path /full/path/to/project
```

#### 2. Large File Warnings

**Problem:** Files are too large to process.

**Solution:**
```bash
# Increase file size limit
rustex extract --max-file-size 50MB

# Or exclude large files
rustex extract --exclude "src/generated/**"
```

#### 3. Parse Errors

**Problem:** Syntax errors in Rust files.

**Solution:**
```bash
# Check if your code compiles
cargo check

# Exclude problematic files temporarily
rustex extract --exclude "src/problematic.rs"
```

#### 4. Configuration Issues

**Problem:** Invalid configuration file.

**Solution:**
```bash
# Validate configuration
rustex config validate

# Show current configuration
rustex config show

# Create fresh configuration
rustex config init --force
```

#### 5. Memory Issues with Large Projects

**Problem:** Out of memory errors.

**Solution:**
```bash
# Process files in smaller batches
rustex extract --include "src/module1/**" --output module1.json
rustex extract --include "src/module2/**" --output module2.json

# Reduce file size limit
rustex extract --max-file-size 5MB
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Enable debug output
rustex extract --verbose

# Redirect debug output to file
rustex extract --verbose 2> debug.log
```

### Getting Help

- **Documentation:** Check this user guide and API documentation
- **Issues:** Report bugs on GitHub
- **Discussions:** Join community discussions for questions
- **CLI Help:** Use `rustex --help` or `rustex extract --help`

## Performance Tips

1. **Use file filtering** to process only relevant files
2. **Exclude large generated files** to reduce processing time
3. **Disable dependency parsing** if not needed (`--no-parse-deps`)
4. **Use binary formats** (MessagePack) for faster I/O
5. **Process in batches** for very large projects

## Next Steps

- Explore the [API Documentation](api-reference.md) for library usage
- Check out [Examples](../examples/) for common usage patterns
- Learn about [Plugin Development](plugin-development.md) for custom analysis
- Read the [Configuration Reference](configuration-reference.md) for advanced settings

---

For more information, visit the [project homepage](https://github.com/your-username/rustex) or read the [API documentation](https://docs.rs/rustex-core).