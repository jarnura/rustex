# RustEx - Rust AST Extractor

[![Build Status](https://github.com/your-username/rustex/workflows/CI/badge.svg)](https://github.com/your-username/rustex/actions)
[![Crates.io](https://img.shields.io/crates/v/rustex-cli.svg)](https://crates.io/crates/rustex-cli)
[![Documentation](https://docs.rs/rustex-core/badge.svg)](https://docs.rs/rustex-core)

A comprehensive, high-performance tool for extracting Abstract Syntax Trees from Rust projects, optimized for LLM/RAG applications and code analysis.

## Features

- 🚀 **Fast & Efficient**: Parse entire Rust projects quickly using the `syn` crate
- 📊 **Comprehensive**: Extract functions, structs, enums, traits, and more with full metadata
- 🔌 **Extensible**: Plugin system for custom analysis (coming soon)
- 📝 **Multiple Formats**: JSON, Markdown, and specialized RAG-optimized outputs
- 🤖 **LLM Ready**: Optimized output formats for language models and RAG systems
- 📈 **Analytics**: Code metrics and complexity analysis
- ⚙️ **Configurable**: Flexible filtering and extraction options

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

## Development Status

**Current Version:** 0.1.0 (Alpha)

### ✅ Completed Features
- Core AST parsing infrastructure
- Complete CLI interface with all commands
- JSON output format
- Basic file discovery and filtering
- Configuration system
- Progress indicators

### 🚧 In Progress
- Full AST extraction implementation
- Documentation extraction
- Import/use statement parsing
- Complexity calculation

### 📋 Planned Features
- Plugin system for extensibility
- RAG-optimized output formats
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