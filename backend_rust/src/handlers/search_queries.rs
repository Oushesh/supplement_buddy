use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    models::{Page, PaginationParams, SearchQuery},
};

/// GET /api/search-queries/
pub async fn list_search_queries(
    State(pool): State<SqlitePool>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Page<SearchQuery>>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_queries")
        .fetch_one(&pool)
        .await?;

    let rows = sqlx::query_as::<_, SearchQuery>(
        r#"
        SELECT
            id,
            query,
            results,
            created_at
        FROM search_queries
        ORDER BY created_at DESC
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
