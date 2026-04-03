//! Integration tests for the Supplement Buddy Axum API.
//!
//! Each test spins up the router with a fresh in-memory SQLite database so
//! tests are fully isolated and require no external services.

use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use supplement_buddy_backend::{build_router, db::run_migrations_for_test};

async fn make_server() -> TestServer {
    // max_connections(1) ensures all queries share the same in-memory database
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory DB");
    run_migrations_for_test(&pool).await.expect("migrations");
    let app = build_router(pool, &[]);
    TestServer::new(app).expect("test server")
}

// ---------------------------------------------------------------------------
// Supplements — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_supplements_empty() {
    let server = make_server().await;

    let res = server.get("/api/supplements").await;

    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["count"], 0);
    assert!(body["results"].as_array().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Supplements — create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_supplement_success() {
    let server = make_server().await;

    let payload = json!({
        "name": "Vitamin C",
        "brand": "HealthPlus",
        "category": "Vitamins",
        "description": "Supports immune health",
        "ingredients": "Ascorbic Acid",
        "serving_size": "500mg"
    });

    let res = server.post("/api/supplements").json(&payload).await;

    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["name"], "Vitamin C");
    assert_eq!(body["brand"], "HealthPlus");
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_create_supplement_missing_name_returns_400() {
    let server = make_server().await;

    let payload = json!({ "brand": "HealthPlus" });

    let res = server.post("/api/supplements").json(&payload).await;

    res.assert_status_bad_request();
}

// ---------------------------------------------------------------------------
// Supplements — get by id
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_supplement_by_id() {
    let server = make_server().await;

    let create_res = server
        .post("/api/supplements")
        .json(&json!({ "name": "Fish Oil", "brand": "OmegaBest" }))
        .await;
    let created: Value = create_res.json();
    let id = created["id"].as_i64().unwrap();

    let get_res = server.get(&format!("/api/supplements/{id}")).await;

    get_res.assert_status_ok();
    let body: Value = get_res.json();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Fish Oil");
}

#[tokio::test]
async fn test_get_supplement_not_found() {
    let server = make_server().await;

    let res = server.get("/api/supplements/99999").await;

    res.assert_status_not_found();
}

// ---------------------------------------------------------------------------
// Supplements — list after inserts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_supplements_returns_all() {
    let server = make_server().await;

    for name in &["Magnesium", "Zinc", "Vitamin D"] {
        server
            .post("/api/supplements")
            .json(&json!({ "name": name }))
            .await;
    }

    let res = server.get("/api/supplements").await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["count"], 3);
    assert_eq!(body["results"].as_array().unwrap().len(), 3);
}

// ---------------------------------------------------------------------------
// Supplements — search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_search_returns_matching_results() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({
            "name": "Vitamin C",
            "category": "Vitamins",
            "description": "Antioxidant support"
        }))
        .await;
    server
        .post("/api/supplements")
        .json(&json!({
            "name": "Fish Oil",
            "description": "Omega-3 fatty acids"
        }))
        .await;

    let res = server.get("/api/supplements/search?q=vitamin").await;
    res.assert_status_ok();
    let results: Value = res.json();
    let arr = results.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Vitamin C");
}

#[tokio::test]
async fn test_search_missing_query_returns_400() {
    let server = make_server().await;

    let res = server.get("/api/supplements/search").await;
    res.assert_status_bad_request();
}

#[tokio::test]
async fn test_search_persists_query() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({ "name": "Magnesium", "description": "Muscle relaxation" }))
        .await;

    server
        .get("/api/supplements/search?q=magnesium")
        .await
        .assert_status_ok();

    let sq_res = server.get("/api/search-queries").await;
    sq_res.assert_status_ok();
    let body: Value = sq_res.json();
    assert_eq!(body["count"], 1);
    assert_eq!(body["results"][0]["query"], "magnesium");
}

// ---------------------------------------------------------------------------
// Search queries — list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_search_queries_empty() {
    let server = make_server().await;

    let res = server.get("/api/search-queries").await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["count"], 0);
}

// ---------------------------------------------------------------------------
// SPLADE vector update
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_update_splade_vector() {
    let server = make_server().await;

    let create_res = server
        .post("/api/supplements")
        .json(&json!({ "name": "Creatine" }))
        .await;
    let id = create_res.json::<Value>()["id"].as_i64().unwrap();

    let vec_payload = json!({ "##creatine": 2.5, "muscle": 1.8, "strength": 1.2 });
    let update_res = server
        .put(&format!("/api/supplements/{id}/splade-vector"))
        .json(&vec_payload)
        .await;

    update_res.assert_status_ok();
    let body: Value = update_res.json();
    assert!(body["splade_vector"].is_object());
}

// ---------------------------------------------------------------------------
// Additional edge-case tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_supplement_whitespace_name_returns_400() {
    let server = make_server().await;

    let res = server
        .post("/api/supplements")
        .json(&json!({ "name": "   " }))
        .await;

    res.assert_status_bad_request();
}

#[tokio::test]
async fn test_create_supplement_empty_optional_fields_default_to_empty_string() {
    let server = make_server().await;

    let res = server
        .post("/api/supplements")
        .json(&json!({ "name": "Biotin" }))
        .await;

    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert_eq!(body["brand"], "");
    assert_eq!(body["category"], "");
    assert_eq!(body["description"], "");
    assert_eq!(body["ingredients"], "");
    assert_eq!(body["serving_size"], "");
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({ "name": "Vitamin C", "category": "Vitamins" }))
        .await;

    let res = server.get("/api/supplements/search?q=VITAMIN").await;

    res.assert_status_ok();
    let arr: Value = res.json();
    assert_eq!(arr.as_array().unwrap().len(), 1);
    assert_eq!(arr[0]["name"], "Vitamin C");
}

#[tokio::test]
async fn test_search_by_brand() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({ "name": "Fish Oil", "brand": "OmegaBest" }))
        .await;
    server
        .post("/api/supplements")
        .json(&json!({ "name": "Vitamin C", "brand": "HealthPlus" }))
        .await;

    let res = server.get("/api/supplements/search?q=OmegaBest").await;

    res.assert_status_ok();
    let arr: Value = res.json();
    let names: Vec<&str> = arr
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Fish Oil"));
    assert!(!names.contains(&"Vitamin C"));
}

#[tokio::test]
async fn test_search_by_description() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({
            "name": "Fish Oil",
            "description": "Supports heart health with omega-3"
        }))
        .await;

    let res = server
        .get("/api/supplements/search?q=heart+health")
        .await;

    res.assert_status_ok();
    let arr: Value = res.json();
    assert_eq!(arr.as_array().unwrap().len(), 1);
    assert_eq!(arr[0]["name"], "Fish Oil");
}

#[tokio::test]
async fn test_search_by_ingredients() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({
            "name": "Vitamin C",
            "ingredients": "Ascorbic Acid"
        }))
        .await;

    let res = server.get("/api/supplements/search?q=Ascorbic").await;

    res.assert_status_ok();
    let arr: Value = res.json();
    assert_eq!(arr.as_array().unwrap().len(), 1);
    assert_eq!(arr[0]["name"], "Vitamin C");
}

#[tokio::test]
async fn test_search_no_results() {
    let server = make_server().await;

    server
        .post("/api/supplements")
        .json(&json!({ "name": "Magnesium" }))
        .await;

    let res = server
        .get("/api/supplements/search?q=xyznonexistentterm")
        .await;

    res.assert_status_ok();
    let arr: Value = res.json();
    assert!(arr.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_search_empty_string_returns_400() {
    let server = make_server().await;

    let res = server.get("/api/supplements/search?q=").await;

    res.assert_status_bad_request();
}

#[tokio::test]
async fn test_search_whitespace_only_returns_400() {
    let server = make_server().await;

    let res = server.get("/api/supplements/search?q=+++").await;

    res.assert_status_bad_request();
}

#[tokio::test]
async fn test_list_supplements_pagination() {
    let server = make_server().await;

    for name in &["Alpha", "Beta", "Gamma", "Delta", "Epsilon"] {
        server
            .post("/api/supplements")
            .json(&json!({ "name": name }))
            .await;
    }

    // First page of 2
    let res = server
        .get("/api/supplements?page=1&page_size=2")
        .await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["count"], 5);
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 2);
    assert_eq!(body["results"].as_array().unwrap().len(), 2);

    // Second page of 2
    let res2 = server
        .get("/api/supplements?page=2&page_size=2")
        .await;
    res2.assert_status_ok();
    let body2: Value = res2.json();
    assert_eq!(body2["count"], 5);
    assert_eq!(body2["page"], 2);
    assert_eq!(body2["results"].as_array().unwrap().len(), 2);

    // Third page has only 1
    let res3 = server
        .get("/api/supplements?page=3&page_size=2")
        .await;
    res3.assert_status_ok();
    let body3: Value = res3.json();
    assert_eq!(body3["results"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_list_search_queries_pagination() {
    let server = make_server().await;

    // Generate 5 search queries by running searches
    for term in &["alpha", "beta", "gamma", "delta", "epsilon"] {
        server
            .get(&format!("/api/supplements/search?q={term}"))
            .await;
    }

    let res = server.get("/api/search-queries?page=1&page_size=3").await;
    res.assert_status_ok();
    let body: Value = res.json();
    assert_eq!(body["count"], 5);
    assert_eq!(body["results"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_list_search_queries_ordered_most_recent_first() {
    let server = make_server().await;

    server.get("/api/supplements/search?q=first").await;
    server.get("/api/supplements/search?q=second").await;

    let res = server.get("/api/search-queries").await;
    res.assert_status_ok();
    let body: Value = res.json();
    // The most recently created query should appear first
    assert_eq!(body["results"][0]["query"], "second");
    assert_eq!(body["results"][1]["query"], "first");
}

#[tokio::test]
async fn test_update_splade_vector_reflected_in_get() {
    let server = make_server().await;

    let created: Value = server
        .post("/api/supplements")
        .json(&json!({ "name": "Creatine" }))
        .await
        .json();
    let id = created["id"].as_i64().unwrap();

    let vector = json!({ "creatine": 2.5, "muscle": 1.8 });
    server
        .put(&format!("/api/supplements/{id}/splade-vector"))
        .json(&vector)
        .await
        .assert_status_ok();

    let get_res = server.get(&format!("/api/supplements/{id}")).await;
    get_res.assert_status_ok();
    let body: Value = get_res.json();
    assert!(body["splade_vector"].is_object());
    assert_eq!(body["splade_vector"]["creatine"], 2.5);
    assert_eq!(body["splade_vector"]["muscle"], 1.8);
}

#[tokio::test]
async fn test_supplement_created_at_and_updated_at_are_set() {
    let server = make_server().await;

    let res = server
        .post("/api/supplements")
        .json(&json!({ "name": "Selenium" }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = res.json();
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
    // Both timestamps should be non-empty
    assert!(!body["created_at"].as_str().unwrap().is_empty());
    assert!(!body["updated_at"].as_str().unwrap().is_empty());
}
