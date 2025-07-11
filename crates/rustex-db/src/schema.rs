//! Database schema definitions and data structures.

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rustex_core::{ProjectAst, FileAst, CodeElement, Visibility};
use crate::error::{ConfigError, Result};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Query timeout in seconds
    pub query_timeout: u64,
    /// Enable SSL mode
    pub ssl_mode: bool,
    /// Application name for connection tracking
    pub application_name: String,
}

impl DbConfig {
    /// Create a new database configuration from a URL.
    pub fn from_url(url: &str) -> Result<Self> {
        if url.is_empty() {
            return Err(ConfigError::InvalidUrl("Empty URL".to_string()).into());
        }

        if !url.starts_with("postgresql://") && !url.starts_with("postgres://") {
            return Err(ConfigError::InvalidUrl("Must be a PostgreSQL URL".to_string()).into());
        }

        Ok(Self {
            database_url: url.to_string(),
            max_connections: 10,
            connection_timeout: 30,
            query_timeout: 60,
            ssl_mode: true,
            application_name: "rustex".to_string(),
        })
    }

    /// Create a configuration for local development.
    pub fn local_dev() -> Self {
        Self {
            database_url: "postgresql://rustex:rustex@localhost:5432/rustex_dev".to_string(),
            max_connections: 5,
            connection_timeout: 10,
            query_timeout: 30,
            ssl_mode: false,
            application_name: "rustex-dev".to_string(),
        }
    }

    /// Create a configuration for testing.
    pub fn test() -> Self {
        Self {
            database_url: "postgresql://rustex:rustex@localhost:5432/rustex_test".to_string(),
            max_connections: 2,
            connection_timeout: 5,
            query_timeout: 10,
            ssl_mode: false,
            application_name: "rustex-test".to_string(),
        }
    }
}

/// Project record in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub rust_edition: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository_url: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub readme_path: Option<String>,
    pub build_script: Option<String>,
    pub workspace_root: Option<String>,
    pub target_directory: Option<String>,
    pub features: serde_json::Value,
    pub dependencies: serde_json::Value,
    pub dev_dependencies: serde_json::Value,
    pub build_dependencies: serde_json::Value,
    pub total_files: i32,
    pub total_lines: i64,
    pub total_functions: i32,
    pub total_structs: i32,
    pub total_enums: i32,
    pub total_traits: i32,
    pub total_modules: i32,
    pub total_impls: i32,
    pub complexity_average: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub analyzed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl ProjectRecord {
    /// Create a new project record from a ProjectAst.
    pub fn from_project_ast(project_ast: &ProjectAst) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            name: project_ast.project.name.clone(),
            version: project_ast.project.version.clone(),
            rust_edition: project_ast.project.rust_edition.clone(),
            description: None, // Not available in current ProjectInfo
            authors: Vec::new(), // Not available in current ProjectInfo
            license: None, // Not available in current ProjectInfo
            repository_url: None, // Not available in current ProjectInfo
            homepage: None, // Not available in current ProjectInfo
            keywords: Vec::new(), // Not available in current ProjectInfo
            categories: Vec::new(), // Not available in current ProjectInfo
            readme_path: None, // Not available in current ProjectInfo
            build_script: None, // Not available in current ProjectInfo
            workspace_root: None, // TODO: Extract from project
            target_directory: None, // TODO: Extract from project
            features: serde_json::json!({}), // TODO: Extract from project
            dependencies: serde_json::json!({}), // TODO: Extract from project
            dev_dependencies: serde_json::json!({}), // TODO: Extract from project
            build_dependencies: serde_json::json!({}), // TODO: Extract from project
            total_files: project_ast.metrics.total_files as i32,
            total_lines: project_ast.metrics.total_lines as i64,
            total_functions: project_ast.metrics.total_functions as i32,
            total_structs: project_ast.metrics.total_structs as i32,
            total_enums: project_ast.metrics.total_enums as i32,
            total_traits: project_ast.metrics.total_traits as i32,
            total_modules: 0, // Not available in current ProjectMetrics
            total_impls: 0, // Not available in current ProjectMetrics
            complexity_average: project_ast.metrics.complexity_average,
            created_at: now,
            updated_at: now,
            analyzed_at: now,
            metadata: serde_json::json!({}),
        }
    }
}

/// File record in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub path: String,
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: i64,
    pub lines_of_code: i32,
    pub function_count: i32,
    pub struct_count: i32,
    pub enum_count: i32,
    pub trait_count: i32,
    pub module_count: i32,
    pub impl_count: i32,
    pub use_count: i32,
    pub macro_count: i32,
    pub const_count: i32,
    pub static_count: i32,
    pub type_alias_count: i32,
    pub complexity_total: i32,
    pub complexity_average: f64,
    pub documentation_coverage: f64,
    pub test_coverage: Option<f64>,
    pub last_modified: DateTime<Utc>,
    pub analyzed_at: DateTime<Utc>,
    pub content_hash: String,
    pub syntax_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: serde_json::Value,
}

impl FileRecord {
    /// Create a new file record from a FileAst.
    pub fn from_file_ast(file_ast: &FileAst, project_id: Uuid) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            project_id,
            path: file_ast.path.to_string_lossy().to_string(),
            relative_path: file_ast.relative_path.to_string_lossy().to_string(),
            file_name: file_ast.path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            extension: file_ast.path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "rs".to_string()),
            size_bytes: 0, // TODO: Get actual file size
            lines_of_code: file_ast.file_metrics.lines_of_code as i32,
            function_count: file_ast.file_metrics.function_count as i32,
            struct_count: file_ast.file_metrics.struct_count as i32,
            enum_count: file_ast.file_metrics.enum_count as i32,
            trait_count: file_ast.file_metrics.trait_count as i32,
            module_count: 0, // Not available in current FileMetrics
            impl_count: 0, // Not available in current FileMetrics
            use_count: 0, // Not available in current FileMetrics
            macro_count: 0, // Not available in current FileMetrics
            const_count: 0, // Not available in current FileMetrics
            static_count: 0, // Not available in current FileMetrics
            type_alias_count: 0, // Not available in current FileMetrics
            complexity_total: file_ast.file_metrics.complexity_total as i32,
            complexity_average: file_ast.file_metrics.complexity_total as f64 / file_ast.file_metrics.function_count.max(1) as f64,
            documentation_coverage: 0.0, // Not available in current FileMetrics
            test_coverage: None, // TODO: Calculate test coverage
            last_modified: now, // TODO: Get actual file modification time
            analyzed_at: now,
            content_hash: String::new(), // TODO: Calculate content hash
            syntax_errors: Vec::new(), // TODO: Collect syntax errors
            warnings: Vec::new(), // TODO: Collect warnings
            metadata: serde_json::json!({}),
        }
    }
}

/// AST element record in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ElementRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_id: Uuid,
    pub element_id: String, // Original element ID from AST
    pub element_type: String,
    pub name: String,
    pub qualified_name: String,
    pub signature: Option<String>,
    pub visibility: String,
    pub line_start: i32,
    pub line_end: i32,
    pub char_start: i32,
    pub char_end: i32,
    pub complexity: Option<i32>,
    pub cyclomatic_complexity: Option<i32>,
    pub cognitive_complexity: Option<i32>,
    pub nesting_depth: Option<i32>,
    pub parameter_count: Option<i32>,
    pub return_count: Option<i32>,
    pub lines_of_code: Option<i32>,
    pub halstead_metrics: Option<serde_json::Value>,
    pub doc_comments: Vec<String>,
    pub inline_comments: Vec<String>,
    pub attributes: Vec<String>,
    pub dependencies: Vec<String>,
    pub generic_params: Vec<String>,
    pub module_path: String,
    pub parent_element_id: Option<Uuid>,
    pub nesting_level: i32,
    pub is_public: bool,
    pub is_test: bool,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_deprecated: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl ElementRecord {
    /// Create a new element record from a CodeElement.
    pub fn from_code_element(
        element: &CodeElement, 
        project_id: Uuid, 
        file_id: Uuid
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            project_id,
            file_id,
            element_id: element.id.clone(),
            element_type: format!("{:?}", element.element_type),
            name: element.name.clone(),
            qualified_name: element.hierarchy.qualified_name.clone(),
            signature: element.signature.clone(),
            visibility: format!("{:?}", element.visibility),
            line_start: element.location.line_start as i32,
            line_end: element.location.line_end as i32,
            char_start: element.location.char_start as i32,
            char_end: element.location.char_end as i32,
            complexity: element.complexity.map(|c| c as i32),
            cyclomatic_complexity: element.complexity_metrics.as_ref()
                .map(|m| m.cyclomatic as i32),
            cognitive_complexity: element.complexity_metrics.as_ref()
                .map(|m| m.cognitive as i32),
            nesting_depth: element.complexity_metrics.as_ref()
                .map(|m| m.nesting_depth as i32),
            parameter_count: element.complexity_metrics.as_ref()
                .map(|m| m.parameter_count as i32),
            return_count: element.complexity_metrics.as_ref()
                .map(|m| m.return_count as i32),
            lines_of_code: element.complexity_metrics.as_ref()
                .map(|m| m.lines_of_code as i32),
            halstead_metrics: element.complexity_metrics.as_ref()
                .map(|m| serde_json::to_value(&m.halstead).unwrap_or_default()),
            doc_comments: element.doc_comments.clone(),
            inline_comments: element.inline_comments.clone(),
            attributes: element.attributes.clone(),
            dependencies: element.dependencies.clone(),
            generic_params: element.generic_params.clone(),
            module_path: element.hierarchy.module_path.clone(),
            parent_element_id: None, // TODO: Map parent IDs
            nesting_level: element.hierarchy.nesting_level as i32,
            is_public: element.visibility == Visibility::Public,
            is_test: element.attributes.iter().any(|attr| attr.contains("test")),
            is_async: element.signature.as_ref()
                .map(|s| s.contains("async"))
                .unwrap_or(false),
            is_unsafe: element.signature.as_ref()
                .map(|s| s.contains("unsafe"))
                .unwrap_or(false),
            is_deprecated: element.attributes.iter()
                .any(|attr| attr.contains("deprecated")),
            created_at: now,
            updated_at: now,
            metadata: serde_json::json!({}),
        }
    }
}

/// Cross-reference relationship record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CrossReferenceRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub from_element_id: Uuid,
    pub to_element_id: Option<Uuid>,
    pub reference_type: String,
    pub reference_text: String,
    pub line_number: i32,
    pub char_position: i32,
    pub context_scope: String,
    pub is_definition: bool,
    pub is_resolved: bool,
    pub confidence_score: f64,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Dependency relationship record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DependencyRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub from_element_id: Uuid,
    pub to_element_id: Uuid,
    pub dependency_type: String,
    pub strength: f64,
    pub is_direct: bool,
    pub is_cyclic: bool,
    pub path_length: i32,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Call chain record for function call relationships
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CallChainRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub caller_id: Uuid,
    pub callee_id: Uuid,
    pub call_type: String, // "direct", "indirect", "virtual", "trait"
    pub call_count: i32,
    pub call_sites: Vec<i32>, // Line numbers where calls occur
    pub is_recursive: bool,
    pub recursion_depth: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Type relationship record for struct/enum/trait relationships
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TypeRelationshipRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub from_type_id: Uuid,
    pub to_type_id: Uuid,
    pub relationship_type: String, // "implements", "extends", "uses", "contains"
    pub relationship_strength: f64,
    pub is_generic: bool,
    pub generic_constraints: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}