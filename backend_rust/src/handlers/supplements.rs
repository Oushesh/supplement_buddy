use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{CreateSupplement, Page, PaginationParams, Supplement},
};

// ---------------------------------------------------------------------------
// GET /api/supplements/
// ---------------------------------------------------------------------------

pub async fn list_supplements(
    State(pool): State<SqlitePool>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Page<Supplement>>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supplements")
        .fetch_one(&pool)
        .await?;

    let rows = sqlx::query_as::<_, Supplement>(
        r#"
        SELECT
            id, name, brand, category, description, ingredients, serving_size,
            splade_vector,
            created_at,
            updated_at
        FROM supplements
        ORDER BY name
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok(Json(Page {
        count,
        page,
        page_size,
        results: rows,
    }))
}

// ---------------------------------------------------------------------------
// POST /api/supplements/
// ---------------------------------------------------------------------------

pub async fn create_supplement(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateSupplement>,
) -> Result<(StatusCode, Json<Supplement>), AppError> {
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| AppError::BadRequest("'name' is required".to_string()))?;

    let now = Utc::now();

    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO supplements (name, brand, category, description, ingredients, serving_size, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&name)
    .bind(&body.brand)
    .bind(&body.category)
    .bind(&body.description)
    .bind(&body.ingredients)
    .bind(&body.serving_size)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .fetch_one(&pool)
    .await?;

    let supplement = fetch_supplement_by_id(&pool, id).await?;
    Ok((StatusCode::CREATED, Json(supplement)))
}

// ---------------------------------------------------------------------------
// GET /api/supplements/:id
// ---------------------------------------------------------------------------

pub async fn get_supplement(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Supplement>, AppError> {
    let supplement = fetch_supplement_by_id(&pool, id).await?;
    Ok(Json(supplement))
}

// ---------------------------------------------------------------------------
// GET /api/supplements/search?q=<query>
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub async fn search_supplements(
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Supplement>>, AppError> {
    let query = params
        .q
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Query parameter 'q' is required".to_string()))?;

    // Text-search fallback (SPLADE inference lives in a separate Python service)
    let pattern = format!("%{}%", query);
    let results = sqlx::query_as::<_, Supplement>(
        r#"
        SELECT
            id, name, brand, category, description, ingredients, serving_size,
            splade_vector,
            created_at,
            updated_at
        FROM supplements
        WHERE  name        LIKE ? COLLATE NOCASE
            OR brand       LIKE ? COLLATE NOCASE
            OR description LIKE ? COLLATE NOCASE
            OR ingredients LIKE ? COLLATE NOCASE
        ORDER BY name
        "#,
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&pool)
    .await?;

    // Persist query for analytics
    let result_ids: Vec<i64> = results.iter().map(|s| s.id).collect();
    let results_json = serde_json::to_string(&result_ids).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO search_queries (query, results, created_at) VALUES (?, ?, ?)",
    )
    .bind(&query)
    .bind(&results_json)
    .bind(now.to_rfc3339())
    .execute(&pool)
    .await?;

    Ok(Json(results))
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn fetch_supplement_by_id(pool: &SqlitePool, id: i64) -> Result<Supplement, AppError> {
    sqlx::query_as::<_, Supplement>(
        r#"
        SELECT
            id, name, brand, category, description, ingredients, serving_size,
            splade_vector,
            created_at,
            updated_at
        FROM supplements
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

// ---------------------------------------------------------------------------
// splade_vector update — called by the indexer script
// PUT /api/supplements/:id/splade-vector
// ---------------------------------------------------------------------------

pub async fn update_splade_vector(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Supplement>, AppError> {
    let now = Utc::now();
    let vec_json = serde_json::to_string(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {e}")))?;

    let rows_affected = sqlx::query(
        "UPDATE supplements SET splade_vector = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&vec_json)
    .bind(now.to_rfc3339())
    .bind(id)
    .execute(&pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    let supplement = fetch_supplement_by_id(&pool, id).await?;
    Ok(Json(supplement))
}
