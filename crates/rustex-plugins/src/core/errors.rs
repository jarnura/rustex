//! Plugin error types and handling.

use std::fmt;
use serde::{Deserialize, Serialize};

/// Errors that can occur during plugin execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginError {
    /// Plugin processing failed with a specific error message
    ProcessingFailed(String),
    
    /// Plugin configuration is invalid or missing required settings
    ConfigurationError(String),
    
    /// Plugin has missing dependencies
    DependencyMissing(String),
    
    /// Plugin initialization failed
    InitializationFailed(String),
    
    /// Plugin version incompatibility
    VersionIncompatible {
        required: String,
        found: String,
    },
    
    /// Plugin context is invalid for the operation
    InvalidContext(String),
    
    /// I/O error during plugin operation
    IoError(String),
    
    /// Serialization/deserialization error
    SerializationError(String),
    
    /// Generic plugin error with custom message
    Custom(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::ProcessingFailed(msg) => {
                write!(f, "Plugin processing failed: {}", msg)
            }
            PluginError::ConfigurationError(msg) => {
                write!(f, "Plugin configuration error: {}", msg)
            }
            PluginError::DependencyMissing(dep) => {
                write!(f, "Plugin dependency missing: {}", dep)
            }
            PluginError::InitializationFailed(msg) => {
                write!(f, "Plugin initialization failed: {}", msg)
            }
            PluginError::VersionIncompatible { required, found } => {
                write!(f, "Plugin version incompatible: required {}, found {}", required, found)
            }
            PluginError::InvalidContext(msg) => {
                write!(f, "Invalid plugin context: {}", msg)
            }
            PluginError::IoError(msg) => {
                write!(f, "Plugin I/O error: {}", msg)
            }
            PluginError::SerializationError(msg) => {
                write!(f, "Plugin serialization error: {}", msg)
            }
            PluginError::Custom(msg) => {
                write!(f, "Plugin error: {}", msg)
            }
        }
    }
}

impl std::error::Error for PluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        PluginError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(error: serde_json::Error) -> Self {
        PluginError::SerializationError(error.to_string())
    }
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Helper macro for creating plugin errors.
#[macro_export]
macro_rules! plugin_error {
    ($kind:ident, $($arg:tt)*) => {
        PluginError::$kind(format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_error_display() {
        let error = PluginError::ProcessingFailed("test error".to_string());
        assert_eq!(error.to_string(), "Plugin processing failed: test error");
    }

    #[test]
    fn test_plugin_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let plugin_error = PluginError::from(io_error);
        
        match plugin_error {
            PluginError::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_plugin_error_macro() {
        let error = plugin_error!(ProcessingFailed, "Failed to process {}", "file.rs");
        assert_eq!(error.to_string(), "Plugin processing failed: Failed to process file.rs");
    }
}