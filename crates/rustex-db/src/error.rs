//! Error types for the database layer.

use thiserror::Error;

/// Result type alias for database operations.
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Database-specific errors.
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// SQL execution error
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Graph traversal error
    #[error("Graph traversal error: {0}")]
    GraphTraversal(String),

    /// Data integrity error
    #[error("Data integrity error: {0}")]
    DataIntegrity(String),

    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Generic error
    #[error("Database error: {0}")]
    Generic(String),
}

impl DatabaseError {
    /// Create a new migration error.
    pub fn migration<S: Into<String>>(msg: S) -> Self {
        Self::Migration(msg.into())
    }

    /// Create a new connection error.
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a new graph traversal error.
    pub fn graph_traversal<S: Into<String>>(msg: S) -> Self {
        Self::GraphTraversal(msg.into())
    }

    /// Create a new data integrity error.
    pub fn data_integrity<S: Into<String>>(msg: S) -> Self {
        Self::DataIntegrity(msg.into())
    }

    /// Create a new not found error.
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new validation error.
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a generic error.
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
}

impl From<anyhow::Error> for DatabaseError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

/// Database configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid database URL
    #[error("Invalid database URL: {0}")]
    InvalidUrl(String),

    /// Missing configuration parameter
    #[error("Missing configuration parameter: {0}")]
    MissingParameter(String),

    /// Invalid configuration value
    #[error("Invalid configuration value for {param}: {value}")]
    InvalidValue { param: String, value: String },
}

impl From<ConfigError> for DatabaseError {
    fn from(err: ConfigError) -> Self {
        Self::Config(err.to_string())
    }
}