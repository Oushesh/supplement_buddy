pub mod config;
pub mod db;
pub mod entities; // Added to expose the generated SeaORM models
pub mod error;
pub mod handlers;

use axum::{
    routing::{get, put},
    Router,
};
use sea_orm::DatabaseConnection; // The new "Heart" of your DB logic
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// We keep your handlers, but they will now expect DatabaseConnection in their State
use handlers::{
    search_queries::list_search_queries,
    supplements::{
        create_supplement, get_supplement, list_supplements, search_supplements,
        update_splade_vector,
    },
};

/// Builds the Axum router with a SeaORM DatabaseConnection.
/// This allows the entire app to share a single connection pool.
pub fn build_router(db: DatabaseConnection, cors_origins: &[String]) -> Router {
    // 1. Setup CORS (Cross-Origin Resource Sharing)
    let cors = if cors_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new().allow_origin(origins)
    };

    // 2. Define the Routes
    Router::new()
        // Supplements API
        .route("/api/supplements", get(list_supplements).post(create_supplement))
        .route("/api/supplements/search", get(search_supplements))
        .route("/api/supplements/:id", get(get_supplement))
        .route("/api/supplements/:id/splade-vector", put(update_splade_vector))

        // Search Queries API
        .route("/api/search-queries", get(list_search_queries))

        // 3. Layer on Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())

        // 4. Inject Database State
        // This makes 'db' available to every handler function via State(db)
        .with_state(db)
}