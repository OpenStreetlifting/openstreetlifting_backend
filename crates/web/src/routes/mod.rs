use actix_web::web;

pub mod competitions;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api").configure(competitions::configure));
}
