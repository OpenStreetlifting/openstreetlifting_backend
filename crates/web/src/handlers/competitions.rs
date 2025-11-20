use actix_web::{HttpResponse, web};
use storage::{
    Database,
    dto::competition::{
        CompetitionListResponse, CompetitionResponse, CreateCompetitionRequest,
        UpdateCompetitionRequest,
    },
    repository::competition::CompetitionRepository,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{WebError, WebResult};

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

#[utoipa::path(
    get,
    path = "/api/competitions/detailed",
    responses(
        (status = 200, description = "List all competitions with detailed information (federation and movements)", body = Vec<CompetitionListResponse>)
    ),
    tag = "competitions"
)]
pub async fn list_competitions_detailed(db: web::Data<Database>) -> WebResult<HttpResponse> {
    let repo = CompetitionRepository::new(db.pool());
    let competitions = repo.list_with_details().await?;

    Ok(HttpResponse::Ok().json(competitions))
}

#[utoipa::path(
    get,
    path = "/api/competitions/{id}",
    params(
        ("id" = Uuid, Path, description = "Competition ID")
    ),
    responses(
        (status = 200, description = "Competition found", body = CompetitionResponse),
        (status = 404, description = "Competition not found")
    ),
    tag = "competitions"
)]
pub async fn get_competition(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let repo = CompetitionRepository::new(db.pool());
    let competition = repo.find_by_id(id).await?;

    Ok(HttpResponse::Ok().json(CompetitionResponse::from(competition)))
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
    db: web::Data<Database>,
    payload: web::Json<CreateCompetitionRequest>,
) -> WebResult<HttpResponse> {
    let req = payload.into_inner();

    req.validate()?;

    req.validate_dates()
        .map_err(|e| WebError::BadRequest(e.to_string()))?;

    let repo = CompetitionRepository::new(db.pool());
    let competition = repo.create(&req).await?;

    Ok(HttpResponse::Created().json(CompetitionResponse::from(competition)))
}

#[utoipa::path(
    put,
    path = "/api/competitions/{id}",
    params(
        ("id" = Uuid, Path, description = "Competition ID")
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
    path: web::Path<Uuid>,
    payload: web::Json<UpdateCompetitionRequest>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let update_req = payload.into_inner();

    update_req.validate()?;

    let repo = CompetitionRepository::new(db.pool());

    let existing = repo.find_by_id(id).await?;

    let updated = repo.update(id, &existing, &update_req).await?;

    Ok(HttpResponse::Ok().json(CompetitionResponse::from(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/competitions/{id}",
    params(
        ("id" = Uuid, Path, description = "Competition ID")
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
    path: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let id = path.into_inner();
    let repo = CompetitionRepository::new(db.pool());
    repo.delete(id).await?;

    Ok(HttpResponse::NoContent().finish())
}
