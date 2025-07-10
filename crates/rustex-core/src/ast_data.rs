//! Data structures for representing extracted AST information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete AST representation of a Rust project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAst {
    /// Project metadata
    pub project: ProjectInfo,
    /// List of analyzed files
    pub files: Vec<FileAst>,
    /// Dependency information
    pub dependencies: DependencyInfo,
    /// Project-wide metrics
    pub metrics: ProjectMetrics,
    /// Timestamp of extraction
    pub extracted_at: DateTime<Utc>,
}

/// Project metadata information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Rust edition used
    pub rust_edition: String,
    /// Root path of the project
    pub root_path: PathBuf,
}

/// AST representation of a single Rust file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAst {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Path relative to project root
    pub relative_path: PathBuf,
    /// Extracted code elements
    pub elements: Vec<CodeElement>,
    /// Import statements
    pub imports: Vec<ImportInfo>,
    /// File-level metrics
    pub file_metrics: FileMetrics,
}

/// A single code element (function, struct, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    /// Type of the code element
    pub element_type: ElementType,
    /// Name of the element
    pub name: String,
    /// Full signature (for functions)
    pub signature: Option<String>,
    /// Visibility modifier
    pub visibility: Visibility,
    /// Documentation comments
    pub doc_comments: Vec<String>,
    /// Inline comments
    pub inline_comments: Vec<String>,
    /// Location in source code
    pub location: CodeLocation,
    /// Attributes applied to this element
    pub attributes: Vec<String>,
    /// Cyclomatic complexity (if applicable)
    pub complexity: Option<u32>,
    /// Detailed complexity metrics
    pub complexity_metrics: Option<crate::complexity::ComplexityMetrics>,
    /// Dependencies on other elements
    pub dependencies: Vec<String>,
    /// Generic parameters
    pub generic_params: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of code elements that can be extracted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    /// Function definition
    Function,
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Trait definition
    Trait,
    /// Implementation block
    Impl,
    /// Module definition
    Module,
    /// Constant definition
    Constant,
    /// Static variable
    Static,
    /// Type alias
    TypeAlias,
    /// Macro definition
    Macro,
    /// Union definition
    Union,
}

/// Visibility levels for code elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Public visibility
    Public,
    /// Restricted visibility (e.g., pub(crate))
    Restricted(String),
    /// Private (inherited) visibility
    Private,
}

/// Location information for code elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    /// Starting line number
    pub line_start: usize,
    /// Ending line number
    pub line_end: usize,
    /// Starting character position
    pub char_start: usize,
    /// Ending character position
    pub char_end: usize,
    /// File path
    pub file_path: PathBuf,
}

/// Import/use statement information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    /// Module path being imported
    pub module_path: String,
    /// Specific items being imported
    pub imported_items: Vec<String>,
    /// Whether this is a glob import
    pub is_glob: bool,
    /// Alias for the import
    pub alias: Option<String>,
}

/// Dependency information for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Direct dependencies
    pub direct: Vec<String>,
    /// Transitive dependencies
    pub transitive: Vec<String>,
    /// Development dependencies
    pub dev_dependencies: Vec<String>,
}

/// Project-wide metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Total lines of code
    pub total_lines: usize,
    /// Total number of files
    pub total_files: usize,
    /// Total number of functions
    pub total_functions: usize,
    /// Total number of structs
    pub total_structs: usize,
    /// Total number of enums
    pub total_enums: usize,
    /// Total number of traits
    pub total_traits: usize,
    /// Average complexity
    pub complexity_average: f64,
    /// Maximum complexity
    pub complexity_max: u32,
}

/// File-level metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    /// Lines of code (excluding comments/blank lines)
    pub lines_of_code: usize,
    /// Lines of comments
    pub lines_of_comments: usize,
    /// Total complexity for this file
    pub complexity_total: u32,
    /// Number of functions in this file
    pub function_count: usize,
    /// Number of structs in this file
    pub struct_count: usize,
    /// Number of enums in this file
    pub enum_count: usize,
    /// Number of traits in this file
    pub trait_count: usize,
}
