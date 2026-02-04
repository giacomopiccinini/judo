use anyhow::{Context, Result};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

/// Create connection to SQLite DB pool and create DB if not present
pub async fn get_db_pool(db_connection_str: &str) -> Result<SqlitePool> {
    // Create connection options
    let opts = SqliteConnectOptions::from_str(db_connection_str)
        .with_context(|| "Failed to create options for DB")?
        .create_if_missing(true);

    // Connect in a pool
    let pool = SqlitePool::connect_with(opts)
        .await
        .with_context(|| "Failed to create DB pool")?;

    Ok(pool)
}

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Embed the migration files into binary
    static MIGRATOR: Migrator = sqlx::migrate!();

    MIGRATOR
        .run(pool)
        .await
        .with_context(|| "Failed to run database migrations")?;

    Ok(())
}

/// Initialize database with connection and run migrations
/// This is safe to call on every startup - migrations are idempotent
pub async fn init_db(connection_str: &str) -> Result<SqlitePool> {
    let pool = get_db_pool(connection_str).await?;

    // Always run migrations on startup - they're idempotent and fast
    run_migrations(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_connection_pool_in_memory_success() -> Result<()> {
        let connection_str = "sqlite::memory:".to_string();
        let _pool = get_db_pool(&connection_str).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_migrations() -> Result<()> {
        let connection_str = "sqlite::memory:".to_string();
        let pool = get_db_pool(&connection_str).await?;
        run_migrations(&pool).await?;
        Ok(())
    }
}
