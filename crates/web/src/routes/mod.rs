use actix_web::web;

mod competitions;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api").configure(competitions::configure));
}
