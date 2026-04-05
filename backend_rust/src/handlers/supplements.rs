use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use sea_orm::Condition;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    entities::supplement::{self, Column, Entity as SupplementEntity},
    entities::search_query,
    error::AppError,
    models::{CreateSupplement, Page, PaginationParams, Supplement},
};

// ---------------------------------------------------------------------------
// GET /api/supplements/
// ---------------------------------------------------------------------------

pub async fn list_supplements(
    State(db): State<DatabaseConnection>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Page<Supplement>>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = ((page - 1) * page_size) as u64;

    let count = SupplementEntity::find().count(&db).await? as i64;

    let rows: Vec<Supplement> = SupplementEntity::find()
        .order_by_asc(Column::Name)
        .offset(offset)
        .limit(page_size as u64)
        .all(&db)
        .await?
        .into_iter()
        .map(Supplement::from)
        .collect();

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
    State(db): State<DatabaseConnection>,
    Json(body): Json<CreateSupplement>,
) -> Result<(StatusCode, Json<Supplement>), AppError> {
    let name = body
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| AppError::BadRequest("'name' is required".to_string()))?;

    let now = Utc::now().to_rfc3339();

    let new_supplement = supplement::ActiveModel {
        name: Set(name),
        brand: Set(body.brand),
        category: Set(body.category),
        description: Set(body.description),
        ingredients: Set(body.ingredients),
        serving_size: Set(body.serving_size),
        splade_vector: Set(None),
        created_at: Set(now.clone()),
        updated_at: Set(now),
        ..Default::default()
    };

    let inserted = new_supplement.insert(&db).await?;
    Ok((StatusCode::CREATED, Json(Supplement::from(inserted))))
}

// ---------------------------------------------------------------------------
// GET /api/supplements/:id
// ---------------------------------------------------------------------------

pub async fn get_supplement(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i64>,
) -> Result<Json<Supplement>, AppError> {
    let model = fetch_supplement_by_id(&db, id).await?;
    Ok(Json(Supplement::from(model)))
}

// ---------------------------------------------------------------------------
// GET /api/supplements/search?q=<query>
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub async fn search_supplements(
    State(db): State<DatabaseConnection>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Supplement>>, AppError> {
    let query = params
        .q
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Query parameter 'q' is required".to_string()))?;

    let pattern = format!("%{}%", query);

    let results: Vec<Supplement> = SupplementEntity::find()
        .filter(
            Condition::any()
                .add(Column::Name.like(&pattern))
                .add(Column::Brand.like(&pattern))
                .add(Column::Description.like(&pattern))
                .add(Column::Ingredients.like(&pattern)),
        )
        .order_by_asc(Column::Name)
        .all(&db)
        .await?
        .into_iter()
        .map(Supplement::from)
        .collect();

    // Persist query for analytics
    let result_ids: Vec<i64> = results.iter().map(|s| s.id).collect();
    let results_json = serde_json::to_string(&result_ids).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();

    let new_query = search_query::ActiveModel {
        query: Set(query),
        results: Set(results_json),
        created_at: Set(now),
        ..Default::default()
    };
    new_query.insert(&db).await?;

    Ok(Json(results))
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn fetch_supplement_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> Result<supplement::Model, AppError> {
    SupplementEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}

// ---------------------------------------------------------------------------
// PUT /api/supplements/:id/splade-vector
// ---------------------------------------------------------------------------

pub async fn update_splade_vector(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Supplement>, AppError> {
    let model = fetch_supplement_by_id(&db, id).await?;
    let now = Utc::now().to_rfc3339();

    let vec_json = serde_json::to_string(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {e}")))?;

    let mut active = model.into_active_model();
    active.splade_vector = Set(Some(vec_json));
    active.updated_at = Set(now);

    let updated = active.update(&db).await?;
    Ok(Json(Supplement::from(updated)))
}
