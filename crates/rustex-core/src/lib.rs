//! # RustEx Core
//!
//! Core AST extraction library for Rust projects, optimized for LLM/RAG applications.

pub mod ast_data;
pub mod complexity;
pub mod config;
pub mod errors;
pub mod extractor;
pub mod visitors;

pub mod test_fixtures;

pub use ast_data::*;
pub use complexity::{ComplexityCalculator, ComplexityLevel, ComplexityMetrics, HalsteadMetrics};
pub use config::{ConfigUseCase, ExtractorConfig, FilterConfig, OutputFormat};
pub use errors::{FileProcessingError, FileResult, Result, RustExError};
pub use extractor::AstExtractor;
pub use visitors::CodeElementVisitor;
