//! # RustEx Plugins
//!
//! Plugin system for extending rustex functionality.
//!
//! This crate provides a trait-based plugin architecture that allows
//! extending RustEx's AST extraction capabilities with custom analysis,
//! formatting, and output generation plugins.
//!
//! # Plugin Types
//!
//! - **Analysis Plugins**: Extend complexity analysis and code metrics
//! - **LLM Optimization Plugins**: Optimize output for language models
//! - **Documentation Plugins**: Enhance documentation extraction
//! - **Integration Plugins**: Connect with external tools and services
//!
//! # Example
//!
//! ```rust
//! use rustex_plugins::{Plugin, PluginInfo, PluginPhase, PluginContext, PluginOutput, PluginError};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn info(&self) -> PluginInfo {
//!         PluginInfo {
//!             name: "my-plugin".to_string(),
//!             version: "0.1.0".to_string(),
//!             description: "Example plugin".to_string(),
//!             author: None,
//!             supported_phases: vec![PluginPhase::PostProject],
//!             dependencies: vec![],
//!             default_enabled: false,
//!         }
//!     }
//!     
//!     fn post_project(&self, _context: &PluginContext) -> Result<PluginOutput, PluginError> {
//!         // Custom processing logic
//!         Ok(PluginOutput::default())
//!     }
//! }
//! ```

pub mod core;
pub mod builtin;
pub mod utils;

// Re-export core types for convenience
pub use core::{
    Plugin, PluginInfo, PluginContext, PluginOutput, PluginError, PluginPhase,
    PluginManager, PreProcessContext, PostProcessContext
};

// Re-export built-in plugins
pub use builtin::*;
