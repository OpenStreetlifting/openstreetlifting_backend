use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::handlers::athletes::{
    create_athlete, delete_athlete, get_athlete, get_athlete_detailed, list_athletes,
    update_athlete,
};
use crate::middleware::auth::api_key_validator;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(api_key_validator);

    cfg.service(
        web::scope("/athletes")
            .route("", web::get().to(list_athletes))
            .route("/{slug}", web::get().to(get_athlete))
            .route("/{slug}/detailed", web::get().to(get_athlete_detailed))
            .route("", web::post().to(create_athlete).wrap(auth.clone()))
            .route("/{slug}", web::put().to(update_athlete).wrap(auth.clone()))
            .route("/{slug}", web::delete().to(delete_athlete).wrap(auth)),
    );
}
