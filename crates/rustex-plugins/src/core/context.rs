//! Plugin execution context types.

use std::collections::HashMap;
use rustex_core::{ProjectInfo, FileAst, ProjectAst, ExtractorConfig, CodeElement};

/// Main plugin execution context.
#[derive(Debug)]
pub struct PluginContext<'a> {
    /// Project information
    pub project_info: &'a ProjectInfo,
    
    /// Current file being processed (if applicable)
    pub current_file: Option<&'a mut FileAst>,
    
    /// Complete project AST (if available)
    pub project_ast: Option<&'a mut ProjectAst>,
    
    /// Extractor configuration
    pub config: &'a ExtractorConfig,
    
    /// Plugin-specific metadata
    pub metadata: &'a mut HashMap<String, serde_json::Value>,
    
    /// Context type indicator
    pub context_type: ContextType,
}

/// Type of plugin context.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextType {
    /// Pre-processing context
    PreProcess,
    
    /// Single file processing context
    FileProcess,
    
    /// Project-level processing context
    ProjectProcess,
    
    /// Formatting context
    Format,
}

impl<'a> PluginContext<'a> {
    /// Create a new pre-processing context.
    pub fn new_pre_process(
        project_info: &'a ProjectInfo,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_info,
            current_file: None,
            project_ast: None,
            config,
            metadata,
            context_type: ContextType::PreProcess,
        }
    }
    
    /// Create a new file processing context.
    pub fn new_file_process(
        project_info: &'a ProjectInfo,
        current_file: &'a mut FileAst,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_info,
            current_file: Some(current_file),
            project_ast: None,
            config,
            metadata,
            context_type: ContextType::FileProcess,
        }
    }
    
    /// Create a new project processing context.
    pub fn new_project_process(
        project_info: &'a ProjectInfo,
        project_ast: &'a mut ProjectAst,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_info,
            current_file: None,
            project_ast: Some(project_ast),
            config,
            metadata,
            context_type: ContextType::ProjectProcess,
        }
    }
    
    /// Create a new formatting context.
    pub fn new_format(
        project_info: &'a ProjectInfo,
        project_ast: &'a mut ProjectAst,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_info,
            current_file: None,
            project_ast: Some(project_ast),
            config,
            metadata,
            context_type: ContextType::Format,
        }
    }
    
    /// Get the current file being processed (if any).
    pub fn current_file(&self) -> Option<&FileAst> {
        self.current_file.as_ref().map(|f| &**f)
    }
    
    /// Get a mutable reference to the current file (if any).
    pub fn current_file_mut(&mut self) -> Option<&mut FileAst> {
        self.current_file.as_mut().map(|f| &mut **f)
    }
    
    /// Get the project AST (if available).
    pub fn project_ast(&self) -> Option<&ProjectAst> {
        self.project_ast.as_ref().map(|p| &**p)
    }
    
    /// Get a mutable reference to the project AST (if available).
    pub fn project_ast_mut(&mut self) -> Option<&mut ProjectAst> {
        self.project_ast.as_mut().map(|p| &mut **p)
    }
    
    /// Get all code elements from current context.
    pub fn all_elements(&self) -> Vec<&CodeElement> {
        match &self.context_type {
            ContextType::FileProcess => {
                if let Some(file) = &self.current_file {
                    file.elements.iter().collect()
                } else {
                    vec![]
                }
            }
            ContextType::ProjectProcess | ContextType::Format => {
                if let Some(project) = &self.project_ast {
                    project.files.iter()
                        .flat_map(|file| &file.elements)
                        .collect()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
    
    /// Get metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Set metadata value.
    pub fn set_metadata<V>(&mut self, key: &str, value: V) 
    where 
        V: Into<serde_json::Value>,
    {
        self.metadata.insert(key.to_string(), value.into());
    }
    
    /// Check if this is a file processing context.
    pub fn is_file_context(&self) -> bool {
        matches!(self.context_type, ContextType::FileProcess)
    }
    
    /// Check if this is a project processing context.
    pub fn is_project_context(&self) -> bool {
        matches!(
            self.context_type, 
            ContextType::ProjectProcess | ContextType::Format
        )
    }
}

/// Pre-processing context for plugins that run before file discovery.
#[derive(Debug)]
pub struct PreProcessContext<'a> {
    /// Project information
    pub project_info: &'a ProjectInfo,
    
    /// Extractor configuration
    pub config: &'a ExtractorConfig,
    
    /// Global metadata
    pub metadata: &'a mut HashMap<String, serde_json::Value>,
}

impl<'a> PreProcessContext<'a> {
    /// Create a new pre-processing context.
    pub fn new(
        project_info: &'a ProjectInfo,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_info,
            config,
            metadata,
        }
    }
}

/// Post-processing context for plugins that run after extraction.
#[derive(Debug)]
pub struct PostProcessContext<'a> {
    /// Complete project AST
    pub project_ast: &'a mut ProjectAst,
    
    /// Extractor configuration
    pub config: &'a ExtractorConfig,
    
    /// Global metadata
    pub metadata: &'a mut HashMap<String, serde_json::Value>,
}

impl<'a> PostProcessContext<'a> {
    /// Create a new post-processing context.
    pub fn new(
        project_ast: &'a mut ProjectAst,
        config: &'a ExtractorConfig,
        metadata: &'a mut HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            project_ast,
            config,
            metadata,
        }
    }
    
    /// Get all code elements from all files.
    pub fn all_elements(&self) -> Vec<&CodeElement> {
        self.project_ast.files.iter()
            .flat_map(|file| &file.elements)
            .collect()
    }
    
    /// Get all elements of a specific type.
    pub fn elements_by_type(&self, element_type: rustex_core::ElementType) -> Vec<&CodeElement> {
        self.all_elements()
            .into_iter()
            .filter(|element| element.element_type == element_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustex_core::ProjectInfo;
    use std::path::PathBuf;
    // use chrono::Utc; // Unused import removed

    fn create_test_project_info() -> ProjectInfo {
        ProjectInfo {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            rust_edition: "2021".to_string(),
            root_path: PathBuf::from("/test"),
        }
    }

    fn create_test_config() -> ExtractorConfig {
        ExtractorConfig::default()
    }

    #[test]
    fn test_context_creation() {
        let project_info = create_test_project_info();
        let config = create_test_config();
        let mut metadata = HashMap::new();
        
        let context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        assert_eq!(context.context_type, ContextType::PreProcess);
        assert!(context.current_file.is_none());
        assert!(context.project_ast.is_none());
    }

    #[test]
    fn test_metadata_operations() {
        let project_info = create_test_project_info();
        let config = create_test_config();
        let mut metadata = HashMap::new();
        
        let mut context = PluginContext::new_pre_process(&project_info, &config, &mut metadata);
        context.set_metadata("test_key", "test_value");
        
        assert_eq!(
            context.get_metadata("test_key"), 
            Some(&serde_json::Value::String("test_value".to_string()))
        );
    }
}