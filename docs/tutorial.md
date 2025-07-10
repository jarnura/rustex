# RustEx Tutorial: From Beginner to Advanced

This comprehensive tutorial will take you through RustEx from basic usage to advanced integration patterns. By the end, you'll be able to leverage RustEx for documentation generation, code analysis, LLM training data preparation, and more.

## Table of Contents

- [Tutorial Overview](#tutorial-overview)
- [Part 1: Getting Started](#part-1-getting-started)
- [Part 2: Understanding Output](#part-2-understanding-output)
- [Part 3: Configuration Mastery](#part-3-configuration-mastery)
- [Part 4: Advanced Analysis](#part-4-advanced-analysis)
- [Part 5: LLM Integration](#part-5-llm-integration)
- [Part 6: Automation and CI/CD](#part-6-automation-and-cicd)
- [Part 7: Plugin Development](#part-7-plugin-development)
- [Part 8: Best Practices](#part-8-best-practices)

## Tutorial Overview

### What You'll Learn

- How to extract comprehensive AST data from Rust projects
- Different output formats and their use cases
- Configuration strategies for various scenarios
- Integration with development workflows
- Advanced analysis techniques
- LLM training data preparation
- Plugin development basics

### Prerequisites

- Basic understanding of Rust programming
- Familiarity with command-line tools
- A Rust project to work with (we'll create one if needed)

### Setup

Make sure you have RustEx installed:

```bash
cargo install rustex-cli
rustex --version
```

## Part 1: Getting Started

### 1.1 Creating a Demo Project

Let's start with a realistic Rust project that demonstrates various language features:

```bash
cargo new --lib rustex-tutorial
cd rustex-tutorial
```

Replace the contents of `src/lib.rs`:

```rust
//! # Weather Analysis Library
//! 
//! A comprehensive library for analyzing weather data and patterns.
//! This library provides tools for data collection, analysis, and forecasting.

use std::collections::HashMap;
use std::fmt;

/// Represents different types of weather conditions
#[derive(Debug, Clone, PartialEq)]
pub enum WeatherCondition {
    /// Clear skies with no precipitation
    Clear,
    /// Cloudy conditions with varying coverage
    Cloudy { coverage: u8 }, // 0-100%
    /// Rainy conditions with intensity
    Rainy { intensity: RainIntensity },
    /// Snowy conditions with accumulation
    Snowy { accumulation: f32 }, // in cm
    /// Stormy conditions with wind speed
    Stormy { wind_speed: f32 }, // in km/h
}

/// Different intensities of rainfall
#[derive(Debug, Clone, PartialEq)]
pub enum RainIntensity {
    Light,
    Moderate,
    Heavy,
}

/// Core trait for weather data analysis
pub trait WeatherAnalyzer {
    /// Analyze temperature trends over time
    fn analyze_temperature(&self, data: &[f32]) -> TemperatureAnalysis;
    
    /// Predict weather conditions based on current data
    fn predict_conditions(&self, current: &WeatherReading) -> WeatherCondition;
    
    /// Calculate comfort index (0-100)
    fn comfort_index(&self, reading: &WeatherReading) -> f32;
}

/// A single weather reading with timestamp
#[derive(Debug, Clone)]
pub struct WeatherReading {
    pub timestamp: u64,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub condition: WeatherCondition,
}

/// Results of temperature analysis
#[derive(Debug)]
pub struct TemperatureAnalysis {
    pub average: f32,
    pub min: f32,
    pub max: f32,
    pub trend: TemperatureTrend,
    pub variance: f32,
}

/// Temperature trend indicators
#[derive(Debug, PartialEq)]
pub enum TemperatureTrend {
    Rising,
    Falling,
    Stable,
}

/// Advanced weather station with historical data
pub struct WeatherStation {
    pub id: String,
    pub location: (f64, f64), // latitude, longitude
    readings: Vec<WeatherReading>,
    metadata: HashMap<String, String>,
}

impl WeatherStation {
    /// Create a new weather station
    pub fn new(id: String, location: (f64, f64)) -> Self {
        Self {
            id,
            location,
            readings: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a weather reading
    pub fn add_reading(&mut self, reading: WeatherReading) {
        self.readings.push(reading);
        self.readings.sort_by_key(|r| r.timestamp);
    }
    
    /// Get readings within a time range
    pub fn get_readings_range(&self, start: u64, end: u64) -> Vec<&WeatherReading> {
        self.readings
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }
    
    /// Calculate statistics for the station
    pub fn statistics(&self) -> WeatherStatistics {
        if self.readings.is_empty() {
            return WeatherStatistics::default();
        }
        
        let temperatures: Vec<f32> = self.readings.iter().map(|r| r.temperature).collect();
        let humidity: Vec<f32> = self.readings.iter().map(|r| r.humidity).collect();
        
        WeatherStatistics {
            total_readings: self.readings.len(),
            avg_temperature: average(&temperatures),
            avg_humidity: average(&humidity),
            min_temperature: temperatures.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
            max_temperature: temperatures.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)),
        }
    }
}

impl WeatherAnalyzer for WeatherStation {
    fn analyze_temperature(&self, data: &[f32]) -> TemperatureAnalysis {
        if data.is_empty() {
            return TemperatureAnalysis {
                average: 0.0,
                min: 0.0,
                max: 0.0,
                trend: TemperatureTrend::Stable,
                variance: 0.0,
            };
        }
        
        let average = average(data);
        let min = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        // Calculate trend based on first and last third of data
        let trend = if data.len() >= 3 {
            let first_third = &data[0..data.len()/3];
            let last_third = &data[data.len()*2/3..];
            let first_avg = average(first_third);
            let last_avg = average(last_third);
            
            if last_avg > first_avg + 1.0 {
                TemperatureTrend::Rising
            } else if last_avg < first_avg - 1.0 {
                TemperatureTrend::Falling
            } else {
                TemperatureTrend::Stable
            }
        } else {
            TemperatureTrend::Stable
        };
        
        let variance = calculate_variance(data, average);
        
        TemperatureAnalysis {
            average,
            min,
            max,
            trend,
            variance,
        }
    }
    
    fn predict_conditions(&self, current: &WeatherReading) -> WeatherCondition {
        // Simplified prediction logic based on pressure and humidity
        match (current.pressure, current.humidity) {
            (p, h) if p < 1000.0 && h > 80.0 => WeatherCondition::Rainy { 
                intensity: if h > 90.0 { RainIntensity::Heavy } else { RainIntensity::Moderate }
            },
            (p, h) if p < 995.0 => WeatherCondition::Stormy { wind_speed: 30.0 },
            (_, h) if h < 30.0 => WeatherCondition::Clear,
            _ => WeatherCondition::Cloudy { coverage: 50 },
        }
    }
    
    fn comfort_index(&self, reading: &WeatherReading) -> f32 {
        let temp_comfort = calculate_temperature_comfort(reading.temperature);
        let humidity_comfort = calculate_humidity_comfort(reading.humidity);
        let condition_comfort = match reading.condition {
            WeatherCondition::Clear => 1.0,
            WeatherCondition::Cloudy { coverage } => 1.0 - (coverage as f32 / 200.0),
            WeatherCondition::Rainy { .. } => 0.3,
            WeatherCondition::Snowy { .. } => 0.4,
            WeatherCondition::Stormy { .. } => 0.1,
        };
        
        (temp_comfort + humidity_comfort + condition_comfort) / 3.0 * 100.0
    }
}

/// Statistical summary of weather data
#[derive(Debug, Default)]
pub struct WeatherStatistics {
    pub total_readings: usize,
    pub avg_temperature: f32,
    pub avg_humidity: f32,
    pub min_temperature: f32,
    pub max_temperature: f32,
}

impl fmt::Display for WeatherReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Weather at {}: {:.1}°C, {:.1}% humidity, {:?}", 
               self.timestamp, self.temperature, self.humidity, self.condition)
    }
}

// Helper functions

/// Calculate the average of a slice of f32 values
fn average(data: &[f32]) -> f32 {
    if data.is_empty() {
        0.0
    } else {
        data.iter().sum::<f32>() / data.len() as f32
    }
}

/// Calculate variance of data
fn calculate_variance(data: &[f32], mean: f32) -> f32 {
    if data.len() <= 1 {
        return 0.0;
    }
    
    let sum_squared_diff: f32 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum();
    
    sum_squared_diff / (data.len() - 1) as f32
}

/// Calculate temperature comfort score (0.0 to 1.0)
fn calculate_temperature_comfort(temp: f32) -> f32 {
    // Optimal temperature range: 18-24°C
    match temp {
        t if t >= 18.0 && t <= 24.0 => 1.0,
        t if t >= 15.0 && t < 18.0 => 0.8 - (18.0 - t) * 0.1,
        t if t > 24.0 && t <= 30.0 => 0.8 - (t - 24.0) * 0.1,
        t if t >= 10.0 && t < 15.0 => 0.5 - (15.0 - t) * 0.05,
        t if t > 30.0 && t <= 35.0 => 0.5 - (t - 30.0) * 0.05,
        _ => 0.0,
    }
}

/// Calculate humidity comfort score (0.0 to 1.0)  
fn calculate_humidity_comfort(humidity: f32) -> f32 {
    // Optimal humidity range: 40-60%
    match humidity {
        h if h >= 40.0 && h <= 60.0 => 1.0,
        h if h >= 30.0 && h < 40.0 => 0.8 - (40.0 - h) * 0.02,
        h if h > 60.0 && h <= 70.0 => 0.8 - (h - 60.0) * 0.02,
        h if h >= 20.0 && h < 30.0 => 0.5 - (30.0 - h) * 0.03,
        h if h > 70.0 && h <= 80.0 => 0.5 - (h - 70.0) * 0.03,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_station_creation() {
        let station = WeatherStation::new("STATION001".to_string(), (45.0, -75.0));
        assert_eq!(station.id, "STATION001");
        assert_eq!(station.location, (45.0, -75.0));
    }

    #[test]
    fn test_temperature_analysis() {
        let station = WeatherStation::new("TEST".to_string(), (0.0, 0.0));
        let data = vec![20.0, 22.0, 24.0, 23.0, 21.0];
        let analysis = station.analyze_temperature(&data);
        assert!(analysis.average > 21.0 && analysis.average < 23.0);
    }
}
```

### 1.2 Your First Extraction

Let's start with the most basic extraction:

```bash
rustex extract --pretty
```

**What happened?**
- RustEx scanned your project for Rust files
- It parsed the AST of each file
- It extracted information about functions, structs, enums, traits, and implementations
- It output the results in pretty-printed JSON format

**Key observations:**
- Notice the hierarchical structure showing relationships between elements
- Complexity metrics for functions
- Documentation extracted from doc comments
- Cross-references between related elements

### 1.3 Saving and Examining Output

```bash
rustex extract --include-docs --output weather-ast.json --pretty
```

Open `weather-ast.json` and examine:
- Project metadata (name, version, Rust edition)
- File-level information
- Individual code elements with their properties
- Overall project metrics

## Part 2: Understanding Output

### 2.1 JSON Structure Deep Dive

The RustEx JSON output has this structure:

```json
{
  "project": {
    "name": "rustex-tutorial",
    "version": "0.1.0", 
    "rust_edition": "2021"
  },
  "files": [
    {
      "path": "src/lib.rs",
      "elements": [
        {
          "id": "Function_analyze_temperature_1",
          "element_type": "Function",
          "name": "analyze_temperature",
          "signature": "fn analyze_temperature(&self, data: &[f32]) -> TemperatureAnalysis",
          "doc_comments": ["Analyze temperature trends over time"],
          "complexity": 8,
          "hierarchy": {
            "qualified_name": "crate::WeatherAnalyzer::analyze_temperature",
            "parent_id": "Trait_WeatherAnalyzer_1"
          }
        }
      ]
    }
  ],
  "metrics": {
    "total_functions": 15,
    "complexity_average": 4.2
  }
}
```

**Key fields explained:**

- **`element_type`**: The kind of Rust construct (Function, Struct, Enum, etc.)
- **`hierarchy`**: Shows parent-child relationships and qualified names
- **`complexity`**: Calculated complexity score for the element
- **`doc_comments`**: Extracted documentation
- **`cross_references`**: References to other elements

### 2.2 Markdown Output

Generate human-readable documentation:

```bash
rustex extract --format markdown --include-docs --output weather-docs.md
```

**When to use Markdown:**
- Generating project documentation
- Creating README files
- Publishing to documentation sites
- Code reviews and presentations

### 2.3 Specialized Formats

Try the RAG-optimized format:

```bash
rustex extract --format rag --include-docs --output weather-rag.json --pretty
```

**RAG format features:**
- Chunked data optimized for embedding models
- Semantic grouping of related elements
- Metadata for retrieval systems
- Token count estimates

## Part 3: Configuration Mastery

### 3.1 Creating Your First Configuration

```bash
rustex config init --template documentation
```

This creates `rustex.toml`:

```toml
[extraction]
include_docs = true
include_private = true
output_format = "markdown"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["tests/**", "benches/**"]

[output.markdown]
include_toc = true
syntax_highlighting = true
```

### 3.2 Understanding Configuration Sections

#### Extraction Settings
Control what gets extracted:

```toml
[extraction]
include_docs = true          # Include doc comments
include_private = true       # Include private items
parse_dependencies = false   # Parse external dependencies
max_file_size = "10MB"      # Skip files larger than this
include_tests = false       # Include test code
```

#### File Filtering
Control which files are processed:

```toml
[filters]
include = [
    "src/**/*.rs",
    "lib/**/*.rs",
    "examples/**/*.rs"
]
exclude = [
    "target/**",
    "tests/**",
    "benches/**",
    "**/*_test.rs"
]
```

#### Output Configuration
Control output formatting:

```toml
[output]
pretty_print = true
include_metrics = true
include_hierarchy = true

[output.markdown]
include_toc = true
toc_depth = 3
syntax_highlighting = true
```

### 3.3 Use Case Configurations

Create specialized configurations:

```bash
# For LLM training data
rustex config init --template llm-training --output llm-config.toml

# For code analysis
rustex config init --template code-analysis --output analysis-config.toml

# For testing
rustex config init --template testing --output test-config.toml
```

### 3.4 Using Configurations

```bash
# Use specific configuration
rustex extract --config llm-config.toml --output llm-data.json

# Override configuration settings
rustex extract --config analysis-config.toml --include-private --output detailed-analysis.json
```

### 3.5 Configuration Validation

Always validate your configurations:

```bash
rustex config validate
rustex config validate --file llm-config.toml
```

## Part 4: Advanced Analysis

### 4.1 Complexity Analysis

Let's analyze the complexity of our weather library:

```bash
rustex extract --include-private --output complexity-analysis.json --pretty
```

Look for:
- Functions with high complexity (>10)
- Patterns in complexity distribution
- Relationship between function length and complexity

**Exercise**: Find the most complex function in your project. Can you see why it has high complexity?

### 4.2 Documentation Coverage Analysis

```bash
rustex extract --include-docs --include-private --output doc-coverage.json --pretty
```

Create a simple analysis script:

```bash
# Count documented vs undocumented public functions
jq '.files[].elements[] | select(.element_type == "Function" and .visibility == "Public") | {name: .name, has_docs: (.doc_comments | length > 0)}' doc-coverage.json
```

### 4.3 Architecture Analysis

Analyze the structure of your project:

```bash
rustex extract --include-private --parse-deps --output architecture.json --pretty
```

Look for:
- Module organization and hierarchy
- Dependencies between components
- Public API surface area
- Cross-references and coupling

### 4.4 Metrics Collection

Get comprehensive metrics:

```bash
rustex metrics --complexity --loc --output project-metrics.json
```

The metrics include:
- Lines of code statistics
- Function count and distribution
- Complexity metrics and distribution
- File-level breakdowns

## Part 5: LLM Integration

### 5.1 Preparing Training Data

Create an LLM training configuration:

```toml
# llm-training.toml
[extraction]
include_docs = true
include_private = false
parse_dependencies = true
output_format = "rag"

[filters]
include = ["src/**/*.rs", "examples/**/*.rs"]
exclude = ["tests/**", "benches/**"]

[output.rag]
max_chunk_size = 1000
chunk_overlap = 100
include_semantics = true
```

Generate training data:

```bash
rustex extract --config llm-training.toml --output training-data.json --pretty
```

### 5.2 Creating Question-Answer Pairs

Use the example script to create Q&A pairs from your code:

```bash
cargo run --example llm_data_prep
```

This generates:
- Training examples for different code patterns
- Question-answer pairs from documentation
- Chunked data for RAG systems
- Metadata for training optimization

### 5.3 Fine-tuning Data Format

Convert to JSONL format for training:

```bash
jq -c '.examples[]' training-data.json > training-examples.jsonl
```

### 5.4 Embedding Preparation

Extract just the content for embedding:

```bash
jq -r '.chunks[] | .content' training-data.json > content-for-embedding.txt
```

## Part 6: Automation and CI/CD

### 6.1 Git Hooks Integration

Create a pre-commit hook to check code quality:

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running RustEx analysis..."
rustex extract --include-private --output pre-commit-analysis.json

# Check complexity
AVG_COMPLEXITY=$(jq '.metrics.complexity_average' pre-commit-analysis.json)
if (( $(echo "$AVG_COMPLEXITY > 10.0" | bc -l) )); then
    echo "❌ Average complexity too high: $AVG_COMPLEXITY"
    exit 1
fi

echo "✅ Code quality check passed"
```

### 6.2 GitHub Actions Integration

Create `.github/workflows/rustex.yml`:

```yaml
name: RustEx Analysis

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  analyze:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install RustEx
      run: cargo install rustex-cli
    
    - name: Run Analysis
      run: |
        rustex extract --include-private --output analysis.json
        rustex metrics --complexity --loc --output metrics.json
    
    - name: Check Quality Gates
      run: |
        # Check average complexity
        COMPLEXITY=$(jq '.complexity_average' metrics.json)
        if (( $(echo "$COMPLEXITY > 8.0" | bc -l) )); then
          echo "❌ Complexity too high: $COMPLEXITY"
          exit 1
        fi
        
        # Check documentation coverage
        DOC_FUNCTIONS=$(jq '[.files[].elements[] | select(.element_type == "Function" and .visibility == "Public" and (.doc_comments | length > 0))] | length' analysis.json)
        TOTAL_FUNCTIONS=$(jq '[.files[].elements[] | select(.element_type == "Function" and .visibility == "Public")] | length' analysis.json)
        
        if [ "$TOTAL_FUNCTIONS" -gt 0 ]; then
          COVERAGE=$(echo "scale=2; $DOC_FUNCTIONS * 100 / $TOTAL_FUNCTIONS" | bc)
          echo "Documentation coverage: $COVERAGE%"
          
          if (( $(echo "$COVERAGE < 80.0" | bc -l) )); then
            echo "❌ Documentation coverage too low: $COVERAGE%"
            exit 1
          fi
        fi
    
    - name: Upload Results
      uses: actions/upload-artifact@v3
      with:
        name: rustex-analysis
        path: |
          analysis.json
          metrics.json
```

### 6.3 Build Script Integration

Add to your `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Generate documentation during build
    let output = Command::new("rustex")
        .args(&["extract", "--format", "markdown", "--include-docs", "--output", "target/generated-docs.md"])
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            println!("cargo:rerun-if-changed=src/");
            println!("cargo:warning=Generated documentation at target/generated-docs.md");
        }
        _ => {
            println!("cargo:warning=Failed to generate documentation");
        }
    }
}
```

### 6.4 Documentation Deployment

Create a script to update documentation:

```bash
#!/bin/bash
# deploy-docs.sh

echo "Generating documentation..."
rustex extract --format markdown --include-docs --output README.md

echo "Generating API documentation..."
rustex extract --format json --include-docs --output api-spec.json

echo "Uploading to documentation site..."
# Upload to your documentation hosting service
```

## Part 7: Plugin Development

### 7.1 Understanding the Plugin System

RustEx uses a plugin architecture that allows you to:
- Add custom analysis
- Transform AST data
- Generate custom outputs
- Integrate with external tools

### 7.2 Creating a Simple Plugin

Create `custom-analyzer/Cargo.toml`:

```toml
[package]
name = "custom-analyzer"
version = "0.1.0"
edition = "2021"

[dependencies]
rustex-plugins = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[lib]
crate-type = ["cdylib", "rlib"]
```

Create `custom-analyzer/src/lib.rs`:

```rust
use rustex_plugins::{Plugin, PluginContext, PluginOutput, PluginError, PluginMetadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    pub check_naming: bool,
    pub max_function_length: usize,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            check_naming: true,
            max_function_length: 50,
        }
    }
}

pub struct CustomAnalyzer {
    config: CustomConfig,
}

impl Plugin for CustomAnalyzer {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "custom-analyzer".to_string(),
            version: "0.1.0".to_string(),
            description: "Custom code analysis plugin".to_string(),
            author: "Your Name".to_string(),
            capabilities: vec!["analysis".to_string()],
        }
    }

    fn analyze(&mut self, context: &mut PluginContext) -> Result<PluginOutput, PluginError> {
        let mut output = PluginOutput::new();
        
        if let Some(project_ast) = context.project_ast() {
            let mut naming_issues = 0;
            let mut long_functions = 0;
            
            for file in &project_ast.files {
                for element in &file.elements {
                    // Check naming conventions
                    if self.config.check_naming && !follows_naming_convention(element) {
                        naming_issues += 1;
                        output.add_warning(format!(
                            "Naming convention issue in {}: {}",
                            element.name, file.relative_path.display()
                        ));
                    }
                    
                    // Check function length
                    if let Some(metrics) = &element.complexity_metrics {
                        if metrics.lines_of_code > self.config.max_function_length as u32 {
                            long_functions += 1;
                            output.add_warning(format!(
                                "Function {} is too long: {} lines",
                                element.name, metrics.lines_of_code
                            ));
                        }
                    }
                }
            }
            
            output.add_metric("naming_issues", naming_issues as f64);
            output.add_metric("long_functions", long_functions as f64);
        }
        
        Ok(output)
    }
}

fn follows_naming_convention(element: &rustex_core::CodeElement) -> bool {
    match element.element_type {
        rustex_core::ElementType::Function => {
            // Functions should be snake_case
            element.name.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
        }
        rustex_core::ElementType::Struct | rustex_core::ElementType::Enum => {
            // Types should be PascalCase
            element.name.chars().next().map_or(false, |c| c.is_uppercase())
        }
        _ => true, // Other elements can use any naming
    }
}

#[no_mangle]
pub fn create_plugin(config_json: &str) -> Result<Box<dyn Plugin>, PluginError> {
    let config: CustomConfig = serde_json::from_str(config_json)
        .map_err(|e| PluginError::ConfigurationError(e.to_string()))?;
    Ok(Box::new(CustomAnalyzer { config }))
}
```

### 7.3 Testing Your Plugin

```rust
// custom-analyzer/tests/integration_test.rs
use custom_analyzer::*;
use rustex_plugins::{Plugin, PluginContext};

#[test]
fn test_plugin_basic() {
    let config = CustomConfig::default();
    let mut plugin = CustomAnalyzer { config };
    
    let metadata = plugin.metadata();
    assert_eq!(metadata.name, "custom-analyzer");
}
```

### 7.4 Using Your Plugin

Build and use your plugin:

```bash
cd custom-analyzer
cargo build --release

# Copy plugin to RustEx plugin directory
mkdir -p ~/.rustex/plugins
cp target/release/libcustom_analyzer.so ~/.rustex/plugins/

# Use the plugin
rustex extract --plugins custom-analyzer --output custom-analysis.json
```

## Part 8: Best Practices

### 8.1 Performance Optimization

**For Large Projects:**

```toml
[extraction]
max_file_size = "5MB"
parse_dependencies = false

[filters]
exclude = [
    "target/**",
    "vendor/**", 
    "**/*_generated.rs"
]
```

**Memory Management:**
- Process files in batches for very large codebases
- Use streaming APIs when available
- Exclude unnecessary files early

### 8.2 Configuration Management

**Project-Specific Configs:**
- Keep `rustex.toml` in version control
- Use different configs for different purposes
- Document why specific settings are used

**Environment-Specific Settings:**
```bash
# Development
export RUSTEX_INCLUDE_PRIVATE=true

# CI/CD  
export RUSTEX_INCLUDE_PRIVATE=false
export RUSTEX_OUTPUT_FORMAT=json
```

### 8.3 Data Management

**Organizing Output:**
```bash
mkdir -p analysis/{json,markdown,metrics}

# Separate outputs by type
rustex extract --format json --output analysis/json/full.json
rustex extract --format markdown --output analysis/markdown/docs.md
rustex metrics --output analysis/metrics/project.json
```

**Version Control:**
- Don't commit large JSON outputs
- Do commit configurations and scripts
- Use `.gitignore` for generated files

### 8.4 Integration Patterns

**Makefile Integration:**
```makefile
.PHONY: analyze docs metrics

analyze:
	rustex extract --include-private --output analysis.json

docs:
	rustex extract --format markdown --include-docs --output docs.md

metrics:
	rustex metrics --complexity --loc --output metrics.json

quality-check: analyze
	@echo "Checking code quality..."
	@COMPLEXITY=$$(jq '.metrics.complexity_average' analysis.json); \
	if (( $$(echo "$$COMPLEXITY > 8.0" | bc -l) )); then \
		echo "❌ Complexity too high: $$COMPLEXITY"; \
		exit 1; \
	else \
		echo "✅ Quality check passed"; \
	fi
```

**Docker Integration:**
```dockerfile
FROM rust:1.70

RUN cargo install rustex-cli

WORKDIR /app
COPY . .

RUN rustex extract --output analysis.json
RUN rustex metrics --output metrics.json
```

### 8.5 Error Handling

**Graceful Degradation:**
```bash
# Handle partial failures
rustex extract --output analysis.json 2>errors.log || true

# Check for critical errors
if grep -q "CRITICAL" errors.log; then
    echo "Critical errors found"
    exit 1
fi
```

**Monitoring and Alerting:**
```bash
# Check analysis health
TOTAL_FILES=$(jq '.files | length' analysis.json)
if [ "$TOTAL_FILES" -lt 1 ]; then
    echo "❌ No files analyzed - check configuration"
    exit 1
fi
```

### 8.6 Documentation Workflows

**Automated Documentation:**
```yaml
# .github/workflows/docs.yml
name: Update Documentation

on:
  push:
    branches: [ main ]
    paths: [ 'src/**/*.rs' ]

jobs:
  update-docs:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Generate Documentation
      run: |
        rustex extract --format markdown --include-docs --output README.md
        
    - name: Commit Documentation
      run: |
        git config --local user.email "action@github.com"
        git config --local user.name "GitHub Action"
        git add README.md
        git diff --staged --quiet || git commit -m "Update documentation [skip ci]"
        git push
```

## Conclusion

You've now learned how to:

✅ **Extract comprehensive AST data** from Rust projects  
✅ **Configure RustEx** for different use cases  
✅ **Analyze code quality** and complexity  
✅ **Prepare data** for LLM training  
✅ **Integrate RustEx** into development workflows  
✅ **Create custom plugins** for specialized analysis  
✅ **Follow best practices** for performance and maintainability  

### Next Steps

1. **Apply to Your Projects**: Use RustEx on your own Rust projects
2. **Explore Advanced Features**: Try plugin development and custom analysis
3. **Integrate Workflows**: Add RustEx to your CI/CD pipelines
4. **Contribute**: Share your configurations and plugins with the community

### Resources

- [API Reference](api-reference.md) - Detailed API documentation
- [Configuration Reference](configuration-reference.md) - Complete configuration guide
- [Plugin Development](plugin-development.md) - Creating custom plugins
- [Examples](../examples/) - More code examples and patterns
- [GitHub Repository](https://github.com/your-username/rustex) - Source code and issues

---

**Happy analyzing!** 🦀✨