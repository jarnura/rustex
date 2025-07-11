//! Graph traversal and analysis for call chains and dependencies.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::error::{DatabaseError, Result};
use crate::schema::{CallChainRecord, DependencyRecord};

/// Graph traversal operations
pub struct GraphTraversal {
    pool: PgPool,
}

impl GraphTraversal {
    /// Create a new graph traversal instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all dependencies of an element (forward traversal).
    pub async fn find_dependencies(&self, element_id: Uuid, max_depth: Option<i32>) -> Result<Vec<DependencyPath>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut paths = Vec::new();

        queue.push_back(DependencyPath {
            target_id: element_id,
            path: vec![element_id],
            depth: 0,
            dependency_type: "root".to_string(),
        });

        while let Some(current_path) = queue.pop_front() {
            if let Some(max_d) = max_depth {
                if current_path.depth >= max_d {
                    continue;
                }
            }

            if !visited.insert(current_path.target_id) {
                continue;
            }

            // Find direct dependencies
            let deps = self.get_direct_dependencies(current_path.target_id).await?;
            
            for dep in deps {
                if !current_path.path.contains(&dep.to_element_id) {
                    let mut new_path = current_path.path.clone();
                    new_path.push(dep.to_element_id);
                    
                    let dep_path = DependencyPath {
                        target_id: dep.to_element_id,
                        path: new_path,
                        depth: current_path.depth + 1,
                        dependency_type: dep.dependency_type,
                    };
                    
                    paths.push(dep_path.clone());
                    queue.push_back(dep_path);
                }
            }
        }

        Ok(paths)
    }

    /// Find all dependents of an element (backward traversal).
    pub async fn find_dependents(&self, element_id: Uuid, max_depth: Option<i32>) -> Result<Vec<DependencyPath>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut paths = Vec::new();

        queue.push_back(DependencyPath {
            target_id: element_id,
            path: vec![element_id],
            depth: 0,
            dependency_type: "root".to_string(),
        });

        while let Some(current_path) = queue.pop_front() {
            if let Some(max_d) = max_depth {
                if current_path.depth >= max_d {
                    continue;
                }
            }

            if !visited.insert(current_path.target_id) {
                continue;
            }

            // Find elements that depend on the current element
            let dependents = self.get_direct_dependents(current_path.target_id).await?;
            
            for dep in dependents {
                if !current_path.path.contains(&dep.from_element_id) {
                    let mut new_path = current_path.path.clone();
                    new_path.push(dep.from_element_id);
                    
                    let dep_path = DependencyPath {
                        target_id: dep.from_element_id,
                        path: new_path,
                        depth: current_path.depth + 1,
                        dependency_type: dep.dependency_type,
                    };
                    
                    paths.push(dep_path.clone());
                    queue.push_back(dep_path);
                }
            }
        }

        Ok(paths)
    }

    /// Detect dependency cycles in a project.
    pub async fn detect_cycles(&self, project_id: Uuid) -> Result<Vec<DependencyCycle>> {
        let all_deps = self.get_all_dependencies(project_id).await?;
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        // Build adjacency list
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for dep in &all_deps {
            graph.entry(dep.from_element_id)
                .or_insert_with(Vec::new)
                .push(dep.to_element_id);
        }

        // DFS to detect cycles
        for &node in graph.keys() {
            if !visited.contains(&node) {
                if let Some(cycle) = self.dfs_detect_cycle(
                    node, 
                    &graph, 
                    &mut visited, 
                    &mut rec_stack,
                    &mut Vec::new()
                ) {
                    cycles.push(DependencyCycle {
                        elements: cycle,
                        cycle_type: "Dependency".to_string(),
                        strength: 1.0,
                    });
                }
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection.
    fn dfs_detect_cycle(
        &self,
        node: Uuid,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
        path: &mut Vec<Uuid>,
    ) -> Option<Vec<Uuid>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) = self.dfs_detect_cycle(neighbor, graph, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&x| x == neighbor).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(&node);
        path.pop();
        None
    }

    /// Get direct dependencies of an element.
    async fn get_direct_dependencies(&self, element_id: Uuid) -> Result<Vec<DependencyRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM dependencies WHERE from_element_id = $1 AND is_direct = TRUE"
        )
        .bind(element_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut deps = Vec::new();
        for row in rows {
            deps.push(DependencyRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                from_element_id: row.get("from_element_id"),
                to_element_id: row.get("to_element_id"),
                dependency_type: row.get("dependency_type"),
                strength: row.get("strength"),
                is_direct: row.get("is_direct"),
                is_cyclic: row.get("is_cyclic"),
                path_length: row.get("path_length"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(deps)
    }

    /// Get direct dependents of an element.
    async fn get_direct_dependents(&self, element_id: Uuid) -> Result<Vec<DependencyRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM dependencies WHERE to_element_id = $1 AND is_direct = TRUE"
        )
        .bind(element_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut deps = Vec::new();
        for row in rows {
            deps.push(DependencyRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                from_element_id: row.get("from_element_id"),
                to_element_id: row.get("to_element_id"),
                dependency_type: row.get("dependency_type"),
                strength: row.get("strength"),
                is_direct: row.get("is_direct"),
                is_cyclic: row.get("is_cyclic"),
                path_length: row.get("path_length"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(deps)
    }

    /// Get all dependencies for a project.
    async fn get_all_dependencies(&self, project_id: Uuid) -> Result<Vec<DependencyRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM dependencies WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut deps = Vec::new();
        for row in rows {
            deps.push(DependencyRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                from_element_id: row.get("from_element_id"),
                to_element_id: row.get("to_element_id"),
                dependency_type: row.get("dependency_type"),
                strength: row.get("strength"),
                is_direct: row.get("is_direct"),
                is_cyclic: row.get("is_cyclic"),
                path_length: row.get("path_length"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(deps)
    }
}

/// Call chain analyzer for function call relationships
pub struct CallChainAnalyzer {
    pool: PgPool,
}

impl CallChainAnalyzer {
    /// Create a new call chain analyzer.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Analyze call chains starting from a function.
    pub async fn analyze_call_chain(&self, function_id: Uuid, max_depth: Option<i32>) -> Result<CallChainAnalysis> {
        let forward_calls = self.trace_calls_forward(function_id, max_depth).await?;
        let backward_calls = self.trace_calls_backward(function_id, max_depth).await?;
        let recursive_calls = self.find_recursive_calls(function_id).await?;

        let max_depth = forward_calls.iter().map(|c| c.depth).max().unwrap_or(0);
        let total_calls = forward_calls.len() + backward_calls.len();
        
        Ok(CallChainAnalysis {
            root_function_id: function_id,
            forward_calls,
            backward_calls,
            recursive_calls,
            max_depth,
            total_calls,
        })
    }

    /// Trace function calls forward (what this function calls).
    pub async fn trace_calls_forward(&self, function_id: Uuid, max_depth: Option<i32>) -> Result<Vec<CallPath>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut paths = Vec::new();

        queue.push_back(CallPath {
            target_id: function_id,
            path: vec![function_id],
            depth: 0,
            call_type: "root".to_string(),
            call_count: 0,
        });

        while let Some(current_path) = queue.pop_front() {
            if let Some(max_d) = max_depth {
                if current_path.depth >= max_d {
                    continue;
                }
            }

            if !visited.insert(current_path.target_id) {
                continue;
            }

            let calls = self.get_function_calls(current_path.target_id).await?;
            
            for call in calls {
                if !current_path.path.contains(&call.callee_id) {
                    let mut new_path = current_path.path.clone();
                    new_path.push(call.callee_id);
                    
                    let call_path = CallPath {
                        target_id: call.callee_id,
                        path: new_path,
                        depth: current_path.depth + 1,
                        call_type: call.call_type,
                        call_count: call.call_count,
                    };
                    
                    paths.push(call_path.clone());
                    queue.push_back(call_path);
                }
            }
        }

        Ok(paths)
    }

    /// Trace function calls backward (what calls this function).
    pub async fn trace_calls_backward(&self, function_id: Uuid, max_depth: Option<i32>) -> Result<Vec<CallPath>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut paths = Vec::new();

        queue.push_back(CallPath {
            target_id: function_id,
            path: vec![function_id],
            depth: 0,
            call_type: "root".to_string(),
            call_count: 0,
        });

        while let Some(current_path) = queue.pop_front() {
            if let Some(max_d) = max_depth {
                if current_path.depth >= max_d {
                    continue;
                }
            }

            if !visited.insert(current_path.target_id) {
                continue;
            }

            let callers = self.get_function_callers(current_path.target_id).await?;
            
            for caller in callers {
                if !current_path.path.contains(&caller.caller_id) {
                    let mut new_path = current_path.path.clone();
                    new_path.push(caller.caller_id);
                    
                    let call_path = CallPath {
                        target_id: caller.caller_id,
                        path: new_path,
                        depth: current_path.depth + 1,
                        call_type: caller.call_type,
                        call_count: caller.call_count,
                    };
                    
                    paths.push(call_path.clone());
                    queue.push_back(call_path);
                }
            }
        }

        Ok(paths)
    }

    /// Find recursive calls for a function.
    pub async fn find_recursive_calls(&self, function_id: Uuid) -> Result<Vec<CallChainRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM call_chains WHERE caller_id = $1 AND is_recursive = TRUE"
        )
        .bind(function_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut calls = Vec::new();
        for row in rows {
            calls.push(CallChainRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                caller_id: row.get("caller_id"),
                callee_id: row.get("callee_id"),
                call_type: row.get("call_type"),
                call_count: row.get("call_count"),
                call_sites: row.get("call_sites"),
                is_recursive: row.get("is_recursive"),
                recursion_depth: row.get("recursion_depth"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(calls)
    }

    /// Get direct function calls from a function.
    async fn get_function_calls(&self, function_id: Uuid) -> Result<Vec<CallChainRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM call_chains WHERE caller_id = $1"
        )
        .bind(function_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut calls = Vec::new();
        for row in rows {
            calls.push(CallChainRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                caller_id: row.get("caller_id"),
                callee_id: row.get("callee_id"),
                call_type: row.get("call_type"),
                call_count: row.get("call_count"),
                call_sites: row.get("call_sites"),
                is_recursive: row.get("is_recursive"),
                recursion_depth: row.get("recursion_depth"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(calls)
    }

    /// Get functions that call this function.
    async fn get_function_callers(&self, function_id: Uuid) -> Result<Vec<CallChainRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM call_chains WHERE callee_id = $1"
        )
        .bind(function_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut calls = Vec::new();
        for row in rows {
            calls.push(CallChainRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                caller_id: row.get("caller_id"),
                callee_id: row.get("callee_id"),
                call_type: row.get("call_type"),
                call_count: row.get("call_count"),
                call_sites: row.get("call_sites"),
                is_recursive: row.get("is_recursive"),
                recursion_depth: row.get("recursion_depth"),
                created_at: row.get("created_at"),
                metadata: row.get("metadata"),
            });
        }

        Ok(calls)
    }
}

/// Dependency analyzer for analyzing type and module dependencies
pub struct DependencyAnalyzer {
    pool: PgPool,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Analyze dependencies for a project.
    pub async fn analyze_project_dependencies(&self, project_id: Uuid) -> Result<DependencyAnalysis> {
        let graph_traversal = GraphTraversal::new(self.pool.clone());
        
        let all_deps = graph_traversal.get_all_dependencies(project_id).await?;
        let cycles = graph_traversal.detect_cycles(project_id).await?;
        
        // Calculate dependency metrics
        let total_dependencies = all_deps.len();
        let direct_dependencies = all_deps.iter().filter(|d| d.is_direct).count();
        let cyclic_dependencies = all_deps.iter().filter(|d| d.is_cyclic).count();
        
        // Find most dependent elements
        let mut dependency_counts: HashMap<Uuid, usize> = HashMap::new();
        for dep in &all_deps {
            *dependency_counts.entry(dep.from_element_id).or_insert(0) += 1;
        }
        
        let mut most_dependent: Vec<_> = dependency_counts.into_iter().collect();
        most_dependent.sort_by(|a, b| b.1.cmp(&a.1));
        most_dependent.truncate(10);

        Ok(DependencyAnalysis {
            project_id,
            total_dependencies,
            direct_dependencies,
            cyclic_dependencies,
            dependency_cycles: cycles,
            most_dependent_elements: most_dependent.into_iter().map(|(id, count)| (id, count as i32)).collect(),
            average_dependency_depth: all_deps.iter().map(|d| d.path_length).sum::<i32>() as f64 / total_dependencies as f64,
        })
    }
}

/// Dependency path through the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPath {
    pub target_id: Uuid,
    pub path: Vec<Uuid>,
    pub depth: i32,
    pub dependency_type: String,
}

/// Call path through the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallPath {
    pub target_id: Uuid,
    pub path: Vec<Uuid>,
    pub depth: i32,
    pub call_type: String,
    pub call_count: i32,
}

/// Dependency cycle detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCycle {
    pub elements: Vec<Uuid>,
    pub cycle_type: String,
    pub strength: f64,
}

/// Call chain analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChainAnalysis {
    pub root_function_id: Uuid,
    pub forward_calls: Vec<CallPath>,
    pub backward_calls: Vec<CallPath>,
    pub recursive_calls: Vec<CallChainRecord>,
    pub max_depth: i32,
    pub total_calls: usize,
}

/// Dependency analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub project_id: Uuid,
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub cyclic_dependencies: usize,
    pub dependency_cycles: Vec<DependencyCycle>,
    pub most_dependent_elements: Vec<(Uuid, i32)>,
    pub average_dependency_depth: f64,
}