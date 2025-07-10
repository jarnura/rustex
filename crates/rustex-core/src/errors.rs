//! Error types and handling for RustEx.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for RustEx operations.
#[derive(Error, Debug)]
pub enum RustExError {
    /// IO errors during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Syntax parsing errors
    #[error("Failed to parse Rust syntax in {file}: {error}")]
    SyntaxError { file: PathBuf, error: syn::Error },

    /// File size limit exceeded
    #[error("File too large: {file} ({size} bytes, limit: {limit} bytes)")]
    FileTooLarge {
        file: PathBuf,
        size: usize,
        limit: usize,
    },

    /// Project root not found or invalid
    #[error("Invalid project root: {path}")]
    InvalidProjectRoot { path: PathBuf },

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Multiple file processing errors
    #[error("Failed to process {failed_count} out of {total_count} files")]
    PartialFailure {
        failed_count: usize,
        total_count: usize,
        errors: Vec<FileProcessingError>,
    },
}

/// Specific error for file processing failures.
#[derive(Error, Debug, Clone)]
pub enum FileProcessingError {
    #[error("Parse error in {file}: {error}")]
    ParseError { file: PathBuf, error: String },

    #[error("IO error reading {file}: {error}")]
    IoError { file: PathBuf, error: String },

    #[error("File too large: {file} ({size} bytes)")]
    TooLarge { file: PathBuf, size: usize },

    #[error("Access denied: {file}")]
    AccessDenied { file: PathBuf },
}

impl FileProcessingError {
    pub fn file_path(&self) -> &PathBuf {
        match self {
            FileProcessingError::ParseError { file, .. } => file,
            FileProcessingError::IoError { file, .. } => file,
            FileProcessingError::TooLarge { file, .. } => file,
            FileProcessingError::AccessDenied { file } => file,
        }
    }
}

/// Result type for RustEx operations.
pub type Result<T> = std::result::Result<T, RustExError>;

/// Result type for file processing operations.
pub type FileResult<T> = std::result::Result<T, FileProcessingError>;
