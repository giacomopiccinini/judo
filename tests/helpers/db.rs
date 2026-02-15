use anyhow::{Context, Result};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

static MIGRATOR: Migrator = sqlx::migrate!();
static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create an in-memory SQLite database for testing
pub async fn setup_test_db() -> Result<SqlitePool> {
    // Connection options as in main implementation
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .with_context(|| "Failed to create options for DB")?
        .create_if_missing(true);

    // Connect in a pool
    let pool = SqlitePool::connect_with(opts)
        .await
        .with_context(|| "Failed to create DB pool")?;

    // Run migrations
    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

/// Create a named shared in-memory SQLite database for testing.
///
/// Returns the pool and the connection string. The connection string can be
/// used to open additional connections to the **same** in-memory database,
/// which is what the CLI ops functions do internally via `get_db_pool`.
///
/// Each call produces a unique database name so tests never interfere.
pub async fn setup_test_db_shared() -> Result<(SqlitePool, String)> {
    let id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let connection_str = format!("sqlite:file:testdb_{}?mode=memory&cache=shared", id);

    let opts = SqliteConnectOptions::from_str(&connection_str)
        .with_context(|| "Failed to create options for DB")?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts)
        .await
        .with_context(|| "Failed to create DB pool")?;

    MIGRATOR.run(&pool).await?;

    Ok((pool, connection_str))
}
