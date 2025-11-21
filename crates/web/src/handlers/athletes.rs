use actix_web::{HttpResponse, web};
use storage::{
    Database,
    dto::athlete::{
        AthleteDetailResponse, AthleteResponse, CreateAthleteRequest, UpdateAthleteRequest,
    },
    repository::athlete::AthleteRepository,
};
use validator::Validate;

use crate::error::WebResult;

#[utoipa::path(
    get,
    path = "/api/athletes",
    responses(
        (status = 200, description = "List all athletes successfully", body = Vec<AthleteResponse>)
    ),
    tag = "athletes"
)]
pub async fn list_athletes(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let repo = AthleteRepository::new(db.pool());
    let athletes = repo.list().await?;

    let response: Vec<AthleteResponse> = athletes.into_iter().map(AthleteResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
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
    db: web::Data<Database>,
    path: web::Path<String>,
) -> WebResult<HttpResponse> {
    let slug = path.into_inner();
    let repo = AthleteRepository::new(db.pool());
    let athlete = repo.find_by_slug(&slug).await?;

    Ok(HttpResponse::Ok().json(AthleteResponse::from(athlete)))
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
    db: web::Data<Database>,
    path: web::Path<String>,
) -> WebResult<HttpResponse> {
    let slug = path.into_inner();
    let repo = AthleteRepository::new(db.pool());
    let athlete = repo.find_by_slug_detailed(&slug).await?;

    Ok(HttpResponse::Ok().json(athlete))
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
    db: web::Data<Database>,
    payload: web::Json<CreateAthleteRequest>,
) -> WebResult<HttpResponse> {
    let req = payload.into_inner();

    req.validate()?;

    let repo = AthleteRepository::new(db.pool());
    let athlete = repo.create(&req).await?;

    Ok(HttpResponse::Created().json(AthleteResponse::from(athlete)))
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
    db: web::Data<Database>,
    path: web::Path<String>,
    payload: web::Json<UpdateAthleteRequest>,
) -> WebResult<HttpResponse> {
    let slug = path.into_inner();
    let update_req = payload.into_inner();

    update_req.validate()?;

    let repo = AthleteRepository::new(db.pool());

    let existing = repo.find_by_slug(&slug).await?;

    let updated = repo
        .update(existing.athlete_id, &existing, &update_req)
        .await?;

    Ok(HttpResponse::Ok().json(AthleteResponse::from(updated)))
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
    db: web::Data<Database>,
    path: web::Path<String>,
) -> WebResult<HttpResponse> {
    let slug = path.into_inner();
    let repo = AthleteRepository::new(db.pool());
    let athlete = repo.find_by_slug(&slug).await?;
    repo.delete(athlete.athlete_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
