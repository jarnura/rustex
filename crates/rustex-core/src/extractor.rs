//! Core AST extraction functionality.

use crate::{ast_data::*, config::ExtractorConfig, errors::*, visitors::CodeElementVisitor};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

/// Main AST extractor for Rust projects.
pub struct AstExtractor {
    /// Configuration for extraction
    config: ExtractorConfig,
    /// Root path of the project to extract
    root_path: PathBuf,
}

impl AstExtractor {
    /// Create a new AST extractor with configuration and root path.
    ///
    /// # Arguments
    /// * `config` - Configuration for extraction behavior
    /// * `root_path` - Root directory of the Rust project
    pub fn new(config: ExtractorConfig, root_path: PathBuf) -> Self {
        Self { config, root_path }
    }

    /// Extract AST from the configured Rust project.
    ///
    /// # Returns
    /// A `Result` containing the extracted project AST or an error.
    ///
    /// # Error Handling
    /// This method uses partial failure recovery - it will attempt to process
    /// all files and collect errors, only failing completely if no files
    /// can be processed successfully.
    pub fn extract_project(&self) -> Result<ProjectAst> {
        tracing::info!(
            "Starting AST extraction for project at {:?}",
            self.root_path
        );

        let project_info = self.extract_project_info()?;
        let rust_files = self.discover_rust_files()?;

        let mut files = Vec::new();
        let mut project_metrics = ProjectMetrics::default();
        let mut file_errors = Vec::new();
        let total_files = rust_files.len();

        for file_path in rust_files {
            match self.extract_file(&file_path) {
                Ok(file_ast) => {
                    self.update_project_metrics(&mut project_metrics, &file_ast.file_metrics);
                    files.push(file_ast);
                }
                Err(e) => {
                    tracing::warn!("Failed to extract AST from {:?}: {}", file_path, e);
                    file_errors.push(e);
                }
            }
        }

        // Check if we have too many failures
        let failed_count = file_errors.len();
        if failed_count > 0 {
            let success_rate = 1.0 - (failed_count as f64 / total_files as f64);
            if success_rate < 0.5 {
                // More than 50% failure rate - return error
                return Err(RustExError::PartialFailure {
                    failed_count,
                    total_count: total_files,
                    errors: file_errors,
                });
            } else {
                // Log summary of partial failures but continue
                tracing::warn!(
                    "Partial failure: {}/{} files failed to process ({}% success rate)",
                    failed_count,
                    total_files,
                    (success_rate * 100.0) as u32
                );
            }
        }

        let dependencies = self.extract_dependencies()?;

        Ok(ProjectAst {
            project: project_info,
            files,
            dependencies,
            metrics: project_metrics,
            extracted_at: chrono::Utc::now(),
        })
    }

    /// Extract project metadata from Cargo.toml.
    fn extract_project_info(&self) -> Result<ProjectInfo> {
        let cargo_toml_path = self.root_path.join("Cargo.toml");

        if cargo_toml_path.exists() {
            let cargo_content =
                fs::read_to_string(&cargo_toml_path).map_err(|e| RustExError::Io(e))?;

            // Simple parsing - in a real implementation, we'd use a TOML parser
            let name = extract_toml_field(&cargo_content, "name")
                .unwrap_or_else(|| "unknown-project".to_string());
            let version = extract_toml_field(&cargo_content, "version")
                .unwrap_or_else(|| "0.1.0".to_string());
            let edition =
                extract_toml_field(&cargo_content, "edition").unwrap_or_else(|| "2021".to_string());

            Ok(ProjectInfo {
                name,
                version,
                rust_edition: edition,
                root_path: self.root_path.clone(),
            })
        } else {
            // Fallback for non-Cargo projects
            Ok(ProjectInfo {
                name: self
                    .root_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown-project")
                    .to_string(),
                version: "0.1.0".to_string(),
                rust_edition: "2021".to_string(),
                root_path: self.root_path.clone(),
            })
        }
    }

    /// Discover all Rust files in the project.
    fn discover_rust_files(&self) -> Result<Vec<PathBuf>> {
        let mut rust_files = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if self.should_include_file(path) {
                    rust_files.push(path.to_path_buf());
                }
            }
        }

        tracing::debug!("Found {} Rust files", rust_files.len());
        Ok(rust_files)
    }

    /// Check if a file should be included based on filters.
    fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check exclude patterns
        for exclude in &self.config.filters.exclude {
            if glob_match(exclude, &path_str) {
                return false;
            }
        }

        // Check include patterns
        for include in &self.config.filters.include {
            if glob_match(include, &path_str) {
                return true;
            }
        }

        false
    }

    /// Extract AST from a single file.
    fn extract_file(&self, file_path: &Path) -> FileResult<FileAst> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                FileProcessingError::AccessDenied {
                    file: file_path.to_path_buf(),
                }
            } else {
                FileProcessingError::IoError {
                    file: file_path.to_path_buf(),
                    error: e.to_string(),
                }
            }
        })?;

        if content.len() > self.config.max_file_size {
            return Err(FileProcessingError::TooLarge {
                file: file_path.to_path_buf(),
                size: content.len(),
            });
        }

        let syntax_tree =
            syn::parse_file(&content).map_err(|e| FileProcessingError::ParseError {
                file: file_path.to_path_buf(),
                error: e.to_string(),
            })?;

        let relative_path = file_path
            .strip_prefix(&self.root_path)
            .unwrap_or(file_path)
            .to_path_buf();

        let mut visitor = CodeElementVisitor::new(file_path.to_path_buf(), &self.config);
        visitor.visit_file(&syntax_tree);

        let elements = visitor.into_elements();
        let imports = extract_imports(&syntax_tree);
        let file_metrics = calculate_file_metrics(&content, &elements);

        Ok(FileAst {
            path: file_path.to_path_buf(),
            relative_path,
            elements,
            imports,
            file_metrics,
        })
    }

    /// Extract dependency information.
    ///
    /// # Error Handling
    /// This method currently returns a placeholder and will not fail.
    /// Future implementations will include proper error handling for
    /// Cargo.toml and Cargo.lock parsing failures.
    fn extract_dependencies(&self) -> Result<DependencyInfo> {
        // TODO: Parse Cargo.lock and Cargo.toml for dependency information
        // For now, return empty dependencies with proper error handling structure
        Ok(DependencyInfo {
            direct: vec![],
            transitive: vec![],
            dev_dependencies: vec![],
        })
    }

    /// Update project metrics with file metrics.
    fn update_project_metrics(&self, metrics: &mut ProjectMetrics, file_metrics: &FileMetrics) {
        metrics.total_lines += file_metrics.lines_of_code;
        metrics.total_files += 1;
        metrics.total_functions += file_metrics.function_count;
        metrics.total_structs += file_metrics.struct_count;
        metrics.total_enums += file_metrics.enum_count;
        metrics.total_traits += file_metrics.trait_count;

        // Update complexity metrics
        if file_metrics.complexity_total > metrics.complexity_max {
            metrics.complexity_max = file_metrics.complexity_total;
        }

        // Calculate average complexity
        let total_elements = metrics.total_functions
            + metrics.total_structs
            + metrics.total_enums
            + metrics.total_traits;
        if total_elements > 0 {
            metrics.complexity_average = (metrics.total_functions as f64 * 1.0
                + metrics.total_structs as f64 * 1.0
                + metrics.total_enums as f64 * 2.0
                + metrics.total_traits as f64 * 2.0)
                / total_elements as f64;
        }
    }

    /// Get the configuration used by this extractor.
    pub fn config(&self) -> &ExtractorConfig {
        &self.config
    }

    /// Get the root path of the project.
    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}

impl Default for ProjectMetrics {
    fn default() -> Self {
        Self {
            total_lines: 0,
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            complexity_average: 0.0,
            complexity_max: 0,
        }
    }
}

/// Simple glob pattern matching (simplified implementation).
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern.contains("**") {
        let prefix = pattern.split("**").next().unwrap_or("");
        text.contains(prefix)
    } else if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        text.starts_with(prefix)
    } else {
        text.contains(pattern)
    }
}

/// Extract imports from a Rust file.
fn extract_imports(file: &syn::File) -> Vec<ImportInfo> {
    let mut imports = Vec::new();

    for item in &file.items {
        if let syn::Item::Use(use_item) = item {
            if let Some(import_info) = parse_use_tree(&use_item.tree) {
                imports.push(import_info);
            }
        }
    }

    imports
}

/// Parse a use tree into import information.
fn parse_use_tree(tree: &syn::UseTree) -> Option<ImportInfo> {
    match tree {
        syn::UseTree::Path(path) => Some(ImportInfo {
            module_path: path.ident.to_string(),
            imported_items: vec![],
            is_glob: false,
            alias: None,
        }),
        syn::UseTree::Name(name) => Some(ImportInfo {
            module_path: "".to_string(),
            imported_items: vec![name.ident.to_string()],
            is_glob: false,
            alias: None,
        }),
        syn::UseTree::Glob(_) => Some(ImportInfo {
            module_path: "".to_string(),
            imported_items: vec![],
            is_glob: true,
            alias: None,
        }),
        syn::UseTree::Group(group) => {
            // Handle grouped imports like `use std::{vec, collections};`
            let items: Vec<String> = group
                .items
                .iter()
                .filter_map(|item| {
                    if let syn::UseTree::Name(name) = item {
                        Some(name.ident.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            Some(ImportInfo {
                module_path: "".to_string(),
                imported_items: items,
                is_glob: false,
                alias: None,
            })
        }
        syn::UseTree::Rename(rename) => Some(ImportInfo {
            module_path: "".to_string(),
            imported_items: vec![rename.ident.to_string()],
            is_glob: false,
            alias: Some(rename.rename.to_string()),
        }),
    }
}

/// Calculate metrics for a file.
fn calculate_file_metrics(content: &str, elements: &[CodeElement]) -> FileMetrics {
    let lines: Vec<&str> = content.lines().collect();
    let lines_of_code = lines
        .iter()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count();

    let lines_of_comments = lines
        .iter()
        .filter(|line| line.trim().starts_with("//"))
        .count();

    FileMetrics {
        lines_of_code,
        lines_of_comments,
        complexity_total: elements.iter().map(|e| e.complexity.unwrap_or(0)).sum(),
        function_count: elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Function))
            .count(),
        struct_count: elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Struct))
            .count(),
        enum_count: elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Enum))
            .count(),
        trait_count: elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Trait))
            .count(),
    }
}

/// Extract a field value from TOML content (simplified parser).
fn extract_toml_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{} =", field)) {
            if let Some(value_part) = line.split('=').nth(1) {
                let value = value_part.trim().trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}
