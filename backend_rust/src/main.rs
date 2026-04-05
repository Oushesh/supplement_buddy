use std::net::SocketAddr;

use supplement_buddy_backend::{build_router, config::Config, db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Load .env if present (ignore errors — not required in production)
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let seed_mode = std::env::args().any(|a| a == "--seed");

    let cfg = Config::from_env();

    tracing::info!("Connecting to database: {}", cfg.database_url);
    let database = db::connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    if seed_mode {
        tracing::info!("Seed mode: populating sample data…");
        db::seed_database(&database)
            .await
            .expect("Failed to seed database");
        tracing::info!("Done.");
        return;
    }

    let app = build_router(database, &cfg.cors_origins);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .expect("Invalid bind address");

    tracing::info!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

