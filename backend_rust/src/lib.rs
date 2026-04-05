pub mod config;
pub mod db;
pub mod entities;
pub mod error;
pub mod handlers;
pub mod migration;
pub mod models;

use axum::{
    routing::{get, put},
    Router,
};
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use handlers::{
    search_queries::list_search_queries,
    supplements::{
        create_supplement, get_supplement, list_supplements, search_supplements,
        update_splade_vector,
    },
};

/// Builds the Axum router with the given SeaORM database connection and CORS origins.
/// Extracted into the library so integration tests can call it directly.
pub fn build_router(db: DatabaseConnection, cors_origins: &[String]) -> Router {
    let cors = if cors_origins.is_empty() {
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new().allow_origin(origins)
    };

    Router::new()
        // Supplements
        .route("/api/supplements", get(list_supplements).post(create_supplement))
        .route("/api/supplements/search", get(search_supplements))
        .route("/api/supplements/:id", get(get_supplement))
        .route("/api/supplements/:id/splade-vector", put(update_splade_vector))
        // Search queries
        .route("/api/search-queries", get(list_search_queries))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}
