//! Advanced graph traversal algorithms for dependency and call chain analysis.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::cmp::Reverse;
use serde::{Deserialize, Serialize};
use crate::error::{DatabaseError, Result};

/// Advanced graph traversal engine with BFS/DFS and cycle detection
pub struct GraphTraversalEngine {
    pool: PgPool,
}

impl GraphTraversalEngine {
    /// Create a new graph traversal engine.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Perform breadth-first search from a starting element.
    pub async fn bfs_traversal(&self, start_element: Uuid, traversal_type: TraversalType, max_depth: Option<usize>) -> Result<BfsResult> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = BfsResult::new(start_element);
        
        // Initialize with starting element
        queue.push_back(TraversalNode {
            element_id: start_element,
            depth: 0,
            path: vec![start_element],
            relationship_type: "root".to_string(),
            weight: 0.0,
        });

        while let Some(current) = queue.pop_front() {
            // Check depth limit
            if let Some(max_d) = max_depth {
                if current.depth >= max_d {
                    continue;
                }
            }

            // Skip if already visited
            if !visited.insert(current.element_id) {
                continue;
            }

            // Add to result
            result.add_node(current.clone());

            // Get neighbors based on traversal type
            let neighbors = self.get_neighbors(current.element_id, &traversal_type).await?;
            
            for neighbor in neighbors {
                if !visited.contains(&neighbor.target_id) {
                    let mut new_path = current.path.clone();
                    new_path.push(neighbor.target_id);
                    
                    queue.push_back(TraversalNode {
                        element_id: neighbor.target_id,
                        depth: current.depth + 1,
                        path: new_path,
                        relationship_type: neighbor.relationship_type,
                        weight: neighbor.weight,
                    });
                }
            }
        }

        Ok(result)
    }

    /// Perform depth-first search from a starting element.
    pub async fn dfs_traversal(&self, start_element: Uuid, traversal_type: TraversalType, max_depth: Option<usize>) -> Result<DfsResult> {
        let mut visited = HashSet::new();
        let mut result = DfsResult::new(start_element);
        let mut stack = Vec::new();
        
        // Use iterative DFS to avoid recursion issues
        stack.push((start_element, 0, vec![start_element]));

        while let Some((current_element, depth, path)) = stack.pop() {
            if depth > max_depth.unwrap_or(usize::MAX) {
                continue;
            }

            if !visited.insert(current_element) {
                continue;
            }

            // Add current node to result
            result.add_node(TraversalNode {
                element_id: current_element,
                depth,
                path: path.clone(),
                relationship_type: if depth == 0 { "root".to_string() } else { "dependency".to_string() },
                weight: 0.0,
            });

            // Get neighbors and add to stack
            let neighbors = self.get_neighbors(current_element, &traversal_type).await?;
            
            for neighbor in neighbors {
                if !visited.contains(&neighbor.target_id) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.target_id);
                    stack.push((neighbor.target_id, depth + 1, new_path));
                }
            }
        }

        Ok(result)
    }

    /// Detect all cycles in the dependency graph using iterative approach.
    pub async fn detect_all_cycles(&self, project_id: Uuid, traversal_type: TraversalType) -> Result<CycleDetectionResult> {
        let graph = self.build_adjacency_list(project_id, &traversal_type).await?;
        let mut result = CycleDetectionResult::new();
        let mut global_visited: HashSet<Uuid> = HashSet::new();

        // Simple iterative cycle detection
        for &start_node in graph.keys() {
            if !global_visited.contains(&start_node) {
                if let Some(cycle) = self.find_simple_cycle(start_node, &graph) {
                    result.add_cycle(cycle);
                    global_visited.insert(start_node);
                }
            }
        }

        Ok(result)
    }

    /// Find a simple cycle starting from a node using iterative DFS.
    fn find_simple_cycle(&self, start_node: Uuid, graph: &HashMap<Uuid, Vec<Uuid>>) -> Option<Cycle> {
        let mut visited = HashMap::new();
        let mut stack = Vec::new();
        
        stack.push((start_node, vec![start_node]));
        
        while let Some((current, path)) = stack.pop() {
            if let Some(&first_occurrence) = visited.get(&current) {
                if first_occurrence < path.len() - 1 {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&x| x == current).unwrap_or(0);
                    let cycle_path = path[cycle_start..].to_vec();
                    
                    return Some(Cycle {
                        elements: cycle_path.clone(),
                        cycle_type: CycleType::Simple,
                        length: cycle_path.len(),
                        strength: 1.0 / cycle_path.len() as f64,
                    });
                }
            } else {
                visited.insert(current, path.len() - 1);
                
                if let Some(neighbors) = graph.get(&current) {
                    for &neighbor in neighbors {
                        if path.len() < 20 { // Prevent infinite loops
                            let mut new_path = path.clone();
                            new_path.push(neighbor);
                            stack.push((neighbor, new_path));
                        }
                    }
                }
            }
        }
        
        None
    }


    /// Find shortest path between two elements using Dijkstra's algorithm.
    pub async fn shortest_path(&self, start: Uuid, end: Uuid, traversal_type: TraversalType) -> Result<Option<ShortestPath>> {
        let mut distances: HashMap<Uuid, f64> = HashMap::new();
        let mut previous: HashMap<Uuid, Uuid> = HashMap::new();
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Initialize
        distances.insert(start, 0.0);
        heap.push(Reverse(DijkstraNode { distance: 0.0, element_id: start }));

        while let Some(Reverse(DijkstraNode { distance, element_id })) = heap.pop() {
            if element_id == end {
                // Reconstruct path
                let path = self.reconstruct_path(&previous, start, end)?;
                return Ok(Some(ShortestPath {
                    path,
                    total_distance: distance,
                    edge_count: previous.len(),
                }));
            }

            if visited.contains(&element_id) {
                continue;
            }
            visited.insert(element_id);

            // Get neighbors
            let neighbors = self.get_neighbors(element_id, &traversal_type).await?;
            
            for neighbor in neighbors {
                let new_distance = distance + neighbor.weight;
                
                if new_distance < *distances.get(&neighbor.target_id).unwrap_or(&f64::INFINITY) {
                    distances.insert(neighbor.target_id, new_distance);
                    previous.insert(neighbor.target_id, element_id);
                    heap.push(Reverse(DijkstraNode {
                        distance: new_distance,
                        element_id: neighbor.target_id,
                    }));
                }
            }
        }

        Ok(None) // No path found
    }

    /// Find all paths between two elements up to a maximum depth using iterative approach.
    pub async fn find_all_paths(&self, start: Uuid, end: Uuid, traversal_type: TraversalType, max_depth: usize) -> Result<Vec<Path>> {
        let mut paths = Vec::new();
        let mut stack = Vec::new();
        
        // Stack contains: (current_node, path, depth, visited_in_path)
        stack.push((start, vec![start], 0, HashSet::from([start])));

        while let Some((current, path, depth, visited_in_path)) = stack.pop() {
            if depth > max_depth {
                continue;
            }

            if current == end {
                let total_weight = self.calculate_path_weight(&path).await?;
                let path_length = path.len();
                paths.push(Path {
                    elements: path,
                    length: path_length,
                    total_weight,
                });
                continue;
            }

            let neighbors = self.get_neighbors(current, &traversal_type).await?;
            
            for neighbor in neighbors {
                if !visited_in_path.contains(&neighbor.target_id) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.target_id);
                    
                    let mut new_visited = visited_in_path.clone();
                    new_visited.insert(neighbor.target_id);
                    
                    stack.push((neighbor.target_id, new_path, depth + 1, new_visited));
                }
            }
        }

        Ok(paths)
    }

    /// Get neighbors of an element based on traversal type.
    async fn get_neighbors(&self, element_id: Uuid, traversal_type: &TraversalType) -> Result<Vec<GraphEdge>> {
        let query = match traversal_type {
            TraversalType::Dependencies => {
                "SELECT to_element_id, dependency_type, strength FROM dependencies WHERE from_element_id = $1"
            }
            TraversalType::Dependents => {
                "SELECT from_element_id as to_element_id, dependency_type, strength FROM dependencies WHERE to_element_id = $1"
            }
            TraversalType::CallChains => {
                "SELECT callee_id as to_element_id, call_type as dependency_type, 1.0 as strength FROM call_chains WHERE caller_id = $1"
            }
            TraversalType::CallersChain => {
                "SELECT caller_id as to_element_id, call_type as dependency_type, 1.0 as strength FROM call_chains WHERE callee_id = $1"
            }
            TraversalType::TypeRelationships => {
                "SELECT to_type_id as to_element_id, relationship_type as dependency_type, relationship_strength as strength FROM type_relationships WHERE from_type_id = $1"
            }
        };

        let rows = sqlx::query(query)
            .bind(element_id)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut edges = Vec::new();
        for row in rows {
            edges.push(GraphEdge {
                target_id: row.get("to_element_id"),
                relationship_type: row.get("dependency_type"),
                weight: row.get::<f64, _>("strength"),
            });
        }

        Ok(edges)
    }

    /// Build adjacency list for the entire graph.
    async fn build_adjacency_list(&self, project_id: Uuid, traversal_type: &TraversalType) -> Result<HashMap<Uuid, Vec<Uuid>>> {
        let query = match traversal_type {
            TraversalType::Dependencies | TraversalType::Dependents => {
                "SELECT from_element_id, to_element_id FROM dependencies WHERE project_id = $1"
            }
            TraversalType::CallChains | TraversalType::CallersChain => {
                "SELECT caller_id as from_element_id, callee_id as to_element_id FROM call_chains WHERE project_id = $1"
            }
            TraversalType::TypeRelationships => {
                "SELECT from_type_id as from_element_id, to_type_id as to_element_id FROM type_relationships WHERE project_id = $1"
            }
        };

        let rows = sqlx::query(query)
            .bind(project_id)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        for row in rows {
            let from_id: Uuid = row.get("from_element_id");
            let to_id: Uuid = row.get("to_element_id");
            
            graph.entry(from_id).or_default().push(to_id);
        }

        Ok(graph)
    }

    /// Reconstruct path from Dijkstra's previous map.
    fn reconstruct_path(&self, previous: &HashMap<Uuid, Uuid>, start: Uuid, end: Uuid) -> Result<Vec<Uuid>> {
        let mut path = Vec::new();
        let mut current = end;
        
        while current != start {
            path.push(current);
            current = *previous.get(&current)
                .ok_or_else(|| DatabaseError::generic("Invalid path reconstruction"))?;
        }
        path.push(start);
        path.reverse();
        
        Ok(path)
    }

    /// Calculate the strength of a cycle.
    async fn calculate_cycle_strength(&self, cycle_path: &[Uuid]) -> Result<f64> {
        // Simplified calculation - could be enhanced with actual edge weights
        Ok(1.0 / cycle_path.len() as f64)
    }

    /// Calculate the total weight of a path.
    async fn calculate_path_weight(&self, path: &[Uuid]) -> Result<f64> {
        // Simplified calculation - could query actual edge weights
        Ok(path.len() as f64)
    }
}

/// Type of graph traversal to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraversalType {
    /// Forward dependency traversal (what this element depends on)
    Dependencies,
    /// Backward dependency traversal (what depends on this element)
    Dependents,
    /// Forward call chain traversal (what this function calls)
    CallChains,
    /// Backward call chain traversal (what calls this function)
    CallersChain,
    /// Type relationship traversal
    TypeRelationships,
}

/// Node in the traversal graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalNode {
    pub element_id: Uuid,
    pub depth: usize,
    pub path: Vec<Uuid>,
    pub relationship_type: String,
    pub weight: f64,
}

/// Edge in the graph
#[derive(Debug, Clone)]
struct GraphEdge {
    target_id: Uuid,
    relationship_type: String,
    weight: f64,
}

/// BFS traversal result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BfsResult {
    pub start_element: Uuid,
    pub nodes: Vec<TraversalNode>,
    pub max_depth: usize,
    pub total_nodes: usize,
}

impl BfsResult {
    fn new(start_element: Uuid) -> Self {
        Self {
            start_element,
            nodes: Vec::new(),
            max_depth: 0,
            total_nodes: 0,
        }
    }

    fn add_node(&mut self, node: TraversalNode) {
        self.max_depth = self.max_depth.max(node.depth);
        self.nodes.push(node);
        self.total_nodes += 1;
    }
}

/// DFS traversal result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfsResult {
    pub start_element: Uuid,
    pub nodes: Vec<TraversalNode>,
    pub max_depth: usize,
    pub total_nodes: usize,
}

impl DfsResult {
    fn new(start_element: Uuid) -> Self {
        Self {
            start_element,
            nodes: Vec::new(),
            max_depth: 0,
            total_nodes: 0,
        }
    }

    fn add_node(&mut self, node: TraversalNode) {
        self.max_depth = self.max_depth.max(node.depth);
        self.nodes.push(node);
        self.total_nodes += 1;
    }
}

/// Cycle detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleDetectionResult {
    pub cycles: Vec<Cycle>,
    pub total_cycles: usize,
    pub simple_cycles: usize,
    pub complex_cycles: usize,
}

impl CycleDetectionResult {
    fn new() -> Self {
        Self {
            cycles: Vec::new(),
            total_cycles: 0,
            simple_cycles: 0,
            complex_cycles: 0,
        }
    }

    fn add_cycle(&mut self, cycle: Cycle) {
        match cycle.cycle_type {
            CycleType::Simple => self.simple_cycles += 1,
            _ => self.complex_cycles += 1,
        }
        self.cycles.push(cycle);
        self.total_cycles += 1;
    }
}

/// A detected cycle in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub elements: Vec<Uuid>,
    pub cycle_type: CycleType,
    pub length: usize,
    pub strength: f64,
}

/// Type of cycle detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleType {
    /// Simple cycle (direct loop)
    Simple,
    /// Strongly connected component
    StronglyConnected,
    /// Complex multi-component cycle
    Complex,
}

/// Shortest path result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortestPath {
    pub path: Vec<Uuid>,
    pub total_distance: f64,
    pub edge_count: usize,
}

/// A path through the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub elements: Vec<Uuid>,
    pub length: usize,
    pub total_weight: f64,
}

/// Node for Dijkstra's algorithm
#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct DijkstraNode {
    distance: f64,
    element_id: Uuid,
}

impl Eq for DijkstraNode {}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.partial_cmp(&other.distance).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_traversal_type_serialization() {
        let traversal_type = TraversalType::Dependencies;
        let serialized = serde_json::to_string(&traversal_type).unwrap();
        let deserialized: TraversalType = serde_json::from_str(&serialized).unwrap();
        
        matches!(deserialized, TraversalType::Dependencies);
    }

    #[test]
    fn test_bfs_result_creation() {
        let start_element = Uuid::new_v4();
        let result = BfsResult::new(start_element);
        
        assert_eq!(result.start_element, start_element);
        assert_eq!(result.total_nodes, 0);
        assert_eq!(result.max_depth, 0);
    }

    #[test]
    fn test_cycle_detection_result() {
        let mut result = CycleDetectionResult::new();
        
        let cycle = Cycle {
            elements: vec![Uuid::new_v4(), Uuid::new_v4()],
            cycle_type: CycleType::Simple,
            length: 2,
            strength: 0.5,
        };
        
        result.add_cycle(cycle);
        
        assert_eq!(result.total_cycles, 1);
        assert_eq!(result.simple_cycles, 1);
        assert_eq!(result.complex_cycles, 0);
    }
}