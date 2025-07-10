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
    /// Project-wide cross-references
    pub cross_references: Vec<CrossReference>,
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
    /// Cross-references within this file
    pub cross_references: Vec<CrossReference>,
}

/// A single code element (function, struct, etc.) with hierarchical relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    /// Unique identifier for this element
    pub id: String,
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
    /// Hierarchical relationships
    pub hierarchy: ElementHierarchy,
}

/// Types of code elements that can be extracted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Hierarchical relationship information for code elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementHierarchy {
    /// ID of the parent element (None for top-level elements)
    pub parent_id: Option<String>,
    /// IDs of direct child elements
    pub children_ids: Vec<String>,
    /// Nesting level (0 for top-level, 1 for first level nesting, etc.)
    pub nesting_level: u32,
    /// Module path from root (e.g., "crate::module::submodule")
    pub module_path: String,
    /// Fully qualified name including parent context
    pub qualified_name: String,
    /// Namespace information for this element
    pub namespace: ElementNamespace,
}

/// Namespace information for code elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNamespace {
    /// The element's simple name
    pub simple_name: String,
    /// Full canonical path (e.g., "crate::module::MyStruct")
    pub canonical_path: String,
    /// Alternative names this element can be referenced by
    pub aliases: Vec<String>,
    /// Import paths that bring this element into scope
    pub import_paths: Vec<String>,
    /// Whether this element is publicly accessible
    pub is_public: bool,
    /// Visibility scope (crate, super, self, etc.)
    pub visibility_scope: VisibilityScope,
}

/// Visibility scope for namespace resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityScope {
    /// Public to all crates
    Public,
    /// Visible within current crate
    Crate,
    /// Visible within parent module
    Super,
    /// Visible within current module only
    Module(String),
    /// Private to current item
    Private,
}

/// Hierarchical structure helper for building relationships during AST traversal.
#[derive(Debug, Clone)]
pub struct HierarchyBuilder {
    /// Stack of parent elements during traversal
    parent_stack: Vec<String>,
    /// Current module path
    current_module_path: String,
    /// Element counter for generating unique IDs
    element_counter: u32,
}

impl ElementHierarchy {
    /// Create a new hierarchy with no parent (top-level element).
    pub fn new_root(module_path: String, qualified_name: String, namespace: ElementNamespace) -> Self {
        Self {
            parent_id: None,
            children_ids: Vec::new(),
            nesting_level: 0,
            module_path,
            qualified_name,
            namespace,
        }
    }
    
    /// Create a new hierarchy with a parent element.
    pub fn new_child(parent_id: String, nesting_level: u32, module_path: String, qualified_name: String, namespace: ElementNamespace) -> Self {
        Self {
            parent_id: Some(parent_id),
            children_ids: Vec::new(),
            nesting_level,
            module_path,
            qualified_name,
            namespace,
        }
    }
    
    /// Add a child element ID to this hierarchy.
    pub fn add_child(&mut self, child_id: String) {
        self.children_ids.push(child_id);
    }
}

impl HierarchyBuilder {
    /// Create a new hierarchy builder for a file.
    pub fn new(module_path: String) -> Self {
        Self {
            parent_stack: Vec::new(),
            current_module_path: module_path,
            element_counter: 0,
        }
    }
    
    /// Generate a unique element ID.
    pub fn generate_id(&mut self, element_type: &ElementType, name: &str) -> String {
        self.element_counter += 1;
        format!("{:?}_{}_{}", element_type, name, self.element_counter)
    }
    
    /// Enter a new scope (push parent onto stack).
    pub fn enter_scope(&mut self, parent_id: String) {
        self.parent_stack.push(parent_id);
    }
    
    /// Exit current scope (pop parent from stack).
    pub fn exit_scope(&mut self) {
        self.parent_stack.pop();
    }
    
    /// Get the current parent ID (top of stack).
    pub fn current_parent(&self) -> Option<&String> {
        self.parent_stack.last()
    }
    
    /// Get current nesting level.
    pub fn current_nesting_level(&self) -> u32 {
        self.parent_stack.len() as u32
    }
    
    /// Build hierarchy for an element with namespace information.
    pub fn build_hierarchy(&self, _element_type: &ElementType, name: &str, visibility: &Visibility) -> ElementHierarchy {
        let qualified_name = self.build_qualified_name(name);
        let canonical_path = self.build_canonical_path(name);
        
        let namespace = ElementNamespace::new(
            name.to_string(),
            canonical_path,
            visibility,
        );
        
        if let Some(parent_id) = self.current_parent() {
            ElementHierarchy::new_child(
                parent_id.clone(),
                self.current_nesting_level(),
                self.current_module_path.clone(),
                qualified_name,
                namespace,
            )
        } else {
            ElementHierarchy::new_root(
                self.current_module_path.clone(),
                qualified_name,
                namespace,
            )
        }
    }
    
    /// Build a qualified name including parent context.
    fn build_qualified_name(&self, name: &str) -> String {
        if let Some(parent) = self.current_parent() {
            format!("{}::{}", parent, name)
        } else {
            format!("{}::{}", self.current_module_path, name)
        }
    }
    
    /// Build a canonical path for namespace resolution.
    fn build_canonical_path(&self, name: &str) -> String {
        format!("{}::{}", self.current_module_path, name)
    }
    
    /// Get the current namespace scope.
    pub fn current_namespace_scope(&self) -> String {
        self.current_module_path.clone()
    }
    
    /// Update module path (when entering a module).
    pub fn enter_module(&mut self, module_name: &str) {
        if self.current_module_path == "crate" {
            self.current_module_path = format!("crate::{}", module_name);
        } else {
            self.current_module_path.push_str(&format!("::{}", module_name));
        }
    }
    
    /// Exit module (when leaving a module).
    pub fn exit_module(&mut self) {
        if let Some(pos) = self.current_module_path.rfind("::") {
            self.current_module_path.truncate(pos);
        } else {
            self.current_module_path = "crate".to_string();
        }
    }
}

/// Cross-reference information tracking relationships between code elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// ID of the referencing element
    pub from_element_id: String,
    /// ID of the referenced element (if resolved)
    pub to_element_id: Option<String>,
    /// Type of reference
    pub reference_type: ReferenceType,
    /// Raw reference text as it appears in code
    pub reference_text: String,
    /// Location where the reference occurs
    pub location: CodeLocation,
    /// Whether the reference was successfully resolved
    pub is_resolved: bool,
    /// Additional context about the reference
    pub context: ReferenceContext,
}

/// Types of cross-references that can be tracked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Function or method call
    FunctionCall,
    /// Type usage (struct, enum, trait)
    TypeUsage,
    /// Variable or field access
    VariableAccess,
    /// Module reference
    ModuleReference,
    /// Trait implementation
    TraitImplementation,
    /// Generic parameter usage
    GenericUsage,
    /// Macro invocation
    MacroInvocation,
    /// Import/use statement
    ImportReference,
}

/// Context information for cross-references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceContext {
    /// Whether this is a definition or usage
    pub is_definition: bool,
    /// Scope where the reference occurs
    pub scope: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Cross-reference resolver for tracking relationships between elements.
#[derive(Debug, Clone)]
pub struct CrossReferenceResolver {
    /// Map of element names to their IDs for quick lookup
    element_map: HashMap<String, Vec<String>>,
    /// Collected cross-references
    cross_references: Vec<CrossReference>,
    /// Current scope stack for reference resolution
    scope_stack: Vec<String>,
    /// Import aliases for resolving references
    import_aliases: HashMap<String, String>,
}

impl CrossReference {
    /// Create a new cross-reference.
    pub fn new(
        from_element_id: String,
        reference_type: ReferenceType,
        reference_text: String,
        location: CodeLocation,
        context: ReferenceContext,
    ) -> Self {
        Self {
            from_element_id,
            to_element_id: None,
            reference_type,
            reference_text,
            location,
            is_resolved: false,
            context,
        }
    }
    
    /// Mark this reference as resolved to a specific element.
    pub fn resolve_to(&mut self, target_element_id: String) {
        self.to_element_id = Some(target_element_id);
        self.is_resolved = true;
    }
}

impl ReferenceContext {
    /// Create a new reference context.
    pub fn new(is_definition: bool, scope: String) -> Self {
        Self {
            is_definition,
            scope,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the context.
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl CrossReferenceResolver {
    /// Create a new cross-reference resolver.
    pub fn new() -> Self {
        Self {
            element_map: HashMap::new(),
            cross_references: Vec::new(),
            scope_stack: Vec::new(),
            import_aliases: HashMap::new(),
        }
    }
    
    /// Register an element for cross-reference resolution.
    pub fn register_element(&mut self, name: String, element_id: String) {
        self.element_map
            .entry(name)
            .or_default()
            .push(element_id);
    }
    
    /// Add an import alias for reference resolution.
    pub fn add_import_alias(&mut self, alias: String, target: String) {
        self.import_aliases.insert(alias, target);
    }
    
    /// Enter a new scope for reference resolution.
    pub fn enter_scope(&mut self, scope: String) {
        self.scope_stack.push(scope);
    }
    
    /// Exit the current scope.
    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }
    
    /// Get current scope as a string.
    pub fn current_scope(&self) -> String {
        self.scope_stack.join("::")        
    }
    
    /// Add a cross-reference (initially unresolved).
    pub fn add_reference(&mut self, reference: CrossReference) {
        self.cross_references.push(reference);
    }
    
    /// Resolve all cross-references based on registered elements.
    pub fn resolve_references(&mut self) {
        // Collect references to resolve to avoid borrowing conflicts
        let references_to_resolve: Vec<(usize, String, String)> = self.cross_references
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.is_resolved)
            .map(|(i, r)| (i, r.reference_text.clone(), r.context.scope.clone()))
            .collect();
            
        for (index, reference_text, scope) in references_to_resolve {
            if let Some(target_id) = self.resolve_reference_text(&reference_text, &scope) {
                self.cross_references[index].resolve_to(target_id);
            }
        }
    }
    
    /// Attempt to resolve a reference text to an element ID.
    fn resolve_reference_text(&self, reference_text: &str, scope: &str) -> Option<String> {
        // Check import aliases first
        if let Some(aliased) = self.import_aliases.get(reference_text) {
            return self.find_element_by_name(aliased);
        }
        
        // Try direct name lookup
        if let Some(element_id) = self.find_element_by_name(reference_text) {
            return Some(element_id);
        }
        
        // Try scope-qualified lookup
        let qualified_name = format!("{}::{}", scope, reference_text);
        self.find_element_by_name(&qualified_name)
    }
    
    /// Find an element by name, returning the first match.
    fn find_element_by_name(&self, name: &str) -> Option<String> {
        self.element_map.get(name)?.first().cloned()
    }
    
    /// Get all collected cross-references.
    pub fn get_cross_references(&self) -> &[CrossReference] {
        &self.cross_references
    }
    
    /// Get cross-references for a specific element.
    pub fn get_references_from_element(&self, element_id: &str) -> Vec<&CrossReference> {
        self.cross_references
            .iter()
            .filter(|r| r.from_element_id == element_id)
            .collect()
    }
    
    /// Get cross-references to a specific element.
    pub fn get_references_to_element(&self, element_id: &str) -> Vec<&CrossReference> {
        self.cross_references
            .iter()
            .filter(|r| r.to_element_id.as_deref() == Some(element_id))
            .collect()
    }
}

impl ElementNamespace {
    /// Create a new namespace for an element.
    pub fn new(simple_name: String, canonical_path: String, visibility: &Visibility) -> Self {
        let (is_public, visibility_scope) = match visibility {
            Visibility::Public => (true, VisibilityScope::Public),
            Visibility::Restricted(scope) => {
                let is_public = scope.contains("pub");
                let scope_type = if scope.contains("crate") {
                    VisibilityScope::Crate
                } else if scope.contains("super") {
                    VisibilityScope::Super
                } else {
                    VisibilityScope::Module(scope.clone())
                };
                (is_public, scope_type)
            },
            Visibility::Private => (false, VisibilityScope::Private),
        };
        
        Self {
            simple_name,
            canonical_path,
            aliases: Vec::new(),
            import_paths: Vec::new(),
            is_public,
            visibility_scope,
        }
    }
    
    /// Add an alias for this element.
    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
    }
    
    /// Add an import path that brings this element into scope.
    pub fn add_import_path(&mut self, import_path: String) {
        if !self.import_paths.contains(&import_path) {
            self.import_paths.push(import_path);
        }
    }
    
    /// Check if this element is accessible from a given scope.
    pub fn is_accessible_from(&self, from_scope: &str) -> bool {
        match &self.visibility_scope {
            VisibilityScope::Public => true,
            VisibilityScope::Crate => true, // Assume same crate for now
            VisibilityScope::Super => {
                // Check if from_scope is a child of this element's parent
                from_scope.starts_with(&self.canonical_path)
            },
            VisibilityScope::Module(module) => {
                from_scope.starts_with(module)
            },
            VisibilityScope::Private => {
                // Only accessible from same module
                from_scope == self.canonical_path
            },
        }
    }
    
    /// Get all possible names this element can be referenced by.
    pub fn get_reference_names(&self) -> Vec<String> {
        let mut names = vec![self.simple_name.clone(), self.canonical_path.clone()];
        names.extend(self.aliases.clone());
        names.extend(self.import_paths.clone());
        names
    }
}

impl Default for CrossReferenceResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Namespace resolver for handling imports and qualified names.
#[derive(Debug, Clone)]
pub struct NamespaceResolver {
    /// Map of import aliases to their full paths
    import_map: HashMap<String, String>,
    /// Map of use statements to imported items
    use_map: HashMap<String, Vec<String>>,
    /// Current module path context
    current_module: String,
    /// Map of simple names to their canonical paths in current scope
    scope_map: HashMap<String, String>,
    /// External crate dependencies
    extern_crates: HashMap<String, String>,
}

impl NamespaceResolver {
    /// Create a new namespace resolver.
    pub fn new(module_path: String) -> Self {
        Self {
            import_map: HashMap::new(),
            use_map: HashMap::new(),
            current_module: module_path,
            scope_map: HashMap::new(),
            extern_crates: HashMap::new(),
        }
    }
    
    /// Add a use statement to the resolver.
    pub fn add_use_statement(&mut self, import_info: &ImportInfo) {
        let module_path = &import_info.module_path;
        
        if import_info.is_glob {
            // Handle glob imports like `use std::collections::*;`
            self.use_map.insert(module_path.clone(), vec!["*".to_string()]);
        } else if !import_info.imported_items.is_empty() {
            // Handle specific imports like `use std::collections::{HashMap, BTreeMap};`
            for item in &import_info.imported_items {
                let full_path = if module_path.is_empty() {
                    item.clone()
                } else {
                    format!("{}::{}", module_path, item)
                };
                
                let local_name = if let Some(alias) = &import_info.alias {
                    alias.clone()
                } else {
                    item.clone()
                };
                
                self.import_map.insert(local_name, full_path.clone());
                self.scope_map.insert(item.clone(), full_path);
            }
        } else {
            // Handle module imports like `use std::collections;`
            let local_name = if let Some(alias) = &import_info.alias {
                alias.clone()
            } else {
                module_path.split("::").last().unwrap_or(module_path).to_string()
            };
            
            self.import_map.insert(local_name, module_path.clone());
        }
    }
    
    /// Resolve a reference to its canonical path.
    pub fn resolve_reference(&self, reference: &str) -> Option<String> {
        // Check direct imports first
        if let Some(full_path) = self.import_map.get(reference) {
            return Some(full_path.clone());
        }
        
        // Check scope map
        if let Some(full_path) = self.scope_map.get(reference) {
            return Some(full_path.clone());
        }
        
        // Check if it's a qualified path
        if reference.contains("::") {
            return Some(reference.to_string());
        }
        
        // Try to resolve within current module
        let current_module_path = format!("{}::{}", self.current_module, reference);
        Some(current_module_path)
    }
    
    /// Add an external crate dependency.
    pub fn add_extern_crate(&mut self, crate_name: String, crate_path: String) {
        self.extern_crates.insert(crate_name, crate_path);
    }
    
    /// Check if a path is accessible from the current scope.
    pub fn is_accessible(&self, path: &str, from_scope: &str) -> bool {
        // Simple accessibility check - can be enhanced
        if path.starts_with("pub ") || path.starts_with("crate::") {
            return true;
        }
        
        // Check if it's in the same module or a parent module
        from_scope.starts_with(path) || path.starts_with(from_scope)
    }
    
    /// Get all possible names for a given canonical path.
    pub fn get_aliases_for_path(&self, canonical_path: &str) -> Vec<String> {
        let mut aliases = Vec::new();
        
        // Add direct aliases
        for (alias, path) in &self.import_map {
            if path == canonical_path {
                aliases.push(alias.clone());
            }
        }
        
        // Add simple name if it's in scope
        if let Some(simple_name) = canonical_path.split("::").last() {
            if self.scope_map.get(simple_name).map(|s| s.as_str()) == Some(canonical_path) {
                aliases.push(simple_name.to_string());
            }
        }
        
        aliases
    }
}

impl Default for NamespaceResolver {
    fn default() -> Self {
        Self::new("crate".to_string())
    }
}
