use actix_web::{web, HttpResponse};
use storage::{
    dto::competition::{CompetitionResponse, CreateCompetitionRequest, UpdateCompetitionRequest},
    repository::competition::CompetitionRepository,
    Database,
};
use validator::Validate;

use crate::error::{WebError, WebResult};

/// List all competitions
#[utoipa::path(
    get,
    path = "/api/competitions",
    responses(
        (status = 200, description = "List all competitions successfully", body = Vec<CompetitionResponse>)
    ),
    tag = "competitions"
)]
pub async fn list_competitions(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let repo = CompetitionRepository::new(db.pool());
    let competitions = repo.list().await?;

    let response: Vec<CompetitionResponse> = competitions
        .into_iter()
        .map(CompetitionResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

/// Get a competition by ID
#[utoipa::path(
    get,
    path = "/api/competitions/{id}",
    params(
        ("id" = i32, Path, description = "Competition ID")
    ),
    responses(
        (status = 200, description = "Competition found", body = CompetitionResponse),
        (status = 404, description = "Competition not found")
    ),
    tag = "competitions"
)]
pub async fn get_competition(
    db: web::Data<Database>,
    path: web::Path<i32>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let repo = CompetitionRepository::new(db.pool());
    let competition = repo.find_by_id(id).await?;

    Ok(HttpResponse::Ok().json(CompetitionResponse::from(competition)))
}

/// Create a new competition
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
    db: web::Data<Database>,
    payload: web::Json<CreateCompetitionRequest>,
) -> WebResult<HttpResponse> {
    let req = payload.into_inner();

    // Validate using validator crate
    req.validate()?;

    // Additional cross-field validation
    req.validate_dates()
        .map_err(|e| WebError::BadRequest(e.to_string()))?;

    let repo = CompetitionRepository::new(db.pool());
    let competition = repo.create(&req).await?;

    Ok(HttpResponse::Created().json(CompetitionResponse::from(competition)))
}

/// Update an existing competition
#[utoipa::path(
    put,
    path = "/api/competitions/{id}",
    params(
        ("id" = i32, Path, description = "Competition ID")
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
    db: web::Data<Database>,
    path: web::Path<i32>,
    payload: web::Json<UpdateCompetitionRequest>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let update_req = payload.into_inner();

    // Validate using validator crate
    update_req.validate()?;

    let repo = CompetitionRepository::new(db.pool());

    // Fetch existing competition
    let existing = repo.find_by_id(id).await?;

    // Merge update fields with existing data (following Arcadia pattern - update all fields)
    let updated = repo
        .update(
            id,
            update_req.name.unwrap_or(existing.name),
            update_req.slug.unwrap_or(existing.slug),
            update_req.status.unwrap_or(existing.status),
            update_req.federation_id.unwrap_or(existing.federation_id),
            update_req.venue.or(existing.venue),
            update_req.city.or(existing.city),
            update_req.country.or(existing.country),
            update_req.start_date.unwrap_or(existing.start_date),
            update_req.end_date.unwrap_or(existing.end_date),
            update_req.number_of_judge.or(existing.number_of_judge),
        )
        .await?;

    Ok(HttpResponse::Ok().json(CompetitionResponse::from(updated)))
}

/// Delete a competition
#[utoipa::path(
    delete,
    path = "/api/competitions/{id}",
    params(
        ("id" = i32, Path, description = "Competition ID")
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
    db: web::Data<Database>,
    path: web::Path<i32>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let repo = CompetitionRepository::new(db.pool());
    repo.delete(id).await?;

    Ok(HttpResponse::NoContent().finish())
}
