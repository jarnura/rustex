//! Built-in plugins for RustEx.
//!
//! This module contains plugins that ship with RustEx by default,
//! providing common functionality for AST analysis and enhancement.

pub mod complexity;
pub mod llm_optimizer;
pub mod doc_enhancer;
pub mod metrics;

// Re-export built-in plugins
pub use complexity::ComplexityAnalyzer;
pub use llm_optimizer::LlmOptimizer;
pub use doc_enhancer::DocEnhancer;
pub use metrics::MetricsCollector;

/// Register all built-in plugins with a plugin manager.
pub fn register_builtin_plugins(
    manager: &mut crate::core::PluginManager
) -> Result<(), crate::core::PluginError> {
    manager.register_plugin(Box::new(ComplexityAnalyzer::default()))?;
    manager.register_plugin(Box::new(LlmOptimizer::default()))?;
    manager.register_plugin(Box::new(DocEnhancer::default()))?;
    manager.register_plugin(Box::new(MetricsCollector::default()))?;
    
    Ok(())
}