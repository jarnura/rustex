//! Plugin manager for registering and executing plugins.

use std::collections::HashMap;
use tracing::{debug, warn, error, info};
use serde::{Deserialize, Serialize};
use super::{Plugin, PluginPhase, PluginContext, PluginError};
use super::plugin::PluginOutput;

/// Configuration for the plugin manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    /// List of enabled plugin names
    pub enabled_plugins: Vec<String>,
    
    /// List of disabled plugin names (takes precedence over enabled)
    pub disabled_plugins: Vec<String>,
    
    /// Plugin-specific configurations
    pub plugin_configs: HashMap<String, serde_json::Value>,
    
    /// Whether to continue execution if a plugin fails
    pub continue_on_error: bool,
    
    /// Maximum number of plugins to run concurrently
    pub max_concurrent: usize,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            enabled_plugins: vec![],
            disabled_plugins: vec![],
            plugin_configs: HashMap::new(),
            continue_on_error: true,
            max_concurrent: 4,
        }
    }
}

/// Manages plugin registration and execution.
pub struct PluginManager {
    /// Registered plugins
    plugins: HashMap<String, Box<dyn Plugin>>,
    
    /// Plugin execution order by phase
    execution_order: HashMap<PluginPhase, Vec<String>>,
    
    /// Manager configuration
    config: PluginManagerConfig,
    
    /// Plugin execution statistics
    stats: PluginStats,
}

/// Plugin execution statistics.
#[derive(Debug, Default, Clone)]
pub struct PluginStats {
    /// Number of plugins executed per phase
    pub executions_per_phase: HashMap<PluginPhase, usize>,
    
    /// Number of plugin failures per phase
    pub failures_per_phase: HashMap<PluginPhase, usize>,
    
    /// Total execution time per plugin (in milliseconds)
    pub execution_times: HashMap<String, u64>,
    
    /// Total number of successful executions
    pub total_successes: usize,
    
    /// Total number of failures
    pub total_failures: usize,
}

impl PluginManager {
    /// Create a new plugin manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(PluginManagerConfig::default())
    }
    
    /// Create a new plugin manager with custom configuration.
    pub fn with_config(config: PluginManagerConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            execution_order: HashMap::new(),
            config,
            stats: PluginStats::default(),
        }
    }
    
    /// Register a plugin with the manager.
    pub fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let plugin_name = plugin.name();
        
        debug!("Registering plugin: {}", plugin_name);
        
        // Check if plugin is already registered
        if self.plugins.contains_key(&plugin_name) {
            return Err(PluginError::ConfigurationError(
                format!("Plugin '{}' is already registered", plugin_name)
            ));
        }
        
        // Initialize plugin with its specific configuration
        if let Some(plugin_config) = self.config.plugin_configs.get(&plugin_name) {
            plugin.initialize(plugin_config)?;
        } else {
            plugin.initialize(&serde_json::Value::Null)?;
        }
        
        // Update execution order for supported phases
        let supported_phases = plugin.info().supported_phases.clone();
        for phase in supported_phases {
            self.execution_order
                .entry(phase)
                .or_insert_with(Vec::new)
                .push(plugin_name.clone());
        }
        
        self.plugins.insert(plugin_name.clone(), plugin);
        
        info!("Plugin '{}' registered successfully", plugin_name);
        Ok(())
    }
    
    /// Unregister a plugin.
    pub fn unregister_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        if let Some(mut plugin) = self.plugins.remove(plugin_name) {
            // Cleanup plugin resources
            plugin.cleanup()?;
            
            // Remove from execution orders
            for phase_plugins in self.execution_order.values_mut() {
                phase_plugins.retain(|name| name != plugin_name);
            }
            
            info!("Plugin '{}' unregistered successfully", plugin_name);
            Ok(())
        } else {
            Err(PluginError::ConfigurationError(
                format!("Plugin '{}' is not registered", plugin_name)
            ))
        }
    }
    
    /// Execute all plugins for a specific phase.
    pub fn execute_phase(
        &mut self, 
        phase: PluginPhase, 
        context: &PluginContext
    ) -> Result<PluginOutput, PluginError> {
        let start_time = std::time::Instant::now();
        
        debug!("Executing plugins for phase: {:?}", phase);
        
        let mut combined_output = PluginOutput::new();
        let plugin_names = self.get_enabled_plugins_for_phase(phase);
        
        if plugin_names.is_empty() {
            debug!("No plugins enabled for phase: {:?}", phase);
            return Ok(combined_output);
        }
        
        let mut executed = 0;
        let mut failed = 0;
        
        for plugin_name in plugin_names {
            if let Some(plugin) = self.plugins.get(&plugin_name) {
                let plugin_start = std::time::Instant::now();
                
                match plugin.execute(phase, context) {
                    Ok(output) => {
                        debug!("Plugin '{}' executed successfully", plugin_name);
                        combined_output.merge(output);
                        executed += 1;
                        
                        // Record execution time
                        let elapsed = plugin_start.elapsed().as_millis() as u64;
                        self.stats.execution_times
                            .entry(plugin_name.clone())
                            .and_modify(|time| *time += elapsed)
                            .or_insert(elapsed);
                    }
                    Err(e) => {
                        failed += 1;
                        error!("Plugin '{}' failed: {}", plugin_name, e);
                        
                        if !self.config.continue_on_error {
                            return Err(e);
                        }
                        
                        // Add error message to output
                        combined_output.add_message(
                            super::plugin::MessageLevel::Error,
                            format!("Plugin '{}' failed: {}", plugin_name, e)
                        );
                    }
                }
            } else {
                warn!("Plugin '{}' not found in registry", plugin_name);
            }
        }
        
        // Update statistics
        self.stats.executions_per_phase
            .entry(phase)
            .and_modify(|count| *count += executed)
            .or_insert(executed);
            
        self.stats.failures_per_phase
            .entry(phase)
            .and_modify(|count| *count += failed)
            .or_insert(failed);
            
        self.stats.total_successes += executed;
        self.stats.total_failures += failed;
        
        let total_time = start_time.elapsed();
        info!(
            "Phase {:?} completed: {} plugins executed, {} failed in {:?}",
            phase, executed, failed, total_time
        );
        
        Ok(combined_output)
    }
    
    /// Get list of enabled plugins for a specific phase.
    fn get_enabled_plugins_for_phase(&self, phase: PluginPhase) -> Vec<String> {
        let phase_plugins = self.execution_order.get(&phase)
            .map(|plugins| plugins.clone())
            .unwrap_or_default();
            
        phase_plugins.into_iter()
            .filter(|plugin_name| self.is_plugin_enabled(plugin_name))
            .collect()
    }
    
    /// Check if a plugin is enabled.
    fn is_plugin_enabled(&self, plugin_name: &str) -> bool {
        // Disabled list takes precedence
        if self.config.disabled_plugins.contains(&plugin_name.to_string()) {
            return false;
        }
        
        // If enabled list is empty, all plugins are enabled by default
        if self.config.enabled_plugins.is_empty() {
            return true;
        }
        
        // Check if plugin is in enabled list
        self.config.enabled_plugins.contains(&plugin_name.to_string())
    }
    
    /// Get list of all registered plugins.
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    /// Get plugin information by name.
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<super::plugin::PluginInfo> {
        self.plugins.get(plugin_name).map(|plugin| plugin.info())
    }
    
    /// Get plugin execution statistics.
    pub fn get_stats(&self) -> &PluginStats {
        &self.stats
    }
    
    /// Reset plugin execution statistics.
    pub fn reset_stats(&mut self) {
        self.stats = PluginStats::default();
    }
    
    /// Update plugin configuration.
    pub fn update_config(&mut self, config: PluginManagerConfig) {
        self.config = config;
    }
    
    /// Get current configuration.
    pub fn get_config(&self) -> &PluginManagerConfig {
        &self.config
    }
    
    /// Enable a plugin.
    pub fn enable_plugin(&mut self, plugin_name: &str) {
        let plugin_name = plugin_name.to_string();
        
        // Remove from disabled list if present
        self.config.disabled_plugins.retain(|name| name != &plugin_name);
        
        // Add to enabled list if not already present
        if !self.config.enabled_plugins.contains(&plugin_name) {
            self.config.enabled_plugins.push(plugin_name);
        }
    }
    
    /// Disable a plugin.
    pub fn disable_plugin(&mut self, plugin_name: &str) {
        let plugin_name = plugin_name.to_string();
        
        // Remove from enabled list if present
        self.config.enabled_plugins.retain(|name| name != &plugin_name);
        
        // Add to disabled list if not already present
        if !self.config.disabled_plugins.contains(&plugin_name) {
            self.config.disabled_plugins.push(plugin_name);
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::plugin::{PluginInfo, PluginPhase};
    use rustex_core::{ProjectInfo, ExtractorConfig};
    use std::path::PathBuf;

    struct TestPlugin {
        name: String,
        should_fail: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: false,
            }
        }
        
        fn with_failure(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: true,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo {
                name: self.name.clone(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: None,
                supported_phases: vec![PluginPhase::PostProject],
                dependencies: vec![],
                default_enabled: true,
            }
        }
        
        fn post_project(&self, _context: &PluginContext) -> Result<PluginOutput, PluginError> {
            if self.should_fail {
                Err(PluginError::ProcessingFailed("Test failure".to_string()))
            } else {
                let mut output = PluginOutput::new();
                output.add_metric("test_metric", 1.0);
                Ok(output)
            }
        }
    }

    fn create_test_context() -> (ProjectInfo, ExtractorConfig, HashMap<String, serde_json::Value>) {
        let project_info = ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        };
        let config = ExtractorConfig::default();
        let metadata = HashMap::new();
        (project_info, config, metadata)
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));
        
        assert!(manager.register_plugin(plugin).is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
        assert!(manager.list_plugins().contains(&"test".to_string()));
    }

    #[test]
    fn test_duplicate_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin1 = Box::new(TestPlugin::new("test"));
        let plugin2 = Box::new(TestPlugin::new("test"));
        
        assert!(manager.register_plugin(plugin1).is_ok());
        assert!(manager.register_plugin(plugin2).is_err());
    }

    #[test]
    fn test_plugin_execution() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));
        
        manager.register_plugin(plugin).unwrap();
        
        let (project_info, config, mut metadata) = create_test_context();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        let result = manager.execute_phase(PluginPhase::PostProject, &context);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.metrics.get("test_metric"), Some(&1.0));
    }

    #[test]
    fn test_plugin_failure_handling() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::with_failure("failing_plugin"));
        
        manager.register_plugin(plugin).unwrap();
        
        let (project_info, config, mut metadata) = create_test_context();
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        
        // With continue_on_error = true (default)
        let result = manager.execute_phase(PluginPhase::PostProject, &context);
        assert!(result.is_ok());
        
        // Check statistics
        let stats = manager.get_stats();
        assert_eq!(stats.total_failures, 1);
    }

    #[test]
    fn test_plugin_enable_disable() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));
        
        manager.register_plugin(plugin).unwrap();
        
        // Initially enabled
        assert!(manager.is_plugin_enabled("test"));
        
        // Disable plugin
        manager.disable_plugin("test");
        assert!(!manager.is_plugin_enabled("test"));
        
        // Re-enable plugin
        manager.enable_plugin("test");
        assert!(manager.is_plugin_enabled("test"));
    }
}