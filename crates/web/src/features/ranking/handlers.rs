use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use storage::{
    Database,
    dto::{
        common::PaginatedResponse,
        ranking::{GlobalRankingEntry, GlobalRankingFilter},
    },
};

use crate::error::WebError;

use super::services;

#[utoipa::path(
    get,
    path = "/api/rankings/global",
    params(GlobalRankingFilter),
    responses(
        (status = 200, description = "Global ranking retrieved successfully", body = PaginatedResponse<GlobalRankingEntry>),
        (status = 400, description = "Invalid query parameters")
    ),
    tag = "rankings"
)]
pub async fn get_global_ranking(
    State(db): State<Database>,
    Query(filter): Query<GlobalRankingFilter>,
) -> Result<Response, WebError> {
    filter.validate().map_err(WebError::BadRequest)?;

    let (entries, total_items) = services::get_global_ranking(db.pool(), &filter).await?;

    let response = PaginatedResponse::new(
        entries,
        filter.pagination.page,
        filter.pagination.page_size,
        total_items,
    );

    Ok(Json(response).into_response())
}
