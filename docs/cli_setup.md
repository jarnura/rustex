// ============================================================================
// crates/rustex-cli/Cargo.toml
// ============================================================================
/*
[package]
name = "rustex-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rustex"
path = "src/main.rs"

[dependencies]
rustex-core = { path = "../rustex-core" }
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
serde_json = "1.0"
colored = "2.0"
indicatif = "0.17"
*/

// ============================================================================
// crates/rustex-cli/src/main.rs
// ============================================================================

use anyhow::Result;
use clap::{Parser, Subcommand};
use rustex_core::{AstExtractor, ExtractorConfig, OutputFormat};
use std::path::PathBuf;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "rustex")]
#[command(about = "A comprehensive Rust AST extractor for LLM and RAG applications")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract AST from Rust project
    Extract {
        /// Output format
        #[arg(short, long, value_enum, default_value = "json")]
        format: CliOutputFormat,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include documentation
        #[arg(long)]
        include_docs: bool,
        
        /// Include private items
        #[arg(long)]
        include_private: bool,
        
        /// Parse dependencies
        #[arg(long)]
        parse_deps: bool,
        
        /// Maximum file size in bytes
        #[arg(long, default_value = "10485760")] // 10MB
        max_file_size: usize,
        
        /// Files to include (glob patterns)
        #[arg(long, value_delimiter = ',')]
        include: Vec<String>,
        
        /// Files to exclude (glob patterns)
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
        
        /// Enable plugins
        #[arg(long, value_delimiter = ',')]
        plugins: Vec<String>,
        
        /// Pretty print JSON output
        #[arg(long)]
        pretty: bool,
    },
    
    /// Analyze project dependencies
    Deps {
        /// Visualize dependencies
        #[arg(long)]
        visualize: bool,
        
        /// Output file for visualization
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Calculate project metrics
    Metrics {
        /// Include complexity analysis
        #[arg(long)]
        complexity: bool,
        
        /// Include lines of code
        #[arg(long)]
        loc: bool,
        
        /// Output file for metrics
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Initialize configuration file
    Init {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum CliOutputFormat {
    Json,
    MessagePack,
    Markdown,
    GraphQL,
    Rag,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(cli_format: CliOutputFormat) -> Self {
        match cli_format {
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::MessagePack => OutputFormat::MessagePack,
            CliOutputFormat::Markdown => OutputFormat::Markdown,
            CliOutputFormat::GraphQL => OutputFormat::GraphQL,
            CliOutputFormat::Rag => OutputFormat::Rag,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("rustex={}", log_level))
        .init();
    
    match cli.command {
        Commands::Extract { 
            format, 
            output, 
            include_docs,
            include_private,
            parse_deps,
            max_file_size,
            include,
            exclude,
            plugins,
            pretty,
        } => {
            extract_command(
                cli.path,
                format.into(),
                output,
                include_docs,
                include_private,
                parse_deps,
                max_file_size,
                include,
                exclude,
                plugins,
                pretty,
            ).await?;
        }
        Commands::Deps { visualize, output } => {
            deps_command(cli.path, visualize, output).await?;
        }
        Commands::Metrics { complexity, loc, output } => {
            metrics_command(cli.path, complexity, loc, output).await?;
        }
        Commands::Init { force } => {
            init_command(cli.path, force).await?;
        }
    }
    
    Ok(())
}

async fn extract_command(
    project_path: PathBuf,
    format: OutputFormat,
    output: Option<PathBuf>,
    include_docs: bool,
    include_private: bool,
    parse_deps: bool,
    max_file_size: usize,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    plugins: Vec<String>,
    pretty: bool,
) -> Result<()> {
    info!("Starting AST extraction for project at {:?}", project_path);
    
    let config = create_config(
        format,
        include_docs,
        include_private,
        parse_deps,
        max_file_size,
        include_patterns,
        exclude_patterns,
        plugins,
    );
    
    let extractor = AstExtractor::new(config, project_path);
    
    // Show progress bar
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Extracting AST...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    
    let ast_data = match extractor.extract_project() {
        Ok(data) => {
            pb.finish_with_message("✓ AST extraction completed");
            data
        }
        Err(e) => {
            pb.finish_with_message("✗ AST extraction failed");
            error!("Extraction failed: {}", e);
            return Err(e);
        }
    };
    
    // Output results
    let output_content = match format {
        OutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&ast_data)?
            } else {
                serde_json::to_string(&ast_data)?
            }
        }
        OutputFormat::Markdown => {
            generate_markdown_output(&ast_data)?
        }
        _ => {
            error!("Output format not yet implemented");
            return Ok(());
        }
    };
    
    match output {
        Some(path) => {
            std::fs::write(&path, output_content)?;
            println!("✓ Output written to {}", path.display());
        }
        None => {
            println!("{}", output_content);
        }
    }
    
    // Print summary
    print_extraction_summary(&ast_data);
    
    Ok(())
}

async fn deps_command(
    _project_path: PathBuf,
    _visualize: bool,
    _output: Option<PathBuf>,
) -> Result<()> {
    println!("🚧 Dependency analysis not yet implemented");
    Ok(())
}

async fn metrics_command(
    _project_path: PathBuf,
    _complexity: bool,
    _loc: bool,
    _output: Option<PathBuf>,
) -> Result<()> {
    println!("🚧 Metrics analysis not yet implemented");
    Ok(())
}

async fn init_command(project_path: PathBuf, force: bool) -> Result<()> {
    let config_path = project_path.join("rustex.toml");
    
    if config_path.exists() && !force {
        error!("Configuration file already exists. Use --force to overwrite.");
        return Ok(());
    }
    
    let default_config = r#"[extraction]
include_docs = true
include_private = false
parse_dependencies = false
max_file_size = "10MB"
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**"]

[plugins]
enabled = []

# Plugin configurations can be added here
"#;
    
    std::fs::write(&config_path, default_config)?;
    println!("✓ Created configuration file at {}", config_path.display());
    
    Ok(())
}

fn create_config(
    format: OutputFormat,
    include_docs: bool,
    include_private: bool,
    parse_deps: bool,
    max_file_size: usize,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    plugins: Vec<String>,
) -> ExtractorConfig {
    let mut config = ExtractorConfig::default();
    
    config.output_format = format;
    config.include_docs = include_docs;
    config.include_private = include_private;
    config.parse_dependencies = parse_deps;
    config.max_file_size = max_file_size;
    config.plugins = plugins;
    
    if !include_patterns.is_empty() {
        config.filters.include = include_patterns;
    }
    
    if !exclude_patterns.is_empty() {
        config.filters.exclude = exclude_patterns;
    }
    
    config
}

fn generate_markdown_output(ast_data: &rustex_core::ProjectAst) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("# {} AST Analysis\n\n", ast_data.project.name));
    output.push_str(&format!("**Version:** {}\n", ast_data.project.version));
    output.push_str(&format!("**Rust Edition:** {}\n", ast_data.project.rust_edition));
    output.push_str(&format!("**Extracted:** {}\n\n", ast_data.extracted_at.format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Project metrics
    output.push_str("## Project Metrics\n\n");
    output.push_str(&format!("- **Total Files:** {}\n", ast_data.metrics.total_files));
    output.push_str(&format!("- **Total Lines:** {}\n", ast_data.metrics.total_lines));
    output.push_str(&format!("- **Functions:** {}\n", ast_data.metrics.total_functions));
    output.push_str(&format!("- **Structs:** {}\n", ast_data.metrics.total_structs));
    output.push_str(&format!("- **Enums:** {}\n", ast_data.metrics.total_enums));
    output.push_str(&format!("- **Traits:** {}\n", ast_data.metrics.total_traits));
    output.push_str(&format!("- **Average Complexity:** {:.2}\n\n", ast_data.metrics.complexity_average));
    
    // File breakdown
    output.push_str("## Files\n\n");
    for file in &ast_data.files {
        output.push_str(&format!("### {}\n\n", file.relative_path.display()));
        
        for element in &file.elements {
            output.push_str(&format!("#### {} `{}`\n\n", 
                format!("{:?}", element.element_type), 
                element.name
            ));
            
            if !element.doc_comments.is_empty() {
                output.push_str("**Documentation:**\n");
                for doc in &element.doc_comments {
                    output.push_str(&format!("> {}\n", doc));
                }
                output.push_str("\n");
            }
            
            if let Some(ref signature) = element.signature {
                output.push_str(&format!("```rust\n{}\n```\n\n", signature));
            }
        }
    }
    
    Ok(output)
}

fn print_extraction_summary(ast_data: &rustex_core::ProjectAst) {
    use colored::*;
    
    println!("\n{}", "📊 Extraction Summary".bold().green());
    println!("{}", "─".repeat(50));
    
    println!("📁 Project: {}", ast_data.project.name.cyan());
    println!("📄 Files processed: {}", ast_data.metrics.total_files.to_string().yellow());
    println!("📏 Total lines: {}", ast_data.metrics.total_lines.to_string().yellow());
    
    println!("\n{}", "🔍 Code Elements:".bold());
    println!("  🔧 Functions: {}", ast_data.metrics.total_functions.to_string().blue());
    println!("  🏗️  Structs: {}", ast_data.metrics.total_structs.to_string().blue());
    println!("  🎯 Enums: {}", ast_data.metrics.total_enums.to_string().blue());
    println!("  🎭 Traits: {}", ast_data.metrics.total_traits.to_string().blue());
    
    if ast_data.metrics.complexity_average > 0.0 {
        let complexity_color = if ast_data.metrics.complexity_average > 10.0 {
            "red"
        } else if ast_data.metrics.complexity_average > 5.0 {
            "yellow"
        } else {
            "green"
        };
        
        println!("📈 Avg. Complexity: {}", 
            format!("{:.2}", ast_data.metrics.complexity_average)
                .color(complexity_color.parse().unwrap())
        );
    }
    
    println!();
}

// ============================================================================
// Project Setup Script (setup.sh)
// ============================================================================

/*
#!/bin/bash

# Rust AST Extractor Setup Script

echo "🚀 Setting up Rust AST Extractor project..."

# Create project structure
mkdir -p rust-ast-extractor
cd rust-ast-extractor

# Create workspace structure
mkdir -p crates/{rustex-core,rustex-cli,rustex-plugins,rustex-formats}
mkdir -p {examples,tests/{integration,fixtures,benchmarks},docs/{api,guides,examples},scripts}
mkdir -p .github/{workflows,ISSUE_TEMPLATE}

# Create workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/rustex-core",
    "crates/rustex-cli", 
    "crates/rustex-plugins",
    "crates/rustex-formats"
]

[workspace.dependencies]
syn = { version = "2.0", features = ["full", "extra-traits", "visit"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
clap = { version = "4.0", features = ["derive"] }
quote = "1.0"
walkdir = "2.0"
chrono = { version = "0.4", features = ["serde"] }
EOF

# Create core crate
cat > crates/rustex-core/Cargo.toml << 'EOF'
[package]
name = "rustex-core"
version = "0.1.0"
edition = "2021"
description = "Core AST extraction library for Rust projects"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-username/rust-ast-extractor"

[dependencies]
syn = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
quote = { workspace = true }
walkdir = { workspace = true }
chrono = { workspace = true }
EOF

# Create CLI crate
cat > crates/rustex-cli/Cargo.toml << 'EOF'
[package]
name = "rustex-cli"
version = "0.1.0"
edition = "2021"
description = "Command-line interface for Rust AST extraction"
license = "MIT OR Apache-2.0"

[[bin]]
name = "rustex"
path = "src/main.rs"

[dependencies]
rustex-core = { path = "../rustex-core" }
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = { workspace = true }
serde_json = { workspace = true }
colored = "2.0"
indicatif = "0.17"
EOF

# Create basic lib.rs files
echo 'pub mod extractor;
pub mod config;
pub mod ast_data;
pub mod visitors;

pub use extractor::AstExtractor;
pub use config::ExtractorConfig;
pub use ast_data::*;' > crates/rustex-core/src/lib.rs

touch crates/rustex-core/src/{extractor.rs,config.rs,ast_data.rs,visitors.rs}
touch crates/rustex-cli/src/main.rs

# Create README
cat > README.md << 'EOF'
# Rust AST Extractor

A comprehensive, high-performance tool for extracting Abstract Syntax Trees from Rust projects, optimized for LLM/RAG applications and code analysis.

## Features

- 🚀 **Fast & Efficient**: Parse entire Rust projects quickly
- 📊 **Comprehensive**: Extract functions, structs, enums, traits, and more
- 🔌 **Extensible**: Plugin system for custom extractors
- 📝 **Multiple Formats**: JSON, Markdown, GraphQL, and more
- 🤖 **LLM Ready**: Optimized output for language models and RAG systems
- 📈 **Analytics**: Code metrics and complexity analysis

## Quick Start

```bash
# Install
cargo install rustex-cli

# Extract AST from current project
rustex extract

# Extract with documentation
rustex extract --include-docs --format json --output ast.json

# Generate markdown documentation
rustex extract --format markdown --output docs.md
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
EOF

# Create CI workflow
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - uses: actions-rs/cargo@v1
      with:
        command: test
        args: --all-features

  clippy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: clippy
        override: true
    - uses: actions-rs/clippy-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
        args: --all-features

  fmt:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt
        override: true
    - uses: actions-rs/cargo@v1
      with:
        command: fmt
        args: --all -- --check
EOF

echo "✅ Project structure created!"
echo "📁 Next steps:"
echo "   1. cd rust-ast-extractor"
echo "   2. Add the implementation code from the artifacts"
echo "   3. cargo build"
echo "   4. cargo test"
echo "   5. ./target/debug/rustex extract"

echo ""
echo "🎯 Happy coding!"
*/
