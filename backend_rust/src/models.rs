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

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pagination_params_defaults() {
        let params: PaginationParams =
            serde_json::from_value(json!({})).expect("deserialize");
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
    }

    #[test]
    fn pagination_params_custom_values() {
        let params: PaginationParams =
            serde_json::from_value(json!({ "page": 3, "page_size": 50 })).expect("deserialize");
        assert_eq!(params.page, 3);
        assert_eq!(params.page_size, 50);
    }

    #[test]
    fn page_serializes_correctly() {
        let page: Page<String> = Page {
            count: 42,
            page: 2,
            page_size: 10,
            results: vec!["a".to_string(), "b".to_string()],
        };
        let json = serde_json::to_value(&page).expect("serialize");
        assert_eq!(json["count"], 42);
        assert_eq!(json["page"], 2);
        assert_eq!(json["page_size"], 10);
        assert_eq!(json["results"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn create_supplement_name_is_optional() {
        let with_name: CreateSupplement =
            serde_json::from_value(json!({ "name": "Zinc" })).expect("deserialize");
        assert_eq!(with_name.name.as_deref(), Some("Zinc"));

        let without_name: CreateSupplement =
            serde_json::from_value(json!({})).expect("deserialize");
        assert!(without_name.name.is_none());
    }

    #[test]
    fn create_supplement_optional_string_fields_default_to_empty() {
        let s: CreateSupplement =
            serde_json::from_value(json!({ "name": "Test" })).expect("deserialize");
        assert_eq!(s.brand, "");
        assert_eq!(s.category, "");
        assert_eq!(s.description, "");
        assert_eq!(s.ingredients, "");
        assert_eq!(s.serving_size, "");
    }
}
