//! Storage layer for AST data persistence and retrieval.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use rustex_core::{ProjectAst, FileAst, CodeElement, CrossReference};
use crate::error::{DatabaseError, Result};
use crate::schema::{ProjectRecord, FileRecord, ElementRecord};

/// Main AST storage interface
pub struct AstStorage {
    pool: PgPool,
}

impl AstStorage {
    /// Create a new AST storage instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Store a complete project AST in the database.
    pub async fn store_project_ast(&self, project_ast: &ProjectAst) -> Result<Uuid> {
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        // Store project
        let project_record = ProjectRecord::from_project_ast(project_ast);
        let project_id = project_record.id;

        sqlx::query(
            r#"
            INSERT INTO projects (
                id, name, version, rust_edition, description, authors, license,
                repository_url, homepage, keywords, categories, readme_path,
                total_files, total_lines, total_functions, total_structs,
                total_enums, total_traits, total_modules, total_impls,
                complexity_average, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            "#
        )
        .bind(project_record.id)
        .bind(&project_record.name)
        .bind(&project_record.version)
        .bind(&project_record.rust_edition)
        .bind(&project_record.description)
        .bind(&project_record.authors)
        .bind(&project_record.license)
        .bind(&project_record.repository_url)
        .bind(&project_record.homepage)
        .bind(&project_record.keywords)
        .bind(&project_record.categories)
        .bind(&project_record.readme_path)
        .bind(project_record.total_files)
        .bind(project_record.total_lines)
        .bind(project_record.total_functions)
        .bind(project_record.total_structs)
        .bind(project_record.total_enums)
        .bind(project_record.total_traits)
        .bind(project_record.total_modules)
        .bind(project_record.total_impls)
        .bind(project_record.complexity_average)
        .bind(&project_record.metadata)
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::from)?;

        // Store files and elements
        for file_ast in &project_ast.files {
            let file_id = self.store_file_ast(&mut tx, file_ast, project_id).await?;
            
            // Store elements for this file
            for element in &file_ast.elements {
                self.store_element(&mut tx, element, project_id, file_id).await?;
            }
        }

        // Store cross-references
        for cross_ref in &project_ast.cross_references {
            self.store_cross_reference(&mut tx, cross_ref, project_id).await?;
        }

        tx.commit().await
            .map_err(DatabaseError::from)?;

        Ok(project_id)
    }

    /// Store a file AST record.
    async fn store_file_ast(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        file_ast: &FileAst,
        project_id: Uuid,
    ) -> Result<Uuid> {
        let file_record = FileRecord::from_file_ast(file_ast, project_id);
        let file_id = file_record.id;

        sqlx::query(
            r#"
            INSERT INTO files (
                id, project_id, path, relative_path, file_name, extension,
                lines_of_code, function_count, struct_count, enum_count,
                trait_count, module_count, impl_count, use_count,
                macro_count, const_count, static_count, type_alias_count,
                complexity_total, complexity_average, documentation_coverage,
                analyzed_at, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            )
            "#
        )
        .bind(file_record.id)
        .bind(file_record.project_id)
        .bind(&file_record.path)
        .bind(&file_record.relative_path)
        .bind(&file_record.file_name)
        .bind(&file_record.extension)
        .bind(file_record.lines_of_code)
        .bind(file_record.function_count)
        .bind(file_record.struct_count)
        .bind(file_record.enum_count)
        .bind(file_record.trait_count)
        .bind(file_record.module_count)
        .bind(file_record.impl_count)
        .bind(file_record.use_count)
        .bind(file_record.macro_count)
        .bind(file_record.const_count)
        .bind(file_record.static_count)
        .bind(file_record.type_alias_count)
        .bind(file_record.complexity_total)
        .bind(file_record.complexity_average)
        .bind(file_record.documentation_coverage)
        .bind(file_record.analyzed_at)
        .bind(&file_record.metadata)
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(file_id)
    }

    /// Store an AST element record.
    async fn store_element(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        element: &CodeElement,
        project_id: Uuid,
        file_id: Uuid,
    ) -> Result<Uuid> {
        let element_record = ElementRecord::from_code_element(element, project_id, file_id);
        let record_id = element_record.id;

        sqlx::query(
            r#"
            INSERT INTO ast_elements (
                id, project_id, file_id, element_id, element_type, name,
                qualified_name, signature, visibility, line_start, line_end,
                char_start, char_end, complexity, cyclomatic_complexity,
                cognitive_complexity, nesting_depth, parameter_count,
                return_count, lines_of_code, halstead_metrics, doc_comments,
                inline_comments, attributes, dependencies, generic_params,
                module_path, nesting_level, is_public, is_test, is_async,
                is_unsafe, is_deprecated, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                $25, $26, $27, $28, $29, $30, $31, $32, $33, $34
            )
            "#
        )
        .bind(element_record.id)
        .bind(element_record.project_id)
        .bind(element_record.file_id)
        .bind(&element_record.element_id)
        .bind(sqlx::types::Json(&element_record.element_type))
        .bind(&element_record.name)
        .bind(&element_record.qualified_name)
        .bind(&element_record.signature)
        .bind(sqlx::types::Json(&element_record.visibility))
        .bind(element_record.line_start)
        .bind(element_record.line_end)
        .bind(element_record.char_start)
        .bind(element_record.char_end)
        .bind(element_record.complexity)
        .bind(element_record.cyclomatic_complexity)
        .bind(element_record.cognitive_complexity)
        .bind(element_record.nesting_depth)
        .bind(element_record.parameter_count)
        .bind(element_record.return_count)
        .bind(element_record.lines_of_code)
        .bind(&element_record.halstead_metrics)
        .bind(&element_record.doc_comments)
        .bind(&element_record.inline_comments)
        .bind(&element_record.attributes)
        .bind(&element_record.dependencies)
        .bind(&element_record.generic_params)
        .bind(&element_record.module_path)
        .bind(element_record.nesting_level)
        .bind(element_record.is_public)
        .bind(element_record.is_test)
        .bind(element_record.is_async)
        .bind(element_record.is_unsafe)
        .bind(element_record.is_deprecated)
        .bind(&element_record.metadata)
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(record_id)
    }

    /// Store a cross-reference record.
    async fn store_cross_reference(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        cross_ref: &CrossReference,
        project_id: Uuid,
    ) -> Result<Uuid> {
        let record_id = Uuid::new_v4();

        // Find element IDs from element_id strings
        let from_element_uuid = self.find_element_uuid_by_id(tx, project_id, &cross_ref.from_element_id).await?;
        let to_element_uuid = if let Some(ref to_id) = cross_ref.to_element_id {
            self.find_element_uuid_by_id(tx, project_id, to_id).await.ok()
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO cross_references (
                id, project_id, from_element_id, to_element_id, reference_type,
                reference_text, line_number, char_position, context_scope,
                is_definition, is_resolved, confidence_score, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
            )
            "#
        )
        .bind(record_id)
        .bind(project_id)
        .bind(from_element_uuid)
        .bind(to_element_uuid)
        .bind(sqlx::types::Json(&format!("{:?}", cross_ref.reference_type)))
        .bind(&cross_ref.reference_text)
        .bind(cross_ref.location.line_start as i32)
        .bind(cross_ref.location.char_start as i32)
        .bind(&cross_ref.context.scope)
        .bind(cross_ref.context.is_definition)
        .bind(cross_ref.is_resolved)
        .bind(1.0_f64) // Default confidence score
        .bind(serde_json::json!({}))
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::from)?;

        Ok(record_id)
    }

    /// Find element UUID by original element ID.
    async fn find_element_uuid_by_id(
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

    /// Retrieve a project AST by ID.
    pub async fn get_project_ast(&self, _project_id: Uuid) -> Result<ProjectAst> {
        // This is a complex reconstruction that would need to be implemented
        // For now, return an error indicating it's not yet implemented
        Err(DatabaseError::generic("Project AST reconstruction not yet implemented"))
    }

    /// Delete a project and all its data.
    pub async fn delete_project(&self, project_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(project_id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }

    /// Get project statistics.
    pub async fn get_project_stats(&self, project_id: Uuid) -> Result<ProjectStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                total_files, total_lines, total_functions, total_structs,
                total_enums, total_traits, total_modules, total_impls,
                complexity_average
            FROM projects WHERE id = $1
            "#
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(ProjectStats {
            total_files: row.get("total_files"),
            total_lines: row.get("total_lines"),
            total_functions: row.get("total_functions"),
            total_structs: row.get("total_structs"),
            total_enums: row.get("total_enums"),
            total_traits: row.get("total_traits"),
            total_modules: row.get("total_modules"),
            total_impls: row.get("total_impls"),
            complexity_average: row.get("complexity_average"),
        })
    }
}

/// Project storage operations
pub struct ProjectStorage {
    pool: PgPool,
}

impl ProjectStorage {
    /// Create a new project storage instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List all projects.
    pub async fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let rows = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(rows)
    }

    /// Find a project by name and version.
    pub async fn find_project(&self, name: &str, version: &str) -> Result<Option<ProjectRecord>> {
        let row = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects WHERE name = $1 AND version = $2"
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row)
    }

    /// Get a project by ID.
    pub async fn get_project(&self, project_id: Uuid) -> Result<ProjectRecord> {
        let row = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects WHERE id = $1"
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(row)
    }
}

/// Element storage operations
pub struct ElementStorage {
    pool: PgPool,
}

impl ElementStorage {
    /// Create a new element storage instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find elements by name pattern.
    pub async fn find_elements_by_name(&self, project_id: Uuid, pattern: &str) -> Result<Vec<ElementRecord>> {
        let rows = sqlx::query_as::<_, ElementRecord>(
            "SELECT * FROM ast_elements WHERE project_id = $1 AND name ILIKE $2"
        )
        .bind(project_id)
        .bind(format!("%{}%", pattern))
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(rows)
    }

    /// Get elements by type.
    pub async fn get_elements_by_type(&self, project_id: Uuid, element_type: &str) -> Result<Vec<ElementRecord>> {
        let rows = sqlx::query_as::<_, ElementRecord>(
            "SELECT * FROM ast_elements WHERE project_id = $1 AND element_type = $2"
        )
        .bind(project_id)
        .bind(element_type)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(rows)
    }

    /// Get public elements for a project.
    pub async fn get_public_elements(&self, project_id: Uuid) -> Result<Vec<ElementRecord>> {
        let rows = sqlx::query_as::<_, ElementRecord>(
            "SELECT * FROM ast_elements WHERE project_id = $1 AND is_public = TRUE"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(rows)
    }

    /// Get high complexity elements.
    pub async fn get_high_complexity_elements(&self, project_id: Uuid, threshold: i32) -> Result<Vec<ElementRecord>> {
        let rows = sqlx::query_as::<_, ElementRecord>(
            "SELECT * FROM ast_elements WHERE project_id = $1 AND complexity > $2 ORDER BY complexity DESC"
        )
        .bind(project_id)
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(rows)
    }
}

/// Project statistics
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_files: i32,
    pub total_lines: i64,
    pub total_functions: i32,
    pub total_structs: i32,
    pub total_enums: i32,
    pub total_traits: i32,
    pub total_modules: i32,
    pub total_impls: i32,
    pub complexity_average: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_stats() {
        let stats = ProjectStats {
            total_files: 10,
            total_lines: 5000,
            total_functions: 100,
            total_structs: 25,
            total_enums: 10,
            total_traits: 15,
            total_modules: 8,
            total_impls: 30,
            complexity_average: 3.5,
        };

        assert_eq!(stats.total_files, 10);
        assert_eq!(stats.total_functions, 100);
        assert_eq!(stats.complexity_average, 3.5);
    }
}