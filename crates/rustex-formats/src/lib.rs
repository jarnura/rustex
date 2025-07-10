//! # RustEx Formats
//!
//! Output formatters for rustex AST extraction, including specialized formats
//! for LLM training, RAG systems, and various documentation formats.

pub mod formatters;
pub mod rag;

// Re-export main formatting functions
pub use formatters::{
    format_project_ast, format_as_markdown, format_as_graphql_schema,
    create_rag_formatter, format_as_rag_with_config, format_as_rag_jsonl,
    format_for_embeddings,
};

// Re-export RAG-specific types and functions
pub use rag::{
    RagDocument, RagFormatter, RagConfig, RagChunk, RagMetadata, RagSemantics,
    ChunkMetadata, DocumentationQuality, EmbeddingStrategy, SemanticDepth,
    TrainingExample, TaskType, DifficultyLevel, EmbeddingInput,
    format_as_json, format_as_jsonl,
};
