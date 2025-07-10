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

### Phase 2: Core Implementation ✅ COMPLETED
- ✅ AstExtractor with syn-based parsing
- ✅ File discovery and filtering logic with glob patterns
- ✅ JSON output formatting with actual AST data
- ✅ Import/use statement parsing (basic implementation)
- ✅ Progress indicators for CLI operations
- ✅ Markdown output formatter
- ✅ CodeElementVisitor with full element extraction
- ✅ Documentation extraction from doc comments
- ✅ Error handling and recovery mechanisms
- ✅ Complexity calculation algorithms
- ✅ Hierarchical relationships and cross-references
- ✅ Namespace-aware element naming

### Phase 3: Advanced Features ✅ COMPLETED
- ✅ Plugin system architecture
- ✅ Multiple output formats (JSON, Markdown, RAG, GraphQL, MessagePack)
- ✅ RAG-optimized output formats for LLM applications
- ✅ Comprehensive testing suite with benchmarks
- ✅ Complete documentation and examples
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

### Git Commit Hygiene Rules

**MANDATORY: Every completed task must have a corresponding git commit.** This ensures proper progress tracking and maintains a clean development history.

#### Rule 1: Task-Based Commits
For every task completion:

```bash
# 1. Complete the task implementation
# 2. Test thoroughly (build, test, examples)
# 3. Update related documentation
# 4. Stage all changes
git add .

# 5. Commit with descriptive message
git commit -m "$(cat <<'EOF'
Complete Task [ID]: [Brief Description]

- [Specific change 1]
- [Specific change 2] 
- [Documentation updates]
- [Tests/examples updated]
EOF
)"
```

#### Rule 2: Commit Message Format
Use this standardized format for all commits:

```
Complete Task [ID]: [Brief Description]

- Bullet point of main changes
- Documentation updates made
- Tests or examples modified
- Any breaking changes noted

```

**Examples:**
```bash
# Example 1: Feature implementation
git commit -m "Complete Task 44: Implement PostgreSQL Database Schema

- Add database schema with ast_nodes, call_relationships, dependencies tables
- Create optimized indexes for graph traversal queries
- Add JSONB columns for flexible metadata storage
- Update documentation with schema design
- Add migration examples

# Example 2: Bug fix
git commit -m "Complete Task 25: Add RAG-optimized output formats

- Implement RAG document structure with semantic analysis
- Add intelligent chunking strategies (fixed, semantic, adaptive)
- Create training example generation for LLM fine-tuning
- Update examples with rag_output_demo.rs
- Enhance documentation with RAG format specification
```

#### Rule 3: Pre-Commit Checklist
Before each commit, verify:

```bash
□ Task is 100% complete and functional
□ All compilation errors resolved
□ All warnings addressed (cargo clippy passes)
□ Tests pass: cargo test --workspace
□ Examples compile: cargo check --examples
□ Documentation updated for any user-facing changes
□ No temporary/debug code left in
□ Commit message follows standard format
```

#### Rule 4: Commit Timing
- **Complete tasks individually** - Don't bundle multiple tasks in one commit
- **Commit immediately** after task completion - Don't accumulate changes
- **Small, focused commits** - Each commit should represent one logical unit of work
- **Working state only** - Never commit broken or half-finished code

#### Rule 5: Todo List Synchronization
After each commit:

```bash
# 1. Update todo status
# 2. Mark task as completed in TodoWrite
# 3. Verify todo list reflects current state
# 4. Proceed to next task
```

#### Enforcement Commands
```bash
# Pre-commit validation
cargo build --workspace && \
cargo test --workspace && \
cargo clippy --workspace -- -D warnings && \
cargo check --examples && \
echo "✅ Ready to commit"

# Check for uncommitted changes before starting new task
git status --porcelain | grep -q . && echo "⚠️  Uncommitted changes - commit first!" || echo "✅ Clean working directory"
```

### Documentation Maintenance Rules

**ALWAYS update related documentation when making changes.** This is a critical rule for maintaining project quality and usability.

#### Rule 1: Code Changes Require Documentation Updates
When making any code changes, immediately update all related documentation:

- **API Changes** → Update `docs/api-reference.md` and relevant docstrings
- **CLI Changes** → Update `README.md`, `docs/user-guide.md`, and `docs/cli-reference.md`
- **New Features** → Update `README.md` features section, user guides, and examples
- **Configuration Changes** → Update `docs/configuration-reference.md`
- **Output Format Changes** → Update format-specific documentation
- **Example Changes** → Update `examples/README.md` and relevant guides

#### Rule 2: New Features Documentation Checklist
Before marking any feature as complete, ensure:

```bash
□ README.md features section updated
□ User guide includes new feature usage
□ API reference documents new public APIs
□ Configuration reference updated if applicable
□ Examples created or updated to demonstrate feature
□ Getting started guide mentions feature if user-facing
□ Troubleshooting section updated with common issues
```

#### Rule 3: Example and Testing Updates
When adding or fixing examples:

```bash
□ examples/README.md documents all examples
□ Each example has clear description and usage
□ Generated output files are documented
□ Compilation requirements and dependencies listed
□ Integration with docs/getting-started.md updated
□ Troubleshooting section includes common issues
```

#### Rule 4: Dependency and Infrastructure Changes
When adding dependencies or changing build process:

```bash
□ README.md installation section updated
□ docs/getting-started.md setup instructions updated
□ CI/CD documentation reflects new requirements
□ Docker or container setup updated if applicable
□ Development setup instructions in CLAUDE.md updated
```

#### Rule 5: Breaking Changes Documentation
For any breaking changes:

```bash
□ Migration guide created or updated
□ CHANGELOG.md entry added
□ Version compatibility documented
□ Examples updated to work with new version
□ Deprecation notices added where appropriate
```

#### Enforcement Commands
```bash
# Check documentation is current after changes
git diff --name-only | grep -E '\.(rs|toml)$' && echo "⚠️  Code changed - check docs!"

# Validate all examples still work
cargo check --examples && echo "✅ Examples compile"

# Ensure documentation builds
# (Add doc build commands here when available)
```

#### Documentation Quality Standards
- **Be Comprehensive**: Cover all features and edge cases
- **Be Current**: Update immediately when code changes
- **Be Accessible**: Write for users at different skill levels
- **Be Consistent**: Follow established documentation patterns
- **Be Tested**: Ensure examples and instructions actually work

### Critical Rule: Fix Before Proceeding
**NEVER proceed to the next task if the current step has compilation errors or issues.** 

When implementing any feature:
1. **Fix ALL compilation errors** before moving forward
2. **Ensure tests pass** for the current implementation
3. **Verify functionality works** with manual testing if needed
4. **Only then** proceed to the next task in the todo list

This prevents cascading issues and maintains code quality throughout development.

### Warning Policy: Treat Warnings as Errors
**All compiler warnings must be resolved before proceeding.** This includes:

1. **Unused imports** - Remove or use conditional compilation attributes
2. **Dead code warnings** - Remove unused code or add `#[allow(dead_code)]` if intentional
3. **Clippy warnings** - Follow clippy suggestions or add targeted allows with justification
4. **Any other warnings** - Address root cause rather than suppress

#### Enforcement Commands
```bash
# Check for warnings (should produce no output)
cargo check 2>&1 | grep warning && echo "❌ Warnings found" || echo "✅ No warnings"

# Build with warnings as errors for CI
cargo build --workspace -- -D warnings

# Clippy with warnings as errors
cargo clippy --workspace -- -D warnings
```

#### When to Allow Warnings
Only use `#[allow(...)]` attributes when:
- Intentional design decision (document why)
- External crate compatibility requirements
- Generated code that can't be modified

Always include a comment explaining why the warning is allowed.

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
- **5 Working Examples**: All examples compile and run successfully
  - `basic_usage.rs` - Demonstrates core AST extraction
  - `documentation_generator.rs` - Shows markdown documentation generation
  - `code_analyzer.rs` - Quality analysis with complexity metrics
  - `llm_data_prep.rs` - LLM training data preparation
  - `rag_output_demo.rs` - RAG format demonstrations

### Running Examples
```bash
# From rustex-core crate
cargo run --example basic_usage
cargo run --example documentation_generator
cargo run --example code_analyzer
cargo run --example llm_data_prep

# From rustex-formats crate
cd crates/rustex-formats
cargo run --example rag_output_demo
```

### Fixing Compilation Issues
When encountering compilation errors:
1. **Identify the root cause** - read error messages carefully
2. **Fix one error at a time** - don't try to fix multiple issues simultaneously
3. **Test after each fix** - ensure each fix doesn't break other things
4. **Update related code** - fix cascading issues from the changes

The project is currently in early development phase with core parsing functionality implemented but many advanced features still in planning/development stage.