use sqlx::sqlite::SqlitePool;
use sqlx::migrate::Migrator;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Database connection pool wrapper
#[derive(Clone, Debug)]
pub struct DatabasePool {
    pool: SqlitePool,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(db_path: &Path) -> Result<Self, DatabaseError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Get absolute path for SQLite
        let absolute_path = std::fs::canonicalize(db_path)
            .unwrap_or_else(|_| db_path.to_path_buf());

        // Ensure database file exists (SQLite requires the file to exist for some operations)
        if !absolute_path.exists() {
            std::fs::File::create(&absolute_path)?;
        }

        // Use file URI format for SQLite
        let url = format!("file:{}", absolute_path.to_string_lossy());
        let pool = SqlitePool::connect(&url).await?;

        Ok(Self { pool })
    }

    /// Run migrations
    pub async fn migrate(&self, migrations_path: &Path) -> Result<(), DatabaseError> {
        let m = Migrator::new(migrations_path).await?;
        m.run(&self.pool).await?;
        Ok(())
    }

    /// Get the underlying pool reference
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get mutable pool reference
    pub fn get_pool_mut(&mut self) -> &mut SqlitePool {
        &mut self.pool
    }
}

impl From<SqlitePool> for DatabasePool {
    fn from(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AsRef<SqlitePool> for DatabasePool {
    fn as_ref(&self) -> &SqlitePool {
        &self.pool
    }
}
