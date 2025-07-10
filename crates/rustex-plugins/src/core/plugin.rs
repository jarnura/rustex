//! Core plugin trait and related types.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rustex_core::CodeElement;
use super::errors::{PluginError, PluginResult};
use super::context::PluginContext;

/// Phases in which plugins can execute during AST extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginPhase {
    /// Before file discovery and processing begins
    PreProcess,
    
    /// After a single file's AST is extracted but before project assembly
    PostFileExtract,
    
    /// After all files are processed and project AST is assembled
    PostProject,
    
    /// Before output formatting
    PreFormat,
    
    /// After output formatting
    PostFormat,
}

impl PluginPhase {
    /// Returns all available plugin phases in execution order.
    pub fn all() -> Vec<PluginPhase> {
        vec![
            PluginPhase::PreProcess,
            PluginPhase::PostFileExtract,
            PluginPhase::PostProject,
            PluginPhase::PreFormat,
            PluginPhase::PostFormat,
        ]
    }
    
    /// Returns a human-readable description of the phase.
    pub fn description(&self) -> &'static str {
        match self {
            PluginPhase::PreProcess => "Before file discovery and processing",
            PluginPhase::PostFileExtract => "After individual file extraction",
            PluginPhase::PostProject => "After project assembly",
            PluginPhase::PreFormat => "Before output formatting",
            PluginPhase::PostFormat => "After output formatting",
        }
    }
}

/// Output produced by a plugin during execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginOutput {
    /// Modified or additional code elements
    pub modified_elements: Vec<CodeElement>,
    
    /// Additional metadata to be merged into the project
    pub additional_metadata: HashMap<String, serde_json::Value>,
    
    /// Custom metrics calculated by the plugin
    pub metrics: HashMap<String, f64>,
    
    /// Log messages from the plugin
    pub messages: Vec<PluginMessage>,
    
    /// Whether the plugin made any modifications
    pub has_modifications: bool,
}

impl PluginOutput {
    /// Create a new empty plugin output.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a code element to the output.
    pub fn add_element(&mut self, element: CodeElement) {
        self.modified_elements.push(element);
        self.has_modifications = true;
    }
    
    /// Add metadata to the output.
    pub fn add_metadata<K, V>(&mut self, key: K, value: V) 
    where 
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.additional_metadata.insert(key.into(), value.into());
        self.has_modifications = true;
    }
    
    /// Add a metric to the output.
    pub fn add_metric<K>(&mut self, key: K, value: f64) 
    where 
        K: Into<String>,
    {
        self.metrics.insert(key.into(), value);
        self.has_modifications = true;
    }
    
    /// Add a log message to the output.
    pub fn add_message(&mut self, level: MessageLevel, message: String) {
        self.messages.push(PluginMessage { level, message });
    }
    
    /// Merge another plugin output into this one.
    pub fn merge(&mut self, other: PluginOutput) {
        self.modified_elements.extend(other.modified_elements);
        self.additional_metadata.extend(other.additional_metadata);
        self.metrics.extend(other.metrics);
        self.messages.extend(other.messages);
        self.has_modifications = self.has_modifications || other.has_modifications;
    }
}

/// Log message from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub level: MessageLevel,
    pub message: String,
}

/// Message severity levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Plugin metadata and information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: Option<String>,
    
    /// Supported phases
    pub supported_phases: Vec<PluginPhase>,
    
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    
    /// Whether the plugin is enabled by default
    pub default_enabled: bool,
}

/// Core plugin trait that all plugins must implement.
pub trait Plugin: Send + Sync {
    /// Get plugin information.
    fn info(&self) -> PluginInfo;
    
    /// Get plugin name (convenience method).
    fn name(&self) -> String {
        self.info().name
    }
    
    /// Get plugin version (convenience method).
    fn version(&self) -> String {
        self.info().version
    }
    
    /// Get plugin description (convenience method).
    fn description(&self) -> String {
        self.info().description
    }
    
    /// Initialize the plugin with configuration.
    fn initialize(&mut self, config: &serde_json::Value) -> PluginResult<()> {
        let _ = config; // Suppress unused parameter warning
        Ok(())
    }
    
    /// Check if the plugin supports a specific phase.
    fn supports_phase(&self, phase: PluginPhase) -> bool {
        self.info().supported_phases.contains(&phase)
    }
    
    /// Execute the plugin for a specific phase.
    fn execute(&self, phase: PluginPhase, context: &PluginContext) -> PluginResult<PluginOutput> {
        if !self.supports_phase(phase) {
            return Err(PluginError::InvalidContext(
                format!("Plugin {} does not support phase {:?}", self.info().name, phase)
            ));
        }
        
        match phase {
            PluginPhase::PreProcess => self.pre_process(context),
            PluginPhase::PostFileExtract => self.post_file_extract(context),
            PluginPhase::PostProject => self.post_project(context),
            PluginPhase::PreFormat => self.pre_format(context),
            PluginPhase::PostFormat => self.post_format(context),
        }
    }
    
    // Phase-specific hooks (default implementations do nothing)
    
    /// Execute during pre-processing phase.
    fn pre_process(&self, _context: &PluginContext) -> PluginResult<PluginOutput> {
        Ok(PluginOutput::new())
    }
    
    /// Execute after file extraction phase.
    fn post_file_extract(&self, _context: &PluginContext) -> PluginResult<PluginOutput> {
        Ok(PluginOutput::new())
    }
    
    /// Execute after project assembly phase.
    fn post_project(&self, _context: &PluginContext) -> PluginResult<PluginOutput> {
        Ok(PluginOutput::new())
    }
    
    /// Execute before formatting phase.
    fn pre_format(&self, _context: &PluginContext) -> PluginResult<PluginOutput> {
        Ok(PluginOutput::new())
    }
    
    /// Execute after formatting phase.
    fn post_format(&self, _context: &PluginContext) -> PluginResult<PluginOutput> {
        Ok(PluginOutput::new())
    }
    
    /// Clean up plugin resources.
    fn cleanup(&mut self) -> PluginResult<()> {
        Ok(())
    }
}

/// Helper macro for creating plugin info.
#[macro_export]
macro_rules! plugin_info {
    ($name:expr, $version:expr, $description:expr) => {
        PluginInfo {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            author: None,
            supported_phases: vec![PluginPhase::PostProject],
            dependencies: vec![],
            default_enabled: false,
        }
    };
    
    ($name:expr, $version:expr, $description:expr, phases: [$($phase:expr),*]) => {
        PluginInfo {
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            author: None,
            supported_phases: vec![$($phase),*],
            dependencies: vec![],
            default_enabled: false,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            plugin_info!("test", "0.1.0", "Test plugin")
        }
    }

    #[test]
    fn test_plugin_info_macro() {
        let info = plugin_info!("test", "0.1.0", "Test plugin");
        assert_eq!(info.name, "test");
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.description, "Test plugin");
    }

    #[test]
    fn test_plugin_phases() {
        let phases = PluginPhase::all();
        assert_eq!(phases.len(), 5);
        assert!(phases.contains(&PluginPhase::PreProcess));
        assert!(phases.contains(&PluginPhase::PostProject));
    }

    #[test]
    fn test_plugin_output() {
        let mut output = PluginOutput::new();
        output.add_metric("complexity", 5.0);
        output.add_metadata("custom", "value");
        
        assert!(output.has_modifications);
        assert_eq!(output.metrics.get("complexity"), Some(&5.0));
        assert_eq!(output.additional_metadata.get("custom"), Some(&serde_json::Value::String("value".to_string())));
    }

    #[test]
    fn test_plugin_supports_phase() {
        let plugin = TestPlugin;
        assert!(plugin.supports_phase(PluginPhase::PostProject));
        assert!(!plugin.supports_phase(PluginPhase::PreProcess));
    }
}