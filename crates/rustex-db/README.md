# RustEx Database Layer

PostgreSQL-based storage and graph traversal for RustEx AST data.

## Features

- **Optimized Schema**: PostgreSQL schema designed for efficient graph queries
- **Migration System**: Version-controlled schema changes with rollback support
- **Graph Traversal**: BFS/DFS algorithms for dependency and call chain analysis
- **Connection Pooling**: High-performance database connection management
- **ACID Transactions**: Reliable data consistency for complex operations

## Quick Start

```rust
use rustex_db::{DatabaseManager, schema::DbConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure database
    let config = DbConfig::from_url("postgresql://user:pass@localhost/rustex")?;
    let db = DatabaseManager::new(config).await?;
    
    // Run migrations
    db.migrate().await?;
    
    // Store AST data
    let storage = AstStorage::new(db.pool_clone());
    storage.store_project_ast(&project_ast).await?;
    
    Ok(())
}
```

## Schema Overview

The database schema is optimized for graph traversal operations:

- **projects**: Project metadata and metrics
- **files**: File-level information and complexity metrics  
- **ast_elements**: Individual AST elements with location data
- **cross_references**: References between elements
- **dependencies**: Processed dependency relationships
- **call_chains**: Function call relationships
- **type_relationships**: Type-level relationships

## Graph Operations

### Dependency Traversal

```rust
let graph = GraphTraversal::new(pool);
let deps = graph.find_dependencies(element_id, Some(5)).await?;
let dependents = graph.find_dependents(element_id, Some(5)).await?;
```

### Call Chain Analysis

```rust
let analyzer = CallChainAnalyzer::new(pool);
let analysis = analyzer.analyze_call_chain(function_id, Some(10)).await?;
```

### Cycle Detection

```rust
let cycles = graph.detect_cycles(project_id).await?;
```

## Query Builder

Fluent interface for complex queries:

```rust
let elements = db.query()
    .project(project_id)
    .element_types(vec!["Function", "Struct"])
    .complexity_range(10, 50)
    .with_metrics()
    .order_by("complexity", "DESC")
    .limit(100)
    .fetch_elements()
    .await?;
```

## Migrations

The migration system provides version control for schema changes:

```bash
# Check migration status
rustex-db migrate status

# Apply pending migrations  
rustex-db migrate up

# Rollback last migration
rustex-db migrate down
```

## Performance

The schema includes optimized indexes for:

- Graph traversal queries (BFS/DFS)
- Full-text search across elements
- Complexity-based filtering
- Type and visibility queries
- Cross-reference lookups

## Development

```bash
# Set up test database
createdb rustex_test
export DATABASE_URL="postgresql://localhost/rustex_test"

# Run tests
cargo test

# Run integration tests with database
cargo test --features integration-tests
```