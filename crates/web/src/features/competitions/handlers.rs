use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use storage::{
    Database,
    dto::competition::{
        CompetitionDetailResponse, CompetitionListResponse, CompetitionResponse,
        CreateCompetitionRequest, UpdateCompetitionRequest,
    },
};
use validator::Validate;

use crate::error::WebError;

use super::services;

#[utoipa::path(
    get,
    path = "/api/competitions",
    responses(
        (status = 200, description = "List all competitions successfully", body = Vec<CompetitionResponse>)
    ),
    tag = "competitions"
)]
pub async fn list_competitions(
    State(db): State<Database>,
) -> Result<Json<Vec<CompetitionResponse>>, WebError> {
    let competitions = services::list_competitions(db.pool()).await?;

    let response: Vec<CompetitionResponse> = competitions
        .into_iter()
        .map(CompetitionResponse::from)
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/competitions/detailed",
    responses(
        (status = 200, description = "List all competitions with detailed information (federation and movements)", body = Vec<CompetitionListResponse>)
    ),
    tag = "competitions"
)]
pub async fn list_competitions_detailed(State(db): State<Database>) -> Result<Response, WebError> {
    let competitions = services::list_competitions_detailed(db.pool()).await?;

    Ok(Json(competitions).into_response())
}

#[utoipa::path(
    get,
    path = "/api/competitions/{slug}",
    params(
        ("slug" = String, Path, description = "Competition slug")
    ),
    responses(
        (status = 200, description = "Competition found", body = CompetitionResponse),
        (status = 404, description = "Competition not found")
    ),
    tag = "competitions"
)]
pub async fn get_competition(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    let competition = services::get_competition_by_slug(db.pool(), &slug).await?;

    Ok(Json(CompetitionResponse::from(competition)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/competitions/{slug}/detailed",
    params(
        ("slug" = String, Path, description = "Competition slug")
    ),
    responses(
        (status = 200, description = "Competition with full details including category-merged participants and computed rankings", body = CompetitionDetailResponse),
        (status = 404, description = "Competition not found")
    ),
    tag = "competitions"
)]
pub async fn get_competition_detailed(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    let competition = services::get_competition_detailed(db.pool(), &slug).await?;

    Ok(Json(competition).into_response())
}

#[utoipa::path(
    post,
    path = "/api/competitions",
    request_body = CreateCompetitionRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Competition created successfully", body = CompetitionResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Slug already exists")
    ),
    tag = "competitions"
)]
pub async fn create_competition(
    State(db): State<Database>,
    Json(req): Json<CreateCompetitionRequest>,
) -> Result<Response, WebError> {
    req.validate()?;

    req.validate_dates()
        .map_err(|e| WebError::BadRequest(e.to_string()))?;

    let competition = services::create_competition(db.pool(), &req).await?;

    Ok((
        StatusCode::CREATED,
        Json(CompetitionResponse::from(competition)),
    )
        .into_response())
}

#[utoipa::path(
    put,
    path = "/api/competitions/{slug}",
    params(
        ("slug" = String, Path, description = "Competition slug")
    ),
    request_body = UpdateCompetitionRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Competition updated successfully", body = CompetitionResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Competition not found"),
        (status = 409, description = "Slug already exists")
    ),
    tag = "competitions"
)]
pub async fn update_competition(
    State(db): State<Database>,
    Path(slug): Path<String>,
    Json(update_req): Json<UpdateCompetitionRequest>,
) -> Result<Response, WebError> {
    update_req.validate()?;

    let updated = services::update_competition(db.pool(), &slug, &update_req).await?;

    Ok(Json(CompetitionResponse::from(updated)).into_response())
}

#[utoipa::path(
    delete,
    path = "/api/competitions/{slug}",
    params(
        ("slug" = String, Path, description = "Competition slug")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 204, description = "Competition deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Competition not found")
    ),
    tag = "competitions"
)]
pub async fn delete_competition(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    services::delete_competition(db.pool(), &slug).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
