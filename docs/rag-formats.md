# RAG-Optimized Output Formats

RustEx provides specialized output formats optimized for Retrieval-Augmented Generation (RAG) systems and Large Language Model (LLM) applications. These formats are designed to maximize the utility of Rust code for AI/ML workflows.

## Overview

The RAG output format transforms extracted AST data into structures optimized for:

- **Embedding Models**: Chunked text optimized for vector embeddings
- **Retrieval Systems**: Semantic metadata for efficient search and ranking
- **LLM Training**: Structured training examples and fine-tuning data
- **Knowledge Graphs**: Semantic relationships and concept hierarchies

## Key Features

### 1. Intelligent Chunking

```rust
use rustex_formats::{RagConfig, RagFormatter, SemanticDepth};

let config = RagConfig {
    target_chunk_size: 512,      // Optimal for embedding models
    max_chunk_size: 1024,        // Hard limit
    chunk_overlap: 50,           // Context preservation
    semantic_analysis_depth: SemanticDepth::Deep,
    ..Default::default()
};

let formatter = RagFormatter::new(config);
let rag_doc = formatter.format(&project_ast)?;
```

### 2. Multiple Output Formats

#### Standard RAG JSON
```bash
rustex extract --format rag --output rag-data.json --pretty
```

#### JSONL for Streaming
```bash
rustex extract --format rag --output chunks.jsonl
```

#### Embedding-Optimized
```rust
let embedding_inputs = format_for_embeddings(&project_ast)?;
for input in embedding_inputs {
    // Send to embedding model
    let embedding = embed_text(&input.text)?;
    store_embedding(&input.id, embedding, &input.metadata)?;
}
```

## Document Structure

### RAG Document Schema

```json
{
  "metadata": {
    "project_name": "my-project",
    "total_chunks": 150,
    "total_tokens": 75000,
    "semantic_categories": ["function_definition", "data_structure"],
    "complexity_distribution": {
      "simple": 45,
      "moderate": 78,
      "complex": 27
    }
  },
  "chunks": [...],
  "semantics": {...},
  "training_examples": [...]
}
```

### Chunk Structure

Each chunk contains:

```json
{
  "id": "chunk_1",
  "content": "/// Calculate fibonacci number\npub fn fibonacci(n: u32) -> u64 {...}",
  "content_with_context": "// File: src/math.rs\n// Module: crate::math\n\n...",
  "metadata": {
    "element_type": "Function",
    "element_name": "fibonacci",
    "qualified_name": "crate::math::fibonacci",
    "token_count": 45,
    "complexity": 3,
    "semantic_category": "function_definition",
    "embedding_strategy": "Combined",
    "retrieval_keywords": ["fibonacci", "recursive", "math"],
    "documentation_quality": "Good"
  },
  "semantic_hash": "a1b2c3d4"
}
```

## Embedding Strategies

RustEx automatically selects optimal embedding strategies:

### Combined Strategy
Best for most use cases - embeds code and documentation together.

```rust
/// Calculate the area of a circle
/// 
/// # Examples
/// ```
/// assert_eq!(circle_area(2.0), std::f64::consts::PI * 4.0);
/// ```
pub fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}
```

### Code-Only Strategy
For complex algorithms where implementation details matter more than documentation.

### Documentation-Only Strategy
For well-documented APIs where the interface is more important than implementation.

### Specialized Strategy
For domain-specific content requiring custom processing.

## Semantic Analysis

### Concept Hierarchy
```json
{
  "concept_hierarchy": [
    {
      "id": "math_module",
      "name": "Mathematical Operations",
      "concept_type": "Module",
      "related_chunks": ["chunk_1", "chunk_5", "chunk_12"],
      "importance_score": 0.85
    }
  ]
}
```

### Relationships
```json
{
  "relationships": [
    {
      "from_chunk": "chunk_1",
      "to_chunk": "chunk_5", 
      "relationship_type": "Uses",
      "strength": 0.7,
      "description": "fibonacci function uses helper_function"
    }
  ]
}
```

### API Surface Analysis
```json
{
  "api_surface": {
    "public_functions": [...],
    "complexity_metrics": {
      "total_public_items": 45,
      "avg_parameter_count": 2.3,
      "documentation_coverage": 0.87
    }
  }
}
```

## Training Examples

RustEx generates various types of training examples:

### Code Explanation
```json
{
  "input": "Explain what this Rust function does:",
  "output": "This function calculates the nth Fibonacci number using recursion...",
  "task_type": "CodeExplanation",
  "difficulty": "Intermediate"
}
```

### Code Completion
```json
{
  "input": "Complete this function signature:\npub fn fibonacci(n: u32) ->", 
  "output": "u64",
  "task_type": "CodeCompletion",
  "difficulty": "Beginner"
}
```

### API Usage
```json
{
  "input": "How do you use the fibonacci function?",
  "output": "```rust\nlet result = fibonacci(10);\nprintln!(\"10th Fibonacci: {}\", result);\n```",
  "task_type": "ApiUsage",
  "difficulty": "Beginner"
}
```

## Configuration Options

### Chunking Configuration
```toml
[extraction]
output_format = "rag"

[output.rag]
target_chunk_size = 512
max_chunk_size = 1024
min_chunk_size = 100
chunk_overlap = 50

# Content filtering
include_private_items = false
include_test_code = false
min_complexity_for_inclusion = 2
min_documentation_quality = "Basic"

# Analysis depth
semantic_analysis_depth = "Standard"  # Basic, Standard, Deep

# Training data
generate_training_examples = true
max_training_examples_per_chunk = 3
```

### Quality Thresholds
```toml
[quality_gates]
min_documentation_quality = "Good"
min_complexity_for_inclusion = 3
max_function_complexity = 15
```

## Use Cases

### 1. Vector Database Population

```rust
use rustex_formats::{format_for_embeddings, EmbeddingInput};

let inputs = format_for_embeddings(&project_ast)?;
for input in inputs {
    let embedding = embed_text(&input.text).await?;
    
    vector_db.insert(VectorRecord {
        id: input.id,
        embedding,
        metadata: input.metadata,
        content: input.text,
    }).await?;
}
```

### 2. LLM Fine-tuning Dataset

```rust
let rag_doc = formatter.format(&project_ast)?;

// Convert to training format
let training_data: Vec<TrainingRecord> = rag_doc.training_examples
    .into_iter()
    .map(|example| TrainingRecord {
        instruction: example.input,
        output: example.output,
        difficulty: example.difficulty,
    })
    .collect();

// Save as JSONL for training
let jsonl = training_data.iter()
    .map(|record| serde_json::to_string(record))
    .collect::<Result<Vec<_>, _>>()?
    .join("\n");

std::fs::write("training_data.jsonl", jsonl)?;
```

### 3. Knowledge Graph Construction

```rust
let semantics = &rag_doc.semantics;

// Build graph nodes
for concept in &semantics.concept_hierarchy {
    graph.add_node(GraphNode {
        id: concept.id.clone(),
        label: concept.name.clone(),
        node_type: concept.concept_type,
        importance: concept.importance_score,
    });
}

// Build graph edges
for relationship in &semantics.relationships {
    graph.add_edge(GraphEdge {
        from: relationship.from_chunk.clone(),
        to: relationship.to_chunk.clone(),
        edge_type: relationship.relationship_type,
        weight: relationship.strength,
    });
}
```

### 4. Retrieval System Integration

```rust
use rustex_formats::{RagChunk, ChunkMetadata};

impl SearchIndex for RustCodeIndex {
    fn search(&self, query: &str, filters: &SearchFilters) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        for chunk in &self.chunks {
            let score = self.calculate_relevance(query, chunk);
            
            if score > 0.5 && self.matches_filters(chunk, filters) {
                results.push(SearchResult {
                    chunk_id: chunk.id.clone(),
                    score,
                    snippet: self.generate_snippet(chunk, query),
                    metadata: chunk.metadata.clone(),
                });
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }
}
```

## Best Practices

### 1. Chunking Strategy Selection

- **Small chunks (128-256 tokens)**: Embedding models, fine-grained search
- **Medium chunks (512-768 tokens)**: General-purpose RAG applications  
- **Large chunks (1024+ tokens)**: Context-aware models, complex reasoning

### 2. Quality Filtering

```rust
let config = RagConfig {
    min_documentation_quality: DocumentationQuality::Good,
    min_complexity_for_inclusion: Some(3),
    include_private_items: false,  // Focus on public API
    include_test_code: false,      // Exclude test artifacts
    ..Default::default()
};
```

### 3. Semantic Depth Selection

- **Basic**: Fast processing, minimal relationships
- **Standard**: Balanced analysis with key relationships
- **Deep**: Comprehensive analysis with patterns and concepts

### 4. Training Data Quality

- Filter examples by difficulty level for targeted training
- Balance different task types (explanation, completion, usage)
- Include diverse complexity levels for robust model training

## Performance Considerations

### Memory Usage

Large codebases can generate substantial RAG documents. Consider:

```rust
// Process in batches for large projects
let config = RagConfig {
    max_chunk_size: 512,  // Smaller chunks for memory efficiency
    semantic_analysis_depth: SemanticDepth::Basic,  // Reduce analysis overhead
    generate_training_examples: false,  // Skip if not needed
    ..Default::default()
};
```

### Processing Speed

- Use `SemanticDepth::Basic` for faster processing
- Disable training example generation if not needed
- Filter content early using quality thresholds

### Storage Optimization

```bash
# Compressed storage for large datasets
rustex extract --format rag --output data.json
gzip data.json

# JSONL for streaming processing
rustex extract --format rag --output chunks.jsonl
# Process line by line without loading entire file
```

## Integration Examples

See the `/examples` directory for complete integration examples:

- `rag_output_demo.rs` - Comprehensive RAG format demonstration
- `llm_data_prep.rs` - LLM training data preparation
- `code_analyzer.rs` - Quality analysis with RAG output

## Advanced Features

### Custom Embedding Strategies

```rust
impl RagFormatter {
    pub fn with_custom_strategy<F>(&mut self, strategy: F) 
    where F: Fn(&CodeElement) -> EmbeddingStrategy {
        // Custom strategy implementation
    }
}
```

### Plugin Integration

```rust
// Custom semantic analysis plugin
let config = RagConfig {
    semantic_analysis_depth: SemanticDepth::Deep,
    plugins: vec!["custom-semantic-analyzer".to_string()],
    ..Default::default()
};
```

## Troubleshooting

### Common Issues

1. **Large memory usage**: Reduce chunk size or use Basic semantic depth
2. **Poor embedding quality**: Increase chunk overlap or improve documentation
3. **Missing relationships**: Use deeper semantic analysis or better cross-references
4. **Low-quality training data**: Increase documentation quality thresholds

### Debugging

```bash
# Verbose output for debugging
RUST_LOG=debug rustex extract --format rag --verbose

# Validate chunk sizes
rustex extract --format rag | jq '.metadata.chunk_size_stats'

# Check semantic categories
rustex extract --format rag | jq '.metadata.semantic_categories'
```