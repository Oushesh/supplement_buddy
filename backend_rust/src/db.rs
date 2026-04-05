use std::str::FromStr;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use sqlx::sqlite::SqliteConnectOptions;

/*
/// Create (or connect to) the SQLite pool and run embedded migrations.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    run_migrations(&pool).await?;
    Ok(pool)
}
*/

// New connection with option to create the db if file is missing
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    //1. Parse the string into connection options
    let connection_options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    // 2. Tell SQLX to create .db file if it isn't there

    //Connect using those specific options
    let pool = SqlitePoolOptions::new().max_connections(5).connect_with(connection_options).await?;
    //Use connect_with instead of connect

    run_migrations(&pool).await?;
    Ok(pool)
}

/// Public alias used by integration tests so they can set up an in-memory DB.
pub async fn run_migrations_for_test(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    run_migrations(pool).await
}

/// Inline migrations — avoids a separate `migrations/` folder so the binary
/// is fully self-contained when deployed.
async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS supplements (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT    NOT NULL,
            brand        TEXT    NOT NULL DEFAULT '',
            category     TEXT    NOT NULL DEFAULT '',
            description  TEXT    NOT NULL DEFAULT '',
            ingredients  TEXT    NOT NULL DEFAULT '',
            serving_size TEXT    NOT NULL DEFAULT '',
            splade_vector TEXT,
            created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS search_queries (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            query      TEXT NOT NULL,
            results    TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
