//! Configuration structures for AST extraction.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration for AST extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Include documentation comments in extraction
    #[serde(default = "default_include_docs")]
    pub include_docs: bool,
    /// Include private items in extraction
    #[serde(default)]
    pub include_private: bool,
    /// Parse dependency information
    #[serde(default)]
    pub parse_dependencies: bool,
    /// Maximum file size to process (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: usize,
    /// Output format for extracted data
    #[serde(default)]
    pub output_format: OutputFormat,
    /// File filtering configuration
    #[serde(default)]
    pub filters: FilterConfig,
    /// Enabled plugins
    #[serde(default)]
    pub plugins: Vec<String>,
}

fn default_include_docs() -> bool {
    true
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

/// Available output formats for AST data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    /// JSON format
    #[default]
    Json,
    /// MessagePack binary format
    MessagePack,
    /// Markdown documentation format
    Markdown,
    /// GraphQL schema format
    GraphQL,
    /// RAG-optimized format
    Rag,
}

/// File filtering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Glob patterns for files to include
    #[serde(default = "default_include_patterns")]
    pub include: Vec<String>,
    /// Glob patterns for files to exclude
    #[serde(default = "default_exclude_patterns")]
    pub exclude: Vec<String>,
}

fn default_include_patterns() -> Vec<String> {
    vec!["src/**/*.rs".to_string()]
}

fn default_exclude_patterns() -> Vec<String> {
    vec!["target/**".to_string(), "tests/**".to_string()]
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            include: default_include_patterns(),
            exclude: default_exclude_patterns(),
        }
    }
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            include_docs: true,
            include_private: false,
            parse_dependencies: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            output_format: OutputFormat::Json,
            filters: FilterConfig {
                include: vec!["src/**/*.rs".to_string()],
                exclude: vec!["target/**".to_string(), "tests/**".to_string()],
            },
            plugins: vec![],
        }
    }
}

impl ExtractorConfig {
    /// Load configuration from a TOML file.
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: ExtractorConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {}", path.as_ref().display()))?;

        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn to_toml_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Load configuration from a TOML string.
    pub fn from_toml_str(content: &str) -> Result<Self> {
        let config: ExtractorConfig =
            toml::from_str(content).context("Failed to parse TOML configuration")?;

        Ok(config)
    }

    /// Convert configuration to TOML string.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize config to TOML")
    }

    /// Try to load configuration from various standard locations.
    ///
    /// Looks for configuration files in the following order:
    /// 1. ./rustex.toml (project-specific config)
    /// 2. ./rustex-config.toml
    /// 3. ~/.config/rustex/config.toml (user config)
    /// 4. Falls back to default configuration
    pub fn load_from_standard_locations() -> Self {
        // Try project-specific configs first
        for filename in &["rustex.toml", "rustex-config.toml", ".rustex.toml"] {
            if let Ok(config) = Self::from_toml_file(filename) {
                return config;
            }
        }

        // Try user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let user_config = config_dir.join("rustex").join("config.toml");
            if let Ok(config) = Self::from_toml_file(&user_config) {
                return config;
            }
        }

        // Fall back to default
        Self::default()
    }

    /// Create example configuration file with comments.
    pub fn create_example_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let example_content = r#"# RustEx Configuration File
# This file configures the Rust AST extractor behavior.

# Include documentation comments in extraction
include_docs = true

# Include private items in extraction
include_private = false

# Parse dependency information from Cargo.toml
parse_dependencies = false

# Maximum file size to process (in bytes)
max_file_size = 10485760  # 10MB

# Output format for extracted data
output_format = "Json"  # Options: Json, MessagePack, Markdown, GraphQL, Rag

# Enabled plugins
plugins = []

[filters]
# Glob patterns for files to include
include = ["src/**/*.rs"]

# Glob patterns for files to exclude  
exclude = ["target/**", "tests/**"]
"#;

        fs::write(&path, example_content).with_context(|| {
            format!(
                "Failed to write example config: {}",
                path.as_ref().display()
            )
        })?;

        Ok(())
    }

    /// Validate the configuration for common issues.
    pub fn validate(&self) -> Result<()> {
        // Validate max file size
        if self.max_file_size == 0 {
            anyhow::bail!("max_file_size must be greater than 0");
        }

        if self.max_file_size > 100 * 1024 * 1024 {
            anyhow::bail!("max_file_size is too large (>100MB), this may cause memory issues");
        }

        // Validate include patterns
        if self.filters.include.is_empty() {
            anyhow::bail!("At least one include pattern must be specified");
        }

        // Check for conflicting patterns
        for include_pattern in &self.filters.include {
            for exclude_pattern in &self.filters.exclude {
                if include_pattern == exclude_pattern {
                    anyhow::bail!(
                        "Include and exclude patterns cannot be identical: {}",
                        include_pattern
                    );
                }
            }
        }

        Ok(())
    }

    /// Merge this configuration with another, preferring values from `other`.
    pub fn merge_with(&mut self, other: ExtractorConfig) {
        if other.include_docs != self.include_docs {
            self.include_docs = other.include_docs;
        }
        if other.include_private != self.include_private {
            self.include_private = other.include_private;
        }
        if other.parse_dependencies != self.parse_dependencies {
            self.parse_dependencies = other.parse_dependencies;
        }
        if other.max_file_size != 10 * 1024 * 1024 {
            // Not default value
            self.max_file_size = other.max_file_size;
        }

        // Merge filters
        if !other.filters.include.is_empty() {
            self.filters.include = other.filters.include;
        }
        if !other.filters.exclude.is_empty() {
            self.filters.exclude = other.filters.exclude;
        }

        // Merge plugins
        if !other.plugins.is_empty() {
            self.plugins = other.plugins;
        }
    }

    /// Create a configuration optimized for different use cases.
    pub fn for_use_case(use_case: ConfigUseCase) -> Self {
        let mut config = Self::default();

        match use_case {
            ConfigUseCase::Documentation => {
                config.include_docs = true;
                config.include_private = false;
                config.output_format = OutputFormat::Markdown;
                config.filters.include =
                    vec!["src/**/*.rs".to_string(), "examples/**/*.rs".to_string()];
                config.filters.exclude = vec!["target/**".to_string()];
            }
            ConfigUseCase::CodeAnalysis => {
                config.include_docs = true;
                config.include_private = true;
                config.parse_dependencies = true;
                config.output_format = OutputFormat::Json;
                config.filters.include = vec!["**/*.rs".to_string()];
                config.filters.exclude = vec!["target/**".to_string()];
            }
            ConfigUseCase::LlmTraining => {
                config.include_docs = true;
                config.include_private = false;
                config.parse_dependencies = false;
                config.output_format = OutputFormat::Rag;
                config.filters.include =
                    vec!["src/**/*.rs".to_string(), "examples/**/*.rs".to_string()];
                config.filters.exclude = vec!["target/**".to_string(), "tests/**".to_string()];
            }
            ConfigUseCase::Testing => {
                config.include_docs = false;
                config.include_private = true;
                config.parse_dependencies = false;
                config.max_file_size = 1024 * 1024; // 1MB for faster testing
                config.output_format = OutputFormat::Json;
                config.filters.include =
                    vec!["tests/**/*.rs".to_string(), "src/**/*.rs".to_string()];
                config.filters.exclude = vec!["target/**".to_string()];
            }
        }

        config
    }
}

/// Predefined configuration use cases.
#[derive(Debug, Clone, Copy)]
pub enum ConfigUseCase {
    /// Optimized for generating documentation
    Documentation,
    /// Optimized for code analysis and metrics
    CodeAnalysis,
    /// Optimized for LLM training data
    LlmTraining,
    /// Optimized for testing scenarios
    Testing,
}
