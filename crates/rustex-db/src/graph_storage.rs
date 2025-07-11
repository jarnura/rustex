//! Graph storage layer for call chains and dependency relationships.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::collections::HashMap;
use rustex_core::{ProjectAst, ElementType, ReferenceType};
use crate::error::{DatabaseError, Result};
use crate::storage::AstStorage;

/// Graph storage manager for call chains and dependencies
pub struct GraphStorage {
    pool: PgPool,
    ast_storage: AstStorage,
}

impl GraphStorage {
    /// Create a new graph storage instance.
    pub fn new(pool: PgPool) -> Self {
        let ast_storage = AstStorage::new(pool.clone());
        Self { pool, ast_storage }
    }

    /// Build and store complete call chain graph from project AST.
    pub async fn build_call_chain_graph(&self, project_ast: &ProjectAst) -> Result<CallChainStats> {
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        let project_id = match self.get_project_id_by_name(&project_ast.project.name, &project_ast.project.version).await? {
            Some(id) => id,
            None => {
                // Store the project first if it doesn't exist
                self.ast_storage.store_project_ast(project_ast).await?
            }
        };

        // Extract function elements and build call relationships
        let function_elements = self.extract_function_elements(project_ast);
        let call_relationships = self.analyze_call_relationships(project_ast, &function_elements)?;

        let mut stored_calls = 0;
        let mut recursive_calls = 0;

        // Store call chain records
        for call_chain in call_relationships {
            let is_recursive = call_chain.caller_id == call_chain.callee_id || 
                               self.is_recursive_call(&call_chain, &function_elements);
            
            if is_recursive {
                recursive_calls += 1;
            }

            self.store_call_chain_record(&mut tx, &call_chain, project_id, is_recursive).await?;
            stored_calls += 1;
        }

        tx.commit().await
            .map_err(DatabaseError::from)?;

        Ok(CallChainStats {
            total_functions: function_elements.len(),
            total_call_chains: stored_calls,
            recursive_calls,
            max_call_depth: self.calculate_max_call_depth(project_id).await?,
        })
    }

    /// Build and store dependency graph from project AST.
    pub async fn build_dependency_graph(&self, project_ast: &ProjectAst) -> Result<DependencyStats> {
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        let project_id = self.get_project_id_by_name(&project_ast.project.name, &project_ast.project.version).await?
            .ok_or_else(|| DatabaseError::generic("Project not found"))?;

        // Extract all elements and build dependency relationships
        let all_elements = self.extract_all_elements(project_ast);
        let dependencies = self.analyze_dependencies(project_ast, &all_elements)?;

        let mut stored_deps = 0;
        let mut cyclic_deps = 0;

        // Store dependency records
        for dependency in dependencies {
            let is_cyclic = self.would_create_cycle(&dependency, project_id).await?;
            
            if is_cyclic {
                cyclic_deps += 1;
            }

            self.store_dependency_record(&mut tx, &dependency, project_id, is_cyclic).await?;
            stored_deps += 1;
        }

        tx.commit().await
            .map_err(DatabaseError::from)?;

        Ok(DependencyStats {
            total_elements: all_elements.len(),
            total_dependencies: stored_deps,
            cyclic_dependencies: cyclic_deps,
            dependency_depth: self.calculate_dependency_depth(project_id).await?,
        })
    }

    /// Build and store type relationship graph.
    pub async fn build_type_relationship_graph(&self, project_ast: &ProjectAst) -> Result<TypeRelationshipStats> {
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        let project_id = self.get_project_id_by_name(&project_ast.project.name, &project_ast.project.version).await?
            .ok_or_else(|| DatabaseError::generic("Project not found"))?;

        // Extract type elements (structs, enums, traits)
        let type_elements = self.extract_type_elements(project_ast);
        let type_relationships = self.analyze_type_relationships(project_ast, &type_elements)?;

        let mut stored_relationships = 0;
        let mut trait_implementations = 0;

        // Store type relationship records
        for relationship in type_relationships {
            if relationship.relationship_type == "implements" {
                trait_implementations += 1;
            }

            self.store_type_relationship(&mut tx, &relationship, project_id).await?;
            stored_relationships += 1;
        }

        tx.commit().await
            .map_err(DatabaseError::from)?;

        Ok(TypeRelationshipStats {
            total_types: type_elements.len(),
            total_relationships: stored_relationships,
            trait_implementations,
            inheritance_depth: self.calculate_inheritance_depth(project_id).await?,
        })
    }

    /// Extract function elements from project AST.
    fn extract_function_elements(&self, project_ast: &ProjectAst) -> HashMap<String, ElementInfo> {
        let mut functions = HashMap::new();

        for file_ast in &project_ast.files {
            for element in &file_ast.elements {
                if matches!(element.element_type, ElementType::Function) {
                    functions.insert(
                        element.id.clone(),
                        ElementInfo {
                            id: element.id.clone(),
                            name: element.name.clone(),
                            qualified_name: element.hierarchy.qualified_name.clone(),
                            file_path: file_ast.path.to_string_lossy().to_string(),
                            element_type: element.element_type.clone(),
                        }
                    );
                }
            }
        }

        functions
    }

    /// Extract all elements from project AST.
    fn extract_all_elements(&self, project_ast: &ProjectAst) -> HashMap<String, ElementInfo> {
        let mut elements = HashMap::new();

        for file_ast in &project_ast.files {
            for element in &file_ast.elements {
                elements.insert(
                    element.id.clone(),
                    ElementInfo {
                        id: element.id.clone(),
                        name: element.name.clone(),
                        qualified_name: element.hierarchy.qualified_name.clone(),
                        file_path: file_ast.path.to_string_lossy().to_string(),
                        element_type: element.element_type.clone(),
                    }
                );
            }
        }

        elements
    }

    /// Extract type elements (structs, enums, traits) from project AST.
    fn extract_type_elements(&self, project_ast: &ProjectAst) -> HashMap<String, ElementInfo> {
        let mut types = HashMap::new();

        for file_ast in &project_ast.files {
            for element in &file_ast.elements {
                if matches!(element.element_type, ElementType::Struct | ElementType::Enum | ElementType::Trait) {
                    types.insert(
                        element.id.clone(),
                        ElementInfo {
                            id: element.id.clone(),
                            name: element.name.clone(),
                            qualified_name: element.hierarchy.qualified_name.clone(),
                            file_path: file_ast.path.to_string_lossy().to_string(),
                            element_type: element.element_type.clone(),
                        }
                    );
                }
            }
        }

        types
    }

    /// Analyze call relationships from cross-references.
    fn analyze_call_relationships(
        &self,
        project_ast: &ProjectAst,
        function_elements: &HashMap<String, ElementInfo>,
    ) -> Result<Vec<CallChainInfo>> {
        let mut call_chains = Vec::new();

        for cross_ref in &project_ast.cross_references {
            if matches!(cross_ref.reference_type, ReferenceType::FunctionCall) {
                if let Some(caller) = function_elements.get(&cross_ref.from_element_id) {
                    if let Some(ref to_id) = cross_ref.to_element_id {
                        if let Some(callee) = function_elements.get(to_id) {
                            call_chains.push(CallChainInfo {
                                caller_id: caller.id.clone(),
                                callee_id: callee.id.clone(),
                                call_type: self.determine_call_type(&cross_ref.reference_text),
                                call_sites: vec![cross_ref.location.line_start as i32],
                                context: cross_ref.context.scope.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Aggregate multiple calls between same functions
        self.aggregate_call_chains(call_chains)
    }

    /// Analyze dependency relationships from cross-references.
    fn analyze_dependencies(
        &self,
        project_ast: &ProjectAst,
        all_elements: &HashMap<String, ElementInfo>,
    ) -> Result<Vec<DependencyInfo>> {
        let mut dependencies = Vec::new();

        for cross_ref in &project_ast.cross_references {
            if matches!(cross_ref.reference_type, 
                ReferenceType::TypeUsage | 
                ReferenceType::VariableAccess | 
                ReferenceType::ModuleReference
            ) {
                if let Some(from_elem) = all_elements.get(&cross_ref.from_element_id) {
                    if let Some(ref to_id) = cross_ref.to_element_id {
                        if let Some(to_elem) = all_elements.get(to_id) {
                            dependencies.push(DependencyInfo {
                                from_element_id: from_elem.id.clone(),
                                to_element_id: to_elem.id.clone(),
                                dependency_type: format!("{:?}", cross_ref.reference_type),
                                strength: self.calculate_dependency_strength(&cross_ref.reference_type),
                                is_direct: !matches!(cross_ref.reference_type, ReferenceType::ModuleReference),
                            });
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze type relationships (inheritance, implementations).
    fn analyze_type_relationships(
        &self,
        project_ast: &ProjectAst,
        type_elements: &HashMap<String, ElementInfo>,
    ) -> Result<Vec<TypeRelationshipInfo>> {
        let mut relationships = Vec::new();

        for cross_ref in &project_ast.cross_references {
            if matches!(cross_ref.reference_type, ReferenceType::TraitImplementation | ReferenceType::TypeUsage) {
                if let Some(from_type) = type_elements.get(&cross_ref.from_element_id) {
                    if let Some(ref to_id) = cross_ref.to_element_id {
                        if let Some(to_type) = type_elements.get(to_id) {
                            let relationship_type = match cross_ref.reference_type {
                                ReferenceType::TraitImplementation => "implements",
                                ReferenceType::TypeUsage => "uses",
                                _ => "references",
                            };

                            relationships.push(TypeRelationshipInfo {
                                from_type_id: from_type.id.clone(),
                                to_type_id: to_type.id.clone(),
                                relationship_type: relationship_type.to_string(),
                                relationship_strength: self.calculate_relationship_strength(relationship_type),
                                is_generic: cross_ref.reference_text.contains('<'),
                                generic_constraints: self.extract_generic_constraints(&cross_ref.reference_text),
                            });
                        }
                    }
                }
            }
        }

        Ok(relationships)
    }

    /// Aggregate call chains between same functions.
    fn aggregate_call_chains(&self, call_chains: Vec<CallChainInfo>) -> Result<Vec<CallChainInfo>> {
        let mut aggregated: HashMap<(String, String), CallChainInfo> = HashMap::new();

        for chain in call_chains {
            let key = (chain.caller_id.clone(), chain.callee_id.clone());
            
            if let Some(existing) = aggregated.get_mut(&key) {
                existing.call_sites.extend(chain.call_sites);
            } else {
                aggregated.insert(key, chain);
            }
        }

        Ok(aggregated.into_values().collect())
    }

    /// Determine call type from reference text.
    fn determine_call_type(&self, reference_text: &str) -> String {
        if reference_text.contains("::") {
            "static".to_string()
        } else if reference_text.contains(".") {
            "method".to_string()
        } else {
            "direct".to_string()
        }
    }

    /// Calculate dependency strength based on reference type.
    fn calculate_dependency_strength(&self, ref_type: &ReferenceType) -> f64 {
        match ref_type {
            ReferenceType::TypeUsage => 0.9,
            ReferenceType::VariableAccess => 0.8,
            ReferenceType::FunctionCall => 0.7,
            ReferenceType::ModuleReference => 0.3,
            _ => 0.5,
        }
    }

    /// Calculate type relationship strength.
    fn calculate_relationship_strength(&self, relationship_type: &str) -> f64 {
        match relationship_type {
            "implements" => 0.9,
            "extends" => 0.8,
            "uses" => 0.6,
            "contains" => 0.7,
            _ => 0.5,
        }
    }

    /// Extract generic constraints from reference text.
    fn extract_generic_constraints(&self, reference_text: &str) -> Vec<String> {
        // Simple extraction - could be enhanced with proper parsing
        if let Some(start) = reference_text.find('<') {
            if let Some(end) = reference_text.find('>') {
                let constraints = &reference_text[start+1..end];
                return constraints.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    /// Check if a call is recursive.
    fn is_recursive_call(&self, call_chain: &CallChainInfo, _functions: &HashMap<String, ElementInfo>) -> bool {
        call_chain.caller_id == call_chain.callee_id
    }

    /// Check if adding a dependency would create a cycle.
    async fn would_create_cycle(&self, _dependency: &DependencyInfo, _project_id: Uuid) -> Result<bool> {
        // Simplified cycle detection - could be enhanced with actual graph traversal
        Ok(false)
    }

    /// Store call chain record in database.
    async fn store_call_chain_record(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        call_chain: &CallChainInfo,
        project_id: Uuid,
        is_recursive: bool,
    ) -> Result<()> {
        // First get UUIDs for caller and callee
        let caller_uuid = self.get_element_uuid(tx, project_id, &call_chain.caller_id).await?;
        let callee_uuid = self.get_element_uuid(tx, project_id, &call_chain.callee_id).await?;

        sqlx::query(
            r#"
            INSERT INTO call_chains (
                id, project_id, caller_id, callee_id, call_type, call_count,
                call_sites, is_recursive, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(project_id)
        .bind(caller_uuid)
        .bind(callee_uuid)
        .bind(&call_chain.call_type)
        .bind(call_chain.call_sites.len() as i32)
        .bind(&call_chain.call_sites)
        .bind(is_recursive)
        .bind(serde_json::json!({"context": call_chain.context}))
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
    }

    /// Store dependency record in database.
    async fn store_dependency_record(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        dependency: &DependencyInfo,
        project_id: Uuid,
        is_cyclic: bool,
    ) -> Result<()> {
        let from_uuid = self.get_element_uuid(tx, project_id, &dependency.from_element_id).await?;
        let to_uuid = self.get_element_uuid(tx, project_id, &dependency.to_element_id).await?;

        sqlx::query(
            r#"
            INSERT INTO dependencies (
                id, project_id, from_element_id, to_element_id, dependency_type,
                strength, is_direct, is_cyclic, path_length, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(project_id)
        .bind(from_uuid)
        .bind(to_uuid)
        .bind(&dependency.dependency_type)
        .bind(dependency.strength)
        .bind(dependency.is_direct)
        .bind(is_cyclic)
        .bind(1_i32) // Direct dependency has path length 1
        .bind(serde_json::json!({}))
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
    }

    /// Store type relationship in database.
    async fn store_type_relationship(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        relationship: &TypeRelationshipInfo,
        project_id: Uuid,
    ) -> Result<()> {
        let from_uuid = self.get_element_uuid(tx, project_id, &relationship.from_type_id).await?;
        let to_uuid = self.get_element_uuid(tx, project_id, &relationship.to_type_id).await?;

        sqlx::query(
            r#"
            INSERT INTO type_relationships (
                id, project_id, from_type_id, to_type_id, relationship_type,
                relationship_strength, is_generic, generic_constraints, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(project_id)
        .bind(from_uuid)
        .bind(to_uuid)
        .bind(&relationship.relationship_type)
        .bind(relationship.relationship_strength)
        .bind(relationship.is_generic)
        .bind(&relationship.generic_constraints)
        .bind(serde_json::json!({}))
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
    }

    /// Get element UUID by element ID string.
    async fn get_element_uuid(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        project_id: Uuid,
        element_id: &str,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            "SELECT id FROM ast_elements WHERE project_id = $1 AND element_id = $2"
        )
        .bind(project_id)
        .bind(element_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row.get("id"))
    }

    /// Get project ID by name and version.
    async fn get_project_id_by_name(&self, name: &str, version: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query(
            "SELECT id FROM projects WHERE name = $1 AND version = $2"
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row.map(|r| r.get("id")))
    }

    /// Calculate maximum call depth.
    async fn calculate_max_call_depth(&self, project_id: Uuid) -> Result<i32> {
        let row = sqlx::query(
            r#"
            WITH RECURSIVE call_depth AS (
                SELECT caller_id, callee_id, 1 as depth
                FROM call_chains 
                WHERE project_id = $1
                
                UNION ALL
                
                SELECT cc.caller_id, cc.callee_id, cd.depth + 1
                FROM call_chains cc
                JOIN call_depth cd ON cc.caller_id = cd.callee_id
                WHERE cd.depth < 100  -- Prevent infinite recursion
            )
            SELECT COALESCE(MAX(depth), 0) as max_depth FROM call_depth
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row.get("max_depth"))
    }

    /// Calculate dependency depth.
    async fn calculate_dependency_depth(&self, project_id: Uuid) -> Result<i32> {
        let row = sqlx::query(
            r#"
            WITH RECURSIVE dep_depth AS (
                SELECT from_element_id, to_element_id, 1 as depth
                FROM dependencies 
                WHERE project_id = $1 AND is_direct = true
                
                UNION ALL
                
                SELECT d.from_element_id, d.to_element_id, dd.depth + 1
                FROM dependencies d
                JOIN dep_depth dd ON d.from_element_id = dd.to_element_id
                WHERE dd.depth < 100
            )
            SELECT COALESCE(MAX(depth), 0) as max_depth FROM dep_depth
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row.get("max_depth"))
    }

    /// Calculate inheritance depth.
    async fn calculate_inheritance_depth(&self, project_id: Uuid) -> Result<i32> {
        let row = sqlx::query(
            r#"
            WITH RECURSIVE inheritance_depth AS (
                SELECT from_type_id, to_type_id, 1 as depth
                FROM type_relationships 
                WHERE project_id = $1 AND relationship_type = 'implements'
                
                UNION ALL
                
                SELECT tr.from_type_id, tr.to_type_id, id.depth + 1
                FROM type_relationships tr
                JOIN inheritance_depth id ON tr.from_type_id = id.to_type_id
                WHERE id.depth < 50
            )
            SELECT COALESCE(MAX(depth), 0) as max_depth FROM inheritance_depth
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row.get("max_depth"))
    }
}

/// Element information for graph building
#[derive(Debug, Clone)]
struct ElementInfo {
    id: String,
    name: String,
    qualified_name: String,
    file_path: String,
    element_type: ElementType,
}

/// Call chain information
#[derive(Debug, Clone)]
struct CallChainInfo {
    caller_id: String,
    callee_id: String,
    call_type: String,
    call_sites: Vec<i32>,
    context: String,
}

/// Dependency information
#[derive(Debug, Clone)]
struct DependencyInfo {
    from_element_id: String,
    to_element_id: String,
    dependency_type: String,
    strength: f64,
    is_direct: bool,
}

/// Type relationship information
#[derive(Debug, Clone)]
struct TypeRelationshipInfo {
    from_type_id: String,
    to_type_id: String,
    relationship_type: String,
    relationship_strength: f64,
    is_generic: bool,
    generic_constraints: Vec<String>,
}

/// Statistics for call chain graph
#[derive(Debug, Clone)]
pub struct CallChainStats {
    pub total_functions: usize,
    pub total_call_chains: usize,
    pub recursive_calls: usize,
    pub max_call_depth: i32,
}

/// Statistics for dependency graph
#[derive(Debug, Clone)]
pub struct DependencyStats {
    pub total_elements: usize,
    pub total_dependencies: usize,
    pub cyclic_dependencies: usize,
    pub dependency_depth: i32,
}

/// Statistics for type relationship graph
#[derive(Debug, Clone)]
pub struct TypeRelationshipStats {
    pub total_types: usize,
    pub total_relationships: usize,
    pub trait_implementations: usize,
    pub inheritance_depth: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_call_type() {
        let graph_storage = GraphStorage::new(PgPool::connect("postgresql://test").unwrap());
        
        assert_eq!(graph_storage.determine_call_type("MyStruct::new()"), "static");
        assert_eq!(graph_storage.determine_call_type("instance.method()"), "method");
        assert_eq!(graph_storage.determine_call_type("function_call()"), "direct");
    }

    #[test]
    fn test_extract_generic_constraints() {
        let graph_storage = GraphStorage::new(PgPool::connect("postgresql://test").unwrap());
        
        let constraints = graph_storage.extract_generic_constraints("Vec<String, Clone>");
        assert_eq!(constraints, vec!["String", "Clone"]);
        
        let no_constraints = graph_storage.extract_generic_constraints("Vec");
        assert!(no_constraints.is_empty());
    }
}