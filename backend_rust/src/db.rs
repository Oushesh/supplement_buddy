use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::migration::Migrator;

/// Connect to the database and run all pending migrations.
/// For file-based SQLite databases, creates the file if it does not already exist.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    ensure_sqlite_file_exists(database_url);

    let opt = ConnectOptions::new(database_url.to_string())
        .max_connections(5)
        .to_owned();

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Public alias used by integration tests so they can set up an in-memory DB.
pub async fn run_migrations_for_test(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}

/// Pre-creates the SQLite database file if the URL points to a file path that
/// does not yet exist. This mirrors the `create_if_missing` behaviour from sqlx
/// that sea-orm's `ConnectOptions` does not currently expose.
fn ensure_sqlite_file_exists(database_url: &str) {
    // Extract the filesystem path from common SQLite URL formats:
    //   sqlite://./relative/path.db  ->  ./relative/path.db
    //   sqlite:///absolute/path.db   ->  /absolute/path.db
    //   sqlite::memory:              ->  skip (in-memory)
    let path_str = if let Some(p) = database_url.strip_prefix("sqlite://./") {
        format!("./{p}")
    } else if let Some(p) = database_url.strip_prefix("sqlite:///") {
        format!("/{p}")
    } else {
        return; // in-memory, bare file, or unknown format — leave to the driver
    };

    if path_str.is_empty() {
        return;
    }

    let path = std::path::Path::new(&path_str);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let _ = std::fs::File::create(path);
    }
}
