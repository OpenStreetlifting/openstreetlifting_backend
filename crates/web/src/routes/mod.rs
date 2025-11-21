use actix_web::web;

pub mod athletes;
pub mod competitions;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(competitions::configure)
            .configure(athletes::configure),
    );
}
