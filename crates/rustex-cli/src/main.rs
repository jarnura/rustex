//! # RustEx CLI
//!
//! Command-line interface for Rust AST extraction.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustex_core::{AstExtractor, ConfigUseCase, ExtractorConfig, OutputFormat};
use std::path::{Path, PathBuf};
use tracing::{error, info};

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

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
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

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Initialize a new configuration file
    Init {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,

        /// Use case template to generate
        #[arg(long, value_enum)]
        template: Option<CliConfigUseCase>,

        /// Configuration file path
        #[arg(short, long, default_value = "rustex.toml")]
        output: PathBuf,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Show current configuration
    Show {
        /// Configuration file path
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Generate example configuration for different use cases
    Template {
        /// Use case to generate template for
        #[arg(value_enum)]
        use_case: CliConfigUseCase,

        /// Output file path
        #[arg(short, long, default_value = "rustex-template.toml")]
        output: PathBuf,
    },
}

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
enum CliConfigUseCase {
    Documentation,
    CodeAnalysis,
    LlmTraining,
    Testing,
}

impl From<CliConfigUseCase> for ConfigUseCase {
    fn from(cli_use_case: CliConfigUseCase) -> Self {
        match cli_use_case {
            CliConfigUseCase::Documentation => ConfigUseCase::Documentation,
            CliConfigUseCase::CodeAnalysis => ConfigUseCase::CodeAnalysis,
            CliConfigUseCase::LlmTraining => ConfigUseCase::LlmTraining,
            CliConfigUseCase::Testing => ConfigUseCase::Testing,
        }
    }
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
            // Load base configuration
            let mut config = load_config(&cli.config, &cli.path)?;

            // Override with CLI arguments
            override_config_with_cli_args(
                &mut config,
                CliOverrides {
                    format: format.into(),
                    include_docs,
                    include_private,
                    parse_deps,
                    max_file_size,
                    include_patterns: include,
                    exclude_patterns: exclude,
                    plugins,
                },
            );

            extract_command(cli.path, config, output, pretty).await?;
        }
        Commands::Deps { visualize, output } => {
            deps_command(cli.path, visualize, output).await?;
        }
        Commands::Metrics {
            complexity,
            loc,
            output,
        } => {
            metrics_command(cli.path, complexity, loc, output).await?;
        }
        Commands::Config { action } => {
            config_command(action, cli.config.as_ref()).await?;
        }
    }

    Ok(())
}

async fn extract_command(
    project_path: PathBuf,
    config: ExtractorConfig,
    output: Option<PathBuf>,
    pretty: bool,
) -> Result<()> {
    info!("Starting AST extraction for project at {:?}", project_path);

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(e);
    }

    let extractor = AstExtractor::new(config.clone(), project_path);

    // Show progress bar
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Extracting AST...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match extractor.extract_project() {
        Ok(ast_data) => {
            pb.finish_with_message("✓ AST extraction completed");

            let output_content = match config.output_format {
                OutputFormat::Json => {
                    if pretty {
                        serde_json::to_string_pretty(&ast_data)?
                    } else {
                        serde_json::to_string(&ast_data)?
                    }
                }
                OutputFormat::Markdown => generate_markdown_output(&ast_data)?,
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
                    // Handle broken pipe gracefully (e.g., when piping to `head`)
                    use std::io::Write;
                    if let Err(e) = std::io::stdout().write_all(output_content.as_bytes()) {
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            // Broken pipe is normal when using tools like `head` - exit gracefully
                            std::process::exit(0);
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }

            // Print summary
            print_extraction_summary(&ast_data);
        }
        Err(e) => {
            pb.finish_with_message("✗ AST extraction failed");

            // Provide more helpful error messages based on error type
            match &e {
                rustex_core::RustExError::PartialFailure {
                    failed_count,
                    total_count,
                    ..
                } => {
                    error!(
                        "Partial extraction failure: {}/{} files failed",
                        failed_count, total_count
                    );
                    error!("Consider checking file permissions, syntax errors, or adjusting file size limits");
                }
                rustex_core::RustExError::InvalidProjectRoot { path } => {
                    error!("Invalid project root: {:?}", path);
                    error!("Make sure the path exists and contains a valid Rust project");
                }
                _ => {
                    error!("Extraction failed: {}", e);
                }
            }

            return Err(e.into());
        }
    }

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

async fn config_command(action: ConfigAction, global_config_path: Option<&PathBuf>) -> Result<()> {
    match action {
        ConfigAction::Init {
            force,
            template,
            output,
        } => {
            if output.exists() && !force {
                error!(
                    "Configuration file already exists at {}. Use --force to overwrite.",
                    output.display()
                );
                return Ok(());
            }

            let config = match template {
                Some(use_case) => ExtractorConfig::for_use_case(use_case.into()),
                None => ExtractorConfig::default(),
            };

            config.to_toml_file(&output)?;
            println!("✓ Created configuration file at {}", output.display());

            if let Some(template) = template {
                println!("   Template: {:?}", template);
            }
        }

        ConfigAction::Validate { file } => {
            let config_path = resolve_config_path(file.as_ref(), global_config_path)?;
            let config = ExtractorConfig::from_toml_file(&config_path)?;

            match config.validate() {
                Ok(()) => {
                    println!("✓ Configuration is valid: {}", config_path.display());
                }
                Err(e) => {
                    error!("✗ Configuration validation failed: {}", e);
                    return Err(e);
                }
            }
        }

        ConfigAction::Show { file } => {
            let config_path = resolve_config_path(file.as_ref(), global_config_path)?;
            let config = ExtractorConfig::from_toml_file(&config_path)?;

            println!("Configuration from: {}", config_path.display());
            println!("{}", config.to_toml_string()?);
        }

        ConfigAction::Template { use_case, output } => {
            let config = ExtractorConfig::for_use_case(use_case.into());
            config.to_toml_file(&output)?;

            println!(
                "✓ Created {} template at {}",
                format!("{:?}", use_case).to_lowercase(),
                output.display()
            );
        }
    }

    Ok(())
}

/// Load configuration from file or use defaults.
fn load_config(config_path: &Option<PathBuf>, project_path: &Path) -> Result<ExtractorConfig> {
    if let Some(path) = config_path {
        // Use explicitly provided config file
        ExtractorConfig::from_toml_file(path)
            .with_context(|| format!("Failed to load configuration from {}", path.display()))
    } else {
        // Try to find config in standard locations relative to project
        let project_config = project_path.join("rustex.toml");
        if project_config.exists() {
            info!("Using project config: {}", project_config.display());
            return ExtractorConfig::from_toml_file(&project_config);
        }

        let alt_project_config = project_path.join(".rustex.toml");
        if alt_project_config.exists() {
            info!("Using project config: {}", alt_project_config.display());
            return ExtractorConfig::from_toml_file(&alt_project_config);
        }

        // Fall back to standard locations
        let config = ExtractorConfig::load_from_standard_locations();
        info!("Using default configuration");
        Ok(config)
    }
}

/// CLI arguments for overriding configuration.
struct CliOverrides {
    format: OutputFormat,
    include_docs: bool,
    include_private: bool,
    parse_deps: bool,
    max_file_size: usize,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    plugins: Vec<String>,
}

/// Override configuration with CLI arguments.
fn override_config_with_cli_args(config: &mut ExtractorConfig, overrides: CliOverrides) {
    // Only override if CLI args were explicitly provided
    // For most args, clap provides default values, so we check against defaults

    // Always override format if provided
    config.output_format = overrides.format;

    // These need special handling as clap provides false as default
    // In a real implementation, you'd use Option<bool> and check for Some(value)
    config.include_docs = overrides.include_docs;
    config.include_private = overrides.include_private;
    config.parse_dependencies = overrides.parse_deps;

    // Override file size if not default
    if overrides.max_file_size != 10485760 {
        // 10MB default
        config.max_file_size = overrides.max_file_size;
    }

    // Override patterns if provided
    if !overrides.include_patterns.is_empty() {
        config.filters.include = overrides.include_patterns;
    }

    if !overrides.exclude_patterns.is_empty() {
        config.filters.exclude = overrides.exclude_patterns;
    }

    // Override plugins if provided
    if !overrides.plugins.is_empty() {
        config.plugins = overrides.plugins;
    }
}

/// Resolve configuration file path.
fn resolve_config_path(
    file_path: Option<&PathBuf>,
    global_config_path: Option<&PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = file_path {
        Ok(path.clone())
    } else if let Some(path) = global_config_path {
        Ok(path.clone())
    } else {
        // Try to find config in current directory
        let local_config = PathBuf::from("rustex.toml");
        if local_config.exists() {
            Ok(local_config)
        } else {
            anyhow::bail!("No configuration file found. Use --file to specify one or run 'rustex config init' to create one.");
        }
    }
}

/// Generate markdown output from AST data.
fn generate_markdown_output(ast_data: &rustex_core::ProjectAst) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!("# {} AST Analysis\n\n", ast_data.project.name));
    output.push_str(&format!("**Version:** {}\n", ast_data.project.version));
    output.push_str(&format!(
        "**Rust Edition:** {}\n",
        ast_data.project.rust_edition
    ));
    output.push_str(&format!(
        "**Extracted:** {}\n\n",
        ast_data.extracted_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Project metrics
    output.push_str("## Project Metrics\n\n");
    output.push_str(&format!(
        "- **Total Files:** {}\n",
        ast_data.metrics.total_files
    ));
    output.push_str(&format!(
        "- **Total Lines:** {}\n",
        ast_data.metrics.total_lines
    ));
    output.push_str(&format!(
        "- **Functions:** {}\n",
        ast_data.metrics.total_functions
    ));
    output.push_str(&format!(
        "- **Structs:** {}\n",
        ast_data.metrics.total_structs
    ));
    output.push_str(&format!("- **Enums:** {}\n", ast_data.metrics.total_enums));
    output.push_str(&format!(
        "- **Traits:** {}\n",
        ast_data.metrics.total_traits
    ));
    output.push_str(&format!(
        "- **Average Complexity:** {:.2}\n\n",
        ast_data.metrics.complexity_average
    ));

    // File breakdown
    if !ast_data.files.is_empty() {
        output.push_str("## Files\n\n");
        for file in &ast_data.files {
            output.push_str(&format!("### {}\n\n", file.relative_path.display()));

            if !file.elements.is_empty() {
                for element in &file.elements {
                    output.push_str(&format!(
                        "#### {:?} `{}`\n\n",
                        element.element_type,
                        element.name
                    ));

                    if !element.doc_comments.is_empty() {
                        output.push_str("**Documentation:**\n");
                        for doc in &element.doc_comments {
                            output.push_str(&format!("> {}\n", doc));
                        }
                        output.push('\n');
                    }

                    if let Some(ref signature) = element.signature {
                        output.push_str(&format!("```rust\n{}\n```\n\n", signature));
                    }
                }
            } else {
                output.push_str("*No extractable elements found*\n\n");
            }
        }
    }

    Ok(output)
}

/// Print extraction summary to terminal.
fn print_extraction_summary(ast_data: &rustex_core::ProjectAst) {
    use colored::*;

    println!("\n{}", "📊 Extraction Summary".bold().green());
    println!("{}", "─".repeat(50));

    println!("📁 Project: {}", ast_data.project.name.cyan());
    println!(
        "📄 Files processed: {}",
        ast_data.metrics.total_files.to_string().yellow()
    );
    println!(
        "📏 Total lines: {}",
        ast_data.metrics.total_lines.to_string().yellow()
    );

    println!("\n{}", "🔍 Code Elements:".bold());
    println!(
        "  🔧 Functions: {}",
        ast_data.metrics.total_functions.to_string().blue()
    );
    println!(
        "  🏗️  Structs: {}",
        ast_data.metrics.total_structs.to_string().blue()
    );
    println!(
        "  🎯 Enums: {}",
        ast_data.metrics.total_enums.to_string().blue()
    );
    println!(
        "  🎭 Traits: {}",
        ast_data.metrics.total_traits.to_string().blue()
    );

    if ast_data.metrics.complexity_average > 0.0 {
        let complexity_color = if ast_data.metrics.complexity_average > 10.0 {
            "red"
        } else if ast_data.metrics.complexity_average > 5.0 {
            "yellow"
        } else {
            "green"
        };

        let formatted_complexity = format!("{:.2}", ast_data.metrics.complexity_average);
        println!(
            "📈 Avg. Complexity: {}",
            match complexity_color {
                "red" => formatted_complexity.red(),
                "yellow" => formatted_complexity.yellow(),
                _ => formatted_complexity.green(),
            }
        );
    }

    println!();
}
