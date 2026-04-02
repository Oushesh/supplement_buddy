use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Supplement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Supplement {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub category: String,
    pub description: String,
    pub ingredients: String,
    pub serving_size: String,
    /// SPLADE sparse vector stored as JSON (token → weight map)
    pub splade_vector: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload accepted when creating a supplement via POST /api/supplements/
#[derive(Debug, Deserialize)]
pub struct CreateSupplement {
    pub name: Option<String>,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub ingredients: String,
    #[serde(default)]
    pub serving_size: String,
}

// ---------------------------------------------------------------------------
// SearchQuery
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchQuery {
    pub id: i64,
    pub query: String,
    /// JSON array of matched supplement IDs
    pub results: Value,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub count: i64,
    pub page: i64,
    pub page_size: i64,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    20
}
