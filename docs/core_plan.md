// Cargo.toml for the workspace
/*
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
*/

// ============================================================================
// crates/rustex-core/src/lib.rs
// ============================================================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use syn::visit::Visit;

pub mod extractor;
pub mod config;
pub mod ast_data;
pub mod visitors;

pub use extractor::AstExtractor;
pub use config::ExtractorConfig;
pub use ast_data::*;

// ============================================================================
// crates/rustex-core/src/config.rs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    pub include_docs: bool,
    pub include_private: bool,
    pub parse_dependencies: bool,
    pub max_file_size: usize,
    pub output_format: OutputFormat,
    pub filters: FilterConfig,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    MessagePack,
    Markdown,
    GraphQL,
    Rag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
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

// ============================================================================
// crates/rustex-core/src/ast_data.rs
// ============================================================================

use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAst {
    pub project: ProjectInfo,
    pub files: Vec<FileAst>,
    pub dependencies: DependencyInfo,
    pub metrics: ProjectMetrics,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub rust_edition: String,
    pub root_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAst {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub elements: Vec<CodeElement>,
    pub imports: Vec<ImportInfo>,
    pub file_metrics: FileMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub element_type: ElementType,
    pub name: String,
    pub signature: Option<String>,
    pub visibility: Visibility,
    pub doc_comments: Vec<String>,
    pub inline_comments: Vec<String>,
    pub location: CodeLocation,
    pub attributes: Vec<String>,
    pub complexity: Option<u32>,
    pub dependencies: Vec<String>,
    pub generic_params: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Constant,
    Static,
    TypeAlias,
    Macro,
    Union,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Restricted(String),
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub line_start: usize,
    pub line_end: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module_path: String,
    pub imported_items: Vec<String>,
    pub is_glob: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub direct: Vec<String>,
    pub transitive: Vec<String>,
    pub dev_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub total_lines: usize,
    pub total_files: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_traits: usize,
    pub complexity_average: f64,
    pub complexity_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub lines_of_code: usize,
    pub lines_of_comments: usize,
    pub complexity_total: u32,
    pub function_count: usize,
    pub struct_count: usize,
}

// ============================================================================
// crates/rustex-core/src/extractor.rs
// ============================================================================

use crate::{ast_data::*, config::ExtractorConfig, visitors::*};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct AstExtractor {
    config: ExtractorConfig,
    root_path: PathBuf,
}

impl AstExtractor {
    pub fn new(config: ExtractorConfig, root_path: PathBuf) -> Self {
        Self { config, root_path }
    }

    pub fn extract_project(&self) -> Result<ProjectAst> {
        tracing::info!("Starting AST extraction for project at {:?}", self.root_path);
        
        let project_info = self.extract_project_info()?;
        let rust_files = self.discover_rust_files()?;
        
        let mut files = Vec::new();
        let mut project_metrics = ProjectMetrics::default();
        
        for file_path in rust_files {
            match self.extract_file(&file_path) {
                Ok(file_ast) => {
                    self.update_project_metrics(&mut project_metrics, &file_ast.file_metrics);
                    files.push(file_ast);
                }
                Err(e) => {
                    tracing::warn!("Failed to extract AST from {:?}: {}", file_path, e);
                }
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

    fn extract_project_info(&self) -> Result<ProjectInfo> {
        let cargo_toml_path = self.root_path.join("Cargo.toml");
        let cargo_content = fs::read_to_string(&cargo_toml_path)
            .context("Failed to read Cargo.toml")?;
        
        // Parse Cargo.toml (simplified)
        Ok(ProjectInfo {
            name: "extracted-project".to_string(), // TODO: Parse from Cargo.toml
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: self.root_path.clone(),
        })
    }

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
        
        Ok(rust_files)
    }

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

    fn extract_file(&self, file_path: &Path) -> Result<FileAst> {
        let content = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {:?}", file_path))?;
        
        if content.len() > self.config.max_file_size {
            anyhow::bail!("File too large: {} bytes", content.len());
        }

        let syntax_tree = syn::parse_file(&content)
            .context("Failed to parse Rust syntax")?;

        let relative_path = file_path.strip_prefix(&self.root_path)
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

    fn extract_dependencies(&self) -> Result<DependencyInfo> {
        // TODO: Parse Cargo.lock and Cargo.toml for dependency information
        Ok(DependencyInfo {
            direct: vec![],
            transitive: vec![],
            dev_dependencies: vec![],
        })
    }

    fn update_project_metrics(&self, metrics: &mut ProjectMetrics, file_metrics: &FileMetrics) {
        metrics.total_lines += file_metrics.lines_of_code;
        metrics.total_files += 1;
        metrics.total_functions += file_metrics.function_count;
        metrics.total_structs += file_metrics.struct_count;
        // TODO: Update other metrics
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

// Simplified glob matching (replace with proper glob crate in production)
fn glob_match(pattern: &str, text: &str) -> bool {
    // Very simplified - use the `glob` crate for production
    if pattern.contains("**") {
        let prefix = pattern.split("**").next().unwrap_or("");
        text.starts_with(prefix)
    } else {
        text.contains(pattern.trim_end_matches("*"))
    }
}

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

fn parse_use_tree(tree: &syn::UseTree) -> Option<ImportInfo> {
    match tree {
        syn::UseTree::Path(path) => {
            // Handle path::to::module
            Some(ImportInfo {
                module_path: path.ident.to_string(),
                imported_items: vec![],
                is_glob: false,
                alias: None,
            })
        }
        syn::UseTree::Name(name) => {
            Some(ImportInfo {
                module_path: name.ident.to_string(),
                imported_items: vec![name.ident.to_string()],
                is_glob: false,
                alias: None,
            })
        }
        syn::UseTree::Glob(_) => {
            Some(ImportInfo {
                module_path: "".to_string(),
                imported_items: vec![],
                is_glob: true,
                alias: None,
            })
        }
        _ => None,
    }
}

fn calculate_file_metrics(content: &str, elements: &[CodeElement]) -> FileMetrics {
    let lines: Vec<&str> = content.lines().collect();
    let lines_of_code = lines.len();
    let lines_of_comments = lines.iter()
        .filter(|line| line.trim().starts_with("//"))
        .count();
    
    FileMetrics {
        lines_of_code,
        lines_of_comments,
        complexity_total: elements.iter()
            .map(|e| e.complexity.unwrap_or(0))
            .sum(),
        function_count: elements.iter()
            .filter(|e| matches!(e.element_type, ElementType::Function))
            .count(),
        struct_count: elements.iter()
            .filter(|e| matches!(e.element_type, ElementType::Struct))
            .count(),
    }
}

// ============================================================================
// crates/rustex-core/src/visitors.rs
// ============================================================================

use crate::ast_data::*;
use crate::config::ExtractorConfig;
use std::path::PathBuf;
use syn::visit::Visit;

pub struct CodeElementVisitor {
    elements: Vec<CodeElement>,
    file_path: PathBuf,
    config: ExtractorConfig,
}

impl CodeElementVisitor {
    pub fn new(file_path: PathBuf, config: &ExtractorConfig) -> Self {
        Self {
            elements: Vec::new(),
            file_path,
            config: config.clone(),
        }
    }

    pub fn into_elements(self) -> Vec<CodeElement> {
        self.elements
    }

    fn extract_doc_comments(&self, attrs: &[syn::Attribute]) -> Vec<String> {
        attrs.iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let syn::Meta::NameValue(meta) = &attr.meta {
                        if let syn::Expr::Lit(syn::ExprLit { 
                            lit: syn::Lit::Str(lit_str), .. 
                        }) = &meta.value {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                }
                None
            })
            .collect()
    }

    fn get_visibility(&self, vis: &syn::Visibility) -> Visibility {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Restricted(restricted) => {
                Visibility::Restricted(format!("{}", quote::quote!(#restricted)))
            }
            syn::Visibility::Inherited => Visibility::Private,
        }
    }

    fn create_location(&self, span: proc_macro2::Span) -> CodeLocation {
        let start = span.start();
        let end = span.end();
        
        CodeLocation {
            line_start: start.line,
            line_end: end.line,
            char_start: start.column,
            char_end: end.column,
            file_path: self.file_path.clone(),
        }
    }
}

impl<'ast> Visit<'ast> for CodeElementVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if !self.config.include_private && matches!(self.get_visibility(&node.vis), Visibility::Private) {
            return;
        }

        let signature = format!("{}", quote::quote!(#node.sig));
        let docs = if self.config.include_docs {
            self.extract_doc_comments(&node.attrs)
        } else {
            vec![]
        };

        let element = CodeElement {
            element_type: ElementType::Function,
            name: node.sig.ident.to_string(),
            signature: Some(signature),
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![], // TODO: Extract inline comments
            location: self.create_location(node.sig.ident.span()),
            attributes: node.attrs.iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(calculate_complexity(&node.block)),
            dependencies: vec![], // TODO: Extract function dependencies
            generic_params: node.sig.generics.params.iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if !self.config.include_private && matches!(self.get_visibility(&node.vis), Visibility::Private) {
            return;
        }

        let docs = if self.config.include_docs {
            self.extract_doc_comments(&node.attrs)
        } else {
            vec![]
        };

        let element = CodeElement {
            element_type: ElementType::Struct,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node.attrs.iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(1), // Base complexity for structs
            dependencies: vec![],
            generic_params: node.generics.params.iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        if !self.config.include_private && matches!(self.get_visibility(&node.vis), Visibility::Private) {
            return;
        }

        let docs = if self.config.include_docs {
            self.extract_doc_comments(&node.attrs)
        } else {
            vec![]
        };

        let element = CodeElement {
            element_type: ElementType::Enum,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node.attrs.iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(node.variants.len() as u32),
            dependencies: vec![],
            generic_params: node.generics.params.iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if !self.config.include_private && matches!(self.get_visibility(&node.vis), Visibility::Private) {
            return;
        }

        let docs = if self.config.include_docs {
            self.extract_doc_comments(&node.attrs)
        } else {
            vec![]
        };

        let element = CodeElement {
            element_type: ElementType::Trait,
            name: node.ident.to_string(),
            signature: None,
            visibility: self.get_visibility(&node.vis),
            doc_comments: docs,
            inline_comments: vec![],
            location: self.create_location(node.ident.span()),
            attributes: node.attrs.iter()
                .map(|attr| format!("{}", quote::quote!(#attr)))
                .collect(),
            complexity: Some(node.items.len() as u32),
            dependencies: vec![],
            generic_params: node.generics.params.iter()
                .map(|param| format!("{}", quote::quote!(#param)))
                .collect(),
            metadata: std::collections::HashMap::new(),
        };

        self.elements.push(element);
        syn::visit::visit_item_trait(self, node);
    }
}

// Simplified complexity calculation
fn calculate_complexity(block: &syn::Block) -> u32 {
    let mut complexity = 1; // Base complexity
    
    for stmt in &block.stmts {
        complexity += match stmt {
            syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => {
                calculate_expr_complexity(expr)
            }
            _ => 0,
        };
    }
    
    complexity
}

fn calculate_expr_complexity(expr: &syn::Expr) -> u32 {
    match expr {
        syn::Expr::If(_) => 1,
        syn::Expr::Match(match_expr) => match_expr.arms.len() as u32,
        syn::Expr::While(_) | syn::Expr::ForLoop(_) | syn::Expr::Loop(_) => 1,
        _ => 0,
    }
}
