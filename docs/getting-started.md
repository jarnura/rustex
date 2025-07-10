# Getting Started with RustEx

Welcome to RustEx! This guide will help you get up and running quickly with RustEx for extracting and analyzing Rust ASTs.

## Table of Contents

- [What is RustEx?](#what-is-rustex)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Concepts](#basic-concepts)
- [Common Use Cases](#common-use-cases)
- [Configuration Basics](#configuration-basics)
- [Next Steps](#next-steps)

## What is RustEx?

RustEx is a comprehensive tool for extracting Abstract Syntax Trees (ASTs) from Rust projects. It's designed to bridge the gap between Rust code analysis and modern AI/ML workflows, making it easy to:

- **Extract structured data** from Rust codebases for analysis
- **Generate documentation** automatically from your code
- **Prepare training data** for language models
- **Analyze code quality** and complexity metrics
- **Integrate** with other development tools and workflows

### Key Features

- 🚀 **Fast extraction** using the proven `syn` crate
- 📊 **Rich analysis** including complexity metrics and cross-references
- 🔌 **Extensible** plugin system for custom analysis
- 📝 **Multiple output formats** (JSON, Markdown, RAG-optimized)
- ⚙️ **Flexible configuration** for different use cases
- 🤖 **LLM-ready** output formats for AI applications

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

# The binary will be available at target/release/rustex
# You can copy it to your PATH or use it directly
sudo cp target/release/rustex /usr/local/bin/
```

### Option 3: Download Pre-built Binaries

Visit the [releases page](https://github.com/your-username/rustex/releases) to download pre-built binaries for your platform.

### Verify Installation

```bash
rustex --version
rustex --help
```

You should see output similar to:
```
rustex 0.1.0
A comprehensive Rust AST extractor for LLM and RAG applications
```

## Quick Start

Let's extract AST information from a Rust project in just a few steps!

### 1. Navigate to a Rust Project

```bash
cd /path/to/your/rust/project
```

Or create a simple test project:

```bash
cargo new hello-rustex
cd hello-rustex
```

### 2. Basic AST Extraction

Extract AST with pretty-printed JSON output:

```bash
rustex extract --pretty
```

This will analyze your project and output structured JSON to the terminal. You should see information about functions, structs, and other code elements.

### 3. Save Output to File

```bash
rustex extract --pretty --output my-project-ast.json
```

### 4. Include Documentation

```bash
rustex extract --include-docs --pretty --output documented-ast.json
```

### 5. Generate Readable Documentation

```bash
rustex extract --format markdown --include-docs --output project-docs.md
```

Open `project-docs.md` to see a comprehensive documentation of your project!

## Basic Concepts

### AST (Abstract Syntax Tree)

An AST is a tree representation of your code's structure. RustEx extracts this information and presents it in a structured format that's easy to process programmatically.

### Code Elements

RustEx identifies and extracts different types of code elements:

- **Functions**: Function definitions with signatures and complexity metrics
- **Structs**: Data structure definitions with fields
- **Enums**: Enumeration types with variants
- **Traits**: Trait definitions with methods
- **Modules**: Module structure and organization
- **Implementations**: `impl` blocks for types

### Hierarchical Structure

RustEx understands the hierarchical relationships between code elements:

- Parent-child relationships (modules contain functions, etc.)
- Qualified names (e.g., `my_module::my_function`)
- Nesting levels and scope information
- Cross-references between elements

### Metrics and Analysis

RustEx automatically calculates various metrics:

- **Complexity metrics**: Cyclomatic, cognitive, and Halstead complexity
- **Code metrics**: Lines of code, function counts, etc.
- **Quality indicators**: Documentation coverage, dependency analysis
- **Cross-references**: How code elements reference each other

## Common Use Cases

### 1. Project Documentation

Generate comprehensive documentation for your Rust project:

```bash
# Create project documentation
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

### 2. Code Analysis

Analyze your code quality and identify areas for improvement:

```bash
# Basic analysis
rustex extract --include-private --pretty --output analysis.json

# Get detailed metrics
rustex metrics --complexity --loc --output metrics.json

# Analyze with plugins
rustex extract --plugins "complexity-analyzer" --output detailed-analysis.json
```

### 3. LLM Training Data

Prepare your Rust code for language model training:

```bash
# Generate LLM-optimized data
rustex extract \
  --format rag \
  --include-docs \
  --exclude "tests/**,benches/**" \
  --output llm-training-data.json
```

### 4. Dependency Analysis

Understand your project's structure and dependencies:

```bash
# Analyze dependencies
rustex deps --visualize --output dependency-graph.svg

# Extract with dependency parsing
rustex extract --parse-deps --include-docs --output full-analysis.json
```

### 5. CI/CD Integration

Integrate RustEx into your continuous integration pipeline:

```bash
# Check code quality in CI
rustex extract --plugins "complexity-analyzer" --output ci-analysis.json

# Generate docs for deployment
rustex extract --format markdown --include-docs --output deployment-docs.md
```

## Configuration Basics

### Configuration Files

RustEx supports configuration files to customize extraction behavior. Create a `rustex.toml` file in your project root:

```toml
[extraction]
include_docs = true
include_private = false
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["tests/**", "benches/**"]

[plugins]
enabled = []
```

### Configuration Templates

Use pre-configured templates for common scenarios:

```bash
# Create config for documentation generation
rustex config init --template documentation

# Create config for code analysis
rustex config init --template code-analysis

# Create config for LLM training
rustex config init --template llm-training
```

### CLI Overrides

Command-line arguments override configuration file settings:

```bash
# Override output format
rustex extract --format markdown

# Override include settings
rustex extract --include-docs --include-private

# Override file filters
rustex extract --include "src/**/*.rs" --exclude "tests/**"
```

## Your First Analysis

Let's do a complete analysis of a project step by step:

### Step 1: Create a Test Project

```bash
cargo new --lib rustex-demo
cd rustex-demo
```

### Step 2: Add Some Code

Edit `src/lib.rs`:

```rust
//! A demo library for RustEx analysis

/// Calculates the factorial of a number
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(factorial(5), 120);
/// ```
pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

/// Represents a geometric shape
pub trait Shape {
    /// Calculate the area of the shape
    fn area(&self) -> f64;
}

/// A rectangle with width and height
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}
```

### Step 3: Analyze the Project

```bash
# Basic analysis
rustex extract --include-docs --pretty

# Generate documentation
rustex extract --format markdown --include-docs --output README.md

# Get metrics
rustex metrics --complexity --loc
```

### Step 4: Examine the Results

Look at the generated files:

```bash
# View the documentation
cat README.md

# View the JSON analysis (if you saved it)
cat analysis.json | jq .
```

You should see:
- Function definitions with documentation
- Struct and trait information
- Implementation blocks
- Complexity metrics
- Cross-references between elements

## Understanding the Output

### JSON Output Structure

```json
{
  "project": {
    "name": "rustex-demo",
    "version": "0.1.0",
    "rust_edition": "2021"
  },
  "files": [
    {
      "path": "src/lib.rs",
      "elements": [
        {
          "element_type": "Function",
          "name": "factorial",
          "signature": "pub fn factorial(n: u64) -> u64",
          "doc_comments": ["Calculates the factorial of a number"],
          "complexity": 3,
          "hierarchy": {
            "qualified_name": "crate::factorial",
            "module_path": "crate"
          }
        }
      ]
    }
  ],
  "metrics": {
    "total_functions": 3,
    "total_structs": 1,
    "total_traits": 1,
    "complexity_average": 2.3
  }
}
```

### Key Fields Explained

- **`element_type`**: The type of code element (Function, Struct, Enum, etc.)
- **`name`**: The identifier name
- **`signature`**: The full signature (for functions) or declaration
- **`doc_comments`**: Extracted documentation comments
- **`complexity`**: Calculated complexity score
- **`hierarchy`**: Information about the element's position in the code structure
- **`location`**: File location (line numbers, character positions)

## Next Steps

Now that you have RustEx running, explore these advanced features:

### 1. Learn More About Configuration

Read the [Configuration Reference](configuration-reference.md) to customize RustEx for your specific needs.

### 2. Explore the API

If you want to use RustEx programmatically, check out the [API Reference](api-reference.md).

### 3. Try Different Output Formats

Experiment with different output formats:

```bash
# Markdown for human-readable docs
rustex extract --format markdown --include-docs --output docs.md

# RAG format for LLM applications
rustex extract --format rag --include-docs --output rag-data.json

# MessagePack for efficient binary format
rustex extract --format messagepack --output data.msgpack
```

### 4. Use Plugins

Explore built-in plugins for enhanced analysis:

```bash
# Complexity analysis
rustex extract --plugins "complexity-analyzer" --output analysis.json

# Documentation enhancement
rustex extract --plugins "doc-enhancer" --output enhanced.json
```

### 5. Integrate with Your Workflow

- Add RustEx to your CI/CD pipeline
- Use it in build scripts
- Integrate with documentation generators
- Create custom analysis tools

### 6. Advanced Topics

- [Plugin Development](plugin-development.md) - Create custom plugins
- [User Guide](user-guide.md) - Comprehensive usage guide
- [Examples](../examples/) - Real-world usage patterns

## Getting Help

If you run into issues or have questions:

1. **Check the documentation** in the `docs/` directory
2. **Run with verbose output** using `--verbose` flag for debugging
3. **Validate your configuration** using `rustex config validate`
4. **Try the examples** in the `examples/` directory
5. **Report issues** on the GitHub repository

## Common Issues

### "No Rust files found"

Make sure you're in a directory with Rust files (`.rs` extensions). Check your file filters:

```bash
# List discovered files
rustex extract --verbose 2>&1 | grep "Found.*files"

# Check current directory
ls *.rs src/**/*.rs
```

### "Parse errors"

If RustEx can't parse your Rust files, make sure they compile:

```bash
# Check if your code compiles
cargo check

# Run RustEx with verbose output to see specific errors
rustex extract --verbose
```

### "Configuration errors"

Validate your configuration:

```bash
# Check configuration
rustex config validate

# Show current configuration
rustex config show
```

---

Welcome to RustEx! Happy analyzing! 🦀✨