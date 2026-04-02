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

#[tokio::test]
async fn test_update_splade_vector_not_found() {
    let server = make_server().await;

    let res = server
        .put("/api/supplements/99999/splade-vector")
        .json(&json!({ "token": 1.0 }))
        .await;

    res.assert_status_not_found();
}
