//! Query builder and optimization for graph operations.

use sqlx::{PgPool, Row, QueryBuilder};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::error::{DatabaseError, Result};
use crate::schema::ElementRecord;

/// Query builder for complex graph queries
pub struct GraphQueryBuilder {
    pool: PgPool,
}

impl GraphQueryBuilder {
    /// Create a new query builder.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new graph query.
    pub fn query(&self) -> GraphQuery {
        GraphQuery::new(self.pool.clone())
    }
}

/// Graph query with fluent interface
pub struct GraphQuery {
    pool: PgPool,
    project_filter: Option<Uuid>,
    element_types: Vec<String>,
    visibility_filter: Option<String>,
    complexity_range: Option<(i32, i32)>,
    name_pattern: Option<String>,
    depth_limit: Option<i32>,
    include_metrics: bool,
    include_relationships: bool,
    order_by: Vec<String>,
    limit: Option<i64>,
}

impl GraphQuery {
    /// Create a new graph query.
    fn new(pool: PgPool) -> Self {
        Self {
            pool,
            project_filter: None,
            element_types: Vec::new(),
            visibility_filter: None,
            complexity_range: None,
            name_pattern: None,
            depth_limit: None,
            include_metrics: false,
            include_relationships: false,
            order_by: Vec::new(),
            limit: None,
        }
    }

    /// Filter by project ID.
    pub fn project(mut self, project_id: Uuid) -> Self {
        self.project_filter = Some(project_id);
        self
    }

    /// Filter by element types.
    pub fn element_types(mut self, types: Vec<&str>) -> Self {
        self.element_types = types.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter by visibility.
    pub fn visibility(mut self, visibility: &str) -> Self {
        self.visibility_filter = Some(visibility.to_string());
        self
    }

    /// Filter by complexity range.
    pub fn complexity_range(mut self, min: i32, max: i32) -> Self {
        self.complexity_range = Some((min, max));
        self
    }

    /// Filter by name pattern.
    pub fn name_like(mut self, pattern: &str) -> Self {
        self.name_pattern = Some(pattern.to_string());
        self
    }

    /// Set traversal depth limit.
    pub fn max_depth(mut self, depth: i32) -> Self {
        self.depth_limit = Some(depth);
        self
    }

    /// Include complexity metrics in results.
    pub fn with_metrics(mut self) -> Self {
        self.include_metrics = true;
        self
    }

    /// Include relationship data.
    pub fn with_relationships(mut self) -> Self {
        self.include_relationships = true;
        self
    }

    /// Add ordering.
    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", column, direction));
        self
    }

    /// Set result limit.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the query and return elements.
    pub async fn fetch_elements(self) -> Result<Vec<ElementRecord>> {
        let mut query_builder = QueryBuilder::new(
            "SELECT e.* FROM ast_elements e"
        );

        // Add joins if needed
        if self.include_relationships {
            query_builder.push(" LEFT JOIN dependencies d ON e.id = d.from_element_id");
        }

        query_builder.push(" WHERE 1=1");

        // Add filters
        if let Some(project_id) = self.project_filter {
            query_builder.push(" AND e.project_id = ");
            query_builder.push_bind(project_id);
        }

        if !self.element_types.is_empty() {
            query_builder.push(" AND e.element_type = ANY(");
            query_builder.push_bind(self.element_types);
            query_builder.push(")");
        }

        if let Some(visibility) = self.visibility_filter {
            query_builder.push(" AND e.visibility = ");
            query_builder.push_bind(visibility);
        }

        if let Some((min, max)) = self.complexity_range {
            query_builder.push(" AND e.complexity BETWEEN ");
            query_builder.push_bind(min);
            query_builder.push(" AND ");
            query_builder.push_bind(max);
        }

        if let Some(pattern) = self.name_pattern {
            query_builder.push(" AND e.name ILIKE ");
            query_builder.push_bind(format!("%{}%", pattern));
        }

        // Add ordering
        if !self.order_by.is_empty() {
            query_builder.push(" ORDER BY ");
            query_builder.push(self.order_by.join(", "));
        }

        // Add limit
        if let Some(limit) = self.limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }

        let query = query_builder.build_query_as::<ElementRecord>();
        let elements = query.fetch_all(&self.pool).await
            .map_err(DatabaseError::from)?;

        Ok(elements)
    }

    /// Execute a dependency traversal query.
    pub async fn traverse_dependencies(self, start_element: Uuid) -> Result<Vec<DependencyNode>> {
        let max_depth = self.depth_limit.unwrap_or(10);
        
        let query = format!(
            r#"
            WITH RECURSIVE dependency_tree AS (
                -- Base case: start element
                SELECT 
                    e.id, e.name, e.element_type, e.qualified_name,
                    0 as depth,
                    ARRAY[e.id] as path,
                    NULL::UUID as parent_id,
                    'root'::TEXT as dependency_type
                FROM ast_elements e
                WHERE e.id = $1
                
                UNION ALL
                
                -- Recursive case: follow dependencies
                SELECT 
                    e.id, e.name, e.element_type, e.qualified_name,
                    dt.depth + 1,
                    dt.path || e.id,
                    dt.id as parent_id,
                    d.dependency_type::TEXT
                FROM dependency_tree dt
                JOIN dependencies d ON dt.id = d.from_element_id
                JOIN ast_elements e ON d.to_element_id = e.id
                WHERE 
                    dt.depth < $2 
                    AND NOT (e.id = ANY(dt.path))  -- Avoid cycles
                    {}
            )
            SELECT * FROM dependency_tree
            ORDER BY depth, name
            "#,
            self.build_additional_filters()
        );

        let rows = sqlx::query(&query)
            .bind(start_element)
            .bind(max_depth)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(DependencyNode {
                id: row.get("id"),
                name: row.get("name"),
                element_type: row.get("element_type"),
                qualified_name: row.get("qualified_name"),
                depth: row.get("depth"),
                parent_id: row.get("parent_id"),
                dependency_type: row.get("dependency_type"),
                path: row.get::<Vec<Uuid>, _>("path"),
            });
        }

        Ok(nodes)
    }

    /// Execute a call chain traversal query.
    pub async fn traverse_call_chain(self, start_function: Uuid, direction: TraversalDirection) -> Result<Vec<CallNode>> {
        let max_depth = self.depth_limit.unwrap_or(10);
        
        let (join_condition, direction_filter) = match direction {
            TraversalDirection::Forward => ("cc.caller_id = ct.id", "cc.callee_id = e.id"),
            TraversalDirection::Backward => ("cc.callee_id = ct.id", "cc.caller_id = e.id"),
        };

        let query = format!(
            r#"
            WITH RECURSIVE call_tree AS (
                -- Base case: start function
                SELECT 
                    e.id, e.name, e.element_type, e.qualified_name,
                    0 as depth,
                    ARRAY[e.id] as path,
                    NULL::UUID as parent_id,
                    'root'::TEXT as call_type,
                    0 as call_count
                FROM ast_elements e
                WHERE e.id = $1 AND e.element_type = 'Function'
                
                UNION ALL
                
                -- Recursive case: follow call chains
                SELECT 
                    e.id, e.name, e.element_type, e.qualified_name,
                    ct.depth + 1,
                    ct.path || e.id,
                    ct.id as parent_id,
                    cc.call_type::TEXT,
                    cc.call_count
                FROM call_tree ct
                JOIN call_chains cc ON {}
                JOIN ast_elements e ON {}
                WHERE 
                    ct.depth < $2 
                    AND NOT (e.id = ANY(ct.path))  -- Avoid cycles
                    AND e.element_type = 'Function'
                    {}
            )
            SELECT * FROM call_tree
            ORDER BY depth, call_count DESC, name
            "#,
            join_condition,
            direction_filter,
            self.build_additional_filters()
        );

        let rows = sqlx::query(&query)
            .bind(start_function)
            .bind(max_depth)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(CallNode {
                id: row.get("id"),
                name: row.get("name"),
                element_type: row.get("element_type"),
                qualified_name: row.get("qualified_name"),
                depth: row.get("depth"),
                parent_id: row.get("parent_id"),
                call_type: row.get("call_type"),
                call_count: row.get("call_count"),
                path: row.get::<Vec<Uuid>, _>("path"),
            });
        }

        Ok(nodes)
    }

    /// Build additional filter conditions for recursive queries.
    fn build_additional_filters(&self) -> String {
        let mut filters = Vec::new();

        if let Some(project_id) = self.project_filter {
            filters.push(format!("AND e.project_id = '{}'", project_id));
        }

        if let Some(visibility) = &self.visibility_filter {
            filters.push(format!("AND e.visibility = '{}'", visibility));
        }

        if let Some((min, max)) = self.complexity_range {
            filters.push(format!("AND e.complexity BETWEEN {} AND {}", min, max));
        }

        filters.join(" ")
    }

    /// Execute a full-text search across elements.
    pub async fn search(self, search_term: &str) -> Result<Vec<SearchResult>> {
        let query = r#"
            SELECT 
                e.id, e.name, e.element_type, e.qualified_name,
                e.signature, e.doc_comments, e.complexity,
                ts_rank(
                    to_tsvector('english', 
                        COALESCE(e.name, '') || ' ' || 
                        COALESCE(e.qualified_name, '') || ' ' || 
                        COALESCE(e.signature, '') || ' ' ||
                        COALESCE(array_to_string(e.doc_comments, ' '), '')
                    ),
                    plainto_tsquery('english', $1)
                ) as rank
            FROM ast_elements e
            WHERE 
                to_tsvector('english', 
                    COALESCE(e.name, '') || ' ' || 
                    COALESCE(e.qualified_name, '') || ' ' || 
                    COALESCE(e.signature, '') || ' ' ||
                    COALESCE(array_to_string(e.doc_comments, ' '), '')
                ) @@ plainto_tsquery('english', $1)
                {}
            ORDER BY rank DESC, e.name
            {}
        "#;

        let mut project_filter = String::new();
        if let Some(project_id) = self.project_filter {
            project_filter = format!("AND e.project_id = '{}'", project_id);
        }

        let limit_clause = if let Some(limit) = self.limit {
            format!("LIMIT {}", limit)
        } else {
            String::new()
        };

        let final_query = query.replace("{}", &project_filter).replace("{}", &limit_clause);

        let rows = sqlx::query(&final_query)
            .bind(search_term)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(SearchResult {
                id: row.get("id"),
                name: row.get("name"),
                element_type: row.get("element_type"),
                qualified_name: row.get("qualified_name"),
                signature: row.get("signature"),
                doc_comments: row.get("doc_comments"),
                complexity: row.get("complexity"),
                rank: row.get("rank"),
            });
        }

        Ok(results)
    }
}

/// Query builder alias for backwards compatibility
pub type RustexQueryBuilder = GraphQueryBuilder;

/// Traversal direction for call chains
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TraversalDirection {
    Forward,  // What this function calls
    Backward, // What calls this function
}

/// Node in a dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: Uuid,
    pub name: String,
    pub element_type: String,
    pub qualified_name: String,
    pub depth: i32,
    pub parent_id: Option<Uuid>,
    pub dependency_type: String,
    pub path: Vec<Uuid>,
}

/// Node in a call tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallNode {
    pub id: Uuid,
    pub name: String,
    pub element_type: String,
    pub qualified_name: String,
    pub depth: i32,
    pub parent_id: Option<Uuid>,
    pub call_type: String,
    pub call_count: i32,
    pub path: Vec<Uuid>,
}

/// Search result with ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub name: String,
    pub element_type: String,
    pub qualified_name: String,
    pub signature: Option<String>,
    pub doc_comments: Vec<String>,
    pub complexity: Option<i32>,
    pub rank: f32,
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub execution_time_ms: u64,
    pub rows_examined: u64,
    pub rows_returned: u64,
    pub index_usage: Vec<String>,
    pub cache_hit_ratio: f64,
}

/// Query optimizer for analyzing and improving query performance
pub struct QueryOptimizer {
    pool: PgPool,
}

impl QueryOptimizer {
    /// Create a new query optimizer.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Analyze query performance.
    pub async fn analyze_query(&self, query: &str) -> Result<QueryMetrics> {
        let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", query);
        
        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let explain_result: serde_json::Value = row.get(0);
        
        // Parse the explain result to extract metrics
        let plan = &explain_result[0]["Plan"];
        let execution_time = explain_result[0]["Execution Time"].as_f64().unwrap_or(0.0) as u64;
        let rows_returned = plan["Actual Rows"].as_u64().unwrap_or(0);

        Ok(QueryMetrics {
            execution_time_ms: execution_time,
            rows_examined: 0, // Would need to parse from explain output
            rows_returned,
            index_usage: Vec::new(), // Would need to parse from explain output
            cache_hit_ratio: 0.0, // Would need to calculate from buffer stats
        })
    }

    /// Get index usage statistics.
    pub async fn get_index_stats(&self) -> Result<Vec<IndexStats>> {
        let query = r#"
            SELECT 
                schemaname, tablename, indexname,
                idx_tup_read, idx_tup_fetch,
                idx_scan
            FROM pg_stat_user_indexes
            WHERE schemaname = 'public'
            ORDER BY idx_scan DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(IndexStats {
                schema_name: row.get("schemaname"),
                table_name: row.get("tablename"),
                index_name: row.get("indexname"),
                tuples_read: row.get("idx_tup_read"),
                tuples_fetched: row.get("idx_tup_fetch"),
                scans: row.get("idx_scan"),
            });
        }

        Ok(stats)
    }

    /// Suggest query optimizations.
    pub async fn suggest_optimizations(&self, query: &str) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();

        // Analyze the query structure
        if query.contains("SELECT *") {
            suggestions.push("Consider selecting only needed columns instead of using SELECT *".to_string());
        }

        if query.contains("LIKE '%") {
            suggestions.push("Consider using full-text search instead of LIKE with leading wildcard".to_string());
        }

        if !query.to_uppercase().contains("LIMIT") {
            suggestions.push("Consider adding a LIMIT clause for large result sets".to_string());
        }

        Ok(suggestions)
    }
}

/// Index usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub schema_name: String,
    pub table_name: String,
    pub index_name: String,
    pub tuples_read: i64,
    pub tuples_fetched: i64,
    pub scans: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_builder() {
        // Mock pool for testing
        // In real tests, this would use a test database
        let pool = PgPool::connect("postgresql://test").await.expect("Test pool");
        let builder = GraphQueryBuilder::new(pool);
        
        let query = builder.query()
            .element_types(vec!["Function", "Struct"])
            .complexity_range(1, 10)
            .name_like("test")
            .limit(100);

        assert!(query.element_types.contains(&"Function".to_string()));
        assert_eq!(query.complexity_range, Some((1, 10)));
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_traversal_direction() {
        let forward = TraversalDirection::Forward;
        let backward = TraversalDirection::Backward;
        
        // Test serialization
        let forward_json = serde_json::to_string(&forward).unwrap();
        let backward_json = serde_json::to_string(&backward).unwrap();
        
        assert!(forward_json.contains("Forward"));
        assert!(backward_json.contains("Backward"));
    }
}