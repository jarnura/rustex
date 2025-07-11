//! # RustEx Database Layer
//!
//! This crate provides persistent storage and graph traversal capabilities for RustEx AST data.
//! It includes:
//!
//! - PostgreSQL schema optimized for graph queries
//! - AST node storage and retrieval
//! - Call chain and dependency graph traversal
//! - Migration system with versioning
//! - Connection pooling and transaction management
//!
//! ## Features
//!
//! - **Graph Storage**: Efficient storage of call chains and type relationships
//! - **Optimized Queries**: Indexes designed for graph traversal operations
//! - **ACID Transactions**: Reliable data consistency
//! - **Migration System**: Version-controlled schema changes
//! - **Connection Pooling**: High-performance database connections
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rustex_db::{DatabaseManager, schema::DbConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = DbConfig::from_url("postgresql://user:pass@localhost/rustex")?;
//! let db = DatabaseManager::new(config).await?;
//!
//! // Run migrations
//! db.migrate().await?;
//!
//! // Store AST data
//! // db.store_project_ast(project_ast).await?;
//! # Ok(())
//! # }
//! ```

pub mod schema;
pub mod migrations;
pub mod connection;
pub mod storage;
pub mod query;
pub mod error;

#[cfg(feature = "graph-algorithms")]
pub mod graph;

pub use error::{DatabaseError, Result};
pub use connection::{DatabaseManager, ConnectionPool};
pub use schema::{DbConfig, ProjectRecord, FileRecord, ElementRecord};
pub use storage::{AstStorage, ProjectStorage, ElementStorage};

#[cfg(feature = "graph-algorithms")]
pub use graph::{GraphTraversal, CallChainAnalyzer, DependencyAnalyzer};

/// Database configuration and initialization
pub mod prelude {
    pub use crate::{
        DatabaseManager, DatabaseError, Result,
        schema::{DbConfig, ProjectRecord, FileRecord, ElementRecord},
        storage::{AstStorage, ProjectStorage, ElementStorage},
        query::{RustexQueryBuilder, GraphQuery},
    };
    
    #[cfg(feature = "graph-algorithms")]
    pub use crate::graph::{GraphTraversal, CallChainAnalyzer, DependencyAnalyzer};
}