//! Core plugin system components.
//!
//! This module provides the fundamental traits, types, and managers
//! for the plugin system architecture.

pub mod plugin;
pub mod manager;
pub mod context;
pub mod errors;

// Re-export core types
pub use plugin::{Plugin, PluginPhase, PluginInfo, PluginOutput};
pub use manager::PluginManager;
pub use context::{PluginContext, PreProcessContext, PostProcessContext};
pub use errors::PluginError;