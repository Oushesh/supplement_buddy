use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// SQLite database URL, e.g. `sqlite://./supplement_buddy.db` or `sqlite::memory:`
    pub database_url: String,
    /// Host to bind the HTTP server on
    pub host: String,
    /// Port to bind the HTTP server on
    pub port: u16,
    /// Allowed CORS origins (space-separated)
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://./supplement_buddy.db".to_string());

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8001);

        let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000 http://127.0.0.1:3000".to_string())
            .split_whitespace()
            .map(str::to_string)
            .collect();

        Self {
            database_url,
            host,
            port,
            cors_origins,
        }
    }
}
