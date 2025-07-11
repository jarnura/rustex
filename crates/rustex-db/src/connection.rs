//! Database connection management with pooling and transaction support.

use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use crate::error::{DatabaseError, Result};
use crate::schema::DbConfig;
use crate::migrations::MigrationManager;

/// Database connection pool wrapper
pub type ConnectionPool = PgPool;

/// Database manager providing high-level database operations
pub struct DatabaseManager {
    pool: PgPool,
    config: DbConfig,
}

impl DatabaseManager {
    /// Create a new database manager with the given configuration.
    pub async fn new(config: DbConfig) -> Result<Self> {
        let pool = Self::create_pool(&config).await?;
        
        Ok(Self {
            pool,
            config,
        })
    }

    /// Create a connection pool with the given configuration.
    async fn create_pool(config: &DbConfig) -> Result<PgPool> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout))
            .idle_timeout(Duration::from_secs(300)) // 5 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            .test_before_acquire(true)
            .connect(&config.database_url)
            .await
            .map_err(|e| DatabaseError::connection(
                format!("Failed to create connection pool: {}", e)
            ))?;

        Ok(pool)
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get a clone of the connection pool.
    pub fn pool_clone(&self) -> PgPool {
        self.pool.clone()
    }

    /// Test the database connection.
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(DatabaseError::from)?;
        
        Ok(())
    }

    /// Run database migrations.
    pub async fn migrate(&self) -> Result<()> {
        let migration_manager = MigrationManager::new(self.pool.clone());
        migration_manager.migrate().await?;
        Ok(())
    }

    /// Get migration status.
    pub async fn migration_status(&self) -> Result<crate::migrations::MigrationStatus> {
        let migration_manager = MigrationManager::new(self.pool.clone());
        migration_manager.status().await
    }

    /// Rollback the last migration.
    pub async fn rollback_migration(&self) -> Result<()> {
        let migration_manager = MigrationManager::new(self.pool.clone());
        migration_manager.rollback().await?;
        Ok(())
    }

    /// Begin a database transaction.
    pub async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        let tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;
        
        Ok(DatabaseTransaction::new(tx))
    }

    /// Execute a closure within a transaction, automatically committing or rolling back.
    pub async fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: for<'c> FnOnce(&mut Transaction<'c, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'c>>,
    {
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await
                    .map_err(DatabaseError::from)?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(DatabaseError::from)?;
                Err(e)
            }
        }
    }

    /// Get database connection statistics.
    pub fn connection_stats(&self) -> ConnectionStats {
        ConnectionStats {
            size: self.pool.size(),
            num_idle: self.pool.num_idle(),
            is_closed: self.pool.is_closed(),
        }
    }

    /// Close the database connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Check if the database schema is up to date.
    pub async fn is_schema_up_to_date(&self) -> Result<bool> {
        let migration_manager = MigrationManager::new(self.pool.clone());
        migration_manager.is_up_to_date().await
    }

    /// Get database configuration.
    pub fn config(&self) -> &DbConfig {
        &self.config
    }
}

/// Transaction wrapper providing additional functionality
pub struct DatabaseTransaction {
    transaction: Option<Transaction<'static, Postgres>>,
    committed: bool,
    rolled_back: bool,
}

impl DatabaseTransaction {
    fn new(transaction: Transaction<'static, Postgres>) -> Self {
        Self {
            transaction: Some(transaction),
            committed: false,
            rolled_back: false,
        }
    }

    /// Commit the transaction.
    pub async fn commit(mut self) -> Result<()> {
        if let Some(tx) = self.transaction.take() {
            tx.commit().await
                .map_err(DatabaseError::from)?;
            self.committed = true;
        }
        Ok(())
    }

    /// Rollback the transaction.
    pub async fn rollback(mut self) -> Result<()> {
        if let Some(tx) = self.transaction.take() {
            tx.rollback().await
                .map_err(DatabaseError::from)?;
            self.rolled_back = true;
        }
        Ok(())
    }

    /// Get a mutable reference to the transaction.
    pub fn as_mut(&mut self) -> Option<&mut Transaction<'static, Postgres>> {
        self.transaction.as_mut()
    }

    /// Check if the transaction has been committed.
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// Check if the transaction has been rolled back.
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }
}

impl Drop for DatabaseTransaction {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            tracing::warn!("Transaction dropped without commit or rollback");
            // The underlying transaction will be automatically rolled back
        }
    }
}

/// Database connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Total number of connections in the pool
    pub size: u32,
    /// Number of idle connections
    pub num_idle: usize,
    /// Whether the pool is closed
    pub is_closed: bool,
}

/// Connection health check
pub struct HealthCheck {
    manager: DatabaseManager,
}

impl HealthCheck {
    /// Create a new health check instance.
    pub fn new(manager: DatabaseManager) -> Self {
        Self { manager }
    }

    /// Perform a comprehensive health check.
    pub async fn check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        
        // Test basic connectivity
        let connection_ok = self.manager.test_connection().await.is_ok();
        
        // Check pool status
        let stats = self.manager.connection_stats();
        let pool_healthy = !stats.is_closed && stats.size > 0;
        
        // Check migration status
        let migrations_ok = self.manager.is_schema_up_to_date().await
            .unwrap_or(false);
        
        let response_time = start_time.elapsed();
        
        let status = if connection_ok && pool_healthy && migrations_ok {
            HealthStatusLevel::Healthy
        } else if connection_ok {
            HealthStatusLevel::Degraded
        } else {
            HealthStatusLevel::Unhealthy
        };

        Ok(HealthStatus {
            status,
            connection_ok,
            pool_healthy,
            migrations_ok,
            response_time,
            connection_stats: stats,
        })
    }
}

/// Health status levels
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatusLevel {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Comprehensive health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: HealthStatusLevel,
    pub connection_ok: bool,
    pub pool_healthy: bool,
    pub migrations_ok: bool,
    pub response_time: Duration,
    pub connection_stats: ConnectionStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DbConfig;

    #[tokio::test]
    async fn test_database_manager_creation() {
        let config = DbConfig::test();
        
        // This test would require a real database connection
        // In practice, you'd use testcontainers for integration tests
        assert_eq!(config.max_connections, 2);
        assert_eq!(config.application_name, "rustex-test");
    }

    #[test]
    fn test_connection_stats() {
        // Unit test for connection stats structure
        let stats = ConnectionStats {
            size: 5,
            num_idle: 3,
            is_closed: false,
        };
        
        assert_eq!(stats.size, 5);
        assert_eq!(stats.num_idle, 3);
        assert!(!stats.is_closed);
    }
}