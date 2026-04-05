mod entities; //This links to your generated entities
mod seed; // This links your see.rs file


use std::net::SocketAddr;
use std::env; //Added to check arguments
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

    let cfg = Config::from_env();
    tracing::info!("Connecting to database: {}", cfg.database_url);
    let db_conn = db::connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    /*
    The server here to seed if it was not run by user.
    */

    //4. Check for Seed Flag
    // We check args before starting the server
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--seed".to_string()) {
        tracing::info!("Seed flag detected. Populating database...");

        if let Err(e) = seed::run(&db_conn).await {
            tracing::error!("Seeding failed: {:?}", e);
            std::process::exit(1);
        }
        tracing::info!("Seeding successful. Exiting.");
        return; // Exit after seeding so you can restart the server clean
    }

    //5. Build and Start Server
    let app = build_router(db_conn.clone(),&cfg.cors_origins);



    let app = build_router(db_conn, &cfg.cors_origins);

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .expect("Invalid bind address");

    tracing::info!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

