# RustEx Benchmark Suite

This directory contains comprehensive performance benchmarks for the RustEx AST extraction library.

## Overview

The benchmark suite measures performance across all critical aspects of RustEx:

- **AST Parsing**: Raw syn parsing performance with different code complexities
- **Complexity Calculation**: Our sophisticated complexity algorithms (cyclomatic, cognitive, Halstead)
- **Full Extraction**: End-to-end project extraction workflow  
- **Visitor Performance**: AST visitor pattern efficiency
- **Output Formatting**: Serialization to different formats (JSON, MessagePack, Markdown, RAG)
- **File Filtering**: Glob pattern matching and file discovery
- **Memory Usage**: Allocation patterns and memory efficiency
- **Scalability**: Performance with different project sizes

## Running Benchmarks

### Quick Test
```bash
cargo bench --bench benchmarks -- --quick
```

### Full Benchmark Suite
```bash
cargo bench --bench benchmarks
```

### Specific Benchmark Groups
```bash
# Test only AST parsing performance
cargo bench --bench benchmarks ast_parsing

# Test only complexity calculation
cargo bench --bench benchmarks complexity_calculation

# Test full extraction workflow
cargo bench --bench benchmarks full_extraction
```

### Profiling Mode
```bash
# Run with profiling for detailed analysis
cargo bench --bench benchmarks -- --profile-time=5
```

## Benchmark Results

### Performance Baselines (Typical Results)

#### AST Parsing Performance
- **Simple Functions**: ~25 µs (8.7 MiB/s throughput)
- **Complex Functions**: ~150 µs (17.6 MiB/s throughput)  
- **Large Structs**: ~440 µs (13.7 MiB/s throughput)
- **Complex Enums**: ~140 µs (14.5 MiB/s throughput)
- **Large Traits**: ~1.2 ms (11.1 MiB/s throughput)
- **Real-world Files**: ~580 µs (12.9 MiB/s throughput)

#### Complexity Calculation Performance
- **Simple Functions**: ~625 ns
- **Complex Functions**: ~8 µs
- **Structural Items**: ~16-82 ns

#### Full Project Extraction
- **7-file Test Project**: ~3.4 ms

#### Visitor Performance
- **Simple Code**: ~5 µs (590K elements/s)
- **Complex Code**: ~26 µs (38K elements/s)
- **Large Structures**: ~19 µs (103K elements/s)

#### Output Formatting
- **JSON**: ~16 µs (440K elements/s)
- **MessagePack**: ~12 µs (595K elements/s) 
- **Markdown**: ~77 ns (91M elements/s)
- **RAG**: ~3 µs (2.4M elements/s)

#### Scalability
- **1 file**: ~750 µs
- **5 files**: ~2.9 ms
- **10 files**: ~5.5 ms
- **25 files**: ~13 ms
- **50 files**: ~26 ms

## Performance Analysis

### Key Insights

1. **Parsing Performance**: Scales linearly with code size, with good throughput (8-18 MiB/s)
2. **Complexity Calculation**: Extremely fast (sub-microsecond for most cases)
3. **Memory Efficiency**: Linear scaling with project size
4. **Output Format Efficiency**: MessagePack > JSON > RAG > Markdown

### Performance Characteristics

- **CPU Bound**: Parsing and complexity calculation are CPU-intensive
- **Memory Efficient**: Low memory overhead, good allocation patterns
- **Linear Scaling**: Performance scales predictably with project size
- **Format Overhead**: Binary formats (MessagePack) significantly faster than text

### Optimization Opportunities

1. **Parallel Processing**: File parsing could benefit from parallelization
2. **Caching**: Repeated parsing of same files could be cached
3. **Streaming**: Large project extraction could use streaming
4. **Memory Pools**: Object reuse for reduced allocations

## Regression Testing

The benchmark suite serves as a regression testing framework:

### Performance Thresholds
- **Parsing Regression**: >10% slowdown
- **Complexity Regression**: >15% slowdown
- **Extraction Regression**: >20% slowdown
- **Formatting Regression**: >5% slowdown

### Continuous Monitoring
Use in CI/CD to detect performance regressions:

```bash
# Compare against baseline
cargo bench --bench benchmarks -- --save-baseline main

# Check for regressions
cargo bench --bench benchmarks -- --baseline main
```

## Hardware Considerations

Benchmark results vary by hardware:

### CPU Impact
- **Single-core Performance**: Critical for parsing speed
- **Memory Bandwidth**: Important for large projects
- **Cache Size**: Affects repeated processing

### Recommended Hardware
- **Development**: Modern multi-core CPU, 16GB+ RAM
- **CI/CD**: Consistent hardware for reproducible results
- **Production**: CPU-optimized instances for best performance

## Customization

### Adding New Benchmarks

1. Add benchmark function to `benches/benchmarks.rs`
2. Include in `criterion_group!` macro
3. Document expected performance characteristics

### Benchmark Configuration

Modify `benchmark-config.toml` to customize:
- Sample sizes
- Measurement duration
- Output formats
- Performance thresholds

### Test Data

The benchmark suite includes realistic test data:
- Simple functions (basic operations)
- Complex functions (nested control flow)
- Large data structures (50+ fields)
- Real-world patterns (production-like code)

## Troubleshooting

### Inconsistent Results
- Ensure stable system load
- Run multiple iterations
- Check for thermal throttling

### Memory Issues
- Monitor system memory during benchmarks
- Consider reducing sample sizes
- Check for memory leaks in test data

### Platform Differences
- Results vary between platforms
- Use relative comparisons
- Maintain platform-specific baselines

## Future Enhancements

1. **Parallel Benchmarks**: Test multi-threaded performance
2. **Memory Profiling**: Detailed allocation tracking  
3. **Cache Benchmarks**: Test with incremental parsing
4. **Network Benchmarks**: Remote file system performance
5. **Plugin Benchmarks**: Extensibility performance impact