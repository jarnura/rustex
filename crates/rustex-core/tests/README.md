# RustEx Core Tests

This directory contains comprehensive unit and integration tests for the RustEx core library.

## Test Structure

### `test_extractor.rs` - Integration Tests (11 tests)
- **Basic extraction**: Project-level AST extraction
- **Function extraction**: Public/private function handling
- **Struct extraction**: Struct parsing with attributes and docs
- **Enum extraction**: Enum variants and complexity
- **Trait extraction**: Trait definitions and methods
- **Visibility filtering**: Private vs public element filtering
- **Documentation extraction**: Doc comment parsing
- **Import extraction**: Use statement parsing
- **Metrics calculation**: Project and file metrics
- **Generic parameters**: Generic type extraction
- **Complexity calculation**: Function complexity analysis

### `test_visitors.rs` - Unit Tests (8 tests)
- **Function visitor**: Function extraction with signatures
- **Struct visitor**: Struct parsing with generic parameters
- **Enum visitor**: Enum variant complexity calculation
- **Trait visitor**: Trait method counting
- **Visibility filtering**: Include/exclude private items
- **Documentation extraction**: Doc comment processing
- **Attribute extraction**: Derive and other attributes
- **Nested items**: Module and impl block handling

### `test_errors.rs` - Error Handling Tests (7 tests)
- **Parse error handling**: Invalid syntax recovery
- **File size limits**: Large file handling
- **Missing project root**: Invalid path handling
- **High failure rate**: Partial failure thresholds
- **Error types**: Custom error structure validation
- **Recovery mechanisms**: Continued processing after errors

### `test_config.rs` - Configuration Tests (8 tests)
- **Default configuration**: Sensible defaults
- **Config modification**: Runtime configuration changes
- **Filter configuration**: Include/exclude patterns
- **Output formats**: All supported formats
- **Serialization**: JSON serialization/deserialization
- **Custom configs**: User-defined configurations
- **Config validation**: Edge case handling
- **Plugin configuration**: Plugin system setup

### `test_ast_data.rs` - Data Structure Tests (11 tests)
- **Project AST creation**: Complete project structure
- **Code element creation**: Individual element construction
- **Element types**: All supported AST node types
- **Visibility types**: Public/private/restricted visibility
- **Import information**: Use statement representation
- **Dependency information**: Project dependencies
- **Metrics calculation**: Statistical data computation
- **File metrics**: Per-file statistics
- **Serialization**: Data structure serialization
- **Code location**: Source position tracking
- **Default implementations**: Sensible defaults

## Test Coverage Summary

- **Total Tests**: 45 tests across 5 test files
- **Core Functionality**: ✅ Comprehensive AST extraction
- **Error Handling**: ✅ Robust error recovery
- **Configuration**: ✅ Flexible configuration system
- **Data Structures**: ✅ Complete data model validation
- **Integration**: ✅ End-to-end workflow testing

## Running Tests

```bash
# Run all core tests
cargo test --package rustex-core

# Run specific test file
cargo test --package rustex-core --test test_extractor

# Run with output
cargo test --package rustex-core -- --nocapture

# Run specific test
cargo test --package rustex-core test_basic_extraction
```

## Test Quality Features

- **Isolated environments**: Using `tempfile` for clean test environments
- **Comprehensive scenarios**: Real-world Rust code patterns
- **Error simulation**: Deliberate error injection for resilience testing
- **Edge case coverage**: Boundary conditions and unusual inputs
- **Integration testing**: Full extraction pipeline validation
- **Unit testing**: Individual component verification