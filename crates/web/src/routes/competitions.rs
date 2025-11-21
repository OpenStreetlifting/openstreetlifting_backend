use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::handlers::competitions::{
    create_competition, delete_competition, get_competition, get_competition_detailed,
    list_competitions, list_competitions_detailed, update_competition,
};
use crate::middleware::auth::api_key_validator;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(api_key_validator);

    cfg.service(
        web::scope("/competitions")
            .route("", web::get().to(list_competitions))
            .route("/detailed", web::get().to(list_competitions_detailed))
            .route("/{slug}", web::get().to(get_competition))
            .route("/{slug}/detailed", web::get().to(get_competition_detailed))
            .route("", web::post().to(create_competition).wrap(auth.clone()))
            .route(
                "/{slug}",
                web::put().to(update_competition).wrap(auth.clone()),
            )
            .route("/{slug}", web::delete().to(delete_competition).wrap(auth)),
    );
}
