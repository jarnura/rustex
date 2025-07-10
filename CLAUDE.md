# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**RustEx** is a comprehensive Rust AST (Abstract Syntax Tree) extractor designed for LLM/RAG applications and code analysis. The project extracts structured representations of Rust code to enable AI-powered development tools and research.

## Development Commands

### Building and Testing
```bash
# Build all workspace crates
cargo build

# Run tests (when implemented)
cargo test

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy -- -D warnings
```

### Running the CLI
```bash
# Build and run the CLI tool
cargo run --bin rustex -- extract

# Extract with specific options
cargo run --bin rustex -- extract --include-docs --format json --pretty
```

## Architecture

### Workspace Structure
The project is organized as a Rust workspace with multiple crates:

- **`crates/rustex-core/`** - Core AST extraction library
  - `src/lib.rs` - Main library interface
  - `src/extractor.rs` - Main extraction engine
  - `src/config.rs` - Configuration structures
  - `src/ast_data.rs` - Data structures for AST representation
  - `src/visitors.rs` - AST visitor implementations

- **`crates/rustex-cli/`** - Command-line interface
  - `src/main.rs` - CLI implementation with clap-based commands

- **`crates/rustex-plugins/`** - Plugin system (planned)
- **`crates/rustex-formats/`** - Output formatters (planned)

### Core Components

1. **AstExtractor** - Main extraction engine that orchestrates the parsing process
2. **CodeElementVisitor** - syn::visit-based visitor that extracts code elements
3. **ExtractorConfig** - Configuration system for extraction options
4. **Data Structures** - Rich AST representation including metadata and metrics

### Key Dependencies
- `syn` - Rust parser and AST library (version 2.0 with full features)
- `serde` - Serialization framework for output formats
- `clap` - CLI argument parsing
- `walkdir` - Directory traversal for project discovery
- `tracing` - Structured logging

## Development Status

### Phase 1: Foundation ✅ COMPLETED
- ✅ Rust workspace with 4 crates (core, CLI, plugins, formats)
- ✅ Complete AST data structures (15+ types with serde support)
- ✅ Full CLI interface with extract/deps/metrics/init commands
- ✅ Configuration system with TOML support
- ✅ Progress indicators and colored terminal output
- ✅ Project infrastructure (README, licenses, .gitignore)

### Phase 2: Core Implementation 🚧 IN PROGRESS
- ✅ AstExtractor with syn-based parsing
- ✅ File discovery and filtering logic with glob patterns
- ✅ JSON output formatting with actual AST data
- ✅ Import/use statement parsing (basic implementation)
- ✅ Progress indicators for CLI operations
- ✅ Markdown output formatter
- 🚧 CodeElementVisitor improvements (functions not being extracted)
- ⏸️ Documentation extraction from doc comments
- ⏸️ Error handling and recovery mechanisms

### Phase 3: Advanced Features ⏸️ PLANNED
- ⏸️ Plugin system architecture
- ⏸️ Multiple output formats (Markdown, RAG)
- ⏸️ Complexity calculation algorithms
- ⏸️ Incremental parsing with caching
- ⏸️ Parallel processing support

### Key Files to Understand
- `docs/core_plan.md` - Contains complete implementation with all code
- `docs/project_prd.txt` - Comprehensive product requirements and features
- `docs/project_plan.md` - Detailed technical architecture and roadmap

## Implementation Notes

### Code Organization
The implementation follows a visitor pattern using `syn::visit::Visit` to traverse and extract information from Rust AST nodes. The main extraction flow:

1. **Discovery** - Find Rust files in project using `walkdir`
2. **Parsing** - Parse each file using `syn::parse_file`
3. **Extraction** - Use visitor pattern to extract code elements
4. **Processing** - Apply configuration filters and transformations
5. **Output** - Format results according to specified output format

### Configuration System
Uses a TOML-based configuration system (`rustex.toml`) for:
- Extraction options (docs, private items, dependencies)
- File filtering (include/exclude patterns)
- Output format preferences
- Plugin configuration

### Error Handling
The codebase uses `anyhow::Result` for error handling throughout, with structured error messages and graceful degradation for parsing failures.

### Performance Considerations
- File size limits to prevent memory issues
- Streaming architecture planned for large projects
- Parallel processing support planned
- Incremental parsing with caching planned

## Development Rules

### Critical Rule: Fix Before Proceeding
**NEVER proceed to the next task if the current step has compilation errors or issues.** 

When implementing any feature:
1. **Fix ALL compilation errors** before moving forward
2. **Ensure tests pass** for the current implementation
3. **Verify functionality works** with manual testing if needed
4. **Only then** proceed to the next task in the todo list

This prevents cascading issues and maintains code quality throughout development.

## Common Tasks

### Adding New Code Element Extraction
1. Add new variant to `ElementType` enum in `ast_data.rs`
2. Implement visitor method in `CodeElementVisitor` 
3. Update metrics calculation if needed
4. Add tests for the new element type

### Adding New Output Format
1. Create new variant in `OutputFormat` enum
2. Implement formatting logic in CLI or separate formatter
3. Update CLI to handle new format option
4. Add documentation and examples

### Testing Strategy
- Unit tests for individual extractors and visitors
- Integration tests with sample Rust projects
- Benchmark tests for performance regression
- End-to-end CLI testing

### Fixing Compilation Issues
When encountering compilation errors:
1. **Identify the root cause** - read error messages carefully
2. **Fix one error at a time** - don't try to fix multiple issues simultaneously
3. **Test after each fix** - ensure each fix doesn't break other things
4. **Update related code** - fix cascading issues from the changes

The project is currently in early development phase with core parsing functionality implemented but many advanced features still in planning/development stage.