# RustEx - Rust AST Extractor

[![Build Status](https://github.com/your-username/rustex/workflows/CI/badge.svg)](https://github.com/your-username/rustex/actions)
[![Crates.io](https://img.shields.io/crates/v/rustex-cli.svg)](https://crates.io/crates/rustex-cli)
[![Documentation](https://docs.rs/rustex-core/badge.svg)](https://docs.rs/rustex-core)

A comprehensive, high-performance tool for extracting Abstract Syntax Trees from Rust projects, optimized for LLM/RAG applications and code analysis.

## Features

- 🚀 **Fast & Efficient**: Parse entire Rust projects quickly using the `syn` crate
- 📊 **Comprehensive**: Extract functions, structs, enums, traits, and more with full metadata
- 🔌 **Extensible**: Plugin system for custom analysis
- 📝 **Multiple Formats**: JSON, Markdown, RAG, GraphQL, and MessagePack outputs
- 🤖 **LLM Ready**: Specialized RAG-optimized formats for language models and embedding systems
- 🧠 **AI Integration**: Intelligent chunking, semantic analysis, and training data generation
- 🎯 **RAG Support**: Context-aware chunking, metadata enrichment, and embedding optimization
- 📈 **Analytics**: Advanced code metrics, complexity analysis, and quality assessment
- 🔍 **Cross-References**: Hierarchical relationships and namespace-aware element tracking
- ⚙️ **Configurable**: Flexible filtering and extraction options with TOML configuration

## Quick Start

### Installation

```bash
# Install from crates.io (when published)
cargo install rustex-cli

# Or build from source
git clone https://github.com/your-username/rustex.git
cd rustex
cargo build --release
```

### Basic Usage

```bash
# Extract AST from current project
rustex extract

# Extract with pretty-printed JSON output
rustex extract --pretty --output ast.json

# Extract with documentation included
rustex extract --include-docs --format json

# Generate markdown documentation
rustex extract --format markdown --output docs.md

# Initialize configuration file
rustex init
```

### Advanced Usage

```bash
# Extract with custom file filters
rustex extract --include "src/**/*.rs" --exclude "tests/**"

# Extract private items and dependencies
rustex extract --include-private --parse-deps

# Generate RAG-optimized output
rustex extract --format rag --output rag-data.json

# Extract with plugins (coming soon)
rustex extract --plugins llm-optimizer,rag-preprocessor

# Project metrics analysis
rustex metrics --complexity --loc --output metrics.json

# Dependency analysis
rustex deps --visualize --output deps.svg
```

## Project Structure

```
rustex/
├── crates/
│   ├── rustex-core/      # Core AST extraction library
│   ├── rustex-cli/       # Command-line interface
│   ├── rustex-plugins/   # Plugin system (planned)
│   └── rustex-formats/   # Output formatters (planned)
├── docs/                 # Project documentation
├── examples/             # Usage examples
└── tests/                # Integration tests
```

## Configuration

Create a `rustex.toml` file in your project root:

```toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = false
max_file_size = "10MB"
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**"]

[plugins]
enabled = []
```

## Output Formats

### JSON Format

Structured data perfect for programmatic processing and LLM consumption:

```json
{
  "project": {
    "name": "my-project",
    "version": "0.1.0",
    "rust_edition": "2021"
  },
  "files": [
    {
      "path": "src/main.rs",
      "elements": [
        {
          "element_type": "Function",
          "name": "main",
          "signature": "fn main()",
          "visibility": "Public",
          "doc_comments": ["Entry point of the application"],
          "complexity": 3
        }
      ]
    }
  ],
  "metrics": {
    "total_lines": 150,
    "total_functions": 8,
    "complexity_average": 2.3
  }
}
```

### Markdown Format

Human-readable documentation with metrics and code summaries.

### RAG Format

Optimized for Retrieval-Augmented Generation and LLM training:

```json
{
  "metadata": {
    "project_name": "my-project",
    "chunking_strategy": "semantic",
    "embedding_strategy": "combined"
  },
  "chunks": [
    {
      "id": "chunk_1",
      "content": "/// Calculate fibonacci number\nfn fibonacci(n: u64) -> u64 { ... }",
      "metadata": {
        "element_type": "Function",
        "complexity": 3,
        "semantic_tags": ["algorithm", "recursive"],
        "context_window": 512
      },
      "embeddings": {
        "code_vector": [0.1, 0.2, ...],
        "doc_vector": [0.3, 0.4, ...]
      }
    }
  ],
  "training_examples": [
    {
      "input": "Write a function to calculate fibonacci numbers",
      "output": "fn fibonacci(n: u64) -> u64 { ... }",
      "difficulty": "intermediate"
    }
  ]
}
```

## Development Status

**Current Version:** 0.1.0 (Alpha)

### ✅ Completed Features
- ✅ Complete AST extraction for functions, structs, enums, traits, and modules
- ✅ Hierarchical code structure analysis with parent-child relationships
- ✅ Cross-reference resolution and tracking
- ✅ Namespace-aware element naming with qualified paths
- ✅ Comprehensive CLI interface with all commands
- ✅ JSON, Markdown, and RAG output formats
- ✅ RAG-optimized output with intelligent chunking and semantic analysis
- ✅ Advanced file discovery and filtering with glob patterns
- ✅ Configuration system with TOML support and use-case templates
- ✅ Documentation extraction from doc comments
- ✅ Import/use statement parsing and alias resolution
- ✅ Complexity calculation (cyclomatic, cognitive, Halstead metrics)
- ✅ Progress indicators and colored terminal output
- ✅ Comprehensive error handling and recovery
- ✅ Plugin system architecture
- ✅ Test fixtures and property-based testing
- ✅ Benchmark suite for performance testing
- ✅ Working examples demonstrating all major features

### 🚧 In Progress
- 📚 Comprehensive documentation and examples
- 🔌 Built-in plugins (complexity analysis, LLM optimization, documentation enhancement)

### 📋 Planned Features
- Incremental parsing with caching
- Parallel processing for large projects
- Advanced complexity metrics
- Dependency analysis and visualization

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-username/rustex.git
cd rustex

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run --bin rustex -- --help
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench
```

## Examples

The project includes comprehensive examples demonstrating all major features:

### Basic Usage (`basic_usage.rs`)
```bash
cargo run --example basic_usage
```
Demonstrates fundamental AST extraction and JSON output.

### Documentation Generator (`documentation_generator.rs`)
```bash
cargo run --example documentation_generator
```
Generates comprehensive project documentation with metrics and API reference.

### Code Analyzer (`code_analyzer.rs`)
```bash
cargo run --example code_analyzer
```
Performs complexity analysis, quality assessment, and generates improvement recommendations.

### LLM Data Preparation (`llm_data_prep.rs`)
```bash
cargo run --example llm_data_prep
```
Prepares training data for language models with chunking and Q&A generation.

### RAG Output Demo (`rag_output_demo.rs`)
```bash
cargo run --example rag_output_demo
```
Demonstrates RAG-optimized output formats with semantic analysis and embedding preparation.

All examples generate output files that demonstrate the capabilities of each format and use case.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built on the excellent [`syn`](https://github.com/dtolnay/syn) crate for Rust parsing
- Inspired by the growing ecosystem of AI-powered development tools
- Special thanks to the Rust community for feedback and contributions

---

**RustEx** - Bridging Rust code analysis and AI/ML workflows