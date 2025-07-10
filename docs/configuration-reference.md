# RustEx Configuration Reference

This document provides a comprehensive reference for all RustEx configuration options, including TOML configuration files, CLI arguments, and environment variables.

## Table of Contents

- [Configuration Sources](#configuration-sources)
- [Configuration File Structure](#configuration-file-structure)
- [Extraction Settings](#extraction-settings)
- [File Filtering](#file-filtering)
- [Output Configuration](#output-configuration)
- [Plugin Configuration](#plugin-configuration)
- [Use Case Templates](#use-case-templates)
- [CLI Overrides](#cli-overrides)
- [Environment Variables](#environment-variables)
- [Validation](#validation)
- [Examples](#examples)

## Configuration Sources

RustEx loads configuration from multiple sources in the following priority order (highest to lowest):

1. **CLI arguments** - Direct command-line flags
2. **Explicit config file** - File specified with `--config` flag
3. **Project config file** - `rustex.toml` in project root
4. **Hidden project config** - `.rustex.toml` in project root
5. **User config** - `~/.config/rustex/config.toml` (Unix) or `%APPDATA%\rustex\config.toml` (Windows)
6. **Default values** - Built-in defaults

## Configuration File Structure

Basic TOML structure:

```toml
[extraction]
# Core extraction settings

[filters]
# File filtering rules

[output]
# Output formatting options

[plugins]
# Plugin configuration

[advanced]
# Advanced/experimental features
```

## Extraction Settings

### `[extraction]` Section

Core settings that control how AST extraction is performed.

```toml
[extraction]
# Include documentation comments in output
include_docs = true

# Include private items (functions, structs, etc.)
include_private = false

# Parse project dependencies
parse_dependencies = false

# Maximum file size to process (in bytes or with units)
max_file_size = "10MB"  # Also accepts: 10485760, "10MiB", "5GB"

# Default output format
output_format = "json"  # Options: json, markdown, messagepack, graphql, rag

# Include inline comments (// comments within code)
include_inline_comments = false

# Extract macro definitions and invocations
include_macros = true

# Follow symbolic links
follow_symlinks = false

# Parse test modules and functions
include_tests = false

# Extract const and static declarations
include_constants = true

# Include type aliases
include_type_aliases = true
```

#### Data Types and Validation

- **`include_docs`**: `boolean` - Default: `true`
- **`include_private`**: `boolean` - Default: `false`  
- **`parse_dependencies`**: `boolean` - Default: `false`
- **`max_file_size`**: `string | integer` - Default: `"10MB"`
  - Accepts: raw bytes (`10485760`), or units (`"10MB"`, `"5MiB"`, `"1GB"`)
  - Maximum: `"100MB"`
- **`output_format`**: `string` - Default: `"json"`
  - Options: `"json"`, `"markdown"`, `"messagepack"`, `"graphql"`, `"rag"`
- **`include_inline_comments`**: `boolean` - Default: `false`
- **`include_macros`**: `boolean` - Default: `true`
- **`follow_symlinks`**: `boolean` - Default: `false`
- **`include_tests`**: `boolean` - Default: `false`
- **`include_constants`**: `boolean` - Default: `true`
- **`include_type_aliases`**: `boolean` - Default: `true`

## File Filtering

### `[filters]` Section

Control which files are processed during extraction.

```toml
[filters]
# Glob patterns for files to include
include = [
    "src/**/*.rs",
    "lib/**/*.rs",
    "examples/**/*.rs"
]

# Glob patterns for files to exclude  
exclude = [
    "target/**",
    "tests/**",
    "benches/**",
    "**/*_test.rs",
    "**/generated/**"
]

# Minimum file size to process (filters out empty files)
min_file_size = 1

# Maximum depth for directory traversal
max_depth = 10

# Include hidden files and directories (starting with .)
include_hidden = false

# Case-sensitive pattern matching
case_sensitive = false
```

#### Pattern Syntax

RustEx uses standard glob patterns:

- `*` - Matches any number of characters except `/`
- `**` - Matches any number of characters including `/`
- `?` - Matches exactly one character except `/`
- `[abc]` - Matches any character in the set
- `[!abc]` - Matches any character not in the set
- `{a,b,c}` - Matches any of the patterns (requires brace expansion)

#### Examples

```toml
[filters]
# Include only library code
include = ["src/lib.rs", "src/lib/**/*.rs"]

# Exclude test and build artifacts
exclude = [
    "target/**",
    "**/*_test.rs", 
    "**/test_*.rs",
    "tests/**",
    "benches/**"
]

# Include specific modules
include = [
    "src/api/**/*.rs",
    "src/core/**/*.rs",
    "src/utils.rs"
]

# Exclude internal and experimental code
exclude = [
    "src/internal/**",
    "src/experimental/**",
    "**/*_internal.rs"
]
```

## Output Configuration

### `[output]` Section

Control output formatting and behavior.

```toml
[output]
# Pretty-print JSON output
pretty_print = true

# Include file path in output
include_file_paths = true

# Include source code snippets
include_source_snippets = false

# Maximum length for source snippets
max_snippet_length = 200

# Include metrics in output
include_metrics = true

# Include cross-references between elements
include_cross_references = true

# Include element hierarchy information
include_hierarchy = true

# Timestamp format for extracted_at field
timestamp_format = "iso8601"  # Options: iso8601, unix, rfc3339

# Character encoding for text output
encoding = "utf-8"

# Line ending style for text output
line_endings = "unix"  # Options: unix (\n), windows (\r\n), mac (\r)
```

### Format-Specific Settings

#### JSON Output

```toml
[output.json]
# Compact vs pretty-printed JSON
compact = false

# Include null fields
include_nulls = false

# Escape non-ASCII characters
escape_unicode = false

# Maximum nesting depth
max_depth = 50
```

#### Markdown Output

```toml
[output.markdown]
# Include table of contents
include_toc = true

# TOC depth (heading levels)
toc_depth = 3

# Include syntax highlighting in code blocks
syntax_highlighting = true

# Markdown dialect
dialect = "github"  # Options: github, commonmark, kramdown

# Include file paths as headers
file_headers = true

# Include metrics summary
include_summary = true
```

#### RAG Output

```toml
[output.rag]
# Chunking strategy for code segmentation
chunking_strategy = "semantic"  # Options: fixed, semantic, adaptive, function_based

# Maximum chunk size in tokens
max_chunk_size = 1000

# Overlap between chunks (in tokens)
chunk_overlap = 100

# Minimum chunk size (avoid tiny chunks)
min_chunk_size = 50

# Embedding strategy for optimization
embedding_strategy = "combined"  # Options: code_only, doc_only, combined, specialized

# Include semantic analysis and concept extraction
include_semantics = true

# Include training examples generation
include_training_examples = true

# Include quality assessment metrics
include_quality_assessment = true

# Context window size for each chunk
context_window = 512

# Maximum number of training examples to generate
max_training_examples = 1000

# Difficulty levels for training examples
difficulty_levels = ["beginner", "intermediate", "advanced"]

# Include embedding vectors (when available)
include_embeddings = false

# Base64 encode binary data
base64_encode_binary = true

# Include cross-references in chunks
include_cross_references = true

# Semantic analysis configuration
[output.rag.semantic]
# Extract concepts from code and documentation
extract_concepts = true

# Analyze API surface area
analyze_api_surface = true

# Generate complexity distribution
complexity_distribution = true

# Maximum concepts to extract per chunk
max_concepts_per_chunk = 10

# Concept extraction confidence threshold
concept_confidence_threshold = 0.7

# Training example configuration
[output.rag.training]
# Generate function implementation examples
function_examples = true

# Generate documentation examples
documentation_examples = true

# Generate code explanation examples
explanation_examples = true

# Generate refactoring examples
refactoring_examples = false

# Include metadata in training examples
include_metadata = true

# Quality assessment configuration
[output.rag.quality]
# Assess documentation quality
documentation_quality = true

# Assess code complexity appropriateness
complexity_assessment = true

# Assess API design quality
api_quality = true

# Include quality scores in output
include_scores = true
```

## Plugin Configuration

### `[plugins]` Section

Configure plugin system and built-in plugins.

```toml
[plugins]
# List of enabled plugins
enabled = [
    "complexity-analyzer",
    "llm-optimizer", 
    "doc-enhancer"
]

# Plugin execution order
execution_order = [
    "pre-process",
    "extract", 
    "analyze",
    "post-process"
]

# Plugin-specific configurations
[plugins.complexity-analyzer]
# Enable different complexity metrics
cyclomatic = true
cognitive = true
halstead = true
nesting = true

# Thresholds for warnings
warning_threshold = 10
error_threshold = 20

[plugins.llm-optimizer]
# Target model for optimization
target_model = "gpt-4"

# Maximum context length
max_context_length = 8000

# Include training hints
include_training_hints = true

# Chunk overlap strategy
overlap_strategy = "semantic"

[plugins.doc-enhancer]
# Documentation style
style = "rust-standard"  # Options: rust-standard, google, jsdoc

# Minimum documentation coverage
min_coverage = 0.8

# Auto-generate missing documentation
auto_generate = false

# Include examples in documentation
include_examples = true
```

## Use Case Templates

Pre-configured templates for common scenarios.

### Documentation Generation

```bash
rustex config init --template documentation
```

```toml
[extraction]
include_docs = true
include_private = true
parse_dependencies = false
output_format = "markdown"
include_tests = false

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**", "benches/**"]

[output]
pretty_print = true
include_metrics = true

[plugins]
enabled = ["doc-enhancer"]

[plugins.doc-enhancer]
style = "rust-standard"
min_coverage = 0.8
include_examples = true
```

### Code Analysis

```bash
rustex config init --template code-analysis
```

```toml
[extraction]
include_docs = true
include_private = true
parse_dependencies = true
output_format = "json"
include_tests = true

[filters]
include = ["src/**/*.rs", "lib/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**"]

[output]
pretty_print = true
include_metrics = true
include_cross_references = true
include_hierarchy = true

[plugins]
enabled = ["complexity-analyzer", "metrics"]

[plugins.complexity-analyzer]
cyclomatic = true
cognitive = true
halstead = true
warning_threshold = 10
error_threshold = 20
```

### LLM Training

```bash
rustex config init --template llm-training
```

```toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = true
output_format = "rag"
include_tests = false

[filters]
include = ["src/**/*.rs", "lib/**/*.rs", "examples/**/*.rs"]
exclude = ["target/**", "tests/**", "benches/**"]

[output]
include_metrics = false
include_source_snippets = true
max_snippet_length = 500

[plugins]
enabled = ["llm-optimizer"]

[plugins.llm-optimizer]
target_model = "gpt-4"
max_context_length = 8000
chunk_overlap = 100
include_training_hints = true

[output.rag]
max_chunk_size = 1000
chunk_overlap = 100
include_semantics = true
include_context = true
```

### Testing

```bash
rustex config init --template testing
```

```toml
[extraction]
include_docs = false
include_private = true
parse_dependencies = false
output_format = "json"
include_tests = true

[filters]
include = ["src/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**", "benches/**"]

[output]
pretty_print = false
include_metrics = true

[plugins]
enabled = []
```

## CLI Overrides

Command-line arguments override configuration file settings:

```bash
# Override output format
rustex extract --format markdown

# Override include settings
rustex extract --include-docs --include-private

# Override file filters  
rustex extract --include "src/**/*.rs" --exclude "tests/**"

# Override file size limit
rustex extract --max-file-size 20MB

# Override plugins
rustex extract --plugins "complexity,llm-optimizer"

# Use specific config file
rustex extract --config custom-config.toml
```

### CLI Flag Reference

| Flag | Config Equivalent | Description |
|------|------------------|-------------|
| `--format` | `output_format` | Output format |
| `--include-docs` | `include_docs` | Include documentation |
| `--include-private` | `include_private` | Include private items |
| `--parse-deps` | `parse_dependencies` | Parse dependencies |
| `--max-file-size` | `max_file_size` | Maximum file size |
| `--include` | `filters.include` | Include patterns |
| `--exclude` | `filters.exclude` | Exclude patterns |
| `--plugins` | `plugins.enabled` | Enable plugins |
| `--pretty` | `output.pretty_print` | Pretty-print output |

## Environment Variables

Environment variables provide another configuration layer:

```bash
# Set default output format
export RUSTEX_OUTPUT_FORMAT=json

# Set default include docs
export RUSTEX_INCLUDE_DOCS=true

# Set default config file location
export RUSTEX_CONFIG_FILE=~/.rustex/config.toml

# Set plugin directory
export RUSTEX_PLUGIN_DIR=~/.rustex/plugins

# Enable debug logging
export RUSTEX_LOG_LEVEL=debug

# Set maximum memory usage
export RUSTEX_MAX_MEMORY=2GB
```

### Environment Variable Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `RUSTEX_OUTPUT_FORMAT` | Default output format | `json` |
| `RUSTEX_INCLUDE_DOCS` | Include documentation by default | `true` |
| `RUSTEX_INCLUDE_PRIVATE` | Include private items by default | `false` |
| `RUSTEX_CONFIG_FILE` | Default config file location | Platform-specific |
| `RUSTEX_PLUGIN_DIR` | Plugin directory | `~/.rustex/plugins` |
| `RUSTEX_LOG_LEVEL` | Logging level | `info` |
| `RUSTEX_MAX_MEMORY` | Maximum memory usage | `1GB` |
| `RUSTEX_CACHE_DIR` | Cache directory | Platform-specific |
| `RUSTEX_NO_COLOR` | Disable colored output | `false` |

## Validation

### Automatic Validation

RustEx automatically validates configuration:

```bash
# Validate current configuration
rustex config validate

# Validate specific file
rustex config validate --file custom-config.toml
```

### Manual Validation in Code

```rust
use rustex_core::ExtractorConfig;

let config = ExtractorConfig::from_toml_file("rustex.toml")?;

// Validate configuration
match config.validate() {
    Ok(()) => println!("Configuration is valid"),
    Err(e) => eprintln!("Configuration error: {}", e),
}
```

### Common Validation Errors

1. **Invalid file size**: Must be between 1KB and 100MB
2. **Invalid output format**: Must be one of: json, markdown, messagepack, graphql, rag
3. **Empty include patterns**: At least one include pattern required
4. **Conflicting patterns**: Include and exclude patterns overlap
5. **Invalid plugin names**: Plugin not found or invalid
6. **Invalid file paths**: Config file paths must be absolute

## Examples

### Minimal Configuration

```toml
# Minimal rustex.toml
[extraction]
include_docs = true

[filters]
include = ["src/**/*.rs"]
```

### Documentation Project

```toml
# Configuration for generating project documentation
[extraction]
include_docs = true
include_private = true
output_format = "markdown"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["tests/**", "benches/**", "examples/**"]

[output]
pretty_print = true
include_metrics = true

[output.markdown]
include_toc = true
toc_depth = 3
syntax_highlighting = true
include_summary = true

[plugins]
enabled = ["doc-enhancer"]

[plugins.doc-enhancer]
style = "rust-standard"
auto_generate = false
include_examples = true
```

### Large Project Configuration

```toml
# Configuration for large codebases
[extraction]
include_docs = false
include_private = false
parse_dependencies = false
max_file_size = "5MB"

[filters]
include = ["src/**/*.rs"]
exclude = [
    "target/**",
    "tests/**", 
    "benches/**",
    "**/*_generated.rs",
    "vendor/**"
]
max_depth = 8
min_file_size = 100

[output]
pretty_print = false
include_metrics = false
include_cross_references = false

[plugins]
enabled = []
```

### CI/CD Configuration

```toml
# Configuration optimized for CI/CD
[extraction]
include_docs = true
include_private = false
parse_dependencies = false
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**"]

[output]
pretty_print = false
include_metrics = true
include_file_paths = false

[plugins]
enabled = ["complexity-analyzer"]

[plugins.complexity-analyzer]
warning_threshold = 10
error_threshold = 15
```

### Development Configuration

```toml
# Configuration for development and debugging
[extraction]
include_docs = true
include_private = true
parse_dependencies = true
include_tests = true
include_inline_comments = true

[filters]
include = ["src/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**"]

[output]
pretty_print = true
include_metrics = true
include_cross_references = true
include_hierarchy = true
include_source_snippets = true
max_snippet_length = 300

[plugins]
enabled = ["complexity-analyzer", "doc-enhancer"]
```

## Best Practices

1. **Start with templates**: Use `rustex config init --template <use-case>` for common scenarios
2. **Validate regularly**: Run `rustex config validate` after changes
3. **Use environment variables**: For CI/CD and automated environments
4. **Version control**: Include `rustex.toml` in your repository
5. **Document overrides**: Comment why specific settings are used
6. **Test configurations**: Run extractions with different configs to ensure they work
7. **Use filters effectively**: Exclude unnecessary files to improve performance
8. **Monitor file sizes**: Large files can slow down extraction significantly

---

For more configuration examples, see the [examples directory](../examples/) in the repository.