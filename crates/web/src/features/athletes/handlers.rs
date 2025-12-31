use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use storage::{
    Database,
    dto::athlete::{
        AthleteDetailResponse, AthleteResponse, CreateAthleteRequest, UpdateAthleteRequest,
    },
};
use validator::Validate;

use crate::error::WebError;

use super::services;

#[utoipa::path(
    get,
    path = "/api/athletes",
    responses(
        (status = 200, description = "List all athletes successfully", body = Vec<AthleteResponse>)
    ),
    tag = "athletes"
)]
pub async fn list_athletes(State(db): State<Database>) -> Result<Response, WebError> {
    let athletes = services::list_athletes(db.pool()).await?;

    let response: Vec<AthleteResponse> = athletes.into_iter().map(AthleteResponse::from).collect();

    Ok(Json(response).into_response())
}

#[utoipa::path(
    get,
    path = "/api/athletes/{slug}",
    params(
        ("slug" = String, Path, description = "Athlete slug")
    ),
    responses(
        (status = 200, description = "Athlete found", body = AthleteResponse),
        (status = 404, description = "Athlete not found")
    ),
    tag = "athletes"
)]
pub async fn get_athlete(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    let athlete = services::get_athlete_by_slug(db.pool(), &slug).await?;

    Ok(Json(AthleteResponse::from(athlete)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/athletes/{slug}/detailed",
    params(
        ("slug" = String, Path, description = "Athlete slug")
    ),
    responses(
        (status = 200, description = "Athlete with full details including competition history", body = AthleteDetailResponse),
        (status = 404, description = "Athlete not found")
    ),
    tag = "athletes"
)]
pub async fn get_athlete_detailed(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    let athlete = services::get_athlete_detailed(db.pool(), &slug).await?;

    Ok(Json(athlete).into_response())
}

#[utoipa::path(
    post,
    path = "/api/athletes",
    request_body = CreateAthleteRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Athlete created successfully", body = AthleteResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "athletes"
)]
pub async fn create_athlete(
    State(db): State<Database>,
    Json(req): Json<CreateAthleteRequest>,
) -> Result<Response, WebError> {
    req.validate()?;

    let athlete = services::create_athlete(db.pool(), &req).await?;

    Ok((StatusCode::CREATED, Json(AthleteResponse::from(athlete))).into_response())
}

#[utoipa::path(
    put,
    path = "/api/athletes/{slug}",
    params(
        ("slug" = String, Path, description = "Athlete slug")
    ),
    request_body = UpdateAthleteRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Athlete updated successfully", body = AthleteResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Athlete not found")
    ),
    tag = "athletes"
)]
pub async fn update_athlete(
    State(db): State<Database>,
    Path(slug): Path<String>,
    Json(update_req): Json<UpdateAthleteRequest>,
) -> Result<Response, WebError> {
    update_req.validate()?;

    let updated = services::update_athlete(db.pool(), &slug, &update_req).await?;

    Ok(Json(AthleteResponse::from(updated)).into_response())
}

#[utoipa::path(
    delete,
    path = "/api/athletes/{slug}",
    params(
        ("slug" = String, Path, description = "Athlete slug")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 204, description = "Athlete deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Athlete not found")
    ),
    tag = "athletes"
)]
pub async fn delete_athlete(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Response, WebError> {
    services::delete_athlete(db.pool(), &slug).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
