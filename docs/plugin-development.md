# RustEx Plugin Development Guide

This guide explains how to develop custom plugins for RustEx to extend its analysis capabilities and integrate with your specific workflows.

## Table of Contents

- [Plugin Architecture](#plugin-architecture)
- [Plugin Types](#plugin-types)
- [Creating a Basic Plugin](#creating-a-basic-plugin)
- [Plugin API Reference](#plugin-api-reference)
- [Built-in Plugin Examples](#built-in-plugin-examples)
- [Testing Plugins](#testing-plugins)
- [Distribution](#distribution)
- [Best Practices](#best-practices)

## Plugin Architecture

RustEx uses a flexible plugin system that allows you to:

- **Transform AST data** during extraction
- **Add custom analysis** and metrics
- **Generate specialized output formats**
- **Integrate with external tools** and services
- **Implement custom validation rules**

### Plugin Lifecycle

```
1. Plugin Discovery → 2. Initialization → 3. Execution → 4. Cleanup
```

1. **Discovery**: RustEx discovers plugins from configured directories
2. **Initialization**: Plugins are loaded and configured
3. **Execution**: Plugins run during AST processing phases
4. **Cleanup**: Resources are released after processing

### Execution Phases

Plugins can hook into different phases of AST extraction:

- **Pre-process**: Before AST extraction begins
- **Extract**: During element extraction
- **Analyze**: After extraction, before output
- **Post-process**: After analysis, before final output

## Plugin Types

### 1. Analysis Plugins

Analyze code structure and add metrics:

```rust
// Complexity analysis, code quality metrics, dependency analysis
impl Plugin for ComplexityAnalyzer {
    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        // Analyze complexity and add metrics
    }
}
```

### 2. Transform Plugins

Modify or enhance AST data:

```rust
// Add documentation, resolve references, normalize naming
impl Plugin for DocumentationEnhancer {
    fn transform(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        // Transform AST elements
    }
}
```

### 3. Output Plugins

Generate custom output formats:

```rust
// Custom serialization, integration formats, reports
impl Plugin for CustomFormatter {
    fn format(&mut self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        // Generate custom output
    }
}
```

### 4. Integration Plugins

Connect with external systems:

```rust
// Database storage, API calls, file generation
impl Plugin for DatabaseIntegrator {
    fn post_process(&mut self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        // Send data to external system
    }
}
```

## Creating a Basic Plugin

### Step 1: Set up the Plugin Crate

Create a new Rust library:

```bash
cargo new --lib my-rustex-plugin
cd my-rustex-plugin
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
rustex-plugins = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"

[lib]
crate-type = ["cdylib", "rlib"]
```

### Step 2: Implement the Plugin Trait

```rust
// src/lib.rs
use rustex_plugins::{Plugin, PluginContext, PluginOutput, PluginError, PluginMetadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPluginConfig {
    pub threshold: f64,
    pub enabled_checks: Vec<String>,
}

impl Default for MyPluginConfig {
    fn default() -> Self {
        Self {
            threshold: 10.0,
            enabled_checks: vec!["complexity".to_string(), "documentation".to_string()],
        }
    }
}

pub struct MyPlugin {
    config: MyPluginConfig,
}

impl MyPlugin {
    pub fn new(config: MyPluginConfig) -> Self {
        Self { config }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My custom RustEx plugin".to_string(),
            author: "Your Name".to_string(),
            capabilities: vec![
                "analysis".to_string(),
                "metrics".to_string(),
            ],
        }
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
        // Initialize plugin state
        println!("Initializing MyPlugin for project: {}", context.project_name());
        Ok(())
    }

    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        // Access project AST
        if let Some(project_ast) = context.project_ast() {
            // Perform custom analysis
            let custom_metric = self.calculate_custom_metric(project_ast);
            
            // Add metrics to output
            output.add_metric("custom_score", custom_metric);
            
            // Add metadata
            output.add_metadata("analysis_type", serde_json::json!("custom"));
            output.add_metadata("threshold_used", serde_json::json!(self.config.threshold));
        }
        
        Ok(output)
    }

    fn finalize(&mut self, _context: &PluginContext) -> Result<(), PluginError> {
        // Cleanup resources
        println!("Finalizing MyPlugin");
        Ok(())
    }
}

impl MyPlugin {
    fn calculate_custom_metric(&self, project_ast: &rustex_core::ProjectAst) -> f64 {
        // Implement your custom analysis logic
        let total_functions = project_ast.metrics.total_functions as f64;
        let avg_complexity = project_ast.metrics.complexity_average;
        
        // Example: Custom quality score
        if total_functions > 0.0 {
            (100.0 - avg_complexity).max(0.0)
        } else {
            0.0
        }
    }
}

// Export plugin factory function
#[no_mangle]
pub fn create_plugin(config_json: &str) -> Result<Box<dyn Plugin>, PluginError> {
    let config: MyPluginConfig = serde_json::from_str(config_json)
        .map_err(|e| PluginError::ConfigurationError(e.to_string()))?;
    Ok(Box::new(MyPlugin::new(config)))
}
```

### Step 3: Configure the Plugin

Create a configuration file for your plugin:

```toml
# my-plugin-config.toml
[my-plugin]
threshold = 15.0
enabled_checks = ["complexity", "documentation", "naming"]
```

### Step 4: Test the Plugin

```rust
// tests/integration_test.rs
use rustex_plugins::{Plugin, PluginContext};
use my_rustex_plugin::{MyPlugin, MyPluginConfig};

#[test]
fn test_plugin_basic_functionality() {
    let config = MyPluginConfig::default();
    let mut plugin = MyPlugin::new(config);
    
    // Create mock context
    let context = PluginContext::new_mock();
    
    // Test initialization
    assert!(plugin.initialize(&context).is_ok());
    
    // Test metadata
    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "my-plugin");
    assert_eq!(metadata.version, "0.1.0");
}
```

## Plugin API Reference

### Core Traits

#### `Plugin` Trait

The main trait that all plugins must implement:

```rust
pub trait Plugin: Send + Sync {
    /// Plugin metadata and information
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize plugin with context
    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Pre-processing phase
    fn pre_process(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        Ok(PluginOutput::new())
    }
    
    /// Analysis phase (main processing)
    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        Ok(PluginOutput::new())
    }
    
    /// Post-processing phase
    fn post_process(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        Ok(PluginOutput::new())
    }
    
    /// Custom output formatting
    fn format(&mut self, context: &PluginContext) -> Result<PluginOutput, PluginError> {
        Ok(PluginOutput::new())
    }
    
    /// Cleanup and finalization
    fn finalize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
}
```

#### `PluginContext`

Provides access to AST data and configuration:

```rust
impl PluginContext {
    /// Get project name
    pub fn project_name(&self) -> &str;
    
    /// Get project AST (immutable access)
    pub fn project_ast(&self) -> Option<&rustex_core::ProjectAst>;
    
    /// Get mutable project AST (for transform plugins)
    pub fn project_ast_mut(&mut self) -> Option<&mut rustex_core::ProjectAst>;
    
    /// Get current file being processed
    pub fn current_file(&self) -> Option<&rustex_core::FileAst>;
    
    /// Get mutable current file
    pub fn current_file_mut(&mut self) -> Option<&mut rustex_core::FileAst>;
    
    /// Get plugin configuration
    pub fn config(&self) -> &serde_json::Value;
    
    /// Get extraction configuration
    pub fn extractor_config(&self) -> &rustex_core::ExtractorConfig;
}
```

#### `PluginOutput`

Structure for plugin results:

```rust
impl PluginOutput {
    /// Create new empty output
    pub fn new() -> Self;
    
    /// Add a metric value
    pub fn add_metric<T: Into<f64>>(&mut self, name: String, value: T);
    
    /// Add metadata
    pub fn add_metadata<T: Into<serde_json::Value>>(&mut self, key: String, value: T);
    
    /// Add a warning message
    pub fn add_warning(&mut self, message: String);
    
    /// Add an error message
    pub fn add_error(&mut self, message: String);
    
    /// Merge another output into this one
    pub fn merge(&mut self, other: PluginOutput);
}
```

### Error Handling

```rust
pub enum PluginError {
    ConfigurationError(String),
    ProcessingError(String),
    IoError(std::io::Error),
    SerializationError(String),
    Custom(String),
}
```

## Built-in Plugin Examples

### Complexity Analyzer Plugin

```rust
// Example: Complexity analysis plugin
use rustex_plugins::{Plugin, PluginContext, PluginOutput, PluginError};

pub struct ComplexityAnalyzer {
    config: ComplexityConfig,
}

#[derive(serde::Deserialize)]
pub struct ComplexityConfig {
    pub warning_threshold: u32,
    pub error_threshold: u32,
    pub include_cognitive: bool,
    pub include_halstead: bool,
}

impl Plugin for ComplexityAnalyzer {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "complexity-analyzer".to_string(),
            version: "0.1.0".to_string(),
            description: "Analyzes code complexity metrics".to_string(),
            author: "RustEx Team".to_string(),
            capabilities: vec!["analysis".to_string(), "metrics".to_string()],
        }
    }

    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        if let Some(project_ast) = context.project_ast() {
            let mut high_complexity_count = 0;
            let mut very_high_complexity_count = 0;
            
            for file in &project_ast.files {
                for element in &file.elements {
                    if let Some(complexity) = element.complexity {
                        if complexity > self.config.error_threshold {
                            very_high_complexity_count += 1;
                            output.add_error(format!(
                                "Very high complexity in {}: {} (complexity: {})",
                                element.name, file.relative_path.display(), complexity
                            ));
                        } else if complexity > self.config.warning_threshold {
                            high_complexity_count += 1;
                            output.add_warning(format!(
                                "High complexity in {}: {} (complexity: {})",
                                element.name, file.relative_path.display(), complexity
                            ));
                        }
                    }
                }
            }
            
            // Add metrics
            output.add_metric("high_complexity_functions", high_complexity_count);
            output.add_metric("very_high_complexity_functions", very_high_complexity_count);
            
            // Add summary metadata
            output.add_metadata("complexity_analysis", serde_json::json!({
                "warning_threshold": self.config.warning_threshold,
                "error_threshold": self.config.error_threshold,
                "high_complexity_count": high_complexity_count,
                "very_high_complexity_count": very_high_complexity_count
            }));
        }
        
        Ok(output)
    }
}
```

### Documentation Enhancer Plugin

```rust
// Example: Documentation enhancement plugin
use rustex_plugins::{Plugin, PluginContext, PluginOutput, PluginError};

pub struct DocumentationEnhancer {
    config: DocConfig,
}

#[derive(serde::Deserialize)]
pub struct DocConfig {
    pub min_coverage: f64,
    pub auto_generate: bool,
    pub style: String, // "rust", "google", "jsdoc"
}

impl Plugin for DocumentationEnhancer {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "doc-enhancer".to_string(),
            version: "0.1.0".to_string(),
            description: "Enhances and validates documentation".to_string(),
            author: "RustEx Team".to_string(),
            capabilities: vec!["transform".to_string(), "validation".to_string()],
        }
    }

    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        if let Some(project_ast) = context.project_ast_mut() {
            let mut documented_items = 0;
            let mut total_public_items = 0;
            
            for file in &mut project_ast.files {
                for element in &mut file.elements {
                    if element.visibility == rustex_core::Visibility::Public {
                        total_public_items += 1;
                        
                        if !element.doc_comments.is_empty() {
                            documented_items += 1;
                            
                            // Enhance existing documentation
                            self.enhance_documentation(element);
                        } else if self.config.auto_generate {
                            // Generate basic documentation
                            element.doc_comments.push(
                                self.generate_basic_doc(element)
                            );
                            documented_items += 1;
                        } else {
                            output.add_warning(format!(
                                "Missing documentation for public {}: {}",
                                format!("{:?}", element.element_type).to_lowercase(),
                                element.name
                            ));
                        }
                    }
                }
            }
            
            let coverage = if total_public_items > 0 {
                documented_items as f64 / total_public_items as f64
            } else {
                1.0
            };
            
            output.add_metric("documentation_coverage", coverage);
            
            if coverage < self.config.min_coverage {
                output.add_error(format!(
                    "Documentation coverage {:.1}% is below minimum {:.1}%",
                    coverage * 100.0,
                    self.config.min_coverage * 100.0
                ));
            }
            
            output.add_metadata("documentation_analysis", serde_json::json!({
                "coverage": coverage,
                "documented_items": documented_items,
                "total_public_items": total_public_items,
                "style": self.config.style
            }));
        }
        
        Ok(output)
    }
}

impl DocumentationEnhancer {
    fn enhance_documentation(&self, element: &mut rustex_core::CodeElement) {
        // Add style-specific enhancements
        match self.config.style.as_str() {
            "google" => self.apply_google_style(element),
            "jsdoc" => self.apply_jsdoc_style(element),
            _ => self.apply_rust_style(element),
        }
    }
    
    fn generate_basic_doc(&self, element: &rustex_core::CodeElement) -> String {
        match element.element_type {
            rustex_core::ElementType::Function => {
                format!("Function: {}", element.name)
            },
            rustex_core::ElementType::Struct => {
                format!("Struct: {}", element.name)
            },
            rustex_core::ElementType::Enum => {
                format!("Enum: {}", element.name)
            },
            _ => format!("{:?}: {}", element.element_type, element.name),
        }
    }
    
    fn apply_rust_style(&self, element: &mut rustex_core::CodeElement) {
        // Apply Rust documentation conventions
    }
    
    fn apply_google_style(&self, element: &mut rustex_core::CodeElement) {
        // Apply Google documentation style
    }
    
    fn apply_jsdoc_style(&self, element: &mut rustex_core::CodeElement) {
        // Apply JSDoc-style documentation
    }
}
```

## Testing Plugins

### Unit Tests

```rust
// tests/unit_tests.rs
use my_rustex_plugin::{MyPlugin, MyPluginConfig};
use rustex_plugins::{Plugin, PluginContext};

#[test]
fn test_plugin_initialization() {
    let config = MyPluginConfig::default();
    let mut plugin = MyPlugin::new(config);
    let context = PluginContext::new_mock();
    
    assert!(plugin.initialize(&context).is_ok());
}

#[test]
fn test_custom_metric_calculation() {
    let config = MyPluginConfig::default();
    let plugin = MyPlugin::new(config);
    
    // Create test project AST
    let project_ast = rustex_core::test_fixtures::MockDataGenerator::project_ast(5, 10);
    
    let metric = plugin.calculate_custom_metric(&project_ast);
    assert!(metric >= 0.0);
    assert!(metric <= 100.0);
}
```

### Integration Tests

```rust
// tests/integration_tests.rs
use rustex_core::{AstExtractor, ExtractorConfig};
use my_rustex_plugin::{MyPlugin, MyPluginConfig};

#[test]
fn test_plugin_with_real_project() {
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, std::path::PathBuf::from("."));
    
    let project_ast = extractor.extract_project().unwrap();
    
    let plugin_config = MyPluginConfig::default();
    let mut plugin = MyPlugin::new(plugin_config);
    
    let mut context = PluginContext::new(&project_ast);
    let output = plugin.analyze(&mut context).unwrap();
    
    assert!(!output.metrics().is_empty());
}
```

### Mock Data Testing

```rust
use rustex_core::test_fixtures::TestFixtureBuilder;

#[test]
fn test_plugin_with_fixtures() {
    let fixture = TestFixtureBuilder::new()
        .with_project_name("test-project")
        .with_file("lib.rs", r#"
            /// A test function
            pub fn test_function() -> i32 {
                42
            }
        "#)
        .build();
    
    let config = ExtractorConfig::default();
    let extractor = AstExtractor::new(config, fixture.project_root().to_path_buf());
    let project_ast = extractor.extract_project().unwrap();
    
    // Test plugin with fixture data
    let mut context = PluginContext::new(&project_ast);
    let mut plugin = MyPlugin::new(MyPluginConfig::default());
    let output = plugin.analyze(&mut context).unwrap();
    
    assert!(output.metrics().contains_key("custom_score"));
}
```

## Distribution

### Publishing to Crates.io

1. **Prepare your crate**:
   ```toml
   [package]
   name = "rustex-plugin-my-plugin"
   version = "0.1.0"
   description = "My custom RustEx plugin"
   license = "MIT OR Apache-2.0"
   repository = "https://github.com/username/rustex-plugin-my-plugin"
   keywords = ["rustex", "plugin", "analysis"]
   categories = ["development-tools"]
   ```

2. **Publish**:
   ```bash
   cargo publish
   ```

### Plugin Registry

Create a registry entry for discoverability:

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "My custom RustEx plugin",
  "author": "Your Name",
  "repository": "https://github.com/username/rustex-plugin-my-plugin",
  "capabilities": ["analysis", "metrics"],
  "configuration_schema": {
    "type": "object",
    "properties": {
      "threshold": {
        "type": "number",
        "default": 10.0
      },
      "enabled_checks": {
        "type": "array",
        "items": {"type": "string"},
        "default": ["complexity", "documentation"]
      }
    }
  }
}
```

## Best Practices

### Performance

1. **Efficient AST traversal**:
   ```rust
   // Good: Process elements in single pass
   for file in &project_ast.files {
       for element in &file.elements {
           self.process_element(element);
       }
   }
   
   // Avoid: Multiple passes over the same data
   ```

2. **Memory management**:
   ```rust
   // Use streaming for large datasets
   impl Plugin for MyPlugin {
       fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
           let mut output = PluginOutput::new();
           
           // Process files one at a time for large projects
           if let Some(file) = context.current_file() {
               self.process_file_streaming(file, &mut output)?;
           }
           
           Ok(output)
       }
   }
   ```

### Error Handling

1. **Graceful degradation**:
   ```rust
   fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
       let mut output = PluginOutput::new();
       
       match self.try_complex_analysis(context) {
           Ok(result) => output.merge(result),
           Err(e) => {
               output.add_warning(format!("Complex analysis failed: {}", e));
               // Fall back to simpler analysis
               output.merge(self.simple_analysis(context)?);
           }
       }
       
       Ok(output)
   }
   ```

2. **Clear error messages**:
   ```rust
   return Err(PluginError::ProcessingError(
       format!("Failed to process element '{}' in file '{}': {}", 
               element.name, file.path.display(), e)
   ));
   ```

### Configuration

1. **Comprehensive configuration validation**:
   ```rust
   impl MyPluginConfig {
       pub fn validate(&self) -> Result<(), String> {
           if self.threshold < 0.0 {
               return Err("Threshold must be non-negative".to_string());
           }
           if self.enabled_checks.is_empty() {
               return Err("At least one check must be enabled".to_string());
           }
           Ok(())
       }
   }
   ```

2. **Provide sensible defaults**:
   ```rust
   impl Default for MyPluginConfig {
       fn default() -> Self {
           Self {
               threshold: 10.0,
               enabled_checks: vec![
                   "complexity".to_string(),
                   "documentation".to_string(),
               ],
               // ... other sensible defaults
           }
       }
   }
   ```

### Documentation

1. **Document plugin capabilities**:
   ```rust
   /// # My Plugin
   /// 
   /// This plugin analyzes code quality metrics and provides warnings
   /// for potential issues.
   /// 
   /// ## Capabilities
   /// - Complexity analysis
   /// - Documentation coverage
   /// - Custom quality scoring
   /// 
   /// ## Configuration
   /// ```toml
   /// [my-plugin]
   /// threshold = 10.0
   /// enabled_checks = ["complexity", "documentation"]
   /// ```
   pub struct MyPlugin {
       // ...
   }
   ```

2. **Provide usage examples**:
   ```rust
   /// # Example
   /// 
   /// ```rust
   /// use my_rustex_plugin::{MyPlugin, MyPluginConfig};
   /// 
   /// let config = MyPluginConfig {
   ///     threshold: 15.0,
   ///     enabled_checks: vec!["complexity".to_string()],
   /// };
   /// 
   /// let plugin = MyPlugin::new(config);
   /// ```
   ```

---

For more plugin examples and advanced patterns, see the [built-in plugins](../crates/rustex-plugins/src/builtin/) in the RustEx repository.