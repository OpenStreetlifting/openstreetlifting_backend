use actix_web::{HttpResponse, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;
use storage::models::Competition;

use crate::middleware::auth::api_key_validator;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(api_key_validator);

    cfg.service(
        web::scope("/competitions")
            .route("", web::get().to(list_competitions))
            .route("/{id}", web::get().to(get_competition))
            .route("", web::post().to(create_competition).wrap(auth.clone()))
            .route(
                "/{id}",
                web::put().to(update_competition).wrap(auth.clone()),
            )
            .route("/{id}", web::delete().to(delete_competition).wrap(auth)),
    );
}

#[utoipa::path(
    get,
    path = "/api/competitions",
    responses(
        (status = 200, description = "List all competitions", body = Vec<Competition>)
    ),
    tag = "competitions"
)]
pub async fn list_competitions() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "competitions": [] }))
}

#[utoipa::path(
    get,
    path = "/api/competitions/{id}",
    params(
        ("id" = i32, Path, description = "Competition ID")
    ),
    responses(
        (status = 200, description = "Get competition by ID", body = Competition)
    ),
    tag = "competitions"
)]
pub async fn get_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "competition_id": id, "name": "Example Competition" }))
}

#[utoipa::path(
    post,
    path = "/api/competitions",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Competition created"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "competitions"
)]
pub async fn create_competition() -> HttpResponse {
    HttpResponse::Created().json(json!({ "message": "Competition created" }))
}

#[utoipa::path(
    put,
    path = "/api/competitions/{id}",
    params(
        ("id" = i32, Path, description = "Competition ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Competition updated"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "competitions"
)]
pub async fn update_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "message": format!("Competition {} updated", id) }))
}

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
        (status = 200, description = "Competition deleted"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "competitions"
)]
pub async fn delete_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "message": format!("Competition {} deleted", id) }))
}
