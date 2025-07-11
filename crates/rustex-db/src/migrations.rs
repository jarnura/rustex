//! Database migration system with versioning and rollback capabilities.

use sqlx::{PgPool, Row};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::{DatabaseError, Result};

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub description: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
    pub checksum: String,
}

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    pool: PgPool,
    migrations_dir: String,
}

impl MigrationManager {
    /// Create a new migration manager.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            migrations_dir: "migrations".to_string(),
        }
    }

    /// Create a new migration manager with custom migrations directory.
    pub fn with_migrations_dir(pool: PgPool, migrations_dir: &str) -> Self {
        Self {
            pool,
            migrations_dir: migrations_dir.to_string(),
        }
    }

    /// Initialize the migration system by creating the migrations table.
    pub async fn initialize(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS __rustex_migrations (
                version INTEGER PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                checksum VARCHAR(64) NOT NULL,
                applied_at TIMESTAMPTZ DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_migrations_applied_at 
            ON __rustex_migrations(applied_at);
        "#;

        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }

    /// Get all available migrations from the migrations directory.
    pub fn discover_migrations(&self) -> Result<Vec<Migration>> {
        let migrations_path = Path::new(&self.migrations_dir);
        
        if !migrations_path.exists() {
            return Err(DatabaseError::migration(
                format!("Migrations directory not found: {}", self.migrations_dir)
            ));
        }

        let mut migrations = Vec::new();
        let entries = fs::read_dir(migrations_path)
            .map_err(|e| DatabaseError::migration(format!("Failed to read migrations directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| DatabaseError::migration(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(migration) = self.parse_migration_file(&path, file_name)? {
                        migrations.push(migration);
                    }
                }
            }
        }

        // Sort migrations by version
        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Parse a migration file and extract metadata.
    fn parse_migration_file(&self, path: &Path, file_name: &str) -> Result<Option<Migration>> {
        // Parse version from filename (e.g., "001_initial_schema.sql" -> 1)
        let parts: Vec<&str> = file_name.split('_').collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let version = parts[0].parse::<i32>()
            .map_err(|_| DatabaseError::migration(
                format!("Invalid migration version in filename: {}", file_name)
            ))?;

        let name = parts[1..].join("_");
        
        let content = fs::read_to_string(path)
            .map_err(|e| DatabaseError::migration(
                format!("Failed to read migration file {}: {}", path.display(), e)
            ))?;

        // Calculate checksum
        let checksum = format!("{:x}", md5::compute(&content));

        // Look for description in comments
        let description = self.extract_description(&content);

        // Split into up and down migrations if present
        let (up_sql, down_sql) = self.split_migration_content(&content);

        Ok(Some(Migration {
            version,
            name,
            description,
            up_sql,
            down_sql,
            applied_at: None,
            checksum,
        }))
    }

    /// Extract description from migration file comments.
    fn extract_description(&self, content: &str) -> String {
        for line in content.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix("-- Description:") {
                return stripped.trim().to_string();
            } else if line.starts_with("--") && !line.starts_with("---") {
                // Use first comment as description
                return line[2..].trim().to_string();
            }
        }
        "No description provided".to_string()
    }

    /// Split migration content into up and down parts.
    fn split_migration_content(&self, content: &str) -> (String, Option<String>) {
        let lines: Vec<&str> = content.lines().collect();
        let mut up_lines = Vec::new();
        let mut down_lines = Vec::new();
        let mut in_down_section = false;

        for line in lines {
            if line.trim().to_lowercase().starts_with("-- down") ||
               line.trim().to_lowercase().starts_with("-- rollback") {
                in_down_section = true;
                continue;
            }

            if in_down_section {
                down_lines.push(line);
            } else {
                up_lines.push(line);
            }
        }

        let up_sql = up_lines.join("\n");
        let down_sql = if down_lines.is_empty() {
            None
        } else {
            Some(down_lines.join("\n"))
        };

        (up_sql, down_sql)
    }

    /// Get all applied migrations from the database.
    pub async fn get_applied_migrations(&self) -> Result<Vec<Migration>> {
        let rows = sqlx::query(
            "SELECT version, name, description, checksum, applied_at 
             FROM __rustex_migrations 
             ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::from)?;

        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(Migration {
                version: row.get("version"),
                name: row.get("name"),
                description: row.get::<Option<String>, _>("description").unwrap_or_default(),
                up_sql: String::new(), // Not stored in DB
                down_sql: None,
                applied_at: Some(row.get("applied_at")),
                checksum: row.get("checksum"),
            });
        }

        Ok(migrations)
    }

    /// Get pending migrations that need to be applied.
    pub async fn get_pending_migrations(&self) -> Result<Vec<Migration>> {
        let available = self.discover_migrations()?;
        let applied = self.get_applied_migrations().await?;
        
        let applied_versions: std::collections::HashSet<i32> = 
            applied.iter().map(|m| m.version).collect();

        let pending: Vec<Migration> = available
            .into_iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();

        Ok(pending)
    }

    /// Apply all pending migrations.
    pub async fn migrate(&self) -> Result<Vec<Migration>> {
        self.initialize().await?;
        
        let pending = self.get_pending_migrations().await?;
        let mut applied = Vec::new();

        for migration in pending {
            self.apply_migration(&migration).await?;
            applied.push(migration);
        }

        Ok(applied)
    }

    /// Apply a specific migration.
    pub async fn apply_migration(&self, migration: &Migration) -> Result<()> {
        tracing::info!("Applying migration {}: {}", migration.version, migration.name);

        // Start transaction
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        // Execute the migration
        sqlx::query(&migration.up_sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::migration(
                format!("Failed to apply migration {}: {}", migration.version, e)
            ))?;

        // Record the migration
        sqlx::query(
            "INSERT INTO __rustex_migrations (version, name, description, checksum) 
             VALUES ($1, $2, $3, $4)"
        )
        .bind(migration.version)
        .bind(&migration.name)
        .bind(&migration.description)
        .bind(&migration.checksum)
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::from)?;

        // Commit transaction
        tx.commit().await
            .map_err(DatabaseError::from)?;

        tracing::info!("Successfully applied migration {}", migration.version);
        Ok(())
    }

    /// Rollback the last applied migration.
    pub async fn rollback(&self) -> Result<Option<Migration>> {
        let applied = self.get_applied_migrations().await?;
        
        if let Some(last_migration) = applied.last() {
            self.rollback_migration(last_migration.version).await?;
            Ok(Some(last_migration.clone()))
        } else {
            Ok(None)
        }
    }

    /// Rollback a specific migration by version.
    pub async fn rollback_migration(&self, version: i32) -> Result<()> {
        // Find the migration file to get the down SQL
        let available = self.discover_migrations()?;
        let migration = available
            .iter()
            .find(|m| m.version == version)
            .ok_or_else(|| DatabaseError::migration(
                format!("Migration version {} not found", version)
            ))?;

        let down_sql = migration.down_sql.as_ref()
            .ok_or_else(|| DatabaseError::migration(
                format!("No rollback SQL found for migration {}", version)
            ))?;

        tracing::info!("Rolling back migration {}: {}", version, migration.name);

        // Start transaction
        let mut tx = self.pool.begin().await
            .map_err(DatabaseError::from)?;

        // Execute the rollback
        sqlx::query(down_sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DatabaseError::migration(
                format!("Failed to rollback migration {}: {}", version, e)
            ))?;

        // Remove the migration record
        sqlx::query("DELETE FROM __rustex_migrations WHERE version = $1")
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::from)?;

        // Commit transaction
        tx.commit().await
            .map_err(DatabaseError::from)?;

        tracing::info!("Successfully rolled back migration {}", version);
        Ok(())
    }

    /// Check if migrations are up to date.
    pub async fn is_up_to_date(&self) -> Result<bool> {
        let pending = self.get_pending_migrations().await?;
        Ok(pending.is_empty())
    }

    /// Get migration status information.
    pub async fn status(&self) -> Result<MigrationStatus> {
        let available = self.discover_migrations()?;
        let applied = self.get_applied_migrations().await?;
        let pending = self.get_pending_migrations().await?;

        Ok(MigrationStatus {
            total_migrations: available.len(),
            applied_count: applied.len(),
            pending_count: pending.len(),
            last_applied: applied.last().map(|m| m.version),
            is_up_to_date: pending.is_empty(),
        })
    }

    /// Validate migration checksums to detect modifications.
    pub async fn validate_checksums(&self) -> Result<Vec<String>> {
        let available = self.discover_migrations()?;
        let applied = self.get_applied_migrations().await?;
        
        let mut errors = Vec::new();

        for applied_migration in &applied {
            if let Some(available_migration) = available
                .iter()
                .find(|m| m.version == applied_migration.version) {
                
                if applied_migration.checksum != available_migration.checksum {
                    errors.push(format!(
                        "Migration {} has been modified (checksum mismatch)",
                        applied_migration.version
                    ));
                }
            } else {
                errors.push(format!(
                    "Applied migration {} not found in migration files",
                    applied_migration.version
                ));
            }
        }

        Ok(errors)
    }
}

/// Migration status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub total_migrations: usize,
    pub applied_count: usize,
    pub pending_count: usize,
    pub last_applied: Option<i32>,
    pub is_up_to_date: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_parse_migration_filename() {
        let temp_dir = TempDir::new().unwrap();
        let migration_path = temp_dir.path().join("001_initial_schema.sql");
        
        fs::write(&migration_path, "-- Initial schema\nCREATE TABLE test();").unwrap();
        
        let manager = MigrationManager::new(
            // This would be a real pool in practice
            PgPool::connect("postgresql://test").await.unwrap()
        );
        
        let migration = manager
            .parse_migration_file(&migration_path, "001_initial_schema")
            .unwrap()
            .unwrap();
        
        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "initial_schema");
        assert!(migration.up_sql.contains("CREATE TABLE test"));
    }

    #[tokio::test]
    async fn test_split_migration_content() {
        let manager = MigrationManager::new(
            PgPool::connect("postgresql://test").await.unwrap()
        );
        
        let content = r#"
CREATE TABLE test();

-- DOWN
DROP TABLE test;
        "#;
        
        let (up_sql, down_sql) = manager.split_migration_content(content);
        
        assert!(up_sql.contains("CREATE TABLE test"));
        assert!(down_sql.is_some());
        assert!(down_sql.unwrap().contains("DROP TABLE test"));
    }
}