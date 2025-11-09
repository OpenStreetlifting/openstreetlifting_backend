use actix_web::{web, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;

use crate::middleware::auth::api_key_validator;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/competitions")
            .route("", web::get().to(list_competitions))
            .route("/{id}", web::get().to(get_competition))
            .service(
                web::scope("/admin")
                    .wrap(HttpAuthentication::bearer(api_key_validator))
                    .route("", web::post().to(create_competition))
                    .route("/{id}", web::put().to(update_competition))
                    .route("/{id}", web::delete().to(delete_competition)),
            ),
    );
}

async fn list_competitions() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "competitions": [] }))
}

async fn get_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "id": id, "name": "Example Competition" }))
}

async fn create_competition() -> HttpResponse {
    HttpResponse::Created().json(json!({ "message": "Competition created" }))
}

async fn update_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "message": format!("Competition {} updated", id) }))
}

async fn delete_competition(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();
    HttpResponse::Ok().json(json!({ "message": format!("Competition {} deleted", id) }))
}
