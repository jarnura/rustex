#!/bin/bash
# RustEx CLI Usage Examples
# 
# This script demonstrates various ways to use the RustEx CLI tool
# for different scenarios and use cases.

set -e

echo "🦀 RustEx CLI Examples"
echo "═══════════════════════"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}📋 $1${NC}"
    echo "────────────────────────"
}

# Function to print commands before executing
run_command() {
    echo -e "\n${YELLOW}$ $1${NC}"
    eval $1
}

# Function to check if rustex is installed
check_rustex() {
    if ! command -v rustex &> /dev/null; then
        echo -e "${RED}❌ RustEx not found. Please install it first:${NC}"
        echo "cargo install rustex-cli"
        exit 1
    fi
    echo -e "${GREEN}✅ RustEx found: $(rustex --version)${NC}"
}

# Create a sample project for demonstrations
create_sample_project() {
    print_section "Creating Sample Project"
    
    if [ -d "rustex-demo" ]; then
        rm -rf rustex-demo
    fi
    
    run_command "cargo new --lib rustex-demo"
    cd rustex-demo
    
    # Create a more complex lib.rs
    cat > src/lib.rs << 'EOF'
//! RustEx Demo Library
//! 
//! A sample library to demonstrate RustEx capabilities.

use std::collections::HashMap;

/// Calculates the factorial of a number using recursion
/// 
/// # Arguments
/// 
/// * `n` - The number to calculate factorial for
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(factorial(5), 120);
/// assert_eq!(factorial(0), 1);
/// ```
/// 
/// # Panics
/// 
/// This function will panic if `n` is greater than 20.
pub fn factorial(n: u64) -> u64 {
    if n > 20 {
        panic!("Input too large for factorial calculation");
    }
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

/// Represents different types of geometric shapes
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A circle with radius
    Circle { radius: f64 },
    /// A rectangle with width and height
    Rectangle { width: f64, height: f64 },
    /// A triangle with base and height
    Triangle { base: f64, height: f64 },
}

/// Trait for calculating geometric properties
pub trait Geometry {
    /// Calculate the area of the shape
    fn area(&self) -> f64;
    
    /// Calculate the perimeter of the shape
    fn perimeter(&self) -> f64;
    
    /// Get a description of the shape
    fn description(&self) -> String {
        format!("A geometric shape with area {:.2}", self.area())
    }
}

impl Geometry for Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
    
    fn perimeter(&self) -> f64 {
        match self {
            Shape::Circle { radius } => 2.0 * std::f64::consts::PI * radius,
            Shape::Rectangle { width, height } => 2.0 * (width + height),
            Shape::Triangle { base, height } => {
                // Simplified calculation assuming isosceles triangle
                let side = ((height * height) + (base * base / 4.0)).sqrt();
                base + 2.0 * side
            }
        }
    }
}

/// A collection manager for geometric shapes
pub struct ShapeCollection {
    shapes: Vec<Shape>,
    metadata: HashMap<String, String>,
}

impl ShapeCollection {
    /// Create a new empty shape collection
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a shape to the collection
    pub fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }
    
    /// Get the total area of all shapes
    pub fn total_area(&self) -> f64 {
        self.shapes.iter().map(|s| s.area()).sum()
    }
    
    /// Find shapes with area greater than threshold
    pub fn find_large_shapes(&self, threshold: f64) -> Vec<&Shape> {
        self.shapes
            .iter()
            .filter(|s| s.area() > threshold)
            .collect()
    }
    
    /// Add metadata to the collection
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get collection statistics
    pub fn stats(&self) -> CollectionStats {
        let total_area = self.total_area();
        let count = self.shapes.len();
        let avg_area = if count > 0 { total_area / count as f64 } else { 0.0 };
        
        let mut shape_counts = HashMap::new();
        for shape in &self.shapes {
            let shape_type = match shape {
                Shape::Circle { .. } => "Circle",
                Shape::Rectangle { .. } => "Rectangle", 
                Shape::Triangle { .. } => "Triangle",
            };
            *shape_counts.entry(shape_type.to_string()).or_insert(0) += 1;
        }
        
        CollectionStats {
            total_shapes: count,
            total_area,
            average_area: avg_area,
            shape_types: shape_counts,
        }
    }
}

impl Default for ShapeCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a shape collection
#[derive(Debug)]
pub struct CollectionStats {
    pub total_shapes: usize,
    pub total_area: f64,
    pub average_area: f64,
    pub shape_types: HashMap<String, usize>,
}

/// Private helper function for complex calculations
fn complex_calculation(data: &[f64]) -> f64 {
    let mut result = 0.0;
    for (i, &value) in data.iter().enumerate() {
        for j in 0..10 {
            if i % 2 == 0 {
                if j < 5 {
                    result += value * (j as f64);
                } else {
                    result -= value / (j as f64);
                }
            } else {
                match j {
                    0..=2 => result *= 1.1,
                    3..=6 => result += value * 0.5,
                    _ => result -= value * 0.2,
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_shape_area() {
        let circle = Shape::Circle { radius: 1.0 };
        assert!((circle.area() - std::f64::consts::PI).abs() < 0.001);
        
        let rect = Shape::Rectangle { width: 2.0, height: 3.0 };
        assert_eq!(rect.area(), 6.0);
    }
}
EOF

    # Create a module file
    mkdir -p src/utils
    cat > src/utils/mod.rs << 'EOF'
//! Utility functions for the demo library

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees  
pub fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Calculate the distance between two points
pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
EOF

    # Update main lib.rs to include the module
    echo -e "\npub mod utils;" >> src/lib.rs
    
    echo -e "${GREEN}✅ Sample project created${NC}"
}

# Basic usage examples
basic_examples() {
    print_section "Basic Usage Examples"
    
    echo "1. Simple AST extraction with pretty output"
    run_command "rustex extract --pretty"
    
    echo -e "\n2. Extract and save to file"
    run_command "rustex extract --output basic-ast.json --pretty"
    echo -e "${GREEN}📄 Output saved to basic-ast.json${NC}"
    
    echo -e "\n3. Include documentation in extraction"
    run_command "rustex extract --include-docs --pretty"
    
    echo -e "\n4. Generate markdown documentation"
    run_command "rustex extract --format markdown --include-docs --output docs.md"
    echo -e "${GREEN}📄 Documentation saved to docs.md${NC}"
}

# Advanced filtering examples
filtering_examples() {
    print_section "File Filtering Examples"
    
    echo "1. Include only specific files"
    run_command "rustex extract --include 'src/lib.rs' --pretty"
    
    echo -e "\n2. Exclude test files"
    run_command "rustex extract --exclude 'src/**/*test*' --pretty"
    
    echo -e "\n3. Multiple include patterns"
    run_command "rustex extract --include 'src/**/*.rs,lib/**/*.rs' --pretty"
    
    echo -e "\n4. Complex filtering"
    run_command "rustex extract --include 'src/**/*.rs' --exclude 'src/test*,**/*_test.rs' --pretty"
}

# Output format examples
format_examples() {
    print_section "Output Format Examples"
    
    echo "1. JSON format (default)"
    run_command "rustex extract --format json --output output.json --pretty"
    
    echo -e "\n2. Markdown format"
    run_command "rustex extract --format markdown --include-docs --output output.md"
    
    echo -e "\n3. MessagePack format (binary)"
    run_command "rustex extract --format messagepack --output output.msgpack"
    
    echo -e "\n4. RAG-optimized format"
    run_command "rustex extract --format rag --include-docs --output rag-data.json --pretty"
}

# Configuration examples
config_examples() {
    print_section "Configuration Examples"
    
    echo "1. Initialize default configuration"
    run_command "rustex config init"
    echo -e "${GREEN}📄 Configuration created: rustex.toml${NC}"
    
    echo -e "\n2. Initialize with documentation template"
    run_command "rustex config init --template documentation --output doc-config.toml"
    
    echo -e "\n3. Initialize with LLM training template"
    run_command "rustex config init --template llm-training --output llm-config.toml"
    
    echo -e "\n4. Validate configuration"
    run_command "rustex config validate"
    
    echo -e "\n5. Show current configuration"
    run_command "rustex config show"
    
    echo -e "\n6. Use specific configuration file"
    run_command "rustex extract --config doc-config.toml --output doc-extract.json --pretty"
}

# Metrics and analysis examples
metrics_examples() {
    print_section "Metrics and Analysis Examples"
    
    echo "1. Basic project metrics"
    run_command "rustex metrics"
    
    echo -e "\n2. Complexity analysis"
    run_command "rustex metrics --complexity"
    
    echo -e "\n3. Lines of code analysis"
    run_command "rustex metrics --loc"
    
    echo -e "\n4. Comprehensive metrics with output"
    run_command "rustex metrics --complexity --loc --output metrics.json"
    echo -e "${GREEN}📄 Metrics saved to metrics.json${NC}"
}

# Plugin examples (when available)
plugin_examples() {
    print_section "Plugin Examples"
    
    echo "Note: Plugin system is available but built-in plugins are still in development"
    
    echo -e "\n1. List available plugins"
    echo "$ rustex plugins list"
    echo "(Not implemented yet)"
    
    echo -e "\n2. Extract with complexity analysis plugin"
    echo "$ rustex extract --plugins complexity-analyzer --output analysis.json"
    echo "(Plugin not available yet)"
    
    echo -e "\n3. Extract with multiple plugins"
    echo "$ rustex extract --plugins 'complexity-analyzer,doc-enhancer' --output enhanced.json"
    echo "(Plugins not available yet)"
}

# Performance and optimization examples
performance_examples() {
    print_section "Performance and Optimization Examples"
    
    echo "1. Set maximum file size limit"
    run_command "rustex extract --max-file-size 5MB --pretty"
    
    echo -e "\n2. Exclude large directories for faster processing"
    run_command "rustex extract --exclude 'target/**,vendor/**' --pretty"
    
    echo -e "\n3. Process only essential files"
    run_command "rustex extract --include 'src/lib.rs,src/main.rs' --pretty"
}

# CI/CD integration examples
cicd_examples() {
    print_section "CI/CD Integration Examples"
    
    echo "1. Extract for analysis (non-pretty for parsing)"
    run_command "rustex extract --include-private --output ci-analysis.json"
    
    echo -e "\n2. Generate documentation for deployment"
    run_command "rustex extract --format markdown --include-docs --output deployment-docs.md"
    
    echo -e "\n3. Check project metrics"
    run_command "rustex metrics --complexity --loc --output ci-metrics.json"
    
    # Example of using output in CI
    echo -e "\n4. Example CI script usage:"
    cat << 'EOF'
#!/bin/bash
# CI script example
rustex extract --include-private --output analysis.json

# Check if analysis was successful
if [ $? -eq 0 ]; then
    echo "✅ AST extraction successful"
    
    # Parse metrics (requires jq)
    if command -v jq &> /dev/null; then
        COMPLEXITY=$(jq '.metrics.complexity_average' analysis.json)
        echo "Average complexity: $COMPLEXITY"
        
        # Fail CI if complexity is too high
        if (( $(echo "$COMPLEXITY > 15.0" | bc -l) )); then
            echo "❌ Complexity too high: $COMPLEXITY"
            exit 1
        fi
    fi
else
    echo "❌ AST extraction failed"
    exit 1
fi
EOF
}

# Error handling examples
error_examples() {
    print_section "Error Handling and Debugging Examples"
    
    echo "1. Verbose output for debugging"
    run_command "rustex extract --verbose --pretty 2>&1 | head -20"
    
    echo -e "\n2. Extract from non-existent directory (will show error)"
    echo "$ rustex extract --path /nonexistent/path --pretty"
    echo "(This would show an error - not running to avoid failure)"
    
    echo -e "\n3. Validate configuration (current config should be valid)"
    run_command "rustex config validate"
}

# Cleanup function
cleanup() {
    print_section "Cleaning Up"
    
    echo "Removing generated files..."
    rm -f *.json *.md *.msgpack *.toml
    
    echo "Going back to original directory..."
    cd ..
    
    echo "Removing sample project..."
    rm -rf rustex-demo
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Help and documentation examples
help_examples() {
    print_section "Help and Documentation"
    
    echo "1. General help"
    run_command "rustex --help"
    
    echo -e "\n2. Extract command help"
    run_command "rustex extract --help"
    
    echo -e "\n3. Config command help"
    run_command "rustex config --help"
    
    echo -e "\n4. Version information"
    run_command "rustex --version"
}

# Main execution
main() {
    echo "This script demonstrates various RustEx CLI usage patterns."
    echo "Press Enter to continue or Ctrl+C to exit..."
    read -r
    
    check_rustex
    
    create_sample_project
    
    basic_examples
    echo -e "\nPress Enter to continue to filtering examples..."
    read -r
    
    filtering_examples
    echo -e "\nPress Enter to continue to format examples..."
    read -r
    
    format_examples
    echo -e "\nPress Enter to continue to configuration examples..."
    read -r
    
    config_examples
    echo -e "\nPress Enter to continue to metrics examples..."
    read -r
    
    metrics_examples
    echo -e "\nPress Enter to continue to plugin examples..."
    read -r
    
    plugin_examples
    echo -e "\nPress Enter to continue to performance examples..."
    read -r
    
    performance_examples
    echo -e "\nPress Enter to continue to CI/CD examples..."
    read -r
    
    cicd_examples
    echo -e "\nPress Enter to continue to error handling examples..."
    read -r
    
    error_examples
    echo -e "\nPress Enter to continue to help examples..."
    read -r
    
    help_examples
    
    echo -e "\nPress Enter to clean up and finish..."
    read -r
    
    cleanup
    
    echo -e "\n${GREEN}🎉 RustEx CLI examples complete!${NC}"
    echo "For more information, visit: https://github.com/your-username/rustex"
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi