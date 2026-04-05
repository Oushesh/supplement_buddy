use axum::{
    extract::{Query, State},
    Json,
};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect};

use crate::{
    entities::search_query::{Column, Entity as SearchQueryEntity},
    error::AppError,
    models::{Page, PaginationParams, SearchQuery},
};

/// GET /api/search-queries/
pub async fn list_search_queries(
    State(db): State<DatabaseConnection>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Page<SearchQuery>>, AppError> {
    let page = params.page.max(1);
    let page_size = params.page_size.clamp(1, 100);
    let offset = ((page - 1) * page_size) as u64;

    let count = SearchQueryEntity::find().count(&db).await? as i64;

    let rows: Vec<SearchQuery> = SearchQueryEntity::find()
        .order_by_desc(Column::CreatedAt)
        .offset(offset)
        .limit(page_size as u64)
        .all(&db)
        .await?
        .into_iter()
        .map(SearchQuery::from)
        .collect();

    Ok(Json(Page {
        count,
        page,
        page_size,
        results: rows,
    }))
}
