use actix_web::{HttpResponse, web};
use storage::{
    Database,
    dto::{
        common::PaginatedResponse,
        ranking::{GlobalRankingEntry, GlobalRankingFilter},
    },
    repository::ranking::RankingRepository,
};

use crate::error::{WebError, WebResult};

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
    db: web::Data<Database>,
    query: web::Query<GlobalRankingFilter>,
) -> WebResult<HttpResponse> {
    let filter = query.into_inner();

    filter.validate().map_err(WebError::BadRequest)?;

    let repo = RankingRepository::new(db.pool());
    let (entries, total_items) = repo.get_global_ranking(&filter).await?;

    let response = PaginatedResponse::new(
        entries,
        filter.pagination.page,
        filter.pagination.page_size,
        total_items,
    );

    Ok(HttpResponse::Ok().json(response))
}
